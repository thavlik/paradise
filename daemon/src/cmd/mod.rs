use clap::Clap;

pub mod info;
pub mod daemon;

#[derive(Clap)]
pub enum SubCommand {
    /// Enumerate IO details on all audio devices
    #[clap(name = "info")]
    Info(info::InfoArgs),

    /// Runs the daemon
    #[clap(name = "info")]
    Daemon(daemon::DaemonArgs),
}

/// Bare metal daemon for Paradise audio engine
#[derive(Clap)]
#[clap(version = "1.0", author = "Tom Havlik")]
pub struct Opts {
    #[clap(subcommand)]
    pub subcmd: SubCommand,
}

