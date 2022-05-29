use nom::{
  combinator::all_consuming,
  multi::{many0, many_m_n},
  number::complete::be_i32,
  IResult,
};
use serde::Serialize;

#[derive(Serialize, Clone)]
pub struct Gps5 {
  pub latitude: f64,
  pub longitude: f64,
  pub altitude: f64,
  pub speed_2d: f64,
  pub speed_3d: f64,
}

impl Gps5 {
  pub fn parse_one(input: &[u8]) -> IResult<&[u8], Self> {
    let (input, parsed) = many_m_n(5, 5, be_i32)(input)?;

    let gps5 = match parsed.as_slice() {
      [latitude, longitude, altitude, speed_2d, speed_3d] => Gps5 {
        latitude: f64::from(*latitude),
        longitude: f64::from(*longitude),
        altitude: f64::from(*altitude),
        speed_2d: f64::from(*speed_2d),
        speed_3d: f64::from(*speed_3d),
      },

      _ => unreachable!(),
    };

    Ok((input, gps5))
  }

  pub fn parse_many(input: &[u8]) -> IResult<&[u8], Vec<Self>> {
    all_consuming(many0(Self::parse_one))(input)
  }
}
