pub mod focus;
pub mod session;

use std::collections::BTreeMap;

use chrono::naive::NaiveDate;

use self::{
  focus::Focus,
  session::{File, Session},
};
use crate::{error::Result, mpv};

#[derive(Default)]
pub struct State {
  pub src_dir: Option<String>,
  pub dst_dir: Option<String>,

  pub focus: Focus,
  pub error: Option<String>,

  pub sessions: BTreeMap<NaiveDate, Session>,

  pub sessions_idx: usize,
  pub files_idx: usize,
}

impl State {
  pub fn session(&self) -> Option<&Session> {
    self.sessions.iter().nth(self.sessions_idx).map(|e| e.1)
  }

  pub fn file(&self) -> Option<&File> {
    self.session().map(|s| &s.files[self.files_idx])
  }

  pub fn files_len(&self) -> usize {
    self.session().map_or(0, |session| session.files.len())
  }

  pub fn add_file(&mut self, file: File) {
    let date = file.datetime.naive_local().date();

    match self.sessions.get_mut(&date) {
      Some(session) => session.files.push(file),
      None => {
        self.sessions.insert(
          date,
          Session {
            date,
            files: vec![file],
          },
        );
      }
    }
  }

  pub fn error(&mut self, error: String) {
    self.error = Some(error)
  }

  pub fn toggle_focus(&mut self) {
    self.focus = match self.focus {
      Focus::Files => Focus::Sessions,
      Focus::Sessions => Focus::Files,
    }
  }

  pub fn enter(&self) -> Result<()> {
    if let Some(file) = self.file() {
      mpv::preview(file)?;
    };

    Ok(())
  }

  pub fn escape(&mut self) {
    self.error = None
  }

  pub fn list_up(&mut self) {
    match self.focus {
      Focus::Files => self.files_idx_dec(),
      Focus::Sessions => self.sessions_idx_dec(),
    };

    self.clamp_idxs();
  }

  pub fn list_down(&mut self) {
    match self.focus {
      Focus::Files => self.files_idx_inc(),
      Focus::Sessions => self.sessions_idx_inc(),
    };

    self.clamp_idxs();
  }

  pub fn clamp_idxs(&mut self) {
    self.files_idx = clamp(0, self.files_idx, self.files_len() - 1);
    self.sessions_idx = clamp(0, self.sessions_idx, self.sessions.len() - 1);
  }

  pub fn files_idx_inc(&mut self) {
    self.files_idx = clamp(0, self.files_idx.saturating_add(1), self.files_len() - 1);
  }

  pub fn files_idx_dec(&mut self) {
    self.files_idx = clamp(0, self.files_idx.saturating_sub(1), self.files_len() - 1);
  }

  pub fn sessions_idx_inc(&mut self) {
    self.sessions_idx = clamp(0, self.sessions_idx.saturating_add(1), self.sessions.len() - 1);
  }

  pub fn sessions_idx_dec(&mut self) {
    self.sessions_idx = clamp(0, self.sessions_idx.saturating_sub(1), self.sessions.len() - 1);
  }
}

fn clamp(min: usize, x: usize, max: usize) -> usize {
  if x < min {
    min
  } else if x > max {
    max
  } else {
    x
  }
}
