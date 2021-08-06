pub use crate::class::ClassError;
use std::error::Error;
use std::fmt;
use std::string::FromUtf8Error;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TMCError {
  /// An error occurred in a generic USB operation
  Rusb(rusb::Error),

  /// An error occurred in the handling of a USB TMC class operation
  Class(ClassError),

  /// The application requested a string response, but the data from the device was not valid UTF-8
  FromUtf8Error(FromUtf8Error),
}

pub type TMCResult<T> = Result<T, TMCError>;

impl From<rusb::Error> for TMCError {
  fn from(item: rusb::Error) -> Self {
    TMCError::Rusb(item)
  }
}

impl From<ClassError> for TMCError {
  fn from(item: ClassError) -> Self {
    TMCError::Class(item)
  }
}

impl From<FromUtf8Error> for TMCError {
  fn from(item: FromUtf8Error) -> Self {
    TMCError::FromUtf8Error(item)
  }
}

impl fmt::Display for TMCError {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    use TMCError::*;

    match self {
      Rusb(msg) => {
        write!(f, "USB Error: {}", msg)
      }
      Class(msg) => {
        write!(f, "USB TMC Error: {}", msg)
      }
      FromUtf8Error(msg) => {
        write!(f, "Error decoding UTF-8 data: {}", msg)
      }
    }
  }
}

impl Error for TMCError {}
