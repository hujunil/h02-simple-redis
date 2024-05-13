use bytes::BytesMut;

use crate::{extract_simple_frame_data, RespDecode, RespEncode, RespError};

use super::CRLF_LEN;

#[derive(Debug, Clone, PartialOrd)]
pub struct Double(pub(crate) f64);

impl PartialEq for Double {
    fn eq(&self, other: &Self) -> bool {
        self.0.to_bits() == other.0.to_bits()
    }
}

#[allow(clippy::derive_ord_xor_partial_ord)]
impl Ord for Double {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.0.partial_cmp(&other.0).unwrap_or_else(|| {
            if self.0.is_nan() && other.0.is_nan() {
                std::cmp::Ordering::Equal
            } else if self.0.is_nan() {
                std::cmp::Ordering::Greater
            } else {
                std::cmp::Ordering::Less
            }
        })
    }
}

impl Eq for Double {}

impl RespEncode for Double {
    fn encode(self) -> Vec<u8> {
        format!(",{}\r\n", self.0).into_bytes()
    }
}

impl From<f64> for Double {
    fn from(f: f64) -> Self {
        Double(f)
    }
}

impl RespDecode for Double {
    const PREFIX: &'static str = ",";
    fn decode(buf: &mut BytesMut) -> Result<Self, RespError> {
        let end = extract_simple_frame_data(buf, Self::PREFIX)?;
        let s = String::from_utf8_lossy(&buf[Self::PREFIX.len()..end]);
        Ok(s.parse::<f64>()?.into())
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
        let frame = Double::encode(i.into());
        assert_eq!(frame, b",123\r\n");
    }

    #[test]
    fn test_double_decode() -> Result<()> {
        let mut buf = BytesMut::from(",123.123\r\n");
        let number = Double::decode(&mut buf)?;
        assert_eq!(number, Double(123.123));

        let mut buf = BytesMut::from(",-123.123\r\n");
        let number = Double::decode(&mut buf)?;
        assert_eq!(number, Double(-123.123));

        Ok(())
    }

    #[test]
    fn test_double_decode2() {
        let mut buf = BytesMut::from(",1.23456e-9\r\n");
        let number = Double::decode(&mut buf).unwrap();
        assert_eq!(number, Double(1.23456e-9f64));
    }
}
