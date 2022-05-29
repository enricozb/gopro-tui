pub mod types;
mod utils;

use nom::{
  bytes::complete::take,
  combinator::all_consuming,
  multi::many0,
  number::complete::{be_i16, be_i32, be_u16, be_u32, be_u8},
  IResult,
};
use serde::Serialize;
use stable_eyre::eyre::eyre;
use types::Gps5;

use crate::error::Result;

#[derive(Serialize)]
pub enum Gpmf {
  Klv {
    key: String,
    entries: Vec<Self>,
  },

  Gps5 {
    entries: Vec<Gps5>,
  },

  Scale {
    divisors: Vec<i64>,
  },

  Unknown {
    key: String,
    kind: char,
    size: u8,
    count: u16,

    #[serde(skip_serializing)]
    data: Vec<u8>,
  },
}

impl Gpmf {
  fn parse_one(input: &[u8]) -> IResult<&[u8], Self> {
    let (input, key) = be_u32(input)?;
    let key = Self::key_string(key);

    let (input, kind) = be_u8(input)?;
    let (input, size) = be_u8(input)?;
    let (input, count) = be_u16(input)?;

    let data_size = utils::align((size as usize) * (count as usize));

    let (input, data) = take(data_size)(input)?;

    let gpmf = match (key.as_str(), kind) {
      ("DEVC", kind::NULL) => Self::Klv {
        key,
        entries: Self::parse_many(data)?.1,
      },

      ("STRM", kind::NULL) => Self::Klv {
        key,
        entries: Self::parse_strm(data)?.1,
      },

      ("SCAL", kind::I32) => Self::Scale {
        divisors: all_consuming(many0(be_i32))(data)?
          .1
          .into_iter()
          .map(i64::from)
          .collect(),
      },

      ("SCAL", kind::I16) => Self::Scale {
        divisors: all_consuming(many0(be_i16))(data)?
          .1
          .into_iter()
          .map(i64::from)
          .collect(),
      },

      ("GPS5", kind::I32) => Self::Gps5 {
        entries: Gps5::parse_many(data)?.1,
      },

      _ => Self::Unknown {
        key,
        kind: kind as char,
        size,
        count,
        data: data.to_vec(),
      },
    };

    Ok((input, gpmf))
  }

  fn parse_many(input: &[u8]) -> IResult<&[u8], Vec<Self>> {
    all_consuming(many0(Self::parse_one))(input)
  }

  fn parse_strm(mut input: &[u8]) -> IResult<&[u8], Vec<Self>> {
    let mut divisors = Vec::new();
    let mut klvs = Vec::new();

    while !input.is_empty() {
      let (rest, gpmf) = Self::parse_one(input)?;

      let gpmf = match &gpmf {
        Self::Scale { divisors: divs } => {
          divisors = divs.clone();
          gpmf
        }

        Self::Gps5 { entries } => Self::Gps5 {
          entries: entries
            .iter()
            .map(|g| Gps5 {
              latitude: g.latitude / *divisors.get(0).unwrap_or(&1) as f64,
              longitude: g.longitude / *divisors.get(1).unwrap_or(&1) as f64,
              altitude: g.altitude / *divisors.get(2).unwrap_or(&1) as f64,
              speed_2d: g.speed_2d / *divisors.get(3).unwrap_or(&1) as f64,
              speed_3d: g.speed_3d / *divisors.get(4).unwrap_or(&1) as f64,
            })
            .collect(),
        },

        _ => gpmf,
      };

      klvs.push(gpmf);

      input = rest;
    }

    Ok((input, klvs))
  }

  fn key_string(key: u32) -> String {
    key.to_be_bytes().into_iter().map(|b| b as char).collect()
  }

  pub fn parse(input: &[u8]) -> Result<Vec<Self>> {
    match Self::parse_many(input) {
      Ok(([], entries)) => Ok(entries),
      Ok(_) => unreachable!(),
      Err(error) => Err(eyre!("Failed to parse gpmf data: {}", error)),
    }
  }
}

mod kind {
  pub const NULL: u8 = b'\x00';
  pub const I16: u8 = b's';
  pub const I32: u8 = b'l';
}
