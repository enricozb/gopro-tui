pub mod destination;
pub mod focus;
pub mod progress;
pub mod session;

use std::{
  collections::{BTreeMap, BTreeSet},
  path::PathBuf,
  sync::mpsc::Sender,
};

use self::{
  destination::Destination,
  focus::Focus,
  progress::Progress,
  session::{Date, File, Session, Status},
};
use super::{events::Event, render::search};
use crate::{error::Result, mode::Mode, mpv::Player, writer::Writer};

pub struct State {
  pub mode: Mode,

  pub focus: Focus,
  pub input: Option<String>,
  pub search: Option<String>,
  pub error: Option<String>,
  pub progress: Option<Progress>,

  pub sessions: BTreeMap<Date, Session>,
  pub destinations: BTreeMap<PathBuf, BTreeSet<Destination>>,
  pub destination_sessions: BTreeMap<PathBuf, BTreeSet<PathBuf>>,

  pub session_idx: usize,
  pub file_idx: usize,

  pub player: Player,
  pub writer: Writer,
}

impl State {
  pub fn new(mode: Mode, event_sender: Sender<Event>) -> Result<Self> {
    Ok(Self {
      mode,

      focus: Focus::default(),
      input: None,
      search: None,
      error: None,
      progress: None,

      sessions: BTreeMap::new(),
      destinations: BTreeMap::new(),
      destination_sessions: BTreeMap::new(),

      session_idx: 0,
      file_idx: 0,

      player: Player::new()?,
      writer: Writer::new(event_sender),
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

  pub fn new_destination_sessions(&self) -> BTreeMap<PathBuf, BTreeSet<PathBuf>> {
    let mut destination_sessions = BTreeMap::<PathBuf, BTreeSet<PathBuf>>::new();

    for session in self.sessions.values() {
      let destination_path = if let Some(destination) = &session.destination {
        &destination.abs
      } else {
        continue;
      };

      let session_destination_path = destination_path.join(session.date.clone());

      if let Some(sessions) = destination_sessions.get_mut(destination_path) {
        sessions.insert(session_destination_path);
      } else {
        destination_sessions.insert(destination_path.clone(), BTreeSet::from([session_destination_path]));
      }
    }

    destination_sessions
  }

  pub fn popup(&self) -> Popup {
    match (&self.input, &self.search, &self.error, &self.progress) {
      (Some(_), _, _, _) => Popup::Input,
      (_, Some(_), _, _) => Popup::Search,
      (_, _, Some(_), _) => Popup::Error,
      (_, _, _, Some(_)) => Popup::Progress,
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

  pub fn add_destination_session(&mut self, path: PathBuf) {
    if let Some(parent) = path.parent() {
      if let Some(destination_sessions) = self.destination_sessions.get_mut(parent) {
        destination_sessions.insert(path);
      } else {
        self.destination_sessions.insert(parent.to_path_buf(), BTreeSet::from([path]));
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

  pub fn import(&mut self) {
    let progress = self.writer.spawn(self.sessions.clone().into_values().collect());
    self.progress = Some(progress);
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

    if let Some(progress) = &self.progress {
      if progress.bare().unwrap().done {
        self.progress = None;
      }
    }
  }
}

pub enum Popup {
  None,
  Input,
  Search,
  Error,
  Progress,
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
