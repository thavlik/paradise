use anyhow::{Result, Error};
use cpal::traits::{DeviceTrait, HostTrait};
use std::process::Command;
use std::path::PathBuf;
use uuid::Uuid;

pub struct Device {
    name: String,
    display_name: String,
}

#[cfg(target_os = "macos")]
mod macos {
    use super::*;
    use std::fs;
    use std::io::Write;
    use std::os::unix::fs::PermissionsExt;

    const PLUGIN_PREFIX: &'static str = "paradise-";
    const PLUGIN_PATH: &'static str = "/Library/Audio/Plug-Ins/HAL";

    #[cfg(debug_assertions)]
    mod fixtures {
        pub const INFO_PLIST: &'static str = include_str!("../../../../../../ProxyAudioDevice.driver/Contents/Info.plist");
        pub const LOCALIZABLE_STRINGS: &'static [u8] = include_bytes!("../../../../../../ProxyAudioDevice.driver/Contents/Resources/English.lproj/Localizable.strings");
        pub const CODE_RESOURCES: &'static str = include_str!("../../../../../../ProxyAudioDevice.driver/Contents/_CodeSignature/CodeResources");
        pub const DEVICE_ICON: &'static [u8] = include_bytes!("../../../../../../ProxyAudioDevice.driver/Contents/Resources/DeviceIcon.icns");
        pub const DRIVER_BINARY: &'static [u8] = include_bytes!("../../../../../../ProxyAudioDevice.driver/Contents/MacOS/ProxyAudioDevice");
    }

    #[cfg(not(debug_assertions))]
    mod fixtures {
        pub const INFO_PLIST: &'static str = include_str!("../../../../device/platform/macOS/build/Release/ProxyAudioDevice.driver/Contents/Info.plist");
        pub const LOCALIZABLE_STRINGS: &'static str = include_str!("../../../../device/platform/macOS/build/Release/ProxyAudioDevice.driver/Contents/Resources/English.lproj/Localizable.strings");
        pub const CODE_RESOURCES: &'static str = include_str!("../../../../device/platform/macOS/build/Release/ProxyAudioDevice.driver/Contents/_CodeSignature/CodeResources");
        pub const DEVICE_ICON: &'static [u8] = include_bytes!("../../../../device/platform/macOS/build/Release/ProxyAudioDevice.driver/Contents/Resources/DeviceIcon.icns");
        pub const DRIVER_BINARY: &'static [u8] = include_bytes!("../../../../device/platform/macOS/build/Release/ProxyAudioDevice.driver/Contents/MacOS/ProxyAudioDevice");
    }

    fn driver_path(name: &str) -> String {
        //format!("{}/{}{}.driver", PLUGIN_PATH, PLUGIN_PREFIX, name)
        format!("{}/ProxyAudioDevice.driver", PLUGIN_PATH)
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
            .write_all(fixtures::LOCALIZABLE_STRINGS)?;
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
            return Err(Error::msg(format!("mv command failed with code {:?}", status.code())))
        }
        let cmd = format!("chmod 755 {}/Contents/MacOS/ProxyAudioDevice", &dest);
        let output = Command::new("sudo")
            .arg("sh")
            .arg("-c")
            .arg(&cmd)
            .output()?;
        if !output.status.success() {
            return Err(Error::msg(format!("command '{}' failed with code {:?}", &cmd, output.status.code())))
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
        let status = Command::new("sudo")
            .arg("sh")
            .arg("-c")
            .arg("launchctl kickstart -k system/com.apple.audio.coreaudiod")
            .status()?;
        if status.success() {
            Ok(())
        } else {
            Err(Error::msg(format!("command failed with code {:?}", status.code())))
        }
    }

    fn verify_device(device: &Device) -> Result<()> {
        if !device_exists(&device.name)? {
            return Err(Error::msg(format!("device '{}' does not exist", &device.name)));
        }
        let available_hosts = cpal::available_hosts();
        let mut found = false;
        for host_id in available_hosts {
            let host = cpal::host_from_id(host_id)?;
            for (_, d) in host.devices()?.enumerate() {
                if let Ok(name) = d.name() {
                    if name == device.display_name {
                        // At least one device with the same display name was found.
                        found = true;
                        break;
                    }
                }
            }
        }
        if !found {
            return Err(Error::msg(format!("device '{}' not loaded by CoreAudio", &device.name)));
        }
        Ok(())
    }

    #[cfg(test)]
    mod test {
        use super::*;

        fn test_device_name() -> String {
            format!("test-{}", &Uuid::new_v4().to_string()[..8])
        }

        //#[test]
        //fn restart_core_audio_should_work() {
        //    restart_core_audio().unwrap();
        //}

        #[test]
        fn install_uninstall_should_work() {
            let name = test_device_name();
            // TODO: ensure device with this name does not already exist
            let device = Device{
                name,
            };
            install_device(&device).unwrap();
            restart_core_audio().unwrap();
            verify_device(&device).unwrap();
            //// TODO: test streaming with UDP/QUIC
            remove_device(&device.name).expect("remove");
            //// TODO: ensure device is still streaming
            restart_core_audio().unwrap();
            //// TODO: verify stream is stopped
            assert!(!device_exists(&device.name)?);
            verify_device(&device).expect_err("verify");
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
