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
use tracing::{error, info, info_span};
use tracing_futures::Instrument as _;
use paradise_core::Frame;

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

    /// Source network interface, e.g. 0.0.0.0:30000
    /// for all interfaces port 30000. Only defined
    /// when patching from network to a device output.
    #[clap(long = "source")]
    source: String,

    /// QUIC only: enable stateless retry
    #[clap(long = "stateless-retry")]
    stateless_retry: bool,
}

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
            info!("using default host \"{:?}\"", host.id());
            host
        },
    };

    let addr: SocketAddr = args.source.parse()?;

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
        None => match host.default_output_device() {
            Some(device) => {
                println!("using default output device \"{}\"", &device.name().unwrap_or(String::from("NULL")));
                device
            },
            None => return Err(Error::msg(format!("default output device not available"))),
        },
    };

    let mut transport_config = quinn::TransportConfig::default();
    transport_config.stream_window_uni(0);
    let mut server_config = quinn::ServerConfig::default();
    server_config.transport = std::sync::Arc::new(transport_config);
    let mut server_config = quinn::ServerConfigBuilder::new(server_config);
    server_config.protocols(ALPN_QUIC_HTTP);
    if args.stateless_retry {
        server_config.use_stateless_retry(true);
    }

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

    let mut endpoint = quinn::Endpoint::builder();
    endpoint.listen(server_config.build());

    let mut incoming = {
        let (endpoint, incoming) = endpoint.bind(&addr)?;
        info!("listening on {}", endpoint.local_addr()?);
        incoming
    };

    while let Some(conn) = incoming.next().await {
        let quinn::NewConnection {
            connection,
            mut datagrams,
            ..
        } = conn.await?;
        while let Some(data) = datagrams.next().await {
            let frame: Frame = bincode::deserialize(data?.as_ref())?;
        }
    }
    let conf = device.default_output_config().unwrap();
    let conf: cpal::StreamConfig = conf.into();
    let output_data_fn = move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
    };
    let err_fn = |err| eprintln!("an error occurred on stream: {}", err);
    let output_stream = device.build_output_stream(&conf, output_data_fn, err_fn)?;
    output_stream.play()?;

    Ok(())
}
