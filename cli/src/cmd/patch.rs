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

    /// Transport protocol.
    /// Valid options: "tcp", "udp", "quic"
    #[clap(long = "protocol", default_value = "udp")]
    protocol: String,

    /// TCP and QUIC only: TLS private key
    #[clap(parse(from_os_str), long = "key")]
    key: Option<PathBuf>,

    /// TCP and QUIC only: TLS public cert
    #[clap(parse(from_os_str), long = "cert")]
    cert: Option<PathBuf>,

    /// QUIC only: enable stateless retry
    #[clap(long = "stateless-retry")]
    stateless_retry: bool,
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
    match args.protocol.as_str() {
        "udp" => {
            if args.cert.is_some() {
                return Err(Error::msg("--cert is not valid with udp"))
            }
            if args.key.is_some() {
                return Err(Error::msg("--key is not valid with udp"))
            }
        },
        _ => return Err(Error::msg("only udp is currently supported")),
    }

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
        (Some(_), Some(_)) => return Err(Error::msg("source and sink cannot be specified at the same time")),
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
        let mut transport_config = quinn::TransportConfig::default();
        transport_config.stream_window_uni(0);
        let mut server_config = quinn::ServerConfig::default();
        server_config.transport = std::sync::Arc::new(transport_config);
        let mut server_config = quinn::ServerConfigBuilder::new(server_config);
        server_config.protocols(ALPN_QUIC_HTTP);
        if args.stateless_retry {
            server_config.use_stateless_retry(true);
        }
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

        let mut endpoint = quinn::Endpoint::builder();
        endpoint.listen(server_config.build());

        let root: PathBuf = ".".into();
        let root = Arc::<Path>::from(root);
        if !root.exists() {
            bail!("root path does not exist");
        }

        let mut incoming = {
            let (endpoint, incoming) = endpoint.bind(&addr)?;
            info!("listening on {}", endpoint.local_addr()?);
            incoming
        };

        while let Some(conn) = incoming.next().await {
            info!("connection incoming");
            tokio::spawn(
                handle_connection(root.clone(), conn).unwrap_or_else(move |e| {
                    error!("connection failed: {reason}", reason = e.to_string())
                }),
            );
        }
        let conf = device.default_output_config().unwrap();
        let conf: cpal::StreamConfig = conf.into();
        let output_data_fn = move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
        };
        let output_stream = device.build_output_stream(&conf, output_data_fn, err_fn)?;
        output_stream.play()?;
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


async fn handle_connection(root: Arc<Path>, conn: quinn::Connecting) -> Result<()> {
    let quinn::NewConnection {
        connection,
        mut bi_streams,
        ..
    } = conn.await?;
    let span = info_span!(
        "connection",
        remote = %connection.remote_address(),
        protocol = %connection
            .authentication_data()
            .protocol
            .map_or_else(|| "<none>".into(), |x| String::from_utf8_lossy(&x).into_owned())
    );
    async {
        info!("established");

        // Each stream initiated by the client constitutes a new request.
        while let Some(stream) = bi_streams.next().await {
            let stream = match stream {
                Err(quinn::ConnectionError::ApplicationClosed { .. }) => {
                    info!("connection closed");
                    return Ok(());
                }
                Err(e) => {
                    return Err(e);
                }
                Ok(s) => s,
            };
            tokio::spawn(
                handle_request(root.clone(), stream)
                    .unwrap_or_else(move |e| error!("failed: {reason}", reason = e.to_string()))
                    .instrument(info_span!("request")),
            );
        }
        Ok(())
    }
        .instrument(span)
        .await?;
    Ok(())
}

async fn handle_request(
    root: Arc<Path>,
    (mut send, recv): (quinn::SendStream, quinn::RecvStream),
) -> Result<()> {
    let req = recv
        .read_to_end(64 * 1024)
        .await
        .map_err(|e| anyhow!("failed reading request: {}", e))?;
    let mut escaped = String::new();
    for &x in &req[..] {
        let part = ascii::escape_default(x).collect::<Vec<_>>();
        escaped.push_str(str::from_utf8(&part).unwrap());
    }
    info!(content = %escaped);
    // Execute the request
    let resp = process_get(&root, &req).unwrap_or_else(|e| {
        error!("failed: {}", e);
        format!("failed to process request: {}\n", e)
            .into_bytes()
            .into()
    });
    // Write the response
    send.write_all(&resp)
        .await
        .map_err(|e| anyhow!("failed to send response: {}", e))?;
    // Gracefully terminate the stream
    send.finish()
        .await
        .map_err(|e| anyhow!("failed to shutdown stream: {}", e))?;
    info!("complete");
    Ok(())
}

fn process_get(root: &Path, x: &[u8]) -> Result<Box<[u8]>> {
    if x.len() < 4 || &x[0..4] != b"GET " {
        bail!("missing GET");
    }
    if x[4..].len() < 2 || &x[x.len() - 2..] != b"\r\n" {
        bail!("missing \\r\\n");
    }
    let x = &x[4..x.len() - 2];
    let end = x.iter().position(|&c| c == b' ').unwrap_or_else(|| x.len());
    let path = str::from_utf8(&x[..end]).context("path is malformed UTF-8")?;
    let path = Path::new(&path);
    let mut real_path = PathBuf::from(root);
    let mut components = path.components();
    match components.next() {
        Some(path::Component::RootDir) => {}
        _ => {
            bail!("path must be absolute");
        }
    }
    for c in components {
        match c {
            path::Component::Normal(x) => {
                real_path.push(x);
            }
            x => {
                bail!("illegal component in path: {:?}", x);
            }
        }
    }
    let data = fs::read(&real_path).context("failed reading file")?;
    Ok(data.into())
}