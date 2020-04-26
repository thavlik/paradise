
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};

const LATENCY_MS: f32 = 0.0 ;//150.0;

/// A subcommand for controlling testing
#[derive(clap::Clap)]
pub struct DaemonArgs {
    /// The audio host used for IO
    #[clap(name = "audio-host")]
    audio_host: Option<String>,
}

fn get_host_by_name(name: &str) -> Result<cpal::Host, anyhow::Error> {
    let available_hosts = cpal::available_hosts();
    for host_id in available_hosts {
        if host_id.name() == name {
            return Ok(cpal::host_from_id(host_id)?);
        }
    }
    Err(anyhow::Error::msg(format!("host \"{}\" not found", name)))
}

pub fn main(args: DaemonArgs) -> Result<(), anyhow::Error> {
    let host = match std::env::var("AUDIO_HOST") {
        Ok(name) => get_host_by_name(&name)?,
        _ => match args.audio_host {
            Some(name) => get_host_by_name(&name)?,
            None => cpal::default_host(),
        }
    };
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
