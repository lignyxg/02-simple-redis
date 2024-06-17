use crate::backend::Backend;
use crate::cmd::ExecuteError::{InvalidArgument, InvalidCommand};
use crate::cmd::{into_args_iter, CommandExecutor, ExecuteError, RET_OK};
use crate::resp::array::RespArray;
use crate::resp::frame::RespFrame;
use crate::resp::null::RespNull;

// Get: "*2\r\n$3\r\nget\r\n$5\r\nhello\r\n"
#[derive(Debug)]
pub struct GetCommand {
    key: String,
}

// Set: "*3\r\n$3\r\nset\r\n$5\r\nhello\r\n$5\r\nworld\r\n"
#[derive(Debug)]
pub struct SetCommand {
    key: String,
    value: RespFrame,
}

impl CommandExecutor for GetCommand {
    fn execute(self, backend: Backend) -> anyhow::Result<RespFrame> {
        match backend.get(&self.key) {
            None => Ok(RespNull.into()),
            Some(v) => Ok(v),
        }
    }
}

impl CommandExecutor for SetCommand {
    fn execute(self, mut backend: Backend) -> anyhow::Result<RespFrame> {
        backend.set(&self.key, self.value.clone());
        Ok(RET_OK.clone())
    }
}

impl TryFrom<RespArray> for GetCommand {
    type Error = ExecuteError;

    fn try_from(arr: RespArray) -> Result<Self, Self::Error> {
        let Some(arr) = arr.0 else {
            return Err(InvalidCommand("command exists".to_string()));
        };
        if arr.len() != 2 {
            return Err(InvalidArgument(format!(
                "expected 1, got {}",
                arr.len() - 1
            )));
        }
        match &arr[1] {
            RespFrame::BulkString(key) => {
                let key = String::from_utf8(key.as_ref().expect("key has to exist").to_vec())?;
                Ok(GetCommand { key })
            }
            _ => Err(InvalidCommand("get key should be a bulkstring".to_string())),
        }
    }
}

impl TryFrom<RespArray> for SetCommand {
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
                Ok(SetCommand { key, value: frame })
            }
            _ => Err(InvalidCommand("set key should be a bulkstring".to_string())),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::resp::bulkstring::RespBulkString;

    use super::*;

    #[test]
    fn get_try_from() -> anyhow::Result<()> {
        let arr = RespArray::new(vec![
            RespBulkString::new("get").into(),
            RespBulkString::new("hello").into(),
        ]);
        let get = GetCommand::try_from(arr)?;
        assert_eq!("hello", get.key);
        Ok(())
    }

    #[test]
    fn set_try_from() -> anyhow::Result<()> {
        let arr = RespArray::new(vec![
            RespBulkString::new("set").into(),
            RespBulkString::new("hello").into(),
            RespBulkString::new("world").into(),
        ]);
        let set = SetCommand::try_from(arr)?;
        assert_eq!("hello", set.key);
        assert_eq!(
            Into::<RespFrame>::into(RespBulkString::new("world")),
            set.value
        );
        Ok(())
    }
}
