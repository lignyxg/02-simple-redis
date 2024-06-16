use crate::resp::frame::{DecodeErr, Decoded, EncodeErr, RespDecode, RespEncode};
use crate::resp::split_r_n;

#[derive(Debug, Ord, PartialOrd, Eq, PartialEq, Clone)]
pub struct RespNull;

impl RespEncode for RespNull {
    fn encode(self) -> Result<Vec<u8>, EncodeErr> {
        Ok(b"_\r\n".to_vec())
    }
}

impl RespDecode for RespNull {
    fn decode(buf: &impl AsRef<[u8]>) -> anyhow::Result<Decoded<Self>, DecodeErr> {
        let (pre, _) = split_r_n(buf)?;
        Ok(Decoded(None, pre.len() + 2))
    }
}
