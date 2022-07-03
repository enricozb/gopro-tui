use std::sync::{Arc, Mutex};

use crate::error::{err, Result};

#[derive(Clone)]
pub struct Progress {
  inner: Arc<Mutex<Bare>>,
}

impl Progress {
  pub fn new(file_total: usize) -> Self {
    Self {
      inner: Arc::new(Mutex::new(Bare::new(file_total))),
    }
  }

  pub fn set_file_idx(&mut self, file_idx: usize) -> Result<()> {
    let mut inner = self.inner.lock().map_err(|error| err!("lock: {}", error))?;
    (*inner).file_idx = file_idx;

    Ok(())
  }

  pub fn set_done(&mut self) -> Result<()> {
    let mut inner = self.inner.lock().map_err(|error| err!("lock: {}", error))?;
    (*inner).done = true;

    Ok(())
  }

  pub fn bare(&self) -> Result<Bare> {
    let inner = self.inner.lock().map_err(|error| err!("lock: {}", error))?;
    Ok(inner.clone())
  }
}

#[derive(Clone)]
pub struct Bare {
  pub file_idx: usize,
  pub file_total: usize,

  pub done: bool,
}

impl Bare {
  pub fn new(file_total: usize) -> Self {
    Self {
      file_idx: 0,
      file_total,

      done: false,
    }
  }
}
