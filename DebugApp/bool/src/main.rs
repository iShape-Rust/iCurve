mod examples;

use crate::examples::{BoolExample, CurvePoint, load_examples};
use debug_ui::{
    camera::Camera,
    egui::{
        self, Color32, CursorIcon, Id, Painter, Pos2, Rect, Sense, Shape, Stroke, Ui, Vec2,
        epaint::PathShape,
    },
    grid::{Grid, paint_camera_readout},
};
use i_curve::{
    CurveBuildError as CurveError, CurveBuilder, FillRule, FloatCurveOverlay, FloatCurvePath,
    FloatCurveSegment, FloatCurveShape, OverlayRule, float::arc::RationalArc,
};
use std::fmt::Write;

const OVERLAY_RULES: [OverlayRule; 5] = [
    OverlayRule::Intersect,
    OverlayRule::Union,
    OverlayRule::Difference,
    OverlayRule::InverseDifference,
    OverlayRule::Xor,
];

const FILL_RULES: [FillRule; 4] = [
    FillRule::EvenOdd,
    FillRule::NonZero,
    FillRule::Positive,
    FillRule::Negative,
];

#[derive(Clone, Copy, PartialEq, Eq)]
enum ShowMode {
    Inputs,
    Result,
    Both,
}

impl ShowMode {
    fn label(self) -> &'static str {
        match self {
            Self::Inputs => "initial curves",
            Self::Result => "result curves",
            Self::Both => "all curves",
        }
    }
}

struct OverlayResult {
    shapes: Vec<FloatCurveShape<CurvePoint>>,
}

struct BoolApp {
    camera: Camera,
    grid: Grid,
    examples: Vec<BoolExample>,
    active_example: usize,
    overlay_rule: OverlayRule,
    fill_rule: FillRule,
    show_mode: ShowMode,
    result: Result<OverlayResult, String>,
}

impl Default for BoolApp {
    fn default() -> Self {
        let examples = load_examples();
        let mut app = Self {
            camera: Camera {
                zoom: 1.15,
                ..Camera::default()
            },
            grid: Grid::default(),
            examples,
            active_example: 0,
            overlay_rule: OverlayRule::Union,
            fill_rule: FillRule::NonZero,
            show_mode: ShowMode::Both,
            result: Err("not calculated".to_owned()),
        };
        app.refresh_result();
        app.fit_active_example();
        app
    }
}

