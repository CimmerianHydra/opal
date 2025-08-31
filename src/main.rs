mod steam_start_stop;

use serde::Deserialize;
use std::fs;
use std::fs::DirEntry;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::collections::BTreeMap;
use std::time::Duration;
use eframe::egui::{self, ColorImage, IconData, Vec2};

use steam_shortcuts_util::{
  parse_shortcuts,
  shortcuts_to_bytes,
  shortcut::{Shortcut, ShortcutOwned},
  app_id_generator::calculate_app_id_for_shortcut
};

use steamlocate::SteamDir;
use log::{warn, info};

use crate::steam_start_stop::{ensure_steam_started, ensure_steam_stopped, start_steam};

const APP_NAME : &str = "Opal";
const DOT_MINECRAFT_FOLDER_NAME : &str = ".minecraft"; // Used to check whether a given folder really is a modpack, for old modpacks.
const MINECRAFT_FOLDER_NAME : &str = "minecraft"; // Used to check whether a given folder really is a modpack, for newer modpacks.
const APP_INSTANCE_GRID_COLS : usize = 3;
const APP_INSTANCE_GRID_MAX_HEIGHT : f32 = 200.0;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
#[derive(Default, Clone)]
struct Config {
  prism_main_path: PathBuf,
  prism_inst_path: PathBuf,
  steam_shortcuts_path: PathBuf,
}

#[derive(Debug)]
struct Instance {
  folder_name : String,
  folder_path : String,
  checked : bool,
}

/// Your app's "desired shortcut" input. Adapt as needed.
#[derive(Debug, Clone)]
pub struct DesiredShortcut {
    pub app_name: String,
    pub exe: String,
    pub start_dir: String,
    pub icon: String,
    pub launch_options: String,
    pub tags: Vec<String>,
    // Optional: populate if you use Steam's "Shortcut Path" field
    pub shortcut_path: String,
}
impl DesiredShortcut {
  fn make_owned(&self, order: usize) -> ShortcutOwned {
      // Build with borrowed &strs just for this call, immediately convert to owned.
      let order_string = order.to_string();
      let tmp: Shortcut = Shortcut::new(
          &order_string,         // order is a string in VDF
          &self.app_name,
          &self.exe,
          &self.start_dir,
          &self.icon,
          &self.shortcut_path,
          &self.launch_options,
      );

      let mut owned = tmp.to_owned();
      // Tags are owned strings on `ShortcutOwned`
      owned.tags = self.tags.clone();

      // Compute app_id using the borrowed view of our owned struct
      owned.app_id = calculate_app_id_for_shortcut(&owned.borrow());

      // Sensible defaults (match crate’s intent)
      owned.is_hidden = false;
      owned.allow_desktop_config = true;
      owned.allow_overlay = true;
      owned.open_vr = 0;
      owned.dev_kit = 0;
      owned.dev_kit_overrite_app_id = 0;
      owned.last_play_time = 0;

      owned
  }
}

#[derive(Default)]
struct Opal {
  config : Config,
  instance_vector: Vec<Instance>,
  log_printout : String,
}
impl Opal {
  fn new(config : &Config) -> Self {
    Self {
      config :config.clone(),
      instance_vector : get_instances_from_path(&config.prism_inst_path),
      log_printout : format!("Welcome to {} ver. {}", APP_NAME, env!("CARGO_PKG_VERSION")),
    }
  }

  fn update_instance_vector(&mut self) {
    self.instance_vector = get_instances_from_path(&self.config.prism_inst_path);
  }

