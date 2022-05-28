use std::io::{self, Stdout};

use tui::{backend::CrosstermBackend, Terminal};

use crate::error::Result;

pub struct Ui {
  terminal: Terminal<CrosstermBackend<Stdout>>,
}

impl Ui {
  pub fn new() -> Result<Self> {
    Ok(Self {
      terminal: Terminal::new(CrosstermBackend::new(io::stdout()))?,
    })
  }

  pub fn run(&self) -> Result<()> {
    Ok(())
  }
}