impl eframe::App for BoolApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        egui::Panel::left("bool_panel")
            .resizable(false)
            .default_size(250.0)
            .frame(egui::Frame::default().fill(Color32::from_rgb(24, 27, 32)))
            .show_inside(ui, |ui| {
                ui.spacing_mut().item_spacing = Vec2::new(6.0, 6.0);
                ui.add_space(8.0);

                let mut selected = None;
                ui.label("Test");
                for (index, example) in self.examples.iter().enumerate() {
                    if ui
                        .selectable_label(index == self.active_example, example.name)
                        .clicked()
                    {
                        selected = Some(index);
                    }
                }

                if let Some(index) = selected {
                    self.active_example = index;
                    self.refresh_result();
                    self.fit_active_example();
                }

                ui.add_space(8.0);
                ui.separator();

                let previous_rule = self.overlay_rule;
                egui::ComboBox::from_label("Operation")
                    .selected_text(self.overlay_rule.to_string())
                    .show_ui(ui, |ui| {
                        for rule in OVERLAY_RULES {
                            ui.selectable_value(&mut self.overlay_rule, rule, rule.to_string());
                        }
                    });

                let previous_fill_rule = self.fill_rule;
                egui::ComboBox::from_label("Fill rule")
                    .selected_text(self.fill_rule.to_string())
                    .show_ui(ui, |ui| {
                        for rule in FILL_RULES {
                            ui.selectable_value(&mut self.fill_rule, rule, rule.to_string());
                        }
                    });

                if previous_rule != self.overlay_rule || previous_fill_rule != self.fill_rule {
                    self.refresh_result();
                }

                egui::ComboBox::from_label("Show")
                    .selected_text(self.show_mode.label())
                    .show_ui(ui, |ui| {
                        ui.selectable_value(
                            &mut self.show_mode,
                            ShowMode::Inputs,
                            ShowMode::Inputs.label(),
                        );
                        ui.selectable_value(
                            &mut self.show_mode,
                            ShowMode::Result,
                            ShowMode::Result.label(),
                        );
                        ui.selectable_value(
                            &mut self.show_mode,
                            ShowMode::Both,
                            ShowMode::Both.label(),
                        );
                    });

                ui.add_space(8.0);
                ui.separator();

                let active = self.active_example();
                ui.label(format!("Subject shapes: {}", active.subject.len()));
                ui.label(format!("Clip shapes: {}", active.clip.len()));
                match &self.result {
                    Ok(result) => {
                        let contour_count = result
                            .shapes
                            .iter()
                            .map(|shape| shape.contours().len())
                            .sum::<usize>();
                        ui.label(format!("Result shapes: {}", result.shapes.len()));
                        ui.label(format!("Result contours: {contour_count}"));
                    }
                    Err(error) => {
                        ui.colored_label(Color32::from_rgb(240, 118, 118), error);
                    }
                }

                if ui.button("Fit view").clicked() {
                    self.fit_active_example();
                }
            });

        egui::CentralPanel::default()
            .frame(egui::Frame::default().fill(Color32::from_rgb(18, 20, 24)))
            .show_inside(ui, |ui| {
                let available_size = ui.available_size();
                let (response, painter) =
                    ui.allocate_painter(available_size, Sense::click_and_drag());
                let rect = response.rect;

                self.grid
                    .handle_input(ui, &response, rect, &mut self.camera);
                self.grid.paint(&painter, rect, &self.camera);

                let camera = self.camera;
                let mut inputs_changed = false;

                if matches!(self.show_mode, ShowMode::Inputs | ShowMode::Both) {
                    let active = self.active_example_mut();
                    inputs_changed |= edit_float_shapes(
                        ui,
                        &painter,
                        rect,
                        &camera,
                        "subject",
                        &mut active.subject,
                        ShapeStyle {
                            fill: Color32::from_rgba_unmultiplied(73, 170, 255, 18),
                            stroke: Stroke::new(
                                2.0_f32,
                                Color32::from_rgba_unmultiplied(86, 196, 255, 150),
                            ),
                            controls: ControlStyle::input(),
                        },
                    );
                    inputs_changed |= edit_float_shapes(
                        ui,
                        &painter,
                        rect,
                        &camera,
                        "clip",
                        &mut active.clip,
                        ShapeStyle {
                            fill: Color32::from_rgba_unmultiplied(255, 107, 107, 18),
                            stroke: Stroke::new(
                                2.0_f32,
                                Color32::from_rgba_unmultiplied(240, 118, 118, 150),
                            ),
                            controls: ControlStyle::input(),
                        },
                    );
                }

                if inputs_changed {
                    self.refresh_result();
                }

                if matches!(self.show_mode, ShowMode::Result | ShowMode::Both)
                    && let Ok(result) = &self.result
                {
                    paint_float_shapes(
                        &painter,
                        rect,
                        &camera,
                        &result.shapes,
                        ShapeStyle {
                            fill: Color32::TRANSPARENT,
                            stroke: Stroke::new(3.0_f32, Color32::from_rgb(78, 180, 91)),
                            controls: ControlStyle::result(),
                        },
                    );
                }

                paint_camera_readout(&painter, rect, &self.camera);
            });
    }
}

impl BoolApp {
    fn active_example(&self) -> &BoolExample {
        &self.examples[self.active_example]
    }

    fn active_example_mut(&mut self) -> &mut BoolExample {
        &mut self.examples[self.active_example]
    }

    fn refresh_result(&mut self) {
        let example = self.active_example().clone();
        let overlay_rule = self.overlay_rule;
        let fill_rule = self.fill_rule;
        print_overlay_input(&example, overlay_rule, fill_rule);

        self.result = match std::panic::catch_unwind(move || {
            build_overlay_result(&example, overlay_rule, fill_rule)
        }) {
            Ok(result) => result,
            Err(payload) => Err(panic_message(payload)),
        };
    }

    fn fit_active_example(&mut self) {
        let Some(bounds) = example_bounds(self.active_example()) else {
            return;
        };

        self.camera.center = Pos2::new(
            (bounds.min_x + bounds.max_x) * 0.5,
            (bounds.min_y + bounds.max_y) * 0.5,
        );
    }
}

fn build_overlay_result(
    example: &BoolExample,
    overlay_rule: OverlayRule,
    fill_rule: FillRule,
) -> Result<OverlayResult, String> {
    let overlay = FloatCurveOverlay::with_subj_and_clip(&example.subject, &example.clip);

    Ok(OverlayResult {
        shapes: overlay.overlay(overlay_rule, fill_rule),
    })
}

