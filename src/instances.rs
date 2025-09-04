use std::{default, io};
use std::path::{Path, PathBuf};
use std::collections::{HashMap};
use serde::Deserialize;
use directories::BaseDirs;

#[derive(Debug)]
pub struct Instance {
  pub folder_name : String,
  pub group : String,
  pub icon_path : Option<PathBuf>,
  pub checked : bool,
}
impl Default for Instance {
    fn default() -> Self {
        Self {
            folder_name : String::new(),
            group : String::new(),
            icon_path : None,
            checked : false
        }
    }
}

#[derive(Debug, Deserialize)]
struct Root {
    #[serde(rename = "formatVersion")]
    format_version: String,
    groups: HashMap<String, Group>,
}

#[derive(Debug, Deserialize)]
struct Group {
    hidden: bool,
    instances: Vec<String>,
}

/// Parse the JSON and build instances. Set `include_hidden` to false to skip hidden groups.
pub fn get_instances_from_path(path: impl AsRef<Path>, include_hidden: bool)
-> Result<Vec<Instance>, Box<dyn std::error::Error>> {
  let json_file = std::fs::read_to_string(path)?;
  let root: Root = serde_json::from_str(&json_file)?;

  let mut instances = Vec::new();

  for (group_name, group) in root.groups.into_iter() {
    if !include_hidden && group.hidden { continue; }
    else { 
      for folder_name in group.instances.into_iter() {
          instances.push(Instance {
            folder_name, group : group_name.clone(), ..Default::default()
        });
      }
    }
  } // I know this can be done better, but who's gonna learn closures man

  Ok(instances)
}

pub fn default_prism_path() -> Result<PathBuf, io::Error> {

    #[cfg(target_os = "windows")]
    {
        if let Some(base_dirs) = BaseDirs::new() {
            let mut appdata_local_path = base_dirs.config_local_dir().to_owned();
            appdata_local_path.push("Programs\\PrismLauncher");

            return Ok(appdata_local_path)
        } else {
            return Err(io::Error::last_os_error())
        }
    }

}