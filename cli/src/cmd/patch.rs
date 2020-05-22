use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use std::{
    ascii, fs, io,
    net::SocketAddr,
    path::{self, Path, PathBuf},
    str,
    sync::Arc,
};
use futures::future::{Abortable, AbortHandle, Aborted};
use crossbeam::{Receiver, Sender};
use anyhow::{anyhow, bail, Error, Context, Result};
use futures::{StreamExt, TryFutureExt};
use paradise_core::Frame;
use ringbuf::{RingBuffer, Producer};

const LATENCY_MS: f32 = 300.0;

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

fn get_device(name: &Option<String>, host: &cpal::Host) -> Result<cpal::Device> {
    match name {
        Some(name) => {
            match name.parse::<usize>() {
                Ok(index) => {
                    if index >= host.devices()?.count() {
                        return Err(anyhow!("device index out of range (tip: run info)"));
                    }
                    match host.devices()?
                        .skip(index)
                        .next() {
                        Some(device) => {
                            info!("found device {}. \"{}\"", &name, device.name().unwrap_or(String::from("NULL")));
                            Ok(device)
                        },
                        None => Err(anyhow!("device at index \"{}\" not found (tip: run info)", &name)),
                    }
                },
                _ => match host.devices()?
                    .enumerate()
                    .find(|(_, d)| match d.name() {
                        Ok(n) => &n == name,
                        _ => false,
                    }) {
                    Some((_, d)) => {
                        info!("found device \"{}\"", &name);
                        Ok(d)
                    },
                    None => Err(anyhow!("device \"{}\" not found", name)),
                },
            }
        },
        None => match host.default_output_device() {
            Some(device) => {
                info!("using default output device \"{}\"", &device.name().unwrap_or(String::from("NULL")));
                Ok(device)
            },
            None => Err(anyhow!("default output device not available")),
        },
    }
}

fn get_host(name: &Option<String>) -> Result<cpal::Host> {
    match name {
        Some(name) => {
            let host = crate::util::get_host_by_name(name)?;
            info!("found host \"{}\"", name);
            Ok(host)
        },
        None => {
            let host = cpal::default_host();
            info!("using default host \"{:?}\"", host.id());
            Ok(host)
        },
    }
}

pub async fn main(args: PatchArgs) -> Result<()> {
    let host = get_host(&args.host)?;
    let device = get_device(&args.device, &host)?;
    let addr: SocketAddr = args.source.parse()?;
    let config: cpal::StreamConfig = device.default_output_config()?.into();
    let latency_frames = (LATENCY_MS / 1_000.0) * config.sample_rate.0 as f32;
    let latency_samples = latency_frames as usize * config.channels as usize;
    let ring = RingBuffer::new(latency_samples * 2);
    let (mut producer, mut consumer) = ring.split();
    let (abort_handle, abort_registration) = AbortHandle::new_pair();
    let future = Abortable::new(async move {
        server_entry(addr, producer).await
    }, abort_registration);
    tokio::spawn(async move {
        // Future should eventually be aborted. For whatever
        // reason, it's not yielding an error. This code works
        // and this discrepancy is trivial.
        // TODO: make sure server exiting with error results in error
        assert!(future.await.is_err());
    });
    let _guard = scopeguard::guard((), move |_| {
        abort_handle.abort();
    });
    let conf = device.default_output_config().unwrap();
    let conf: cpal::StreamConfig = conf.into();
    let output_data_fn = move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
        let mut input_fell_behind = None;
        for sample in data {
            *sample = match consumer.pop() {
                Ok(s) => s,
                Err(err) => {
                    input_fell_behind = Some(err);
                    0.0
                }
            };
        }
        if let Some(err) = input_fell_behind {
            error!(
                "input stream fell behind: {:?}: try increasing latency",
                err
            );
        }
    };
    let err_fn = |err| error!("an error occurred on stream: {}", err);
    let output_stream = device.build_output_stream(&conf, output_data_fn, err_fn)?;
    output_stream.play()?;
    Ok(())
}

async fn server_entry(addr: SocketAddr, mut producer: Producer<f32>) -> Result<()> {
    let mut transport_config = quinn::TransportConfig::default();
    transport_config.stream_window_uni(0);
    let mut server_config = quinn::ServerConfig::default();
    server_config.transport = std::sync::Arc::new(transport_config);
    let mut server_config = quinn::ServerConfigBuilder::new(server_config);
    server_config.protocols(ALPN_QUIC_HTTP);
    let dirs = directories::ProjectDirs::from("org", "quinn", "quinn-examples").unwrap();
    let path = dirs.data_local_dir();
    let cert_path = path.join("cert.der");
    let key_path = path.join("key.der");
    let (cert, key) = match fs::read(&cert_path).and_then(|x| Ok((x, fs::read(&key_path)?))) {
        Ok(x) => x,
        Err(ref e) if e.kind() == io::ErrorKind::NotFound => {
            info!("generating self-signed certificate");
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
            //connection,
            mut datagrams,
            ..
        } = conn.await?;
        while let Some(data) = datagrams.next().await {
            let frame: Frame = bincode::deserialize(data?.as_ref())?;
            // TODO: verify timestamp
            if frame.buffer.len() % 4 != 0 {
                return Err(anyhow!("encountered buffer with non-divisible by four length"))
            }
            let samples = unsafe { std::slice::from_raw_parts(frame.buffer.as_ptr() as *const f32, frame.buffer.len()/4) };
            producer.push_slice(samples).map_err(|e| anyhow!("{:?}", e))?;
        }
    }
    Ok(())
}