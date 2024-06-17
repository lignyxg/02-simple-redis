use crate::backend::Backend;
use crate::cmd::ExecuteError::{InvalidArgument, InvalidCommand};
use crate::cmd::{into_args_iter, CommandExecutor, ExecuteError};
use crate::resp::array::RespArray;
use crate::resp::frame::RespFrame;

#[derive(Debug)]
pub struct SaddCommand {
    key: String,
    member: RespFrame,
}

#[derive(Debug)]
pub struct SismemberCommand {
    key: String,
    member: RespFrame,
}

impl CommandExecutor for SaddCommand {
    fn execute(self, backend: Backend) -> anyhow::Result<RespFrame> {
        let ret = backend.sadd(&self.key, self.member);
        Ok(ret.into())
    }
}

impl CommandExecutor for SismemberCommand {
    fn execute(self, backend: Backend) -> anyhow::Result<RespFrame> {
        let ret = backend.sismember(&self.key, &self.member);
        Ok(ret.into())
    }
}

impl TryFrom<RespArray> for SaddCommand {
    type Error = ExecuteError;

    fn try_from(arr: RespArray) -> Result<Self, Self::Error> {
        let Some(arr) = arr.0 else {
            return Err(InvalidCommand("command exists".to_string()));
        };
        if arr.len() != 3 {
            return Err(InvalidArgument(format!(
                "expected 2, got {}",
                arr.len() - 1
            )));
        }
        let mut args = into_args_iter(arr, 1);
        match (args.next(), args.next()) {
            (Some(RespFrame::BulkString(key)), Some(frame)) => {
                let key = String::from_utf8(key.as_ref().expect("key has to exist").to_vec())?;
                Ok(SaddCommand { key, member: frame })
            }
            _ => Err(InvalidCommand(
                "sadd key should be a bulkstring".to_string(),
            )),
        }
    }
}

impl TryFrom<RespArray> for SismemberCommand {
    type Error = ExecuteError;

    fn try_from(arr: RespArray) -> Result<Self, Self::Error> {
        let Some(arr) = arr.0 else {
            return Err(InvalidCommand("command exists".to_string()));
        };
        if arr.len() != 3 {
            return Err(InvalidArgument(format!(
                "expected 2, got {}",
                arr.len() - 1
            )));
        }
        let mut args = into_args_iter(arr, 1);
        match (args.next(), args.next()) {
            (Some(RespFrame::BulkString(key)), Some(frame)) => {
                let key = String::from_utf8(key.as_ref().expect("key has to exist").to_vec())?;
                Ok(SismemberCommand { key, member: frame })
            }
            _ => Err(InvalidCommand("set key should be a bulkstring".to_string())),
        }
    }
}
