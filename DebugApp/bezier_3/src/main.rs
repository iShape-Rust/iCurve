use debug_ui::{
    bezier::{CubicBezier, CubicBezierEditor},
    camera::Camera,
    egui::{self, Color32, Sense, Vec2},
    grid::{Grid, paint_camera_readout},
};

struct BezierApp {
    camera: Camera,
    grid: Grid,
    curve: CubicBezier,
}

impl Default for BezierApp {
    fn default() -> Self {
        Self {
            camera: Camera {
                zoom: 1.35,
                ..Camera::default()
            },
            grid: Grid::default(),
            curve: CubicBezier::default(),
        }
    }
}

impl eframe::App for BezierApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default()
            .frame(egui::Frame::default().fill(Color32::from_rgb(18, 20, 24)))
            .show(ctx, |ui| {
                let available_size = ui.available_size();
                let (response, painter) =
                    ui.allocate_painter(available_size, Sense::click_and_drag());
                let rect = response.rect;

                self.grid
                    .handle_input(ui, &response, rect, &mut self.camera);
                self.grid.paint(&painter, rect, &self.camera);

                CubicBezierEditor::new("main_curve").show(
                    ui,
                    &painter,
                    rect,
                    &self.camera,
                    &mut self.curve,
                );

                paint_camera_readout(&painter, rect, &self.camera);
            });
    }
}

fn main() -> eframe::Result<()> {
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("Bezier 3 Debug")
            .with_inner_size(Vec2::new(960.0, 720.0)),
        ..eframe::NativeOptions::default()
    };

    eframe::run_native(
        "Bezier 3 Debug",
        native_options,
        Box::new(|_cc| Ok(Box::new(BezierApp::default()))),
    )
}
