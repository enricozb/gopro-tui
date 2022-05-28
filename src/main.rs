mod args;
mod error;
mod ui;

use clap::Parser;

use crate::{
  args::Args,
  error::Result,
  ui::events::{self, channel::Channel},
};

fn main() -> Result<()> {
  let _args = Args::parse();

  stable_eyre::install()?;

  let event_channel = Channel::new();
  let result_channel = Channel::new();

  events::spawn(event_channel.sender.clone(), result_channel.sender.clone());
  ui::spawn(event_channel, result_channel.sender.clone());

  result_channel.poll()??;

  Ok(())
}
