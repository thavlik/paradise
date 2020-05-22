#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate crossbeam;
//extern crate paradise_device;
//#[macro_use]
//extern crate serde;

use anyhow::{Error, Result};
use clap::Clap;

//type Result<T> = std::result::Result<T, anyhow::Error>;
//
//type TxStream = paradise_core::stream::tx::udp::UdpTxStream<
//    paradise_core::stream::tx::locking::LockingTxBuffer,
//>;
//type RxStream = paradise_core::stream::rx::udp::UdpRxStream<
//    paradise_core::stream::rx::locking::LockingRxBuffer,
//>;

mod api;
mod cmd;
//mod stream;
mod util;

fn main() {
    tokio::runtime::Builder::new()
        .threaded_scheduler()
        .enable_all()
        .build()
        .unwrap()
        .block_on(async {
            let opts: cmd::Opts = cmd::Opts::parse();
            match opts.subcmd {
                cmd::SubCommand::Apply(args) => cmd::apply::main(args).await.unwrap(),
                cmd::SubCommand::Daemon(args) => cmd::daemon::main(args).await.unwrap(),
                cmd::SubCommand::Create(args) => cmd::device::create::main(args).await.unwrap(),
                cmd::SubCommand::Delete(args) => cmd::device::delete::main(args).await.unwrap(),
                cmd::SubCommand::List(args) => cmd::device::list::main(args).await.unwrap(),
                //cmd::SubCommand::Echo(args) => cmd::echo::main(args).await.unwrap(),
                cmd::SubCommand::Info(args) => cmd::info::main(args).await.unwrap(),
                cmd::SubCommand::Patch(args) => cmd::patch::main(args).await.unwrap(),
                cmd::SubCommand::Reconcile(args) => cmd::reconcile::main(args).await.unwrap(),
            };
        });
}
