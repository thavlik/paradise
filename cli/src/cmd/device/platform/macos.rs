use anyhow::{anyhow, Error, Result, Context, bail};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use serde::{Deserialize, Serialize};
use std::default::Default;
use std::path::{self, Path, PathBuf};
use std::process::Command;
use uuid::Uuid;
use quinn::{
    ServerConfig,
    ServerConfigBuilder,
    TransportConfig,
    CertificateChain,
    PrivateKey,
    Certificate,
};
use futures::future::{Abortable, AbortHandle, Aborted};
use std::{
    ascii,
    io,
    str,
    net::SocketAddr,
    sync::{Arc, mpsc, Mutex, atomic::{AtomicU64, Ordering}},
    fs,
};
use paradise_core::{Frame, device::{DeviceSpec, Endpoint}};
use crossbeam::channel::{Sender, Receiver};
use std::io::Write;
use std::os::unix::fs::PermissionsExt;
use std::time::{Duration, SystemTime};
use futures::{StreamExt, TryFutureExt};

#[allow(unused)]
pub const ALPN_QUIC_HTTP: &[&[u8]] = &[b"hq-27"];

lazy_static! {
    static ref CORE_AUDIO_LOCK: Mutex<()> = Mutex::new(());
    //static ref LAST_CORE_AUDIO_RESTART: Arc<Mutex<Option<SystemTime>>> = Arc::new(Mutex::new(None));
}

const PLUGIN_PREFIX: &'static str = "paradise-";
const PLUGIN_PATH: &'static str = "/Library/Audio/Plug-Ins/HAL";
const DEVICE_MANUFACTURER: &'static str = "Paradise Project";

#[cfg(debug_assertions)]
mod fixtures {
    pub const INFO_PLIST: &'static str = include_str!("../../../../../device/platform/macOS/build/Debug/ProxyAudioDevice.driver/Contents/Info.plist");
    pub const CODE_RESOURCES: &'static str = include_str!("../../../../../device/platform/macOS/build/Debug/ProxyAudioDevice.driver/Contents/_CodeSignature/CodeResources");
    pub const DEVICE_ICON: &'static [u8] = include_bytes!("../../../../../device/platform/macOS/build/Debug/ProxyAudioDevice.driver/Contents/Resources/DeviceIcon.icns");
    pub const DRIVER_BINARY: &'static [u8] = include_bytes!("../../../../../device/platform/macOS/build/Debug/ProxyAudioDevice.driver/Contents/MacOS/ProxyAudioDevice");
}

#[cfg(not(debug_assertions))]
mod fixtures {
    pub const INFO_PLIST: &'static str = include_str!("../../../../../device/platform/macOS/build/Release/ProxyAudioDevice.driver/Contents/Info.plist");
    pub const CODE_RESOURCES: &'static str = include_str!("../../../../../device/platform/macOS/build/Release/ProxyAudioDevice.driver/Contents/_CodeSignature/CodeResources");
    pub const DEVICE_ICON: &'static [u8] = include_bytes!("../../../../../device/platform/macOS/build/Release/ProxyAudioDevice.driver/Contents/Resources/DeviceIcon.icns");
    pub const DRIVER_BINARY: &'static [u8] = include_bytes!("../../../../../device/platform/macOS/build/Release/ProxyAudioDevice.driver/Contents/MacOS/ProxyAudioDevice");
}

fn driver_path(name: &str) -> String {
    format!("{}/{}{}.driver", PLUGIN_PATH, PLUGIN_PREFIX, name)
}

fn generate_localizable_strings(device: &DeviceSpec) -> String {
    format!(
        r#"DriverName = "{}";
DriverPath = "{}";
DeviceName = "{}";
BoxName = "{}";
ManufacturerName = "{}";
"#,
        &device.name,
        &driver_path(&device.name),
        &device.display_name,
        &device.display_name,
        DEVICE_MANUFACTURER,
    )
}

