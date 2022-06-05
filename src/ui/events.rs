use std::{sync::mpsc::Sender, thread, time::Duration};

use crossterm::event::{self, Event as CrosstermEvent, KeyCode, KeyEvent, KeyModifiers};

use super::state::{destination::Destination, session::File};
use crate::{
  channel::{EventChannel, ResultChannel},
  error::Result,
};

pub enum Event {
  Key { code: KeyCode, modifiers: KeyModifiers },
  File(Box<File>),
  Destination(Destination),
  Error(String),
  Tick,
}

pub fn spawn(event_channel: &EventChannel, result_channel: &ResultChannel) {
  let event_sender = event_channel.sender();
  let result_sender = result_channel.sender();

  thread::spawn(move || result_sender.send(key_event_loop(&event_sender)));
}

fn key_event_loop(event_sender: &Sender<Event>) -> Result<()> {
  loop {
    match event_tick() {
      Ok(event) => event_sender.send(event)?,
      Err(error) => event_sender.send(Event::Error(error.to_string()))?,
    }
  }
}

const TICK_RATE: Duration = Duration::from_millis(50);

fn event_tick() -> Result<Event> {
  let event = if event::poll(TICK_RATE)? {
    match event::read()? {
      CrosstermEvent::Key(KeyEvent { code, modifiers }) => Event::Key { code, modifiers },
      _ => Event::Tick,
    }
  } else {
    Event::Tick
  };

  Ok(event)
}
