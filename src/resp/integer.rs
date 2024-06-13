use crate::resp::frame::{DecodeErr, Decoded, EncodeErr, RespDecode, RespEncode};
use crate::resp::split_r_n;

// :[<+|->]<value>\r\n
impl RespEncode for i64 {
    fn encode(self) -> Result<Vec<u8>, EncodeErr> {
        let sign = if self > 0 { "+" } else { "" };
        Ok(format!(":{}{}\r\n", sign, self).into_bytes())
    }
}

impl RespDecode for i64 {
    fn decode(buf: &impl AsRef<[u8]>) -> anyhow::Result<Decoded<Self>, DecodeErr> {
        let (pre, _) = split_r_n(buf)?;
        let num = pre[1..].parse::<i64>()?;

        Ok(Decoded(Some(num), pre.len() + 2))
    }
}
