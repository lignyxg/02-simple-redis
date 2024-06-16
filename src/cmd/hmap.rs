use crate::backend::Backend;
use crate::cmd::ExecuteError::{InvalidArgument, InvalidCommand};
use crate::cmd::{into_args_iter, CommandExecutor, ExecuteError, RET_OK};
use crate::resp::array::RespArray;
use crate::resp::frame::RespFrame;
use crate::resp::null::RespNull;

#[derive(Debug)]
pub struct HGetCommand {
    field: String,
    key: String,
}

#[derive(Debug)]
pub struct HSetCommand {
    field: String,
    key: String,
    value: RespFrame,
}

#[derive(Debug)]
pub struct HGetAllCommand {
    field: String,
}

impl CommandExecutor for HGetCommand {
    fn execute(self, backend: Backend) -> anyhow::Result<RespFrame> {
        match backend.hget(&self.field, &self.key) {
            Some(value) => Ok(value),
            None => Ok(RespNull.into()),
        }
    }
}

impl CommandExecutor for HSetCommand {
    fn execute(self, mut backend: Backend) -> anyhow::Result<RespFrame> {
        backend.hset(&self.field, &self.key, self.value);
        Ok(RET_OK.clone())
    }
}

impl CommandExecutor for HGetAllCommand {
    fn execute(self, backend: Backend) -> anyhow::Result<RespFrame> {
        match backend.hgetall(&self.field) {
            Some(v) => {
                let resp_arr = RespArray::try_from(v)?.into();
                Ok(resp_arr)
            }
            None => Ok(RespNull.into()),
        }
    }
}

// HGet: "*3\r\n$4\r\nhget\r\n$3\r\nmap\r\n$5\r\nhello\r\n"
// HSet: "*4\r\n$4\r\nhset\r\n$3\r\nmap\r\n$5\r\nhello\r\n$5\r\nworld\r\n"
// HGetAll: "*2\r\n$7\r\nhgetall\r\n$3\r\nmap\r\n"

impl TryFrom<RespArray> for HGetCommand {
    type Error = ExecuteError;

    fn try_from(arr: RespArray) -> Result<Self, Self::Error> {
        let len = arr.len();
        if len != 3 {
            return Err(InvalidArgument(format!("expected 2, got {}", len - 1)));
        }
        let mut args = into_args_iter(arr, 1);
        match (args.next(), args.next()) {
            (Some(RespFrame::BulkString(field)), Some(RespFrame::BulkString(key))) => {
                let field = String::from_utf8(field.to_vec())?;
                let key = String::from_utf8(key.to_vec())?;
                Ok(HGetCommand { field, key })
            }
            _ => Err(InvalidCommand(
                "hget field and key should be bulkstring".to_string(),
            )),
        }
    }
}

impl TryFrom<RespArray> for HSetCommand {
    type Error = ExecuteError;

    fn try_from(arr: RespArray) -> Result<Self, Self::Error> {
        let len = arr.len();
        if len != 4 {
            return Err(InvalidArgument(format!("expected 2, got {}", len - 1)));
        }
        let mut args = into_args_iter(arr, 1);
        match (args.next(), args.next(), args.next()) {
            (Some(RespFrame::BulkString(field)), Some(RespFrame::BulkString(key)), Some(frame)) => {
                let field = String::from_utf8(field.to_vec())?;
                let key = String::from_utf8(key.to_vec())?;
                Ok(HSetCommand {
                    field,
                    key,
                    value: frame,
                })
            }
            _ => Err(InvalidCommand(
                "hset field and key should be bulkstring".to_string(),
            )),
        }
    }
}

impl TryFrom<RespArray> for HGetAllCommand {
    type Error = ExecuteError;

    fn try_from(arr: RespArray) -> Result<Self, Self::Error> {
        if arr.len() != 2 {
            return Err(InvalidArgument(format!(
                "expected 1, got {}",
                arr.len() - 1
            )));
        }
        match &arr[1] {
            RespFrame::BulkString(field) => {
                let field = String::from_utf8(field.to_vec())?;
                Ok(HGetAllCommand { field })
            }
            _ => Err(InvalidCommand("get key should be a bulkstring".to_string())),
        }
    }
}