fn print_overlay_input(example: &BoolExample, overlay_rule: OverlayRule, fill_rule: FillRule) {
    println!("{}", overlay_input_source(example, overlay_rule, fill_rule));
}

fn overlay_input_source(
    example: &BoolExample,
    overlay_rule: OverlayRule,
    fill_rule: FillRule,
) -> String {
    let mut source = String::new();

    writeln!(source, "\n// FloatCurveOverlay input: {}", example.name).unwrap();
    writeln!(source, "#[test]").unwrap();
    writeln!(source, "fn reproduced_overlay_case() {{").unwrap();
    writeln!(
        source,
        "    use i_curve::{{CurveBuilder, FillRule, FloatCurveOverlay, OverlayRule}};"
    )
    .unwrap();
    writeln!(
        source,
        "    use i_curve::float::arc::{{Ellipse, EllipticArc}};\n"
    )
    .unwrap();

    writeln!(source, "    let subject = CurveBuilder::new()").unwrap();
    write_float_shape_steps(&mut source, &example.subject);
    writeln!(source, "        .build().unwrap();\n").unwrap();

    writeln!(source, "    let clip = CurveBuilder::new()").unwrap();
    write_float_shape_steps(&mut source, &example.clip);
    writeln!(source, "        .build().unwrap();\n").unwrap();

    writeln!(
        source,
        "    let result = FloatCurveOverlay::with_subj_and_clip(&subject, &clip)"
    )
    .unwrap();
    writeln!(
        source,
        "        .overlay(OverlayRule::{overlay_rule}, FillRule::{fill_rule});"
    )
    .unwrap();
    writeln!(source, "    dbg!(result);").unwrap();
    writeln!(source, "}}").unwrap();

    source
}

fn write_float_shape_steps(source: &mut String, shapes: &[FloatCurveShape<CurvePoint>]) {
    for shape in shapes {
        for contour in shape.contours() {
            writeln!(
                source,
                "        .move_to([{:?}, {:?}]).unwrap()",
                contour.start()[0],
                contour.start()[1]
            )
            .unwrap();
            for segment in contour.segments() {
                match segment {
                    FloatCurveSegment::Line { to } => {
                        writeln!(
                            source,
                            "        .line_to([{:?}, {:?}]).unwrap()",
                            to[0], to[1]
                        )
                        .unwrap();
                    }
                    FloatCurveSegment::Quad { ctrl, to } => {
                        writeln!(
                            source,
                            "        .quad_to([{:?}, {:?}], [{:?}, {:?}]).unwrap()",
                            ctrl[0], ctrl[1], to[0], to[1]
                        )
                        .unwrap();
                    }
                    FloatCurveSegment::Cubic { ctrl0, ctrl1, to } => {
                        writeln!(
                            source,
                            "        .cubic_to([{:?}, {:?}], [{:?}, {:?}], [{:?}, {:?}]).unwrap()",
                            ctrl0[0], ctrl0[1], ctrl1[0], ctrl1[1], to[0], to[1]
                        )
                        .unwrap();
                    }
                    FloatCurveSegment::Arc { arc } => {
                        writeln!(source, "        .arc_to(EllipticArc {{").unwrap();
                        writeln!(source, "            ellipse: Ellipse {{").unwrap();
                        writeln!(
                            source,
                            "                center: [{:?}, {:?}],",
                            arc.ellipse.center[0], arc.ellipse.center[1]
                        )
                        .unwrap();
                        writeln!(
                            source,
                            "                radius_x: {:?},",
                            arc.ellipse.radius_x
                        )
                        .unwrap();
                        writeln!(
                            source,
                            "                radius_y: {:?},",
                            arc.ellipse.radius_y
                        )
                        .unwrap();
                        writeln!(
                            source,
                            "                rotation: {:?},",
                            arc.ellipse.rotation
                        )
                        .unwrap();
                        writeln!(source, "            }},").unwrap();
                        writeln!(source, "            start_angle: {:?},", arc.start_angle)
                            .unwrap();
                        writeln!(source, "            sweep_angle: {:?},", arc.sweep_angle)
                            .unwrap();
                        writeln!(source, "        }}).unwrap()").unwrap();
                    }
                }
            }
        }
    }
}

fn panic_message(payload: Box<dyn std::any::Any + Send>) -> String {
    let message = payload
        .downcast_ref::<&str>()
        .copied()
        .or_else(|| payload.downcast_ref::<String>().map(String::as_str))
        .unwrap_or("unknown panic");

    format!("Overlay panic: {message}")
}

