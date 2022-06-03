mod datetime;
mod ffmpeg;
mod gpmf;

use std::{
  path::{Path, PathBuf},
  sync::mpsc::Sender,
  thread,
};

use walkdir::{DirEntry, WalkDir};

use crate::{
  cache::CacheEntry,
  channel::{EventChannel, ResultChannel},
  error::Result,
  events::Event,
  ui::state::session::File,
  utils,
};

pub fn spawn(src_dir: PathBuf, event_channel: &EventChannel, result_channel: &ResultChannel, cache: CacheEntry) {
  let event_sender = event_channel.sender();
  let result_sender = result_channel.sender();

  thread::spawn(move || match run(&src_dir, &event_sender, cache) {
    Ok(_) => (),
    error => result_sender.send(error).unwrap(),
  });
}

fn run(src_dir: &Path, event_sender: &Sender<Event>, cache: CacheEntry) -> Result<()> {
  for file in WalkDir::new(src_dir.join("DCIM")).into_iter().filter_map(std::result::Result::ok) {
    if !is_mp4(&file) {
      continue;
    };

    let path = file.path();
    let file_name = utils::file_name(path)?;

    let (date, seconds, note, status) = if let Some(file) = cache.get(&file_name) {
      (file.date, file.seconds, file.note, file.status)
    } else {
      let ffprobe_info = ffmpeg::ffprobe(path)?;
      (
        datetime::approximate(path, &ffprobe_info)?.naive_local().date().to_string(),
        ffprobe_info.seconds,
        None,
        None,
      )
    };

    event_sender.send(Event::File(Box::new(File {
      path: path.to_path_buf(),
      metadata: file.metadata()?,
      date,
      seconds,
      note,
      status,
    })))?;
  }

  Ok(())
}

fn is_mp4(entry: &DirEntry) -> bool {
  matches!(entry.path().extension(), Some(ext) if ext == "MP4")
}
