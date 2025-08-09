use crate::app::res::TestResource;
use debug_ui::ext::color::Color32Ext;
use debug_ui::util::camera::Camera;
use debug_ui::view::curve::CurveView;
use debug_ui::view::grid::{GridView, Step};
use eframe::egui::{Color32, Sense, Shape, Stroke};
use eframe::{App, Frame, egui};
use i_curve::int::base::spline::IntSpline;
use i_curve::int::bezier::spline_cubic::IntCubicSpline;
use i_curve::int::collision::approximation::SplineApproximation;
use i_curve::int::collision::solver::SplineOverlay;
use i_curve::int::collision::space::Space;
use i_curve::int::collision::x_segment::XOverlap;
use i_curve::int::math::normalize::VectorNormalization16Util;
use i_curve::int::math::point::IntPoint;

pub struct EditorApp {
    grid: GridView,
    curve_0: CurveView,
    curve_1: CurveView,
    camera: Camera,
    cos_value: f64,
    min_len: u32,
    resource: TestResource,
    selected_test: usize,
    approximation: bool,
}

impl Default for EditorApp {
    fn default() -> Self {
        let mut resource = TestResource::with_path("./test");
        let selected_test = 0;
        let test = resource.load(selected_test).unwrap();
        let primary = test.primary.map(|p| p.into());
        let secondary = test.secondary.map(|p| p.into());

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
            curve_0: CurveView::new(primary),
            curve_1: CurveView::new(secondary),
            camera,
            cos_value: 0.95,
            min_len: 16,
            resource,
            selected_test,
            approximation: false,
        }
    }
}

impl App for EditorApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut Frame) {
        egui::SidePanel::left("test_list_panel").show(ctx, |ui| {
            ui.heading("Tests");
            for i in 0..self.resource.count {
                let label = format!("Test_{}", i);
                if ui
                    .selectable_label(self.selected_test == i, label)
                    .clicked()
                {
                    self.selected_test = i;

                    let test = self.resource.load(i).unwrap();
                    let primary = test.primary.map(|p| p.into());
                    let secondary = test.secondary.map(|p| p.into());
                    self.curve_0 = CurveView::new(primary);
                    self.curve_1 = CurveView::new(secondary);
                }
            }
        });

        egui::TopBottomPanel::top("slider_panel").show(ctx, |ui| {
            ui.checkbox(&mut self.approximation, "Approximation");
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

            let cube_0 = IntCubicSpline {
                anchors: self.curve_0.anchors(),
            };
            let cube_1 = IntCubicSpline {
                anchors: self.curve_1.anchors(),
            };

            let space = Space::default();
            
            let result =
                IntSpline::Cubic(cube_0.clone()).overlay(&IntSpline::Cubic(cube_1.clone()), &space);
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
                        let vp = self.camera.world_to_view(p.into());
                        painter.add(Shape::circle_filled(vp.into(), 8.0, Color32::RED));
                    }
                    XOverlap::Segment(s) => {
                        let va = self.camera.world_to_view(s.a.into());
                        let vb = self.camera.world_to_view(s.b.into());

                        painter
                            .line_segment([va.into(), vb.into()], Stroke::new(4.0, Color32::RED));
                    }
                }
            }

            if self.approximation {
                let space = Space::default();
                let c0 = cube_0.into_collider(&space);
                let c1 = cube_1.into_collider(&space);
                if let Some(apx) = c0.approximation {
                    self.draw_approximation(&painter, apx.slice());
                }
                if let Some(apx) = c1.approximation {
                    self.draw_approximation(&painter, apx.slice());
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

impl EditorApp {
    fn draw_approximation(&self, painter: &egui::Painter, convex: &[IntPoint]) {
        if convex.len() > 2 {
            let view_points: Vec<_> = convex
                .iter()
                .map(|wp| self.camera.world_to_view(wp.into()).into())
                .collect();

            painter.add(Shape::convex_polygon(
                view_points,
                Color32::LIGHT_YELLOW.with_opacity(0.2),
                Stroke::new(2.0, Color32::YELLOW.with_opacity(0.8)),
            ));
        } else {
            let points = [convex[0], convex[1]].map(|wp| self.camera.world_to_view(wp.into()).into());
            painter.line_segment(points, Stroke::new(2.0, Color32::BLUE));
        }
    }
}
