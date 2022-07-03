mod datetime;
pub mod destinations;
mod ffmpeg;
mod gpmf;

use std::{path::Path, sync::mpsc::Sender, thread};

use walkdir::{DirEntry, WalkDir};

use crate::{
  cache::Source as SourceCache,
  channel::{EventChannel, ResultChannel},
  error::Result,
  events::Event,
  mode::Mode,
  ui::state::session::File,
  utils,
};

pub fn spawn(mode: &Mode, event_channel: &EventChannel, result_channel: &ResultChannel, cache: SourceCache) {
  let input_dir = mode.input_dir();
  let event_sender = event_channel.sender();
  let result_sender = result_channel.sender();

  thread::spawn(move || match run(&input_dir, &event_sender, cache) {
    Ok(_) => (),
    error => result_sender.send(error).unwrap(),
  });
}

fn run(input_dir: &Path, event_sender: &Sender<Event>, cache: SourceCache) -> Result<()> {
  for file in WalkDir::new(input_dir.join("DCIM")).into_iter().filter_map(std::result::Result::ok) {
    if !is_mp4(&file) {
      continue;
    };

    let path = file.path();
    let file_name = utils::file_name(path)?;

    let (date, seconds, note, status) = if let Some(file) = cache.get(&file_name) {
      (file.date, file.seconds, file.note, file.status)
    } else {
      let ffprobe_info = ffmpeg::ffprobe(path)?;
      let date = if let Ok(datetime) = datetime::approximate(path, &ffprobe_info) {
        datetime.naive_local().date().to_string()
      } else {
        "?".to_string()
      };

      (date, ffprobe_info.seconds, None, None)
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
