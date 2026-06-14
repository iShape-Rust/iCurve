mod examples;

use crate::examples::{CrossCurve, CrossExample, load_examples};
use debug_ui::{
    camera::Camera,
    curve::{CurveEditor, CurvePointReadout, CurveStyle, DebugCurve, paint_curve_point_readout},
    egui::{self, Align2, Color32, FontId, Painter, Pos2, Rect, Sense, Stroke, Vec2},
    grid::{Grid, paint_camera_readout},
};
use i_curve::curve::{builder::CurveBuilder, path::CurvePath};

type CurvePoint = [f32; 2];

struct CrossApp {
    camera: Camera,
    grid: Grid,
    examples: Vec<CrossExample>,
    active_example: usize,
    curve_a: CrossCurve,
    curve_b: CrossCurve,
    active_point: Option<CurvePointReadout>,
}

impl Default for CrossApp {
    fn default() -> Self {
        let examples = load_examples();
        let first = examples.first().expect("cross examples").clone();
        let mut app = Self {
            camera: Camera {
                zoom: 1.25,
                ..Camera::default()
            },
            grid: Grid::default(),
            examples,
            active_example: 0,
            curve_a: first.curve_a,
            curve_b: first.curve_b,
            active_point: None,
        };
        app.fit_active_example();
        app
    }
}

impl eframe::App for CrossApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        egui::Panel::left("cross_panel")
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
                    self.select_example(index);
                }

                ui.add_space(8.0);
                ui.separator();

                let result = self.intersection_points();
                ui.label(format!("Curve A: {}", self.curve_a.kind_name()));
                ui.label(format!("Curve B: {}", self.curve_b.kind_name()));

                match &result {
                    Ok(points) => {
                        ui.label(format!("Intersections: {}", points.len()));
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

                let mut debug_a = self.curve_a.to_debug_curve();
                let mut debug_b = self.curve_b.to_debug_curve();

                let response_a = CurveEditor::new("curve_a")
                    .with_style(curve_a_style())
                    .show(ui, &painter, rect, &self.camera, &mut debug_a);
                let response_b = CurveEditor::new("curve_b")
                    .with_style(curve_b_style())
                    .show(ui, &painter, rect, &self.camera, &mut debug_b);

                self.curve_a = CrossCurve::from_debug_curve(debug_a).expect("curve a kind");
                self.curve_b = CrossCurve::from_debug_curve(debug_b).expect("curve b kind");
                self.active_point = response_a.active_point.or(response_b.active_point);

                if let Ok(points) = self.intersection_points() {
                    paint_intersection_points(&painter, rect, &self.camera, &points);
                    paint_intersection_readout(&painter, rect, &points);
                }

                paint_camera_readout(&painter, rect, &self.camera);

                if let Some(active_point) = self.active_point {
                    paint_curve_point_readout(&painter, rect, active_point);
                }
            });
    }
}

impl CrossApp {
    fn select_example(&mut self, index: usize) {
        if let Some(example) = self.examples.get(index).cloned() {
            self.active_example = index;
            self.curve_a = example.curve_a;
            self.curve_b = example.curve_b;
            self.active_point = None;
            self.fit_active_example();
        }
    }

    fn intersection_points(&self) -> Result<Vec<CurvePoint>, String> {
        let path_a = self
            .curve_a
            .build_path()
            .map_err(|error| format!("{error:?}"))?;
        let path_b = self
            .curve_b
            .build_path()
            .map_err(|error| format!("{error:?}"))?;

        path_a
            .try_intersection_points(&path_b)
            .map(|points| points.into_iter().map(|point| [point.x, point.y]).collect())
            .map_err(|error| format!("{error:?}"))
    }

    fn fit_active_example(&mut self) {
        let Some(bounds) = Bounds::from_curves(&self.curve_a, &self.curve_b) else {
            return;
        };

        self.camera.center = Pos2::new(
            (bounds.min_x + bounds.max_x) * 0.5,
            (bounds.min_y + bounds.max_y) * 0.5,
        );
    }
}

impl CrossCurve {
    fn to_debug_curve(&self) -> DebugCurve {
        match self {
            Self::Line(points) => DebugCurve::Line(*points),
            Self::Quad(points) => DebugCurve::Quad(*points),
        }
    }

    fn from_debug_curve(curve: DebugCurve) -> Option<Self> {
        match curve {
            DebugCurve::Line(points) => Some(Self::Line(points)),
            DebugCurve::Quad(points) => Some(Self::Quad(points)),
            DebugCurve::Cubic(_) | DebugCurve::Arc(_) => None,
        }
    }

