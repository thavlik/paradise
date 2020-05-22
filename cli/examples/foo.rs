





use quinn::{
    ServerConfig,
    ServerConfigBuilder,
    Endpoint,
    TransportConfig,
    CertificateChain,
    PrivateKey,
    Certificate,
};
use std::{
    io,
    net::SocketAddr,
    sync::{Arc},
    fs,
};

use anyhow::Context;



#[allow(unused)]
pub const ALPN_QUIC_HTTP: &[&[u8]] = &[b"hq-27"];

async fn socket() {
    // No panic - can find runtime
    tokio::spawn(async move {});

    let mut transport_config = TransportConfig::default();
    transport_config.stream_window_uni(0);
    let mut server_config = ServerConfig::default();
    server_config.transport = Arc::new(transport_config);
    let mut server_config = ServerConfigBuilder::new(server_config);
    server_config.protocols(ALPN_QUIC_HTTP);
    let dirs = directories::ProjectDirs::from("org", "quinn", "quinn-examples").unwrap();
    let path = dirs.data_local_dir();
    let cert_path = path.join("cert.der");
    let key_path = path.join("key.der");
    let (cert, key) = match fs::read(&cert_path).and_then(|x| Ok((x, fs::read(&key_path).unwrap()))) {
        Ok(x) => x,
        Err(ref e) if e.kind() == io::ErrorKind::NotFound => {
            let cert = rcgen::generate_simple_self_signed(vec!["localhost".into()]).unwrap();
            let key = cert.serialize_private_key_der();
            let cert = cert.serialize_der().unwrap();
            fs::create_dir_all(&path).context("failed to create certificate directory").unwrap();
            fs::write(&cert_path, &cert).context("failed to write certificate").unwrap();
            fs::write(&key_path, &key).context("failed to write private key").unwrap();
            (cert, key)
        }
        Err(e) => {
            panic!("failed to read certificate: {}", e);
        }
    };
    let key = PrivateKey::from_der(&key).unwrap();
    let cert = Certificate::from_der(&cert).unwrap();
    server_config.certificate(CertificateChain::from_certs(vec![cert]), key).unwrap();
    let port = portpicker::pick_unused_port().expect("pick port");
    let addr: SocketAddr = format!("127.0.0.1:{}", port).parse().unwrap();
    let mut endpoint = Endpoint::builder();
    endpoint.listen(server_config.build());
    let _incoming = {
        let (_endpoint, incoming) = endpoint.bind(&addr).unwrap(); // PANIC - cannot find runtime
        incoming
    };
    println!("All good!");
}


#[tokio::main]
async fn main() {
    socket().await;
}
