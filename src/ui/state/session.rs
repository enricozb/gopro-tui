use std::{fs::Metadata, path::PathBuf};

use chrono::{naive::NaiveDate, offset::FixedOffset, DateTime};

pub struct Session {
  pub date: NaiveDate,
  pub files: Vec<File>,
}

pub struct File {
  pub path: PathBuf,
  pub metadata: Metadata,
  pub datetime: DateTime<FixedOffset>,
  pub seconds: f64,

  pub note: Option<String>,
}

impl File {
  pub fn new(path: PathBuf, metadata: Metadata, datetime: DateTime<FixedOffset>, seconds: f64) -> Self {
    Self {
      path,
      metadata,
      datetime,
      seconds,

      note: None,
    }
  }
}
