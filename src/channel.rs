use std::sync::mpsc::{self, Receiver, Sender};

use crate::{error::Result, ui::events::Event};

// A channel for sending UI `Event`s.
pub type EventChannel = Channel<Event>;

// A channel for sending `Result<()>`s.`
pub type ResultChannel = Channel<Result<()>>;

pub struct Channel<T> {
  pub sender: Sender<T>,
  pub receiver: Receiver<T>,
}

impl<T> Channel<T> {
  pub fn new() -> Self {
    let (sender, receiver) = mpsc::channel();
    Self { sender, receiver }
  }

  pub fn poll(&self) -> Result<T> {
    Ok(self.receiver.recv()?)
  }

  pub fn sender(&self) -> Sender<T> {
    self.sender.clone()
  }
}
