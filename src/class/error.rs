use crate::class::*;

use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ClassError {
  IllegalStatus,
  InvalidCapabilities,
  InvalidMsgId,
  InvalidTermChar,
  TagCheckFailure,
  TruncatedBulkOut,
  TruncatedControlResponse,
  TruncatedHeader,
  UnexpectedStatus(Status),
  UnsupportedFeature,
}

impl fmt::Display for ClassError {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    // TODO: better formatting probably
    write!(f, "{:?}", self)
  }
}
