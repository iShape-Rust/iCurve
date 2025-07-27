use debug_ui::util::camera::Camera;
use debug_ui::view::curve::CurveView;
use debug_ui::view::grid::{GridView, Step};
use eframe::egui::{Color32, Pos2, Sense, Shape, Stroke};
use eframe::{App, Frame, egui};
use i_curve::float::math::point::Point;
use i_curve::int::bezier::spline_cube::IntCubeSpline;
use i_curve::int::collision::solver::cross::SplineOverlay;
use i_curve::int::collision::solver::x_segment::XOverlap;
use i_curve::int::collision::spline::Spline;
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
                Step::new(4096.0, Color32::ORANGE, 0.5),
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
            min_len: 16,
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

            let cube_0 = Spline::Cube(IntCubeSpline {
                anchors: self.curve_0.anchors(),
            });
            let cube_1 = Spline::Cube(IntCubeSpline {
                anchors: self.curve_1.anchors(),
            });

            let result = cube_0.overlay(&cube_1);
            let stroke = Stroke::new(2.0, Color32::GRAY);

            let min_cos = VectorNormalization16Util::normalize_unit_value(self.cos_value);
            let (_, dragged_0) = self.curve_0.draw_editable(
                ui,
                &painter,
                &self.camera,
                min_cos,
                self.min_len,
                stroke,
                true,
                0,
            );
            let (_, dragged_1) = self.curve_1.draw_editable(
                ui,
                &painter,
                &self.camera,
                min_cos,
                self.min_len,
                stroke,
                true,
                4,
            );
            let dragged = dragged_0 || dragged_1;

            for overlay in result.iter() {
                match overlay {
                    XOverlap::Point(p) => {
                        let vp = self
                            .camera
                            .world_to_view(Point::new(p.x as f64, p.y as f64));
                        let sp = Pos2::new(vp.x as f32, vp.y as f32);
                        painter.add(Shape::circle_filled(sp, 8.0, Color32::RED));
                    }
                    XOverlap::Segment(s) => {
                        let wa = Point {
                            x: s.a.x as f64,
                            y: s.a.y as f64,
                        };
                        let wb = Point {
                            x: s.b.x as f64,
                            y: s.b.y as f64,
                        };
                        let va = self.camera.world_to_view(wa);
                        let vb = self.camera.world_to_view(wb);
                        let sa = Pos2::new(va.x as f32, va.y as f32);
                        let sb = Pos2::new(vb.x as f32, vb.y as f32);

                        painter.line_segment([sa, sb], Stroke::new(4.0, Color32::RED));
                    }
                }
            }

            if !dragged {
                let delta = response.drag_delta();
                self.camera.move_by_view_xy(delta.x as f64, delta.y as f64);
            }

            ctx.input(|i| {
                let scroll = i.smooth_scroll_delta.y as f64;
                if scroll != 0.0 {
                    if let Some(mouse) = i.pointer.hover_pos() {
                        self.camera
                            .zoom_by_view_xy(mouse.x as f64, mouse.y as f64, scroll);
                    }
                }
            });
        });
    }
}
