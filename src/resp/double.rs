use std::ops::Deref;

use typed_floats::tf64::NonNaN;

use crate::resp::frame::{DecodeErr, Decoded, EncodeErr, RespDecode, RespEncode};
use crate::resp::split_r_n;

#[derive(Debug, Ord, PartialOrd, Eq, PartialEq, Clone)]
pub struct RespDouble(NonNaN);

impl Deref for RespDouble {
    type Target = NonNaN;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

// ,[<+|->]<integral>[.<fractional>][<E|e>[sign]<exponent>]\r\n
// ,1.23\r\n
impl RespEncode for RespDouble {
    fn encode(self) -> Result<Vec<u8>, EncodeErr> {
        let ret = if self.abs() < 1e-8 || self.abs() > 1e+8 {
            format!(",{:+e}\r\n", &self.get())
        } else {
            let sign = if self.0 < 0.0 { "" } else { "+" };
            format!(",{}{}\r\n", sign, self.0)
        };
        Ok(ret.into_bytes())
    }
}

impl RespDecode for RespDouble {
    fn decode(buf: &impl AsRef<[u8]>) -> anyhow::Result<Decoded<Self>, DecodeErr> {
        let (pre, _) = split_r_n(buf)?;
        let num = pre[1..].parse::<f64>()?;
        let num = RespDouble::new(num);

        Ok(Decoded(Some(num), pre.len() + 2))
    }
}

impl RespDouble {
    pub fn new(f: f64) -> Self {
        let tf64 = NonNaN::new(f);
        Self(tf64.unwrap())
    }
}
