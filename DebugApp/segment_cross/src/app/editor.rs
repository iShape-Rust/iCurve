use debug_ui::util::camera::Camera;
use debug_ui::view::grid::{GridView, Step};
use debug_ui::view::segm::SegmentView;
use eframe::egui::{Color32, Pos2, Sense, Shape, Stroke};
use eframe::{App, Frame, egui};
use i_curve::float::math::point::Point;
use i_curve::int::collision::solver::x_segment::XResult;
use i_curve::int::math::x_segment::XSegment;

pub struct EditorApp {
    grid: GridView,
    seg_view_0: SegmentView,
    seg_view_1: SegmentView,
    camera: Camera,
}

impl Default for EditorApp {
    fn default() -> Self {

        let mut camera = Camera::empty();
        camera.set_scale(10.0);
        Self {
            grid: GridView::new(vec![
                Step::new(64.0, Color32::RED, 0.5),
                Step::new(4096.0, Color32::ORANGE,0.5),
                Step::new(262144.0, Color32::YELLOW, 0.5),
                Step::new(16777216.0, Color32::GREEN, 0.5),
                Step::new(1073741824.0, Color32::BLUE, 0.5),
            ]),
            seg_view_0: SegmentView::new([Point::new(-10.0, 0.0), Point::new(10.0, 0.0)]),
            seg_view_1: SegmentView::new([Point::new(0.0, -10.0), Point::new(0.0, 10.0)]),
            camera,
        }
    }
}

impl App for EditorApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut Frame) {
        egui::TopBottomPanel::top("segments").show(ctx, |ui| {
            ui.label(format!("A: {:?}, B: {:?}", &self.seg_view_0.points(), &self.seg_view_1.points()));
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            let (rect, response) = ui.allocate_exact_size(ui.available_size(), Sense::drag());
            self.camera.update_view(rect);

            let pnt_0 = self.seg_view_0.points();
            let pnt_1 = self.seg_view_1.points();
            let s0 = XSegment::new(pnt_0[0], pnt_0[1]);
            let s1 = XSegment::new(pnt_1[0], pnt_1[1]);

            let stroke = Stroke::new(2.0, Color32::GRAY);

            let painter = ui.painter_at(rect);
            self.grid.draw(&painter, &self.camera);

            let dragged_0 = self.seg_view_0.draw(ui, &painter, &self.camera, stroke,0);
            let dragged_1 = self.seg_view_1.draw(ui, &painter, &self.camera, stroke,2);

            match s0.cross(&s1) {
                XResult::Segment(s) => {
                    let wa = Point { x: s.a.x as f64, y: s.a.y as f64 };
                    let wb = Point { x: s.b.x as f64, y: s.b.y as f64 };
                    let va = self.camera.world_to_view(wa);
                    let vb = self.camera.world_to_view(wb);
                    let sa = Pos2::new(va.x as f32, va.y as f32);
                    let sb = Pos2::new(vb.x as f32, vb.y as f32);


                    painter.line_segment([sa, sb], Stroke::new(4.0, Color32::ORANGE));
                }
                XResult::Point(p) => {
                    let wp = Point { x: p.x as f64, y: p.y as f64 };
                    let vp = self.camera.world_to_view(wp);
                    let sp = Pos2::new(vp.x as f32, vp.y as f32);
                    painter.add(Shape::circle_filled(sp, 8.0, Color32::ORANGE));
                }
                XResult::None => {}
            }

            if !(dragged_0 || dragged_1) {
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

