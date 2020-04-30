use clap::Clap;

pub mod info;
pub mod daemon;
pub mod patch;
pub mod echo;

#[derive(Clap)]
pub enum SubCommand {
    /// Enumerate IO details on all audio devices
    #[clap(name = "info")]
    Info(info::InfoArgs),

    /// Runs the daemon
    #[clap(name = "daemon")]
    Daemon(daemon::DaemonArgs),

    /// Proxies a local port to a destination address
    #[clap(name = "echo")]
    Echo(echo::EchoArgs),

    /// Patch mode
    #[clap(name = "patch")]
    Patch(patch::PatchArgs),
}

/// Bare metal daemon for Paradise audio engine
#[derive(Clap)]
#[clap(version = "1.0", author = "Tom Havlik")]
pub struct Opts {
    #[clap(subcommand)]
    pub subcmd: SubCommand,
}
