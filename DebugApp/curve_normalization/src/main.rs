mod examples;

use crate::examples::{load_examples, CurveExample};
use debug_ui::{
    camera::Camera,
    curve::{paint_curve_point_readout, ArcCurve, CurveEditor, CurvePointReadout, DebugCurve},
    egui::{self, Color32, Painter, Pos2, Rect, Sense, Shape, Stroke, Vec2},
    grid::{paint_camera_readout, Grid},
};
use i_curve::{
    curve::{
        arc::EllipticArc,
        builder::{CurveError, CurveShapeBuilder},
        shape::CurveShape,
    },
    flatten::{
        convert::ShapeToSegments,
        segment::{ArcSegment, NormalizedSegment, Segment},
    },
};
use i_overlay::core::overlay::ShapeType;

type CurvePoint = [f32; 2];

struct CurveApp {
    camera: Camera,
    grid: Grid,
    examples: Vec<CurveExample>,
    active_example: usize,
    curve: DebugCurve,
    active_point: Option<CurvePointReadout>,
    load_error: Option<String>,
}

impl Default for CurveApp {
    fn default() -> Self {
        let loaded = load_examples();
        let curve = loaded
            .examples
            .first()
            .map(|example| example.curve.clone())
            .unwrap_or_default();
        let active_point = curve.first_point();

        Self {
            camera: Camera {
                zoom: 1.35,
                ..Camera::default()
            },
            grid: Grid::default(),
            examples: loaded.examples,
            active_example: 0,
            curve,
            active_point,
            load_error: loaded.error,
        }
    }
}

impl eframe::App for CurveApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        egui::Panel::left("curves_panel")
            .resizable(false)
            .default_size(220.0)
            .frame(egui::Frame::default().fill(Color32::from_rgb(24, 27, 32)))
            .show_inside(ui, |ui| {
                ui.spacing_mut().item_spacing = Vec2::new(6.0, 6.0);
                ui.add_space(8.0);

                let mut selected = None;

                for (index, example) in self.examples.iter().enumerate() {
                    if ui
                        .selectable_label(index == self.active_example, &example.name)
                        .clicked()
                    {
                        selected = Some(index);
                    }
                }

                if let Some(index) = selected {
                    self.select_example(index);
                }

                ui.add_space(8.0);
                ui.separator();

                match self.normalized_segments() {
                    Ok(segments) => {
                        ui.label(format!("Normalized segments: {}", segments.len()));
                    }
                    Err(error) => {
                        ui.colored_label(
                            Color32::from_rgb(240, 118, 118),
                            format!("Shape error: {error:?}"),
                        );
                    }
                }

                if let Some(error) = &self.load_error {
                    ui.add_space(8.0);
                    ui.separator();
                    ui.colored_label(Color32::from_rgb(240, 118, 118), error);
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

                let editor_response = CurveEditor::new("main_curve").show(
                    ui,
                    &painter,
                    rect,
                    &self.camera,
                    &mut self.curve,
                );

                if let Some(active_point) = editor_response.active_point {
                    self.active_point = Some(active_point);
                }

                if let Ok(segments) = self.normalized_segments() {
                    paint_normalized_segments(&painter, rect, &self.camera, &segments);
                }

                paint_camera_readout(&painter, rect, &self.camera);

                if let Some(active_point) = self.active_point {
                    paint_curve_point_readout(&painter, rect, active_point);
                }
            });
    }
}

impl CurveApp {
    fn select_example(&mut self, index: usize) {
        if let Some(example) = self.examples.get(index) {
            self.active_example = index;
            self.curve = example.curve.clone();
            self.active_point = self.curve.first_point();
        }
    }

    fn normalized_segments(&self) -> Result<Vec<Segment<CurvePoint>>, CurveError> {
        Ok(self
            .curve
            .to_curve_shape()?
            .to_normalize_segments(ShapeType::Subject))
    }
}

