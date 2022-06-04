use std::{
  collections::BTreeMap,
  fs::{self, File as StdFile},
  io::BufReader,
  path::Path,
};

use serde::{Deserialize, Serialize};

use super::{version::SerialNumber, Source};
use crate::error::Result;

const VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Serialize, Deserialize)]
pub struct User {
  pub version: String,
  pub sources: BTreeMap<SerialNumber, Source>,
}

impl Default for User {
  fn default() -> Self {
    Self {
      version: VERSION.to_string(),
      sources: BTreeMap::default(),
    }
  }
}

impl User {
  pub fn from(path: &Path) -> Result<Self> {
    let file = StdFile::open(path)?;
    let reader = BufReader::new(file);

    Ok(serde_json::from_reader(reader)?)
  }

  pub fn save(self, path: &Path) -> Result<()> {
    let json = serde_json::to_string_pretty(&self)?;
    Ok(fs::write(path, json)?)
  }
}
