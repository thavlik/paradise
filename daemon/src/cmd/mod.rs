use clap::Clap;

pub mod info;

#[derive(Clap)]
pub enum SubCommand {
    /// Enumerate IO details on all audio devices
    #[clap(name = "info")]
    Info(info::InfoArgs),
}

/// Bare metal daemon for Paradise audio engine
#[derive(Clap)]
#[clap(version = "1.0", author = "Tom Havlik")]
pub struct Opts {
    #[clap(subcommand)]
    pub subcmd: SubCommand,
}

