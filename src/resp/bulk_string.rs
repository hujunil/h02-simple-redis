use bytes::{Buf, BytesMut};

use crate::{RespDecode, RespEncode, RespError};

use super::{parse_length2, CRLF_LEN};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum BulkString {
    String(Vec<u8>),
    Null,
}

impl RespEncode for BulkString {
    fn encode(self) -> Vec<u8> {
        match self {
            BulkString::String(s) => {
                let mut buf = Vec::new();
                buf.extend_from_slice(b"$");
                buf.extend_from_slice(s.len().to_string().as_bytes());
                buf.extend_from_slice(b"\r\n");
                buf.extend_from_slice(&s);
                buf.extend_from_slice(b"\r\n");
                buf
            }
            BulkString::Null => b"$-1\r\n".to_vec(),
        }
    }
}

impl RespDecode for BulkString {
    const PREFIX: &'static str = "$";

    fn decode(buf: &mut BytesMut) -> Result<Self, RespError> {
        let (end, len) = parse_length2(buf, Self::PREFIX)?;
        if len == -1 {
            return Ok(BulkString::new_null());
        }

        let len = len as usize;

        let remained = &buf[end + CRLF_LEN..];
        if remained.len() < len + CRLF_LEN {
            return Err(RespError::NotComplete);
        }

        buf.advance(end + CRLF_LEN);

        let s = buf.split_to(len);
        buf.advance(CRLF_LEN);

        Ok(BulkString::new(s.to_vec()))
    }

    fn expect_length(buf: &[u8]) -> Result<usize, RespError> {
        let (end, len) = parse_length2(buf, Self::PREFIX)?;
        if len == -1 {
            Ok(end + CRLF_LEN)
        } else {
            Ok(end + CRLF_LEN + (len as usize) + CRLF_LEN)
        }
    }
}

impl BulkString {
    pub fn new(s: impl Into<Vec<u8>>) -> Self {
        BulkString::String(s.into())
    }

    pub fn new_null() -> Self {
        BulkString::Null
    }
}

impl AsRef<[u8]> for BulkString {
    fn as_ref(&self) -> &[u8] {
        match self {
            BulkString::String(s) => s.as_ref(),
            BulkString::Null => &[],
        }
    }
}

impl From<Vec<u8>> for BulkString {
    fn from(s: Vec<u8>) -> Self {
        BulkString::String(s)
    }
}

impl From<&str> for BulkString {
    fn from(s: &str) -> Self {
        BulkString::String(s.as_bytes().to_vec())
    }
}

impl From<&[u8]> for BulkString {
    fn from(s: &[u8]) -> Self {
        BulkString::String(s.to_vec())
    }
}

impl<const N: usize> From<&[u8; N]> for BulkString {
    fn from(s: &[u8; N]) -> Self {
        BulkString::String(s.to_vec())
    }
}

impl TryFrom<BulkString> for String {
    type Error = RespError;

    fn try_from(value: BulkString) -> Result<Self, Self::Error> {
        match value {
            BulkString::String(s) => Ok(String::from_utf8(s)?),
            BulkString::Null => Err(RespError::InvalidFrame("Null BulkString".to_string())),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::RespFrame;
    use anyhow::Result;

    use super::*;

    #[test]
    fn test_bulk_string_encode() {
        let frame: RespFrame = BulkString::new(b"hello".to_vec()).into();
        assert_eq!(frame.encode(), b"$5\r\nhello\r\n");
    }

    #[test]
    fn test_null_bulk_string_encode() {
        let frame: RespFrame = BulkString::new_null().into();
        assert_eq!(frame.encode(), b"$-1\r\n");
    }

    #[test]
    fn test_bulk_string_decode() -> Result<()> {
        let mut buf = BytesMut::new();
        buf.extend_from_slice(b"$5\r\nhello\r\n");

        let frame = BulkString::decode(&mut buf)?;
        assert_eq!(frame, BulkString::new(b"hello"));

        buf.extend_from_slice(b"$5\r\nhello");
        let ret = BulkString::decode(&mut buf);
        assert_eq!(ret.unwrap_err(), RespError::NotComplete);

        buf.extend_from_slice(b"\r\n");
        let frame = BulkString::decode(&mut buf)?;
        assert_eq!(frame, BulkString::new(b"hello"));

        Ok(())
    }

    #[test]
    fn test_null_bulk_string_decode() -> Result<()> {
        let mut buf = BytesMut::new();
        buf.extend_from_slice(b"$-1\r\n");

        let frame = BulkString::decode(&mut buf)?;
        assert_eq!(frame, BulkString::Null);

        Ok(())
    }
}
