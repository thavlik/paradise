use anyhow::Error;
use cpal::traits::{DeviceTrait, HostTrait};

/// Create a virtual audio device
#[derive(clap::Clap)]
pub struct CreateArgs {
}

pub async fn main(args: CreateArgs) -> Result<(), Error> {
    Err(Error::msg("unimplemented"))
}
