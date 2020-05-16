use anyhow::{Result, Error};
use cpal::traits::{DeviceTrait, HostTrait};
use std::process::Command;
use std::path::PathBuf;
use uuid::Uuid;

pub struct Device {
    name: String,
    display_name: String,
}

impl Device {
    fn verify(&self) -> Result<()> {
        let available_hosts = cpal::available_hosts();
        for host_id in available_hosts {
            let host = cpal::host_from_id(host_id)?;
            for (_, d) in host.devices()?.enumerate() {
                // custom built proxy-audio-device works
                // paradise device with rust lib does not
                if let Ok(name) = d.name() {
                    if name == self.display_name {
                        // At least one device with the same display name was found.
                        // TODO: verify input config
                        // TODO: verify output config
                        return Ok(());
                    }
                }
            }
        }
        return Err(Error::msg(format!("device '{}' not loaded by CoreAudio", &self.name)));
    }
}

#[cfg(target_os = "macos")]
mod macos {
    use super::*;
    use std::fs;
    use std::io::Write;
    use std::os::unix::fs::PermissionsExt;
    use std::sync::{Mutex, Arc};
    use std::time::{SystemTime, Duration};

    lazy_static! {
        static ref CORE_AUDIO_LOCK: Mutex<()> = Mutex::new(());
        static ref LAST_CORE_AUDIO_RESTART: Arc<Mutex<Option<SystemTime>>> = Arc::new(Mutex::new(None));
    }

    const PLUGIN_PREFIX: &'static str = "paradise-";
    const PLUGIN_PATH: &'static str = "/Library/Audio/Plug-Ins/HAL";
    const DEVICE_MANUFACTURER: &'static str = "Paradise Project";

    #[cfg(debug_assertions)]
    mod fixtures {
        pub const INFO_PLIST: &'static str = include_str!("/Users/thomashavlik/ProxyAudioDevice.driver/Contents/Info.plist");
        pub const CODE_RESOURCES: &'static str = include_str!("/Users/thomashavlik/ProxyAudioDevice.driver/Contents/_CodeSignature/CodeResources");
        pub const DEVICE_ICON: &'static [u8] = include_bytes!("/Users/thomashavlik/ProxyAudioDevice.driver/Contents/Resources/DeviceIcon.icns");
        pub const DRIVER_BINARY: &'static [u8] = include_bytes!("/Users/thomashavlik/ProxyAudioDevice.driver/Contents/MacOS/ProxyAudioDevice");
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
        //format!("{}/ProxyAudioDevice.driver", PLUGIN_PATH)
    }

    fn generate_localizable_strings(device: &Device) -> String {
        format!(r#"DeviceName = "{}";
BoxName = "{}";
ManufacturerName = "{}";
"#, &device.display_name, &device.display_name, DEVICE_MANUFACTURER)
    }

    fn generate_driver(device: &Device) -> Result<PathBuf> {
        let path = PathBuf::from(format!("/tmp/{}{}.driver-{}", PLUGIN_PREFIX, &device.name, Uuid::new_v4()));
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
        Ok(path)
    }

    fn device_exists(name: &str) -> Result<bool> {
        match std::fs::metadata(&driver_path(name)) {
            Ok(_) => Ok(true),
            Err(e) => if e.kind() == std::io::ErrorKind::NotFound {
                Ok(false)
            } else {
                Err(e.into())
            },
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
            return Err(Error::msg(format!("mv command failed with code {:?}", status.code())));
        }
        let cmd = format!("chmod 755 {}/Contents/MacOS/ProxyAudioDevice", &dest);
        let output = Command::new("sudo")
            .arg("sh")
            .arg("-c")
            .arg(&cmd)
            .output()?;
        if !output.status.success() {
            return Err(Error::msg(format!("command '{}' failed with code {:?}", &cmd, output.status.code())));
        }
        Ok(())
    }

    // Generates and installs a driver package for the given Device.
    // Requires sudo.
    fn install_device(device: &Device) -> Result<()> {
        if device_exists(&device.name)? {
            return Err(Error::msg(format!("device '{}' already exists", &device.name)));
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
            Err(Error::msg(format!("command failed with code {:?}", status.code())))
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
            Err(Error::msg(format!("command failed with code {:?}", status.code())))
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

        #[test]
        fn basic_stream() {
            let _l = CORE_AUDIO_LOCK.lock().unwrap();
            cleanup();
            let name = test_device_name();
            assert!(!device_exists(&name).unwrap());
            let device = Device {
                display_name: format!("Test Virtual Device ({})", &name),
                name,
            };
            install_device(&device).unwrap();
            assert!(device_exists(&device.name).unwrap());
            restart_core_audio().unwrap();
            device.verify().unwrap();
            //// TODO: create output stream to ProxyAudioDevice and verify exact audio can be received
            remove_device(&device.name).expect("remove");
            device.verify().unwrap();
            assert_eq!(false, device_exists(&device.name).unwrap());
            //// TODO: ensure device is still streaming
            restart_core_audio().unwrap();
            //// TODO: verify stream is stopped
            device.verify().expect_err("should not exist");
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
