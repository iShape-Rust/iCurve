use i_curve::int::collision::convex_hull::FourConvexPathExt;
use debug_ui::ext::color::Color32Ext;
use debug_ui::util::camera::Camera;
use debug_ui::view::curve::CurveView;
use debug_ui::view::grid::{GridView, Step};
use eframe::egui::{Color32, Pos2, Sense, Stroke};
use eframe::{App, Frame, egui};
use eframe::epaint::Shape;
use i_curve::float::math::point::Point;
use i_curve::int::bezier::spline::{IntBezierSplineApi, SplitPosition};
use i_curve::int::bezier::spline_cube::IntCubeSpline;
use i_curve::int::math::normalize::VectorNormalization16Util;

pub struct EditorApp {
    grid: GridView,
    curve: CurveView,
    camera: Camera,
    cos_value: f64,
    min_len: u32,

    split_value: f64,
    split_power: u32,
}

impl Default for EditorApp {
    fn default() -> Self {
        let mut camera = Camera::empty();
        camera.set_scale(0.02);
        Self {
            grid: GridView::new(vec![
                Step::new(64.0, Color32::RED, 0.5),
                Step::new(4096.0, Color32::ORANGE,0.5),
                Step::new(262144.0, Color32::YELLOW, 0.5),
                Step::new(16777216.0, Color32::GREEN, 0.5),
                Step::new(1073741824.0, Color32::BLUE, 0.5),
            ]),
            curve: CurveView::new([
                Point::new(0.0, 0.0),
                Point::new(0.0, 2048.0),
                Point::new(2048.0, 4096.0),
                Point::new(4096.0, 4096.0),
            ]),
            camera,
            cos_value: 0.95,
            min_len: 16,
            split_value: 0.5,
            split_power: 4,
        }
    }
}

impl App for EditorApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut Frame) {
        egui::TopBottomPanel::top("slider_panel").show(ctx, |ui| {
            ui.add(egui::Slider::new(&mut self.cos_value, 0.9..=1.0).text("Min Cos"));
            ui.add(egui::Slider::new(&mut self.min_len, 4..=4096).text("Min Len"));
            ui.add(egui::Slider::new(&mut self.split_value, 0.0..=1.0).text("Split Value"));
            ui.add(egui::Slider::new(&mut self.split_power, 3..=10).text("Split Factor"));
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            let (rect, response) = ui.allocate_exact_size(ui.available_size(), Sense::drag());
            self.camera.update_view(rect);

            let painter = ui.painter_at(rect);
            self.grid.draw(&painter, &self.camera);

            let view_points: Vec<_> = self.curve.anchors().to_convex_hull().slice().iter().map(|wp|{
                let vp = self.camera.world_to_view(Point::new(wp.x as f64, wp.y as f64));
                Pos2::new(vp.x as f32, vp.y as f32)
            }).collect();

            painter.add(Shape::convex_polygon(
                view_points,
                Color32::LIGHT_YELLOW.with_opacity(0.2),
                Stroke::new(2.0, Color32::YELLOW.with_opacity(0.8)),
            ));

            let min_cos = VectorNormalization16Util::normalize_unit_value(self.cos_value);
            let main_stroke = Stroke::new(4.0, Color32::GRAY);
            let child_stroke = Stroke::new(1.0, Color32::GREEN);
            let (_, dragged) = self.curve.draw_editable(ui, &painter, &self.camera, min_cos, self.min_len, main_stroke, false, 0);

            let main_spline = IntCubeSpline { anchors: self.curve.anchors() };
            let value = ((1 << self.split_power) as f64 * self.split_value).round() as u64;
            let position = SplitPosition { power: self.split_power, value };
            let (spline_a, spline_b) = main_spline.split(&position);
            let anchors_a = spline_a.anchors.map(|p| Point { x: p.x as f64, y: p.y as f64 });
            let anchors_b = spline_b.anchors.map(|p| Point { x: p.x as f64, y: p.y as f64 });
            let curve_a = CurveView::new(anchors_a);
            let curve_b = CurveView::new(anchors_b);

            curve_a.draw(ui, &painter, &self.camera, min_cos, self.min_len, child_stroke, false);
            curve_b.draw(ui, &painter, &self.camera, min_cos, self.min_len, child_stroke, false);

            if !dragged {
                let delta = response.drag_delta();
                self.camera.move_by_view_xy(delta.x as f64, delta.y as f64);
            }

            ctx.input(|i| {
                let scroll = i.smooth_scroll_delta.y as f64;
                if scroll != 0.0 {
                    if let Some(mouse) = i.pointer.hover_pos() {
                        self.camera.zoom_by_view_xy(mouse.x as f64, mouse.y as f64, scroll);
                    }
                }
            });
        });
    }
}
