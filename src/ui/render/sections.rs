use std::io::Stdout;

use tui::{
  backend::CrosstermBackend,
  layout::{Constraint, Direction, Layout, Rect},
  style::{Color, Modifier, Style},
  text::{Span, Spans},
  widgets::{Block, Borders, Row, Table},
  Frame,
};

use super::super::state::State;

pub fn render(frame: &mut Frame<CrosstermBackend<Stdout>>, state: &State) {
  Sections::new(frame.size(), state).render(frame);
}

struct Sections<'a> {
  sessions: Rect,
  files: Rect,
  outputs: Rect,

  state: &'a State,
}

impl<'a> Sections<'a> {
  pub fn new(frame: Rect, state: &'a State) -> Self {
    let layout = Layout::default()
      .direction(Direction::Horizontal)
      .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
      .split(frame);

    let left = Layout::default()
      .direction(Direction::Vertical)
      .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
      .split(layout[0]);

    Self {
      sessions: left[0],
      files: left[1],
      outputs: layout[1],

      state,
    }
  }

  pub fn render(&self, frame: &mut Frame<CrosstermBackend<Stdout>>) {
    frame.render_widget(self.sessions(), self.sessions);
    frame.render_widget(self.files(), self.files);
    frame.render_widget(self.outputs(), self.outputs);
  }

  fn sessions(&self) -> Table {
    let title = match &self.state.src_dir {
      Some(dir) => format!("Sessions - {}", dir),
      None => "Sessions".to_string(),
    };

    let title = Spans::from(vec![Span::styled(title, Style::default().fg(Color::Blue))]);

    Table::new([])
      .header(
        Row::new(vec!["date", "files", "size", "output"]).style(Style::default().add_modifier(Modifier::UNDERLINED)),
      )
      .block(Block::default().title(title).borders(Borders::ALL))
      .widths(&[
        Constraint::Length(11),
        Constraint::Length(6),
        Constraint::Length(9),
        Constraint::Length(10),
      ])
  }

  fn files(&self) -> Table {
    let title = Spans::from(vec![Span::styled("Files", Style::default().fg(Color::Blue))]);

    Table::new([])
      .block(Block::default().title(title).borders(Borders::ALL))
      .widths(&[
        Constraint::Length(12),
        Constraint::Length(10),
        Constraint::Length(5),
        Constraint::Length(31),
      ])
  }

  fn outputs(&self) -> Block {
    let title = match &self.state.dst_dir {
      Some(dir) => format!("Outputs - {}", dir),
      None => "Outputs".to_string(),
    };

    Block::default().title(title).borders(Borders::ALL)
  }
}
