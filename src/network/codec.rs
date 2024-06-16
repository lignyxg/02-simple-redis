use bytes::{Buf, BytesMut};
use tokio_util::codec::{Decoder, Encoder};
use tracing::debug;

use crate::resp::frame::{RespDecode, RespEncode, RespFrame};

#[derive(Debug)]
pub struct RespCodec;

impl Encoder<RespFrame> for RespCodec {
    type Error = anyhow::Error;

    fn encode(&mut self, item: RespFrame, dst: &mut BytesMut) -> Result<(), Self::Error> {
        let encoded = item.encode()?;
        dst.extend_from_slice(&encoded);
        Ok(())
    }
}

impl Decoder for RespCodec {
    type Item = RespFrame;
    type Error = anyhow::Error;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        debug!("decode: {:?}", String::from_utf8_lossy(src));
        let decoded = RespFrame::decode(src)?;
        match decoded.0 {
            Some(frame) => {
                src.advance(decoded.1);
                Ok(Some(frame))
            }
            None => Ok(None),
        }
    }
}
