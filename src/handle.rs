use crate::class::*;
use crate::{Instrument, TMCResult};
use core::time::Duration;
use rusb::DeviceHandle;
use rusb::UsbContext;
use std::str;
use std::thread::sleep;

pub struct InstrumentHandle<Ctx: UsbContext> {
  usb: DeviceHandle<Ctx>,

  b_tag: u8,
  max_transfer_size: u32,
  term_char: Option<u8>,
  timeout: Duration,

  pub instrument: Instrument<Ctx>,
  pub usbtmc_capabilities: USBTMCCapabilities,
  pub usb488_capabilities: Option<USB488Capabilities>,
  pub scpi_id: Option<String>,

  // When connecting, we may need to reconfigure some stuff.  Remember the
  // previous state here and restore it on drop().
  restore_config: Option<u8>,
  reattach_kernel_driver: Vec<u8>,
}

impl<Ctx: UsbContext> Drop for InstrumentHandle<Ctx> {
  fn drop(&mut self) {
    // TODO: is there something more useful we can do if these fail?
    let endpoints = &self.instrument.endpoints;

    let _ = self.usb.release_interface(endpoints.interface_number);

    if let Some(old_config) = self.restore_config {
      let _ = self.usb.set_active_configuration(old_config);
    }

    for &interface in self.reattach_kernel_driver.iter() {
      let _ = self.usb.attach_kernel_driver(interface);
    }
  }
}

impl<Ctx: UsbContext> InstrumentHandle<Ctx> {
  pub(crate) fn connect(instrument: Instrument<Ctx>) -> TMCResult<Self> {
    let usb = instrument.device.open()?;

    let mut handle = Self {
      instrument,
      usb,

      b_tag: 0,
      max_transfer_size: 1024 * 1024,
      timeout: Duration::from_secs(1),
      term_char: None,

      restore_config: None,
      reattach_kernel_driver: Vec::new(),

      usbtmc_capabilities: USBTMCCapabilities::new(),
      usb488_capabilities: None,
      scpi_id: None,
    };
    let usb = &mut handle.usb;
    let endpoints = &handle.instrument.endpoints;

    let old_config = usb.active_configuration()?;

    if old_config != 0 {
      match handle.instrument.device.config_descriptor(old_config) {
        Err(rusb::Error::NotFound) => {}
        Err(rusb_error) => return Err(rusb_error.into()),
        Ok(old_config_desc) => {
          for interface in 0..old_config_desc.num_interfaces() {
            if usb.kernel_driver_active(interface)? {
              handle.reattach_kernel_driver.push(interface);
              usb.detach_kernel_driver(interface)?;
            }
          }
        }
      };
    }

    let new_config = handle.instrument.config_desc.number();
    if old_config != new_config {
      handle.restore_config = Some(old_config);
      usb.set_active_configuration(new_config)?;
    }

    usb.claim_interface(endpoints.interface_number)?;

    handle.clear()?;
    handle.get_capabilities()?;

    if let Some(caps) = &handle.usb488_capabilities {
      if caps.scpi {
        if let Ok(id_str) = handle.ask("*IDN?") {
          handle.scpi_id = Some(id_str.trim().to_owned());
        }
      }
    }

    Ok(handle)
  }

  pub fn get_max_transfer_size(&self) -> u32 {
    self.max_transfer_size
  }

  pub fn set_max_transfer_size(&mut self, max_transfer_size: u32) {
    self.max_transfer_size = max_transfer_size;
  }

  pub fn get_term_char(&self) -> Option<u8> {
    self.term_char
  }

  pub fn set_term_char(&mut self, term_char: Option<u8>) -> TMCResult<()> {
    if term_char == Some(0) {
      return Err(ClassError::InvalidTermChar.into());
    }

    if term_char.is_some() && !self.usbtmc_capabilities.term_char {
      return Err(ClassError::UnsupportedFeature.into());
    }

    self.term_char = term_char;
    Ok(())
  }

  pub fn get_timeout(&self) -> Duration {
    self.timeout
  }

  pub fn set_timeout(&mut self, timeout: Duration) {
    self.timeout = timeout;
  }

  fn read_control(
    &self,
    request: ControlRequest,
    read_size: usize,
    out: &mut Vec<u8>,
  ) -> TMCResult<()> {
    let request_type = rusb::request_type(
      rusb::Direction::In,
      rusb::RequestType::Class,
      rusb::Recipient::Interface,
    );

    out.resize(read_size, 0);
    let size = self.usb.read_control(
      request_type,
      request as u8,
      0x0000,
      self.instrument.endpoints.interface_number as u16,
      out,
      self.timeout,
    )?;
    out.truncate(size);

    Ok(())
  }

  // TODO: these messages are defined in the class spec, are they useful?
  //
  // I think it might be useful to use abort_bulk_in when connecting to
  // make sure we don't read stale data if the last connection was interrupted.
  //
  // pub fn abort_bulk_out(...)
  // pub fn abort_bulk_in(...)

