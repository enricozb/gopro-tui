use crate::ui::state::destination::Destination;

pub struct Match<'a> {
  pub destination: &'a Destination,
  pub positions: Vec<usize>,
  pub score: f64,
}

pub fn score<S: AsRef<str>>(search: S, destination: &Destination) -> Option<Match> {
  rff::match_and_score_with_positions(search.as_ref(), destination.rel.as_ref()).map(|(_, score, positions)| Match {
    destination,
    positions,
    score,
  })
}
