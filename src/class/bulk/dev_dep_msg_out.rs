use crate::class::*;
use byteorder::{ByteOrder, LittleEndian};

/// Header for bulk out command messages.  This header should be followed by
/// `transfer_size` bytes of command data.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DevDepMsgOutHeader {
  pub bulk_out_header: BulkOutHeader,
  pub transfer_size: u32,
  pub transfer_attributes: u8,
}

impl DevDepMsgOutHeader {
  pub fn new(b_tag: u8, transfer_size: u32, eom: bool) -> Self {
    let bulk_out_header = BulkOutHeader::new(MsgIdOut::DevDepMsgOut, b_tag);

    let transfer_attributes = if eom { 1 } else { 0 };
    Self {
      bulk_out_header,
      transfer_size,
      transfer_attributes,
    }
  }

  pub fn pack(&self, buf: &mut [u8]) {
    self.bulk_out_header.pack(buf);
    LittleEndian::write_u32(&mut buf[4..8], self.transfer_size);
    buf[8] = self.transfer_attributes;
  }

  pub fn encode_message(b_tag: u8, data: &[u8], eom: bool, buf: &mut Vec<u8>) {
    // add the header
    buf.resize(HEADER_SIZE, 0u8);
    DevDepMsgOutHeader::new(b_tag, data.len() as u32, eom).pack(buf);

    // add the data
    buf.extend_from_slice(data);

    // pad to next multiple of 4
    let len = buf.len();
    let padded_len = (len + 3) & !3;
    if len != padded_len {
      buf.resize(padded_len, 0);
    }
  }
}
