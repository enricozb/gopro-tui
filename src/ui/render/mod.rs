use std::io::Stdout;

use tui::{backend::CrosstermBackend, Frame};

use super::state::State;

pub fn render(f: &mut Frame<CrosstermBackend<Stdout>>, state: &State) {}