  // Send USBTMC "clear" command
  pub fn clear(&mut self) -> TMCResult<()> {
    let mut out = Vec::with_capacity(2);
    self.read_control(ControlRequest::InitiateClear, 1, &mut out)?;

    ControlRequest::check_response_status(&out)?;

    // device accepted `clear` command, wait while status is "pending"
    loop {
      self.read_control(ControlRequest::CheckClearStatus, 2, &mut out)?;

      match ControlRequest::read_response_status(&out)? {
        Status::Success => break,
        Status::Pending => {}
        status => status.check()?,
      };

      sleep(Duration::from_millis(100));
    }

    self
      .usb
      .clear_halt(self.instrument.endpoints.bulk_out_address)?;
    Ok(())
  }

  fn get_capabilities(&mut self) -> TMCResult<()> {
    // 64 bytes is the largest possible control transfer, so use that to avoid
    // overflow if the device sends a lot back.
    let mut out = vec![0u8; 64];
    self.read_control(ControlRequest::GetCapabilities, 64, &mut out)?;

    self.usbtmc_capabilities = USBTMCCapabilities::parse(&out)?;

    if self.instrument.endpoints.interface_protocol == 1 {
      self.usb488_capabilities = USB488Capabilities::parse(&self.usbtmc_capabilities, &out)?;
    }

    Ok(())
  }

  pub fn pulse(&self) -> TMCResult<()> {
    if !self.usbtmc_capabilities.pulse {
      return Err(ClassError::UnsupportedFeature.into());
    }

    let mut out = Vec::with_capacity(1);
    self.read_control(ControlRequest::IndicatorPulse, 1, &mut out)?;
    ControlRequest::check_response_status(&out)?;
    Ok(())
  }

  fn incr_b_tag(&mut self) {
    // bTag must be different on each successive bulk-out transfer and not 0
    self.b_tag = if self.b_tag == 255 { 1 } else { self.b_tag + 1 };
  }

  /// Write a command message to the instrument
  pub fn write_raw(&mut self, data: &[u8]) -> TMCResult<()> {
    let ep = self.instrument.endpoints.bulk_out_address;

    let mut buf = Vec::with_capacity(HEADER_SIZE + data.len() + 3);
    let mut end_offset: usize = 0;

    for block in data.chunks(self.max_transfer_size as usize) {
      end_offset += block.len();
      let eom = end_offset >= data.len();

      self.incr_b_tag();
      DevDepMsgOutHeader::encode_message(self.b_tag, data, eom, &mut buf);

      let n_written = self.usb.write_bulk(ep, &buf, self.timeout)?;
      if n_written < block.len() {
        return Err(ClassError::TruncatedBulkOut.into());
      }
    }

    Ok(())
  }

  /// Read response data from the instrument
  pub fn read_raw(&mut self, transfer_size: Option<u32>) -> TMCResult<Vec<u8>> {
    let transfer_size = match transfer_size {
      Some(size) if size < self.max_transfer_size => size,
      _ => self.max_transfer_size,
    };

    let mut read_data = Vec::with_capacity(HEADER_SIZE + transfer_size as usize + 3);
    let mut buf = Vec::new();

    loop {
      // Send OUT command header to request device send data
      self.incr_b_tag();
      RequestDevDepMsgInHeader::encode_message(self.b_tag, transfer_size, self.term_char, &mut buf);
      self.usb.write_bulk(
        self.instrument.endpoints.bulk_out_address,
        &buf,
        self.timeout,
      )?;

      // Read the requested data from the device. Extra space in output buffer is
      // for the bulk-in header and 3 potential alignment-padding bytes.
      buf.resize(HEADER_SIZE + transfer_size as usize + 3, 0);
      let n_read = self.usb.read_bulk(
        self.instrument.endpoints.bulk_in_address,
        &mut buf,
        self.timeout,
      )?;
      buf.truncate(n_read);

      let (header, data) = DevDepMsgInHeader::decode_transfer(&buf)?;
      read_data.extend_from_slice(data);

      if header.is_eom() {
        break;
      }
    }

    Ok(read_data)
  }

  /// Read UTF-8 response data from the instrument
  pub fn read(&mut self, transfer_size: Option<u32>) -> TMCResult<String> {
    let read_data = self.read_raw(transfer_size)?;
    Ok(String::from_utf8(read_data)?)
  }

  /// Write a UTF-8 command message to the instrument
  pub fn write(&mut self, message: &str) -> TMCResult<()> {
    self.write_raw(message.as_bytes())
  }

  /// Write a UTF-8 command message to the instrument and read a UTF-8 response
  pub fn ask(&mut self, data: &str) -> TMCResult<String> {
    let response_data = self.ask_raw(data.as_bytes())?;
    let response_str = String::from_utf8(response_data)?;
    Ok(response_str)
  }

  /// Write a command message to the instrument and read a response
  pub fn ask_raw(&mut self, data: &[u8]) -> TMCResult<Vec<u8>> {
    self.write_raw(data)?;
    self.read_raw(None)
  }

  // TODO: support for vendor-specific bulk transfers
  // TODO: support for interrupt in endpoint
  // TODO: more complete support for USB488 features
}
