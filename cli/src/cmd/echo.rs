use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};

type TxStream = paradise_core::stream::tx::udp::UdpTxStream<
    paradise_core::stream::tx::locking::LockingTxBuffer,
>;
type RxStream = paradise_core::stream::rx::udp::UdpRxStream<
    paradise_core::stream::rx::locking::LockingRxBuffer,
>;

/// A subcommand for controlling testing
#[derive(clap::Clap)]
pub struct EchoArgs {
    /// Source UDP port
    #[clap(long = "source", short = "s")]
    source: String,

    /// Source UDP port
    #[clap(long = "dest", short = "d")]
    dest: String,
}

pub async fn main(args: EchoArgs) -> Result<(), anyhow::Error> {
    let rx = RxStream::new(args.source.parse()?)?;
    let tx = TxStream::new(args.dest.parse()?)?;
    println!("{} -> {}", args.source, args.dest);
    loop {
        std::thread::yield_now();
    }
    println!("shutting down");
    Ok(())
}
