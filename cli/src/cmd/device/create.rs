use anyhow::{anyhow, Error, Result, Context, bail};
use cpal::traits::{DeviceTrait};
use paradise_core::device::{DeviceSpec, Endpoint};
use super::platform;

/// Create a virtual audio device
#[derive(clap::Clap)]
pub struct CreateArgs {
    /// Accept the changes without prompting for user input
    #[clap(short = "y")]
    yes: bool,

    /// Virtual device name
    #[clap(long = "name", short = "n")]
    name: String,

    /// Destination address for receiving audio
    #[clap(long = "dest", short = "d")]
    dest: String,
}

pub async fn main(args: CreateArgs) -> Result<()> {
    info!(
        "installing device, name = {}, dest = {}, yes = {}",
        &args.name, &args.dest, args.yes,
    );

    let device = DeviceSpec {
        name: args.name.clone(),
        outputs: 2,
        inputs: 2,
        endpoints: vec![Endpoint {
            name: String::from("default"),
            insecure: true,
            addr: args.dest.clone(),
        }],
        display_name: format!("{} (Paradise)", &args.name),
    };

    platform::install_device(&device).await?;

    platform::restart().await?;

    info!("installed device '{}'", &args.name);

    Ok(())
}
