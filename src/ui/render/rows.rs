use std::os::unix::fs::MetadataExt;

use tui::{
  style::{Color, Modifier, Style},
  text::Span,
  widgets::Row,
};

use super::super::state::{
  focus::Focus,
  session::{File, Session},
  State,
};

trait Rowable<'a> {
  fn row(&self, selected: bool, focused: bool) -> Row<'a>;
}

impl<'a> Rowable<'a> for Session {
  fn row(&self, selected: bool, focused: bool) -> Row<'a> {
    let modifier = if selected { Modifier::BOLD } else { Modifier::empty() };

    let (date_color, count_color, size_color) = if selected && focused {
      (Color::LightGreen, Color::White, Color::LightYellow)
    } else {
      (Color::Green, Color::Gray, Color::Yellow)
    };

    Row::new(vec![
      Span::styled(self.date.clone(), Style::default().fg(date_color).add_modifier(modifier)),
      Span::styled(
        self.files.len().to_string(),
        Style::default().fg(count_color).add_modifier(modifier),
      ),
      Span::styled(
        human_readable_size(self.files.values().map(|f| f.metadata.size()).sum::<u64>()),
        Style::default().fg(size_color).add_modifier(modifier),
      ),
    ])
  }
}

impl<'a> Rowable<'a> for File {
  fn row(&self, selected: bool, focused: bool) -> Row<'a> {
    let modifier = if selected { Modifier::BOLD } else { Modifier::empty() };

    let (path_color, size_color, date_color) = if selected && focused {
      (Color::White, Color::LightYellow, Color::LightGreen)
    } else {
      (Color::Gray, Color::Yellow, Color::Green)
    };

    Row::new(vec![
      Span::styled(
        self.path.file_name().unwrap().to_string_lossy().into_owned(),
        Style::default().fg(path_color).add_modifier(modifier),
      ),
      Span::styled(
        format!("{:>9}", human_readable_size(self.metadata.len())),
        Style::default().fg(size_color).add_modifier(modifier),
      ),
      Span::styled(
        format!("{:>5}", human_readable_seconds(self.seconds as i64)),
        Style::default().fg(date_color).add_modifier(modifier),
      ),
      Span::styled(
        format!(" {}", self.note.as_deref().unwrap_or("")),
        Style::default().fg(path_color).add_modifier(modifier),
      ),
    ])
  }
}

pub fn sessions(state: &State) -> Vec<Row<'_>> {
  state
    .sessions
    .iter()
    .enumerate()
    .map(|(i, (_, s))| (s.row(i == state.sessions_idx, state.focus == Focus::Sessions)))
    .collect()
}

pub fn files(state: &State) -> Vec<Row<'_>> {
  match state.session() {
    None => vec![],
    Some(Session { files, .. }) => files
      .values()
      .enumerate()
      .map(|(i, f)| (f.row(i == state.files_idx, state.focus == Focus::Files)))
      .collect(),
  }
}

fn human_readable_size(size: u64) -> String {
  if size < 1_000 {
    format!("{:.1} B", size)
  } else if size < 1_000_000 {
    format!("{:.1} kB", size as f64 / 1_000.0)
  } else if size < 1_000_000_000 {
    format!("{:.1} MB", size as f64 / 1_000_000.0)
  } else if size < 1_000_000_000_000 {
    format!("{:.1} GB", size as f64 / 1_000_000_000.0)
  } else {
    format!("{:.1} TB", size as f64 / 1_000_000_000_000.0)
  }
}

fn human_readable_seconds(seconds: i64) -> String {
  let secs = seconds % 60;
  let mins = (seconds / 60) % 60;

  match (mins, secs) {
    (0, secs) => format!("{}", secs),
    (mins, secs) => format!("{}:{:02}", mins, secs),
  }
}
