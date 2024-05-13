use crate::{Backend, BulkString, RespArray, RespFrame};

use super::{extract_args, validate_command, CommandError, CommandExecutor};

#[derive(Debug)]
pub(crate) struct Echo {
    pub(crate) message: String,
}

impl CommandExecutor for Echo {
    fn execute(self, _: &Backend) -> RespFrame {
        RespFrame::BulkString(BulkString::new(self.message))
    }
}

impl TryFrom<RespArray> for Echo {
    type Error = CommandError;

    fn try_from(value: RespArray) -> Result<Self, Self::Error> {
        validate_command(&value, &["echo"], 1)?;

        let mut args = extract_args(value, 1)?.into_iter();

        match args.next() {
            Some(RespFrame::BulkString(key)) => Ok(Echo {
                message: String::try_from(key)?,
            }),
            _ => Err(CommandError::InvalidCommand("Invalid key".to_string())),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{RespArray, RespDecode};

    use super::super::Echo;
    use anyhow::Result;
    use bytes::BytesMut;

    #[test]
    fn test_echo_from_resp_array() -> Result<()> {
        let mut buf = BytesMut::new();
        buf.extend_from_slice(b"*2\r\n$4\r\necho\r\n$5\r\nhello\r\n");

        let frame = RespArray::decode(&mut buf)?;

        let result: Echo = frame.try_into()?;
        assert_eq!(result.message, "hello");

        Ok(())
    }
}
