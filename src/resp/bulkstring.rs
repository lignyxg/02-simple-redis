use std::ops::Deref;

use crate::resp::frame::DecodeErr::InvalidLength;
use crate::resp::frame::{DecodeErr, Decoded, EncodeErr, RespDecode, RespEncode};
use crate::resp::split_r_n;

#[derive(Debug, Ord, PartialOrd, Eq, PartialEq, Clone)]
pub struct RespBulkString(Option<Vec<u8>>);

impl Deref for RespBulkString {
    type Target = Option<Vec<u8>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl RespBulkString {
    pub fn new(s: impl AsRef<[u8]>) -> Self {
        Self(Some(s.as_ref().to_vec()))
    }

    pub fn null() -> Self {
        Self(None)
    }
}

// $5\r\nhello\r\n
// Null bulk strings: $-1\r\n
impl RespEncode for RespBulkString {
    fn encode(self) -> Result<Vec<u8>, EncodeErr> {
        let Some(bulk_string) = self.0 else {
            return Ok(Vec::from(b"$-1\r\n"));
        };
        let len = bulk_string.len();
        let mut ret = Vec::with_capacity(len + 16);
        ret.extend_from_slice(&format!("${}\r\n", len).into_bytes());
        ret.extend_from_slice(&bulk_string);
        ret.extend_from_slice(b"\r\n");
        Ok(ret)
    }
}

impl RespDecode for RespBulkString {
    fn decode(buf: &impl AsRef<[u8]>) -> anyhow::Result<Decoded<Self>, DecodeErr> {
        let (pre, rest) = split_r_n(buf)?;
        let mut total_length = pre.len() + 2;

        if rest.is_empty() && pre.split_at(1).1.parse::<i64>()? == -1 {
            return Ok(Decoded(Some(RespBulkString::null()), total_length));
        }

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
