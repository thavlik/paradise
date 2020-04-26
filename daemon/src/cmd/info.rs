use cpal::traits::{DeviceTrait, HostTrait};

/// A subcommand for controlling testing
#[derive(clap::Clap)]
pub struct InfoArgs {
}


pub fn main(args: InfoArgs) -> Result<(), anyhow::Error> {
    println!("Supported hosts:\n  {:?}", cpal::ALL_HOSTS);
    let available_hosts = cpal::available_hosts();
    println!("Available hosts:\n  {:?}", available_hosts);

    for host_id in available_hosts {
        println!("{}", host_id.name());
        let host = cpal::host_from_id(host_id)?;
        let default_in = host.default_input_device()
            .map(|e| e.name())
            .filter(|e| e.is_ok())
            .map(|e| e.unwrap());
        let default_out = host.default_output_device()
            .map(|e| e.name())
            .filter(|e| e.is_ok())
            .map(|e| e.unwrap());
        println!("  Default Input Device:\n    {:?}", default_in);
        println!("  Default Output Device:\n    {:?}", default_out);
        let devices = host.devices()?;
        println!("  Devices: ");
        for (device_index, device) in devices.enumerate() {
            match device.name() {
                Ok(name) => {
                    println!("  {}. \"{}\"", device_index, name);
                },
                Err(e) => {
                    println!("  {}. ERROR: {}", device_index, e);
                    continue;
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
    }

    Ok(())
}
