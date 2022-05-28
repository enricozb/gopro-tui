use std::path::PathBuf;

use clap::Parser;

#[derive(Parser, Debug)]
#[clap(version)]
pub struct Args {
  /// Source directory for gopro files
  pub src_dir: PathBuf,

  /// Destination directory for categorized files
  pub dst_dir: PathBuf,
}
