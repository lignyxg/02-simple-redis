use std::collections::BTreeMap;
use std::ops::{Deref, DerefMut};

use crate::resp::frame::{DecodeErr, Decoded, EncodeErr, RespDecode, RespEncode, RespFrame};
use crate::resp::split_r_n;

#[derive(Debug, Ord, PartialOrd, Eq, PartialEq)]
pub struct RespMap(BTreeMap<RespFrame, RespFrame>);

impl Deref for RespMap {
    type Target = BTreeMap<RespFrame, RespFrame>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for RespMap {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

// %<number-of-entries>\r\n<key-1><value-1>...<key-n><value-n>
// %2\r\n
// +first\r\n
// :1\r\n
// +second\r\n
// :2\r\n
impl RespEncode for RespMap {
    fn encode(self) -> Result<Vec<u8>, EncodeErr> {
        let n_elems = self.len();
        let mut ret = Vec::with_capacity(4096);
        ret.extend_from_slice(&format!("%{}\r\n", n_elems).into_bytes());
        for (key, value) in self.0 {
            let en_key = key.encode()?;
            let en_val = value.encode()?;
            ret.extend_from_slice(&en_key);
            ret.extend_from_slice(&en_val);
        }
        Ok(ret)
    }
}

impl RespDecode for RespMap {
    fn decode(buf: &impl AsRef<[u8]>) -> anyhow::Result<Decoded<Self>, DecodeErr> {
        let (pre, rest) = split_r_n(buf)?;
        let n_elem = pre[1..].parse::<usize>()?; // num of elements in map
        let mut map = RespMap::new();
        let mut total_length = pre.len() + 2;

        let mut remainder = rest;
        for _ in 0..n_elem {
            let key = RespFrame::decode(&remainder)?;
            total_length += key.1;
            remainder = remainder.split_off(key.1);

            let value = RespFrame::decode(&remainder)?;
            total_length += value.1;
            remainder = remainder.split_off(value.1);
            map.insert(key.0.unwrap(), value.0.unwrap());
        }

        Ok(Decoded(Some(map), total_length))
    }
}

impl RespMap {
    pub fn new() -> Self {
        Self(BTreeMap::new())
    }
}
