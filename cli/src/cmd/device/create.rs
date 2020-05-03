use anyhow::Error;
use cpal::traits::{DeviceTrait, HostTrait};

/// Create a virtual audio device
#[derive(clap::Clap)]
pub struct CreateArgs {
    /// Accept the changes without prompting for user input
    #[clap(short = "y")]
    yes: bool,
}

pub async fn main(args: CreateArgs) -> Result<(), Error> {
    Err(Error::msg("unimplemented"))
}
