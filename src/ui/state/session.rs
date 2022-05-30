use std::{collections::BTreeMap, fs::Metadata, iter, path::PathBuf, time::SystemTime};

use crate::error::Result;

pub type Date = String;

pub struct Session {
  pub date: Date,
  pub files: BTreeMap<SystemTime, File>,
}

impl Session {
  pub fn new(date: Date, files: Vec<File>) -> Result<Self> {
    let times: Result<Vec<_>> = files.iter().map(|f| f.time()).collect();

    Ok(Self {
      date,
      files: iter::zip(times?, files).collect(),
    })
  }

  pub fn insert_file(&mut self, file: File) -> Result<()> {
    self.files.insert(file.time()?, file);

    Ok(())
  }
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

  pub fn time(&self) -> Result<SystemTime> {
    Ok(self.metadata.created()?)
  }
}
