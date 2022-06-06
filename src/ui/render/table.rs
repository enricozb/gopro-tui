use std::cmp;

use tui::{
  buffer::Buffer,
  layout::{Constraint, Rect},
  style::{Modifier, Style},
  text::{Span, Spans},
  widgets::{Block, Borders, Cell as TuiCell, Row, StatefulWidget, Table as TuiTable, TableState, Widget},
};

use crate::ui::colors::Colors;

#[derive(PartialEq, Eq, Clone)]
pub enum Alignment {
  Left,
  Right,
}

type Rows<'a> = Vec<Vec<Spans<'a>>>;

#[derive(Default)]
pub struct Table<'a> {
  rows: Rows<'a>,
  title: Option<String>,
  alignments: Vec<Alignment>,
  focused: bool,
}

impl<'a> Table<'a> {
  pub fn new(rows: Rows<'a>) -> Self {
    Self { rows, ..Self::default() }
  }

  pub fn alignments<Alignments: Into<Vec<Alignment>>>(self, alignments: Alignments) -> Self {
    Self {
      alignments: alignments.into(),
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

  fn constraints(&self) -> Vec<Constraint> {
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

  fn border_style(&self) -> (Style, Style) {
    let border_style = if self.focused {
      Style::default().fg(Colors::normal().focused_block).add_modifier(Modifier::BOLD)
    } else {
      Style::default()
    };

    let title_style = border_style.fg(Colors::normal().section_title);

    (title_style, border_style)
  }

  fn aligned_rows(rows: Rows<'a>, alignments: Vec<Alignment>, constraints: &'a [Constraint]) -> Vec<Row<'a>> {
    rows
      .into_iter()
      .map(|mut row| {
        for (spans, alignment, constraint) in itertools::izip!(&mut row, &alignments, constraints) {
          if *alignment == Alignment::Right {
            if let Constraint::Length(width) = constraint {
              spans.0.insert(0, Span::raw(" ".repeat((*width as usize) - spans.width())));
            }
          }
        }

        Row::new(row.into_iter().map(TuiCell::from).collect::<Vec<_>>())
      })
      .collect()
  }

  fn widget(self, constraints: &'a [Constraint]) -> TuiTable<'a> {
    let title = self.title.clone().unwrap_or_default();
    let (title_style, border_style) = self.border_style();

    TuiTable::new(Self::aligned_rows(self.rows, self.alignments, &constraints))
      .block(
        Block::default()
          .title(Span::styled(title, title_style))
          .border_style(border_style)
          .borders(Borders::ALL),
      )
      .widths(&constraints)
  }
}

impl<'a> Widget for Table<'a> {
  fn render(self, area: Rect, buf: &mut Buffer) {
    let constraints = self.constraints();

    Widget::render(self.widget(&constraints), area, buf);
  }
}

impl<'a> StatefulWidget for Table<'a> {
  type State = TableState;

  fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
    let constraints = self.constraints();

    StatefulWidget::render(self.widget(&constraints), area, buf, state);
  }
}
