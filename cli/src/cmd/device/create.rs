use anyhow::{Result, Error};
use cpal::traits::{DeviceTrait, HostTrait};
use std::process::Command;

#[cfg(target_os = "macos")]
mod macos {
    use super::*;

    const PLUGIN_PREFIX: &'static str = "paradise-";
    const PLUGIN_PATH: &'static str = "/Library/Audio/Plug-Ins/HAL";

    //fn install_device(device_config: DeviceConfig) -> Result<()> {
    //}

    // Removes the driver from the system without restarting Core Audio.
    // Requires sudo.
    fn remove_device(name: &str) -> Result<()> {
        let status = Command::new("sudo")
            .arg("sh")
            .arg("-c")
            .arg(format!("rm -rf {}/{}{}", PLUGIN_PATH, PLUGIN_PREFIX, name))
            .status()?;
        if status.success() {
            Ok(())
        } else {
            Err(Error::msg(format!("command failed with code {:?}", status.code())))
        }
    }

    // Restarts core audio. Requires sudo.
    fn restart_core_audio() -> Result<()> {
        let status = Command::new("sudo")
            .arg("sh")
            .arg("-c")
            .arg("launchctl kickstart -k system/com.apple.audio.coreaudiod")
            .status()?;
        if status.success() {
            Ok(())
        } else {
            Err(Error::msg(format!("command failed with code {:?}", status.code())))
        }
    }

    #[cfg(test)]
    mod test {
        use super::*;

        //#[test]
        //fn restart_core_audio_should_work() {
        //    restart_core_audio().unwrap();
        //}
    }
}

/// Create a virtual audio device
#[derive(clap::Clap)]
pub struct CreateArgs {
    /// Accept the changes without prompting for user input
    #[clap(short = "y")]
    yes: bool,

    /// Virtual device name
    name: String,

    /// Number of input channels
    #[clap(long = "inputs")]
    inputs: Option<usize>,

    /// Network interfaces on which the device should listen
    #[clap(long = "listen")]
    listeners: Vec<String>,

    /// Number of output channels
    #[clap(long = "outputs")]
    outputs: Option<usize>,

    /// Destination addresses for receiving audio
    #[clap(long = "destination")]
    destinations: Vec<String>,
}

pub async fn main(args: CreateArgs) -> Result<(), Error> {
    Err(Error::msg(format!(
        "name = {}, yes = {}, inputs = {:?}, outputs = {:?}",
        &args.name, args.yes, &args.inputs, &args.outputs
    )))
}
