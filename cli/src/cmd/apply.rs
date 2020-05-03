use anyhow::Error;
use cpal::traits::{DeviceTrait, HostTrait};

/// Apply a configuration file
#[derive(clap::Clap)]
pub struct ApplyArgs {
    /// Config file path
    #[clap(short = "f")]
    filename: String,

    /// Accept the changes without prompting for user input
    #[clap(short = "y")]
    yes: bool,
}

pub async fn main(args: ApplyArgs) -> Result<(), Error> {
    Err(Error::msg(format!("filename is {}, yes is {}", &args.filename, args.yes)))
}
