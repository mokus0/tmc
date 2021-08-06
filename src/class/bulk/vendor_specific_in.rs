use crate::class::*;
use byteorder::{ByteOrder, LittleEndian};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RequestVendorSpecificInHeader {
  pub bulk_out_header: BulkOutHeader,
  pub transfer_size: u32,
}

impl RequestVendorSpecificInHeader {
  pub fn new(b_tag: u8, transfer_size: u32) -> Self {
    let bulk_out_header = BulkOutHeader::new(MsgIdOut::RequestVendorSpecificIn, b_tag);

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

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct VendorSpecificInHeader {
  pub bulk_in_header: BulkInHeader,
  pub transfer_size: u32,
}

impl VendorSpecificInHeader {
  pub fn unpack(data: &[u8]) -> Result<Self, ClassError> {
    let bulk_in_header = BulkTransferHeader::unpack(data)?;

    let transfer_size = LittleEndian::read_u32(&data[4..8]);

    Ok(Self {
      bulk_in_header,
      transfer_size,
    })
  }

  pub fn decode_transfer(buf: &[u8]) -> Result<(Self, &[u8]), ClassError> {
    let header = Self::unpack(buf)?;
    let data = &buf[HEADER_SIZE..HEADER_SIZE + header.transfer_size as usize];
    Ok((header, data))
  }
}
