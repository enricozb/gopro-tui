pub mod colors;
pub mod events;
mod render;
pub mod state;

use std::{
  io::{self, Stdout},
  thread,
  time::{Duration, SystemTime},
};

use crossterm::{
  event::{
    DisableMouseCapture, EnableMouseCapture,
    KeyCode::{Backspace, Char, Enter, Esc},
  },
  execute,
  terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use tui::{backend::CrosstermBackend, Terminal};

use self::{
  events::Event,
  render::sections,
  state::{focus::Focus, Popup, State},
};
use crate::{
  cache::Source as SourceCache,
  channel::{EventChannel, ResultChannel},
  error::Result,
  mode::Mode,
};

const RENDER_MIN_ELAPSED: Duration = Duration::from_millis(50);

pub struct Ui {
  cache: SourceCache,
  event_channel: EventChannel,
  state: State,
  terminal: Terminal<CrosstermBackend<Stdout>>,
}

impl Ui {
  pub fn new(mode: Mode, cache: SourceCache, event_channel: EventChannel) -> Result<Self> {
    Ok(Self {
      cache,
      event_channel,
      state: State::new(mode)?,
      terminal: Terminal::new(CrosstermBackend::new(io::stdout()))?,
    })
  }

  pub fn run(&mut self) -> Result<()> {
    Self::setup()?;

    let mut time_at_last_render = SystemTime::UNIX_EPOCH;

    loop {
      if time_at_last_render.elapsed().unwrap() > RENDER_MIN_ELAPSED {
        time_at_last_render = SystemTime::now();
        self.render()?;
      }

      match (&self.state.focus, &self.state.popup(), self.event_channel.poll()?) {
        (_, Popup::None, Event::Key { code: Char('q'), .. }) => break,

        (_, Popup::None, Event::Key { code: Char('k'), .. }) => self.state.list_up(),
        (_, Popup::None, Event::Key { code: Char('j'), .. }) => self.state.list_down(),
        (_, Popup::None, Event::Key { code: Char('h' | 'l'), .. }) => self.state.toggle_focus(),

        (Focus::Sessions, Popup::None, Event::Key { code: Char('n'), .. }) => self.state.search(),

        (Focus::Files, Popup::None, Event::Key { code: Char('a'), .. }) => {
          self.state.toggle_file_import();
          self.update_file_cache()?;
        }
        (Focus::Files, Popup::None, Event::Key { code: Char('d'), .. }) => {
          self.state.toggle_file_ignore();
          self.update_file_cache()?;
        }
        (Focus::Files, Popup::None, Event::Key { code: Char('n'), .. }) => self.state.input(),
        (Focus::Files, Popup::None, Event::Key { code: Enter, .. }) => {
          if let Err(error) = self.state.preview_file() {
            self.event_channel.sender.send(Event::Error(format!("{:?}", error)))?;
          }
        }

        (_, Popup::Search, Event::Key { code: Char(c), .. }) => self.state.search_char(c),
        (_, Popup::Search, Event::Key { code: Backspace, .. }) => self.state.search_del(),
        (_, Popup::Search, Event::Key { code: Enter, .. }) => {
          self.state.set_session_destination();
          self.update_session_destination_cache()?;
        }

        (_, Popup::Input, Event::Key { code: Char(c), .. }) => self.state.input_char(c),
        (_, Popup::Input, Event::Key { code: Backspace, .. }) => self.state.input_del(),
        (_, Popup::Input, Event::Key { code: Enter, .. }) => {
          self.state.write_note();
          self.update_file_cache()?;
        }

        (_, _, Event::Key { code: Esc, .. }) => self.state.escape(),

        (_, _, Event::File(file)) => {
          let destination = self.cache.get_session_destination(&file.date);

          self.cache.set(&file)?;
          self.state.add_file(*file, destination)?;
          self.cache.save()?;
        }

        (_, _, Event::Destination(destination)) => {
          self.state.add_destination(destination);
        }

        (_, _, Event::Error(error)) => self.state.error(error),

        (_, _, Event::Tick) => self.state.sync(),

        _ => (),
      }
    }

    Ok(())
  }

  fn update_file_cache(&mut self) -> Result<()> {
    if let Some(file) = self.state.file() {
      self.cache.set(file)?;
      self.cache.save()?;
    };

    Ok(())
  }

  fn update_session_destination_cache(&mut self) -> Result<()> {
    if let Some(session) = self.state.session() {
      self.cache.set_session_destination(session);
      self.cache.save()?;
    };

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

pub fn spawn(mode: Mode, event_channel: EventChannel, result_channel: &ResultChannel, cache: SourceCache) {
  let result_sender = result_channel.sender();

  thread::spawn(move || result_sender.send(run(mode, cache, event_channel)).unwrap());
}

fn run(mode: Mode, cache: SourceCache, event_channel: EventChannel) -> Result<()> {
  Ui::new(mode, cache, event_channel)?.run()
}
