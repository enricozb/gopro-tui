use std::path::{Path, PathBuf};

use crate::{error::Result, utils};

#[derive(PartialEq, Eq, PartialOrd, Ord)]
pub struct Destination {
  pub abs: PathBuf,
  pub base: String,
}

impl Destination {
  pub fn new(abs_path: &Path) -> Result<Self> {
    Ok(Self {
      abs: abs_path.to_path_buf(),
      base: utils::file_name(abs_path)?,
    })
  }
}
