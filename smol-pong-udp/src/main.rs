use smol::net::UdpSocket;
use std::env;
use std::net::SocketAddr;
use std::sync::Arc;

async fn read_exact(
    socket: &Arc<UdpSocket>,
    buffer: &mut [u8],
) -> Result<SocketAddr, Box<dyn std::error::Error>> {
    let mut read: usize = 0;
    let mut g_addr = "127.0.0.1:8080".parse().unwrap();
    while read < buffer.len() {
        let (n, addr) = socket.recv_from(&mut buffer[read..]).await?;
        g_addr = addr;
        read += n;
    }
    Ok(g_addr)
}

async fn run(addr: SocketAddr, size: usize) -> Result<(), Box<dyn std::error::Error>> {
    let socket = Arc::new(UdpSocket::bind(addr).await?);
    let mut buf = vec![0u8; size];

    loop {
        let addr = read_exact(&socket, &mut buf).await?;
        socket.send_to(&buf, addr).await?;
    }
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

    smol::block_on(async {
        run(addr, size).await.unwrap();
    });
}
