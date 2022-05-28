mod args;
mod channel;
mod error;
mod ui;

use clap::Parser;

use crate::{args::Args, channel::Channel, error::Result, ui::events};

fn main() -> Result<()> {
  let _args = Args::parse();

  stable_eyre::install()?;

  let event_channel = Channel::new();
  let result_channel = Channel::new();

  events::spawn(event_channel.sender(), result_channel.sender());
  ui::spawn(event_channel, result_channel.sender());

  result_channel.poll()??;

  Ok(())
}