#[derive(Clone, Copy)]
struct ShapeStyle {
    fill: Color32,
    stroke: Stroke,
    controls: ControlStyle,
}

#[derive(Clone, Copy)]
struct ControlStyle {
    arm_stroke: Stroke,
    anchor_fill: Color32,
    control_fill: Color32,
    center_fill: Color32,
    point_stroke: Stroke,
}

impl ControlStyle {
    fn input() -> Self {
        Self {
            arm_stroke: Stroke::new(1.0_f32, Color32::from_rgba_unmultiplied(196, 202, 214, 80)),
            anchor_fill: Color32::from_rgba_unmultiplied(255, 206, 102, 150),
            control_fill: Color32::from_rgba_unmultiplied(240, 118, 118, 150),
            center_fill: Color32::from_rgba_unmultiplied(128, 212, 156, 150),
            point_stroke: Stroke::new(1.0_f32, Color32::from_rgba_unmultiplied(18, 20, 24, 180)),
        }
    }

    fn result() -> Self {
        Self {
            arm_stroke: Stroke::new(1.0_f32, Color32::from_rgba_unmultiplied(150, 156, 166, 65)),
            anchor_fill: Color32::from_rgba_unmultiplied(165, 171, 181, 105),
            control_fill: Color32::from_rgba_unmultiplied(135, 141, 151, 90),
            center_fill: Color32::from_rgba_unmultiplied(150, 156, 166, 95),
            point_stroke: Stroke::new(1.0_f32, Color32::from_rgba_unmultiplied(18, 20, 24, 120)),
        }
    }
}

fn paint_float_shapes(
    painter: &Painter,
    rect: Rect,
    camera: &Camera,
    shapes: &[FloatCurveShape<CurvePoint>],
    style: ShapeStyle,
) {
    for shape in shapes {
        for contour in shape.contours() {
            paint_sampled_path(painter, rect, camera, sample_float_contour(contour), style);
            paint_float_control_points(painter, rect, camera, contour, style.controls);
        }
    }
}

fn paint_sampled_path(
    painter: &Painter,
    rect: Rect,
    camera: &Camera,
    world_points: Vec<CurvePoint>,
    style: ShapeStyle,
) {
    if world_points.len() < 2 {
        return;
    }

    let screen_points = world_points
        .iter()
        .map(|point| camera.screen_from_world(rect, point_to_pos(*point)))
        .collect::<Vec<_>>();

    if style.fill != Color32::TRANSPARENT && screen_points.len() >= 3 {
        painter.add(PathShape::convex_polygon(
            screen_points.clone(),
            style.fill,
            Stroke::new(0.0_f32, Color32::TRANSPARENT),
        ));
    }

    painter.add(Shape::closed_line(screen_points, style.stroke));
}

fn edit_float_shapes(
    ui: &mut Ui,
    painter: &Painter,
    rect: Rect,
    camera: &Camera,
    id_source: &'static str,
    shapes: &mut [FloatCurveShape<CurvePoint>],
    style: ShapeStyle,
) -> bool {
    let changed = interact_float_shapes(ui, rect, camera, id_source, shapes);
    paint_float_shapes(painter, rect, camera, shapes, style);
    changed
}

fn interact_float_shapes(
    ui: &mut Ui,
    rect: Rect,
    camera: &Camera,
    id_source: &'static str,
    shapes: &mut [FloatCurveShape<CurvePoint>],
) -> bool {
    let mut changed = false;

    for (shape_index, shape) in shapes.iter_mut().enumerate() {
        let mut edits = Vec::new();
        for (contour_index, contour) in shape.contours().iter().enumerate() {
            let id = Id::new(id_source).with(shape_index).with(contour_index);
            collect_float_contour_edits(ui, rect, camera, id, contour, &mut edits);
        }

        for edit in edits {
            *shape = rebuild_float_shape(shape, edit).expect("edited shape must stay valid");
            changed = true;
        }
    }

    changed
}

#[derive(Clone, Copy)]
enum ControlEdit {
    MovePoint {
        point: CurvePoint,
        position: CurvePoint,
    },
    MoveArc {
        center: CurvePoint,
        start: CurvePoint,
        end: CurvePoint,
        delta: Vec2,
    },
}

