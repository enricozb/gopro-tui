use std::{fs::Metadata, path::PathBuf};

pub type Date = String;

pub struct Session {
  pub date: Date,
  pub files: Vec<File>,
}

pub struct File {
  pub path: PathBuf,
  pub metadata: Metadata,
  pub date: Date,
  pub seconds: f64,

  pub note: Option<String>,
}

impl File {
  pub fn new(path: PathBuf, metadata: Metadata, date: Date, seconds: f64) -> Self {
    Self {
      path,
      metadata,
      date,
      seconds,

      note: None,
    }
  }
}
