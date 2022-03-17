use std::net::UdpSocket;
use std::sync::{Arc, Mutex};
use std::thread;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::time::{Duration, Instant};
use clap::Parser;

#[derive(Parser, Debug)]
struct Args {
    address: SocketAddr,
    remote: SocketAddr,
    size: usize,
    interval: f64,
    #[clap(short, long)]
    wait: bool,
    #[clap(short, long, default_value = "0")]
    spawn: usize,
    #[clap(short, long)]
    csv: bool,
}


fn read_exact(socket: &Arc<UdpSocket>, buffer: &mut [u8]) -> Result<(), Box<dyn std::error::Error>> {
    let mut read: usize = 0;
    while read < buffer.len() {
        let n = socket.recv(&mut buffer[read..])?;
        read += n;
    }
    Ok(())
}

fn run_wait(
    address: SocketAddr,
    remote: SocketAddr,
    size: usize,
    interval: f64,
    csv: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let socket = Arc::new(UdpSocket::bind(address)?);
    socket.connect(remote)?;

    let mut count: u64 = 0;
    let mut payload = vec![0u8; size];

    loop {
        let count_bytes: [u8; 8] = count.to_le_bytes();
        payload[0..8].copy_from_slice(&count_bytes);
        let now = Instant::now();

        socket.send(&payload).unwrap();
        read_exact(&socket, &mut payload).unwrap();

        let elapsed = now.elapsed();
        if csv {
            // framework, transport, test, count, payload, value, unit
            println!(
                "std,udp,rtt,{},{},{},ns",count, payload.len(), elapsed.as_nanos()
            );
        } else {
            println!(
                "{} bytes: seq={} time={:?}",
                payload.len(),
                count,
                elapsed
            );
        }

        thread::sleep(Duration::from_secs_f64(interval));
        count = count.wrapping_add(1);
    }
}

fn run(
    address: SocketAddr,
    remote: SocketAddr,
    size: usize,
    interval: f64,
    csv: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let socket = Arc::new(UdpSocket::bind(address)?);
    socket.connect(remote)?;

    let pending = Arc::new(Mutex::new(HashMap::<u64, Instant>::new()));

    let c_socket = socket.clone();
    let c_pending = pending.clone();
    thread::spawn(move || {
        let mut payload = vec![0u8; size];
        let mut count_bytes = [0u8; 8];
        loop {
            read_exact(&c_socket, &mut payload).unwrap();
            count_bytes.copy_from_slice(&payload[0..8]);
            let count = u64::from_le_bytes(count_bytes);

            let instant = c_pending.lock().unwrap().remove(&count).unwrap();
            if csv {
                // framework, transport, test, count, payload, value, unit
                println!(
                    "std,udp,rtt,{},{},{},ns",count, payload.len(), instant.elapsed().as_nanos()
                );
            } else {
                println!(
                    "{} bytes: seq={} time={:?}",
                    payload.len(),
                    count,
                    instant.elapsed()
                );
            }
        }
    });

    //Perform RUN tests
    let mut count: u64 = 0;
    loop {
        let mut payload = vec![0u8; size];
        let count_bytes: [u8; 8] = count.to_le_bytes();
        payload[0..8].copy_from_slice(&count_bytes);

        pending.lock().unwrap().insert(count, Instant::now());
        socket.send(&payload).unwrap();

        thread::sleep(Duration::from_secs_f64(interval));
        count = count.wrapping_add(1);
    }
}





fn main() {
    let args = Args::parse();

        for _ in 0..args.spawn {
            thread::spawn(move || {
                let mut x : usize = 1;
                loop {
                    x = x.wrapping_mul(2);
                    thread::sleep(Duration::from_millis(1));
                }
            });
        }

        if !args.wait {
            run(args.address, args.remote, args.size, args.interval, args.csv).unwrap();
        }
        run_wait(args.address, args.remote, args.size, args.interval, args.csv).unwrap();

}
