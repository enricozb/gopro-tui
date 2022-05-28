use std::sync::mpsc::{self, Receiver, Sender};

use crate::error::Result;

pub struct Channel<T> {
  pub send: Sender<T>,
  pub recv: Receiver<T>,
}

impl<T> Channel<T> {
  pub fn new() -> Self {
    let (send, recv) = mpsc::channel();
    Self { send, recv }
  }

  pub fn poll(&self) -> Result<T> {
    Ok(self.recv.recv()?)
  }

  pub fn sender(&self) -> Sender<T> {
    self.send.clone()
  }
}
