use serde::Deserialize;
use std::fs;
use std::fs::DirEntry;
use std::path::Path;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct Config {
  prism_main_path: String,
  prism_inst_path: String,
  steam_shortcuts_path: String,
}

struct Instance {
  folder_name: String,
  folder_path: String,
}

fn main() {
  let config_json = fs::read_to_string("config/config.json").expect("Config file not found or unreadable.");
  let config: Config = serde_json::from_str(config_json.as_str()).expect("JSON was not well-formatted.");

  let instances_directory_iterator = fs::read_dir(config.prism_inst_path).expect("Couldn't generate list of instance directories.");

  println!("{:?}", config.prism_main_path);
  println!("{:?}", config.steam_shortcuts_path);
}