mod steam_start_stop;

use serde::Deserialize;
use std::fs;
use std::fs::DirEntry;
use std::path::{Path, PathBuf};
use eframe::egui;

use crate::steam_start_stop::{ensure_steam_stopped};

const APP_NAME : &str = "Opal";
const DOT_MINECRAFT_FOLDER_NAME : &str = ".minecraft"; // Used to check whether a given folder really is a modpack, for old modpacks.
const MINECRAFT_FOLDER_NAME : &str = "minecraft"; // Used to check whether a given folder really is a modpack, for newer modpacks.

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
#[derive(Default)]
struct Config {
  prism_main_path: String,
  prism_inst_path: String,
  steam_shortcuts_path: String,
}

#[derive(Debug)]
struct Instance {
  folder_name : String,
  folder_path : String,
  checked : bool,
}

#[derive(Default)]
struct Opal {
  config : Config,
  instance_vector: Vec<Instance>,
}
impl Opal {
  fn new(config : &Config) -> Self {
    Self {
      config : Config {
            prism_main_path: config.prism_main_path.clone(),
            prism_inst_path: config.prism_inst_path.clone(),
            steam_shortcuts_path: config.steam_shortcuts_path.clone(),
          },
      instance_vector : get_instances_from_path(&config.prism_inst_path),
    }
  }

  fn update_instance_vector(&mut self) {
    self.instance_vector = get_instances_from_path(&self.config.prism_inst_path);
  }
}
impl eframe::App for Opal {
  fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
    ctx.set_pixels_per_point(1.2);
    egui::CentralPanel::default().show(ctx, |ui| {
      ui.heading("Export PrismLauncher to Steam");

      ui.horizontal(|ui| {
        let name_label = ui.label("PrismLauncher Executable Path:");
        ui.text_edit_singleline(&mut self.config.prism_main_path)
          .labelled_by(name_label.id);
      });

      ui.horizontal(|ui| {
        let name_label = ui.label("PrismLauncher Instances Path:");
        ui.text_edit_singleline(&mut self.config.prism_inst_path)
          .labelled_by(name_label.id);
        if ui.button("📂").clicked() {
          if let Some(folder) = pick_folder() {
              self.config.prism_inst_path = folder.to_string_lossy().to_string();
              self.update_instance_vector();
          }
        }
        if ui.button("🔄").clicked() {
          self.update_instance_vector();
        };
      });

      ui.separator();
      
      ui.heading("Instances Found:");

      if self.instance_vector.is_empty() {
        ui.label(format!("No PrismLauncher instance found in {}.", self.config.prism_inst_path));
      } else {
        for inst in &mut self.instance_vector {
          ui.checkbox(&mut inst.checked, &inst.folder_name);
        }
      }

      if ui.button("Export Selected to Steam Shortcuts").clicked() {
          ensure_steam_stopped();
        };
      
      ui.separator();
      
      ui.label(format!("The selected path for instance folder is {}", self.config.prism_inst_path));
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

fn get_instances_from_path(path : &String) -> Vec<Instance> {
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
  let to_return = folders;
  to_return
}

fn main() -> eframe::Result {
  let config_json = fs::read_to_string("config/config.json").expect("Config file not found or unreadable.");
  let config: Config = serde_json::from_str(config_json.as_str()).expect("JSON was not well-formatted.");
  let application = Opal::new(&config);

  env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).
  let options = eframe::NativeOptions {
      viewport: egui::ViewportBuilder::default().with_inner_size([1280.0, 720.0]),
      ..Default::default()
  };
  eframe::run_native(
    APP_NAME,
    options,
    Box::new(|cc| {
        Ok(Box::<Opal>::from(application))
    }),
  )
}