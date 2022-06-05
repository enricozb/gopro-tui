pub mod destination;
pub mod focus;
pub mod session;

use std::{
  collections::{BTreeMap, BTreeSet},
  path::PathBuf,
};

use self::{
  destination::Destination,
  focus::Focus,
  session::{Date, File, Session, Status},
};
use super::render::search;
use crate::{error::Result, mode::Mode, mpv::Player};

pub struct State {
  pub mode: Mode,

  pub focus: Focus,
  pub input: Option<String>,
  pub search: Option<String>,
  pub error: Option<String>,

  pub sessions: BTreeMap<Date, Session>,
  pub destinations: BTreeMap<PathBuf, BTreeSet<Destination>>,

  pub session_idx: usize,
  pub file_idx: usize,

  pub player: Player,
}

impl State {
  pub fn new(mode: Mode) -> Result<Self> {
    Ok(Self {
      mode,

      focus: Focus::default(),
      input: None,
      search: None,
      error: None,

      sessions: BTreeMap::new(),
      destinations: BTreeMap::new(),

      session_idx: 0,
      file_idx: 0,

      player: Player::new()?,
    })
  }

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

  pub fn destinations(&self) -> impl Iterator<Item = &Destination> {
    self.destinations.values().flat_map(BTreeSet::iter)
  }

  pub fn popup(&self) -> Popup {
    match (&self.input, &self.search, &self.error) {
      (Some(_), _, _) => Popup::Input,
      (_, Some(_), _) => Popup::Search,
      (_, _, Some(_)) => Popup::Error,
      _ => Popup::None,
    }
  }

  pub fn add_file(&mut self, file: File, destination: Option<Destination>) -> Result<()> {
    match self.sessions.get_mut(&file.date) {
      Some(session) => session.insert_file(file)?,
      None => {
        self
          .sessions
          .insert(file.date.clone(), Session::new(file.date.clone(), vec![file], destination)?);
      }
    };

    Ok(())
  }

  pub fn add_destination(&mut self, destination: Destination) {
    if let Some(parent) = destination.abs.parent() {
      if let Some(destinations) = self.destinations.get_mut(parent) {
        destinations.insert(destination);
      } else {
        self.destinations.insert(parent.to_path_buf(), BTreeSet::from([destination]));
      }
    }
  }

  pub fn toggle_file_import(&mut self) {
    if let Some(file) = self.file_mut() {
      file.status = if file.status == Some(Status::Import) {
        None
      } else {
        Some(Status::Import)
      }
    }
  }

  pub fn toggle_file_ignore(&mut self) {
    if let Some(file) = self.file_mut() {
      file.status = if file.status == Some(Status::Ignore) {
        None
      } else {
        Some(Status::Ignore)
      }
    }
  }

  pub fn input(&mut self) {
    self.input = self.file().and_then(|f| f.note.clone()).or_else(|| Some("".to_string()));
  }

  pub fn input_char(&mut self, c: char) {
    if let Some(input) = self.input.as_mut() {
      if input.len() < 64 {
        input.push(c);
      }
    }
  }

  pub fn input_del(&mut self) {
    if let Some(input) = self.input.as_mut() {
      input.pop();
    }
  }

  pub fn search(&mut self) {
    self.search = Some("".to_string());
  }

  pub fn search_char(&mut self, c: char) {
    if let Some(search) = self.search.as_mut() {
      if search.len() < 64 {
        search.push(c);
      }
    }
  }

  pub fn search_del(&mut self) {
    if let Some(search) = self.search.as_mut() {
      search.pop();
    }
  }

  pub fn set_session_destination(&mut self) {
    if let Some(search) = self.search.clone() {
      let destination = if let Some(search_match) = search::sorted(search, self.destinations()).get(0) {
        search_match.destination.clone()
      } else {
        return;
      };

      if let Some(session) = self.session_mut() {
        session.destination = Some(destination);
      }
    }

    self.search = None;
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

  pub fn preview_file(&mut self) -> Result<()> {
    if let Some(session) = self.session().cloned() {
      self.player.load_session(&session)?;
      self.player.play(self.file_idx)?;
    };

    Ok(())
  }

  pub fn escape(&mut self) {
    self.input = None;
    self.search = None;
    self.error = None;
  }

  pub fn list_up(&mut self) {
    match self.focus {
      Focus::Files => self.file_idx_dec(),
      Focus::Sessions => self.session_idx_dec(),
    };

    self.clamp_idxs();

    self.update_player().ok();
  }

  pub fn list_down(&mut self) {
    match self.focus {
      Focus::Files => self.file_idx_inc(),
      Focus::Sessions => self.session_idx_inc(),
    };

    self.clamp_idxs();

    self.update_player().ok();
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
        file.note = if input.is_empty() { None } else { Some(input) };
      }
    };

    self.input = None;
  }

  pub fn update_player(&mut self) -> Result<()> {
    match self.focus {
      Focus::Files => {
        if self.player.is_playing() {
          self.player.set_playlist_pos(self.file_idx);
        }
      }
      Focus::Sessions => {
        if let Some(session) = self.session().cloned() {
          self.player.load_session(&session)?;
        }
      }
    };

    Ok(())
  }

  pub fn sync(&mut self) {
    if let Some(idx) = self.player.playlist_pos() {
      self.file_idx = idx;
    }
  }
}

pub enum Popup {
  None,
  Input,
  Search,
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
