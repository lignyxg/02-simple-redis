use crate::backend::Backend;
use crate::cmd::ExecuteError::{InvalidArgument, InvalidCommand};
use crate::cmd::{into_args_iter, CommandExecutor, ExecuteError, RET_OK};
use crate::resp::array::RespArray;
use crate::resp::frame::RespFrame;
use crate::resp::null::RespNull;

#[derive(Debug)]
pub struct HGetCommand {
    key: String,
    field: String,
}

#[derive(Debug)]
pub struct HSetCommand {
    key: String,
    field: String,
    value: RespFrame,
}

#[derive(Debug)]
pub struct HGetAllCommand {
    key: String,
}

#[derive(Debug, PartialEq)]
pub struct HmgetCommand {
    key: String,
    fields: Vec<String>,
}

impl CommandExecutor for HGetCommand {
    fn execute(self, backend: Backend) -> anyhow::Result<RespFrame> {
        match backend.hget(&self.key, &self.field) {
            Some(value) => Ok(value),
            None => Ok(RespNull.into()),
        }
    }
}

impl CommandExecutor for HSetCommand {
    fn execute(self, mut backend: Backend) -> anyhow::Result<RespFrame> {
        backend.hset(&self.key, &self.field, self.value);
        Ok(RET_OK.clone())
    }
}

impl CommandExecutor for HGetAllCommand {
    fn execute(self, backend: Backend) -> anyhow::Result<RespFrame> {
        match backend.hgetall(&self.key) {
            Some(v) => {
                let resp_arr = RespArray::try_from(v)?.into();
                Ok(resp_arr)
            }
            None => Ok(RespNull.into()),
        }
    }
}

impl CommandExecutor for HmgetCommand {
    fn execute(self, backend: Backend) -> anyhow::Result<RespFrame> {
        let vec = backend.hmget(&self.key, &self.fields);
        Ok(RespArray::new(vec).into())
    }
}

// HGet: "*3\r\n$4\r\nhget\r\n$3\r\nmap\r\n$5\r\nhello\r\n"
// HSet: "*4\r\n$4\r\nhset\r\n$3\r\nmap\r\n$5\r\nhello\r\n$5\r\nworld\r\n"
// HGetAll: "*2\r\n$7\r\nhgetall\r\n$3\r\nmap\r\n"

impl TryFrom<RespArray> for HGetCommand {
    type Error = ExecuteError;

    fn try_from(arr: RespArray) -> Result<Self, Self::Error> {
        let Some(arr) = arr.0 else {
            return Err(InvalidCommand("command exists".to_string()));
        };
        let len = arr.len();
        if len != 3 {
            return Err(InvalidArgument(format!("expected 2, got {}", len - 1)));
        }
        let mut args = into_args_iter(arr, 1);
        match (args.next(), args.next()) {
            (Some(RespFrame::BulkString(field)), Some(RespFrame::BulkString(key))) => {
                let field =
                    String::from_utf8(field.as_ref().expect("field has to exist").to_vec())?;
                let key = String::from_utf8(key.as_ref().expect("key has to exist").to_vec())?;
                Ok(HGetCommand {
                    key: field,
                    field: key,
                })
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
        let Some(arr) = arr.0 else {
            return Err(InvalidCommand("command exists".to_string()));
        };

        let len = arr.len();
        if len != 4 {
            return Err(InvalidArgument(format!("expected 3, got {}", len - 1)));
        }
        let mut args = into_args_iter(arr, 1);
        match (args.next(), args.next(), args.next()) {
            (Some(RespFrame::BulkString(field)), Some(RespFrame::BulkString(key)), Some(frame)) => {
                let field =
                    String::from_utf8(field.as_ref().expect("field has to exist").to_vec())?;
                let key = String::from_utf8(key.as_ref().expect("key has to exist").to_vec())?;
                Ok(HSetCommand {
                    key: field,
                    field: key,
                    value: frame.clone(),
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
                Ok(HGetAllCommand { key })
            }
            _ => Err(InvalidCommand("get key should be a bulkstring".to_string())),
        }
    }
}

impl TryFrom<RespArray> for HmgetCommand {
    type Error = ExecuteError;

    fn try_from(arr: RespArray) -> Result<Self, Self::Error> {
        let Some(arr) = arr.0 else {
            return Err(InvalidCommand("command exists".to_string()));
        };
        if arr.len() < 3 {
            return Err(InvalidArgument(format!(
                "expected at least 2, got {}",
                arr.len() - 1
            )));
        }
        let mut key = String::default();
        let mut fields = Vec::with_capacity(arr.len());
        for (i, elem) in arr[1..].iter().enumerate() {
            match elem {
                RespFrame::BulkString(s) => {
                    let s = String::from_utf8(s.as_ref().expect("it has to exist").to_vec())?;
                    if i == 0 {
                        key = s;
                    } else {
                        fields.push(s);
                    }
                }
                _ => {
                    return Err(InvalidCommand(
                        "key and fields should be bulkstring".to_string(),
                    ))
                }
            }
        }

        Ok(HmgetCommand { key, fields })
    }
}

#[cfg(test)]
mod tests {
    use crate::resp::bulkstring::RespBulkString;

    use super::*;

    #[test]
    fn test_hmget_try_from() -> anyhow::Result<()> {
        let arr = RespArray::new(vec![
            RespBulkString::new("hmget").into(),
            RespBulkString::new("k").into(),
            RespBulkString::new("f1").into(),
            RespBulkString::new("f2").into(),
        ]);
        let hmget = HmgetCommand {
            key: "k".to_string(),
            fields: vec!["f1".to_string(), "f2".to_string()],
        };

        let transformed = HmgetCommand::try_from(arr)?;
        assert_eq!(hmget, transformed);
        Ok(())
    }
}
