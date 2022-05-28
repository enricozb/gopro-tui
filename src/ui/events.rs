use std::{
  sync::mpsc::{self, Receiver, Sender},
  thread,
  time::Duration,
};

use crossterm::event::{self, Event as CrosstermEvent, KeyCode, KeyEvent, KeyModifiers};

use crate::error::Result;

pub enum Event {
  Key { code: KeyCode, modifiers: KeyModifiers },
  Error(String),
  Tick,
}

pub struct Channels {
  pub sender: Sender<Event>,
  pub receiver: Receiver<Event>,
}

impl Channels {
  pub fn new() -> Self {
    let (sender, receiver) = mpsc::channel();

    Self { sender, receiver }
  }

  pub fn poll(&self) -> Result<Event> {
    Ok(self.receiver.recv()?)
  }

  pub fn spawn_ticks(&self) {
    let sender = self.sender.clone();

    thread::spawn(move || key_event_loop(&sender));
  }
}

fn key_event_loop(sender: &Sender<Event>) {
  loop {
    match key_event_tick(sender) {
      Ok(()) => (),
      Err(err) => {
        sender.send(Event::Error(err.to_string())).unwrap();
        break;
      }
    }
  }
}

static TICK_RATE: Duration = Duration::from_millis(200);

fn key_event_tick(sender: &Sender<Event>) -> Result<()> {
  if event::poll(TICK_RATE)? {
    match event::read()? {
      CrosstermEvent::Key(KeyEvent { code, modifiers }) => sender.send(Event::Key { code, modifiers })?,
      _ => (),
    };
  };

  sender.send(Event::Tick)?;

  Ok(())
}
