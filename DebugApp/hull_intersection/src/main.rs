mod app;

use crate::app::editor::EditorApp;

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions::default();
    eframe::run_native(
        "Curve App",
        options,
        Box::new(|_cc| Ok(Box::new(EditorApp::default()))),
    )
}
