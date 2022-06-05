use std::io::Stdout;

use tui::{
  backend::CrosstermBackend,
  layout::{Constraint, Direction, Layout, Rect},
  style::{Color, Modifier, Style},
  text::{Span, Spans},
  widgets::{Block, Borders, Clear, Paragraph, Table, TableState, Wrap},
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
  destinations: Rect,

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
      .constraints([Constraint::Percentage(40), Constraint::Percentage(60)].as_ref())
      .split(layout[0]);

    Self {
      sessions: left[0],
      files: left[1],
      destinations: layout[1],

      input: Sections::input_rect(frame),
      popup: Sections::popup_rect(frame),

      state,
    }
  }

  pub fn render(&self, frame: &mut Frame<CrosstermBackend<Stdout>>) {
    frame.render_widget(Clear, frame.size());

    frame.render_stateful_widget(self.sessions(), self.sessions, &mut self.sessions_state());
    frame.render_stateful_widget(self.files(), self.files, &mut self.files_state());
    frame.render_widget(self.destinations(), self.destinations);

    if let Some(error) = &self.state.error {
      frame.render_widget(Clear, self.popup);
      frame.render_widget(self.popup(error.clone()), self.popup);
    }

    if let Some(input) = &self.state.input {
      frame.render_widget(Clear, self.input);
      frame.render_widget(self.input(input.clone()), self.input);

      frame.set_cursor(self.input.x + input.chars().count() as u16 + 1, self.input.y + 1);
    }
  }

  fn sessions(&self) -> Table {
    let (title, border_style) = border_style(&[Some("Sessions")], self.state.focus == Focus::Sessions);

    Table::new(rows::sessions(self.state))
      .block(Block::default().title(title).borders(Borders::ALL).border_style(border_style))
      .widths(&[
        Constraint::Length(11),
        Constraint::Length(6),
        Constraint::Length(9),
        Constraint::Length(20),
      ])
  }

  fn files(&self) -> Table {
    let (title, border_style) = border_style(&[Some("Files")], self.state.focus == Focus::Files);

    Table::new(rows::files(self.state))
      .block(Block::default().title(title).borders(Borders::ALL).border_style(border_style))
      .widths(&[
        Constraint::Length(2),
        Constraint::Length(12),
        Constraint::Length(9),
        Constraint::Length(5),
        Constraint::Length(64),
      ])
  }

  fn destinations(&self) -> Table {
    let title = self.state.mode.output_dir().unwrap_or_default().to_string_lossy().to_string();
    let title = Span::styled(title, Style::default().fg(Color::Blue));

    Table::new(rows::destinations(self.state))
      .block(Block::default().title(title).borders(Borders::ALL))
      .widths(&[Constraint::Percentage(80), Constraint::Percentage(20)])
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
    sessions_state.select(Some(self.state.session_idx));

    sessions_state
  }

  fn files_state(&self) -> TableState {
    let mut files_state = TableState::default();
    files_state.select(Some(self.state.file_idx));

    files_state
  }

  fn input_rect(frame: Rect) -> Rect {
    let (width, height) = (67, 3);

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
