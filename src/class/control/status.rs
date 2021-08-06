use crate::class::*;
use std::convert::TryFrom;

#[repr(u8)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum Status {
  Success,
  Pending,
  Failed,
  TransferNotInProgress,
  SplitNotInProgress,
  SplitInProgress,
  UnknownWarning(u8),
  UnknownFailure(u8),
}

impl TryFrom<u8> for Status {
  type Error = ClassError;
  fn try_from(value: u8) -> Result<Self, Self::Error> {
    use Status::*;
    match value {
      0x00 => Err(ClassError::IllegalStatus),
      0x01 => Ok(Success),
      0x02 => Ok(Pending),
      0x80 => Ok(Failed),
      0x81 => Ok(TransferNotInProgress),
      0x82 => Ok(SplitNotInProgress),
      0x83 => Ok(SplitInProgress),
      _ => {
        if value < 0x80 {
          Ok(UnknownWarning(value))
        } else {
          Ok(UnknownFailure(value))
        }
      }
    }
  }
}

impl Status {
  pub fn check(self) -> Result<(), ClassError> {
    if self == Status::Success {
      Ok(())
    } else {
      Err(ClassError::UnexpectedStatus(self))
    }
  }
}
