use std::{collections::BTreeSet, os::unix::fs::MetadataExt, path::PathBuf};

use tui::{
  style::{Color, Modifier, Style},
  text::{Span, Spans},
};

use super::search::{self, Match};
use crate::{
  mode::Mode,
  ui::{
    colors::Colors,
    state::{
      destination::Destination,
      focus::Focus,
      session::{File, Session, Status},
      State,
    },
  },
};

trait Rowable<'a> {
  fn row(&self, selected: bool, focused: bool) -> Vec<Spans<'a>>;
}

impl<'a> Rowable<'a> for Session {
  fn row(&self, selected: bool, focused: bool) -> Vec<Spans<'a>> {
    let modifier = if selected { Modifier::BOLD } else { Modifier::empty() };

    let colors = Colors::focused(selected && focused);

    let (import_size, uncategorized_size) = size_split(self);

    vec![
      Spans::from(Span::styled(
        self.date.clone(),
        Style::default().fg(colors.date).add_modifier(modifier),
      )),
      human_readable_file_counts(self.files.values(), colors.count, colors.status_import),
      Spans::from(Span::styled(
        human_readable_size(self.files.values().map(|f| f.metadata.size()).sum::<u64>()),
        Style::default().fg(colors.size).add_modifier(modifier),
      )),
      human_readable_size_split(import_size, uncategorized_size, colors.status_import, colors.status_none),
      Spans::from(Span::styled(
        self.destination.as_ref().map_or("".to_string(), |d| d.rel.clone()),
        Style::default().fg(colors.destination).add_modifier(modifier),
      )),
    ]
  }
}

impl<'a> Rowable<'a> for File {
  fn row(&self, selected: bool, focused: bool) -> Vec<Spans<'a>> {
    let modifier = if selected { Modifier::BOLD } else { Modifier::empty() };

    let colors = Colors::focused(selected && focused);

    let (status, status_color) = match self.status {
      None => (" ", colors.status_none),
      Some(Status::Import) => ("+", colors.status_import),
      Some(Status::Ignore) => ("-", colors.status_ignore),
    };

    vec![
      Spans::from(Span::styled(
        format!(" {}", status),
        Style::default().fg(status_color).add_modifier(modifier),
      )),
      Spans::from(Span::styled(
        self.path.file_name().unwrap().to_string_lossy().into_owned(),
        Style::default().fg(colors.filename).add_modifier(modifier),
      )),
      Spans::from(Span::styled(
        human_readable_size(self.metadata.len()),
        Style::default().fg(colors.size).add_modifier(modifier),
      )),
      Spans::from(Span::styled(
        human_readable_seconds(self.seconds as i64),
        Style::default().fg(colors.duration).add_modifier(modifier),
      )),
      Spans::from(Span::styled(
        self.note.clone().unwrap_or_else(|| "".to_string()),
        Style::default().fg(colors.filename).add_modifier(modifier),
      )),
    ]
  }
}

impl<'a> Rowable<'a> for Match<'a> {
  fn row(&self, selected: bool, focused: bool) -> Vec<Spans<'a>> {
    let colors = Colors::focused(selected && focused);

    let mut spans: Vec<_> = self
      .destination
      .rel
      .chars()
      .map(|c| Span::styled(c.to_string(), Style::default().fg(colors.destination)))
      .collect();

    for position in &self.positions {
      spans[*position].style = spans[*position].style.add_modifier(Modifier::UNDERLINED);
    }

    vec![Spans::from(spans)]
  }
}

pub fn sessions(state: &State) -> Vec<Vec<Spans<'_>>> {
  state
    .sessions
    .iter()
    .enumerate()
    .map(|(i, (_, s))| (s.row(i == state.session_idx, state.focus == Focus::Sessions)))
    .collect()
}

pub fn files(state: &State) -> Vec<Vec<Spans<'_>>> {
  match state.session() {
    None => vec![],
    Some(Session { files, .. }) => files
      .values()
      .enumerate()
      .map(|(i, f)| (f.row(i == state.file_idx, state.focus == Focus::Files)))
      .collect(),
  }
}

// destinations computes a tree-like view of rows showing the destination directories.
// this can't be in an implementation of rowable because state must be tracked while
// iterating directories to build the tree-like view.
pub fn destinations(state: &State) -> Vec<Vec<Spans<'_>>> {
  struct DestRow<'a> {
    path: &'a PathBuf,
    file_name: String,
    depth: usize,
    is_last: bool,
  }

  fn extend_stack<'a>(stack: &mut Vec<DestRow<'a>>, destinations: &'a BTreeSet<Destination>, depth: usize) {
    let len = destinations.len();
    stack.extend(
      destinations
        .iter()
        .enumerate()
        .filter_map(|(i, d)| {
          d.abs.file_name().map(|f| f.to_string_lossy().to_string()).map(|file_name| DestRow {
            path: &d.abs,
            file_name,
            depth,
            is_last: (i + 1) == len,
          })
        })
        .rev(),
    );
  }

  let output_dir = if let Mode::Importing { output_dir, .. } = &state.mode {
    output_dir
  } else {
    return Vec::new();
  };

  let colors = Colors::normal();

  let mut prefix = Vec::new();
  let mut rows = Vec::new();
  let mut stack = Vec::new();

  if let Some(destinations) = state.destinations.get(output_dir) {
    extend_stack(&mut stack, destinations, 0);
  };

  while !stack.is_empty() {
    if let Some(dest) = stack.pop() {
      prefix.truncate(dest.depth);

      let (section_sep, prefix_tail) = if dest.is_last { ("   ", " └─") } else { (" │ ", " ├─") };

      rows.push(vec![Spans::from(vec![
        Span::raw(format!("{}{} ", prefix.join(""), prefix_tail)),
        Span::styled(dest.file_name, Style::default().fg(colors.destination)),
      ])]);

      if let Some(destinations) = state.destinations.get(dest.path) {
        extend_stack(&mut stack, destinations, dest.depth + 1);
        prefix.push(section_sep);
      }
    }
  }

  rows
}

pub fn search_matches<S: AsRef<str>>(state: &State, search: S) -> Vec<Vec<Spans<'_>>> {
  search::sorted(search, state.destinations())
    .into_iter()
    .map(|search_match| search_match.row(false, false))
    .collect()
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

fn human_readable_file_counts<'a, I>(files: I, count_color: Color, status_import: Color) -> Spans<'static>
where
  I: Iterator<Item = &'a File>,
{
  let mut count = 0;
  let mut imported = 0;

  for f in files {
    count += 1;
    if f.status == Some(Status::Import) {
      imported += 1;
    }
  }

  let mut parts = vec![Span::styled(count.to_string(), Style::default().fg(count_color))];

  if imported > 0 {
    parts.push(Span::raw(" "));

    parts.push(Span::styled(format!("+{}", imported), Style::default().fg(status_import)));
  };

  Spans::from(parts)
}

fn human_readable_size_split(imported: u64, uncategorized: u64, imported_color: Color, uncategorized_color: Color) -> Spans<'static> {
  let mut parts = Vec::new();

  if imported > 0 {
    parts.push(Span::styled(
      format!("+{}", human_readable_size(imported)),
      Style::default().fg(imported_color),
    ));
  }

  if uncategorized > 0 {
    if imported > 0 {
      parts.push(Span::raw("/"));
    }

    parts.push(Span::styled(
      format!("~{}", human_readable_size(uncategorized)),
      Style::default().fg(uncategorized_color),
    ));
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
