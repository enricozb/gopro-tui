use std::result;

use stable_eyre::eyre::Report;

pub type Result<T> = result::Result<T, Report>;
