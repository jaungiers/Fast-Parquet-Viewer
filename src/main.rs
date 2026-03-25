#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app;
mod loader;
mod table;

fn main() -> eframe::Result {
    let args: Vec<String> = std::env::args().collect();
    let initial_file = args.get(1).cloned();

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("Parquet Viewer")
            .with_inner_size([1280.0, 800.0])
            .with_min_inner_size([640.0, 400.0])
            .with_drag_and_drop(true)
            .with_icon(load_icon()),
        ..Default::default()
    };

    eframe::run_native(
        "Parquet Viewer",
        options,
        Box::new(move |cc| Ok(Box::new(app::ParquetApp::new(cc, initial_file)))),
    )
}

fn load_icon() -> egui::IconData {
    let png_bytes = include_bytes!("../assets/icon.png");
    match image::load_from_memory(png_bytes) {
        Ok(img) => {
            let img = img.into_rgba8();
            let (w, h) = img.dimensions();
            egui::IconData { rgba: img.into_raw(), width: w, height: h }
        }
        Err(_) => egui::IconData::default(),
    }
}