fn collect_float_contour_edits(
    ui: &mut Ui,
    rect: Rect,
    camera: &Camera,
    id: Id,
    contour: &FloatCurvePath<CurvePoint>,
    edits: &mut Vec<ControlEdit>,
) {
    let locks_anchors = contour
        .segments()
        .iter()
        .any(|segment| matches!(segment, FloatCurveSegment::Arc { .. }));
    let mut start = contour.start();
    let mut handled_arc_centers = Vec::new();

    if !locks_anchors
        && let Some(position) =
            interact_point_position(ui, id.with("start"), rect, camera, start, 3.5)
    {
        edits.push(ControlEdit::MovePoint {
            point: start,
            position,
        });
    }

    for (segment_index, segment) in contour.segments().iter().enumerate() {
        let segment_id = id.with(segment_index);
        match segment {
            FloatCurveSegment::Line { to } => {
                if !locks_anchors
                    && let Some(position) =
                        interact_point_position(ui, segment_id.with("to"), rect, camera, *to, 3.5)
                {
                    edits.push(ControlEdit::MovePoint {
                        point: *to,
                        position,
                    });
                }
                start = *to;
            }
            FloatCurveSegment::Quad { ctrl, to } => {
                if let Some(position) =
                    interact_point_position(ui, segment_id.with("ctrl"), rect, camera, *ctrl, 4.5)
                {
                    edits.push(ControlEdit::MovePoint {
                        point: *ctrl,
                        position,
                    });
                }
                if !locks_anchors
                    && let Some(position) =
                        interact_point_position(ui, segment_id.with("to"), rect, camera, *to, 3.5)
                {
                    edits.push(ControlEdit::MovePoint {
                        point: *to,
                        position,
                    });
                }
                start = *to;
            }
            FloatCurveSegment::Cubic { ctrl0, ctrl1, to } => {
                if let Some(position) =
                    interact_point_position(ui, segment_id.with("ctrl0"), rect, camera, *ctrl0, 4.5)
                {
                    edits.push(ControlEdit::MovePoint {
                        point: *ctrl0,
                        position,
                    });
                }
                if let Some(position) =
                    interact_point_position(ui, segment_id.with("ctrl1"), rect, camera, *ctrl1, 4.5)
                {
                    edits.push(ControlEdit::MovePoint {
                        point: *ctrl1,
                        position,
                    });
                }
                if !locks_anchors
                    && let Some(position) =
                        interact_point_position(ui, segment_id.with("to"), rect, camera, *to, 3.5)
                {
                    edits.push(ControlEdit::MovePoint {
                        point: *to,
                        position,
                    });
                }
                start = *to;
            }
            FloatCurveSegment::Arc { arc } => {
                let center = arc.ellipse.center;
                let end = arc.end_point();
                if !handled_arc_centers
                    .iter()
                    .any(|handled| same_point(*handled, center))
                    && let Some(position) = interact_point_position(
                        ui,
                        segment_id.with("center"),
                        rect,
                        camera,
                        center,
                        4.5,
                    )
                {
                    edits.push(ControlEdit::MoveArc {
                        center,
                        start,
                        end,
                        delta: Vec2::new(position[0] - center[0], position[1] - center[1]),
                    });
                    handled_arc_centers.push(center);
                }
                start = end;
            }
        }
    }
}

fn interact_point_position(
    ui: &mut Ui,
    id: Id,
    rect: Rect,
    camera: &Camera,
    point: CurvePoint,
    radius: f32,
) -> Option<CurvePoint> {
    let screen = camera.screen_from_world(rect, point_to_pos(point));
    let hit_rect = Rect::from_center_size(screen, Vec2::splat(radius * 4.0));
    let response = ui
        .interact(hit_rect, id, Sense::drag())
        .on_hover_cursor(CursorIcon::Grab);

    if !response.dragged() {
        return None;
    }

    let screen_position = ui.input(|input| input.pointer.interact_pos())?;
    let world = camera.world_from_screen(rect, screen_position);
    Some([world.x, world.y])
}

