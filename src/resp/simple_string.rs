use std::ops::Deref;

use crate::resp::frame::{DecodeErr, Decoded, EncodeErr, RespDecode, RespEncode};
use crate::resp::split_r_n;

#[derive(Debug, Ord, PartialOrd, Eq, PartialEq, Clone)]
pub struct RespSimpleString(String);

impl Deref for RespSimpleString {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl RespEncode for RespSimpleString {
    fn encode(self) -> Result<Vec<u8>, EncodeErr> {
        Ok(format!("+{}\r\n", self.0).into_bytes())
    }
}

impl RespDecode for RespSimpleString {
    fn decode(buf: &impl AsRef<[u8]>) -> anyhow::Result<Decoded<Self>, DecodeErr> {
        let (pre, _) = split_r_n(buf)?;
        let rss = RespSimpleString::new(&pre[1..]);

        Ok(Decoded(Some(rss), pre.len() + 2))
    }
}

impl RespSimpleString {
    pub fn new(s: impl Into<String>) -> Self {
        Self(s.into())
    }
}
