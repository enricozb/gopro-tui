mod version;

use std::{
  collections::BTreeMap,
  fs::{self, File as StdFile},
  io::BufReader,
  path::Path,
};

use serde::{Deserialize, Serialize};

use self::version::{SerialNumber, Version};
use crate::{dirs, error::Result, ui::state::session::File as UiFile};

const VERSION: &str = env!("CARGO_PKG_VERSION");

type FileName = String;

#[derive(Serialize, Deserialize, Default)]
struct Cache {
  version: String,
  entries: BTreeMap<SerialNumber, CacheEntry>,
}

impl Cache {
  fn from(path: &Path) -> Result<Self> {
    if path.exists() {
      let file = StdFile::open(path)?;
      let reader = BufReader::new(file);

      Ok(serde_json::from_reader(reader)?)
    } else {
      Ok(Self {
        version: VERSION.to_string(),
        ..Self::default()
      })
    }
  }

  fn save(self, path: &Path) -> Result<()> {
    let json = serde_json::to_string_pretty(&self)?;
    Ok(fs::write(path, json)?)
  }
}

#[derive(Serialize, Deserialize, Clone, Default)]
pub struct CacheEntry {
  #[serde(skip)]
  serial: SerialNumber,

  files: BTreeMap<FileName, File>,
}

impl CacheEntry {
  pub fn from(gopro_path: &Path) -> Result<Self> {
    let version = Version::from(gopro_path)?;

    let mut cache_entry = Cache::from(&dirs::config_json()?)?
      .entries
      .get(&version.camera_serial_number)
      .cloned()
      .unwrap_or_default();

    cache_entry.serial = version.camera_serial_number;

    Ok(cache_entry)
  }

  pub fn get(&self, file_name: &str) -> Option<File> {
    self.files.get(file_name).cloned()
  }

  pub fn set(&mut self, file: &UiFile) -> Result<()> {
    self.files.insert(
      file.name()?,
      File {
        note: file.note.clone(),
        date: file.date.clone(),
        seconds: file.seconds,
      },
    );

    Ok(())
  }

  pub fn save(&self) -> Result<()> {
    let mut cache = Cache::from(&dirs::config_json()?)?;
    cache.entries.insert(self.serial.clone(), self.clone());
    cache.save(&dirs::config_json()?)
  }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct File {
  pub note: Option<String>,
  pub date: String,
  pub seconds: f64,
}
