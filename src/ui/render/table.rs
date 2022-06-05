use std::cmp;

use tui::{
  layout::Constraint,
  text::{Span, Spans},
  widgets::{Block, Borders, Cell as TuiCell, Row, Table as TuiTable},
};

#[derive(Default)]
pub struct Table<'a> {
  pub title: Option<String>,
  pub rows: Vec<Vec<Spans<'a>>>,

  pub focused: bool,
}

impl<'a> Table<'a> {
  pub fn new(rows: Vec<Vec<Spans<'a>>>) -> Self {
    Self { rows, ..Self::default() }
  }

  pub fn title<S: Into<String>>(self, title: S) -> Self {
    Self {
      title: Some(title.into()),
      ..self
    }
  }

  // constraints computes the table layout based on the maximum width of each column
  pub fn constraints(&self) -> Vec<Constraint> {
    self
      .rows
      .iter()
      .map(|row| row.iter().map(Spans::width).collect::<Vec<usize>>())
      .reduce(|row1, row2| row1.into_iter().zip(row2.into_iter()).map(|(a, b)| cmp::max(a, b)).collect())
      .unwrap_or_default()
      .into_iter()
      .map(|width| Constraint::Length(width as u16))
      .collect()
  }

  pub fn widget(self, constraints: &'a [Constraint]) -> TuiTable<'a> {
    TuiTable::new(
      self
        .rows
        .into_iter()
        .map(|row| Row::new(row.into_iter().map(TuiCell::from).collect::<Vec<_>>())),
    )
    .block(
      Block::default()
        .title(Span::raw(self.title.unwrap_or_default()))
        .borders(Borders::ALL),
    )
    .widths(constraints)
  }
}
