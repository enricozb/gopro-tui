use std::{fs, path::Path};

use serde::Deserialize;

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
