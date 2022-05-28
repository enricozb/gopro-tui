#[derive(PartialEq, Eq)]
pub enum Focus {
  Sessions,
  Files,
}

impl Default for Focus {
  fn default() -> Self {
    Self::Sessions
  }
}
