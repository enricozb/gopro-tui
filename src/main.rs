mod args;
mod cache;
mod channel;
mod dirs;
mod error;
mod importer;
mod mpv;
mod ui;
mod utils;

use clap::Parser;

use crate::{
  args::Args,
  cache::CacheEntry,
  channel::{EventChannel, ResultChannel},
  error::Result,
  ui::events,
};

fn main() -> Result<()> {
  stable_eyre::install()?;

  let args = Args::parse();
  let cache = CacheEntry::from(&args.src_dir)?;

  let event_channel = EventChannel::new();
  let result_channel = ResultChannel::new();

  events::spawn(&event_channel, &result_channel);
  importer::spawn(args.src_dir, &event_channel, &result_channel, cache.clone());

  ui::spawn(event_channel, &result_channel, cache);
  result_channel.poll()??;

  Ok(())
}