fn rebuild_float_shape(
    shape: &FloatCurveShape<CurvePoint>,
    edit: ControlEdit,
) -> Result<FloatCurveShape<CurvePoint>, CurveError> {
    let mut builder = CurveBuilder::new();
    let endpoint_mappings = arc_endpoint_mappings(shape, edit);

    for contour in shape.contours() {
        builder.move_to(map_point(contour.start(), edit, &endpoint_mappings))?;
        for segment in contour.segments() {
            match segment {
                FloatCurveSegment::Line { to } => {
                    builder.line_to(map_point(*to, edit, &endpoint_mappings))?
                }
                FloatCurveSegment::Quad { ctrl, to } => builder.quad_to(
                    map_point(*ctrl, edit, &endpoint_mappings),
                    map_point(*to, edit, &endpoint_mappings),
                )?,
                FloatCurveSegment::Cubic { ctrl0, ctrl1, to } => builder.cubic_to(
                    map_point(*ctrl0, edit, &endpoint_mappings),
                    map_point(*ctrl1, edit, &endpoint_mappings),
                    map_point(*to, edit, &endpoint_mappings),
                )?,
                FloatCurveSegment::Arc { arc } => builder.rational_arc_to(map_arc(*arc, edit))?,
            };
        }
    }

    builder.build()
}

fn arc_endpoint_mappings(
    shape: &FloatCurveShape<CurvePoint>,
    edit: ControlEdit,
) -> Vec<(CurvePoint, CurvePoint)> {
    let mut mappings = Vec::new();

    for contour in shape.contours() {
        for segment in contour.segments() {
            let FloatCurveSegment::Arc { arc } = segment else {
                continue;
            };
            let mapped = map_arc(*arc, edit);
            if mapped != *arc {
                mappings.push((arc.start_point(), mapped.start_point()));
                mappings.push((arc.end_point(), mapped.end_point()));
            }
        }
    }

    mappings
}

fn map_point(
    point: CurvePoint,
    edit: ControlEdit,
    endpoint_mappings: &[(CurvePoint, CurvePoint)],
) -> CurvePoint {
    if let Some((_, mapped)) = endpoint_mappings
        .iter()
        .find(|(source, _)| same_point(point, *source))
    {
        return *mapped;
    }

    match edit {
        ControlEdit::MovePoint {
            point: target,
            position,
        } if same_point(point, target) => position,
        ControlEdit::MoveArc {
            center,
            start,
            end,
            delta,
        } if same_point(point, center) || same_point(point, start) || same_point(point, end) => {
            [point[0] + delta.x, point[1] + delta.y]
        }
        _ => point,
    }
}

fn map_arc(mut arc: RationalArc<CurvePoint>, edit: ControlEdit) -> RationalArc<CurvePoint> {
    let delta = match edit {
        ControlEdit::MovePoint { point, position } if same_point(arc.ellipse.center, point) => {
            Vec2::new(
                position[0] - arc.ellipse.center[0],
                position[1] - arc.ellipse.center[1],
            )
        }
        ControlEdit::MoveArc { center, delta, .. } if same_point(arc.ellipse.center, center) => {
            delta
        }
        _ => Vec2::ZERO,
    };
    arc.ellipse.center[0] += delta.x;
    arc.ellipse.center[1] += delta.y;
    for point in &mut arc.control_points {
        point[0] += delta.x;
        point[1] += delta.y;
    }
    arc
}

fn same_point(a: CurvePoint, b: CurvePoint) -> bool {
    (a[0] - b[0]).abs() < 0.001 && (a[1] - b[1]).abs() < 0.001
}

