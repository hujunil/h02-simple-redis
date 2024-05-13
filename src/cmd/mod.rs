use enum_dispatch::enum_dispatch;
use lazy_static::lazy_static;
use thiserror::Error;

use crate::{Backend, RespArray, RespError, RespFrame, SimpleString};

mod echo;
mod hmap;
mod map;
mod set;

lazy_static! {
    static ref RESP_OK: RespFrame = SimpleString::new("OK").into();
}

#[enum_dispatch]
pub(crate) trait CommandExecutor {
    fn execute(self, backend: &Backend) -> RespFrame;
}

#[derive(Error, Debug)]
pub enum CommandError {
    #[error("Invalid command: {0}")]
    InvalidCommand(String),

    #[error("Invalid argument: {0}")]
    InvalidArgument(String),

    #[error("{0}")]
    RespError(#[from] RespError),

    #[error("Utf8 error: {0}")]
    Utf8Error(#[from] std::string::FromUtf8Error),
}

#[enum_dispatch(CommandExecutor)]
#[derive(Debug)]
pub enum Command {
    Get(Get),
    Set(Set),
    HGet(HGet),
    HSet(HSet),
    HGetAll(HGetAll),
    HMGet(HMGet),
    Echo(Echo),
    SAdd(SAdd),
    SIsMember(SIsMember),
    Unrecognized(Unrecognized),
}

#[derive(Debug)]
pub(crate) struct SAdd {
    key: String,
    members: Vec<RespFrame>,
}

#[derive(Debug)]
pub(crate) struct SIsMember {
    key: String,
    member: RespFrame,
}

#[derive(Debug)]
pub(crate) struct Echo {
    message: String,
}

#[derive(Debug)]
pub(crate) struct HMGet {
    key: String,
    fields: Vec<String>,
}

#[derive(Debug)]
pub(crate) struct Get {
    key: String,
}

#[derive(Debug)]
pub(crate) struct Set {
    key: String,
    value: RespFrame,
}

#[derive(Debug)]
pub(crate) struct HGet {
    key: String,
    field: String,
}

#[derive(Debug)]
pub(crate) struct HSet {
    key: String,
    field: String,
    value: RespFrame,
}

#[derive(Debug)]
pub(crate) struct HGetAll {
    key: String,
    sort: bool,
}

#[derive(Debug)]
pub(crate) struct Unrecognized;

impl CommandExecutor for Unrecognized {
    fn execute(self, _: &Backend) -> RespFrame {
        RESP_OK.clone()
    }
}

impl TryFrom<RespFrame> for Command {
    type Error = CommandError;

    fn try_from(value: RespFrame) -> Result<Self, Self::Error> {
        match value {
            RespFrame::Array(array) => array.try_into(),
            _ => Err(CommandError::InvalidCommand(
                "Command must be an array".to_string(),
            )),
        }
    }
}

impl TryFrom<RespArray> for Command {
    type Error = CommandError;

    fn try_from(value: RespArray) -> Result<Self, Self::Error> {
        match value.first() {
            Some(RespFrame::BulkString(s)) => match s.as_ref() {
                b"get" => Get::try_from(value).map(Command::Get),
                b"set" => Set::try_from(value).map(Command::Set),
                b"hget" => HGet::try_from(value).map(Command::HGet),
                b"hset" => HSet::try_from(value).map(Command::HSet),
                b"hgetall" => HGetAll::try_from(value).map(Command::HGetAll),
                b"echo" => Echo::try_from(value).map(Command::Echo),
                b"hmget" => HMGet::try_from(value).map(Command::HMGet),
                b"sadd" => SAdd::try_from(value).map(Command::SAdd),
                b"sismember" => SIsMember::try_from(value).map(Command::SIsMember),
                _ => Ok(Command::Unrecognized(Unrecognized)),
            },
            _ => Err(CommandError::InvalidCommand(
                "Command must have a BulkString as the first argument".to_string(),
            )),
        }
    }
}

fn validate_command_for_hmget(
    value: &RespArray,
    names: &[&'static str],
    n_args: usize,
) -> Result<(), CommandError> {
    if value.len() < n_args + names.len() {
        return Err(CommandError::InvalidArgument(format!(
            "{} command must have at least {} argument",
            names.join(" "),
            n_args
        )));
    }
    for (i, name) in names.iter().enumerate() {
        match value[i] {
            RespFrame::BulkString(ref cmd) => {
                if cmd.as_ref().to_ascii_lowercase() != name.as_bytes() {
                    return Err(CommandError::InvalidCommand(format!(
                        "Invalid command: expected {}, got {}",
                        name,
                        String::from_utf8_lossy(cmd.as_ref())
                    )));
                }
            }
            _ => {
                return Err(CommandError::InvalidCommand(
                    "Command must have a BulkString as the first argument".to_string(),
                ));
            }
        }
    }
    Ok(())
}

fn validate_command(
    value: &RespArray,
    names: &[&'static str],
    n_args: usize,
) -> Result<(), CommandError> {
    if value.len() != n_args + names.len() {
        return Err(CommandError::InvalidArgument(format!(
            "{} command must have exactly {} argument",
            names.join(" "),
            n_args
        )));
    }
    for (i, name) in names.iter().enumerate() {
        match value[i] {
            RespFrame::BulkString(ref cmd) => {
                if cmd.as_ref().to_ascii_lowercase() != name.as_bytes() {
                    return Err(CommandError::InvalidCommand(format!(
                        "Invalid command: expected {}, got {}",
                        name,
                        String::from_utf8_lossy(cmd.as_ref())
                    )));
                }
            }
            _ => {
                return Err(CommandError::InvalidCommand(
                    "Command must have a BulkString as the first argument".to_string(),
                ));
            }
        }
    }
    Ok(())
}

fn extract_args(value: RespArray, start: usize) -> Result<Vec<RespFrame>, CommandError> {
    Ok(value.0.into_iter().skip(start).collect())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{RespDecode, RespNullBulkString};
    use anyhow::Result;
    use bytes::BytesMut;

    #[test]
    fn test_command() -> Result<()> {
        let mut buf = BytesMut::new();
        buf.extend_from_slice(b"*2\r\n$3\r\nget\r\n$5\r\nhello\r\n");

        let frame = RespArray::decode(&mut buf)?;

        let cmd: Command = frame.try_into()?;

        let backend = Backend::new();

        let ret = cmd.execute(&backend);
        assert_eq!(ret, RespFrame::NullBulkString(RespNullBulkString));

        Ok(())
    }
}
