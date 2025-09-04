use std::path::PathBuf;
use serde::{Serialize, Deserialize};
use log::{error};
use super::ui::TabPage;
use super::app::{AppModel, APP_HEADER_PADDING};
use super::instances::default_prism_path;
use super::steam::default_steam_shortcuts_path;

// ---------- Settings model ----------
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Config {
    pub prism_main_path: PathBuf,
    pub steam_shortcuts_path: PathBuf,
    pub user_id : u32,
    pub include_hidden: bool,
}
impl Default for Config {
    fn default() -> Self {
        Self {
            prism_main_path: 
                match default_prism_path() {
                    Ok(p) => p,
                    Err(e) => PathBuf::new()
                },
            include_hidden: false,
            user_id: 86176907,
            steam_shortcuts_path :
                match default_steam_shortcuts_path(86176907) {
                    Ok(p) => p,
                    Err(e) => PathBuf::new()
                },
        }
    }
}

/// The “Settings” tab, with a text input as an example of per-tab state.
pub struct SettingsPage;

impl Default for SettingsPage {
    fn default() -> Self {
        Self
    }
}

impl TabPage for SettingsPage {
    fn id(&self) -> &'static str { "settings" }
    fn label(&self) -> &'static str { "Settings" }

    fn ui(&mut self, ui: &mut eframe::egui::Ui, model: &mut AppModel) {
        ui.heading("Settings");

        ui.add_space(APP_HEADER_PADDING);

        ui.horizontal(|ui| {
            let name_label = ui.label("PrismLauncher Folder Path:");
            ui.text_edit_singleline(&mut model.config.prism_main_path.to_string_lossy())
                .labelled_by(name_label.id);
            if ui.button("📂").clicked() {
                if let Some(folder) = rfd::FileDialog::new().set_directory(".").pick_folder() {
                    model.config.prism_main_path = folder;
                    model.update_instances();
                }
            }
            if ui.button("🔄").clicked() {
                model.update_instances();
            };
        });
        
        ui.horizontal(|ui| {
            let name_label = ui.label("Steam User ID:");
            ui.text_edit_singleline(&mut model.config.user_id.to_string())
                .labelled_by(name_label.id);
            if ui.button("🔄").clicked() {
                match default_steam_shortcuts_path(model.config.user_id) {
                    Ok(path) => model.config.steam_shortcuts_path = path,
                    Err(e) => {
                        let msg = format!("\nCouldn't find shortcuts file path! {}", e);
                        error!("{}", &msg);
                        model.log_printout.push_str(&msg);
                        return;
                    }
                }
            };
        });

        ui.horizontal(|ui| {
            let name_label = ui.label("Steam Shortcuts Path:");
            ui.text_edit_singleline(&mut model.config.steam_shortcuts_path.to_string_lossy())
                .labelled_by(name_label.id);
            if ui.button("📂").clicked() {
                if let Some(folder) = rfd::FileDialog::new().set_directory(".").pick_folder() {
                    model.config.steam_shortcuts_path = folder;
                }
            }
        });
        
        ui.horizontal(|ui| {
            let name_label = ui.label("Include Hidden Groups");
            ui.checkbox(&mut model.config.include_hidden, "")
                .labelled_by(name_label.id);
        });
    }
}