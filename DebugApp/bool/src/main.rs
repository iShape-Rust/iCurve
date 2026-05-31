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
    bool::overlay::{CurveOverlay, CurveOverlayOptions},
    curve::{arc::EllipticArc, contour::CurveContour, segment::CurveSegment, shape::CurveShape},
};
use i_overlay::core::{fill_rule::FillRule, overlay_rule::OverlayRule};

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
    show_linear_edges: bool,
    max_angle_deg: f32,
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
            show_linear_edges: false,
            max_angle_deg: 22.5,
        };
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
                    self.fit_active_example();
                }

                ui.add_space(8.0);
                ui.separator();

                egui::ComboBox::from_label("Operation")
                    .selected_text(self.overlay_rule.to_string())
                    .show_ui(ui, |ui| {
                        for rule in OVERLAY_RULES {
                            ui.selectable_value(&mut self.overlay_rule, rule, rule.to_string());
                        }
                    });

                egui::ComboBox::from_label("Fill rule")
                    .selected_text(self.fill_rule.to_string())
                    .show_ui(ui, |ui| {
                        for rule in FILL_RULES {
                            ui.selectable_value(&mut self.fill_rule, rule, rule.to_string());
                        }
                    });

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
                ui.checkbox(&mut self.show_linear_edges, "Linear edges");
                ui.add(
                    egui::Slider::new(&mut self.max_angle_deg, 1.0..=50.0).text("Max angle deg"),
                );

                ui.add_space(8.0);
                ui.separator();

                let result_count = self.result_shapes().len();
                let edge_count = self.linear_edges().len();
                let active = self.active_example();
                ui.label(format!("Subject shapes: {}", active.subject.len()));
                ui.label(format!("Clip shapes: {}", active.clip.len()));
                ui.label(format!("Result shapes: {result_count}"));
                ui.label(format!("Linear edges: {edge_count}"));

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

                if matches!(self.show_mode, ShowMode::Inputs | ShowMode::Both) {
                    let active = self.active_example_mut();
                    edit_shapes(
                        ui,
                        &painter,
                        rect,
                        &camera,
                        "subject",
                        &mut active.subject,
                        ShapeStyle {
                            fill: Color32::from_rgba_unmultiplied(73, 170, 255, 18),
                            stroke: Stroke::new(
                                2.0,
                                Color32::from_rgba_unmultiplied(86, 196, 255, 120),
                            ),
                            controls: ControlStyle::input(),
                        },
                    );
                    edit_shapes(
                        ui,
                        &painter,
                        rect,
                        &camera,
                        "clip",
                        &mut active.clip,
                        ShapeStyle {
                            fill: Color32::from_rgba_unmultiplied(255, 107, 107, 18),
                            stroke: Stroke::new(
                                2.0,
                                Color32::from_rgba_unmultiplied(240, 118, 118, 120),
                            ),
                            controls: ControlStyle::input(),
                        },
                    );
                }

                if self.show_linear_edges {
                    let edges = self.linear_edges();
                    paint_linear_edges(&painter, rect, &camera, &edges);
                }

                if matches!(self.show_mode, ShowMode::Result | ShowMode::Both) {
                    let result = self.result_shapes();
                    paint_shapes(
                        &painter,
                        rect,
                        &camera,
                        &result,
                        ShapeStyle {
                            fill: Color32::TRANSPARENT,
                            stroke: Stroke::new(3.0, Color32::from_rgb(78, 180, 91)),
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

    fn result_shapes(&self) -> Vec<CurveShape<CurvePoint>> {
        let active = self.active_example();
        let mut overlay = CurveOverlay::<CurvePoint, i32>::with_subj_and_clip_custom(
            &active.subject,
            &active.clip,
            self.overlay_options(),
        );
        overlay.overlay(self.overlay_rule, self.fill_rule)
    }

    fn linear_edges(&self) -> Vec<[CurvePoint; 2]> {
        let active = self.active_example();
        let overlay = CurveOverlay::<CurvePoint, i32>::with_subj_and_clip_custom(
            &active.subject,
            &active.clip,
            self.overlay_options(),
        );

        overlay.linear_edges()
    }

    fn overlay_options(&self) -> CurveOverlayOptions<f32, i32> {
        let mut options = CurveOverlayOptions::default();
        options.split.max_angle = self.max_angle_deg.to_radians();
        options
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
            arm_stroke: Stroke::new(1.0, Color32::from_rgba_unmultiplied(196, 202, 214, 80)),
            anchor_fill: Color32::from_rgba_unmultiplied(255, 206, 102, 135),
            control_fill: Color32::from_rgba_unmultiplied(240, 118, 118, 135),
            center_fill: Color32::from_rgba_unmultiplied(128, 212, 156, 135),
            point_stroke: Stroke::new(1.0, Color32::from_rgba_unmultiplied(18, 20, 24, 160)),
        }
    }

    fn result() -> Self {
        Self {
            arm_stroke: Stroke::new(1.0, Color32::from_rgba_unmultiplied(126, 231, 135, 95)),
            anchor_fill: Color32::from_rgb(255, 219, 128),
            control_fill: Color32::from_rgb(230, 122, 122),
            center_fill: Color32::from_rgb(128, 212, 156),
            point_stroke: Stroke::new(1.0, Color32::from_rgb(18, 20, 24)),
        }
    }
}

fn paint_shapes(
    painter: &Painter,
    rect: Rect,
    camera: &Camera,
    shapes: &[CurveShape<CurvePoint>],
    style: ShapeStyle,
) {
    for shape in shapes {
        for contour in &shape.contours {
            let world_points = sample_contour(contour);
            if world_points.len() < 2 {
                continue;
            }

            let screen_points: Vec<_> = world_points
                .iter()
                .map(|point| camera.screen_from_world(rect, point_to_pos(*point)))
                .collect();

            if style.fill != Color32::TRANSPARENT && screen_points.len() >= 3 {
                painter.add(PathShape::convex_polygon(
                    screen_points.clone(),
                    style.fill,
                    Stroke::new(0.0, Color32::TRANSPARENT),
                ));
            }

            painter.add(Shape::closed_line(screen_points, style.stroke));
            paint_control_points(painter, rect, camera, contour, style.controls);
        }
    }
}

fn paint_linear_edges(painter: &Painter, rect: Rect, camera: &Camera, edges: &[[CurvePoint; 2]]) {
    let stroke = Stroke::new(1.5, Color32::from_rgba_unmultiplied(255, 181, 79, 210));
    let point_fill = Color32::from_rgba_unmultiplied(255, 220, 130, 170);

    for edge in edges {
        let a = camera.screen_from_world(rect, point_to_pos(edge[0]));
        let b = camera.screen_from_world(rect, point_to_pos(edge[1]));
        painter.line_segment([a, b], stroke);
        painter.circle_filled(a, 2.5, point_fill);
        painter.circle_filled(b, 2.5, point_fill);
    }
}

fn edit_shapes(
    ui: &mut Ui,
    painter: &Painter,
    rect: Rect,
    camera: &Camera,
    id_source: &'static str,
    shapes: &mut [CurveShape<CurvePoint>],
    style: ShapeStyle,
) {
    interact_shapes(ui, rect, camera, id_source, shapes);
    paint_shapes(painter, rect, camera, shapes, style);
}

fn interact_shapes(
    ui: &mut Ui,
    rect: Rect,
    camera: &Camera,
    id_source: &'static str,
    shapes: &mut [CurveShape<CurvePoint>],
) {
    for (shape_index, shape) in shapes.iter_mut().enumerate() {
        for (contour_index, contour) in shape.contours.iter_mut().enumerate() {
            let id = Id::new(id_source).with(shape_index).with(contour_index);
            interact_contour(ui, rect, camera, id, contour);
        }
    }
}

enum ControlEdit {
    MoveMatching {
        point: CurvePoint,
        delta: Vec2,
    },
    MoveArcCenter {
        center: CurvePoint,
        start: CurvePoint,
        end: CurvePoint,
        delta: Vec2,
    },
}

fn interact_contour(
    ui: &mut Ui,
    rect: Rect,
    camera: &Camera,
    id: Id,
    contour: &mut CurveContour<CurvePoint>,
) {
    let mut edits = Vec::new();
    let mut start = contour.start;
    let mut handled_arc_centers = Vec::new();
    let locks_anchors = contour
        .segments
        .iter()
        .any(|segment| matches!(segment, CurveSegment::Arc { .. }));

    if !locks_anchors
        && let Some(delta) = interact_point_delta(ui, id.with("start"), rect, camera, start, 3.5)
    {
        edits.push(ControlEdit::MoveMatching {
            point: start,
            delta,
        });
    }

    for (segment_index, segment) in contour.segments.iter().enumerate() {
        let segment_id = id.with(segment_index);

        match segment {
            CurveSegment::Line { to } => {
                if !locks_anchors
                    && let Some(delta) =
                        interact_point_delta(ui, segment_id.with("to"), rect, camera, *to, 3.5)
                {
                    edits.push(ControlEdit::MoveMatching { point: *to, delta });
                }
                start = *to;
            }
            CurveSegment::Quad { ctrl, to } => {
                if let Some(delta) =
                    interact_point_delta(ui, segment_id.with("ctrl"), rect, camera, *ctrl, 4.5)
                {
                    edits.push(ControlEdit::MoveMatching {
                        point: *ctrl,
                        delta,
                    });
                }
                if !locks_anchors
                    && let Some(delta) =
                        interact_point_delta(ui, segment_id.with("to"), rect, camera, *to, 3.5)
                {
                    edits.push(ControlEdit::MoveMatching { point: *to, delta });
                }
                start = *to;
            }
            CurveSegment::Cubic { ctrl0, ctrl1, to } => {
                if let Some(delta) =
                    interact_point_delta(ui, segment_id.with("ctrl0"), rect, camera, *ctrl0, 4.5)
                {
                    edits.push(ControlEdit::MoveMatching {
                        point: *ctrl0,
                        delta,
                    });
                }
                if let Some(delta) =
                    interact_point_delta(ui, segment_id.with("ctrl1"), rect, camera, *ctrl1, 4.5)
                {
                    edits.push(ControlEdit::MoveMatching {
                        point: *ctrl1,
                        delta,
                    });
                }
                if !locks_anchors
                    && let Some(delta) =
                        interact_point_delta(ui, segment_id.with("to"), rect, camera, *to, 3.5)
                {
                    edits.push(ControlEdit::MoveMatching { point: *to, delta });
                }
                start = *to;
            }
            CurveSegment::Arc { arc } => {
                let end = arc_point(arc, 1.0);
                if !handled_arc_centers
                    .iter()
                    .any(|center| same_point(*center, arc.center))
                {
                    if let Some(delta) = interact_point_delta(
                        ui,
                        segment_id.with("center"),
                        rect,
                        camera,
                        arc.center,
                        4.0,
                    ) {
                        edits.push(ControlEdit::MoveArcCenter {
                            center: arc.center,
                            start,
                            end,
                            delta,
                        });
                    }
                    handled_arc_centers.push(arc.center);
                }
                start = end;
            }
        }
    }

    for edit in edits {
        match edit {
            ControlEdit::MoveMatching { point, delta } => {
                apply_delta_to_matching(contour, point, delta);
            }
            ControlEdit::MoveArcCenter {
                center,
                start,
                end,
                delta,
            } => {
                apply_delta_to_matching(contour, center, delta);
                apply_delta_to_matching(contour, start, delta);
                apply_delta_to_matching(contour, end, delta);
            }
        }
    }
}

fn interact_point_delta(
    ui: &mut Ui,
    id: Id,
    rect: Rect,
    camera: &Camera,
    point: CurvePoint,
    radius: f32,
) -> Option<Vec2> {
    let screen = camera.screen_from_world(rect, point_to_pos(point));
    let hit_rect = Rect::from_center_size(screen, Vec2::splat(radius * 4.0));
    let response = ui
        .interact(hit_rect, id, Sense::drag())
        .on_hover_cursor(CursorIcon::Grab);

    response
        .dragged()
        .then(|| camera.world_delta_from_screen_delta(response.drag_delta()))
}

fn apply_delta_to_matching(contour: &mut CurveContour<CurvePoint>, point: CurvePoint, delta: Vec2) {
    move_if_matching(&mut contour.start, point, delta);

    for segment in &mut contour.segments {
        match segment {
            CurveSegment::Line { to } => {
                move_if_matching(to, point, delta);
            }
            CurveSegment::Quad { ctrl, to } => {
                move_if_matching(ctrl, point, delta);
                move_if_matching(to, point, delta);
            }
            CurveSegment::Cubic { ctrl0, ctrl1, to } => {
                move_if_matching(ctrl0, point, delta);
                move_if_matching(ctrl1, point, delta);
                move_if_matching(to, point, delta);
            }
            CurveSegment::Arc { arc } => {
                move_if_matching(&mut arc.center, point, delta);
            }
        }
    }
}

fn move_if_matching(target: &mut CurvePoint, point: CurvePoint, delta: Vec2) {
    if same_point(*target, point) {
        target[0] += delta.x;
        target[1] += delta.y;
    }
}

fn same_point(a: CurvePoint, b: CurvePoint) -> bool {
    (a[0] - b[0]).abs() < 0.001 && (a[1] - b[1]).abs() < 0.001
}

fn paint_control_points(
    painter: &Painter,
    rect: Rect,
    camera: &Camera,
    contour: &CurveContour<CurvePoint>,
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
            CurveSegment::Arc { arc } => {
                let end = arc_point(arc, 1.0);
                paint_polyline(
                    painter,
                    rect,
                    camera,
                    &[arc.center, start],
                    style.arm_stroke,
                );
                paint_polyline(painter, rect, camera, &[arc.center, end], style.arm_stroke);
                paint_point(
                    painter,
                    rect,
                    camera,
                    arc.center,
                    4.0,
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

    for shape in example.subject.iter().chain(example.clip.iter()) {
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

fn add_point_to_bounds(bounds: Bounds, point: CurvePoint) -> Bounds {
    Bounds {
        min_x: bounds.min_x.min(point[0]),
        max_x: bounds.max_x.max(point[0]),
        min_y: bounds.min_y.min(point[1]),
        max_y: bounds.max_y.max(point[1]),
    }
}

fn sample_contour(contour: &CurveContour<CurvePoint>) -> Vec<CurvePoint> {
    let mut points = vec![contour.start];
    let mut start = contour.start;

    for segment in &contour.segments {
        match segment {
            CurveSegment::Line { to } => {
                points.push(*to);
                start = *to;
            }
            CurveSegment::Quad { ctrl, to } => {
                push_samples(&mut points, 18, |t| quad_point(start, *ctrl, *to, t));
                start = *to;
            }
            CurveSegment::Cubic { ctrl0, ctrl1, to } => {
                push_samples(&mut points, 28, |t| {
                    cubic_point(start, *ctrl0, *ctrl1, *to, t)
                });
                start = *to;
            }
            CurveSegment::Arc { arc } => {
                push_samples(&mut points, 32, |t| arc_point(arc, t));
                start = arc_point(arc, 1.0);
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

fn arc_point(arc: &EllipticArc<CurvePoint>, t: f32) -> CurvePoint {
    let angle = arc.start_angle + arc.sweep_angle * t;
    let local = [arc.radii[0] * angle.cos(), arc.radii[1] * angle.sin()];
    let rotation_cos = arc.rotation.cos();
    let rotation_sin = arc.rotation.sin();

    [
        arc.center[0] + local[0] * rotation_cos - local[1] * rotation_sin,
        arc.center[1] + local[0] * rotation_sin + local[1] * rotation_cos,
    ]
}

fn main() -> eframe::Result<()> {
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("CurveOverlay Boolean")
            .with_inner_size(Vec2::new(1040.0, 760.0)),
        ..eframe::NativeOptions::default()
    };

    eframe::run_native(
        "CurveOverlay Boolean",
        native_options,
        Box::new(|_cc| Ok(Box::new(BoolApp::default()))),
    )
}
