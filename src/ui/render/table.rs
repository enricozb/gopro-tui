use std::cmp;

use tui::{
  layout::Constraint,
  style::{Color, Modifier, Style},
  text::{Span, Spans},
  widgets::{Block, Borders, Cell as TuiCell, Row, Table as TuiTable},
};

#[derive(PartialEq, Eq)]
pub enum Alignment {
  Left,
  Right,
}

#[derive(Default)]
pub struct Table<'a> {
  pub rows: Vec<Vec<Spans<'a>>>,

  pub title: Option<String>,
  pub alignments: Option<Vec<Alignment>>,

  pub focused: bool,
}

impl<'a> Table<'a> {
  pub fn new(rows: Vec<Vec<Spans<'a>>>) -> Self {
    Self { rows, ..Self::default() }
  }

  pub fn alignments<Alignments: Into<Vec<Alignment>>>(self, alignments: Alignments) -> Self {
    Self {
      alignments: Some(alignments.into()),
      ..self
    }
  }

  pub fn focused(self, focused: bool) -> Self {
    Self { focused, ..self }
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
    let border_style = self.border_style();

    TuiTable::new(Self::align_rows(self.rows, self.alignments.as_ref(), constraints))
      .block(
        Block::default()
          .title(Span::styled(self.title.unwrap_or_default(), Style::default().fg(Color::Blue)))
          .border_style(border_style)
          .borders(Borders::ALL),
      )
      .widths(constraints)
  }

  fn border_style(&self) -> Style {
    if self.focused {
      Style::default().fg(Color::Blue).add_modifier(Modifier::BOLD)
    } else {
      Style::default()
    }
  }

  fn align_rows(rows: Vec<Vec<Spans<'a>>>, alignments: Option<&Vec<Alignment>>, constraints: &[Constraint]) -> Vec<Row<'a>> {
    rows
      .into_iter()
      .map(|mut row| {
        if let Some(alignments) = alignments {
          for (spans, alignment, constraint) in itertools::izip!(&mut row, alignments, constraints) {
            if *alignment == Alignment::Right {
              if let Constraint::Length(width) = constraint {
                spans.0.insert(0, Span::raw(" ".repeat((*width as usize) - spans.width())));
              }
            }
          }
        }

        Row::new(row.into_iter().map(TuiCell::from).collect::<Vec<_>>())
      })
      .collect()
  }
}
