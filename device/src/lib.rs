#[macro_use]
extern crate log;

use std::ffi::{c_void, CStr};
use std::path::PathBuf;
use std::os::raw::c_char;
use anyhow::{Result, Error};
use paradise_core::device::Device;
use futures::StreamExt;
use std::{net::SocketAddr, sync::Arc};
use quinn::{ClientConfig, ClientConfigBuilder, Endpoint};

/// Dummy certificate verifier that treats any certificate as valid.
/// NOTE, such verification is vulnerable to MITM attacks, but convenient for testing.
struct SkipServerVerification;

impl SkipServerVerification {
    fn new() -> Arc<Self> {
        Arc::new(Self)
    }
}

impl rustls::ServerCertVerifier for SkipServerVerification {
    fn verify_server_cert(
        &self,
        _roots: &rustls::RootCertStore,
        _presented_certs: &[rustls::Certificate],
        _dns_name: webpki::DNSNameRef,
        _ocsp_response: &[u8],
    ) -> Result<rustls::ServerCertVerified, rustls::TLSError> {
        Ok(rustls::ServerCertVerified::assertion())
    }
}


async fn run_client(server_addr: SocketAddr) -> Result<()> {
    warn!("configuring client");
    let client_cfg = configure_client();

    warn!("building endpoint");
    let mut endpoint_builder = Endpoint::builder();
    endpoint_builder.default_client_config(client_cfg);

    warn!("binding endpoint");
    let (endpoint, _) = endpoint_builder.bind(&"127.0.0.1:0".parse().unwrap())?;

    warn!("connecting to server");
    let quinn::NewConnection { connection, .. } = endpoint
        .connect(&server_addr, "localhost")
        .unwrap()
        .await?;
    warn!("[client] connected: addr={}", connection.remote_address());
    // Dropping handles allows the corresponding objects to automatically shut down
    drop(connection);
    // Make sure the server has a chance to clean up
    endpoint.wait_idle().await;

    Ok(())
}

fn init_logger() -> Result<()> {
    #[cfg(target_os = "macos")]
        {
            use syslog::{Facility, Formatter3164, BasicLogger};
            use log::{SetLoggerError, LevelFilter};
            let formatter = Formatter3164 {
                facility: Facility::LOG_USER,
                hostname: None,
                process: "proxyaudio".into(),
                pid: std::process::id() as _,
            };
            let mut writer = match syslog::unix(formatter) {
                Ok(writer) => writer,
                Err(e) => return Err(Error::msg(format!("{:?}", e))),
            };
            log::set_boxed_logger(Box::new(BasicLogger::new(writer)))
                .map(|()| log::set_max_level(LevelFilter::max()));
        }
    Ok(())
}

fn configure_client() -> ClientConfig {
    let mut cfg = ClientConfigBuilder::default().build();
    let tls_cfg: &mut rustls::ClientConfig = Arc::get_mut(&mut cfg.crypto).unwrap();
    // this is only available when compiled with "dangerous_configuration" feature
    tls_cfg
        .dangerous()
        .set_certificate_verifier(SkipServerVerification::new());
    cfg
}


#[no_mangle]
pub extern "C" fn rust_initialize_vad(driver_name: *const c_char, driver_path: *const c_char) -> i32 {
    if init_logger().is_err() {
        return 1;
    }

    let driver_name = unsafe { CStr::from_ptr(driver_name) }.to_str().unwrap();

    warn!("driver name is {}", driver_name);

    let driver_path = PathBuf::from(unsafe { CStr::from_ptr(driver_path) }.to_str().unwrap());

    warn!("driver path is {:?}", driver_path);

    let config_path = driver_path.join("Contents/Resources/config.yaml");
    warn!("loading config {}", &config_path.to_str().unwrap());
    let config = match std::fs::read_to_string(&config_path) {
        Ok(config) => config,
        Err(e) => {
            error!("failed to load config '{:?}': {:?}", &config_path, e);
            return 1;
        },
    };
    let device: Device = serde_yaml::from_str(&config).unwrap();
    warn!("{:?}", &device);

    warn!("initializing connection");
    let server_addr: SocketAddr = device.endpoints[0].addr.parse().unwrap();
    tokio::runtime::Builder::new()
        .threaded_scheduler()
        .build()
        .unwrap()
        .block_on(async move {
            match run_client(server_addr).await {
                Ok(()) => warn!("client executed successfully"),
                Err(e) => error!("client error: {:?}", e),
            }
        });
    0
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
