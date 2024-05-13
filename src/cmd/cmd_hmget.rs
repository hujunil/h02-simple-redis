use crate::{Backend, BulkString, RespArray, RespFrame};

use super::{extract_args, validate_command_for_more, CommandError, CommandExecutor};

#[derive(Debug)]
pub(crate) struct HMGet {
    key: String,
    fields: Vec<String>,
}

impl CommandExecutor for HMGet {
    fn execute(self, backend: &Backend) -> RespFrame {
        let hmap = backend.hmap.get(&self.key);
        match hmap {
            Some(hmap) => {
                let mut data = Vec::with_capacity(self.fields.len());

                for field in &self.fields {
                    if let Some(value) = hmap.get(field) {
                        data.push(value.value().clone());
                    } else {
                        data.push(RespFrame::BulkString(BulkString::new_null()));
                    }
                }

                RespArray::new(data).into()
            }
            None => RespArray::new([]).into(),
        }
    }
}

impl TryFrom<RespArray> for HMGet {
    type Error = CommandError;

    fn try_from(value: RespArray) -> Result<Self, Self::Error> {
        // hmget key field [field ...], 至少有两个参数
        validate_command_for_more(&value, &["hmget"], 2)?;

        let args = extract_args(value, 1)?.into_iter();

        let mut ret = HMGet {
            key: String::from(""),
            fields: vec![],
        };

        for (i, frame) in args.into_iter().enumerate() {
            match frame {
                RespFrame::BulkString(field) => {
                    if i == 0 {
                        ret.key = String::try_from(field)?;
                    } else {
                        ret.fields.push(String::try_from(field)?);
                    }
                }
                _ => {
                    return Err(CommandError::InvalidArgument(
                        "Invalid arguments".to_string(),
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
    fn test_hmget_from_resp_array() -> Result<()> {
        let mut buf = BytesMut::new();
        buf.extend_from_slice(b"*4\r\n$5\r\nhmget\r\n$3\r\nmap\r\n$5\r\nhello\r\n$5\r\nworld\r\n");

        let frame = RespArray::decode(&mut buf)?;

        let result: HMGet = frame.try_into()?;
        assert_eq!(result.key, "map");
        assert_eq!(result.fields, vec!["hello", "world"]);

        Ok(())
    }
}
