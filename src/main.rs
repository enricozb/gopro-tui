mod args;
mod cache;
mod channel;
mod dirs;
mod error;
mod mode;
mod mpv;
mod reader;
mod ui;
mod utils;

use clap::Parser;

use crate::{
  args::Args,
  cache::Source as SourceCache,
  channel::{EventChannel, ResultChannel},
  error::Result,
  mode::Mode,
  reader::destinations,
  ui::events,
};

fn main() -> Result<()> {
  stable_eyre::install()?;

  let args = Args::parse();

  let mode = Mode::from(args);

  // TODO(enricozb): don't clone the cache; Arc<Mutex<...>> it.
  let cache = SourceCache::from(&mode)?;

  let event_channel = EventChannel::new();
  let result_channel = ResultChannel::new();

  events::spawn(&event_channel, &result_channel);

  reader::spawn(&mode, &event_channel, &result_channel, cache.clone());
  destinations::spawn(&mode, &event_channel, &result_channel);

  ui::spawn(mode, event_channel, &result_channel, cache);
  result_channel.poll()??;

  Ok(())
}
