

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};

const LATENCY_MS: f32 = 0.0 ;//150.0;

/// A subcommand for controlling testing
#[derive(clap::Clap)]
pub struct PatchArgs {
    /// The audio host used for IO. Default is system default.
    #[clap(long = "host")]
    host: Option<String>,

    /// Output audio default. Can be the name or index. Default is system default.
    #[clap(long = "device", short = "d")]
    device: Option<String>,

    /// Source UDP port
    #[clap(long = "port", short = "p")]
    port: u16,
}

type TxStream = paradise::stream::tx::udp::UdpTxStream::<paradise::stream::tx::locking::LockingTxBuffer>;
type RxStream = paradise::stream::rx::udp::UdpRxStream::<paradise::stream::rx::locking::LockingRxBuffer>;

pub async fn main(args: PatchArgs) -> Result<(), anyhow::Error> {
    let host = match args.host {
        Some(name) => {
            let host = crate::util::get_host_by_name(&name)?;
            println!("found host \"{}\"", &name);
            host
        },
        None => {
            let host = cpal::default_host();
            println!("using default host \"{:?}\"", host.id());
            host
        },
    };
    let device = match args.device {
        Some(name) => {
            match name.parse::<usize>() {
                Ok(index) => {
                    if index >= host.devices()?.count() {
                       return Err(anyhow::Error::msg(format!("device index out of range (tip: run info)")));
                    }
                    match host.devices()?
                        .skip(index)
                        .next() {
                        Some(device) => {
                            println!("found device {}. \"{}\"", &name, device.name().unwrap_or(String::from("NULL")));
                            device
                        },
                        None => return Err(anyhow::Error::msg(format!("device at index \"{}\" not found (tip: run info)", &name))),
                    }
                },
                _ => match host.devices()?
                    .enumerate()
                    .find(|(i, d)| match d.name() {
                        Ok(n) => n == name,
                        _ => false,
                    }) {
                    Some((_, d)) => {
                        println!("found device \"{}\"", &name);
                        d
                    },
                    None => return Err(anyhow::Error::msg(format!("device \"{}\" not found", name))),
                },
            }
        },
        None => match host.default_output_device() {
            Some(device) => {
                println!("using default device \"{}\"", &device.name().unwrap_or(String::from("NULL")));
                device
            },
            None => return Err(anyhow::Error::msg(format!("default output device not available"))),
        },
    };
    println!("listening on {}", args.port);
    let stream = RxStream::new(args.port)?;

    println!("shutting down");
    Ok(())
}

fn err_fn(err: cpal::StreamError) {
    eprintln!("an error occurred on stream: {}", err);
}
