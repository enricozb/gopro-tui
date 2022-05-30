use std::{
  process::{Command, Stdio},
  thread,
};

use mpvipc::{Mpv, MpvCommand, PlaylistAddOptions};

use crate::{
  channel::ResultChannel,
  error::{err, Result, WrapErr},
  ui::state::session::Session,
};

const SOCKET: &'static str = "/tmp/gopro-importer-mpv-socket";

pub fn load_session(session: &Session) -> Result<()> {
  let mpv = Mpv::connect(SOCKET)?;

  for file in session.files.values() {
    mpv.run_command(MpvCommand::LoadFile {
      file: file.path.to_string_lossy().to_string(),
      option: PlaylistAddOptions::Append,
    })?;
  }

  Ok(())
}

pub fn play(file_idx: usize) -> Result<()> {
  let mpv = Mpv::connect(SOCKET)?;

  mpv.playlist_play_id(file_idx)?;

  Ok(())
}

pub fn current_position() -> Option<usize> {
  if let Some(mpv) = Mpv::connect(SOCKET).ok() {
    let pos: Result<f64> = mpv.get_property("playlist-pos").wrap_err("get property");

    match pos {
      Err(_) => None,
      Ok(idx) if idx < 0.0 => None,
      Ok(idx) => Some(idx as usize),
    }
  } else {
    None
  }
}

pub fn spawn(result_channel: &ResultChannel) {
  let result_sender = result_channel.sender();

  thread::spawn(move || result_sender.send(run_idle()).unwrap());
}

fn run_idle() -> Result<()> {
  loop {
    let status = Command::new("mpv")
      .args(["--idle", format!("--input-ipc-server={}", SOCKET).as_str()])
      .stdin(Stdio::null())
      .stdout(Stdio::null())
      .stderr(Stdio::null())
      .status()?;

    if !status.success() {
      return Err(err!("mpv exited with exit code {:?}", status.code()));
    }
  }
}
