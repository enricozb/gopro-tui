mod args;
mod channel;
mod error;
mod importer;
mod mpv;
mod ui;

use clap::Parser;

use crate::{
  args::Args,
  channel::{EventChannel, ResultChannel},
  error::Result,
  ui::events,
};

fn main() -> Result<()> {
  let args = Args::parse();

  stable_eyre::install()?;

  let event_channel = EventChannel::new();
  let result_channel = ResultChannel::new();

  events::spawn(&event_channel, &result_channel);
  importer::spawn(args.src_dir, &event_channel, &result_channel);

  ui::spawn(event_channel, result_channel.sender());

  result_channel.poll()??;

  Ok(())
}
