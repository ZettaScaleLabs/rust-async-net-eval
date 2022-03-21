use std::env;
use std::io::{Read, Write};
use std::net::SocketAddr;
use std::net::TcpListener;
use std::thread;

fn run(addr: SocketAddr, size: usize) -> Result<(), Box<dyn std::error::Error>> {
    let listener = TcpListener::bind(addr)?;
    let incoming = listener.incoming();

    for stream in incoming {
        let mut stream = stream.unwrap();
        stream.set_nodelay(true)?;
        thread::spawn(move || {
            let mut buf = vec![0u8; size];
            loop {
                stream.read_exact(&mut buf).unwrap();
                stream.write_all(&buf).unwrap();
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

    run(addr, size).unwrap();
}
