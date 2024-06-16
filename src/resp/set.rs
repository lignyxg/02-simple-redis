use std::collections::BTreeSet;
use std::ops::{Deref, DerefMut};

use crate::resp::frame::{DecodeErr, Decoded, EncodeErr, RespDecode, RespEncode, RespFrame};
use crate::resp::split_r_n;

#[derive(Debug, Ord, PartialOrd, Eq, PartialEq, Clone)]
pub struct RespSet(BTreeSet<RespFrame>);

impl Deref for RespSet {
    type Target = BTreeSet<RespFrame>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for RespSet {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

// ~<number-of-elements>\r\n<element-1>...<element-n>
impl RespEncode for RespSet {
    fn encode(self) -> Result<Vec<u8>, EncodeErr> {
        let n_elems = self.len();
        let mut ret = Vec::with_capacity(4096);
        ret.extend_from_slice(&format!("~{}\r\n", n_elems).into_bytes());
        for elem in self.0 {
            let encoded = elem.encode()?;
            ret.extend_from_slice(&encoded);
        }
        Ok(ret)
    }
}

impl RespDecode for RespSet {
    fn decode(buf: &impl AsRef<[u8]>) -> anyhow::Result<Decoded<Self>, DecodeErr> {
        let (pre, rest) = split_r_n(buf)?;
        let n_elem = pre[1..].parse::<usize>()?; // num of elements in array
        let mut set = RespSet::new();
        let mut total_length = pre.len() + 2;

        let mut remainder = rest;
        for _ in 0..n_elem {
            let decoded = RespFrame::decode(&remainder)?;
            set.insert(decoded.0.unwrap());
            total_length += decoded.1;
            remainder = remainder.split_off(decoded.1);
        }

        Ok(Decoded(Some(set), total_length))
    }
}

impl RespSet {
    pub fn new() -> Self {
        Self(BTreeSet::new())
    }
}

impl Default for RespSet {
    fn default() -> Self {
        Self::new()
    }
}
