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

pub fn sorted<'a, I, S: AsRef<str>>(search: S, destinations: I) -> Vec<Match<'a>>
where
  I: Iterator<Item = &'a Destination>,
{
  let mut matches: Vec<_> = destinations.filter_map(|d| score(&search, d)).collect();
  matches.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());

  matches
}
