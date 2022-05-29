pub mod events;
mod render;
pub mod state;

use std::{
  io::{self, Stdout},
  thread,
};

use crossterm::{
  event::{
    DisableMouseCapture, EnableMouseCapture,
    KeyCode::{Backspace, Char, Enter, Esc},
  },
  execute,
  terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use stable_eyre::eyre::eyre;
use tui::{backend::CrosstermBackend, Terminal};

use self::{
  events::Event,
  render::sections,
  state::{focus::Focus, session::File, Popup, State},
};
use crate::{
  cache::CacheEntry,
  channel::{EventChannel, ResultChannel},
  error::Result,
};

pub struct Ui {
  cache: CacheEntry,
  event_channel: EventChannel,
  state: State,
  terminal: Terminal<CrosstermBackend<Stdout>>,
}

impl Ui {
  pub fn new(cache: CacheEntry, event_channel: EventChannel) -> Result<Self> {
    Ok(Self {
      cache,
      event_channel,
      state: State::default(),
      terminal: Terminal::new(CrosstermBackend::new(io::stdout()))?,
    })
  }

  pub fn run(&mut self) -> Result<()> {
    Self::setup()?;

    loop {
      self.render()?;

      match (&self.state.focus, &self.state.popup(), self.event_channel.poll()?) {
        (_, Popup::Input, Event::Key { code: Char(c), .. }) => self.state.input_char(c),
        (_, Popup::Input, Event::Key { code: Backspace, .. }) => self.state.input_del(),

        (_, Popup::None, Event::Key { code: Char('q'), .. }) => break,

        (_, Popup::None, Event::Key { code: Char('k'), .. }) => self.state.list_up(),
        (_, Popup::None, Event::Key { code: Char('j'), .. }) => self.state.list_down(),
        (_, Popup::None, Event::Key { code: Char('h' | 'l'), .. }) => self.state.toggle_focus(),

        (Focus::Files, Popup::None, Event::Key { code: Char('n'), .. }) => self.state.input(),

        (Focus::Files, Popup::None, Event::Key { code: Enter, .. }) => {
          if let Err(error) = self.state.enter() {
            self.event_channel.sender.send(Event::Error(format!("{:?}", error)))?;
          }
        }

        (_, Popup::Input, Event::Key { code: Enter, .. }) => {
          self.state.write_note();
          if let Some(File {
            path, note: Some(note), ..
          }) = self.state.file()
          {
            self.cache.notes.insert(
              path
                .file_name()
                .ok_or_else(|| eyre!("file has no basename: {}", path.display()))?
                .to_string_lossy()
                .to_string(),
              note.clone(),
            );
          };
        }

        (_, _, Event::Key { code: Esc, .. }) => self.state.escape(),

        (_, _, Event::File(mut file)) => {
          if let Some(file_name) = file.path.file_name() {
            file.note = self.cache.notes.get(&file_name.to_string_lossy().into_owned()).cloned()
          }

          self.state.add_file(*file)
        }
        (_, _, Event::Error(error)) => self.state.error(error),

        _ => (),
      }
    }

    Ok(())
  }

  fn setup() -> Result<()> {
    enable_raw_mode()?;
    execute!(io::stdout(), EnterAlternateScreen, EnableMouseCapture)?;

    Ok(())
  }

  fn render(&mut self) -> Result<()> {
    self.terminal.draw(|f| sections::render(f, &self.state))?;

    Ok(())
  }

  fn cleanup(&mut self) -> Result<()> {
    self.cache.save()?;

    disable_raw_mode()?;
    execute!(self.terminal.backend_mut(), LeaveAlternateScreen, DisableMouseCapture)?;
    self.terminal.show_cursor()?;

    Ok(())
  }
}

impl Drop for Ui {
  fn drop(&mut self) {
    self.cleanup().unwrap();
  }
}

pub fn spawn(event_channel: EventChannel, result_channel: &ResultChannel, cache: CacheEntry) {
  let result_sender = result_channel.sender();

  thread::spawn(move || result_sender.send(run(cache, event_channel)).unwrap());
}

fn run(cache: CacheEntry, event_channel: EventChannel) -> Result<()> {
  Ui::new(cache, event_channel)?.run()
}
