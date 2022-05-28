pub struct State {
  pub src_dir: Option<String>,
  pub dst_dir: Option<String>,
}

impl Default for State {
  fn default() -> Self {
    Self {
      src_dir: None,
      dst_dir: None,
    }
  }
}
