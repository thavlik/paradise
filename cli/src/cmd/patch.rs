use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use anyhow::{Error, Result};

/// A subcommand for controlling testing
#[derive(clap::Clap)]
pub struct PatchArgs {
    /// The audio host used for IO. Default is system default.
    #[clap(long = "host")]
    host: Option<String>,

    /// Output audio default. Can be the name or index.
    /// Default is system default.
    #[clap(long = "device", short = "d")]
    device: Option<String>,

    /// Channel index. Starts at zero. Multiple channels
    /// should be comma separated, e.g. 0,1,7,8
    /// Default is channel 0 only
    #[clap(long = "channel", short = "c")]
    channel: Option<String>,

    /// Source network interface, e.g. 0.0.0.0:30000
    /// for all interfaces port 30000. Only defined
    /// when patching from network to a device output.
    #[clap(long = "source")]
    source: Option<String>,

    /// Sink address for receiving audio. Only defined
    /// when patching from device output patch, as
    /// patching into a device input entails using
    /// the `source` flag instead.
    #[clap(long = "sink")]
    sink: Option<String>,

    /// Sample rate. Default allows the device to choose.
    #[clap(long = "sample-rate")]
    sample_rate: Option<usize>,
}

type TxStream = paradise_core::stream::tx::udp::UdpTxStream<
    paradise_core::stream::tx::locking::LockingTxBuffer,
>;
type RxStream = paradise_core::stream::rx::udp::UdpRxStream<
    paradise_core::stream::rx::locking::LockingRxBuffer,
>;

pub async fn main(args: PatchArgs) -> Result<()> {
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
    match (&args.source, &args.sink) {
        (Some(source), Some(sink)) => return Err(Error::msg("source and sink cannot be specified at the same time")),
        (None, None) => return Err(Error::msg("you must specify a source or sink address")),
        (Some(source), _) => {},
        (_, Some(sink)) => {},
    };
    /*
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

    let stream = RxStream::new(args.source.parse()?)?;

    let conf = device.default_output_config().unwrap();
    let conf: cpal::StreamConfig = conf.into();
    let s = stream.clone();
    let output_data_fn = move |data: &mut [f32]| {
        unsafe {
            // If we don't zero the buffer, it'll stay at a fixed
            // tone when the stream stops.
            std::ptr::write_bytes(data.as_mut_ptr(), 0, data.len());
        }
        let clock = paradise_core::stream::rx::RxStream::process(&*s, data);
    };
    let output_stream = device.build_output_stream(&conf, output_data_fn, err_fn)?;

    output_stream.play()?;

    println!("{} -> \"{}\"", args.source, device.name().unwrap_or(String::from("NULL")));

    loop {
        std::thread::yield_now();
    }

    println!("shutting down");
    */
    Ok(())
}

fn err_fn(err: cpal::StreamError) {
    eprintln!("an error occurred on stream: {}", err);
}
