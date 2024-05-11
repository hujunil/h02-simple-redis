use bytes::BytesMut;

use crate::{extract_simple_frame_data, RespDecode, RespEncode, RespError};

use super::CRLF_LEN;

impl RespEncode for f64 {
    fn encode(self) -> Vec<u8> {
        format!(",{}\r\n", self).into_bytes()
    }
}

impl RespDecode for f64 {
    const PREFIX: &'static str = ",";
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

    use super::*;
    use anyhow::Result;

    #[test]
    fn test_double_encode() {
        let i = 123f64;
        let frame = f64::encode(i);
        assert_eq!(frame, b",123\r\n");
    }

    #[test]
    fn test_double_decode() -> Result<()> {
        let mut buf = BytesMut::from(",123.123\r\n");
        let number = f64::decode(&mut buf)?;
        assert_eq!(number, 123.123);

        let mut buf = BytesMut::from(",-123.123\r\n");
        let number = f64::decode(&mut buf)?;
        assert_eq!(number, -123.123);

        Ok(())
    }

    #[test]
    fn test_double_decode2() {
        let mut buf = BytesMut::from(",1.23456e-9\r\n");
        let number = f64::decode(&mut buf).unwrap();
        assert_eq!(number, 1.23456e-9);
    }
}
