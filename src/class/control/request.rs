use crate::class::*;
use std::convert::TryFrom;

#[repr(u8)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum ControlRequest {
  InitiateAbortBulkOut = 1,
  CheckAbortBulkOutStatus = 2,
  InitiateAbortBulkIn = 3,
  CheckAbortBulkInStatus = 4,
  InitiateClear = 5,
  CheckClearStatus = 6,
  GetCapabilities = 7,
  IndicatorPulse = 64,
}

impl From<ControlRequest> for u8 {
  fn from(value: ControlRequest) -> Self {
    value as u8
  }
}

impl ControlRequest {
  /// Attempt to read the first byte of the provided buffer as a USB TMC status code
  pub fn read_response_status(buf: &[u8]) -> Result<Status, ClassError> {
    if buf.is_empty() {
      Err(ClassError::TruncatedControlResponse)
    } else {
      Status::try_from(buf[0])
    }
  }

  /// Check the first byte of a buffer (if it's long enough) and ensure it indicates a "success" status.
  pub fn check_response_status(buf: &[u8]) -> Result<(), ClassError> {
    if buf.is_empty() {
      Err(ClassError::TruncatedControlResponse)
    } else {
      Status::try_from(buf[0])?.check()
    }
  }
}
