use anyhow::Error;
use cpal::traits::{DeviceTrait, HostTrait};

/// Delete a virtual audio device
#[derive(clap::Clap)]
pub struct DeleteArgs {
}

pub async fn main(args: DeleteArgs) -> Result<(), Error> {
    Err(Error::msg("unimplemented"))
}
