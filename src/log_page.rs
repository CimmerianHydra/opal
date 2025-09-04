use std::path::PathBuf;
use serde::{Serialize, Deserialize};
use super::ui::TabPage;
use super::app::{AppModel, APP_HEADER_PADDING};

/// The “Settings” tab, with a text input as an example of per-tab state.
pub struct LogPage;

impl Default for LogPage {
    fn default() -> Self {
        Self
    }
}

impl TabPage for LogPage {
    fn id(&self) -> &'static str { "log" }
    fn label(&self) -> &'static str { "Logs" }

    fn ui(&mut self, ui: &mut eframe::egui::Ui, model: &mut AppModel) {
        ui.heading("Logs");

        ui.add_space(APP_HEADER_PADDING);

        ui.label(&model.log_printout);
    }
}