fn paint_float_control_points(
    painter: &Painter,
    rect: Rect,
    camera: &Camera,
    contour: &FloatCurvePath<CurvePoint>,
    style: ControlStyle,
) {
    let mut start = contour.start();
    paint_point(
        painter,
        rect,
        camera,
        start,
        3.5,
        style.anchor_fill,
        style.point_stroke,
    );

    for segment in contour.segments() {
        match segment {
            FloatCurveSegment::Line { to } => {
                paint_point(
                    painter,
                    rect,
                    camera,
                    *to,
                    3.5,
                    style.anchor_fill,
                    style.point_stroke,
                );
                start = *to;
            }
            FloatCurveSegment::Quad { ctrl, to } => {
                paint_polyline(
                    painter,
                    rect,
                    camera,
                    &[start, *ctrl, *to],
                    style.arm_stroke,
                );
                paint_point(
                    painter,
                    rect,
                    camera,
                    *ctrl,
                    4.5,
                    style.control_fill,
                    style.point_stroke,
                );
                paint_point(
                    painter,
                    rect,
                    camera,
                    *to,
                    3.5,
                    style.anchor_fill,
                    style.point_stroke,
                );
                start = *to;
            }
            FloatCurveSegment::Cubic { ctrl0, ctrl1, to } => {
                paint_polyline(
                    painter,
                    rect,
                    camera,
                    &[start, *ctrl0, *ctrl1, *to],
                    style.arm_stroke,
                );
                paint_point(
                    painter,
                    rect,
                    camera,
                    *ctrl0,
                    4.5,
                    style.control_fill,
                    style.point_stroke,
                );
                paint_point(
                    painter,
                    rect,
                    camera,
                    *ctrl1,
                    4.5,
                    style.control_fill,
                    style.point_stroke,
                );
                paint_point(
                    painter,
                    rect,
                    camera,
                    *to,
                    3.5,
                    style.anchor_fill,
                    style.point_stroke,
                );
                start = *to;
            }
            FloatCurveSegment::Arc { arc } => {
                let end = arc.end_point();
                paint_polyline(
                    painter,
                    rect,
                    camera,
                    &[arc.ellipse.center, start],
                    style.arm_stroke,
                );
                paint_polyline(
                    painter,
                    rect,
                    camera,
                    &[arc.ellipse.center, end],
                    style.arm_stroke,
                );
                paint_point(
                    painter,
                    rect,
                    camera,
                    arc.ellipse.center,
                    4.5,
                    style.center_fill,
                    style.point_stroke,
                );
                paint_point(
                    painter,
                    rect,
                    camera,
                    end,
                    3.5,
                    style.anchor_fill,
                    style.point_stroke,
                );
                start = end;
            }
        }
    }
}

fn paint_polyline(
    painter: &Painter,
    rect: Rect,
    camera: &Camera,
    points: &[CurvePoint],
    stroke: Stroke,
) {
    for pair in points.windows(2) {
        painter.line_segment(
            [
                camera.screen_from_world(rect, point_to_pos(pair[0])),
                camera.screen_from_world(rect, point_to_pos(pair[1])),
            ],
            stroke,
        );
    }
}

fn paint_point(
    painter: &Painter,
    rect: Rect,
    camera: &Camera,
    point: CurvePoint,
    radius: f32,
    fill: Color32,
    stroke: Stroke,
) {
    let screen = camera.screen_from_world(rect, point_to_pos(point));
    painter.circle_filled(screen, radius, fill);
    painter.circle_stroke(screen, radius, stroke);
}

#[derive(Clone, Copy)]
struct Bounds {
    min_x: f32,
    max_x: f32,
    min_y: f32,
    max_y: f32,
}

fn example_bounds(example: &BoolExample) -> Option<Bounds> {
    let mut bounds = None;

    for shape in example.subject.iter().chain(&example.clip) {
        for contour in shape.contours() {
            for point in sample_float_contour(contour) {
                bounds = Some(match bounds {
                    Some(bounds) => add_point_to_bounds(bounds, point),
                    None => Bounds {
                        min_x: point[0],
                        max_x: point[0],
                        min_y: point[1],
                        max_y: point[1],
                    },
                });
            }
        }
    }

    bounds
}

fn add_point_to_bounds(bounds: Bounds, point: CurvePoint) -> Bounds {
    Bounds {
        min_x: bounds.min_x.min(point[0]),
        max_x: bounds.max_x.max(point[0]),
        min_y: bounds.min_y.min(point[1]),
        max_y: bounds.max_y.max(point[1]),
    }
}

fn sample_float_contour(contour: &FloatCurvePath<CurvePoint>) -> Vec<CurvePoint> {
    let mut points = vec![contour.start()];
    let mut start = contour.start();

    for segment in contour.segments() {
        match segment {
            FloatCurveSegment::Line { to } => {
                points.push(*to);
                start = *to;
            }
            FloatCurveSegment::Quad { ctrl, to } => {
                push_samples(&mut points, 24, |t| quad_point(start, *ctrl, *to, t));
                start = *to;
            }
            FloatCurveSegment::Cubic { ctrl0, ctrl1, to } => {
                push_samples(&mut points, 32, |t| {
                    cubic_point(start, *ctrl0, *ctrl1, *to, t)
                });
                start = *to;
            }
            FloatCurveSegment::Arc { arc } => {
                push_samples(&mut points, 48, |t| {
                    arc.ellipse.point_at(arc.start_angle + arc.sweep_angle * t)
                });
                start = arc.end_point();
            }
        }
    }

    points
}

fn push_samples(
    points: &mut Vec<CurvePoint>,
    sample_count: usize,
    sample: impl Fn(f32) -> CurvePoint,
) {
    for index in 1..=sample_count {
        let t = index as f32 / sample_count as f32;
        points.push(sample(t));
    }
}

