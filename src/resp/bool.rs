use crate::{RespDecode, RespEncode, RespError};

use super::extract_fixed_data;

impl RespEncode for bool {
    fn encode(self) -> Vec<u8> {
        format!("#{}\r\n", if self { "t" } else { "f" }).into_bytes()
    }
}

impl RespDecode for bool {
    const PREFIX: &'static str = "#";

    fn decode(buf: &mut bytes::BytesMut) -> Result<Self, crate::RespError> {
        match extract_fixed_data(buf, "#t\r\n", "Bool") {
            Ok(_) => Ok(true),
            Err(RespError::NotComplete) => Err(RespError::NotComplete),
            Err(_) => match extract_fixed_data(buf, "#f\r\n", "Bool") {
                Ok(_) => Ok(false),
                Err(e) => Err(e),
            },
        }
    }

    fn expect_length(_buf: &[u8]) -> Result<usize, crate::RespError> {
        Ok(4)
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use anyhow::Result;

    #[test]
    fn test_bool_encode() {
        assert_eq!(bool::encode(true), b"#t\r\n");
        assert_eq!(bool::encode(false), b"#f\r\n");
    }

    #[test]
    fn test_bool_decode() -> Result<()> {
        let mut buf = bytes::BytesMut::from("#t\r\n");
        let b = bool::decode(&mut buf)?;
        assert!(b);

        let mut buf = bytes::BytesMut::from("#f\r\n");
        let b = bool::decode(&mut buf)?;
        assert!(!b);

        Ok(())
    }
}
