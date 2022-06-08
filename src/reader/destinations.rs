use std::{path::Path, sync::mpsc::Sender, thread};

use regex::Regex;
use walkdir::WalkDir;

use crate::{
  channel::{EventChannel, ResultChannel},
  error::Result,
  events::Event,
  mode::Mode,
  ui::state::destination::Destination,
};

pub fn spawn(mode: &Mode, event_channel: &EventChannel, result_channel: &ResultChannel) {
  if let Mode::Importing { output_dir, .. } = mode {
    let output_dir = output_dir.clone();
    let event_sender = event_channel.sender();
    let result_sender = result_channel.sender();

    thread::spawn(move || match run(&output_dir, &event_sender) {
      Ok(_) => (),
      error => result_sender.send(error).unwrap(),
    });
  }
}

fn run(output_dir: &Path, event_sender: &Sender<Event>) -> Result<()> {
  let date_re = Regex::new(r"^\d{4}-\d{2}-\d{2}$").unwrap();

  let mut it = WalkDir::new(output_dir).into_iter();

  loop {
    let entry = match it.next() {
      Some(Ok(entry)) => entry,
      Some(Err(_)) => continue,
      None => break,
    };

    if let Some(file_name) = entry.file_name().to_str() {
      if date_re.is_match(file_name) && entry.file_type().is_dir() {
        event_sender.send(Event::DestinationSession {
          path: entry.path().to_path_buf(),
        })?;
        it.skip_current_dir();
        continue;
      }
    }

    event_sender.send(Event::Destination(Destination::new(entry.path(), output_dir)?))?;
  }

  Ok(())
}
