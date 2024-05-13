use std::ops::Deref;

use bytes::BytesMut;

use crate::{extract_simple_frame_data, resp::CRLF_LEN};

use super::{RespDecode, RespEncode, RespError};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct SimpleString(pub(crate) String);

impl SimpleString {
    pub fn new(s: impl Into<String>) -> Self {
        Self(s.into())
    }
}

// - simple string: "+OK\r\n"
impl RespEncode for SimpleString {
    fn encode(self) -> Vec<u8> {
        format!("+{}\r\n", self.0).into_bytes()
    }
}

impl RespDecode for SimpleString {
    const PREFIX: &'static str = "+";

    fn decode(buf: &mut BytesMut) -> Result<Self, RespError> {
        let end = extract_simple_frame_data(buf, Self::PREFIX)?;
        // split the buffer
        let data = buf.split_to(end + CRLF_LEN);
        // println!("debug SimpleString data: {:?}", data);
        let s = String::from_utf8_lossy(&data[Self::PREFIX.len()..end]);
        Ok(SimpleString::new(s.to_string()))
    }

    fn expect_length(buf: &[u8]) -> Result<usize, RespError> {
        let end = extract_simple_frame_data(buf, Self::PREFIX)?;
        Ok(end + CRLF_LEN)
    }
}

impl From<&str> for SimpleString {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

impl From<String> for SimpleString {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl AsRef<str> for SimpleString {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl Deref for SimpleString {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use bytes::BufMut;

    use crate::resp::frame::RespFrame;
    use anyhow::{Ok, Result};

    use super::*;

    #[test]
    fn test_simple_string_encode() {
        let frame: RespFrame = SimpleString::new("OK").into();
        assert_eq!(frame.encode(), b"+OK\r\n");
    }

    #[test]
    fn test_simple_string_decode() -> Result<()> {
        let mut buf = BytesMut::from("+OK\r\n");
        let frame = SimpleString::decode(&mut buf).unwrap();
        assert_eq!(frame, SimpleString::new("OK"));

        let mut buf = BytesMut::from("+hello\r");
        let ret = SimpleString::decode(&mut buf);
        assert!(ret.is_err());
        assert_eq!(ret.unwrap_err(), RespError::NotComplete);
        buf.put_u8(b'\n');

        let frame = SimpleString::decode(&mut buf)?;
        assert_eq!(frame, SimpleString::new("hello"));
        Ok(())
    }

    #[test]
    fn test_simple_string_expect_length() -> Result<()> {
        let buf = b"+OK\r\n";
        assert_eq!(SimpleString::expect_length(buf)?, 5);

        let buf = b"+hello\r\n";
        assert_eq!(SimpleString::expect_length(buf)?, 8);
        Ok(())
    }
}
