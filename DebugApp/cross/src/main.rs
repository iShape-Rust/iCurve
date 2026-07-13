mod examples;

use crate::examples::{CrossCurve, CrossExample, Point, load_examples};
use debug_ui::{
    camera::Camera,
    curve::{CurveEditor, CurvePointReadout, CurveStyle, DebugCurve, paint_curve_point_readout},
    egui::{self, Align2, Color32, FontId, Painter, Pos2, Rect, Sense, Stroke, Vec2},
    grid::{Grid, paint_camera_readout},
};
use i_curve::kernel::int::cross::intersector::{ContactPoint, ContactType};
use i_curve::kernel::int::curve::cubic::CubicSegment;
use i_curve::kernel::int::curve::line::LineSegment;
use i_curve::kernel::int::curve::param::SegmentParam;
use i_curve::kernel::int::curve::quad::QuadSegment;
use i_curve::kernel::int::curve::segment::Segment;

type CurvePoint = [f32; 2];
type Contact = ContactPoint<i32>;

struct CrossApp {
    camera: Camera,
    grid: Grid,
    examples: Vec<CrossExample>,
    active_example: usize,
    curve_a: CrossCurve,
    curve_b: CrossCurve,
    contacts: Result<Vec<Contact>, String>,
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
            contacts: Ok(Vec::new()),
            active_point: None,
        };
        app.refresh_contacts();
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

                ui.label(format!("Curve A: {}", self.curve_a.kind_name()));
                ui.label(format!("Curve B: {}", self.curve_b.kind_name()));

                match &self.contacts {
                    Ok(contacts) => {
                        let crosses = contacts
                            .iter()
                            .filter(|contact| contact.contact_type == ContactType::Cross)
                            .count();
                        let tangents = contacts.len() - crosses;
                        ui.label(format!("Contacts: {}", contacts.len()));
                        ui.label(format!("Cross: {crosses}"));
                        ui.label(format!("Tangent: {tangents}"));
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

                let curve_a = CrossCurve::from_debug_curve(debug_a).expect("curve a kind");
                let curve_b = CrossCurve::from_debug_curve(debug_b).expect("curve b kind");
                if curve_a != self.curve_a || curve_b != self.curve_b {
                    self.curve_a = curve_a;
                    self.curve_b = curve_b;
                    self.refresh_contacts();
                }
                self.active_point = response_a.active_point.or(response_b.active_point);

                if let Ok(contacts) = &self.contacts {
                    paint_contacts(&painter, rect, &self.camera, contacts);
                    paint_contact_readout(&painter, rect, contacts);
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
            self.refresh_contacts();
            self.fit_active_example();
        }
    }

    fn refresh_contacts(&mut self) {
        let curve_a = self.curve_a;
        let curve_b = self.curve_b;

        print_intersection_input(curve_a, curve_b);
        self.contacts =
            std::panic::catch_unwind(move || curve_a.to_segment().intersect(curve_b.to_segment()))
                .map_err(panic_message);
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

fn panic_message(payload: Box<dyn std::any::Any + Send>) -> String {
    let message = payload
        .downcast_ref::<&str>()
        .copied()
        .or_else(|| payload.downcast_ref::<String>().map(String::as_str))
        .unwrap_or("unknown panic");

    format!("Intersection panic: {message}")
}

impl CrossCurve {
    fn control_points(&self) -> &[Point] {
        match self {
            Self::Line(points) => points,
            Self::Quad(points) => points,
            Self::Cubic(points) => points,
        }
    }

    fn chord(&self) -> [Point; 2] {
        let points = self.control_points();
        [points[0], points[points.len() - 1]]
    }

    fn to_debug_curve(&self) -> DebugCurve {
        match self {
            Self::Line(points) => DebugCurve::Line(points.map(point_to_pos)),
            Self::Quad(points) => DebugCurve::Quad(points.map(point_to_pos)),
            Self::Cubic(points) => DebugCurve::Cubic(points.map(point_to_pos)),
        }
    }

    fn from_debug_curve(curve: DebugCurve) -> Option<Self> {
        match curve {
            DebugCurve::Line(points) => Some(Self::Line(points.map(pos_to_point))),
            DebugCurve::Quad(points) => Some(Self::Quad(points.map(pos_to_point))),
            DebugCurve::Cubic(points) => Some(Self::Cubic(points.map(pos_to_point))),
            DebugCurve::Arc(_) => None,
        }
    }

    fn to_segment(&self) -> Segment<i32> {
        match self {
            Self::Line(points) => Segment::Line(LineSegment {
                control_points: *points,
            }),
            Self::Quad(points) => Segment::Quad(QuadSegment {
                control_points: *points,
            }),
            Self::Cubic(points) => Segment::Cubic(CubicSegment {
                control_points: *points,
            }),
        }
    }

    fn sample_points(&self) -> Vec<CurvePoint> {
        match self {
            Self::Line(points) => points.map(point_to_curve_point).into(),
            Self::Quad(points) => {
                let points = points.map(point_to_curve_point);
                let mut samples = Vec::with_capacity(33);
                for index in 0..=32 {
                    let t = index as f32 / 32.0;
                    samples.push(quad_point(points[0], points[1], points[2], t));
                }
                samples
            }
            Self::Cubic(points) => {
                let points = points.map(point_to_curve_point);
                let mut samples = Vec::with_capacity(65);
                for index in 0..=64 {
                    let t = index as f32 / 64.0;
                    samples.push(cubic_point(points[0], points[1], points[2], points[3], t));
                }
                samples
            }
        }
    }
}

fn print_intersection_input(curve_a: CrossCurve, curve_b: CrossCurve) {
    println!("\nintersection input");
    print_curve_input("A", curve_a);
    print_curve_input("B", curve_b);
}

fn print_curve_input(label: &str, curve: CrossCurve) {
    let [start, end] = curve.chord();
    let dx = end.x - start.x;
    let dy = end.y - start.y;

    println!(
        "{label} {} control points: {:?}",
        curve.kind_name(),
        curve.control_points()
    );
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

fn paint_contacts(painter: &Painter, rect: Rect, camera: &Camera, contacts: &[Contact]) {
    for contact in contacts {
        let color = contact_color(contact.contact_type);
        let screen = camera.screen_from_world(rect, point_to_pos(contact.point));
        painter.circle_filled(screen, 7.0, Color32::from_rgb(255, 255, 255));
        painter.circle_stroke(screen, 7.0, Stroke::new(2.5, color));
        painter.circle_filled(screen, 2.5, color);
    }
}

fn paint_contact_readout(painter: &Painter, rect: Rect, contacts: &[Contact]) {
    let text = if contacts.is_empty() {
        "contacts: none".to_owned()
    } else {
        contacts
            .iter()
            .enumerate()
            .map(|(index, contact)| {
                format!(
                    "p{index} ({}, {}) {} t0={:.4} t1={:.4}",
                    contact.point.x,
                    contact.point.y,
                    contact_kind(contact.contact_type),
                    param_value(contact.t0),
                    param_value(contact.t1),
                )
            })
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

fn contact_kind(contact_type: ContactType) -> &'static str {
    match contact_type {
        ContactType::Cross => "cross",
        ContactType::Tangent => "tangent",
    }
}

fn contact_color(contact_type: ContactType) -> Color32 {
    match contact_type {
        ContactType::Cross => Color32::from_rgb(255, 83, 83),
        ContactType::Tangent => Color32::from_rgb(255, 206, 102),
    }
}

fn param_value(param: SegmentParam<i32>) -> f64 {
    param.value() as f64 / SegmentParam::<i32>::DENOMINATOR as f64
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

fn pos_to_point(point: Pos2) -> Point {
    Point::new(point.x.round() as i32, point.y.round() as i32)
}

fn point_to_pos(point: Point) -> Pos2 {
    Pos2::new(point.x as f32, point.y as f32)
}

fn point_to_curve_point(point: Point) -> CurvePoint {
    [point.x as f32, point.y as f32]
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn examples_use_integer_segment_intersection() {
        let examples = load_examples();

        let cross = examples[0]
            .curve_a
            .to_segment()
            .intersect(examples[0].curve_b.to_segment());
        assert_eq!(cross.len(), 1);
        assert_eq!(cross[0].contact_type, ContactType::Cross);

        let mut app = CrossApp::default();
        for example in examples {
            app.curve_a = example.curve_a;
            app.curve_b = example.curve_b;
            app.refresh_contacts();
        }
    }
}

fn main() -> eframe::Result<()> {
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("Integer Curve Cross")
            .with_inner_size(Vec2::new(1040.0, 760.0)),
        ..eframe::NativeOptions::default()
    };

    eframe::run_native(
        "Integer Curve Cross",
        native_options,
        Box::new(|_cc| Ok(Box::new(CrossApp::default()))),
    )
}
