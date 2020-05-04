use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use std::{
    ascii, fs, io,
    net::SocketAddr,
    path::{self, Path, PathBuf},
    str,
    sync::Arc,
};
use anyhow::{anyhow, bail, Error, Context, Result};
use futures::{StreamExt, TryFutureExt};
use structopt::{self, StructOpt};

/// A subcommand for controlling testing
#[derive(clap::Clap)]
pub struct PatchArgs {
    /// The audio host used for IO. Default is system default.
    #[clap(long = "host")]
    host: Option<String>,

    /// Output audio default. Can be the name or index.
    /// Default is system default.
    #[clap(long = "device", short = "d")]
    device: Option<String>,

    /// Channel index. Starts at zero. Multiple channels
    /// should be comma separated, e.g. 0,1,7,8
    /// Default is channel 0 only
    #[clap(long = "channel", short = "c")]
    channel: Option<String>,

    /// Source network interface, e.g. 0.0.0.0:30000
    /// for all interfaces port 30000. Only defined
    /// when patching from network to a device output.
    #[clap(long = "source")]
    source: Option<String>,

    /// Sink address for receiving audio. Only defined
    /// when patching from device output patch, as
    /// patching into a device input entails using
    /// the `source` flag instead.
    #[clap(long = "sink")]
    sink: Option<String>,

    /// Sample rate. Default allows the device to choose.
    #[clap(long = "sample-rate")]
    sample_rate: Option<usize>,

    ///
    #[clap(parse(from_os_str), long = "key")]
    key: Option<PathBuf>,

    ///
    #[clap(parse(from_os_str), long = "key")]
    cert: Option<PathBuf>,
}

type TxStream = paradise_core::stream::tx::udp::UdpTxStream<
    paradise_core::stream::tx::locking::LockingTxBuffer,
>;
type RxStream = paradise_core::stream::rx::udp::UdpRxStream<
    paradise_core::stream::rx::locking::LockingRxBuffer,
>;

#[allow(unused)]
pub const ALPN_QUIC_HTTP: &[&[u8]] = &[b"hq-27"];

