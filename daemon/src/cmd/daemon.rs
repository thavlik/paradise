
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};

const LATENCY_MS: f32 = 0.0 ;//150.0;

/// A subcommand for controlling testing
#[derive(clap::Clap)]
pub struct DaemonArgs {
}

pub fn main(args: DaemonArgs) -> Result<(), anyhow::Error> {
    let host = cpal::default_host();
    // Default devices.
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
        println!("input len: {}", data.len());
    };
    let output_data_fn = move |data: &mut [f32]| {
        println!("output len: {}", data.len());
    };
    let input_stream = input_device.build_input_stream(&config, input_data_fn, err_fn)?;
    let output_stream = output_device.build_output_stream(&config, output_data_fn, err_fn)?;
    input_stream.play()?;
    output_stream.play()?;
    println!("Playing for 3 seconds... ");
    std::thread::sleep(std::time::Duration::from_secs(3));
    drop(input_stream);
    drop(output_stream);
    println!("Done!");
    Ok(())
}

fn err_fn(err: cpal::StreamError) {
    eprintln!("an error occurred on stream: {}", err);
}
