use std::io::Stdout;

use tui::{
  backend::CrosstermBackend,
  layout::{Constraint, Direction, Layout, Rect},
  style::Style,
  text::Span,
  widgets::{Block, Borders, Clear, Gauge, Paragraph, TableState, Wrap},
  Frame,
};

use super::{
  super::state::{focus::Focus, State},
  rows,
  table::Table,
};
use crate::ui::{colors::Colors, state::progress::Bare as BareProgress};

pub fn render(frame: &mut Frame<CrosstermBackend<Stdout>>, state: &State) {
  Sections::new(frame.size(), state).render(frame);
}

struct Sections<'a> {
  sessions: Rect,
  files: Rect,
  destinations: Rect,

  input: Rect,
  search: Rect,
  search_results: Rect,
  popup: Rect,
  progress: Rect,

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

    let (search, search_results) = Sections::search_rects(frame);

    Self {
      sessions: left[0],
      files: left[1],
      destinations: layout[1],

      input: Sections::centered_small(frame),
      search,
      search_results,
      popup: Sections::centered_large(frame),
      progress: Sections::centered_small(frame),

      state,
    }
  }

  pub fn render(&self, frame: &mut Frame<CrosstermBackend<Stdout>>) {
    frame.render_widget(Clear, frame.size());

    self.render_sessions(frame);
    self.render_files(frame);
    self.render_destinations(frame);

    if let Some(error) = &self.state.error {
      frame.render_widget(Clear, self.popup);
      frame.render_widget(self.popup(error.clone()), self.popup);
    } else if let Some(input) = &self.state.input {
      frame.render_widget(Clear, self.input);
      frame.render_widget(self.input(input.clone()), self.input);

      frame.set_cursor(self.input.x + input.chars().count() as u16 + 1, self.input.y + 1);
    } else if let Some(search) = &self.state.search {
      frame.render_widget(Clear, self.search);
      frame.render_widget(self.search(search.clone()), self.search);

      self.render_search_results(search, frame);

      frame.set_cursor(self.search.x + search.chars().count() as u16 + 1, self.search.y + 1);
    } else if let Some(progress) = &self.state.progress {
      if let Ok(progress) = progress.bare() {
        if !progress.done {
          frame.render_widget(Clear, self.progress);
          frame.render_widget(self.progress(&progress), self.progress);
        }
      }
    }
  }

  fn render_sessions(&self, frame: &mut Frame<CrosstermBackend<Stdout>>) {
    use super::table::Alignment::{Left, Right};

    frame.render_stateful_widget(
      Table::new(rows::sessions(self.state))
        .title("Sessions")
        .focused(self.state.focus == Focus::Sessions)
        .alignments([Left, Left, Right, Right, Left]),
      self.sessions,
      &mut self.sessions_state(),
    );
  }

  fn render_files(&self, frame: &mut Frame<CrosstermBackend<Stdout>>) {
    use super::table::Alignment::{Left, Right};

    frame.render_stateful_widget(
      Table::new(rows::files(self.state))
        .title("Files")
        .focused(self.state.focus == Focus::Files)
        .alignments([Left, Left, Right, Right, Left]),
      self.files,
      &mut self.files_state(),
    );
  }

  fn render_destinations(&self, frame: &mut Frame<CrosstermBackend<Stdout>>) {
    let title = self.state.mode.output_dir().unwrap_or_default().to_string_lossy().to_string();

    frame.render_widget(Table::new(rows::destinations(self.state)).title(title), self.destinations);
  }

  fn input(&self, input: String) -> Paragraph {
    Paragraph::new(Span::raw(input))
      .block(
        Block::default()
          .title("Note")
          .borders(Borders::ALL)
          .border_style(Style::default().fg(Colors::normal().input_block)),
      )
      .style(Style::default().fg(Colors::normal().input_text))
  }

  fn search(&self, input: String) -> Paragraph {
    Paragraph::new(Span::raw(input))
      .block(
        Block::default()
          .title("Destination")
          .borders(Borders::ALL)
          .border_style(Style::default().fg(Colors::normal().input_block)),
      )
      .style(Style::default().fg(Colors::normal().input_text))
  }

  fn render_search_results<S: AsRef<str>>(&self, input: S, frame: &mut Frame<CrosstermBackend<Stdout>>) {
    frame.render_widget(Clear, self.search_results);
    frame.render_widget(Table::new(rows::search_matches(self.state, input)), self.search_results);
  }

  fn progress(&self, progress: &BareProgress) -> Gauge {
    Gauge::default()
      .block(
        Block::default()
          .title(format!("Importing File {} of {}", progress.file_idx + 1, progress.file_total))
          .borders(Borders::ALL)
          .border_style(Style::default().fg(Colors::normal().input_block)),
      )
      .gauge_style(Style::default().fg(Colors::normal().progress))
      .percent((progress.file_idx * 100 / progress.file_total) as u16)
  }

  fn popup(&self, error: String) -> Paragraph {
    Paragraph::new(error)
      .block(Block::default().title("Error").borders(Borders::ALL))
      .style(Style::default().fg(Colors::normal().error_block))
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

  fn centered_small(frame: Rect) -> Rect {
    let (width, height) = (67, 3);

    Rect {
      x: frame.width / 2 - width / 2,
      y: frame.height / 2 - height / 2,
      width,
      height,
    }
  }

  fn search_rects(frame: Rect) -> (Rect, Rect) {
    const INPUT_HEIGHT: u16 = 3;

    let search_layout = Layout::default()
      .direction(Direction::Vertical)
      .constraints([Constraint::Percentage(30), Constraint::Percentage(40), Constraint::Percentage(30)].as_ref())
      .split(frame);

    let rect = Layout::default()
      .direction(Direction::Horizontal)
      .constraints([Constraint::Percentage(30), Constraint::Percentage(40), Constraint::Percentage(30)].as_ref())
      .split(search_layout[1])[1];

    let search = Rect {
      height: INPUT_HEIGHT,
      ..rect
    };
    let results = Rect {
      y: rect.y + INPUT_HEIGHT,
      height: rect.height - INPUT_HEIGHT,
      ..rect
    };

    (search, results)
  }

  fn centered_large(frame: Rect) -> Rect {
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
