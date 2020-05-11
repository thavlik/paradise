use anyhow::{Result, Error};
use cpal::traits::{DeviceTrait, HostTrait};
use std::process::Command;

#[cfg(target_os = "macos")]
mod macos {
    use super::*;

    //fn install_device(device_config: DeviceConfig) -> Result<()> {
    //}

    // TODO: remove the driver from the macOS system, restart CoreAudio with
    //fn remove_device(uid: Uuid) -> Result<()> {
    //}

    // TODO: run this shell script
    fn restart_core_audio() -> Result<()> {
        let status = Command::new("bash")
            .arg("-c")
            .arg("sudo launchctl kickstart -k system/com.apple.audio.coreaudiod")
            .status()?;
        if status.success() {
            Ok(())
        } else {
            Err(Error::msg(format!("restart command exited with code {:?}", status.code())))
        }
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
