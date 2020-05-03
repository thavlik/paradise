use anyhow::Error;
use cpal::traits::{DeviceTrait, HostTrait};

/// Apply a configuration file
#[derive(clap::Clap)]
pub struct ApplyArgs {
    /// Accept the changes without prompting for user input
    #[clap(short = "y")]
    yes: bool,
}

pub async fn main(args: ApplyArgs) -> Result<(), Error> {
    Err(Error::msg(format!("yes is {}", args.yes)))
}
