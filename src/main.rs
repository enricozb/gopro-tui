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
  ui::events,
};

fn main() -> Result<()> {
  stable_eyre::install()?;

  let args = Args::parse();

  let (input_dir, mode) = match args.input_dir {
    Some(dir) => (dir, Mode::Importing),
    None => (args.output_dir, Mode::Viewing),
  };

  let cache = SourceCache::from(&input_dir, mode)?;

  let event_channel = EventChannel::new();
  let result_channel = ResultChannel::new();

  events::spawn(&event_channel, &result_channel);

  // TODO(enricozb): don't clone the cache; Arc<Mutex<...>> it.
  reader::spawn(input_dir, &event_channel, &result_channel, cache.clone());

  ui::spawn(event_channel, &result_channel, cache);
  result_channel.poll()??;

  Ok(())
}
