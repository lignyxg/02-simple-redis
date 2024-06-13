use std::ops::Deref;

use crate::resp::frame::{DecodeErr, Decoded, EncodeErr, RespDecode, RespEncode};
use crate::resp::split_r_n;

#[derive(Debug, Ord, PartialOrd, Eq, PartialEq)]
pub struct RespSimpleError(String);

impl Deref for RespSimpleError {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl RespSimpleError {
    pub fn new(s: impl Into<String>) -> Self {
        Self(s.into())
    }
}

impl RespEncode for RespSimpleError {
    fn encode(self) -> Result<Vec<u8>, EncodeErr> {
        Ok(format!("-{}\r\n", self.0).into_bytes())
    }
}

impl RespDecode for RespSimpleError {
    fn decode(buf: &impl AsRef<[u8]>) -> anyhow::Result<Decoded<Self>, DecodeErr> {
        let (pre, _) = split_r_n(buf)?;
        let rse = RespSimpleError::new(&pre[1..]);
        Ok(Decoded(Some(rse), pre.len() + 2))
    }
}
