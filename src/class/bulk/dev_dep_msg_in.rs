use crate::class::*;
use byteorder::{ByteOrder, LittleEndian};

/// Header for a request to the device to send data.  After this command (which consists of
/// just the command header), the device is expected to send up to `transfer_size` bytes of
/// response data (excluding the header and alignment bytes, if any, sent with the response).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RequestDevDepMsgInHeader {
  pub bulk_out_header: BulkOutHeader,
  pub transfer_size: u32,
  pub transfer_attributes: u8,
  pub term_char: u8,
}

impl RequestDevDepMsgInHeader {
  pub fn new(b_tag: u8, transfer_size: u32, opt_term_char: Option<u8>) -> Self {
    let bulk_out_header = BulkOutHeader::new(MsgIdOut::RequestDevDepMsgIn, b_tag);

    let transfer_attributes: u8;
    let term_char: u8;
    match opt_term_char {
      None => {
        transfer_attributes = 0;
        term_char = 0;
      }
      Some(c) => {
        transfer_attributes = 2;
        term_char = c;
      }
    }

    Self {
      bulk_out_header,
      transfer_size,
      transfer_attributes,
      term_char,
    }
  }

  pub fn pack(&self, buf: &mut [u8]) {
    self.bulk_out_header.pack(buf);
    LittleEndian::write_u32(&mut buf[4..8], self.transfer_size);
    buf[8] = self.transfer_attributes;
    buf[9] = self.term_char;
  }

  pub fn encode_message(b_tag: u8, transfer_size: u32, term_char: Option<u8>, buf: &mut Vec<u8>) {
    buf.resize(HEADER_SIZE, 0);
    RequestDevDepMsgInHeader::new(b_tag, transfer_size, term_char).pack(buf);
  }
}

/// Header sent by device describing the data about to be sent, in response
/// to a RequestDevDepMsgIn command.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DevDepMsgInHeader {
  pub bulk_in_header: BulkInHeader,
  pub transfer_size: u32,
  pub transfer_attributes: u8,
}

impl DevDepMsgInHeader {
  pub fn unpack(data: &[u8]) -> Result<Self, ClassError> {
    let bulk_in_header = BulkTransferHeader::unpack(data)?;

    let transfer_size = LittleEndian::read_u32(&data[4..8]);
    let transfer_attributes = data[8];

    Ok(Self {
      bulk_in_header,
      transfer_size,
      transfer_attributes,
    })
  }

  pub fn decode_transfer(buf: &[u8]) -> Result<(Self, &[u8]), ClassError> {
    let header = Self::unpack(buf)?;
    let data = &buf[HEADER_SIZE..HEADER_SIZE + header.transfer_size as usize];
    Ok((header, data))
  }

  pub fn is_eom(&self) -> bool {
    self.transfer_attributes & 0x01 != 0
  }

  pub fn has_term_char(&self) -> bool {
    self.transfer_attributes & 0x02 != 0
  }
}
