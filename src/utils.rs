use std::path::Path;

use crate::error::{err, Result};

pub fn file_name(path: &Path) -> Result<String> {
  Ok(
    path
      .file_name()
      .ok_or_else(|| err!("file has no basename: {}", path.display()))?
      .to_string_lossy()
      .to_string(),
  )
}
