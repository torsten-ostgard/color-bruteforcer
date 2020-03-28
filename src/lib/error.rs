use std::error;
use std::fmt;

use promptly::ReadlineError;

#[derive(Debug)]
pub enum GetColorError {
    MismatchedColors,
    ReadlineError(ReadlineError),
}

impl fmt::Display for GetColorError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            GetColorError::MismatchedColors => {
                write!(f, "The number of base colors and target colors must match")
            }
            GetColorError::ReadlineError(ref e) => e.fmt(f),
        }
    }
}

impl error::Error for GetColorError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match *self {
            GetColorError::MismatchedColors => None,
            GetColorError::ReadlineError(ref e) => Some(e),
        }
    }
}

impl From<ReadlineError> for GetColorError {
    fn from(err: ReadlineError) -> GetColorError {
        GetColorError::ReadlineError(err)
    }
}
