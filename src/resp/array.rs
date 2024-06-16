use std::ops::Deref;

use dashmap::DashMap;

use crate::resp::bulkstring::RespBulkString;
use crate::resp::frame::{DecodeErr, Decoded, EncodeErr, RespDecode, RespEncode, RespFrame};
use crate::resp::split_r_n;

#[derive(Debug, Ord, PartialOrd, Eq, PartialEq, Clone)]
pub struct RespArray(pub(crate) Vec<RespFrame>);

impl Deref for RespArray {
    type Target = Vec<RespFrame>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

// *<number-of-elements>\r\n<element-1>...<element-n>
// *2\r\n $5\r\nhello\r\n $5\r\nworld\r\n
// TODO: Null array *-1\r\n
impl RespEncode for RespArray {
    fn encode(self) -> Result<Vec<u8>, EncodeErr> {
        let n_elements = self.len();
        let mut ret = Vec::with_capacity(4096);
        ret.extend_from_slice(&format!("*{}\r\n", n_elements).into_bytes());
        for elem in self.0 {
            let encoded = elem.encode()?;
            ret.extend_from_slice(&encoded);
        }
        Ok(ret)
    }
}

impl RespDecode for RespArray {
    fn decode(buf: &impl AsRef<[u8]>) -> anyhow::Result<Decoded<Self>, DecodeErr> {
        let (pre, rest) = split_r_n(buf)?;
        let n_elem = pre[1..].parse::<usize>()?; // num of elements in array
        let mut ret = Vec::new();
        let mut total_length = pre.len() + 2;

        let mut remainder = rest;
        for _ in 0..n_elem {
            let decoded = RespFrame::decode(&remainder)?;
            ret.push(decoded.0.unwrap());
            total_length += decoded.1;
            remainder = remainder.split_off(decoded.1);
        }

        Ok(Decoded(Some(RespArray::new(ret)), total_length))
    }
}

impl RespArray {
    pub fn new(arr: Vec<RespFrame>) -> Self {
        Self(arr)
    }
}

impl TryFrom<DashMap<String, RespFrame>> for RespArray {
    type Error = anyhow::Error;

    fn try_from(value: DashMap<String, RespFrame>) -> Result<Self, Self::Error> {
        let mut vec: Vec<RespFrame> = Vec::with_capacity(value.len() * 2);

        for (k, v) in value {
            let k = RespBulkString::new(k).into();
            vec.push(k);
            vec.push(v);
        }
        Ok(RespArray::new(vec))
    }
}