fn generate_driver(device: &DeviceSpec) -> Result<PathBuf> {
    let path = PathBuf::from(format!(
        "/tmp/{}{}.driver-{}",
        PLUGIN_PREFIX,
        &device.name,
        Uuid::new_v4()
    ));
    fs::create_dir(&path)?;
    fs::create_dir(path.join("Contents"))?;
    fs::create_dir(path.join("Contents/_CodeSignature"))?;
    fs::File::create(path.join("Contents/_CodeSignature/CodeResources"))?
        .write_all(fixtures::CODE_RESOURCES.as_bytes())?;
    fs::File::create(path.join("Contents/Info.plist"))?
        .write_all(fixtures::INFO_PLIST.as_bytes())?;
    fs::create_dir(path.join("Contents/MacOS"))?;
    fs::File::create(path.join("Contents/MacOS/ProxyAudioDevice"))?
        .write_all(fixtures::DRIVER_BINARY)?;
    fs::create_dir(path.join("Contents/Resources"))?;
    fs::File::create(path.join("Contents/Resources/DeviceIcon.icns"))?
        .write_all(fixtures::DEVICE_ICON)?;
    fs::create_dir(path.join("Contents/Resources/English.lproj"))?;
    fs::File::create(path.join("Contents/Resources/English.lproj/Localizable.strings"))?
        .write_all(&generate_localizable_strings(device).into_bytes()[..])?;
    let config: String = serde_yaml::to_string(device)?;
    fs::File::create(path.join("Contents/Resources/config.yaml"))?
        .write_all(&config.into_bytes()[..])?;
    Ok(path)
}

fn device_exists(name: &str) -> Result<bool> {
    match std::fs::metadata(&driver_path(name)) {
        Ok(_) => Ok(true),
        Err(e) => {
            if e.kind() == std::io::ErrorKind::NotFound {
                Ok(false)
            } else {
                Err(e.into())
            }
        }
    }
}

fn install_driver_package(device: &DeviceSpec, path: &PathBuf) -> Result<()> {
    let dest = driver_path(&device.name);
    let status = Command::new("sudo")
        .arg("sh")
        .arg("-c")
        .arg(format!("mv {} {}", path.to_str().unwrap(), &dest))
        .status()?;
    if !status.success() {
        return Err(Error::msg(format!(
            "mv command failed with code {:?}",
            status.code()
        )));
    }
    let cmd = format!("chmod 755 {}/Contents/MacOS/ProxyAudioDevice", &dest);
    let output = Command::new("sudo")
        .arg("sh")
        .arg("-c")
        .arg(&cmd)
        .output()?;
    if !output.status.success() {
        return Err(Error::msg(format!(
            "command '{}' failed with code {:?}",
            &cmd,
            output.status.code()
        )));
    }
    Ok(())
}

// Generates and installs a driver package for the given Device.
// Requires sudo.
pub async fn install_device(device: &DeviceSpec) -> Result<()> {
    if device_exists(&device.name)? {
        return Err(Error::msg(format!(
            "device '{}' already exists",
            &device.name
        )));
    }
    install_driver_package(device, &generate_driver(device)?)
}

// Removes the driver from the system without restarting Core Audio.
// Requires sudo.
pub async fn remove_device(name: &str) -> Result<()> {
    if !device_exists(name)? {
        return Err(Error::msg(format!("device '{}' not found", name)));
    }
    let status = Command::new("sudo")
        .arg("sh")
        .arg("-c")
        .arg(format!("rm -rf {}", driver_path(name)))
        .status()?;
    if status.success() {
        Ok(())
    } else {
        Err(Error::msg(format!(
            "command failed with code {:?}",
            status.code()
        )))
    }
}

pub async fn restart() -> Result<()> {
    restart_core_audio()
}

// Restarts core audio. Requires sudo.
fn restart_core_audio() -> Result<()> {
    //let mut last_restart = LAST_CORE_AUDIO_RESTART.lock().unwrap();
    //if let Some(last_restart) = *last_restart {
    //    let elapsed = SystemTime::now().duration_since(last_restart)?;
    //    let diff = Duration::from_secs(15) - elapsed;
    //    if diff.as_nanos() > 0 {
    //        //std::thread::sleep(diff);
    //    }
    //}
    let status = Command::new("sudo")
        .arg("sh")
        .arg("-c")
        .arg("launchctl kickstart -k system/com.apple.audio.coreaudiod")
        .status()?;
    if status.success() {
        //*last_restart = Some(SystemTime::now());
        std::thread::sleep(Duration::from_secs(10));
        Ok(())
    } else {
        Err(Error::msg(format!(
            "command failed with code {:?}",
            status.code()
        )))
    }
}

#[cfg(test)]
mod test {
    use super::*;

    fn test_device_name() -> String {
        format!("test-{}", &Uuid::new_v4().to_string()[..8])
    }

