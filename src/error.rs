use thiserror::Error;

use std::io;
use std::path::PathBuf;

#[derive(Error, Debug)]
pub enum LineError {
    #[error("If the line contains \"{}\", then it must start with \"{}\".", .0, .0)]
    StartsWith(String),
    #[error("Leave a space between `Mathlib` imports and `Dino` imports.")]
    LeaveSpace,
}

#[derive(Error, Debug)]
pub enum Error {
    #[error("\x1b[31m{}:{}\x1b[m : Error\n{}", .filepath.display(), .line_idx + 1, .err)]
    Content { filepath: PathBuf, line_idx: usize, err: LineError },
    #[error("{}", .0)]
    Io(#[from] io::Error),
    #[error("{}", .0)]
    Custom(String),
}

pub type Result<T, E = Error> = core::result::Result<T, E>;
