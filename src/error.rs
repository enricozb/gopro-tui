use std::result;

use stable_eyre::eyre::Report;
pub use stable_eyre::eyre::{eyre as err, WrapErr};

pub type Result<T> = result::Result<T, Report>;
