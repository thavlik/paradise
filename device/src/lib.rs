#![feature(panic_info_message)]
#![feature(weak_into_raw)]
#[macro_use]
extern crate log;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate futures;
use crossbeam::channel::{Sender, Receiver};
use std::{ptr, ffi::{c_void, CStr}};
use std::path::PathBuf;
use std::os::raw::c_char;
use anyhow::{Result, Error};
use paradise_core::device::DeviceSpec;
use futures::StreamExt;
use std::{net::SocketAddr, sync::{Arc, Weak, Mutex}};
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

    warn!("building endpoint...");
    let mut endpoint_builder = Endpoint::builder();
    endpoint_builder.default_client_config(client_cfg);

    let addr = "127.0.0.1:0".parse()?;
    warn!("binding endpoint {}", &addr);
    let (endpoint, _) = endpoint_builder.bind(&addr)?;

    warn!("connecting to server...");
    let quinn::NewConnection { connection, .. } = endpoint
        .connect(&server_addr, "localhost")?
        .await?;

    warn!("[client] connected: addr={}", connection.remote_address());

    // TODO: move connection ref into struct

    // Dropping handles allows the corresponding objects to automatically shut down
    drop(connection);
    // Make sure the server has a chance to clean up
    endpoint.wait_idle().await;

    Ok(())
}

async fn connect_with_retry(server_addr: SocketAddr) -> Result<(quinn::Endpoint, quinn::Connection)> {
    loop {
        match connect(server_addr.clone()).await {
            Ok(v) => return Ok(v),
            Err(e) => {
                error!("error connecting to {}: {}", &server_addr, e);
            }
        }
    }
}

async fn connect(server_addr: SocketAddr) -> Result<(quinn::Endpoint, quinn::Connection)> {
    warn!("configuring client");
    let client_cfg = configure_client();

    warn!("building endpoint...");
    let mut endpoint_builder = Endpoint::builder();
    endpoint_builder.default_client_config(client_cfg);

    let addr = "127.0.0.1:0".parse()?;
    warn!("binding endpoint {}", &addr);
    let (endpoint, _) = endpoint_builder.bind(&addr)?;

    warn!("connecting to server...");
    let quinn::NewConnection { connection, .. } = endpoint
        .connect(&server_addr, "localhost")?
        .await?;

    warn!("[client] connected: addr={}", connection.remote_address());

    Ok((endpoint, connection))
}

