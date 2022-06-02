use std::{
  path::{Path, PathBuf},
  process::{Child, Command, Stdio},
};

use mpvipc::{Mpv, MpvCommand, PlaylistAddOptions};

use crate::{
  error::{err, Result, WrapErr},
  ui::state::session::Session,
};

const SOCKET: &str = "/tmp/gopro-importer-mpv-socket";

pub struct Player {
  socket: PathBuf,
  process: Child,
}

impl Player {
  pub fn new() -> Result<Self> {
    let socket = PathBuf::from(SOCKET);

    Ok(Self {
      process: spawn_mpv_instance(&socket)?,
      socket,
    })
  }

  pub fn play(&mut self, file_idx: usize) -> Result<()> {
    self.mpv_connection()?.playlist_play_id(file_idx)?;

    Ok(())
  }

  pub fn is_playing(&mut self) -> bool {
    self.playlist_pos() != None
  }

  // TODO(enricozb): should this return a Result?
  pub fn playlist_pos(&mut self) -> Option<usize> {
    if let Ok(mpv) = self.mpv_connection() {
      match mpv.get_property::<f64>("playlist-pos").wrap_err("get property") {
        Err(_) => None,
        Ok(idx) if idx < 0.0 => None,
        Ok(idx) => Some(idx as usize),
      }
    } else {
      None
    }
  }

  // TODO(enricozb): should this return a Result?
  pub fn set_playlist_pos(&mut self, playlist_pos: usize) {
    if let Ok(mpv) = Mpv::connect(SOCKET) {
      mpv.set_property("playlist-pos", playlist_pos).ok();
    }
  }

  pub fn load_session(&mut self, session: &Session) -> Result<()> {
    let mpv = self.mpv_connection()?;

    // if the player is currently playing, then the player position is non-negative.
    // when clearing the playlist, mpv won't clear the currently playing item, so set
    // the playlist position to 0 and replace it on the first iteration of the loop.
    let is_playing = self.is_playing();
    if is_playing {
      mpv.set_property("playlist-pos", 0)?;
    }
    mpv.run_command(MpvCommand::PlaylistClear)?;

    for (i, file) in session.files.values().enumerate() {
      mpv.run_command(MpvCommand::LoadFile {
        file: file.path.to_string_lossy().to_string(),
        option: if i == 0 && is_playing {
          PlaylistAddOptions::Replace
        } else {
          PlaylistAddOptions::Append
        },
      })?;
    }

    Ok(())
  }

  fn mpv_connection(&mut self) -> Result<Mpv> {
    if let Some(status) = self.process.try_wait()? {
      if !status.success() {
        return Err(err!("mpv exited with exit code {:?}", status.code()));
      } else {
        self.process = spawn_mpv_instance(&self.socket)?;
      }
    };

    Ok(Mpv::connect(SOCKET)?)
  }
}

fn spawn_mpv_instance<P: AsRef<Path>>(socket: P) -> Result<Child> {
  Ok(
    Command::new("mpv")
      .args(["--idle", format!("--input-ipc-server={}", socket.as_ref().display()).as_str()])
      .stdin(Stdio::null())
      .stdout(Stdio::null())
      .stderr(Stdio::null())
      .spawn()?,
  )
}
