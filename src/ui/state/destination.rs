use std::path::{Path, PathBuf};

use crate::{error::Result, utils};

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone)]
pub struct Destination {
  pub abs: PathBuf,
  pub rel: String,
  pub base: String,
}

impl Destination {
  pub fn new(abs_path: &Path, root: &Path) -> Result<Self> {
    Ok(Self {
      abs: abs_path.to_path_buf(),
      rel: abs_path.strip_prefix(root)?.to_string_lossy().to_string(),
      base: utils::file_name(abs_path)?,
    })
  }
}
