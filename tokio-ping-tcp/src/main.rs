use clap::Parser;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering::*;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::io::AsyncReadExt;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;
use tokio::runtime::Runtime;
use tokio::sync::Mutex;
use tokio::time;
const MAX_SAMPLES: usize = 100_000_000;

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
    #[clap(short, long, default_value = "60")]
    duration: u64,
}

async fn run_wait(
    address: SocketAddr,
    size: usize,
    interval: f64,
    csv: bool,
    tasks: usize,
    flag: Arc<AtomicBool>,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut stream = TcpStream::connect(address).await?;
    stream.set_nodelay(true)?;
    let mut count: u64 = 0;
    let mut payload = vec![0u8; size];
    let mut samples = Vec::with_capacity(MAX_SAMPLES);
    while flag.load(Relaxed) {
        let count_bytes: [u8; 8] = count.to_le_bytes();
        payload[0..8].copy_from_slice(&count_bytes);
        let now = Instant::now();

        stream.write_all(&payload).await.unwrap();
        stream.read_exact(&mut payload).await.unwrap();
        let elapsed = now.elapsed();
        samples.push(elapsed);

        time::sleep(Duration::from_secs_f64(interval)).await;
        count = count.wrapping_add(1);
    }

    stream.shutdown().await.unwrap();

    for s in samples {
        if csv {
            // framework, transport, test, count, rate, payload, tasks, value, unit
            println!(
                "tokio,tcp,rtt,{},{},{},{},{},ns",
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
    size: usize,
    interval: f64,
    csv: bool,
    tasks: usize,
) -> Result<(), Box<dyn std::error::Error>> {
    let stream = TcpStream::connect(address).await?;
    stream.set_nodelay(true)?;
    let pending = Arc::new(Mutex::new(HashMap::<u64, Instant>::new()));

    let (mut read_stream, mut write_stream) = stream.into_split();

    // let mut c_stream = read_stream.clone();
    let c_pending = pending.clone();
    tokio::task::spawn(async move {
        let mut payload = vec![0u8; size];
        let mut count_bytes = [0u8; 8];
        loop {
            read_stream.read_exact(&mut payload).await.unwrap();
            count_bytes.copy_from_slice(&payload[0..8]);
            let count = u64::from_le_bytes(count_bytes);

            let instant = c_pending.lock().await.remove(&count).unwrap();

            if csv {
                // framework, transport, test, count, rate, payload, tasks, value, unit
                println!(
                    "tokio,tcp,rtt,{},{},{},{},{},ns",
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
        write_stream.write_all(&payload).await.unwrap();

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
            run(args.address, args.size, args.interval, args.csv, args.spawn)
                .await
                .unwrap();
        } else {
            let c_duration = args.duration.clone();
            let c_flag = flag.clone();
            tokio::spawn(async move {
                time::sleep(Duration::from_secs(c_duration)).await;
                c_flag.store(false, Relaxed);
            });

            run_wait(
                args.address,
                args.size,
                args.interval,
                args.csv,
                args.spawn,
                flag,
            )
            .await
            .unwrap();
        }
    });
    rt.block_on(handle).unwrap();
}
