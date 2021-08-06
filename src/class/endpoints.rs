/// Information about a USB device's TMC interface, needed to find the right
/// endpoints and such for communication to the instrument.
#[derive(Debug)]
pub struct TMCInterface {
  /// The ID of a USB interface on the instrument that complies to the
  /// USB Test and Measurement Class
  pub interface_number: u8,

  /// The interface's "protocol code", used to identify sub-classes
  /// such as USB488
  pub interface_protocol: u8,

  /// The endpoint address (not number) of the USB TMC bulk out endpoint
  /// (mandatory per section 2) for sending messages to the instrument
  pub bulk_out_address: u8,

  /// The endpoint address (not number) of the USB TMC bulk in endpoint
  /// (mandatory per section 2) for receiving messages from the instrument
  pub bulk_in_address: u8,

  /// The endpoint address (not number) of the USB TMC interrupt in endpoint
  /// (optional per section 2) for receiving asynchronous notifications from
  /// the instrument
  pub interrupt_in_address: Option<u8>,

  // TODO: these might not actually be needed.  The TMC spec does reference
  // them but I don't think it says anything that isn't implicit in the way
  // USB just works already.
  pub control_in_max_packet_size: u16,
  pub bulk_out_max_packet_size: u16,
  pub bulk_in_max_packet_size: u16,
}
