use crate::resp::frame::DecodeErr;
use crate::resp::frame::DecodeErr::{InComplete, InvalidFrame};

mod array;
mod boolean;
mod bulkstring;
mod double;
mod frame;
mod integer;
mod map;
mod null;
mod set;
mod simple_error;
mod simple_string;

pub fn split_r_n<A>(buf: &A) -> anyhow::Result<(String, String), DecodeErr>
where
    A: AsRef<[u8]>,
{
    let buf = String::from_utf8_lossy(buf.as_ref());
    if buf.len() < 3 {
        return Err(InvalidFrame(format!(
            "expect length is 3, got {}",
            buf.len()
        )));
    }
    let Some((pre, rest)) = buf.split_once("\r\n") else {
        return Err(InComplete);
    };
    Ok((pre.to_string(), rest.to_string()))
}
