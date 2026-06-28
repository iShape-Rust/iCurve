mod examples;

use crate::examples::{CubicExample, Point, examples};
use debug_ui::{
    camera::Camera,
    egui::{
        self, Align2, Color32, CursorIcon, FontId, Id, Painter, Pos2, Rect, Sense, Shape, Stroke,
        Vec2,
    },
    grid::{Grid, paint_camera_readout},
};
use i_curve::int::curve::normalization::{
    NormalizedSegment, cubic_self_intersection, normalize_cubic,
};

struct CubicNormalizationApp {
    camera: Camera,
    grid: Grid,
    examples: Vec<CubicExample>,
    active_example: usize,
    points: [Point; 4],
    active_point: Option<usize>,
}

impl Default for CubicNormalizationApp {
    fn default() -> Self {
        let examples = examples();
        let points = examples
            .first()
            .map(|example| example.points)
            .unwrap_or(default_points());

        Self {
            camera: Camera {
                zoom: 1.35,
                ..Camera::default()
            },
            grid: Grid::default(),
            examples,
            active_example: 0,
            points,
            active_point: None,
        }
    }
}

impl eframe::App for CubicNormalizationApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        egui::Panel::left("cubic_panel")
            .resizable(false)
            .default_size(250.0)
            .frame(egui::Frame::default().fill(Color32::from_rgb(24, 27, 32)))
            .show_inside(ui, |ui| {
                ui.spacing_mut().item_spacing = Vec2::new(6.0, 6.0);
                ui.add_space(8.0);

                let mut selected = None;

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

                let intersection = cubic_self_intersection(self.points);
                let segments = normalize_cubic(self.points);

                ui.label(format!("Normalized segments: {}", segments.len()));
                match intersection {
                    Some(point) => {
                        ui.label(format!("Self intersection: ({}, {})", point.x, point.y));
                    }
                    None => {
                        ui.label("Self intersection: none");
                    }
                }

                ui.add_space(8.0);
                for (index, point) in self.points.iter().enumerate() {
                    ui.label(format!("p{index}: ({}, {})", point.x, point.y));
                }

                ui.add_space(8.0);
                if ui.button("Fit view").clicked() {
                    self.fit_view();
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

                let editor_response = IntCubicEditor::new("cubic").show(
                    ui,
                    &painter,
                    rect,
                    &self.camera,
                    &mut self.points,
                );
                if let Some(index) = editor_response.active_point {
                    self.active_point = Some(index);
                }

                let normalized = normalize_cubic(self.points);
                paint_normalized_segments(&painter, rect, &self.camera, &normalized);

                if let Some(point) = cubic_self_intersection(self.points) {
                    paint_intersection_point(&painter, rect, &self.camera, point);
                }

                paint_camera_readout(&painter, rect, &self.camera);

                if let Some(index) = self.active_point {
                    paint_point_readout(&painter, rect, index, self.points[index]);
                }
            });
    }
}

impl CubicNormalizationApp {
    fn select_example(&mut self, index: usize) {
        if let Some(example) = self.examples.get(index) {
            self.active_example = index;
            self.points = example.points;
            self.active_point = None;
            self.fit_view();
        }
    }

    fn fit_view(&mut self) {
        let mut min_x = self.points[0].x;
        let mut max_x = self.points[0].x;
        let mut min_y = self.points[0].y;
        let mut max_y = self.points[0].y;

        for point in self.points {
            min_x = min_x.min(point.x);
            max_x = max_x.max(point.x);
            min_y = min_y.min(point.y);
            max_y = max_y.max(point.y);
        }

        self.camera.center = Pos2::new((min_x + max_x) as f32 * 0.5, (min_y + max_y) as f32 * 0.5);
    }
}

#[derive(Default)]
struct IntCubicEditorResponse {
    active_point: Option<usize>,
}

struct IntCubicEditor {
    id: Id,
}

impl IntCubicEditor {
    fn new(id_source: impl std::hash::Hash) -> Self {
        Self {
            id: Id::new(id_source),
        }
    }

    fn show(
        &self,
        ui: &mut egui::Ui,
        painter: &Painter,
        rect: Rect,
        camera: &Camera,
        points: &mut [Point; 4],
    ) -> IntCubicEditorResponse {
        let mut interactions = [PointInteraction::default(); 4];

        for index in 0..4 {
            interactions[index] = interact_point(
                ui,
                self.id.with(index),
                rect,
                camera,
                &mut points[index],
                6.0,
            );
        }

        let positions = points.map(point_to_pos);
        paint_control_polygon(
            painter,
            rect,
            camera,
            &positions,
            Stroke::new(1.5, Color32::from_rgb(124, 132, 146)),
        );
        paint_sampled_curve(
            painter,
            rect,
            camera,
            72,
            Stroke::new(3.0, Color32::from_rgb(86, 196, 255)),
            |t| cubic_point(positions[0], positions[1], positions[2], positions[3], t),
        );

        for index in 0..4 {
            let radius = if is_control_point(index) { 5.0 } else { 6.0 };
            let fill = if interactions[index].hovered || interactions[index].dragged {
                Color32::WHITE
            } else if is_control_point(index) {
                Color32::from_rgb(240, 118, 118)
            } else {
                Color32::from_rgb(255, 206, 102)
            };

            painter.circle_filled(
                camera.screen_from_world(rect, positions[index]),
                radius,
                fill,
            );
            painter.circle_stroke(
                camera.screen_from_world(rect, positions[index]),
                radius,
                Stroke::new(1.5, Color32::from_rgb(18, 20, 24)),
            );
        }

        IntCubicEditorResponse {
            active_point: active_point(&interactions),
        }
    }
}