fn init_logger() -> Result<()> {
    std::panic::set_hook(Box::new(|panic_info| {
        let location = if let Some(location) = panic_info.location() {
            format!("{}", location)
        } else {
            format!("unknown")
        };
        let message = if let Some(message) = panic_info.message() {
            format!("{}", message)
        } else {
            format!("(no message available)")
        };
        error!("panic occurred [{}]: {}", location, message);
    }));
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

lazy_static! {
    static ref RUNTIME: Arc<Mutex<tokio::runtime::Runtime>> = Arc::new(Mutex::new(tokio::runtime::Builder::new()
        .threaded_scheduler()
        .enable_all()
        .build()
        .unwrap()));
}

async fn endpoint_entry(server_addr: SocketAddr, stop: Receiver<()>) -> Result<()> {
    run_client(server_addr).await;
    Err(Error::msg("exited prematurely"))
}

async fn listen_for_stop(stop: Receiver<()>) -> Result<()> {
    stop.recv().unwrap();
    Err(Error::msg("exited prematurely"))
}

async fn driver_entry(driver: Arc<Driver>, stop: Receiver<()>) -> Result<()> {
    if driver.spec.endpoints.len() == 0 {
        return Err(Error::msg("no endpoints"));
    }

    // TODO: reconnect logic
    //let mut f = vec![];
    //let mut stops = vec![];
    for endpoint in &driver.spec.endpoints {
        let server_addr = endpoint.addr.parse()?;
        connect_with_retry(server_addr);

        //f.push(connect(server_addr));
        //f.push(endpoint_entry(server_addr, r));
        //stops.push(s);
    }

    //let mut results = futures::future::join_all(f).await;

    //tokio::spawn(async move {
    //    stop.recv().unwrap();
    //    stops.into_iter()
    //        .for_each(|s| {
    //            s.send(()).unwrap()
    //        });
    //});

    //tokio::spawn(async move {
    //    futures::future::try_join_all(f).await;
    //});

    Ok(())
}

pub struct Output {
    pub name: String,
    pub conn: quinn::Connection,
    pub endpoint: quinn::Endpoint,
}

pub struct Driver {
    // TODO: spec for inputs and outputs
    outputs: Vec<Output>,
    spec: DeviceSpec,
    stop: Mutex<Sender<()>>,
}

impl Driver {
    /// Attempt to connect to the output. If this method fails
    /// with an error, it will be immediately retried.
    /// TODO: conceptualize retry config, backoff
    fn connect_with_retry(&self, server_addr: SocketAddr) {
        let results = tokio::spawn(async move {
            connect(server_addr).await
        });
        // TODO: lock self.outputs connections vec and add the item
        Ok(())
    }

    fn io_proc(&self, buffer: &[u8], sample_time: f64) -> Result<()> {
        // TODO: package up buffer and sample_time into message
        // TODO: send packaged message over QUIC clients
        //for output in &self.outputs {
        //    output.conn.send_datagram(data)?;
        //}
        Ok(())
    }

    fn stop(&self) {
        self.stop.lock()
            .unwrap()
            .send(())
            .unwrap();
    }
}

#[no_mangle]
pub extern "C" fn rust_io_proc(driver: *mut c_void, buffer: *const u8, buffer_size: u32, sample_time: f64) {
    let driver: Arc<Driver> = match unsafe {
        Weak::from_raw(driver as _)
    }.upgrade() {
        Some(driver) => driver,
        None => {
            error!("ioproc: driver is deallocated");
            return;
        },
    };
    match driver.io_proc(unsafe {
        std::slice::from_raw_parts(buffer, buffer_size as _)
    }, sample_time) {
        Err(e) => {
            error!("ioproc: {:?}", e)
        },
        _ => {},
    }
}

#[no_mangle]
pub extern "C" fn rust_new_driver(driver_name: *const c_char, driver_path: *const c_char) -> *mut c_void {
    if init_logger().is_err() {
        return ptr::null_mut();
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
            return ptr::null_mut();
        },
    };
    let spec: DeviceSpec = serde_yaml::from_str(&config).unwrap();
    warn!("{:?}", &spec);
    warn!("initializing runtime");
    let (stop_send, stop_recv) = crossbeam::channel::bounded(1);
    let driver = Arc::new(Driver {
        spec,
        stop: Mutex::new(stop_send),
        outputs: vec![],
    });
    let retval = Weak::into_raw(Arc::downgrade(&driver));
    let (ready_send, ready_recv) = crossbeam::channel::bounded(1);
    RUNTIME.clone()
        .lock()
        .unwrap()
        .block_on(async move {
            ready_send.send(driver_entry(driver, stop_recv).await);
        });
    match ready_recv.recv() {
        Ok(result) => match result {
            Ok(()) => {
                warn!("device has signaled ready state");
                retval as _
            },
            Err(e) => {
                error!("failed to initialize: {:?}", e);
                ptr::null_mut()
            },
        },
        Err(e) => {
            error!("ready channel error: {:?}", e);
            ptr::null_mut()
        },
    }
}

#[no_mangle]
pub extern "C" fn rust_stop_driver(driver: *mut c_void) {
    let driver: Arc<Driver> = match unsafe {
        Weak::from_raw(driver as _)
    }.upgrade() {
        Some(driver) => driver,
        None => {
            error!("rust_release_driver: driver is already deallocated");
            return;
        },
    };
    info!("stopping driver for '{}'", &driver.spec.name);
    driver.stop()
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
