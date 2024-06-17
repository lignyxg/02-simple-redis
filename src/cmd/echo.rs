use crate::backend::Backend;
use crate::cmd::ExecuteError::InvalidCommand;
use crate::cmd::{CommandExecutor, ExecuteError};
use crate::resp::array::RespArray;
use crate::resp::frame::RespFrame;

#[derive(Debug)]
pub struct ECHOCommand {
    mirror: RespFrame,
}

impl CommandExecutor for ECHOCommand {
    fn execute(self, _backend: Backend) -> anyhow::Result<RespFrame> {
        Ok(self.mirror)
    }
}

impl TryFrom<RespArray> for ECHOCommand {
    type Error = ExecuteError;

    fn try_from(value: RespArray) -> Result<Self, Self::Error> {
        let Some(arr) = value.0 else {
            return Err(InvalidCommand("command exists".to_string()));
        };

        Ok(ECHOCommand {
            mirror: arr[1].clone(),
        })
    }
}
