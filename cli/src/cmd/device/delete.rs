use anyhow::Error;
use cpal::traits::{DeviceTrait, HostTrait};

/// Delete a virtual audio device
#[derive(clap::Clap)]
pub struct DeleteArgs {
    /// Accept the changes without prompting for user input
    #[clap(short = "y")]
    yes: bool,

    /// Delete all virtual devices
    #[clap(long = "all")]
    all: bool,

    names: Vec<String>,
}

pub async fn main(args: DeleteArgs) -> Result<(), Error> {
    if args.names.len() == 0 && !args.all {
        return Err(Error::msg("you must specify at least one device name or --all to delete all devices"));
    }
    Err(Error::msg("unimplemented"))
}
