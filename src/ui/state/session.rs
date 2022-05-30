use std::{collections::BTreeMap, fs::Metadata, iter, path::PathBuf, time::SystemTime};

use crate::{error::Result, utils};

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
  pub fn time(&self) -> Result<SystemTime> {
    Ok(self.metadata.created()?)
  }

  pub fn name(&self) -> Result<String> {
    utils::file_name(&self.path)
  }
}