pub async fn main(args: PatchArgs) -> Result<()> {
    let host = match &args.host {
        Some(name) => {
            let host = crate::util::get_host_by_name(name)?;
            println!("found host \"{}\"", name);
            host
        },
        None => {
            let host = cpal::default_host();
            println!("using default host \"{:?}\"", host.id());
            host
        },
    };

    let (addr, device_is_output) = match (&args.source, &args.sink) {
        (Some(source), Some(sink)) => return Err(Error::msg("source and sink cannot be specified at the same time")),
        (None, None) => return Err(Error::msg("you must specify a source or sink address")),
        (Some(source), _) => (source, true), // Source address, sink device output
        (_, Some(sink)) => (sink, false), // Source device input, sink address
    };

    let addr: SocketAddr = addr.parse()?;

    let device: cpal::Device = match args.device {
        Some(name) => {
            match name.parse::<usize>() {
                Ok(index) => {
                    if index >= host.devices()?.count() {
                        return Err(Error::msg(format!("device index out of range (tip: run info)")));
                    }
                    match host.devices()?
                        .skip(index)
                        .next() {
                        Some(device) => {
                            println!("found device {}. \"{}\"", &name, device.name().unwrap_or(String::from("NULL")));
                            device
                        },
                        None => return Err(Error::msg(format!("device at index \"{}\" not found (tip: run info)", &name))),
                    }
                },
                _ => match host.devices()?
                    .enumerate()
                    .find(|(_, d)| match d.name() {
                        Ok(n) => n == name,
                        _ => false,
                    }) {
                    Some((_, d)) => {
                        println!("found device \"{}\"", &name);
                        d
                    },
                    None => return Err(Error::msg(format!("device \"{}\" not found", name))),
                },
            }
        },
        None => if device_is_output {
            match host.default_output_device() {
                Some(device) => {
                    println!("using default output device \"{}\"", &device.name().unwrap_or(String::from("NULL")));
                    device
                },
                None => return Err(Error::msg(format!("default output device not available"))),
            }
        } else {
            match host.default_input_device() {
                Some(device) => {
                    println!("using default input device \"{}\"", &device.name().unwrap_or(String::from("NULL")));
                    device
                },
                None => return Err(Error::msg(format!("default input device not available"))),
            }
        },
    };

    if device_is_output {
        // TODO: Listen on addr, play on device
        let mut transport_config = quinn::TransportConfig::default();
        transport_config.stream_window_uni(0);
        let mut server_config = quinn::ServerConfig::default();
        server_config.transport = std::sync::Arc::new(transport_config);
        let mut server_config = quinn::ServerConfigBuilder::new(server_config);
        server_config.protocols(ALPN_QUIC_HTTP);
        //server_config.use_stateless_retry(true);

        if let (Some(key_path), Some(cert_path)) = (&args.key, &args.cert) {
            let key = fs::read(key_path).context("failed to read private key")?;
            let key = if key_path.extension().map_or(false, |x| x == "der") {
                quinn::PrivateKey::from_der(&key)?
            } else {
                quinn::PrivateKey::from_pem(&key)?
            };
            let cert_chain = fs::read(cert_path).context("failed to read certificate chain")?;
            let cert_chain = if cert_path.extension().map_or(false, |x| x == "der") {
                quinn::CertificateChain::from_certs(quinn::Certificate::from_der(&cert_chain))
            } else {
                quinn::CertificateChain::from_pem(&cert_chain)?
            };
            server_config.certificate(cert_chain, key)?;
        } else {
            let dirs = directories::ProjectDirs::from("org", "quinn", "quinn-examples").unwrap();
            let path = dirs.data_local_dir();
            let cert_path = path.join("cert.der");
            let key_path = path.join("key.der");
            let (cert, key) = match fs::read(&cert_path).and_then(|x| Ok((x, fs::read(&key_path)?))) {
                Ok(x) => x,
                Err(ref e) if e.kind() == io::ErrorKind::NotFound => {
                    println!("generating self-signed certificate");
                    let cert = rcgen::generate_simple_self_signed(vec!["localhost".into()]).unwrap();
                    let key = cert.serialize_private_key_der();
                    let cert = cert.serialize_der().unwrap();
                    fs::create_dir_all(&path).context("failed to create certificate directory")?;
                    fs::write(&cert_path, &cert).context("failed to write certificate")?;
                    fs::write(&key_path, &key).context("failed to write private key")?;
                    (cert, key)
                }
                Err(e) => {
                    panic!("failed to read certificate: {}", e);
                }
            };
            let key = quinn::PrivateKey::from_der(&key)?;
            let cert = quinn::Certificate::from_der(&cert)?;
            server_config.certificate(quinn::CertificateChain::from_certs(vec![cert]), key)?;
        }

    } else {
        // TODO: Record audio, send it to addr
    }

    /*


    let stream = RxStream::new(args.source.parse()?)?;

    let conf = device.default_output_config().unwrap();
    let conf: cpal::StreamConfig = conf.into();
    let s = stream.clone();
    let output_data_fn = move |data: &mut [f32]| {
        unsafe {
            // If we don't zero the buffer, it'll stay at a fixed
            // tone when the stream stops.
            std::ptr::write_bytes(data.as_mut_ptr(), 0, data.len());
        }
        let clock = paradise_core::stream::rx::RxStream::process(&*s, data);
    };
    let output_stream = device.build_output_stream(&conf, output_data_fn, err_fn)?;

    output_stream.play()?;

    println!("{} -> \"{}\"", args.source, device.name().unwrap_or(String::from("NULL")));

    loop {
        std::thread::yield_now();
    }

    println!("shutting down");
    */
    Ok(())
}

fn err_fn(err: cpal::StreamError) {
    eprintln!("an error occurred on stream: {}", err);
}
