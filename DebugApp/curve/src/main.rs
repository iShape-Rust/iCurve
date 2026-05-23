mod examples;

use crate::examples::{CurveExample, load_examples};
use debug_ui::{
    camera::Camera,
    curve::{CurveEditor, CurvePointReadout, DebugCurve, paint_curve_point_readout},
    egui::{self, Color32, Painter, Pos2, Rect, Sense, Shape, Stroke, Vec2},
    grid::{Grid, paint_camera_readout},
};
use i_curve::flatten::{
    approx::{LineApproximation, LineApproximationSplit},
    segment::{CubicSegment, LineSegment, QuadSegment},
    split::SplitAt,
};

type CurvePoint = [f32; 2];

struct CurveApp {
    camera: Camera,
    grid: Grid,
    examples: Vec<CurveExample>,
    active_example: usize,
    curve: DebugCurve,
    active_point: Option<CurvePointReadout>,
    load_error: Option<String>,
    angle_deg: f32,
    split_parameter: f32,
    max_split_level: u32,
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
            angle_deg: 12.0,
            split_parameter: 0.5,
            max_split_level: 6,
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
                ui.add(egui::Slider::new(&mut self.angle_deg, 0.0..=50.0).text("Angle deg"));
                ui.add(egui::Slider::new(&mut self.split_parameter, 0.01..=0.99).text("Split t"));
                ui.add(egui::Slider::new(&mut self.max_split_level, 1..=10).text("Max level"));

                let subsegment_count = self.final_subsegments().len();
                ui.label(format!("Subsegments: {subsegment_count}"));

                if matches!(self.curve, DebugCurve::Arc(_)) {
                    ui.colored_label(Color32::from_rgb(240, 118, 118), "Arc skipped");
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

                let subsegments = self.final_subsegments();
                paint_final_subsegments(&painter, rect, &self.camera, &subsegments);
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

    fn approximation(&self) -> LineApproximation<f32> {
        LineApproximation {
            min_cos: self.angle_deg.to_radians().cos(),
            min_segment_sqr_length: 4.0,
        }
    }

    fn final_subsegments(&self) -> Vec<FinalSegment> {
        let approximation = self.approximation();
        let split_parameter = self.split_parameter.clamp(0.01, 0.99);
        let mut subsegments = Vec::new();

        match &self.curve {
            DebugCurve::Line(points) => {
                subsegments.push(FinalSegment::Line(LineSegment {
                    control_points: points.map(pos_to_curve_point),
                }));
            }
            DebugCurve::Quad(points) => {
                split_quad_segment(
                    QuadSegment {
                        control_points: points.map(pos_to_curve_point),
                    },
                    approximation,
                    split_parameter,
                    0,
                    self.max_split_level,
                    &mut subsegments,
                );
            }
            DebugCurve::Cubic(points) => {
                split_cubic_segment(
                    CubicSegment {
                        control_points: points.map(pos_to_curve_point),
                    },
                    approximation,
                    split_parameter,
                    0,
                    self.max_split_level,
                    &mut subsegments,
                );
            }
            DebugCurve::Arc(_) => {}
        }

        subsegments
    }
}

enum FinalSegment {
    Line(LineSegment<CurvePoint>),
    Quad(QuadSegment<CurvePoint>),
    Cubic(CubicSegment<CurvePoint>),
}

fn split_quad_segment(
    segment: QuadSegment<CurvePoint>,
    approximation: LineApproximation<f32>,
    split_parameter: f32,
    level: u32,
    max_level: u32,
    subsegments: &mut Vec<FinalSegment>,
) {
    if level < max_level && segment.is_split_required(approximation) {
        let [a, b] = segment.split_at(split_parameter);
        split_quad_segment(
            a,
            approximation,
            split_parameter,
            level + 1,
            max_level,
            subsegments,
        );
        split_quad_segment(
            b,
            approximation,
            split_parameter,
            level + 1,
            max_level,
            subsegments,
        );
    } else {
        subsegments.push(FinalSegment::Quad(segment));
    }
}

fn split_cubic_segment(
    segment: CubicSegment<CurvePoint>,
    approximation: LineApproximation<f32>,
    split_parameter: f32,
    level: u32,
    max_level: u32,
    subsegments: &mut Vec<FinalSegment>,
) {
    if level < max_level && segment.is_split_required(approximation) {
        let [a, b] = segment.split_at(split_parameter);
        split_cubic_segment(
            a,
            approximation,
            split_parameter,
            level + 1,
            max_level,
            subsegments,
        );
        split_cubic_segment(
            b,
            approximation,
            split_parameter,
            level + 1,
            max_level,
            subsegments,
        );
    } else {
        subsegments.push(FinalSegment::Cubic(segment));
    }
}

fn paint_final_subsegments(
    painter: &Painter,
    rect: Rect,
    camera: &Camera,
    subsegments: &[FinalSegment],
) {
    let curve_stroke = Stroke::new(2.0, Color32::from_rgb(120, 224, 166));
    let handle_stroke = Stroke::new(1.0, Color32::from_rgba_premultiplied(255, 206, 102, 150));
    let point_fill = Color32::from_rgba_premultiplied(120, 224, 166, 210);
    let handle_fill = Color32::from_rgba_premultiplied(255, 206, 102, 180);
    let point_stroke = Stroke::new(1.0, Color32::from_rgb(18, 20, 24));

    for subsegment in subsegments {
        match subsegment {
            FinalSegment::Line(segment) => {
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
            FinalSegment::Quad(segment) => {
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
            FinalSegment::Cubic(segment) => {
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

fn is_control_point(index: usize, count: usize) -> bool {
    count > 2 && index > 0 && index < count - 1
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
            .with_title("Curve Debug")
            .with_inner_size(Vec2::new(960.0, 720.0)),
        ..eframe::NativeOptions::default()
    };

    eframe::run_native(
        "Curve Debug",
        native_options,
        Box::new(|_cc| Ok(Box::new(CurveApp::default()))),
    )
}
