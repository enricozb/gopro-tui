use std::path::PathBuf;

use clap::Parser;

#[derive(Parser, Debug)]
#[clap(version)]
pub struct Args {
  /// Source directory for gopro files
  #[clap(short, long)]
  pub input_dir: Option<PathBuf>,

  /// Destination directory for categorized files
  pub output_dir: PathBuf,
}
