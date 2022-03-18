use clap::Parser;
use std::collections::HashMap;
use std::io::{Read, Write};
use std::net::SocketAddr;
use std::net::TcpStream;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

#[derive(Parser, Debug)]
struct Args {
    address: SocketAddr,
    size: usize,
    interval: f64,
    #[clap(short, long)]
    wait: bool,
    #[clap(short, long, default_value = "0")]
    spawn: usize,
    #[clap(short, long)]
    csv: bool,
}

fn run_wait(
    address: SocketAddr,
    size: usize,
    interval: f64,
    csv: bool,
    tasks: usize,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut stream = TcpStream::connect(address)?;
    stream.set_nodelay(true)?;
    let mut count: u64 = 0;
    let mut payload = vec![0u8; size];

    loop {
        let count_bytes: [u8; 8] = count.to_le_bytes();
        payload[0..8].copy_from_slice(&count_bytes);
        let now = Instant::now();

        stream.write_all(&payload).unwrap();
        stream.read_exact(&mut payload).unwrap();

        let elapsed = now.elapsed();
        if csv {
            // framework, transport, test, count, rate, payload, tasks, value, unit
            println!(
                "std,tcp,rtt,{},{},{},{},{},ns",
                count,
                interval,
                payload.len(),
                tasks,
                elapsed.as_nanos()
            );
        } else {
            println!("{} bytes: seq={} time={:?}", payload.len(), count, elapsed);
        }
        thread::sleep(Duration::from_secs_f64(interval));
        count = count.wrapping_add(1);
    }
}

fn run(
    address: SocketAddr,
    size: usize,
    interval: f64,
    csv: bool,
    tasks: usize,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut stream = TcpStream::connect(address)?;
    stream.set_nodelay(true)?;
    let pending = Arc::new(Mutex::new(HashMap::<u64, Instant>::new()));

    let mut c_stream = stream.try_clone()?;
    let c_pending = pending.clone();
    thread::spawn(move || {
        let mut payload = vec![0u8; size];
        let mut count_bytes = [0u8; 8];
        loop {
            c_stream.read_exact(&mut payload).unwrap();
            count_bytes.copy_from_slice(&payload[0..8]);
            let count = u64::from_le_bytes(count_bytes);

            let instant = c_pending.lock().unwrap().remove(&count).unwrap();
            if csv {
                // framework, transport, test, count, rate, payload, tasks, value, unit
                println!(
                    "std,tcp,rtt,{},{},{},{},{},ns",
                count,
                interval,
                payload.len(),
                tasks,
                instant.elapsed().as_nanos()
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
        stream.write_all(&payload).unwrap();

        thread::sleep(Duration::from_secs_f64(interval));
        count = count.wrapping_add(1);
    }
}

fn main() {
    let args = Args::parse();

    for _ in 0..args.spawn {
        thread::spawn(move || {
            let mut x: usize = 1;
            loop {
                x = x.wrapping_mul(2);
                thread::sleep(Duration::from_millis(1));
            }
        });
    }

    if !args.wait {
        run(args.address, args.size, args.interval, args.csv, args.spawn).unwrap();
    }
    run_wait(args.address, args.size, args.interval, args.csv, args.spawn).unwrap();
}
