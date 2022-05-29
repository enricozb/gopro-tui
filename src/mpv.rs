use std::process::{Command, Stdio};

use stable_eyre::eyre::WrapErr;

use crate::{error::Result, ui::state::session::File};

pub fn preview(file: &File) -> Result<()> {
  Command::new("mpv")
    .arg(file.path.clone())
    .stdin(Stdio::null())
    .stdout(Stdio::null())
    .stderr(Stdio::null())
    .spawn()
    .wrap_err("mpv not found")?;

  Ok(())
}
