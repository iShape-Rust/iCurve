mod convex;
mod examples;

use crate::convex::paint_convex_shape;
use crate::examples::{CurveExample, load_examples};
use debug_ui::{
    camera::Camera,
    curve::{CurveEditor, CurvePointReadout, DebugCurve, paint_curve_point_readout},
    egui::{self, Color32, Sense, Vec2},
    grid::{Grid, paint_camera_readout},
};

struct CurveApp {
    camera: Camera,
    grid: Grid,
    examples: Vec<CurveExample>,
    active_example: usize,
    curve: DebugCurve,
    active_point: Option<CurvePointReadout>,
    load_error: Option<String>,
}

impl Default for CurveApp {
    fn default() -> Self {
        let loaded = load_examples();
        let curve = loaded
            .examples
            .first()
            .map(|example| example.curve.clone())
            .unwrap_or_default();
        let active_point = curve.first_point();

        Self {
            camera: Camera {
                zoom: 1.35,
                ..Camera::default()
            },
            grid: Grid::default(),
            examples: loaded.examples,
            active_example: 0,
            curve,
            active_point,
            load_error: loaded.error,
        }
    }
}

impl eframe::App for CurveApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        egui::Panel::left("curves_panel")
            .resizable(false)
            .default_size(172.0)
            .frame(egui::Frame::default().fill(Color32::from_rgb(24, 27, 32)))
            .show_inside(ui, |ui| {
                ui.spacing_mut().item_spacing = Vec2::new(6.0, 6.0);
                ui.add_space(8.0);

                let mut selected = None;

                for (index, example) in self.examples.iter().enumerate() {
                    if ui
                        .selectable_label(index == self.active_example, &example.name)
                        .clicked()
                    {
                        selected = Some(index);
                    }
                }

                if let Some(index) = selected {
                    self.select_example(index);
                }

                if let Some(error) = &self.load_error {
                    ui.add_space(8.0);
                    ui.separator();
                    ui.colored_label(Color32::from_rgb(240, 118, 118), error);
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
                paint_convex_shape(&painter, rect, &self.camera, &self.curve);

                let editor_response = CurveEditor::new("main_curve").show(
                    ui,
                    &painter,
                    rect,
                    &self.camera,
                    &mut self.curve,
                );

                if let Some(active_point) = editor_response.active_point {
                    self.active_point = Some(active_point);
                }

                paint_camera_readout(&painter, rect, &self.camera);

                if let Some(active_point) = self.active_point {
                    paint_curve_point_readout(&painter, rect, active_point);
                }
            });
    }
}

impl CurveApp {
    fn select_example(&mut self, index: usize) {
        if let Some(example) = self.examples.get(index) {
            self.active_example = index;
            self.curve = example.curve.clone();
            self.active_point = self.curve.first_point();
        }
    }
}

trait CurveReadout {
    fn first_point(&self) -> Option<CurvePointReadout>;
}

impl CurveReadout for DebugCurve {
    fn first_point(&self) -> Option<CurvePointReadout> {
        match self {
            Self::Line(points) => Some(CurvePointReadout {
                label: "start",
                position: points[0],
            }),
            Self::Quad(points) => Some(CurvePointReadout {
                label: "start",
                position: points[0],
            }),
            Self::Cubic(points) => Some(CurvePointReadout {
                label: "start",
                position: points[0],
            }),
            Self::Arc(arc) => Some(CurvePointReadout {
                label: "center",
                position: arc.center,
            }),
        }
    }
}

fn main() -> eframe::Result<()> {
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("Convex Debug")
            .with_inner_size(Vec2::new(960.0, 720.0)),
        ..eframe::NativeOptions::default()
    };

    eframe::run_native(
        "Convex Debug",
        native_options,
        Box::new(|_cc| Ok(Box::new(CurveApp::default()))),
    )
}
