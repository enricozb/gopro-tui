pub mod focus;
pub mod session;

use std::collections::BTreeMap;

use self::{
  focus::Focus,
  session::{Date, File, Session},
};
use crate::{error::Result, mpv};

#[derive(Default)]
pub struct State {
  pub src_dir: Option<String>,
  pub dst_dir: Option<String>,

  pub focus: Focus,
  pub input: Option<String>,
  pub error: Option<String>,

  pub sessions: BTreeMap<Date, Session>,

  pub session_idx: usize,
  pub file_idx: usize,
}

impl State {
  pub fn session(&self) -> Option<&Session> {
    self.sessions.values().nth(self.session_idx)
  }

  pub fn session_mut(&mut self) -> Option<&mut Session> {
    self.sessions.values_mut().nth(self.session_idx)
  }

  pub fn file(&self) -> Option<&File> {
    self.session().and_then(|s| s.files.values().nth(self.file_idx))
  }

  pub fn file_mut(&mut self) -> Option<&mut File> {
    let file_idx = self.file_idx;

    self.session_mut().and_then(|s| s.files.values_mut().nth(file_idx))
  }

  pub fn files_len(&self) -> usize {
    self.session().map_or(0, |session| session.files.len())
  }

  pub fn popup(&self) -> Popup {
    match (&self.input, &self.error) {
      (Some(_), None) => Popup::Input,
      (None, Some(_)) => Popup::Error,
      _ => Popup::None,
    }
  }

  pub fn add_file(&mut self, file: File) -> Result<()> {
    match self.sessions.get_mut(&file.date) {
      Some(session) => session.insert_file(file)?,
      None => {
        self
          .sessions
          .insert(file.date.clone(), Session::new(file.date.clone(), vec![file])?);
      }
    };

    Ok(())
  }

  pub fn input(&mut self) {
    self.input = self.file().and_then(|f| f.note.clone()).or_else(|| Some("".to_string()));
  }

  pub fn input_char(&mut self, c: char) {
    if let Some(s) = self.input.as_mut() {
      s.push(c);
    }
  }

  pub fn input_del(&mut self) {
    if let Some(s) = self.input.as_mut() {
      s.pop();
    }
  }

  pub fn error(&mut self, error: String) {
    self.error = Some(error);
  }

  pub fn toggle_focus(&mut self) {
    self.focus = match self.focus {
      Focus::Files => Focus::Sessions,
      Focus::Sessions => Focus::Files,
    }
  }

  pub fn enter(&self) -> Result<()> {
    if let Some(session) = self.session() {
      mpv::load_session(session)?;
      mpv::play(self.file_idx)?;
    };

    Ok(())
  }

  pub fn escape(&mut self) {
    self.input = None;
    self.error = None;
  }

  pub fn list_up(&mut self) {
    match self.focus {
      Focus::Files => self.file_idx_dec(),
      Focus::Sessions => self.session_idx_dec(),
    };

    self.clamp_idxs();

    self.f().ok();
  }

  pub fn list_down(&mut self) {
    match self.focus {
      Focus::Files => self.file_idx_inc(),
      Focus::Sessions => self.session_idx_inc(),
    };

    self.clamp_idxs();

    self.f().ok();
  }

  pub fn clamp_idxs(&mut self) {
    self.file_idx = clamp(0, self.file_idx, self.files_len() - 1);
    self.session_idx = clamp(0, self.session_idx, self.sessions.len() - 1);
  }

  pub fn file_idx_inc(&mut self) {
    self.file_idx = clamp(0, self.file_idx.saturating_add(1), self.files_len() - 1);
  }

  pub fn file_idx_dec(&mut self) {
    self.file_idx = clamp(0, self.file_idx.saturating_sub(1), self.files_len() - 1);
  }

  pub fn session_idx_inc(&mut self) {
    self.session_idx = clamp(0, self.session_idx.saturating_add(1), self.sessions.len() - 1);
  }

  pub fn session_idx_dec(&mut self) {
    self.session_idx = clamp(0, self.session_idx.saturating_sub(1), self.sessions.len() - 1);
  }

  pub fn write_note(&mut self) {
    if let Some(input) = self.input.clone() {
      if let Some(ref mut file) = self.file_mut() {
        file.note = Some(input)
      }
    };

    self.input = None;
  }

  pub fn f(&self) -> Result<()> {
    match self.focus {
      Focus::Files => {
        if mpv::is_playing() {
          mpv::set_position(self.file_idx)
        }
      }
      Focus::Sessions => {
        if let Some(session) = self.session() {
          mpv::load_session(session)?
        }
      }
    };

    Ok(())
  }

  pub fn sync(&mut self) -> Result<()> {
    if let Some(idx) = mpv::get_position() {
      self.file_idx = idx;
    }

    Ok(())
  }
}

pub enum Popup {
  None,
  Input,
  Error,
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
