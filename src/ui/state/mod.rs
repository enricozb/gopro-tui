pub mod focus;
pub mod session;

use std::collections::BTreeMap;

use chrono::naive::NaiveDate;

use self::{focus::Focus, session::Session};

#[derive(Default)]
pub struct State {
  pub src_dir: Option<String>,
  pub dst_dir: Option<String>,

  pub focus: Focus,

  pub sessions: BTreeMap<NaiveDate, Session>,

  pub sessions_idx: usize,
  pub files_idx: usize,
}

impl State {
  pub fn toggle_focus(&mut self) {
    self.focus = match self.focus {
      Focus::Files => Focus::Sessions,
      Focus::Sessions => Focus::Files,
    }
  }

  pub fn session(&self) -> Option<&Session> {
    self.sessions.iter().nth(self.sessions_idx).map(|e| e.1)
  }
}
