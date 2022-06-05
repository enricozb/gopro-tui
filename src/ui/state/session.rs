use std::{collections::BTreeMap, fs::Metadata, iter, path::PathBuf, time::SystemTime};

use serde::{Deserialize, Serialize};

use super::destination::Destination;
use crate::{error::Result, utils};

pub type Date = String;

#[derive(Clone)]
pub struct Session {
  pub date: Date,
  pub files: BTreeMap<SystemTime, File>,

  pub destination: Option<Destination>,
}

impl Session {
  pub fn new(date: Date, files: Vec<File>, destination: Option<Destination>) -> Result<Self> {
    let times: Result<Vec<_>> = files.iter().map(File::time).collect();

    Ok(Self {
      date,
      files: iter::zip(times?, files).collect(),

      destination,
    })
  }

  pub fn insert_file(&mut self, file: File) -> Result<()> {
    self.files.insert(file.time()?, file);

    Ok(())
  }
}

#[derive(Clone)]
pub struct File {
  pub path: PathBuf,
  pub metadata: Metadata,
  pub date: Date,
  pub seconds: f64,

  pub note: Option<String>,
  pub status: Option<Status>,
}

impl File {
  pub fn time(&self) -> Result<SystemTime> {
    Ok(self.metadata.created()?)
  }

  pub fn name(&self) -> Result<String> {
    utils::file_name(&self.path)
  }
}

#[derive(Clone, PartialEq, Eq, Deserialize, Serialize)]
pub enum Status {
  Import,
  Ignore,
}
