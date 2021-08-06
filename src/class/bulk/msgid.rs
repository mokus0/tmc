use crate::class::ClassError;
use std::convert::{From, Into, TryFrom};

/// A marker trait for message ID enums
pub trait MsgId: Copy + Sized + Into<u8> + TryFrom<u8, Error = ClassError> {}

#[repr(u8)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum MsgIdOut {
  DevDepMsgOut = 1,
  RequestDevDepMsgIn = 2,
  VendorSpecificOut = 3,
  RequestVendorSpecificIn = 4,
}

impl From<MsgIdOut> for u8 {
  fn from(value: MsgIdOut) -> Self {
    value as u8
  }
}

impl TryFrom<u8> for MsgIdOut {
  type Error = ClassError;
  fn try_from(value: u8) -> Result<Self, Self::Error> {
    match value {
      1 => Ok(Self::DevDepMsgOut),
      2 => Ok(Self::RequestDevDepMsgIn),
      3 => Ok(Self::VendorSpecificOut),
      4 => Ok(Self::RequestVendorSpecificIn),
      _ => Err(ClassError::InvalidMsgId),
    }
  }
}

impl MsgId for MsgIdOut {}

#[repr(u8)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum MsgIdIn {
  DevDepMsgIn = 2,
  VendorSpecificIn = 4,
}

impl From<MsgIdIn> for u8 {
  fn from(value: MsgIdIn) -> Self {
    value as u8
  }
}

impl TryFrom<u8> for MsgIdIn {
  type Error = ClassError;
  fn try_from(value: u8) -> Result<Self, Self::Error> {
    match value {
      2 => Ok(Self::DevDepMsgIn),
      4 => Ok(Self::VendorSpecificIn),
      _ => Err(ClassError::InvalidMsgId),
    }
  }
}

impl MsgId for MsgIdIn {}
