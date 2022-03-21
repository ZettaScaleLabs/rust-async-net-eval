use async_std::net::TcpListener;
use async_std::prelude::*;
use async_std::task;
use std::env;
use std::net::SocketAddr;

async fn run(addr: SocketAddr, size: usize) -> Result<(), Box<dyn std::error::Error>> {
    let listener = TcpListener::bind(addr).await?;
    let mut incoming = listener.incoming();

    while let Some(stream) = incoming.next().await {
        let mut stream = stream.unwrap();
        stream.set_nodelay(true)?;
        task::spawn(async move {
            let mut buf = vec![0u8; size];
            loop {
                stream.read_exact(&mut buf).await.unwrap();
                stream.write_all(&buf).await.unwrap();
            }
        });
    }

    Ok(())
}

fn main() {
    let addr: SocketAddr = env::args()
        .nth(1)
        .unwrap()
        .parse()
        .expect("First argument must be a valid socket address");
    let size: usize = env::args()
        .nth(2)
        .unwrap()
        .parse()
        .expect("Second argument must be the buffer size");

    task::block_on(async {
        run(addr, size).await.unwrap();
    });
}