fn paint_normalized_segments(
    painter: &Painter,
    rect: Rect,
    camera: &Camera,
    segments: &[Segment<CurvePoint>],
) {
    let handle_stroke = Stroke::new(1.0, Color32::from_rgba_premultiplied(255, 206, 102, 145));
    let handle_fill = Color32::from_rgba_premultiplied(255, 206, 102, 180);
    let point_fill = Color32::from_rgba_premultiplied(120, 224, 166, 220);
    let center_fill = Color32::from_rgba_premultiplied(128, 212, 156, 210);
    let point_stroke = Stroke::new(1.0, Color32::from_rgb(18, 20, 24));

    for (index, segment) in segments.iter().enumerate() {
        let curve_stroke = Stroke::new(2.0, segment_color(index));

        match &segment.normalized_segment {
            NormalizedSegment::Line(segment) => {
                let points = segment.control_points.map(curve_point_to_pos);
                paint_control_polygon(painter, rect, camera, &points, handle_stroke);
                paint_sampled_subsegment(painter, rect, camera, 1, curve_stroke, |t| {
                    line_point(points[0], points[1], t)
                });
                paint_passive_points(
                    painter,
                    rect,
                    camera,
                    &points,
                    point_fill,
                    handle_fill,
                    point_stroke,
                );
            }
            NormalizedSegment::Quad(segment) => {
                let points = segment.control_points.map(curve_point_to_pos);
                paint_control_polygon(painter, rect, camera, &points, handle_stroke);
                paint_sampled_subsegment(painter, rect, camera, 14, curve_stroke, |t| {
                    quad_point(points[0], points[1], points[2], t)
                });
                paint_passive_points(
                    painter,
                    rect,
                    camera,
                    &points,
                    point_fill,
                    handle_fill,
                    point_stroke,
                );
            }
            NormalizedSegment::Cubic(segment) => {
                let points = segment.control_points.map(curve_point_to_pos);
                paint_control_polygon(painter, rect, camera, &points, handle_stroke);
                paint_sampled_subsegment(painter, rect, camera, 18, curve_stroke, |t| {
                    cubic_point(points[0], points[1], points[2], points[3], t)
                });
                paint_passive_points(
                    painter,
                    rect,
                    camera,
                    &points,
                    point_fill,
                    handle_fill,
                    point_stroke,
                );
            }
            NormalizedSegment::Arc(segment) => {
                paint_arc_handles(painter, rect, camera, segment, handle_stroke);
                paint_sampled_subsegment(painter, rect, camera, 36, curve_stroke, |t| {
                    arc_point(segment, t)
                });
                paint_arc_points(
                    painter,
                    rect,
                    camera,
                    segment,
                    point_fill,
                    center_fill,
                    point_stroke,
                );
            }
        }
    }
}

fn paint_sampled_subsegment(
    painter: &Painter,
    rect: Rect,
    camera: &Camera,
    sample_count: usize,
    stroke: Stroke,
    sample: impl Fn(f32) -> Pos2,
) {
    let mut points = Vec::with_capacity(sample_count + 1);

    for index in 0..=sample_count {
        let t = index as f32 / sample_count as f32;
        points.push(camera.screen_from_world(rect, sample(t)));
    }

    painter.add(Shape::line(points, stroke));
}

fn paint_control_polygon<const N: usize>(
    painter: &Painter,
    rect: Rect,
    camera: &Camera,
    points: &[Pos2; N],
    stroke: Stroke,
) {
    for pair in points.windows(2) {
        painter.line_segment(
            [
                camera.screen_from_world(rect, pair[0]),
                camera.screen_from_world(rect, pair[1]),
            ],
            stroke,
        );
    }
}

fn paint_passive_points<const N: usize>(
    painter: &Painter,
    rect: Rect,
    camera: &Camera,
    points: &[Pos2; N],
    point_fill: Color32,
    handle_fill: Color32,
    stroke: Stroke,
) {
    for (index, point) in points.iter().copied().enumerate() {
        let screen_pos = camera.screen_from_world(rect, point);
        let fill = if is_control_point(index, N) {
            handle_fill
        } else {
            point_fill
        };

        painter.circle_filled(screen_pos, 3.5, fill);
        painter.circle_stroke(screen_pos, 3.5, stroke);
    }
}

fn paint_arc_handles(
    painter: &Painter,
    rect: Rect,
    camera: &Camera,
    segment: &ArcSegment<CurvePoint>,
    stroke: Stroke,
) {
    let center = curve_point_to_pos(segment.center);
    let p0 = curve_point_to_pos(segment.p0);
    let p1 = curve_point_to_pos(segment.p1);

    painter.line_segment(
        [
            camera.screen_from_world(rect, center),
            camera.screen_from_world(rect, p0),
        ],
        stroke,
    );
    painter.line_segment(
        [
            camera.screen_from_world(rect, center),
            camera.screen_from_world(rect, p1),
        ],
        stroke,
    );
}

fn paint_arc_points(
    painter: &Painter,
    rect: Rect,
    camera: &Camera,
    segment: &ArcSegment<CurvePoint>,
    point_fill: Color32,
    center_fill: Color32,
    stroke: Stroke,
) {
    for (point, fill) in [
        (segment.p0, point_fill),
        (segment.center, center_fill),
        (segment.p1, point_fill),
    ] {
        let screen_pos = camera.screen_from_world(rect, curve_point_to_pos(point));
        painter.circle_filled(screen_pos, 3.5, fill);
        painter.circle_stroke(screen_pos, 3.5, stroke);
    }
}

