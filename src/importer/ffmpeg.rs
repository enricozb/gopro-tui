use std::{fmt::Display, path::Path, process::Command, result, str::FromStr};

use serde::{de, Deserialize, Deserializer};
use stable_eyre::eyre::eyre;

use crate::error::Result;

pub struct FFProbeInfo {
  pub gpmd_index: u64,
  pub seconds: f64,
}

#[derive(Deserialize)]
struct StreamInfos {
  format: StreamFormat,
  streams: Vec<StreamInfo>,
}

#[derive(Deserialize)]
struct StreamFormat {
  #[serde(deserialize_with = "deserialize_num_from_str")]
  duration: f64,
}

#[derive(Deserialize)]
struct StreamInfo {
  index: u64,
  codec_tag_string: String,
}

fn deserialize_num_from_str<'de, T, D>(deserializer: D) -> result::Result<T, D::Error>
where
  D: Deserializer<'de>,
  T: FromStr + Deserialize<'de>,
  <T as FromStr>::Err: Display,
{
  #[derive(Deserialize)]
  #[serde(untagged)]
  enum StringOrNum<T> {
    String(String),
    Number(T),
  }

  match StringOrNum::<T>::deserialize(deserializer)? {
    StringOrNum::String(s) => s.parse::<T>().map_err(de::Error::custom),
    StringOrNum::Number(x) => Ok(x),
  }
}

pub fn ffprobe(file: &Path) -> Result<FFProbeInfo> {
  let output = Command::new("ffprobe")
    .args([
      "-loglevel",
      "error",
      "-select_streams",
      "d",
      "-show_streams",
      "-show_entries",
      "format=duration,stream=index,codec_tag_string:tags=:disposition=",
      "-of",
      "json",
    ])
    .arg(file)
    .output()?;

  let StreamInfos { streams, format } = serde_json::from_slice::<StreamInfos>(&output.stdout)?;
  let gpmd_index = streams
    .iter()
    .find(|s| s.codec_tag_string == "gpmd")
    .ok_or(eyre!("no gpmf data found"))?
    .index;

  Ok(FFProbeInfo {
    gpmd_index,
    seconds: format.duration,
  })
}

pub fn gpmf_data(file: &Path, ffprobe_info: &FFProbeInfo) -> Result<Vec<u8>> {
  let output = Command::new("ffmpeg")
    .arg("-i")
    .arg(file)
    .args([
      "-y",
      "-codec",
      "copy",
      "-map",
      format!("0:{}", ffprobe_info.gpmd_index).as_str(),
      "-f",
      "rawvideo",
      "-",
    ])
    .output()?;

  Ok(output.stdout)
}
