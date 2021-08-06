use crate::class::*;
use crate::{InstrumentHandle, TMCResult};
use rusb;

/// Information about an instrument detected on the USB bus.
///
/// Use [list_instruments] to find all connected instruments.
/// Use [Instrument::open] to get an [InstrumentHandle] which
/// can be used to communicate with the instrument.
#[derive(Debug)]
pub struct Instrument<Ctx: rusb::UsbContext> {
  pub device: rusb::Device<Ctx>,
  pub device_desc: rusb::DeviceDescriptor,
  pub config_desc: rusb::ConfigDescriptor,
  pub endpoints: TMCInterface,

  serial_number_loaded: bool,
  serial_number: Option<String>,
}

impl<Ctx: rusb::UsbContext> Instrument<Ctx> {
  fn read_serial_number(&mut self) -> TMCResult<Option<String>> {
    if !self.serial_number_loaded {
      self.serial_number = match self.device_desc.serial_number_string_index() {
        None => None,
        Some(index) => Some(self.device.open()?.read_string_descriptor_ascii(index)?),
      };

      self.serial_number_loaded = true;
    }

    Ok(self.serial_number.clone())
  }

  /// Get the device's resource string; this may involve connecting to it in order to read its serial number.
  pub fn read_resource_string(&mut self) -> TMCResult<String> {
    let vendor_id = self.device_desc.vendor_id();
    let product_id = self.device_desc.product_id();

    match self.read_serial_number()? {
      None => Ok(format!("USB::{}::{}::INSTR", vendor_id, product_id)),
      Some(serial_number) => Ok(format!(
        "USB::{}::{}::{}::INSTR",
        vendor_id, product_id, serial_number
      )),
    }
  }

  pub fn open(mut self) -> TMCResult<InstrumentHandle<Ctx>> {
    self.read_serial_number()?;

    InstrumentHandle::connect(self)
  }
}

fn is_usbtmc_device<Ctx: rusb::UsbContext>(
  device: rusb::Device<Ctx>,
) -> TMCResult<Option<Instrument<Ctx>>> {
  let device_desc = device.device_descriptor()?;

  for cfg_id in 0..device_desc.num_configurations() {
    let config_desc = match device.config_descriptor(cfg_id) {
      Err(_) => continue,
      Ok(desc) => desc,
    };

    let mut found_interface: Option<TMCInterface> = None;
    for interface in config_desc.interfaces() {
      for interface_desc in interface.descriptors() {
        if interface_desc.class_code() == 0xFE && interface_desc.sub_class_code() == 3 {
          let mut control_in_max_packet_size: u16 = 0;
          let mut bulk_in_max_packet_size: u16 = 0;
          let mut bulk_in_address: Option<u8> = None;
          let mut bulk_out_max_packet_size: u16 = 0;
          let mut bulk_out_address: Option<u8> = None;
          let mut interrupt_in_address: Option<u8> = None;

          for ep_desc in interface_desc.endpoint_descriptors() {
            use rusb::Direction::*;
            use rusb::TransferType::*;

            match (ep_desc.transfer_type(), ep_desc.direction()) {
              (Control, In) => {
                control_in_max_packet_size = ep_desc.max_packet_size();
              }
              (Bulk, In) => {
                bulk_in_address = Some(ep_desc.address());
                bulk_in_max_packet_size = ep_desc.max_packet_size();
              }
              (Bulk, Out) => {
                bulk_out_address = Some(ep_desc.address());
                bulk_out_max_packet_size = ep_desc.max_packet_size();
              }
              (Interrupt, In) => {
                interrupt_in_address = Some(ep_desc.address());
              }
              (_, _) => {
                // ignore extra endpoints
              }
            }
          }

          if let (Some(bulk_in_address), Some(bulk_out_address)) =
            (bulk_in_address, bulk_out_address)
          {
            found_interface = Some(TMCInterface {
              interface_number: interface_desc.interface_number(),
              interface_protocol: interface_desc.protocol_code(),
              control_in_max_packet_size,
              bulk_in_address,
              bulk_in_max_packet_size,
              bulk_out_address,
              bulk_out_max_packet_size,
              interrupt_in_address,
            });

            break;
          }
        }
      }

      if found_interface.is_some() {
        break;
      }
    }

    if let Some(endpoints) = found_interface {
      let mut instrument = Instrument {
        device,
        device_desc,
        config_desc,
        endpoints,

        serial_number_loaded: false,
        serial_number: None,
      };

      // Try to read the serial number; this will attempt to connect, but we don't mind
      // if it fails.
      let _ = instrument.read_serial_number();

      return Ok(Some(instrument));
    }
  }

  Ok(None)
}

/// List detected USBTMC devices
pub fn list_instruments<Ctx: rusb::UsbContext>(context: Ctx) -> TMCResult<Vec<Instrument<Ctx>>> {
  let all_devices = context.devices()?;
  let mut usbtmc_devices = Vec::new();

  for device in all_devices.iter() {
    if let Some(device) = is_usbtmc_device(device)? {
      usbtmc_devices.push(device);
    }
  }

  Ok(usbtmc_devices)
}

pub fn find_instrument_with_vid_pid<Ctx: rusb::UsbContext>(
  context: Ctx,
  vendor_id: u16,
  product_id: u16,
) -> TMCResult<Option<Instrument<Ctx>>> {
  // rusb doesn't currently have a simple way to find a Device by vid and pid without opening it
  for device in list_instruments(context)? {
    if device.device_desc.vendor_id() == vendor_id && device.device_desc.product_id() == product_id
    {
      return Ok(Some(device));
    }
  }

  Ok(None)
}
