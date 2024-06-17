use std::string::FromUtf8Error;

use lazy_static::lazy_static;
use thiserror::Error;

use crate::backend::Backend;
use crate::cmd::hmap::{HGetAllCommand, HGetCommand, HSetCommand};
use crate::cmd::map::{GetCommand, SetCommand};
use crate::resp::frame::{DecodeErr, RespFrame};
use crate::resp::simple_string::RespSimpleString;

pub mod hmap;
pub mod map;

lazy_static! {
    static ref RET_OK: RespFrame = RespSimpleString::new("OK").into();
}

pub trait CommandExecutor {
    fn execute(self, backend: Backend) -> anyhow::Result<RespFrame>;
}

#[derive(Debug)]
pub enum Command {
    Get(GetCommand),
    Set(SetCommand),
    HGet(HGetCommand),
    HSet(HSetCommand),
    HGetAll(HGetAllCommand),
}

#[derive(Error, Debug)]
pub enum ExecuteError {
    #[error("Invalid command: {0}")]
    InvalidCommand(String),
    #[error("Invalid argument: {0}")]
    InvalidArgument(String),
    #[error("{0}")]
    DecodeError(#[from] DecodeErr),
    #[error("{0}")]
    FromUtf8Error(#[from] FromUtf8Error),
}

pub fn into_args_iter(val: Vec<RespFrame>, start: usize) -> impl Iterator<Item = RespFrame> {
    val.into_iter().skip(start)
}