  fn update_steam_shortcuts(&mut self) {

    // Make sure the content exists and can be successfully read. If not, print out error.
    // Immediately break lifetimes with `to_owned`.
    let mut existing_owned: Vec<ShortcutOwned> = if self.config.steam_shortcuts_path.exists() {
        let bytes = fs::read(&self.config.steam_shortcuts_path).unwrap_or_default();
        let parsed: Vec<Shortcut> = parse_shortcuts(bytes.as_slice())
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, format!("parse: {e}")))
            .unwrap();
        parsed.into_iter().map(|s| s.to_owned()).collect()
    } else {
        Vec::new()
    };

    // Index existing by app_id (stable identifier for Steam assets).
    let mut by_id: BTreeMap<u32, ShortcutOwned> =
        existing_owned.drain(..).map(|s| (s.app_id, s)).collect();

    // Build desired shortcuts as owned and upsert by app_id.
    // We also re-number "order" later, so the `order` we put here is temporary.
    let exe_path_string = self.config.prism_main_path.to_string_lossy().to_string() + "\\prismlauncher.exe";
    let start_dir_string = self.config.prism_main_path.to_string_lossy().to_string();

    for (i, inst) in self.instance_vector.iter().enumerate() {
        if inst.checked {
          let app_name = inst.folder_name.clone();
          let launch_options = format!("-l \"{}\"", app_name);

          let d = DesiredShortcut {
            // These are the arguments that go into Shortcut::new() as well
            app_name : app_name.clone(),
            exe : exe_path_string.clone(),
            shortcut_path : String::new(),
            start_dir : start_dir_string.clone(),
            launch_options : launch_options,

            // TODO
            icon : String::new(),
            tags : vec![],
          };

          let sc = d.make_owned(i);
          // If you prefer "app name + exe" as the identity instead of app_id, change this keying.
          by_id.insert(sc.app_id, sc);
        }
    }

    // Rebuild a stable, ordered list and fix the `order` field.
    let mut final_owned: Vec<ShortcutOwned> = by_id.into_values().collect();
    final_owned.sort_by(|a, b| a.app_name.cmp(&b.app_name)); // or whatever ordering you like
    for (i, s) in final_owned.iter_mut().enumerate() {
        s.order = i.to_string();
    }

    // Borrow-on-demand to serialize.
    // NOTE: `shortcuts_to_bytes` wants `Vec<Shortcut<'_>>`, so produce a borrowed view.
    let borrowed: Vec<Shortcut> = final_owned.iter().map(|s| s.borrow()).collect();
    let out = shortcuts_to_bytes(&borrowed);

    // Write back to disk.
    fs::write(&self.config.steam_shortcuts_path, out).expect("Couldn't write steam shortcuts to file.");
  }

}
impl eframe::App for Opal {
  fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        ctx.set_pixels_per_point(1.2);

        // --- Fixed footer: always visible at the bottom ---
        egui::TopBottomPanel::bottom("footer").show(ctx, |ui| {

            // If you want the export button to always be visible too, keep it here:
            if ui.button("Export Selected to Steam Shortcuts").clicked() {
              if let Err(e) = ensure_steam_stopped(Duration::from_millis(500)) {
                self.log_printout.push_str(&format!("\nFailed to close Steam: {e}"));
              }

              self.update_steam_shortcuts();
              
              if let Err(e) = start_steam() {
                self.log_printout.push_str(&format!("\nFailed to start Steam: {e}"));
              }
              if let Err(e) = ensure_steam_started(Duration::from_millis(500)) {
                self.log_printout.push_str(&format!("\nFailed to check if Steam started: {e}"));
              }

            }

            ui.separator();

            // Scrollable log that sticks to the bottom as new lines arrive
            egui::ScrollArea::vertical()
                .id_salt("log_scroll")
                .max_height(86.0)
                .stick_to_bottom(true)
                .auto_shrink([false; 2])
                .show(ui, |ui| {
                    ui.monospace(&self.log_printout);
                });
        });

        // --- Main content goes here and will never cover the footer ---
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Export PrismLauncher to Steam");

            ui.horizontal(|ui| {
                let name_label = ui.label("PrismLauncher Executable Path:");
                ui.text_edit_singleline(&mut self.config.prism_main_path.to_string_lossy())
                    .labelled_by(name_label.id);
                if ui.button("📂").clicked() {
                    if let Some(folder) = pick_folder() {
                        self.config.prism_main_path = folder;
                    }
                }
            });