fn segment_color(index: usize) -> Color32 {
    const COLORS: [Color32; 4] = [
        Color32::from_rgb(120, 224, 166),
        Color32::from_rgb(86, 196, 255),
        Color32::from_rgb(240, 118, 118),
        Color32::from_rgb(196, 160, 255),
    ];

    COLORS[index % COLORS.len()]
}

fn pos_to_curve_point(point: Pos2) -> CurvePoint {
    [point.x, point.y]
}

fn curve_point_to_pos(point: CurvePoint) -> Pos2 {
    Pos2::new(point[0], point[1])
}

fn line_point(p0: Pos2, p1: Pos2, t: f32) -> Pos2 {
    p0 + (p1 - p0) * t
}

fn quad_point(p0: Pos2, p1: Pos2, p2: Pos2, t: f32) -> Pos2 {
    let a = line_point(p0, p1, t);
    let b = line_point(p1, p2, t);
    line_point(a, b, t)
}

fn cubic_point(p0: Pos2, p1: Pos2, p2: Pos2, p3: Pos2, t: f32) -> Pos2 {
    let a = quad_point(p0, p1, p2, t);
    let b = quad_point(p1, p2, p3, t);
    line_point(a, b, t)
}

fn arc_point(segment: &ArcSegment<CurvePoint>, t: f32) -> Pos2 {
    let angle = segment.start_angle + segment.sweep_angle * t;
    let x = segment.radii[0] * angle.cos();
    let y = segment.radii[1] * angle.sin();
    let cos = segment.rotation.cos();
    let sin = segment.rotation.sin();

    Pos2::new(
        segment.center[0] + x * cos - y * sin,
        segment.center[1] + x * sin + y * cos,
    )
}

fn is_control_point(index: usize, count: usize) -> bool {
    count > 2 && index > 0 && index < count - 1
}

trait DebugCurveToShape {
    fn to_curve_shape(&self) -> Result<CurveShape<CurvePoint>, CurveError>;
}

impl DebugCurveToShape for DebugCurve {
    fn to_curve_shape(&self) -> Result<CurveShape<CurvePoint>, CurveError> {
        match self {
            Self::Line(points) => CurveShapeBuilder::new()
                .move_to(pos_to_curve_point(points[0]))?
                .line_to(pos_to_curve_point(points[1]))?
                .close_with_line()?
                .build(),
            Self::Quad(points) => CurveShapeBuilder::new()
                .move_to(pos_to_curve_point(points[0]))?
                .quad_to(pos_to_curve_point(points[1]), pos_to_curve_point(points[2]))?
                .close_with_line()?
                .build(),
            Self::Cubic(points) => CurveShapeBuilder::new()
                .move_to(pos_to_curve_point(points[0]))?
                .cubic_to(
                    pos_to_curve_point(points[1]),
                    pos_to_curve_point(points[2]),
                    pos_to_curve_point(points[3]),
                )?
                .close_with_line()?
                .build(),
            Self::Arc(arc) => arc_to_curve_shape(*arc),
        }
    }
}

fn arc_to_curve_shape(arc: ArcCurve) -> Result<CurveShape<CurvePoint>, CurveError> {
    CurveShapeBuilder::new()
        .move_to(pos_to_curve_point(arc.point_at(arc.start_angle)))?
        .arc_to(EllipticArc {
            center: pos_to_curve_point(arc.center),
            radii: [arc.radius, arc.radius],
            rotation: 0.0,
            start_angle: arc.start_angle,
            sweep_angle: arc.sweep_angle,
        })?
        .close_with_line()?
        .build()
}

trait CurveReadout {
    fn first_point(&self) -> Option<CurvePointReadout>;
}

impl CurveReadout for DebugCurve {
    fn first_point(&self) -> Option<CurvePointReadout> {
        match self {
            Self::Line(points) => Some(CurvePointReadout {
                label: "start",
                position: points[0],
            }),
            Self::Quad(points) => Some(CurvePointReadout {
                label: "start",
                position: points[0],
            }),
            Self::Cubic(points) => Some(CurvePointReadout {
                label: "start",
                position: points[0],
            }),
            Self::Arc(arc) => Some(CurvePointReadout {
                label: "center",
                position: arc.center,
            }),
        }
    }
}

fn main() -> eframe::Result<()> {
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("Curve Normalization")
            .with_inner_size(Vec2::new(960.0, 720.0)),
        ..eframe::NativeOptions::default()
    };

    eframe::run_native(
        "Curve Normalization",
        native_options,
        Box::new(|_cc| Ok(Box::new(CurveApp::default()))),
    )
}
