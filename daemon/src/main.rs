use clap::Clap;

mod cmd;

fn main() -> Result<(), anyhow::Error> {
    let opts: cmd::Opts = cmd::Opts::parse();

    match opts.subcmd {
        cmd::SubCommand::Info(args) => cmd::info::main(args),
    }
}
