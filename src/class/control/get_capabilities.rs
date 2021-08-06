use crate::class::*;
use byteorder::{ByteOrder, LittleEndian};
use std::convert::TryFrom;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct USBTMCCapabilities {
  pub bcd_usbtmc: u16,
  pub pulse: bool,
  pub talk_only: bool,
  pub listen_only: bool,
  pub term_char: bool,
}

impl USBTMCCapabilities {
  /// create a new placeholder value, which is not valid.  Mostly just to avoid needing Option
  /// in the instrument handle type, since these will be read immediately during the connection
  /// process.
  pub fn new() -> Self {
    Self {
      bcd_usbtmc: 0,
      pulse: false,
      talk_only: false,
      listen_only: false,
      term_char: false,
    }
  }

  pub fn is_valid(&self) -> bool {
    self.bcd_usbtmc >= 0x0100
  }

  /// parse a "GET_CAPABILTIES" response.  The status field is checked and must be SUCCESS.
  pub fn parse(buf: &[u8]) -> Result<Self, ClassError> {
    if buf.len() < 12 {
      Err(ClassError::TruncatedControlResponse)
    } else {
      Status::try_from(buf[0])?.check()?;

      Ok(Self {
        bcd_usbtmc: LittleEndian::read_u16(&buf[2..4]),
        pulse: buf[4] & 0x04 != 0,
        talk_only: buf[4] & 0x02 != 0,
        listen_only: buf[4] & 0x01 != 0,
        term_char: buf[5] & 0x01 != 0,
      })
    }
  }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct USB488Capabilities {
  pub bcd_usb488: u16,
  pub usb488_2: bool,
  pub remote_local: bool,
  pub trigger: bool,
  pub scpi: bool,
  pub sr: bool,
  pub rl: bool,
  pub dt: bool,
}

impl USB488Capabilities {
  pub fn parse(
    usbtmc_capabilities: &USBTMCCapabilities,
    buf: &[u8],
  ) -> Result<Option<Self>, ClassError> {
    if buf.len() < 16 {
      return Ok(None);
    }

    let usb488_capabilities = USB488Capabilities {
      bcd_usb488: LittleEndian::read_u16(&buf[12..14]),
      usb488_2: buf[14] & 0x04 != 0,
      remote_local: buf[14] & 0x02 != 0,
      trigger: buf[14] & 0x01 != 0,
      scpi: buf[15] & 0x08 != 0,
      sr: buf[15] & 0x04 != 0,
      rl: buf[15] & 0x02 != 0,
      dt: buf[15] & 0x01 != 0,
    };

    if usb488_capabilities.bcd_usb488 < 0x0100 {
      return Ok(None);
    }

    // reject several combinations of features that are defined as invalid in USB488 spec
    if usbtmc_capabilities.talk_only || usbtmc_capabilities.listen_only {
      return Err(ClassError::InvalidCapabilities);
    }

    if usb488_capabilities.dt && !usb488_capabilities.trigger {
      return Err(ClassError::InvalidCapabilities);
    }

    if usb488_capabilities.rl && !usb488_capabilities.remote_local {
      return Err(ClassError::InvalidCapabilities);
    }

    if usb488_capabilities.usb488_2 && !usb488_capabilities.sr {
      return Err(ClassError::InvalidCapabilities);
    }

    if usb488_capabilities.scpi && !usb488_capabilities.usb488_2 {
      return Err(ClassError::InvalidCapabilities);
    }

    // looks good, return the capabilities list
    Ok(Some(usb488_capabilities))
  }
}
