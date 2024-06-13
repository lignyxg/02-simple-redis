use crate::resp::frame::{DecodeErr, Decoded, EncodeErr, RespDecode, RespEncode};
use crate::resp::split_r_n;

// Booleans: #<t|f>\r\n
impl RespEncode for bool {
    fn encode(self) -> Result<Vec<u8>, EncodeErr> {
        let t_f = if self { "t" } else { "f" };
        Ok(format!("#{}\r\n", t_f).into_bytes())
    }
}

impl RespDecode for bool {
    fn decode(buf: &impl AsRef<[u8]>) -> anyhow::Result<Decoded<Self>, DecodeErr> {
        let (pre, _) = split_r_n(buf)?;
        let ret = pre[1..].eq("t");

        Ok(Decoded(Some(ret), pre.len() + 2))
    }
}
