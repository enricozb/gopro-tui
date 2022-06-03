use std::os::unix::fs::MetadataExt;

use tui::{
  style::{Color, Modifier, Style},
  text::{Span, Spans},
  widgets::{Cell, Row},
};

use crate::ui::state::{
  focus::Focus,
  session::{File, Session, Status},
  State,
};

struct Colors {
  date: Color,
  count: Color,
  size: Color,
  duration: Color,
  path: Color,
  status_import: Color,
  status_ignore: Color,
  status_none: Color,
}

impl Colors {
  fn new(highlighted: bool) -> Self {
    let mut colors = [
      Color::Green,  // date
      Color::Gray,   // count
      Color::Yellow, // size
      Color::Green,  // duration
      Color::Gray,   // path
      Color::Green,  // status_import
      Color::Red,    // status_ignore
      Color::Blue,   // status_ignore
    ];

    if highlighted {
      for color in colors.iter_mut() {
        *color = Self::focused_color(*color)
      }
    }

    Self {
      date: colors[0],
      count: colors[1],
      size: colors[2],
      duration: colors[3],
      path: colors[4],
      status_import: colors[5],
      status_ignore: colors[6],
      status_none: colors[7],
    }
  }

  fn focused_color(color: Color) -> Color {
    match color {
      Color::Red => Color::LightRed,
      Color::Green => Color::LightGreen,
      Color::Gray => Color::White,
      Color::Yellow => Color::LightYellow,
      Color::Blue => Color::LightBlue,
      color => color,
    }
  }
}

trait Rowable<'a> {
  fn row(&self, selected: bool, focused: bool) -> Row<'a>;
}

impl<'a> Rowable<'a> for Session {
  fn row(&self, selected: bool, focused: bool) -> Row<'a> {
    let modifier = if selected { Modifier::BOLD } else { Modifier::empty() };

    let colors = Colors::new(selected && focused);

    let (import_size, uncategorized_size) = size_split(self);

    Row::new(vec![
      Cell::from(self.date.clone()).style(Style::default().fg(colors.date).add_modifier(modifier)),
      Cell::from(self.files.len().to_string()).style(Style::default().fg(colors.count).add_modifier(modifier)),
      Cell::from(format!(
        "{:>8}",
        human_readable_size(self.files.values().map(|f| f.metadata.size()).sum::<u64>())
      ))
      .style(Style::default().fg(colors.size).add_modifier(modifier)),
      Cell::from(human_readable_size_split(
        import_size,
        uncategorized_size,
        colors.status_import,
        colors.status_none,
      ))
      .style(Style::default().add_modifier(modifier)),
    ])
  }
}

impl<'a> Rowable<'a> for File {
  fn row(&self, selected: bool, focused: bool) -> Row<'a> {
    let modifier = if selected { Modifier::BOLD } else { Modifier::empty() };

    let colors = Colors::new(selected && focused);

    let (status, status_color) = match self.status {
      None => (" ", Color::White),
      Some(Status::Import) => ("+", colors.status_import),
      Some(Status::Ignore) => ("-", colors.status_ignore),
    };

    Row::new(vec![
      Span::styled(format!(" {}", status), Style::default().fg(status_color).add_modifier(modifier)),
      Span::styled(
        self.path.file_name().unwrap().to_string_lossy().into_owned(),
        Style::default().fg(colors.path).add_modifier(modifier),
      ),
      Span::styled(
        format!("{:>9}", human_readable_size(self.metadata.len())),
        Style::default().fg(colors.size).add_modifier(modifier),
      ),
      Span::styled(
        format!("{:>5}", human_readable_seconds(self.seconds as i64)),
        Style::default().fg(colors.duration).add_modifier(modifier),
      ),
      Span::styled(
        format!(" {}", self.note.as_deref().unwrap_or("")),
        Style::default().fg(colors.path).add_modifier(modifier),
      ),
    ])
  }
}

pub fn sessions(state: &State) -> Vec<Row<'_>> {
  state
    .sessions
    .iter()
    .enumerate()
    .map(|(i, (_, s))| (s.row(i == state.session_idx, state.focus == Focus::Sessions)))
    .collect()
}

pub fn files(state: &State) -> Vec<Row<'_>> {
  match state.session() {
    None => vec![],
    Some(Session { files, .. }) => files
      .values()
      .enumerate()
      .map(|(i, f)| (f.row(i == state.file_idx, state.focus == Focus::Files)))
      .collect(),
  }
}

fn size_split(session: &Session) -> (u64, u64) {
  let imported: u64 = session
    .files
    .values()
    .filter(|f| f.status == Some(Status::Import))
    .map(|f| f.metadata.size())
    .sum();

  let uncategorized: u64 = session.files.values().filter(|f| f.status == None).map(|f| f.metadata.size()).sum();

  (imported, uncategorized)
}

fn human_readable_size_split(imported: u64, uncategorized: u64, imported_color: Color, uncategorized_color: Color) -> Spans<'static> {
  let mut parts = Vec::new();

  if imported > 0 {
    parts.push(Span::styled(
      format!("+{}", human_readable_size(imported)),
      Style::default().fg(imported_color),
    ))
  }

  if uncategorized > 0 {
    if imported > 0 {
      parts.push(Span::raw("/"))
    }

    parts.push(Span::styled(
      format!("~{}", human_readable_size(uncategorized)),
      Style::default().fg(uncategorized_color),
    ))
  }

  Spans::from(parts)
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
