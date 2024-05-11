use bytes::BytesMut;

use crate::{RespDecode, RespEncode, RespError};

use super::{extract_simple_frame_data, CRLF_LEN};

impl RespEncode for i64 {
    fn encode(self) -> Vec<u8> {
        // let sign = if *self < 0 { "" } else { "+" };
        format!(":{}\r\n", self).into_bytes()
    }
}

impl RespDecode for i64 {
    const PREFIX: &'static str = ":";

    fn decode(buf: &mut BytesMut) -> Result<Self, RespError> {
        let end = extract_simple_frame_data(buf, Self::PREFIX)?;
        let s = String::from_utf8_lossy(&buf[Self::PREFIX.len()..end]);
        Ok(s.parse()?)
    }

    fn expect_length(buf: &[u8]) -> Result<usize, RespError> {
        let end = extract_simple_frame_data(buf, Self::PREFIX)?;
        Ok(end + CRLF_LEN)
    }
}

#[cfg(test)]
mod tests {
    use crate::RespFrame;

    use super::*;
    use anyhow::Result;

    #[test]
    fn test_integer_encode() {
        let i = 123;
        let frame = RespFrame::Integer(i);
        assert_eq!(frame.encode(), b":123\r\n");
    }

    #[test]
    fn test_integer_decode() -> Result<()> {
        let mut buf = BytesMut::from(":123\r\n");
        let number = i64::decode(&mut buf)?;
        assert_eq!(number, 123);

        let mut buf = BytesMut::from(":-123\r\n");
        let number = i64::decode(&mut buf)?;
        assert_eq!(number, -123);

        Ok(())
    }
}
