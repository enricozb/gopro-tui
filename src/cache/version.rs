use std::{fs, path::Path};

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::Result;

pub type SerialNumber = String;

#[derive(Deserialize)]
pub struct Version {
  #[serde(alias = "info version")]
  pub info_version: String,

  #[serde(alias = "firmware version")]
  pub firmware_version: String,

  #[serde(alias = "wifi mac")]
  pub wifi_mac: String,

  #[serde(alias = "camera type")]
  pub camera_type: String,

  #[serde(alias = "camera serial number")]
  pub camera_serial_number: String,
}

impl Version {
  pub fn from(path: &Path) -> Result<Self> {
    let version_json = fs::read_to_string(path.join("MISC/version.txt"))?;

    Ok(serde_json::from_str(&version_json.replace(",\n}", "}"))?)
  }
}

#[derive(Serialize, Deserialize)]
pub struct Local {
  pub id: Uuid,
}

impl Default for Local {
  fn default() -> Self {
    Self { id: Uuid::new_v4() }
  }
}

impl Local {
  pub fn from(path: &Path) -> Result<Self> {
    let local_version = path.join("version.txt");

    if local_version.exists() {
      let local_version_json = fs::read_to_string(local_version)?;
      Ok(serde_json::from_str(&local_version_json.replace(",\n}", "}"))?)
    } else {
      let local = Local::default();

      // save newly created local version.txt
      let json = serde_json::to_string_pretty(&local)?;
      fs::write(local_version, json)?;

      Ok(local)
    }
  }
}
