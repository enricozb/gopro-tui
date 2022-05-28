use std::path::PathBuf;

use clap::Parser;

#[derive(Parser, Debug)]
#[clap(version)]
pub struct Args {
  /// Directory containing input files
  pub input_dir: PathBuf,

  /// Directory containing categorized files
  pub output_dir: PathBuf,
}
