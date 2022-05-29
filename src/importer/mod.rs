use std::{path::PathBuf, sync::mpsc::Sender, thread};

use crate::{
  channel::{EventChannel, ResultChannel},
  error::Result,
  events::Event,
};

pub fn spawn(src_dir: PathBuf, event_channel: &EventChannel, result_channel: &ResultChannel) {
  let event_sender = event_channel.sender();
  let result_sender = result_channel.sender();

  thread::spawn(move || match run(src_dir, event_sender) {
    Ok(_) => (),
    error => result_sender.send(error).unwrap(),
  });
}

fn run(src_dir: PathBuf, event_sender: Sender<Event>) -> Result<()> {
  Ok(())
}
