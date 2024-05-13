use crate::{Backend, RespArray, RespFrame};

use super::{extract_args, validate_command_for_more, CommandError, CommandExecutor};

#[derive(Debug)]
pub(crate) struct SAdd {
    key: String,
    members: Vec<RespFrame>,
}

impl CommandExecutor for SAdd {
    fn execute(self, backend: &Backend) -> RespFrame {
        let mut count = 0;
        for member in &self.members {
            if backend.sadd(self.key.clone(), member.to_owned()) {
                count += 1;
            }
        }
        RespFrame::Integer(count)
    }
}

impl TryFrom<RespArray> for SAdd {
    type Error = CommandError;

    fn try_from(value: RespArray) -> Result<Self, Self::Error> {
        // sadd key member [member ...]
        validate_command_for_more(&value, &["sadd"], 2)?;
        let args = extract_args(value, 1)?.into_iter();
        let mut ret = SAdd {
            key: String::from(""),
            members: vec![],
        };

        for (i, frame) in args.into_iter().enumerate() {
            match frame {
                RespFrame::BulkString(bs) => {
                    if i == 0 {
                        ret.key = String::try_from(bs)?;
                    } else {
                        ret.members.push(bs.into());
                    }
                }
                _ => {
                    return Err(CommandError::InvalidArgument(
                        "Invalid key or member".to_string(),
                    ))
                }
            }
        }
        Ok(ret)
    }
}

#[cfg(test)]
mod tests {
    use crate::RespDecode;

    use super::*;
    use anyhow::Result;
    use bytes::BytesMut;

    #[test]
    fn test_sadd_from_resp_array() -> Result<()> {
        let mut buf = BytesMut::new();
        buf.extend_from_slice(b"*3\r\n$4\r\nsadd\r\n$3\r\nkey\r\n$5\r\nhello\r\n");

        let frame = RespArray::decode(&mut buf)?;

        let result: SAdd = frame.try_into()?;
        assert_eq!(result.key, "key");
        assert_eq!(
            result.members,
            vec![RespFrame::BulkString(crate::BulkString::new(b"hello"))]
        );

        Ok(())
    }
}
