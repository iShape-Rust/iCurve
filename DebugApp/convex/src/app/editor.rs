use debug_ui::util::camera::Camera;
use debug_ui::view::curve::CurveView;
use debug_ui::view::grid::{GridView, Step};
use eframe::egui::{Color32, Sense};
use eframe::{App, Frame, egui};
use i_curve::float::math::point::Point;
use i_curve::int::convex::builder::FourConvexBuilder;
use i_curve::int::math::normalize::normalize_unit_value;

pub struct EditorApp {
    grid: GridView,
    curve: CurveView,
    camera: Camera,
    cos_value: f64,
    min_len: u32,
    segments_count: usize
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
            segments_count: 0
        }
    }
}

impl App for EditorApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut Frame) {
        egui::TopBottomPanel::top("slider_panel").show(ctx, |ui| {
            ui.add(egui::Slider::new(&mut self.cos_value, 0.9..=1.0).text("Min Cos"));
            ui.add(egui::Slider::new(&mut self.min_len, 4..=4096).text("Min Len"));
            ui.label(format!("Segments count: {}", self.segments_count));
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            let (rect, response) = ui.allocate_exact_size(ui.available_size(), Sense::drag());
            self.camera.update_view(rect);

            let painter = ui.painter_at(rect);
            self.grid.draw(&painter, &self.camera);

            let min_cos = normalize_unit_value(self.cos_value);
            let (segments_count, dragged) = self.curve.draw(ui, &painter, &self.camera, min_cos, self.min_len, true);
            self.segments_count = segments_count;

            FourConvexBuilder::default().build()
            
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
