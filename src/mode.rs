use std::path::PathBuf;

use crate::args::Args;

#[derive(Clone)]
pub enum Mode {
  Importing { input_dir: PathBuf, output_dir: PathBuf },
  Viewing { input_dir: PathBuf },
}

impl Mode {
  pub fn from(args: Args) -> Self {
    match args.input_dir {
      Some(input_dir) => Mode::Importing {
        input_dir,
        output_dir: args.output_dir,
      },

      None => Mode::Viewing {
        input_dir: args.output_dir,
      },
    }
  }

  pub fn input_dir(&self) -> PathBuf {
    match self {
      Mode::Importing { input_dir, .. } | Mode::Viewing { input_dir } => input_dir.clone(),
    }
  }

  pub fn output_dir(&self) -> Option<PathBuf> {
    match self {
      Mode::Importing { output_dir, .. } => Some(output_dir.clone()),
      Mode::Viewing { .. } => None,
    }
  }
}
