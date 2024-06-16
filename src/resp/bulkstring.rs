use std::ops::Deref;

use crate::resp::frame::DecodeErr::InvalidLength;
use crate::resp::frame::{DecodeErr, Decoded, EncodeErr, RespDecode, RespEncode};
use crate::resp::split_r_n;

#[derive(Debug, Ord, PartialOrd, Eq, PartialEq, Clone)]
pub struct RespBulkString(Vec<u8>);

impl Deref for RespBulkString {
    type Target = Vec<u8>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl RespBulkString {
    pub fn new(s: impl AsRef<[u8]>) -> Self {
        Self(s.as_ref().to_vec())
    }
}

// $5\r\nhello\r\n
// TODO: Null bulk strings: $-1\r\n
impl RespEncode for RespBulkString {
    fn encode(self) -> Result<Vec<u8>, EncodeErr> {
        let len = self.len();
        let mut ret = Vec::with_capacity(len + 16);
        ret.extend_from_slice(&format!("${}\r\n", len).into_bytes());
        ret.extend_from_slice(&self);
        ret.extend_from_slice(b"\r\n");
        Ok(ret)
    }
}

impl RespDecode for RespBulkString {
    fn decode(buf: &impl AsRef<[u8]>) -> anyhow::Result<Decoded<Self>, DecodeErr> {
        let (pre, rest) = split_r_n(buf)?;
        let mut total_length = pre.len() + 2;
        let len = pre.split_at(1).1.parse::<usize>()?;
        let (pre, _) = split_r_n(&rest)?;
        if pre.len() != len {
            return Err(InvalidLength(format!(
                "expected {}, but got {}",
                len,
                pre.len()
            )));
        }
        total_length += pre.len() + 2;
        let rbs = RespBulkString::new(pre);
        Ok(Decoded(Some(rbs), total_length))
    }
}
