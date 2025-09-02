mod steam;
mod ui;
mod app;
mod instances;
mod export_page;
mod settings_page;

use app::*;
use eframe::egui::*;


fn main() -> eframe::Result<()> {

  env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).

  let options = eframe::NativeOptions {
      viewport: ViewportBuilder {
        inner_size : Some(Vec2::new(1280.0, 720.0)),
        icon : ui::load_icon(),
        ..Default::default()
      },
      ..Default::default()
  };

  eframe::run_native(
    format!("{} {}", APP_NAME, env!("CARGO_PKG_VERSION")).as_str(),
    options,
    Box::new(|cc| {
        Ok(Box::new(App::new(cc)))
    }))
}