    // Deletes any residual test drivers in /Library/Audio/Plug-Ins/HAL
    fn cleanup() {
        let status = Command::new("sudo")
            .arg("bash")
            .arg("-c")
            .arg(format!("rm -rf {}/{}*", PLUGIN_PATH, PLUGIN_PREFIX))
            .status()
            .expect("command failed");
        if !status.success() {
            panic!(format!("cleanup failed with code {:?}", status.code()));
        }
    }

    #[test]
    fn install_uninstall() {
        let _l = CORE_AUDIO_LOCK.lock().unwrap();
        cleanup();
        let name = test_device_name();
        assert!(!device_exists(&name).unwrap());
        let device = DeviceSpec {
            display_name: format!("Test Virtual Device ({})", &name),
            name,
            outputs: 2,
            endpoints: vec![Endpoint{
                name: String::from("test"),
                addr: "127.0.0.1:5000".into(),
                insecure: true,
            }],
            ..Default::default()
        };
        install_device(&device).unwrap();
        assert!(device_exists(&device.name).unwrap());
        restart_core_audio().unwrap();
        device.verify().unwrap();
        remove_device(&device.name).expect("remove");
        device.verify().unwrap();
        assert_eq!(false, device_exists(&device.name).unwrap());
        restart_core_audio().unwrap();
        device.verify().expect_err("should not exist");
    }

    async fn handle_connection(conn: quinn::Connecting) -> Result<()> {
        Ok(())
    }

