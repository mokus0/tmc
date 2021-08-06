use crate::class::*;

/// All bulk transfer headers used in this class are the same size.
pub const HEADER_SIZE: usize = 12;

/// Common data in all bulk transfer headers (Sections 3.2 and 3.3), excluding
/// the command-specific portion of the header.  Command-specific header types
/// will embed this struct and add the additional fields.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct BulkTransferHeader<M: MsgId> {
  pub msg_id: M,
  pub b_tag: u8,
  pub b_tag_inverse: u8,
  pub reserved: u8,
}

pub type BulkOutHeader = BulkTransferHeader<MsgIdOut>;
pub type BulkInHeader = BulkTransferHeader<MsgIdIn>;

impl<M: MsgId> BulkTransferHeader<M> {
  pub fn new(msg_id: M, b_tag: u8) -> Self {
    Self {
      msg_id,
      b_tag,
      b_tag_inverse: !b_tag,
      reserved: 0,
    }
  }

  pub fn pack(&self, buf: &mut [u8]) {
    buf[0] = self.msg_id.into();
    buf[1] = self.b_tag;
    buf[2] = self.b_tag_inverse;
    buf[3] = self.reserved;
  }

  pub fn unpack(data: &[u8]) -> Result<Self, ClassError> {
    if data.len() < HEADER_SIZE {
      return Err(ClassError::TruncatedHeader);
    }

    let msg_id = M::try_from(data[0])?;
    let b_tag = data[1];
    let b_tag_inverse = data[2];
    let reserved = data[3];

    if b_tag_inverse != !b_tag {
      Err(ClassError::TagCheckFailure)
    } else {
      Ok(Self {
        msg_id,
        b_tag,
        b_tag_inverse,
        reserved,
      })
    }
  }
}
