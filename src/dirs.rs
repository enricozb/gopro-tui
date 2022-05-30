use std::{fs, path::PathBuf};

use directories::ProjectDirs;

use crate::error::{err, Result};

pub fn project_dirs() -> Result<ProjectDirs> {
  ProjectDirs::from("com", "enricozb", env!("CARGO_PKG_NAME")).ok_or(err!("Couldn't construct project directories"))
}

pub fn config() -> Result<PathBuf> {
  let config_dir = project_dirs()?.config_dir().to_path_buf();

  if !config_dir.exists() {
    fs::create_dir(&config_dir)?;
  };

  Ok(config_dir)
}

pub fn config_json() -> Result<PathBuf> {
  Ok(config()?.join("config.json"))
}