    fn build_path(&self) -> Result<CurvePath<CurvePoint>, i_curve::curve::builder::CurveError> {
        match self {
            Self::Line(points) => CurveBuilder::new()
                .move_to(pos_to_curve_point(points[0]))?
                .line_to(pos_to_curve_point(points[1]))?
                .build_path(),
            Self::Quad(points) => CurveBuilder::new()
                .move_to(pos_to_curve_point(points[0]))?
                .quad_to(pos_to_curve_point(points[1]), pos_to_curve_point(points[2]))?
                .build_path(),
        }
    }

    fn sample_points(&self) -> Vec<CurvePoint> {
        match self {
            Self::Line(points) => points.map(pos_to_curve_point).into(),
            Self::Quad(points) => {
                let points = points.map(pos_to_curve_point);
                let mut samples = Vec::with_capacity(33);
                for index in 0..=32 {
                    let t = index as f32 / 32.0;
                    samples.push(quad_point(points[0], points[1], points[2], t));
                }
                samples
            }
        }
    }
}

fn curve_a_style() -> CurveStyle {
    CurveStyle {
        curve_stroke: Stroke::new(3.0, Color32::from_rgb(86, 196, 255)),
        arm_stroke: Stroke::new(1.5, Color32::from_rgb(124, 132, 146)),
        point_fill: Color32::from_rgb(255, 206, 102),
        control_fill: Color32::from_rgb(240, 118, 118),
        ..CurveStyle::default()
    }
}

fn curve_b_style() -> CurveStyle {
    CurveStyle {
        curve_stroke: Stroke::new(3.0, Color32::from_rgb(128, 212, 156)),
        arm_stroke: Stroke::new(1.5, Color32::from_rgb(94, 150, 130)),
        point_fill: Color32::from_rgb(182, 221, 255),
        control_fill: Color32::from_rgb(255, 181, 79),
        ..CurveStyle::default()
    }
}

fn paint_intersection_points(
    painter: &Painter,
    rect: Rect,
    camera: &Camera,
    points: &[CurvePoint],
) {
    for point in points {
        let screen = camera.screen_from_world(rect, point_to_pos(*point));
        painter.circle_filled(screen, 7.0, Color32::from_rgb(255, 255, 255));
        painter.circle_stroke(
            screen,
            7.0,
            Stroke::new(2.5, Color32::from_rgb(255, 83, 83)),
        );
        painter.circle_filled(screen, 2.5, Color32::from_rgb(255, 83, 83));
    }
}

fn paint_intersection_readout(painter: &Painter, rect: Rect, points: &[CurvePoint]) {
    let text = if points.is_empty() {
        "intersections: none".to_owned()
    } else {
        points
            .iter()
            .enumerate()
            .map(|(index, point)| format!("p{} ({:.2}, {:.2})", index, point[0], point[1]))
            .collect::<Vec<_>>()
            .join("  ")
    };

    painter.text(
        rect.right_bottom() + Vec2::new(-12.0, -10.0),
        Align2::RIGHT_BOTTOM,
        text,
        FontId::monospace(12.0),
        Color32::from_rgb(224, 228, 236),
    );
}

#[derive(Clone, Copy)]
struct Bounds {
    min_x: f32,
    max_x: f32,
    min_y: f32,
    max_y: f32,
}

impl Bounds {
    fn from_curves(curve_a: &CrossCurve, curve_b: &CrossCurve) -> Option<Self> {
        curve_a
            .sample_points()
            .into_iter()
            .chain(curve_b.sample_points())
            .fold(None, |bounds, point| {
                Some(match bounds {
                    Some(bounds) => bounds.add_point(point),
                    None => Self {
                        min_x: point[0],
                        max_x: point[0],
                        min_y: point[1],
                        max_y: point[1],
                    },
                })
            })
    }

    fn add_point(self, point: CurvePoint) -> Self {
        Self {
            min_x: self.min_x.min(point[0]),
            max_x: self.max_x.max(point[0]),
            min_y: self.min_y.min(point[1]),
            max_y: self.max_y.max(point[1]),
        }
    }
}

fn pos_to_curve_point(point: Pos2) -> CurvePoint {
    [point.x, point.y]
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

fn main() -> eframe::Result<()> {
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("Curve Cross")
            .with_inner_size(Vec2::new(1040.0, 760.0)),
        ..eframe::NativeOptions::default()
    };

    eframe::run_native(
        "Curve Cross",
        native_options,
        Box::new(|_cc| Ok(Box::new(CrossApp::default()))),
    )
}
