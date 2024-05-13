use std::ops::Deref;

use bytes::Buf;

use crate::{RespDecode, RespEncode, RespError, RespFrame};

use super::{calc_total_length, parse_length2, BUF_CAP, CRLF_LEN};

#[derive(Debug, Clone, PartialEq, PartialOrd, Eq, Ord)]
pub enum RespArray {
    Array(Vec<RespFrame>),
    Null,
}

impl RespEncode for RespArray {
    fn encode(self) -> Vec<u8> {
        match self {
            RespArray::Array(v) => {
                let mut buf = Vec::with_capacity(BUF_CAP);
                buf.extend_from_slice(&format!("*{}\r\n", v.len()).into_bytes());
                for frame in v {
                    buf.extend_from_slice(&frame.encode());
                }
                buf
            }
            RespArray::Null => b"*-1\r\n".to_vec(),
        }
    }
}

impl RespDecode for RespArray {
    const PREFIX: &'static str = "*";

    fn decode(buf: &mut bytes::BytesMut) -> Result<Self, crate::RespError>
    where
        Self: Sized,
    {
        let (end, len) = parse_length2(buf, Self::PREFIX)?;
        if len == -1 {
            return Ok(RespArray::new_null());
        }

        let len = len as usize;

        let total_len = calc_total_length(buf, end, len, Self::PREFIX)?;

        if buf.len() < total_len {
            return Err(RespError::NotComplete);
        }

        buf.advance(end + CRLF_LEN);

        let mut frames = Vec::with_capacity(len);
        for _ in 0..len {
            frames.push(RespFrame::decode(buf)?);
        }

        Ok(RespArray::new(frames))
    }

    fn expect_length(buf: &[u8]) -> Result<usize, crate::RespError> {
        let (end, len) = parse_length2(buf, Self::PREFIX)?;
        if len == -1 {
            return Ok(end + CRLF_LEN);
        }
        let len = len as usize;
        calc_total_length(buf, end, len, Self::PREFIX)
    }
}

impl RespArray {
    pub fn new(s: impl Into<Vec<RespFrame>>) -> Self {
        RespArray::Array(s.into())
    }

    pub fn new_null() -> Self {
        RespArray::Null
    }
}

impl Deref for RespArray {
    type Target = Vec<RespFrame>;

    fn deref(&self) -> &Self::Target {
        match self {
            RespArray::Array(v) => v,
            RespArray::Null => panic!("Null array"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::BulkString;
    use anyhow::Result;
    use bytes::BytesMut;

    #[test]
    fn test_array_encode() {
        let frame: RespFrame = RespArray::new(vec![
            BulkString::new("set".to_string()).into(),
            BulkString::new("hello".to_string()).into(),
            BulkString::new("world".to_string()).into(),
        ])
        .into();
        assert_eq!(
            &frame.encode(),
            b"*3\r\n$3\r\nset\r\n$5\r\nhello\r\n$5\r\nworld\r\n"
        );
    }

    #[test]
    fn test_null_array_encode() {
        let frame: RespFrame = RespArray::new_null().into();
        assert_eq!(frame.encode(), b"*-1\r\n");
    }

    #[test]
    fn test_null_array_decode() -> Result<()> {
        let mut buf = BytesMut::new();
        buf.extend_from_slice(b"*-1\r\n");

        let frame = RespArray::decode(&mut buf)?;
        assert_eq!(frame, RespArray::Null);

        Ok(())
    }

    #[test]
    fn test_array_decode() -> Result<()> {
        let mut buf = BytesMut::new();
        buf.extend_from_slice(b"*2\r\n$3\r\nset\r\n$5\r\nhello\r\n");

        let frame = RespArray::decode(&mut buf)?;
        assert_eq!(frame, RespArray::new([b"set".into(), b"hello".into()]));

        buf.extend_from_slice(b"*2\r\n$3\r\nset\r\n");
        let ret = RespArray::decode(&mut buf);
        assert_eq!(ret.unwrap_err(), RespError::NotComplete);

        buf.extend_from_slice(b"$5\r\nhello\r\n");
        let frame = RespArray::decode(&mut buf)?;
        assert_eq!(frame, RespArray::new([b"set".into(), b"hello".into()]));

        Ok(())
    }
}