            ui.horizontal(|ui| {
                let name_label = ui.label("PrismLauncher Instances Path:");
                ui.text_edit_singleline(&mut self.config.prism_inst_path.to_string_lossy())
                    .labelled_by(name_label.id);
                if ui.button("📂").clicked() {
                    if let Some(folder) = pick_folder() {
                        self.config.prism_inst_path = folder;
                        self.update_instance_vector();
                    }
                }
                if ui.button("🔄").clicked() {
                    self.update_instance_vector();
                };
            });

            ui.separator();
            ui.heading("Instances Found:");

            egui::ScrollArea::vertical()
                .id_salt("instances_scroll_grid")
                .max_height(APP_INSTANCE_GRID_MAX_HEIGHT) // or: .max_height(ui.available_height())
                .auto_shrink([false; 2])
                .show(ui, |ui| {
                    if self.instance_vector.is_empty() {
                        ui.label(format!(
                            "No PrismLauncher instance found in {}.",
                            self.config.prism_inst_path.display()
                        ));
                        return;
                    }

                    egui::Grid::new("instances_grid")
                        .num_columns(APP_INSTANCE_GRID_COLS)
                        .spacing([12.0, 8.0])
                        .striped(false)
                        .show(ui, |ui| {
                            for row in self.instance_vector.chunks_mut(APP_INSTANCE_GRID_COLS) {
                                for inst in row {
                                    ui.checkbox(&mut inst.checked, &inst.folder_name);
                                }
                                // optional: pad short last rows
                                // for _ in row.len()..APP_INSTANCE_GRID_COLS { ui.allocate_space(egui::vec2(0.0, 0.0)); }
                                ui.end_row();
                            }
                        });
                });

            // If you chose to keep the export button in the footer, remove this:
            // if ui.button("Export Selected to Steam Shortcuts").clicked() {
            //     ensure_steam_stopped();
            //     self.update_steam_shortcuts();
            // };
        });
    }
}

pub fn pick_folder() -> Option<PathBuf> {
    rfd::FileDialog::new().set_directory(".").pick_folder()
}

fn contains_minecraft_folder(base_folder : &PathBuf) -> bool {
    let dot_minecraft_path = Path::new(base_folder).join(DOT_MINECRAFT_FOLDER_NAME);
    let minecraft_path = Path::new(base_folder).join(MINECRAFT_FOLDER_NAME);
    dot_minecraft_path.is_dir() || minecraft_path.is_dir()
}

fn get_instances_from_path(path : &PathBuf) -> Vec<Instance> {
  let mut folders = Vec::new();
  if let Ok(entries) = fs::read_dir(path) {
      for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
          if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
            if contains_minecraft_folder(&path) {
                folders.push(Instance {
                folder_name : name.to_string(),
                folder_path : path.to_string_lossy().to_string(),
                checked : false,
              });
            }
          }
        }
      }
    }
  folders
}

fn load_icon() -> IconData {
	let (icon_rgba, icon_width, icon_height) = {
		let icon = include_bytes!("../assets/icon.png");
		let image = image::load_from_memory(icon)
			.expect("Failed to open icon path")
			.into_rgba8();
		let (width, height) = image.dimensions();
		let rgba = image.into_raw();
		(rgba, width, height)
	};
	
	IconData {
		rgba: icon_rgba,
		width: icon_width,
		height: icon_height,
	}
}

fn main() -> eframe::Result {
  let config_json = fs::read_to_string("config/config.json").expect("Config file not found or unreadable.");
  let config: Config = serde_json::from_str(config_json.as_str()).expect("JSON was not well-formatted.");

  let application = Opal::new(&config);

  env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).
  let options = eframe::NativeOptions {
      viewport: egui::ViewportBuilder {
        inner_size : Some(Vec2::new(640.0, 360.0)),
        icon : Some(load_icon().into()),
        ..Default::default()
      },
      ..Default::default()
  };
  eframe::run_native(
    format!("{} {}", APP_NAME, env!("CARGO_PKG_VERSION")).as_str(),
    options,
    Box::new(|cc| {
        Ok(Box::<Opal>::from(application))
    }),
  )
}