use std::io::Stdout;

use tui::{
  backend::CrosstermBackend,
  layout::{Constraint, Direction, Layout, Rect},
  style::{Color, Modifier, Style},
  text::{Span, Spans},
  widgets::{Block, Borders, Clear, Paragraph, Row, Table, TableState, Wrap},
  Frame,
};

use super::{
  super::state::{focus::Focus, State},
  rows,
};

pub fn render(frame: &mut Frame<CrosstermBackend<Stdout>>, state: &State) {
  Sections::new(frame.size(), state).render(frame);
}

struct Sections<'a> {
  sessions: Rect,
  files: Rect,
  outputs: Rect,

  input: Rect,
  popup: Rect,

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

      input: Sections::input_rect(frame),
      popup: Sections::popup_rect(frame),

      state,
    }
  }

  pub fn render(&self, frame: &mut Frame<CrosstermBackend<Stdout>>) {
    frame.render_stateful_widget(self.sessions(), self.sessions, &mut self.sessions_state());
    frame.render_stateful_widget(self.files(), self.files, &mut self.files_state());
    frame.render_widget(self.outputs(), self.outputs);

    if let Some(error) = &self.state.error {
      frame.render_widget(Clear, self.popup);
      frame.render_widget(self.popup(error.clone()), self.popup);
    }

    if let Some(input) = &self.state.input {
      frame.render_widget(Clear, self.input);
      frame.render_widget(self.input(input.clone()), self.input);
    }
  }

  fn sessions(&self) -> Table {
    let (title, border_style) = border_style(
      &[Some("Sessions"), self.state.src_dir.as_deref()],
      self.state.focus == Focus::Sessions,
    );

    Table::new(rows::sessions(self.state))
      .header(Row::new(vec!["date", "files", "size", "output"]).style(Style::default().add_modifier(Modifier::UNDERLINED)))
      .block(Block::default().title(title).borders(Borders::ALL).border_style(border_style))
      .widths(&[
        Constraint::Length(11),
        Constraint::Length(6),
        Constraint::Length(9),
        Constraint::Length(10),
      ])
  }

  fn files(&self) -> Table {
    let (title, border_style) = border_style(&[Some("Files")], self.state.focus == Focus::Files);

    Table::new(rows::files(self.state))
      .block(Block::default().title(title).borders(Borders::ALL).border_style(border_style))
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

  fn input(&self, input: String) -> Paragraph {
    Paragraph::new(Span::raw(input))
      .block(
        Block::default()
          .title("Note")
          .borders(Borders::ALL)
          .border_style(Style::default().fg(Color::Green)),
      )
      .style(Style::default().fg(Color::White))
  }

  fn popup(&self, error: String) -> Paragraph {
    Paragraph::new(error)
      .block(Block::default().title("Error").borders(Borders::ALL))
      .style(Style::default().fg(Color::Red))
      .wrap(Wrap { trim: true })
  }

  fn sessions_state(&self) -> TableState {
    let mut sessions_state = TableState::default();
    sessions_state.select(Some(self.state.sessions_idx));

    sessions_state
  }

  fn files_state(&self) -> TableState {
    let mut files_state = TableState::default();
    files_state.select(Some(self.state.files_idx));

    files_state
  }

  fn input_rect(frame: Rect) -> Rect {
    let (width, height) = (50, 3);

    Rect {
      x: frame.width / 2 - width / 2,
      y: frame.height / 2 - height / 2,
      width,
      height,
    }
  }

  fn popup_rect(frame: Rect) -> Rect {
    let popup_layout = Layout::default()
      .direction(Direction::Vertical)
      .constraints([Constraint::Percentage(30), Constraint::Percentage(40), Constraint::Percentage(30)].as_ref())
      .split(frame);

    Layout::default()
      .direction(Direction::Horizontal)
      .constraints([Constraint::Percentage(20), Constraint::Percentage(60), Constraint::Percentage(20)].as_ref())
      .split(popup_layout[1])[1]
  }
}

fn border_style<'a>(title: &[Option<&'a str>], focused: bool) -> (Spans<'a>, Style) {
  let style = Style::default().fg(Color::Blue);
  let spans = Spans::from(
    title
      .iter()
      .filter_map(|s| s.map(|s| Span::styled(s, style)))
      .collect::<Vec<Span<'_>>>(),
  );

  if focused {
    (spans, Style::default().fg(Color::Blue).add_modifier(Modifier::BOLD))
  } else {
    (spans, Style::default())
  }
}
