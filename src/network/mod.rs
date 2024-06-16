use anyhow::bail;
use futures::SinkExt;
use tokio::net::TcpStream;
use tokio_stream::StreamExt;
use tokio_util::codec::Framed;
use tracing::info;

use crate::backend::Backend;
use crate::cmd::hmap::{HGetAllCommand, HGetCommand, HSetCommand};
use crate::cmd::map::{GetCommand, SetCommand};
use crate::cmd::CommandExecutor;
use crate::network::codec::RespCodec;
use crate::resp::frame::RespFrame;
use crate::resp::simple_string::RespSimpleString;

mod codec;

#[derive(Debug)]
pub struct RedisRequest {
    frame: RespFrame,
    backend: Backend,
}

#[derive(Debug)]
pub struct RedisResponse {
    frame: RespFrame,
}

pub async fn stream_handler(stream: TcpStream, backend: Backend) -> anyhow::Result<()> {
    let mut resp = Framed::new(stream, RespCodec);
    loop {
        match resp.next().await {
            Some(Ok(frame)) => {
                info!("Received frame: {:?}", frame);
                let req = RedisRequest {
                    frame,
                    backend: backend.clone(),
                };
                let response = request_handler(req).await?;
                info!("Sending response: {:?}", response);
                resp.send(response.frame).await?;
            }
            Some(Err(e)) => {
                bail!(e.to_string());
            }
            None => continue,
        }
    }
}

async fn request_handler(req: RedisRequest) -> anyhow::Result<RedisResponse> {
    let (frame, backend) = (req.frame, req.backend);

    let RespFrame::Array(cmd) = frame else {
        bail!("Invalid command format.")
    };

    let RespFrame::BulkString(s) = &cmd[0] else {
        bail!("Invalid command format.")
    };

    let response = match s.to_ascii_lowercase().as_slice() {
        b"get" => {
            info!("get command");
            let get = GetCommand::try_from(cmd)?;
            get.execute(backend)?
        }
        b"set" => {
            info!("set command");
            let set = SetCommand::try_from(cmd)?;
            set.execute(backend)?
        }
        b"hget" => {
            info!("hget command");
            let hget = HGetCommand::try_from(cmd)?;
            hget.execute(backend)?
        }
        b"hset" => {
            info!("hset command");
            let hset = HSetCommand::try_from(cmd)?;
            hset.execute(backend)?
        }
        b"hgetall" => {
            info!("hgetall command");
            let hgetall = HGetAllCommand::try_from(cmd)?;
            hgetall.execute(backend)?
        }
        _ => {
            let s = format!("unimplemented command: {}", String::from_utf8(s.to_vec())?);
            info!(s);
            RespSimpleString::new(s).into()
        }
    };

    Ok(RedisResponse { frame: response })
}
