use debug_ui::ext::color::Color32Ext;
use debug_ui::util::camera::Camera;
use debug_ui::view::curve::CurveView;
use debug_ui::view::grid::{GridView, Step};
use eframe::egui::{Color32, Pos2, Sense, Shape, Stroke};
use eframe::{App, Frame, egui};
use i_curve::float::math::point::Point;
use i_curve::int::collision::colliding::{Colliding, CollidingResult};
use i_curve::int::collision::convex_hull::FourConvexPathExt;
use i_curve::int::math::normalize::VectorNormalization16Util;

pub struct EditorApp {
    grid: GridView,
    curve_0: CurveView,
    curve_1: CurveView,
    camera: Camera,
    cos_value: f64,
    min_len: u32,
}

impl Default for EditorApp {
    fn default() -> Self {
        let mut camera = Camera::empty();
        camera.set_scale(0.2);
        Self {
            grid: GridView::new(vec![
                Step::new(64.0, Color32::RED, 0.5),
                Step::new(4096.0, Color32::ORANGE,0.5),
                Step::new(262144.0, Color32::YELLOW, 0.5),
                Step::new(16777216.0, Color32::GREEN, 0.5),
                Step::new(1073741824.0, Color32::BLUE, 0.5),
            ]),
            curve_0: CurveView::new([
                Point::new(0.0, -100.0),
                Point::new(0.0, 100.0),
                Point::new(100.0, 0.0),
                Point::new(-100.0, 0.0),
            ]),
            curve_1: CurveView::new([
                Point::new(100.0, 100.0),
                Point::new(100.0, 200.0),
                Point::new(200.0, 100.0),
                Point::new(200.0, 200.0),
            ]),
            camera,
            cos_value: 0.95,
            min_len: 16
        }
    }
}

impl App for EditorApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut Frame) {
        egui::TopBottomPanel::top("slider_panel").show(ctx, |ui| {
            ui.add(egui::Slider::new(&mut self.cos_value, 0.9..=1.0).text("Min Cos"));
            ui.add(egui::Slider::new(&mut self.min_len, 4..=4096).text("Min Len"));
            ui.label(format!("A: {:?}", &self.curve_0.anchors()));
            ui.label(format!("B: {:?}", &self.curve_1.anchors()));
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            let (rect, response) = ui.allocate_exact_size(ui.available_size(), Sense::drag());
            self.camera.update_view(rect);

            let painter = ui.painter_at(rect);
            self.grid.draw(&painter, &self.camera);

            let hull_0 = self.curve_0.anchors().to_convex_hull();
            let hull_1 = self.curve_1.anchors().to_convex_hull();

            let is_overlap = hull_0.collide(&hull_1);

            let stroke = match is_overlap {
                CollidingResult::Overlap => Stroke::new(4.0, Color32::RED),
                CollidingResult::Touch => Stroke::new(4.0, Color32::ORANGE),
                CollidingResult::None => Stroke::new(1.0, Color32::WHITE)
            };

            {
                let view_points: Vec<_> = hull_0.slice().iter().map(|wp|{
                    let vp = self.camera.world_to_view(Point::new(wp.x as f64, wp.y as f64));
                    Pos2::new(vp.x as f32, vp.y as f32)
                }).collect();

                painter.add(Shape::convex_polygon(
                    view_points,
                    Color32::LIGHT_YELLOW.with_opacity(0.2),
                    Stroke::new(2.0, Color32::YELLOW.with_opacity(0.8)),
                ));
            }

            {
                let view_points: Vec<_> = hull_1.slice().iter().map(|wp|{
                    let vp = self.camera.world_to_view(Point::new(wp.x as f64, wp.y as f64));
                    Pos2::new(vp.x as f32, vp.y as f32)
                }).collect();

                painter.add(Shape::convex_polygon(
                    view_points,
                    Color32::LIGHT_YELLOW.with_opacity(0.2),
                    Stroke::new(2.0, Color32::YELLOW.with_opacity(0.8)),
                ));
            }

            let min_cos = VectorNormalization16Util::normalize_unit_value(self.cos_value);
            let (_, dragged_0) = self.curve_0.draw(ui, &painter, &self.camera, min_cos, self.min_len, stroke,true, 0);
            let (_, dragged_1) = self.curve_1.draw(ui, &painter, &self.camera, min_cos, self.min_len, stroke,true, 4);
            let dragged = dragged_0 || dragged_1;

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
