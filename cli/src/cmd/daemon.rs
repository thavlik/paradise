use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use std::net::SocketAddr;

const LATENCY_MS: f32 = 0.0; //150.0;

/// A subcommand for controlling testing
#[derive(clap::Clap)]
pub struct DaemonArgs {
    /// The audio host used for IO
    #[clap(long = "audio-host")]
    audio_host: Option<String>,

    #[clap(long = "port", short = "p", default_value = "8080")]
    port: u16,
}

pub async fn main(args: DaemonArgs) -> Result<(), anyhow::Error> {
    /*
    let host = match std::env::var("AUDIO_HOST") {
        Ok(name) => crate::util::get_host_by_name(&name)?,
        _ => match args.audio_host {
            Some(name) => crate::util::get_host_by_name(&name)?,
            None => cpal::default_host(),
        }
    };
    let devices = host.devices()?;
    let mut port = 30005;
    //let mut rx = vec![];
    for (device_index, device) in devices.enumerate() {
        if device_index != 4 {
            continue;
        }
        match device.name() {
            Ok(name) => {
                println!("  {}. \"{}\"", device_index, name);
            },
            Err(e) => {
                println!("  {}. ERROR: {}", device_index, e);
                continue;
            },
        }
        if let Ok(conf) = device.default_input_config() {
            let conf: cpal::StreamConfig = conf.into();
            //println!("    Default input stream config:\n      {:?}", conf);
            // Create tx socket
            //paradise_core::stream::tx::TxStream::new()
            let input_data_fn = move |data: &[f32]| {
            };
            let input_stream = device.build_input_stream(&conf, input_data_fn, err_fn)?;
        }
        if let Ok(conf) = device.default_output_config() {
            let conf: cpal::StreamConfig = conf.into();
            /*
            //println!("    Default output stream config:\n      {:?}", conf);
            println!("    0.0.0.0:{} -> ANALOG SIGNAL", port);
            println!("");
            let stream = crate::RxStream::new(port).expect("failed to create rx stream");
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
            rx.push((stream.clone(), output_stream));
            */
            port += 1;
        }
    }
    */
    /*
    let input_device = host
        .default_input_device()
        .expect("failed to get default input device");
    let output_device = host
        .default_output_device()
        .expect("failed to get default output device");
    println!("Using default input device: \"{}\"", input_device.name()?);
    println!("Using default output device: \"{}\"", output_device.name()?);
    let config: cpal::StreamConfig = input_device.default_input_config()?.into();
    let input_data_fn = move |data: &[f32]| {
    };
    let output_data_fn = move |data: &mut [f32]| {
    };
    let input_stream = input_device.build_input_stream(&config, input_data_fn, err_fn)?;
    let output_stream = output_device.build_output_stream(&config, output_data_fn, err_fn)?;
    input_stream.play()?;
    */
    println!("Playing for 10000 seconds... ");
    std::thread::sleep(std::time::Duration::from_secs(10000));
    /*
    drop(input_stream);
    drop(output_stream);
    println!("Done!");
    */
    Ok(())
}

fn err_fn(err: cpal::StreamError) {
    eprintln!("an error occurred on stream: {}", err);
}
