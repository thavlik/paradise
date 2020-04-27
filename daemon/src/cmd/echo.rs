use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};

type TxStream = paradise::stream::tx::udp::UdpTxStream::<paradise::stream::tx::locking::LockingTxBuffer>;
type RxStream = paradise::stream::rx::udp::UdpRxStream::<paradise::stream::rx::locking::LockingRxBuffer>;

/// A subcommand for controlling testing
#[derive(clap::Clap)]
pub struct EchoArgs {
    /// Source UDP port
    #[clap(long = "source", short = "s")]
    source: u16,

    /// Source UDP port
    #[clap(long = "dest", short = "d")]
    dest: String,
}

pub async fn main(args: EchoArgs) -> Result<(), anyhow::Error> {
    let rx = RxStream::new(args.source)?;
    let tx = TxStream::new(args.dest.parse()?)?;
    println!("0.0.0.0:{} -> {}", args.source, args.dest);
    loop {
        std::thread::yield_now();
    }
    println!("shutting down");
    Ok(())
}

