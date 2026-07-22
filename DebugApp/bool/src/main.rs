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
use i_curve::int::{
    bool::overlay::IntCurveOverlay,
    curve::{path::CurvePath, segment::CurveSegment, shape::CurveShape},
};
use i_overlay::core::{fill_rule::FillRule, overlay::ShapeType, overlay_rule::OverlayRule};
use i_overlay::i_shape::int::IntPoint;
use std::fmt::Write;

type DrawPoint = [f32; 2];

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

struct BoolApp {
    camera: Camera,
    grid: Grid,
    examples: Vec<BoolExample>,
    active_example: usize,
    overlay_rule: OverlayRule,
    fill_rule: FillRule,
    show_mode: ShowMode,
    result: Result<Vec<CurveShape<i32>>, String>,
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
            result: Ok(Vec::new()),
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
                    Ok(shapes) => {
                        let contour_count = shapes
                            .iter()
                            .map(|shape| shape.contours.len())
                            .sum::<usize>();
                        ui.label(format!("Result shapes: {}", shapes.len()));
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
                    inputs_changed |= edit_shapes(
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
                    inputs_changed |= edit_shapes(
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
                    paint_shapes(
                        &painter,
                        rect,
                        &camera,
                        result,
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

        self.result = std::panic::catch_unwind(move || {
            let capacity = segment_count(&example);
            let mut overlay = IntCurveOverlay::new(capacity);

            for shape in example.subject {
                overlay.add_shape(shape, ShapeType::Subject);
            }
            for shape in example.clip {
                overlay.add_shape(shape, ShapeType::Clip);
            }

            overlay.overlay(overlay_rule, fill_rule)
        })
        .map_err(panic_message);
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

fn print_overlay_input(example: &BoolExample, overlay_rule: OverlayRule, fill_rule: FillRule) {
    println!("{}", overlay_input_source(example, overlay_rule, fill_rule));
}

fn overlay_input_source(
    example: &BoolExample,
    overlay_rule: OverlayRule,
    fill_rule: FillRule,
) -> String {
    let mut source = String::new();
    writeln!(source, "\n// IntCurveOverlay input: {}", example.name).unwrap();
    writeln!(source, "#[test]").unwrap();
    writeln!(source, "fn reproduced_overlay_case() {{").unwrap();
    writeln!(
        source,
        "    use i_curve::int::{{bool::overlay::IntCurveOverlay, curve::{{path::CurvePath, segment::CurveSegment, shape::CurveShape}}}};"
    )
    .unwrap();
    writeln!(
        source,
        "    use i_overlay::core::{{fill_rule::FillRule, overlay::ShapeType, overlay_rule::OverlayRule}};"
    )
    .unwrap();
    writeln!(source, "    use i_overlay::i_shape::int::IntPoint;\n").unwrap();

    write_shapes(&mut source, "subject", &example.subject);
    writeln!(source).unwrap();
    write_shapes(&mut source, "clip", &example.clip);
    writeln!(source).unwrap();
    writeln!(
        source,
        "    let mut overlay = IntCurveOverlay::new({});",
        segment_count(example)
    )
    .unwrap();
    writeln!(source, "    for shape in subject {{").unwrap();
    writeln!(
        source,
        "        overlay.add_shape(shape, ShapeType::Subject);"
    )
    .unwrap();
    writeln!(source, "    }}").unwrap();
    writeln!(source, "    for shape in clip {{").unwrap();
    writeln!(source, "        overlay.add_shape(shape, ShapeType::Clip);").unwrap();
    writeln!(source, "    }}").unwrap();
    writeln!(
        source,
        "    let result = overlay.overlay(OverlayRule::{overlay_rule}, FillRule::{fill_rule});"
    )
    .unwrap();
    writeln!(source, "    dbg!(result);").unwrap();
    writeln!(source, "}}").unwrap();

    source
}

fn write_shapes(source: &mut String, name: &str, shapes: &[CurveShape<i32>]) {
    writeln!(source, "    let {name} = vec![").unwrap();
    for shape in shapes {
        writeln!(source, "        CurveShape {{").unwrap();
        writeln!(source, "            contours: vec![").unwrap();
        for contour in &shape.contours {
            writeln!(source, "                CurvePath {{").unwrap();
            writeln!(
                source,
                "                    start: IntPoint::new({}, {}),",
                contour.start.x, contour.start.y
            )
            .unwrap();
            writeln!(source, "                    segments: vec![").unwrap();
            for segment in &contour.segments {
                write_segment(source, segment);
            }
            writeln!(source, "                    ],").unwrap();
            writeln!(source, "                }},").unwrap();
        }
        writeln!(source, "            ],").unwrap();
        writeln!(source, "        }},").unwrap();
    }
    writeln!(source, "    ];").unwrap();
}

fn write_segment(source: &mut String, segment: &CurveSegment<i32>) {
    match segment {
        CurveSegment::Line { to } => {
            writeln!(
                source,
                "                        CurveSegment::Line {{ to: IntPoint::new({}, {}) }},",
                to.x, to.y
            )
            .unwrap();
        }
        CurveSegment::Quad { ctrl, to } => {
            writeln!(
                source,
                "                        CurveSegment::Quad {{ ctrl: IntPoint::new({}, {}), to: IntPoint::new({}, {}) }},",
                ctrl.x, ctrl.y, to.x, to.y
            )
            .unwrap();
        }
        CurveSegment::Cubic { ctrl0, ctrl1, to } => {
            writeln!(
                source,
                "                        CurveSegment::Cubic {{ ctrl0: IntPoint::new({}, {}), ctrl1: IntPoint::new({}, {}), to: IntPoint::new({}, {}) }},",
                ctrl0.x, ctrl0.y, ctrl1.x, ctrl1.y, to.x, to.y
            )
            .unwrap();
        }
    }
}

fn segment_count(example: &BoolExample) -> usize {
    example
        .subject
        .iter()
        .chain(&example.clip)
        .flat_map(|shape| &shape.contours)
        .map(|contour| contour.segments.len())
        .sum()
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
    point_stroke: Stroke,
}

impl ControlStyle {
    fn input() -> Self {
        Self {
            arm_stroke: Stroke::new(1.0_f32, Color32::from_rgba_unmultiplied(196, 202, 214, 80)),
            anchor_fill: Color32::from_rgba_unmultiplied(255, 206, 102, 150),
            control_fill: Color32::from_rgba_unmultiplied(240, 118, 118, 150),
            point_stroke: Stroke::new(1.0_f32, Color32::from_rgba_unmultiplied(18, 20, 24, 180)),
        }
    }

    fn result() -> Self {
        Self {
            arm_stroke: Stroke::new(1.0_f32, Color32::from_rgba_unmultiplied(150, 156, 166, 65)),
            anchor_fill: Color32::from_rgba_unmultiplied(165, 171, 181, 105),
            control_fill: Color32::from_rgba_unmultiplied(135, 141, 151, 90),
            point_stroke: Stroke::new(1.0_f32, Color32::from_rgba_unmultiplied(18, 20, 24, 120)),
        }
    }
}

fn paint_shapes(
    painter: &Painter,
    rect: Rect,
    camera: &Camera,
    shapes: &[CurveShape<i32>],
    style: ShapeStyle,
) {
    for shape in shapes {
        for contour in &shape.contours {
            let world_points = sample_contour(contour);
            if world_points.len() < 2 {
                continue;
            }

            let screen_points = world_points
                .iter()
                .map(|point| camera.screen_from_world(rect, draw_point_to_pos(*point)))
                .collect::<Vec<_>>();

            if style.fill != Color32::TRANSPARENT && screen_points.len() >= 3 {
                painter.add(PathShape::convex_polygon(
                    screen_points.clone(),
                    style.fill,
                    Stroke::new(0.0_f32, Color32::TRANSPARENT),
                ));
            }

            painter.add(Shape::closed_line(screen_points, style.stroke));
            paint_control_points(painter, rect, camera, contour, style.controls);
        }
    }
}

fn edit_shapes(
    ui: &mut Ui,
    painter: &Painter,
    rect: Rect,
    camera: &Camera,
    id_source: &'static str,
    shapes: &mut [CurveShape<i32>],
    style: ShapeStyle,
) -> bool {
    let changed = interact_shapes(ui, rect, camera, id_source, shapes);
    paint_shapes(painter, rect, camera, shapes, style);
    changed
}

fn interact_shapes(
    ui: &mut Ui,
    rect: Rect,
    camera: &Camera,
    id_source: &'static str,
    shapes: &mut [CurveShape<i32>],
) -> bool {
    let mut changed = false;
    for (shape_index, shape) in shapes.iter_mut().enumerate() {
        for (contour_index, contour) in shape.contours.iter_mut().enumerate() {
            let id = Id::new(id_source).with(shape_index).with(contour_index);
            changed |= interact_contour(ui, rect, camera, id, contour);
        }
    }
    changed
}

fn interact_contour(
    ui: &mut Ui,
    rect: Rect,
    camera: &Camera,
    id: Id,
    contour: &mut CurvePath<i32>,
) -> bool {
    let mut edits = Vec::new();

    if let Some(position) =
        interact_point_position(ui, id.with("start"), rect, camera, contour.start, 3.5)
    {
        edits.push((contour.start, position));
    }

    for (segment_index, segment) in contour.segments.iter().enumerate() {
        let segment_id = id.with(segment_index);
        match segment {
            CurveSegment::Line { to } => {
                if let Some(position) =
                    interact_point_position(ui, segment_id.with("to"), rect, camera, *to, 3.5)
                {
                    edits.push((*to, position));
                }
            }
            CurveSegment::Quad { ctrl, to } => {
                if let Some(position) =
                    interact_point_position(ui, segment_id.with("ctrl"), rect, camera, *ctrl, 4.5)
                {
                    edits.push((*ctrl, position));
                }
                if let Some(position) =
                    interact_point_position(ui, segment_id.with("to"), rect, camera, *to, 3.5)
                {
                    edits.push((*to, position));
                }
            }
            CurveSegment::Cubic { ctrl0, ctrl1, to } => {
                if let Some(position) =
                    interact_point_position(ui, segment_id.with("ctrl0"), rect, camera, *ctrl0, 4.5)
                {
                    edits.push((*ctrl0, position));
                }
                if let Some(position) =
                    interact_point_position(ui, segment_id.with("ctrl1"), rect, camera, *ctrl1, 4.5)
                {
                    edits.push((*ctrl1, position));
                }
                if let Some(position) =
                    interact_point_position(ui, segment_id.with("to"), rect, camera, *to, 3.5)
                {
                    edits.push((*to, position));
                }
            }
        }
    }

    let changed = !edits.is_empty();
    for (point, position) in edits {
        move_matching_points(contour, point, position);
    }
    changed
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
    Some(IntPoint::new(
        world.x.round() as i32,
        world.y.round() as i32,
    ))
}

fn move_matching_points(contour: &mut CurvePath<i32>, point: CurvePoint, position: CurvePoint) {
    move_if_matching(&mut contour.start, point, position);

    for segment in &mut contour.segments {
        match segment {
            CurveSegment::Line { to } => move_if_matching(to, point, position),
            CurveSegment::Quad { ctrl, to } => {
                move_if_matching(ctrl, point, position);
                move_if_matching(to, point, position);
            }
            CurveSegment::Cubic { ctrl0, ctrl1, to } => {
                move_if_matching(ctrl0, point, position);
                move_if_matching(ctrl1, point, position);
                move_if_matching(to, point, position);
            }
        }
    }
}

fn move_if_matching(target: &mut CurvePoint, point: CurvePoint, position: CurvePoint) {
    if *target == point {
        *target = position;
    }
}

fn paint_control_points(
    painter: &Painter,
    rect: Rect,
    camera: &Camera,
    contour: &CurvePath<i32>,
    style: ControlStyle,
) {
    let mut start = contour.start;
    paint_point(
        painter,
        rect,
        camera,
        start,
        3.5,
        style.anchor_fill,
        style.point_stroke,
    );

    for segment in &contour.segments {
        match segment {
            CurveSegment::Line { to } => {
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
            CurveSegment::Quad { ctrl, to } => {
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
            CurveSegment::Cubic { ctrl0, ctrl1, to } => {
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
        for contour in &shape.contours {
            for point in sample_contour(contour) {
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

fn add_point_to_bounds(bounds: Bounds, point: DrawPoint) -> Bounds {
    Bounds {
        min_x: bounds.min_x.min(point[0]),
        max_x: bounds.max_x.max(point[0]),
        min_y: bounds.min_y.min(point[1]),
        max_y: bounds.max_y.max(point[1]),
    }
}

fn sample_contour(contour: &CurvePath<i32>) -> Vec<DrawPoint> {
    let mut points = vec![point_to_draw(contour.start)];
    let mut start = point_to_draw(contour.start);

    for segment in &contour.segments {
        match segment {
            CurveSegment::Line { to } => {
                let end = point_to_draw(*to);
                points.push(end);
                start = end;
            }
            CurveSegment::Quad { ctrl, to } => {
                let control = point_to_draw(*ctrl);
                let end = point_to_draw(*to);
                push_samples(&mut points, 24, |t| quad_point(start, control, end, t));
                start = end;
            }
            CurveSegment::Cubic { ctrl0, ctrl1, to } => {
                let control0 = point_to_draw(*ctrl0);
                let control1 = point_to_draw(*ctrl1);
                let end = point_to_draw(*to);
                push_samples(&mut points, 32, |t| {
                    cubic_point(start, control0, control1, end, t)
                });
                start = end;
            }
        }
    }

    points
}

fn push_samples(
    points: &mut Vec<DrawPoint>,
    sample_count: usize,
    sample: impl Fn(f32) -> DrawPoint,
) {
    for index in 1..=sample_count {
        let t = index as f32 / sample_count as f32;
        points.push(sample(t));
    }
}

fn point_to_pos(point: CurvePoint) -> Pos2 {
    Pos2::new(point.x as f32, point.y as f32)
}

fn point_to_draw(point: CurvePoint) -> DrawPoint {
    [point.x as f32, point.y as f32]
}

fn draw_point_to_pos(point: DrawPoint) -> Pos2 {
    Pos2::new(point[0], point[1])
}

fn line_point(p0: DrawPoint, p1: DrawPoint, t: f32) -> DrawPoint {
    [p0[0] + (p1[0] - p0[0]) * t, p0[1] + (p1[1] - p0[1]) * t]
}

fn quad_point(p0: DrawPoint, p1: DrawPoint, p2: DrawPoint, t: f32) -> DrawPoint {
    let a = line_point(p0, p1, t);
    let b = line_point(p1, p2, t);
    line_point(a, b, t)
}

fn cubic_point(p0: DrawPoint, p1: DrawPoint, p2: DrawPoint, p3: DrawPoint, t: f32) -> DrawPoint {
    let a = quad_point(p0, p1, p2, t);
    let b = quad_point(p1, p2, p3, t);
    line_point(a, b, t)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn console_input_is_a_reproducible_test_template() {
        let example = &load_examples()[0];
        let source = overlay_input_source(example, OverlayRule::Difference, FillRule::EvenOdd);

        assert!(source.contains("fn reproduced_overlay_case()"));
        assert!(source.contains("start: IntPoint::new(-210, -130)"));
        assert!(source.contains("CurveSegment::Line { to: IntPoint::new(70, -130) }"));
        assert!(source.contains("overlay.overlay(OverlayRule::Difference, FillRule::EvenOdd)"));
    }

    #[test]
    fn basic_operations_use_integer_curve_overlay() {
        let mut app = BoolApp::default();

        for example_index in 0..app.examples.len() {
            app.active_example = example_index;
            for rule in OVERLAY_RULES {
                app.overlay_rule = rule;
                for fill_rule in FILL_RULES {
                    app.fill_rule = fill_rule;
                    app.refresh_result();
                    assert!(
                        app.result.is_ok(),
                        "{} with {rule} and {fill_rule}",
                        app.active_example().name
                    );
                }
            }
        }
    }
}

fn main() -> eframe::Result<()> {
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("IntCurveOverlay Boolean")
            .with_inner_size(Vec2::new(1040.0, 760.0)),
        ..eframe::NativeOptions::default()
    };

    eframe::run_native(
        "IntCurveOverlay Boolean",
        native_options,
        Box::new(|_cc| Ok(Box::new(BoolApp::default()))),
    )
}