#[derive(Clone, Copy, Debug, Default)]
struct PointInteraction {
    hovered: bool,
    dragged: bool,
}

fn interact_point(
    ui: &mut egui::Ui,
    id: Id,
    rect: Rect,
    camera: &Camera,
    point: &mut Point,
    radius: f32,
) -> PointInteraction {
    let mut world_pos = point_to_pos(*point);
    let screen_pos = camera.screen_from_world(rect, world_pos);
    let hit_rect = Rect::from_center_size(screen_pos, Vec2::splat(radius * 4.0));
    let response = ui
        .interact(hit_rect, id, Sense::drag())
        .on_hover_cursor(CursorIcon::Grab);

    if response.dragged() {
        world_pos += camera.world_delta_from_screen_delta(response.drag_delta());
        *point = Point::new(world_pos.x.round() as i32, world_pos.y.round() as i32);
    }

    PointInteraction {
        hovered: response.hovered(),
        dragged: response.dragged(),
    }
}

fn active_point(interactions: &[PointInteraction; 4]) -> Option<usize> {
    interactions
        .iter()
        .position(|interaction| interaction.dragged)
        .or_else(|| {
            interactions
                .iter()
                .position(|interaction| interaction.hovered)
        })
}

fn paint_normalized_segments(
    painter: &Painter,
    rect: Rect,
    camera: &Camera,
    segments: &[NormalizedSegment<i32>],
) {
    let handle_stroke = Stroke::new(1.0, Color32::from_rgba_premultiplied(255, 206, 102, 145));
    let point_fill = Color32::from_rgba_premultiplied(120, 224, 166, 220);
    let handle_fill = Color32::from_rgba_premultiplied(255, 206, 102, 180);
    let point_stroke = Stroke::new(1.0, Color32::from_rgb(18, 20, 24));

    for (index, segment) in segments.iter().enumerate() {
        let stroke = Stroke::new(2.0, segment_color(index));

        match segment {
            NormalizedSegment::Line(points) => {
                let points = points.map(point_to_pos);
                paint_control_polygon(painter, rect, camera, &points, handle_stroke);
                paint_sampled_curve(painter, rect, camera, 1, stroke, |t| {
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
            NormalizedSegment::Quad(points) => {
                let points = points.map(point_to_pos);
                paint_control_polygon(painter, rect, camera, &points, handle_stroke);
                paint_sampled_curve(painter, rect, camera, 18, stroke, |t| {
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
            NormalizedSegment::Cubic(points) => {
                let points = points.map(point_to_pos);
                paint_control_polygon(painter, rect, camera, &points, handle_stroke);
                paint_sampled_curve(painter, rect, camera, 24, stroke, |t| {
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

fn paint_intersection_point(painter: &Painter, rect: Rect, camera: &Camera, point: Point) {
    let screen = camera.screen_from_world(rect, point_to_pos(point));
    painter.circle_filled(screen, 8.0, Color32::WHITE);
    painter.circle_stroke(
        screen,
        8.0,
        Stroke::new(2.5, Color32::from_rgb(255, 83, 83)),
    );
    painter.circle_filled(screen, 2.5, Color32::from_rgb(255, 83, 83));
}

fn paint_point_readout(painter: &Painter, rect: Rect, index: usize, point: Point) {
    painter.text(
        rect.left_bottom() + Vec2::new(12.0, -10.0),
        Align2::LEFT_BOTTOM,
        format!("p{index} ({}, {})", point.x, point.y),
        FontId::monospace(12.0),
        Color32::from_rgb(196, 202, 214),
    );
}

fn paint_sampled_curve(
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
        let fill = if index > 0 && index < N - 1 {
            handle_fill
        } else {
            point_fill
        };

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

fn point_to_pos(point: Point) -> Pos2 {
    Pos2::new(point.x as f32, point.y as f32)
}

fn default_points() -> [Point; 4] {
    [
        Point::new(-120, 0),
        Point::new(160, -180),
        Point::new(-160, -180),
        Point::new(120, 0),
    ]
}

fn is_control_point(index: usize) -> bool {
    index == 1 || index == 2
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

fn main() -> eframe::Result<()> {
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("Cubic Normalization")
            .with_inner_size(Vec2::new(960.0, 720.0)),
        ..eframe::NativeOptions::default()
    };

    eframe::run_native(
        "Cubic Normalization",
        native_options,
        Box::new(|_cc| Ok(Box::new(CubicNormalizationApp::default()))),
    )
}
