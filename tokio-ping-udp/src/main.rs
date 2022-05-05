use clap::Parser;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::net::UdpSocket;
use tokio::runtime::Runtime;
use tokio::sync::Mutex;
use tokio::time;

use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering::*;

const MAX_SAMPLES: usize = 100_000_000;

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
    #[clap(short, long, default_value = "60")]
    duration: u64,
}

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

async fn run_wait(
    address: SocketAddr,
    remote: SocketAddr,
    size: usize,
    interval: f64,
    csv: bool,
    tasks: usize,
    flag: Arc<AtomicBool>,
) -> Result<(), Box<dyn std::error::Error>> {
    let socket = Arc::new(UdpSocket::bind(address).await?);
    socket.connect(remote).await?;
    let mut count: u64 = 0;
    let mut payload = vec![0u8; size];

    let mut samples = Vec::with_capacity(MAX_SAMPLES);
    while flag.load(Relaxed) {
        let count_bytes: [u8; 8] = count.to_le_bytes();
        payload[0..8].copy_from_slice(&count_bytes);
        let now = Instant::now();

        socket.send(&payload).await.unwrap();
        read_exact(&socket, &mut payload).await.unwrap();

        let elapsed = now.elapsed();
        samples.push(elapsed);

        time::sleep(Duration::from_secs_f64(interval)).await;
        count = count.wrapping_add(1);
    }

    for s in samples {
        if csv {
            // framework, transport, test, count, rate, payload, tasks, value, unit
            println!(
                "tokio,udp,rtt,{},{},{},{},{},ns",
                count,
                interval,
                payload.len(),
                tasks,
                s.as_nanos()
            );
        } else {
            println!("{} bytes: seq={} time={:?}", payload.len(), count, s);
        }
    }
    Ok(())
}

async fn run(
    address: SocketAddr,
    remote: SocketAddr,
    size: usize,
    interval: f64,
    csv: bool,
    tasks: usize,
) -> Result<(), Box<dyn std::error::Error>> {
    let socket = Arc::new(UdpSocket::bind(address).await?);
    socket.connect(remote).await?;
    let pending = Arc::new(Mutex::new(HashMap::<u64, Instant>::new()));

    let c_socket = socket.clone();
    let c_pending = pending.clone();
    tokio::task::spawn(async move {
        let mut payload = vec![0u8; size];
        let mut count_bytes = [0u8; 8];
        loop {
            read_exact(&c_socket, &mut payload).await.unwrap();
            count_bytes.copy_from_slice(&payload[0..8]);
            let count = u64::from_le_bytes(count_bytes);

            let instant = c_pending.lock().await.remove(&count).unwrap();

            if csv {
                // framework, transport, test, count, rate, payload, tasks, value, unit
                println!(
                    "tokio,udp,rtt,{},{},{},{},{},ns",
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

        pending.lock().await.insert(count, Instant::now());
        socket.send(&payload).await.unwrap();

        time::sleep(Duration::from_secs_f64(interval)).await;
        count = count.wrapping_add(1);
    }
}

fn main() {
    let args = Args::parse();
    let flag = Arc::new(AtomicBool::new(true));

    let rt = Runtime::new().unwrap();
    let handle = rt.spawn(async move {
        for _ in 0..args.spawn {
            tokio::spawn(async move {
                let mut x: usize = 1;
                loop {
                    x = x.wrapping_mul(2);
                    time::sleep(Duration::from_millis(1)).await;
                }
            });
        }

        if !args.wait {
            run(
                args.address,
                args.remote,
                args.size,
                args.interval,
                args.csv,
                args.spawn,
            )
            .await
            .unwrap();
        }

        let c_duration = args.duration.clone();
        let c_flag = flag.clone();
        tokio::spawn(async move {
            time::sleep(Duration::from_secs(c_duration)).await;
            c_flag.store(false, Relaxed);
        });

        run_wait(
            args.address,
            args.remote,
            args.size,
            args.interval,
            args.csv,
            args.spawn,
            flag,
        )
        .await
        .unwrap();
    });
    rt.block_on(handle).unwrap();
}