    fn process_get(x: &[u8]) -> Result<Box<[u8]>> {
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
        let mut real_path = PathBuf::from(".");
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

    #[tokio::test(threaded_scheduler)]
    async fn should_connect() {
        /// The device should automatically connect to the output endpoints
        let port = portpicker::pick_unused_port().expect("pick port");
        let addr: SocketAddr = format!("127.0.0.1:{}", port).parse().unwrap();
        let (mut send_conn, recv_conn) = crossbeam::channel::unbounded();
        tokio::spawn(async move {
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
            let mut endpoint = quinn::Endpoint::builder();
            endpoint.listen(server_config.build());
            let mut incoming = {
                let (endpoint, incoming) = endpoint.bind(&addr).unwrap();
                incoming
            };
            while let Some(conn) = incoming.next().await {
                let new_conn = conn.await.expect("failed to accept incoming connection");
                send_conn.send(());
            }
        });
        // Install a virtual audio device that connects to the server
        let _l = CORE_AUDIO_LOCK.lock().unwrap();
        cleanup();
        let name = test_device_name();
        assert!(!device_exists(&name).unwrap());
        let device = DeviceSpec {
            display_name: format!("Test Virtual Device ({})", &name),
            name,
            outputs: 2,
            endpoints: vec![Endpoint{
                name: String::from("test"),
                addr: addr.to_string(),
                insecure: true,
            }],
            ..Default::default()
        };
        install_device(&device).unwrap();
        assert!(device_exists(&device.name).unwrap());
        restart_core_audio().unwrap();
        device.verify().unwrap();

        // Make sure the device automatically connects the test server
        recv_conn.recv_timeout(Duration::from_secs(3))
            .expect("did not receive connection");

        // Remove the driver folder.
        remove_device(&device.name).expect("remove");

        // The device should still be visible to cpal until CoreAudio is restarted
        device.verify().unwrap();

        // The driver directory shouldn't exist anymore
        assert_eq!(false, device_exists(&device.name).unwrap());

        // Restarting CoreAudio causes the stream to stop and device to be fully removed
        restart_core_audio().unwrap();

        device.verify().expect_err("should not exist");
    }

    async fn handle_request(
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
        info!("{:?}", escaped);
        // Execute the request
        let resp = process_get(&req).unwrap_or_else(|e| {
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

    async fn basic_stream_server(
        addr: SocketAddr,
        mut send_conn: Sender<()>,
        send_data: Arc<Mutex<(Sender<()>, bool)>>,
    ) -> Result<()> {
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
        let (cert, key) = match fs::read(&cert_path).and_then(|x| Ok((x, fs::read(&key_path)?))) {
            Ok(x) => x,
            Err(ref e) if e.kind() == io::ErrorKind::NotFound => {
                let cert = rcgen::generate_simple_self_signed(vec!["localhost".into()])?;
                let key = cert.serialize_private_key_der();
                let cert = cert.serialize_der().unwrap();
                fs::create_dir_all(&path).context("failed to create certificate directory")?;
                fs::write(&cert_path, &cert).context("failed to write certificate")?;
                fs::write(&key_path, &key).context("failed to write private key")?;
                (cert, key)
            }
            Err(e) => {
                return Err(e.into());
            }
        };
        let key = PrivateKey::from_der(&key)?;
        let cert = Certificate::from_der(&cert)?;
        server_config.certificate(CertificateChain::from_certs(vec![cert]), key)?;
        let mut endpoint = quinn::Endpoint::builder();
        endpoint.listen(server_config.build());
        let mut incoming = {
            let (endpoint, incoming) = endpoint.bind(&addr)?;
            incoming
        };
        while let Some(conn) = incoming.next().await {
            let quinn::NewConnection {
                connection,
                mut datagrams,
                ..
            } = conn.await.expect("failed to accept incoming connection");
            send_conn.send(())?;
            while let Some(data) = datagrams.next().await {
                let frame: Frame = bincode::deserialize(data?.as_ref())?;
                let mut send_data = send_data.lock().unwrap();
                if let (s, true) = &*send_data {
                    s.send(())?;
                    send_data.1 = false;
                }
            }
        }
        Ok(())
    }

    #[tokio::test(threaded_scheduler)]
    async fn basic_stream() {
        let port = portpicker::pick_unused_port().expect("pick port");
        let addr: SocketAddr = format!("127.0.0.1:{}", port).parse().unwrap();
        let (mut send_conn, recv_conn) = crossbeam::channel::unbounded();
        let (mut send_data, recv_data) = crossbeam::channel::unbounded();
        let send_data = Arc::new(Mutex::new((send_data, true)));
        let _send_data = send_data.clone();
        let (abort_handle, abort_registration) = AbortHandle::new_pair();
        let future = Abortable::new(async move {
            basic_stream_server(addr, send_conn, _send_data).await
        }, abort_registration);
        tokio::spawn(async move {
            // Future should eventually be aborted. For whatever
            // reason, it's not yielding an error. This code works
            // and this discrepancy is trivial.
            assert!(future.await.is_ok());
        });

        // Install a virtual audio device that connects to the server
        let _l = CORE_AUDIO_LOCK.lock().unwrap();
        cleanup();
        let name = test_device_name();
        assert!(!device_exists(&name).unwrap());
        let device = DeviceSpec {
            display_name: format!("Test Virtual Device ({})", &name),
            name,
            outputs: 2,
            endpoints: vec![Endpoint{
                name: String::from("test"),
                addr: addr.to_string(),
                insecure: true,
            }],
            ..Default::default()
        };
        install_device(&device).unwrap();
        assert!(device_exists(&device.name).unwrap());
        restart_core_audio().unwrap();
        device.verify().unwrap();

        // Make sure the device automatically connects the test server
        recv_conn.recv_timeout(Duration::from_secs(10))
            .expect("did not receive connection signal");

        // Initialize an output stream on the device and play some audio
        let handle = device.get_handle().unwrap();
        let counter: Mutex<usize> = Mutex::new(0);
        let output_data_fn = move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
            let mut v = counter.lock().unwrap();
            *v = *v + 1;
            data[0] = *v as _;
        };
        let err_fn = |err: cpal::StreamError| {
            panic!("an error occurred on stream: {}", err);
        };
        let stream_config: cpal::StreamConfig = handle.default_output_config().unwrap().into();
        let output_stream = handle.build_output_stream(&stream_config, output_data_fn, err_fn).unwrap();

        assert!(recv_data.try_recv().is_err());
        output_stream.play().unwrap();

        recv_data.recv_timeout(Duration::from_secs(15))
            .expect("did not receive data");
        send_data.lock().unwrap().1 = true;
        // Remove the driver folder.
        remove_device(&device.name).expect("remove");

        // Verify we are still receiving data.
        // The driver doesn't stop until CoreAudio is restarted.
        recv_data.recv_timeout(Duration::from_secs(5))
            .expect("did not receive data");
        send_data.lock().unwrap().1 = true;

        // The device should still be visible to cpal until CoreAudio is restarted
        device.verify().unwrap();

        // The driver directory shouldn't exist anymore
        assert_eq!(false, device_exists(&device.name).unwrap());

        // Restarting CoreAudio causes the stream to stop and device to be fully removed
        restart_core_audio().unwrap();

        // Verify we are no longer receiving data
        assert!(recv_data.recv_timeout(Duration::from_secs(5)).is_err());
        drop(output_stream);

        device.verify().expect_err("should not exist");
        abort_handle.abort();
    }
}
