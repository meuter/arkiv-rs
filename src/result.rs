use std::{fmt::Display, io};

/// Error type used throughout this crate
#[derive(Debug)]
pub enum Error {
    /// An error caused by I/O
    Io(std::io::Error),

    /// This file is not a valid archive
    InvalidArchive(&'static str),

    /// This archive is not supported
    UnsupportedArchive(&'static str),

    /// The requested file could not be found in the archive
    FileNotFound,
}

/// Result type used throughout this crate
pub type Result<T> = std::result::Result<T, Error>;

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Self {
        Error::Io(err)
    }
}

impl Display for Error {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Io(err) => write!(fmt, "{err}"),
            Error::InvalidArchive(err) => write!(fmt, "invalid archive: {err}"),
            Error::UnsupportedArchive(err) => write!(fmt, "unsupported archive: {err}"),
            Error::FileNotFound => write!(fmt, "specified file not found in archive"),
        }
    }
}

impl std::error::Error for Error {}
