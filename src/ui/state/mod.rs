pub mod focus;

use focus::Focus;

pub struct State {
  pub src_dir: Option<String>,
  pub dst_dir: Option<String>,

  pub focus: Focus,
}

impl Default for State {
  fn default() -> Self {
    Self {
      src_dir: None,
      dst_dir: None,

      focus: Focus::default(),
    }
  }
}

impl State {
  pub fn toggle_focus(&mut self) {
    self.focus = match self.focus {
      Focus::Files => Focus::Sessions,
      Focus::Sessions => Focus::Files,
    }
  }
}
