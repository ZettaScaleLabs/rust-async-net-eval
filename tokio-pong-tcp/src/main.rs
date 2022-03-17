use std::env;
use std::net::SocketAddr;

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;
use tokio::task;

async fn run(addr: SocketAddr, size: usize) -> Result<(), Box<dyn std::error::Error>> {
    let listener = TcpListener::bind(addr).await?;

    loop {
        let (mut stream, _src) = listener.accept().await?;
        stream.set_nodelay(true)?;
        task::spawn(async move {
            let mut buf = vec![0u8;  size];
            loop {
                stream.read_exact(&mut buf).await.unwrap();
                stream.write_all(&mut buf).await.unwrap();
            }
        });
    }
}

#[tokio::main]
async fn main() {
    let addr: SocketAddr = env::args().nth(1).unwrap().parse()
                    .expect("First argument must be a valid socket address");
    let size: usize = env::args().nth(2).unwrap().parse()
                    .expect("Second argument must be the buffer size");

    run(addr, size).await.unwrap();
}