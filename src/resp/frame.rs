use std::num::{ParseFloatError, ParseIntError};

use enum_dispatch::enum_dispatch;
use thiserror::Error;

use crate::resp::array::RespArray;
use crate::resp::bulkstring::RespBulkString;
use crate::resp::double::RespDouble;
use crate::resp::frame::DecodeErr::InvalidFrameType;
use crate::resp::map::RespMap;
use crate::resp::null::RespNull;
use crate::resp::set::RespSet;
use crate::resp::simple_error::RespSimpleError;
use crate::resp::simple_string::RespSimpleString;

#[enum_dispatch]
pub trait RespEncode {
    fn encode(self) -> Result<Vec<u8>, EncodeErr>;
}

#[derive(Debug)]
pub struct Decoded<T: RespEncode>(pub(crate) Option<T>, pub(crate) usize);

pub trait RespDecode
where
    Self: Sized + RespEncode,
{
    fn decode(buf: &impl AsRef<[u8]>) -> anyhow::Result<Decoded<Self>, DecodeErr>;
}

#[enum_dispatch(RespEncode)]
#[derive(Debug, Ord, PartialOrd, Eq, PartialEq)]
pub enum RespFrame {
    SimpleString(RespSimpleString),
    Error(RespSimpleError),
    Integer(i64),
    BulkString(RespBulkString),
    Null(RespNull),
    Array(RespArray),
    Boolean(bool),
    Double(RespDouble),
    Map(RespMap),
    Set(RespSet),
}

impl RespDecode for RespFrame {
    fn decode(buf: &impl AsRef<[u8]>) -> anyhow::Result<Decoded<Self>, DecodeErr> {
        let mut iter = buf.as_ref().iter().peekable();
        match iter.peek() {
            Some(b'+') => {
                // simple string
                let decoded = RespSimpleString::decode(buf)?;
                let frame = decoded.0.map(|x| x.into());
                Ok(Decoded(frame, decoded.1))
            }
            Some(b'$') => {
                // bulk string
                let decoded = RespBulkString::decode(buf)?;
                let frame = decoded.0.map(|x| x.into());
                Ok(Decoded(frame, decoded.1))
            }
            Some(b'#') => {
                // boolean
                let decoded = bool::decode(buf)?;
                let frame = decoded.0.map(|x| x.into());
                Ok(Decoded(frame, decoded.1))
            }
            Some(b',') => {
                // double
                let decoded = RespDouble::decode(buf)?;
                let frame = decoded.0.map(|x| x.into());
                Ok(Decoded(frame, decoded.1))
            }
            Some(b':') => {
                // integer
                let decoded = i64::decode(buf)?;
                let frame = decoded.0.map(|x| x.into());
                Ok(Decoded(frame, decoded.1))
            }
            Some(b'*') => {
                // array
                let decoded = RespArray::decode(buf)?;
                let frame = decoded.0.map(|x| x.into());
                Ok(Decoded(frame, decoded.1))
            }
            Some(b'%') => {
                // map
                let decoded = RespMap::decode(buf)?;
                let frame = decoded.0.map(|x| x.into());
                Ok(Decoded(frame, decoded.1))
            }
            Some(b'~') => {
                // set
                let decoded = RespSet::decode(buf)?;
                let frame = decoded.0.map(|x| x.into());
                Ok(Decoded(frame, decoded.1))
            }
            Some(b'-') => {
                // simple error
                let decoded = RespSimpleError::decode(buf)?;
                let frame = decoded.0.map(|x| x.into());
                Ok(Decoded(frame, decoded.1))
            }
            _ => Err(InvalidFrameType("not implemented".to_string())),
        }
    }
}

#[derive(Error, Debug)]
pub enum EncodeErr {}

