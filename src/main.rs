use tokio::net::TcpListener;
use tracing::{info, warn};

use simple_redis::backend::Backend;
use simple_redis::network::stream_handler;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let addr = "0.0.0.0:6379";
    info!("Simple-Redis-Server is listening on {}", addr);
    let listener = TcpListener::bind(addr).await?;

    let backend = Backend::default();
    loop {
        let (stream, addr) = listener.accept().await?;
        info!("Accepted connection from {}", addr);
        let backend_cloned = backend.clone();

        tokio::spawn(async move {
            match stream_handler(stream, backend_cloned).await {
                Ok(_) => {
                    info!("connection {} exited.", addr);
                }
                Err(e) => {
                    warn!("handle error for {}: {:?}", addr, e);
                }
            }
        });
    }

    // Ok(())
}
