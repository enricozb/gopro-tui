mod datetime;
mod ffmpeg;
mod gpmf;

use std::{path::PathBuf, sync::mpsc::Sender, thread};

use walkdir::{DirEntry, WalkDir};

use crate::{
  channel::{EventChannel, ResultChannel},
  error::Result,
  events::Event,
  ui::state::session::File,
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
  for file in WalkDir::new(src_dir.join("DCIM")).into_iter().filter_map(|e| e.ok()) {
    if !is_mp4(&file) {
      continue;
    };

    let path = file.path();

    let ffprobe_info = ffmpeg::ffprobe(path)?;
    let datetime = datetime::approximate_datetime(path, &ffprobe_info)?;

    event_sender.send(Event::File(Box::new(File::new(
      path.to_path_buf(),
      file.metadata()?,
      datetime,
      ffprobe_info.seconds,
    ))))?;
  }

  Ok(())
}

fn is_mp4(entry: &DirEntry) -> bool {
  matches!(entry.path().extension(), Some(ext) if ext == "MP4")
}
