use tui::style::Color;

macro_rules! colors {
  ($($name:ident: $color:expr),* $(,)?) => {
    const fn focused_color(color: Color) -> Color {
      match color {
        Color::Red => Color::LightRed,
        Color::Magenta => Color::LightMagenta,
        Color::Green => Color::LightGreen,
        Color::Gray => Color::White,
        Color::Yellow => Color::LightYellow,
        Color::Blue => Color::LightBlue,
        color => color,
      }
    }

    pub struct Colors {
      $(pub $name: Color),*
    }

    impl Colors {
      pub fn normal() -> Colors {
        COLORS_NORMAL
      }

      pub fn focused(focused: bool) -> Colors {
        if focused {
          COLORS_FOCUSED
        } else {
          COLORS_NORMAL
        }
      }
    }

    const COLORS_NORMAL: Colors = Colors {
      $($name: $color),*
    };

    const COLORS_FOCUSED: Colors = Colors {
      $($name: focused_color($color)),*
    };
  };
}

colors! {
 date: Color::Magenta,
 count: Color::Gray,
 size: Color::Yellow,
 duration: Color::Green,
 filename: Color::Gray,
 destination: Color::Blue,
 status_import: Color::Green,
 status_ignore: Color::Red,
 status_none: Color::Blue,

 section_title: Color::Blue,
 focused_block: Color::Blue,

 input_block: Color::Green,
 input_text: Color::White,

 error_block: Color::Red,
}
