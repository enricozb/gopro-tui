use std::{fs, sync::mpsc::Sender, thread};

use crate::{
  error::Result,
  ui::{
    events::Event,
    state::{
      destination::Destination,
      progress::Progress,
      session::{Date, File, Session, Status as FileStatus},
    },
  },
};

pub struct Writer {
  event_sender: Sender<Event>,
}

impl Writer {
  pub fn new(event_sender: Sender<Event>) -> Self {
    Self { event_sender }
  }

  pub fn spawn(&self, sessions: Vec<Session>) -> Progress {
    let files: Vec<FileToImport> = sessions
      .into_iter()
      .filter_map(|session| {
        if let Some(destination) = session.destination {
          Some((session.files, destination, session.date))
        } else {
          None
        }
      })
      .flat_map(|(files, destination, session_date)| {
        files.into_values().filter_map(move |file| {
          if file.status == Some(FileStatus::Import) {
            Some(FileToImport {
              file,
              destination: destination.clone(),
              session_date: session_date.clone(),
            })
          } else {
            None
          }
        })
      })
      .collect();

    let progress = Progress::new(files.len());
    let progress_clone = progress.clone();

    let event_sender = self.event_sender.clone();
    thread::spawn(move || match run(progress, files) {
      Ok(_) => (),
      Err(error) => event_sender.send(Event::Error(format!("spawn writer: {}", error))).unwrap(),
    });

    progress_clone
  }
}

struct FileToImport {
  file: File,
  destination: Destination,
  session_date: Date,
}

impl FileToImport {
  pub fn import(&self) -> Result<()> {
    if let Some(base) = &self.file.path.file_name() {
      let destination_dir = self.destination.abs.join(self.session_date.clone());
      if !destination_dir.exists() {
        fs::create_dir(&destination_dir)?;
      };

      fs::copy(&self.file.path, &destination_dir.join(base))?;
    }

    Ok(())
  }
}

fn run(mut progress: Progress, files: Vec<FileToImport>) -> Result<()> {
  for (file_idx, entry) in files.into_iter().enumerate() {
    progress.set_file_idx(file_idx)?;
    entry.import()?;
  }

  progress.set_done()?;

  Ok(())
}
