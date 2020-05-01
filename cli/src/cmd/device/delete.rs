use anyhow::Error;
use cpal::traits::{DeviceTrait, HostTrait};
use crate::api::Device;

/// Delete a virtual audio device
#[derive(clap::Clap)]
pub struct DeleteArgs {
}

pub async fn main(args: DeleteArgs) -> Result<(), Error> {
    Err(Error::msg("unimplemented"))
}
