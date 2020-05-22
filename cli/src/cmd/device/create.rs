use anyhow::{anyhow, Error, Result, Context, bail};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use serde::{Deserialize, Serialize};
use std::path::{self, Path, PathBuf};
use std::process::Command;
use uuid::Uuid;
use quinn::{
    ServerConfig,
    ServerConfigBuilder,
    TransportConfig,
    CertificateChain,
    PrivateKey,
    Certificate,
};
use futures::future::{Abortable, AbortHandle, Aborted};
use std::{
    ascii,
    io,
    str,
    net::SocketAddr,
    sync::{Arc, mpsc, Mutex, atomic::{AtomicU64, Ordering}},
    fs,
};
use paradise_core::{Frame, device::{DeviceSpec, Endpoint}};
use crossbeam::channel::{Sender, Receiver};

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
