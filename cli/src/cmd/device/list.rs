use anyhow::Error;
use cpal::traits::{DeviceTrait, HostTrait};
use crate::api::Device;

/// Lists virtual audio devices
#[derive(clap::Clap)]
pub struct ListArgs {
}

pub async fn main(args: ListArgs) -> Result<(), Error> {
    println!("Supported hosts:\n  {:?}", cpal::ALL_HOSTS);
    let available_hosts = cpal::available_hosts();
    println!("Available hosts:\n  {:?}", available_hosts);
    for host_id in available_hosts {
        println!("{}", host_id.name());
        let host = cpal::host_from_id(host_id)?;
        let devices = host.devices()?;
        println!("  Devices: ");

        /*
        let d = host.devices()?.enumerate()
            .filter(|(_, d)| match d.supported_input_configs() {
                Ok(mut input_configs) => {
                    match input_configs.peekable().peek() {
                        Some(_) => true,
                        None => false,
                    }
                },
                Err(e) => {
                    println!("device {:?} error: {:?}", d.name(), e);
                    false
                },
            })
            .map(|(_, d)| {
                let d: cpal::Device = d;
                let mut sample_rate_range: Option<(cpal::SampleRate, cpal::SampleRate)> = None;
                if let Ok(mut input_configs) = d.supported_input_configs() {
                    let mut input_configs: std::iter::Peekable<cpal::SupportedInputConfigs> = input_configs.peekable();
                    if input_configs.peek().is_some() {
                        sample_rate_range = Some(input_configs.into_iter()
                            .fold((cpal::SampleRate(u32::MAX), cpal::SampleRate(0)), |p, c| {
                                (p.0.min(c.min_sample_rate()), p.1.max(c.max_sample_rate()))
                            }));
                    }
                }
                if let Ok(mut output_configs) = d.supported_output_configs() {
                    let mut output_configs: std::iter::Peekable<cpal::SupportedOutputConfigs> = output_configs.peekable();
                    if output_configs.peek().is_some() {
                        let (min_sample_rate, max_sample_rate) = output_configs.into_iter()
                            .fold((cpal::SampleRate(u32::MAX), cpal::SampleRate(0)), |p, c| {
                                (p.0.min(c.min_sample_rate()), p.1.max(c.max_sample_rate()))
                            });
                    }
                }
                //let mut supported_sample_rates = vec![44100, 48000, 88200, 96000, 176400, 192000];
                //supported_sample_rates.into_iter().filter(|r| )
                Device{
                    name: String::from(d.name().unwrap_or("NULL".to_string())),
                    inputs: 2,
                    outputs: 2,
                    supported_sample_rates: vec![44100, 48000, 96000, 192000],
                    supported_sample_formats: vec![String::from("F32")],
                }
            })
            .collect::<Vec<_>>();*/

        let d = host.devices()?.enumerate()
            .map(|(_, d)| {
                if let Ok(conf) = d.default_input_config() {
                    println!("    Default input stream config:\n      {:?}", conf);
                }
                if let Ok(mut input_configs) = d.supported_input_configs() {
                    let mut input_configs: std::iter::Peekable<cpal::SupportedInputConfigs> = input_configs.peekable();
                    //let input_sample_rates = match input_configs.peek() {
                    //    Some(_) => input_configs.enumerate().map(|(i, config)| {}).collect::<Vec<_>>(),
                    //    None => return anyhow::Error::msg("unable to read input config"),
                    //};
                    if input_configs.peek().is_some() {
                        let configs = input_configs.enumerate().map(|(i, config)| {
                            (config.channels(), config.min_sample_rate(), config.max_sample_rate(), config.sample_format())
                        }).collect::<Vec<_>>();
                        configs.iter()
                            .enumerate()
                            .for_each(|(i, name)| {
                                println!("        {}. {:?}", i, name);
                            });
                    }
                }
                Device{
                    name: String::from(d.name().unwrap_or("NULL".to_string())),
                    inputs: 2,
                    outputs: 2,
                    supported_sample_rates: vec![48000],
                    supported_sample_formats: vec![String::from("F32")],
                }
            }).collect::<Vec<_>>();

        d.iter().for_each(|d| {});

        /*
        for (device_index, device) in devices.enumerate() {
            match device.name() {
                Ok(name) => {
                    println!("  {}. \"{}\"", device_index, name);
                },
                Err(e) => {
                    println!("  {}. ERROR: {}", device_index, e);
                },
            }

            // Input configs
            if let Ok(conf) = device.default_input_config() {
                println!("    Default input stream config:\n      {:?}", conf);
            }
            let mut input_configs = match device.supported_input_configs() {
                Ok(f) => f.peekable(),
                Err(e) => {
                    println!("Error: {:?}", e);
                    continue;
                }
            };
            if input_configs.peek().is_some() {
                println!("    All supported input stream configs:");
                for (config_index, config) in input_configs.enumerate() {
                    println!(
                        "      {}.{}. {:?}",
                        device_index + 1,
                        config_index + 1,
                        config
                    );
                }
            }

            // Output configs
            if let Ok(conf) = device.default_output_config() {
                println!("    Default output stream config:\n      {:?}", conf);
            }
            let mut output_configs = match device.supported_output_configs() {
                Ok(f) => f.peekable(),
                Err(e) => {
                    println!("Error: {:?}", e);
                    continue;
                }
            };
            if output_configs.peek().is_some() {
                println!("    All supported output stream configs:");
                for (config_index, config) in output_configs.enumerate() {
                    println!(
                        "      {}.{}. {:?}",
                        device_index + 1,
                        config_index + 1,
                        config
                    );
                }
            }
        }
        */
    }

    Ok(())
}