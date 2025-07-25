use serde::Deserialize;
use std::fs;
use std::fs::DirEntry;
use std::path::Path;
use eframe::egui;

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
  folder_name: String,
  folder_path: String,
}

#[derive(Default)]
struct Opal {
  config : Config,
  instance_vector: Vec<Instance>,
  age : u64,
}
impl eframe::App for Opal {
  fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
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
      });
      ui.add(egui::Slider::new(&mut self.age, 0..=120).text("age"));
      if ui.button("Increment").clicked() {
        self.age += 1;
      }
      ui.label(format!("The selected path for instance folder is {}", self.config.prism_inst_path));
    });
  }
}

fn get_instances_from_path(path : String) -> Vec<Instance> {
  let mut folders = Vec::new();
  if let Ok(entries) =  fs::read_dir(path) {
      for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
          if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
            folders.push(Instance {
              folder_name : name.to_string(),
              folder_path : path.to_string_lossy().to_string()
            });
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

  let instance_vector = get_instances_from_path(config.prism_inst_path);
  println!("{:?}", instance_vector);

  env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).
  let options = eframe::NativeOptions {
      viewport: egui::ViewportBuilder::default().with_inner_size([640.0, 480.0]),
      ..Default::default()
  };
  eframe::run_native(
    "Opal",
    options,
    Box::new(|cc| {
        Ok(Box::<Opal>::default())
    }),
  )
}