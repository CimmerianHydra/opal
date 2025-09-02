use super::ui::TabPage;
use eframe::egui::*;
use std::time::Duration;
use super::app::{AppModel, APP_HEADER_PADDING};
use super::steam::{start_steam, ensure_steam_started, ensure_steam_stopped};

const APP_INSTANCE_GRID_COLS : usize = 3;
const APP_INSTANCE_GRID_MAX_HEIGHT : f32 = 200.0;


/// The “Export” tab (no special state for this simple example).
#[derive(Default)]
pub struct ExportPage;

impl TabPage for ExportPage {
    fn id(&self) -> &'static str { "export" }
    fn label(&self) -> &'static str { "Export" }

    fn ui(&mut self, ui: &mut Ui, model: &mut AppModel) {
        ui.heading("Export PrismLauncher to Steam Shortcuts");

        ui.add_space(APP_HEADER_PADDING);

        ui.heading("Instances Found:");

        ScrollArea::vertical()
                .id_salt("instances_scroll_grid")
                .max_height(APP_INSTANCE_GRID_MAX_HEIGHT) // or: .max_height(ui.available_height())
                .auto_shrink([false; 2])
                .show(ui, |ui| {
                    if model.instances.is_empty() {
                        ui.label(format!("No PrismLauncher instance found in specified path."));
                        return;
                    } else {
                        Grid::new("instances_grid")
                            .num_columns(APP_INSTANCE_GRID_COLS)
                            .spacing([12.0, 8.0])
                            .striped(false)
                            .show(ui, |ui| {
                                for row in model.instances.chunks_mut(APP_INSTANCE_GRID_COLS) {
                                    for inst in row {
                                        ui.checkbox(&mut inst.checked, &inst.folder_name);
                                    }
                                    // optional: pad short last rows
                                    // for _ in row.len()..APP_INSTANCE_GRID_COLS { ui.allocate_space(egui::vec2(0.0, 0.0)); }
                                    ui.end_row();
                                }
                            });
                    }
                });
        
        ui.separator();

        if ui.button("Export Selected to Steam Shortcuts").clicked() {
              if let Err(e) = ensure_steam_stopped(Duration::from_millis(1000)) {
                model.log_printout.push_str(&format!("\nFailed to close Steam: {e}"));
              }

              model.update_steam_shortcuts();
              
              if let Err(e) = start_steam() {
                model.log_printout.push_str(&format!("\nFailed to start Steam: {e}"));
              }
              if let Err(e) = ensure_steam_started(Duration::from_millis(1000)) {
                model.log_printout.push_str(&format!("\nFailed to check if Steam started: {e}"));
              }

            }

    }
}