mod user;
mod version;

use std::{collections::BTreeMap, path::Path};

use serde::{Deserialize, Serialize};

use self::{
  user::User,
  version::{Local, SerialNumber, Version},
};
use crate::{
  dirs,
  error::Result,
  mode::Mode,
  ui::state::session::{File as UiFile, Status as UiFileStatus},
};

type FileName = String;

#[derive(Serialize, Deserialize, Clone, Default)]
pub struct Source {
  #[serde(skip)]
  serial: SerialNumber,

  files: BTreeMap<FileName, File>,
}

impl Source {
  pub fn from(input_dir: &Path, mode: Mode) -> Result<Self> {
    let serial = match mode {
      Mode::Importing => Version::from(input_dir)?.camera_serial_number,
      Mode::Viewing => Local::from(input_dir)?.id.to_string(),
    };

    let mut user_cache = User::from(&dirs::config_json()?)?.sources.remove(&serial).unwrap_or_default();

    user_cache.serial = serial;

    Ok(user_cache)
  }

  pub fn get(&self, file_name: &str) -> Option<File> {
    self.files.get(file_name).cloned()
  }

  pub fn set(&mut self, file: &UiFile) -> Result<()> {
    self.files.insert(
      file.name()?,
      File {
        date: file.date.clone(),
        seconds: file.seconds,
        note: file.note.clone(),
        status: file.status.clone(),
      },
    );

    Ok(())
  }

  pub fn save(&self) -> Result<()> {
    let mut cache = User::from(&dirs::config_json()?)?;
    cache.sources.insert(self.serial.clone(), self.clone());
    cache.save(&dirs::config_json()?)
  }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct File {
  pub date: String,
  pub seconds: f64,
  pub note: Option<String>,
  pub status: Option<UiFileStatus>,
}
