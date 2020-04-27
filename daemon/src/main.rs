use clap::Clap;

mod cmd;

fn main() {
    tokio::runtime::Builder::new()
        .threaded_scheduler()
        .build()
        .unwrap()
        .block_on(async {
            let opts: cmd::Opts = cmd::Opts::parse();
            match opts.subcmd {
                cmd::SubCommand::Info(args) => cmd::info::main(args),
                cmd::SubCommand::Daemon(args) => cmd::daemon::main(args).await,
            };
        });
}
