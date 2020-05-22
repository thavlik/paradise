use anyhow::{anyhow, Error, Result, Context, bail};
use cpal::traits::{DeviceTrait};










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
    #[clap(long = "destination", short = "d")]
    dest: String,
}

pub async fn main(args: CreateArgs) -> Result<()> {
    info!(
        "name = {}, dest = {}, yes = {}",
        &args.name, &args.dest, args.yes,
    );
    Ok(())
}
