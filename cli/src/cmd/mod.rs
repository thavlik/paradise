use clap::Clap;

pub mod apply;
pub mod daemon;
pub mod device;
//pub mod echo;
pub mod info;
pub mod patch;
pub mod reconcile;

#[derive(Clap)]
pub enum SubCommand {
    /// Runs the daemon
    #[clap(name = "apply")]
    Apply(apply::ApplyArgs),

    /// Runs the daemon
    #[clap(name = "daemon")]
    Daemon(daemon::DaemonArgs),

    ///
    #[clap(name = "create")]
    Create(device::create::CreateArgs),

    ///
    #[clap(name = "delete")]
    Delete(device::delete::DeleteArgs),

    ///
    #[clap(name = "list")]
    List(device::list::ListArgs),

    /// Proxies a local port to a destination address
    //#[clap(name = "echo")]
    //Echo(echo::EchoArgs),

    /// Enumerate IO details on all audio devices
    #[clap(name = "info")]
    Info(info::InfoArgs),

    /// Patch mode
    #[clap(name = "patch")]
    Patch(patch::PatchArgs),

    /// Reconcile system drivers with config
    #[clap(name = "reconcile")]
    Reconcile(reconcile::ReconcileArgs),
}

/// Bare metal daemon for Paradise audio engine
#[derive(Clap)]
#[clap(version = "1.0", author = "Tom Havlik")]
pub struct Opts {
    #[clap(subcommand)]
    pub subcmd: SubCommand,
}
