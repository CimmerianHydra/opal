use serde::Deserialize;
use std::fs;
use std::fs::DirEntry;
use std::path::Path;
use eframe::egui;

const LAUNCHER_TEMP_FOLDER_NAME : &str = ".LAUNCHER_TEMP";

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
}
impl eframe::App for Opal {
  fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
    ctx.set_pixels_per_point(1.5);
    egui::CentralPanel::default().show(ctx, |ui| {
      ui.heading("PrismLauncher to Steam Shortcuts");

      ui.horizontal(|ui| {
        let name_label = ui.label("PrismLauncher Executable Path:");
        ui.text_edit_singleline(&mut self.config.prism_main_path)
          .labelled_by(name_label.id);
      });

      ui.horizontal(|ui| {
        let name_label = ui.label("PrismLauncher Instances Path:");
        ui.text_edit_singleline(&mut self.config.prism_inst_path)
          .labelled_by(name_label.id);
        if ui.button("Refresh List").clicked() {
          self.instance_vector = get_instances_from_path(&self.config.prism_inst_path);
        };
      });

      ui.separator();
      ui.heading("Instances Found:");
      for inst in &mut self.instance_vector {
        ui.checkbox(&mut inst.checked, &inst.folder_name);
      }

      if ui.button("Export Selected to Steam Shortcuts").clicked() {
          self.instance_vector = get_instances_from_path(&self.config.prism_inst_path);
        };

      ui.label(format!("The selected path for instance folder is {}", self.config.prism_inst_path));
    });
  }
}

fn get_instances_from_path(path : &String) -> Vec<Instance> {
  let mut folders = Vec::new();
  if let Ok(entries) = fs::read_dir(path) {
      for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
          if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
            if name.to_string() != LAUNCHER_TEMP_FOLDER_NAME.to_string() {
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

  let instance_vector = get_instances_from_path(&config.prism_inst_path);
  println!("{:?}", instance_vector);

  let application = Opal::new(&config);

  env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).
  let options = eframe::NativeOptions {
      viewport: egui::ViewportBuilder::default().with_inner_size([640.0, 480.0]),
      ..Default::default()
  };
  eframe::run_native(
    "Opal",
    options,
    Box::new(|cc| {
        Ok(Box::<Opal>::from(application))
    }),
  )
}