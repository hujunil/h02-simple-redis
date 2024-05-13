use crate::{Backend, RespArray, RespFrame};

use super::{extract_args, validate_command, CommandError, CommandExecutor};

#[derive(Debug)]
pub(crate) struct SIsMember {
    key: String,
    member: RespFrame,
}

impl CommandExecutor for SIsMember {
    fn execute(self, backend: &Backend) -> RespFrame {
        let set = backend.set.get(&self.key);
        match set {
            Some(set) => {
                let ret = set.contains(&self.member);
                RespFrame::Integer(if ret { 1 } else { 0 })
            }
            None => RespFrame::Integer(0),
        }
    }
}

impl TryFrom<RespArray> for SIsMember {
    type Error = CommandError;

    fn try_from(value: RespArray) -> Result<Self, Self::Error> {
        // sismember key member
        validate_command(&value, &["sismember"], 2)?;
        let mut args = extract_args(value, 1)?.into_iter();
        let key = match args.next() {
            Some(RespFrame::BulkString(bs)) => String::try_from(bs)?,
            _ => {
                return Err(CommandError::InvalidArgument(
                    "Invalid key or member".to_string(),
                ))
            }
        };
        let member = args.next().unwrap();
        Ok(SIsMember { key, member })
    }
}

#[cfg(test)]
mod tests {
    use crate::RespDecode;

    use super::*;
    use anyhow::Result;
    use bytes::BytesMut;

    #[test]
    fn test_sismember_from_resp_array() -> Result<()> {
        let mut buf = BytesMut::new();
        buf.extend_from_slice(b"*3\r\n$9\r\nsismember\r\n$3\r\nkey\r\n$5\r\nhello\r\n");

        let frame = RespArray::decode(&mut buf)?;

        println!("debug {:?}", frame);

        let result: SIsMember = frame.try_into()?;
        assert_eq!(result.key, "key");
        assert_eq!(
            result.member,
            RespFrame::BulkString(crate::BulkString::new(b"hello"))
        );

        Ok(())
    }
}
