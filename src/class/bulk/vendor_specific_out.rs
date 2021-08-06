use crate::class::*;
use byteorder::{ByteOrder, LittleEndian};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct VendorSpecificOutHeader {
  pub bulk_out_header: BulkOutHeader,
  pub transfer_size: u32,
}

impl VendorSpecificOutHeader {
  pub fn new(b_tag: u8, transfer_size: u32) -> Self {
    let bulk_out_header = BulkOutHeader::new(MsgIdOut::VendorSpecificOut, b_tag);

    Self {
      bulk_out_header,
      transfer_size,
    }
  }

  pub fn pack(&self, buf: &mut [u8]) {
    self.bulk_out_header.pack(buf);
    LittleEndian::write_u32(&mut buf[4..8], self.transfer_size);
  }
}