#[derive(Error, Debug, Eq, PartialEq)]
pub enum DecodeErr {
    #[error("Invalid frame:{0}")]
    InvalidFrame(String),
    #[error("Invalid frame type:{0}")]
    InvalidFrameType(String),
    #[error("Pattern is not complete")]
    InComplete,
    #[error("Parse length failed")]
    ParseIntError(#[from] ParseIntError),
    #[error("Parse double failed")]
    ParseFloatError(#[from] ParseFloatError),
    #[error("Invalid content length:{0}")]
    InvalidLength(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_string_encode() -> anyhow::Result<()> {
        let frame: RespFrame = RespSimpleString::new("OK").into();
        assert_eq!(frame.encode()?, b"+OK\r\n");

        Ok(())
    }

    #[test]
    fn test_simple_string_decode() -> anyhow::Result<()> {
        let decoded = RespSimpleString::decode(b"+OK\r\n")?;
        assert_eq!(decoded.0, Some(RespSimpleString::new("OK")));

        let decoded = RespSimpleString::decode(b"+OK\r");
        assert_eq!(decoded.unwrap_err(), DecodeErr::InComplete);
        Ok(())
    }

    #[test]
    fn test_error_encode() -> anyhow::Result<()> {
        let frame: RespFrame = RespSimpleError::new("Error message").into();
        assert_eq!(frame.encode()?, b"-Error message\r\n");

        Ok(())
    }

    #[test]
    fn test_error_decode() -> anyhow::Result<()> {
        let decoded = RespSimpleError::decode(b"-Error message\r\n")?;
        assert_eq!(decoded.0, Some(RespSimpleError::new("Error message")));
        Ok(())
    }

    #[test]
    fn test_integer_encode() -> anyhow::Result<()> {
        let frame: RespFrame = 123.into();
        assert_eq!(frame.encode()?, b":+123\r\n");

        let frame: RespFrame = (-123).into();
        assert_eq!(frame.encode()?, b":-123\r\n");

        Ok(())
    }

    #[test]
    fn test_integer_decode() -> anyhow::Result<()> {
        let decoded = i64::decode(b":+123\r\n")?;
        assert_eq!(decoded.0, Some(123));

        let decoded = i64::decode(b":123\r\n")?;
        assert_eq!(decoded.0, Some(123));

        let decoded = i64::decode(b":-123\r\n")?;
        assert_eq!(decoded.0, Some(-123));

        Ok(())
    }

    #[test]
    fn test_bulkstring_encode() -> anyhow::Result<()> {
        let frame: RespFrame = RespBulkString::new(b"hello").into();
        assert_eq!(frame.encode()?, b"$5\r\nhello\r\n");

        Ok(())
    }

    #[test]
    fn test_bulkstring_decode() -> anyhow::Result<()> {
        let buf = b"$5\r\nhello\r\n";
        let decoded = RespBulkString::decode(buf)?;
        assert_eq!(decoded.0, Some(RespBulkString::new("hello")));
        Ok(())
    }

    #[test]
    fn test_array_encode() -> anyhow::Result<()> {
        let frame: RespFrame = RespArray::new(vec![
            RespBulkString::new("set").into(),
            RespBulkString::new("hello").into(),
            RespSimpleString::new("world").into(),
        ])
        .into();
        assert_eq!(
            frame.encode()?,
            b"*3\r\n$3\r\nset\r\n$5\r\nhello\r\n+world\r\n"
        );
        Ok(())
    }

    #[test]
    fn test_array_decode() -> anyhow::Result<()> {
        let encoded = b"*3\r\n$3\r\nset\r\n$5\r\nhello\r\n+world\r\n";
        let decoded = RespArray::decode(&encoded)?;
        assert_eq!(encoded.len(), decoded.1);
        assert!(decoded.0.is_some());

        let frame = RespArray::new(vec![
            RespBulkString::new("set").into(),
            RespBulkString::new("hello").into(),
            RespSimpleString::new("world").into(),
        ]);
        assert_eq!(decoded.0.unwrap(), frame);
        Ok(())
    }

    #[test]
    fn test_boolean_encode() -> anyhow::Result<()> {
        let frame: RespFrame = true.into();
        assert_eq!(frame.encode()?, b"#t\r\n");

        let frame: RespFrame = false.into();
        assert_eq!(frame.encode()?, b"#f\r\n");
        Ok(())
    }

    #[test]
    fn test_boolean_decode() -> anyhow::Result<()> {
        let decoded = bool::decode(b"#t\r\n")?;
        assert_eq!(decoded.0, Some(true));

        let decoded = bool::decode(b"#f\r\n")?;
        assert_eq!(decoded.0, Some(false));
        Ok(())
    }

    #[test]
    fn test_double_encode() -> anyhow::Result<()> {
        let frame: RespFrame = RespDouble::new(123.456).into();
        assert_eq!(frame.encode()?, b",+123.456\r\n");

        let frame: RespFrame = RespDouble::new(-123.456).into();
        assert_eq!(frame.encode()?, b",-123.456\r\n");

        let frame: RespFrame = RespDouble::new(1.23456e+8).into();
        assert_eq!(frame.encode()?, b",+1.23456e8\r\n");

        let frame: RespFrame = RespDouble::new(-1.23456e-9).into();
        assert_eq!(frame.encode()?, b",-1.23456e-9\r\n");
        Ok(())
    }

    #[test]
    fn test_double_decode() -> anyhow::Result<()> {
        let decoded = RespDouble::decode(b",+123.456\r\n")?;
        assert_eq!(decoded.0, Some(RespDouble::new(123.456)));

        let decoded = RespDouble::decode(b",-123.456\r\n")?;
        assert_eq!(decoded.0, Some(RespDouble::new(-123.456)));

        let decoded = RespDouble::decode(b",+1.23456e8\r\n")?;
        assert_eq!(decoded.0, Some(RespDouble::new(1.23456e+8)));

        let decoded = RespDouble::decode(b",-1.23456e-9\r\n")?;
        assert_eq!(decoded.0, Some(RespDouble::new(-1.23456e-9)));
        Ok(())
    }

    #[test]
    fn test_map_encode() -> anyhow::Result<()> {
        let mut map = RespMap::new();
        map.insert(
            RespSimpleString::new("hello").into(),
            RespSimpleString::new("world").into(),
        );
        map.insert(
            RespBulkString::new(b"foo").into(),
            RespSimpleString::new("bar").into(),
        );
        let frame: RespFrame = map.into();
        assert_eq!(
            frame.encode()?,
            b"%2\r\n+hello\r\n+world\r\n$3\r\nfoo\r\n+bar\r\n"
        );
        Ok(())
    }

    #[test]
    fn test_map_decode() -> anyhow::Result<()> {
        let mut map = RespMap::new();
        map.insert(
            RespSimpleString::new("hello").into(),
            RespSimpleString::new("world").into(),
        );
        map.insert(
            RespBulkString::new(b"foo").into(),
            RespSimpleString::new("bar").into(),
        );
        let frame: RespFrame = map.into();

        let encoded = b"%2\r\n+hello\r\n+world\r\n$3\r\nfoo\r\n+bar\r\n";
        let decoded = RespFrame::decode(&encoded)?;
        assert_eq!(encoded.len(), decoded.1);
        assert!(decoded.0.is_some());
        assert_eq!(decoded.0.unwrap(), frame);
        Ok(())
    }

    #[test]
    fn test_set_encode() -> anyhow::Result<()> {
        let mut set = RespSet::new();
        set.insert(RespBulkString::new(b"hello").into());
        set.insert(RespSimpleString::new("world").into());
        set.insert(RespArray::new(vec![RespSimpleString::new("foo").into()]).into());
        let frame: RespFrame = set.into();
        assert_eq!(
            frame.encode()?,
            b"~3\r\n+world\r\n$5\r\nhello\r\n*1\r\n+foo\r\n"
        );
        Ok(())
    }

    #[test]
    fn test_set_decode() -> anyhow::Result<()> {
        let mut set = RespSet::new();
        set.insert(RespBulkString::new(b"hello").into());
        set.insert(RespSimpleString::new("world").into());
        set.insert(RespArray::new(vec![RespSimpleString::new("foo").into()]).into());

        let encoded = b"~3\r\n+world\r\n$5\r\nhello\r\n*1\r\n+foo\r\n";
        let decoded = RespSet::decode(&encoded)?;
        assert_eq!(encoded.len(), decoded.1);
        assert!(decoded.0.is_some());
        assert_eq!(decoded.0.unwrap(), set);
        Ok(())
    }
}
