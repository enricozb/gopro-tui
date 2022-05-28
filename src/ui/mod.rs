mod events;
mod render;
mod state;

use std::io::{self, Stdout};

use state::State;
use tui::{backend::CrosstermBackend, Terminal};

use crate::error::Result;

pub struct Ui {
  state: State,
  terminal: Terminal<CrosstermBackend<Stdout>>,
}

impl Ui {
  pub fn new() -> Result<Self> {
    Ok(Self {
      state: State::default(),
      terminal: Terminal::new(CrosstermBackend::new(io::stdout()))?,
    })
  }

  pub fn run(&self) -> Result<()> {
    Ok(())
  }
}
