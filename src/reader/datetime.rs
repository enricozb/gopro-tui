use std::path::Path;

use chrono::{
  offset::{FixedOffset, Utc},
  DateTime, TimeZone,
};

use super::{
  ffmpeg::{self, FFProbeInfo},
  gpmf::{types::Gps5, Gpmf},
};
use crate::error::{err, Result};

pub fn approximate(path: &Path, ffprobe_info: &FFProbeInfo) -> Result<DateTime<FixedOffset>> {
  offset(path, ffprobe_info)?
    .from_local_datetime(&DateTime::<Utc>::from(path.metadata()?.modified()?).naive_utc())
    .earliest()
    .ok_or(err!("Couldn't infer approximate datetime"))
}

fn offset(path: &Path, ffprobe_info: &FFProbeInfo) -> Result<FixedOffset> {
  let data = ffmpeg::gpmf_data(path, ffprobe_info)?;
  let Gps5 { longitude, .. } = first_gps_data(&Gpmf::parse(&data)?).ok_or(err!("No GPS data"))?;

  Ok(FixedOffset::east(3600 * (longitude * 12.0 / 180.0) as i32))
}

fn first_gps_data(gpmf: &[Gpmf]) -> Option<Gps5> {
  gpmf.iter().find_map(|g| match g {
    Gpmf::Klv { entries, .. } => first_gps_data(entries),
    Gpmf::Gps5 { entries } => entries.get(0).cloned(),
    _ => None,
  })
}
