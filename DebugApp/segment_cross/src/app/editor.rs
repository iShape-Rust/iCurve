use debug_ui::util::camera::Camera;
use debug_ui::view::grid::{GridView, Step};
use debug_ui::view::segm::SegmentView;
use eframe::egui::{Color32, Sense, Shape, Stroke};
use eframe::{App, Frame, egui};
use i_curve::int::collision::x_segment::XOverlap;
use i_curve::int::math::x_segment::XSegment;
use crate::app::res::TestResource;

pub struct EditorApp {
    grid: GridView,
    seg_view_0: SegmentView,
    seg_view_1: SegmentView,
    camera: Camera,
    resource: TestResource,
    selected_test: usize
}

impl Default for EditorApp {
    fn default() -> Self {
        let mut resource = TestResource::with_path("./test");
        let selected_test = 0;
        let test = resource.load(selected_test).unwrap();
        let primary= test.primary.map(|p|p.into());
        let secondary= test.secondary.map(|p|p.into());

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
            seg_view_0: SegmentView::new(primary),
            seg_view_1: SegmentView::new(secondary),
            camera,
            resource,
            selected_test
        }
    }
}

impl App for EditorApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut Frame) {
        egui::SidePanel::left("test_list_panel").show(ctx, |ui| {
            ui.heading("Tests");
            for i in 0..self.resource.count {
                let label = format!("Test_{}", i);
                if ui.selectable_label(self.selected_test == i, label).clicked() {
                    self.selected_test = i;

                    let test = self.resource.load(i).unwrap();
                    let primary= test.primary.map(|p|p.into());
                    let secondary= test.secondary.map(|p|p.into());
                    self.seg_view_0 = SegmentView::new(primary);
                    self.seg_view_1 = SegmentView::new(secondary);
                }
            }
        });

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

            if let Some(cross) = s0.cross(&s1) {
                match cross {
                    XOverlap::Segment(s) => {
                        let va = self.camera.world_to_view(s.a.into());
                        let vb = self.camera.world_to_view(s.b.into());

                        painter.line_segment([va.into(), vb.into()], Stroke::new(4.0, Color32::ORANGE));
                    }
                    XOverlap::Point(p) => {
                        let vp = self.camera.world_to_view(p.into());
                        painter.add(Shape::circle_filled(vp.into(), 8.0, Color32::ORANGE));
                    }
                }
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

