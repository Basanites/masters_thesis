use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub enum ImportError {
    MissingFile(String),
    InvalidFormat(String),
}

impl fmt::Display for ImportError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingFile(file) => write!(f, "Missing file: {}", file),
            Self::InvalidFormat(msg) => write!(f, "Invalid format on file: {}", msg),
        }
    }
}

impl Error for ImportError {}
