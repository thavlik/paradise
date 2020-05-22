#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate crossbeam;
#[macro_use]
extern crate log;


use clap::Clap;

mod api;
mod cmd;
mod util;

fn main() {
    let log_level = match std::env::var("LOG_LEVEL") {
        Ok(v) => v,
        _ => String::from("debug"),
    };
    std::env::set_var("RUST_LOG", format!("paradise={}", log_level));
    std::env::set_var("RUST_BACKTRACE", "1");
    env_logger::init();

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
                cmd::SubCommand::Info(args) => cmd::info::main(args).await.unwrap(),
                cmd::SubCommand::Patch(args) => cmd::patch::main(args).await.unwrap(),
                cmd::SubCommand::Reconcile(args) => cmd::reconcile::main(args).await.unwrap(),
            };
        });
}
