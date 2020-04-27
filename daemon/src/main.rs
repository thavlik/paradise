use clap::Clap;

fn get_host_by_name(name: &str) -> Result<cpal::Host, anyhow::Error> {
    let available_hosts = cpal::available_hosts();
    for host_id in available_hosts {
        if host_id.name() == name {
            return Ok(cpal::host_from_id(host_id)?);
        }
    }
    Err(anyhow::Error::msg(format!("host \"{}\" not found", name)))
}

type TxStream = paradise::stream::tx::udp::UdpTxStream::<paradise::stream::tx::locking::LockingTxBuffer>;
type RxStream = paradise::stream::rx::udp::UdpRxStream::<paradise::stream::rx::locking::LockingRxBuffer>;

mod cmd;

fn main() {
    tokio::runtime::Builder::new()
        .threaded_scheduler()
        .build()
        .unwrap()
        .block_on(async {
            let opts: cmd::Opts = cmd::Opts::parse();
            match opts.subcmd {
                cmd::SubCommand::Info(args) => cmd::info::main(args).unwrap(),
                cmd::SubCommand::Daemon(args) => cmd::daemon::main(args).await.unwrap(),
                cmd::SubCommand::Patch(args) => cmd::patch::main(args).await.unwrap(),
            };
        });
}
