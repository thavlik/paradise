use anyhow::{Error, Result};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use serde::{Deserialize, Serialize};
use std::default::Default;
use std::path::PathBuf;
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
use std::{
    io,
    net::SocketAddr,
    sync::{Arc, mpsc},
    fs,
};
use paradise_core::device::{Device, Endpoint};

use anyhow::Context;
use tracing::{error, info, info_span};
use tracing_futures::Instrument as _;

#[allow(unused)]
pub const ALPN_QUIC_HTTP: &[&[u8]] = &[b"hq-27"];

#[tokio::test(threaded_scheduler)]
async fn socket() {
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
    let mut endpoint = quinn::Endpoint::builder();
    endpoint.listen(server_config.build());
    let mut incoming = {
        let (endpoint, incoming) = endpoint.bind(&addr).unwrap();
        incoming
    };
}

#[cfg(target_os = "macos")]
mod macos {
    use super::*;
    use std::fs;
    use std::io::Write;
    use std::os::unix::fs::PermissionsExt;
    use std::sync::{Arc, Mutex};
    use std::time::{Duration, SystemTime};
    use futures::{StreamExt, TryFutureExt};

    lazy_static! {
        static ref CORE_AUDIO_LOCK: Mutex<()> = Mutex::new(());
        //static ref LAST_CORE_AUDIO_RESTART: Arc<Mutex<Option<SystemTime>>> = Arc::new(Mutex::new(None));
    }

    const PLUGIN_PREFIX: &'static str = "paradise-";
    const PLUGIN_PATH: &'static str = "/Library/Audio/Plug-Ins/HAL";
    const DEVICE_MANUFACTURER: &'static str = "Paradise Project";

    #[cfg(debug_assertions)]
    mod fixtures {
        pub const INFO_PLIST: &'static str = include_str!("../../../../device/platform/macOS/build/Debug/ProxyAudioDevice.driver/Contents/Info.plist");
        pub const CODE_RESOURCES: &'static str = include_str!("../../../../device/platform/macOS/build/Debug/ProxyAudioDevice.driver/Contents/_CodeSignature/CodeResources");
        pub const DEVICE_ICON: &'static [u8] = include_bytes!("../../../../device/platform/macOS/build/Debug/ProxyAudioDevice.driver/Contents/Resources/DeviceIcon.icns");
        pub const DRIVER_BINARY: &'static [u8] = include_bytes!("../../../../device/platform/macOS/build/Debug/ProxyAudioDevice.driver/Contents/MacOS/ProxyAudioDevice");
    }

    #[cfg(not(debug_assertions))]
    mod fixtures {
        pub const INFO_PLIST: &'static str = include_str!("../../../../device/platform/macOS/build/Release/ProxyAudioDevice.driver/Contents/Info.plist");
        pub const CODE_RESOURCES: &'static str = include_str!("../../../../device/platform/macOS/build/Release/ProxyAudioDevice.driver/Contents/_CodeSignature/CodeResources");
        pub const DEVICE_ICON: &'static [u8] = include_bytes!("../../../../device/platform/macOS/build/Release/ProxyAudioDevice.driver/Contents/Resources/DeviceIcon.icns");
        pub const DRIVER_BINARY: &'static [u8] = include_bytes!("../../../../device/platform/macOS/build/Release/ProxyAudioDevice.driver/Contents/MacOS/ProxyAudioDevice");
    }

    fn driver_path(name: &str) -> String {
        format!("{}/{}{}.driver", PLUGIN_PATH, PLUGIN_PREFIX, name)
    }

    fn generate_localizable_strings(device: &Device) -> String {
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

    fn generate_driver(device: &Device) -> Result<PathBuf> {
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
            .write_all(&config.into_bytes()[..]);
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

    fn install_driver_package(device: &Device, path: &PathBuf) -> Result<()> {
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
    fn install_device(device: &Device) -> Result<()> {
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
    fn remove_device(name: &str) -> Result<()> {
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
            let device = Device {
                display_name: format!("Test Virtual Device ({})", &name),
                name,
                outputs: 2,
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

        #[tokio::test(threaded_scheduler)]
        async fn basic_stream() {
            let port = portpicker::pick_unused_port().expect("pick port");
            let addr: SocketAddr = format!("127.0.0.1:{}", port).parse().unwrap();
            let (mut send_stop, recv_stop) = crossbeam::channel::unbounded();
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
                    send_conn.send(());
                }
            });

            // Install a virtual audio device that connects to the server
            let _l = CORE_AUDIO_LOCK.lock().unwrap();
            cleanup();
            let name = test_device_name();
            assert!(!device_exists(&name).unwrap());
            let device = Device {
                display_name: format!("Test Virtual Device ({})", &name),
                name,
                outputs: 2,
                endpoints: vec![Endpoint{
                    addr: addr.to_string(),
                    insecure: true,
                }],
                ..Default::default()
            };
            install_device(&device).unwrap();
            assert!(device_exists(&device.name).unwrap());
            restart_core_audio().unwrap();
            device.verify().unwrap();

            //// TODO: The audio driver should connect to the endpoint automatically
            //recv_conn.recv_timeout(Duration::from_secs(3))
            //    .expect("did not receive connection");

            // Initialize an output stream on the device and play some audio
            let handle = device.get_handle().unwrap();
            let output_data_fn = move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
            };
            let err_fn = |err: cpal::StreamError| {
                panic!("an error occurred on stream: {}", err);
            };
            let stream_config: cpal::StreamConfig = handle.default_output_config().unwrap().into();
            let output_stream = handle.build_output_stream(&stream_config, output_data_fn, err_fn).unwrap();
            output_stream.play().unwrap();

            std::thread::sleep(Duration::from_secs(1));
            // TODO: verify device is streaming

            // Remove the driver folder.
            remove_device(&device.name).expect("remove");

            // The device should still be visible to cpal until CoreAudio is restarted
            device.verify().unwrap();

            // The driver directory shouldn't exist anymore
            assert_eq!(false, device_exists(&device.name).unwrap());

            //// TODO: ensure device is still streaming

            // Restarting CoreAudio causes the stream to stop and device to be fully removed
            restart_core_audio().unwrap();
            //// TODO: expect error from stream

            device.verify().expect_err("should not exist");

            send_stop.send(());
        }
    }
}

/// Create a virtual audio device
#[derive(clap::Clap)]
pub struct CreateArgs {
    /// Accept the changes without prompting for user input
    #[clap(short = "y")]
    yes: bool,

    /// Virtual device name
    name: String,

    /// Number of input channels
    #[clap(long = "inputs")]
    inputs: Option<usize>,

    /// Network interfaces on which the device should listen
    #[clap(long = "listen")]
    listeners: Vec<String>,

    /// Number of output channels
    #[clap(long = "outputs")]
    outputs: Option<usize>,

    /// Destination addresses for receiving audio
    #[clap(long = "destination")]
    destinations: Vec<String>,
}

pub async fn main(args: CreateArgs) -> Result<(), Error> {
    Err(Error::msg(format!(
        "name = {}, yes = {}, inputs = {:?}, outputs = {:?}",
        &args.name, args.yes, &args.inputs, &args.outputs
    )))
}