fn point_to_pos(point: CurvePoint) -> Pos2 {
    Pos2::new(point[0], point[1])
}

fn line_point(p0: CurvePoint, p1: CurvePoint, t: f32) -> CurvePoint {
    [p0[0] + (p1[0] - p0[0]) * t, p0[1] + (p1[1] - p0[1]) * t]
}

fn quad_point(p0: CurvePoint, p1: CurvePoint, p2: CurvePoint, t: f32) -> CurvePoint {
    let a = line_point(p0, p1, t);
    let b = line_point(p1, p2, t);
    line_point(a, b, t)
}

fn cubic_point(
    p0: CurvePoint,
    p1: CurvePoint,
    p2: CurvePoint,
    p3: CurvePoint,
    t: f32,
) -> CurvePoint {
    let a = quad_point(p0, p1, p2, t);
    let b = quad_point(p1, p2, p3, t);
    line_point(a, b, t)
}

fn main() -> eframe::Result<()> {
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("FloatCurveOverlay Boolean")
            .with_inner_size(Vec2::new(1040.0, 760.0)),
        ..eframe::NativeOptions::default()
    };

    eframe::run_native(
        "FloatCurveOverlay Boolean",
        native_options,
        Box::new(|_cc| Ok(Box::new(BoolApp::default()))),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn float_arc_count(example: &BoolExample) -> usize {
        example
            .subject
            .iter()
            .chain(&example.clip)
            .flat_map(|shape| shape.contours())
            .flat_map(|contour| contour.segments())
            .filter(|segment| matches!(segment, FloatCurveSegment::Arc { .. }))
            .count()
    }

    #[test]
    fn console_input_uses_float_overlay_api() {
        let example = load_examples()
            .into_iter()
            .find(|example| float_arc_count(example) > 0)
            .expect("arc example");
        let source = overlay_input_source(&example, OverlayRule::Difference, FillRule::EvenOdd);

        assert!(source.contains("CurveBuilder::new()"));
        assert!(source.contains("EllipticArc"));
        assert!(source.contains("FloatCurveOverlay::with_subj_and_clip(&subject, &clip)"));
        assert!(source.contains(".overlay(OverlayRule::Difference, FillRule::EvenOdd)"));
        assert!(!source.contains("IntCurveOverlay"));
        assert!(!source.contains("CurveConverter"));
    }

    #[test]
    fn moving_full_arc_center_preserves_exact_closure() {
        let mut shape = load_examples()
            .into_iter()
            .find(|example| example.name == "rotated arc ellipses")
            .expect("rotated arc example")
            .subject
            .into_iter()
            .next()
            .expect("subject shape");

        for delta in [
            Vec2::new(-50.14, 4.76504),
            Vec2::new(0.137, -0.293),
            Vec2::new(-0.019, 0.071),
        ] {
            let contour = &shape.contours()[0];
            let arc = contour
                .segments()
                .iter()
                .find_map(|segment| match segment {
                    FloatCurveSegment::Arc { arc } => Some(*arc),
                    _ => None,
                })
                .expect("full arc");

            shape = rebuild_float_shape(
                &shape,
                ControlEdit::MoveArc {
                    center: arc.ellipse.center,
                    start: arc.start_point(),
                    end: arc.end_point(),
                    delta,
                },
            )
            .expect("moving an arc center must preserve a valid contour");

            let moved_contour = &shape.contours()[0];
            let moved_arcs: Vec<_> = moved_contour
                .segments()
                .iter()
                .filter_map(|segment| match segment {
                    FloatCurveSegment::Arc { arc } => Some(*arc),
                    _ => None,
                })
                .collect();
            let first = moved_arcs.first().expect("moved arc pieces");
            let last = moved_arcs.last().expect("moved arc pieces");

            assert_eq!(moved_contour.start(), first.start_point());
            assert_eq!(moved_contour.end_point(), Some(last.end_point()));
            assert!(moved_contour.is_closed());
        }
    }

    #[test]
    fn basic_operations_include_arc_examples() {
        let examples = load_examples();
        assert!(examples.iter().any(|example| float_arc_count(example) > 0));

        for example in examples {
            for rule in OVERLAY_RULES {
                for fill_rule in FILL_RULES {
                    assert!(
                        build_overlay_result(&example, rule, fill_rule).is_ok(),
                        "{} with {rule} and {fill_rule}",
                        example.name
                    );
                }
            }
        }
    }
}
