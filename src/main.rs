mod args;
mod error;
mod ui;

use clap::Parser;

use crate::{args::Args, error::Result, ui::Ui};

fn main() -> Result<()> {
  stable_eyre::install()?;

  let _args = Args::parse();

  Ui::new()?.run()?;

  Ok(())
}
