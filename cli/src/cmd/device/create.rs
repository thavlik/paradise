use anyhow::{Result, Error};
use cpal::traits::{DeviceTrait, HostTrait};
use std::process::Command;
use std::path::PathBuf;
use uuid::Uuid;

pub struct Device {
    name: String,
}

#[cfg(target_os = "macos")]
mod macos {
    use super::*;
    use std::fs;
    use std::io::Write;

    const PLUGIN_PREFIX: &'static str = "paradise-";
    const PLUGIN_PATH: &'static str = "/Library/Audio/Plug-Ins/HAL";


    #[cfg(debug_assertions)]
    mod fixtures {
        pub const INFO_PLIST: &'static str = include_str!("../../../../device/platform/macOS/build/Debug/ProxyAudioDevice.driver/Contents/Info.plist");
        pub const LOCALIZABLE_STRINGS: &'static [u8] = include_bytes!("../../../../device/platform/macOS/build/Debug/ProxyAudioDevice.driver/Contents/Resources/English.lproj/Localizable.strings");
        pub const CODE_RESOURCES: &'static str = include_str!("../../../../device/platform/macOS/build/Debug/ProxyAudioDevice.driver/Contents/_CodeSignature/CodeResources");
    }

    #[cfg(not(debug_assertions))]
    mod fixtures {
        pub const INFO_PLIST: &'static str = include_str!("../../../../device/platform/macOS/build/Release/ProxyAudioDevice.driver/Contents/Info.plist");
        pub const LOCALIZABLE_STRINGS: &'static str = include_str!("../../../../device/platform/macOS/build/Release/ProxyAudioDevice.driver/Contents/Resources/English.lproj/Localizable.strings");
        pub const CODE_RESOURCES: &'static str = include_str!("../../../../device/platform/macOS/build/Release/ProxyAudioDevice.driver/Contents/_CodeSignature/CodeResources");
    }

    fn driver_path(name: &str) -> String {
        format!("{}/{}{}.driver", PLUGIN_PATH, PLUGIN_PREFIX, name)
    }

    fn generate_driver(device: &Device) -> Result<PathBuf> {
        // ProxyAudioDevice.driver/Contents
        // ProxyAudioDevice.driver/Contents/_CodeSignature
        // ProxyAudioDevice.driver/Contents/_CodeSignature/CodeResources
        // ProxyAudioDevice.driver/Contents/MacOS
        // ProxyAudioDevice.driver/Contents/MacOS/ProxyAudioDevice
        // ProxyAudioDevice.driver/Contents/Resources
        // ProxyAudioDevice.driver/Contents/Resources/DeviceIcon.icns
        // ProxyAudioDevice.driver/Contents/Resources/English.lproj
        // ProxyAudioDevice.driver/Contents/Resources/English.lproj/Localizable.strings
        // ProxyAudioDevice.driver/Contents/Info.plist
        let path = PathBuf::from(format!("/tmp/{}{}.driver-{}", PLUGIN_PREFIX, &device.name, Uuid::new_v4()));
        fs::create_dir(&path)?;
        fs::create_dir(path.join("Contents"))?;
        fs::create_dir(path.join("Contents/MacOS/_CodeSignature"))?;
        fs::create_dir(path.join("Contents/MacOS"))?;
        fs::create_dir(path.join("Contents/MacOS/Resources"))?;
        fs::create_dir(path.join("Contents/MacOS/Resources/English.lproj"))?;
        fs::File::create(path.join("Contents/Info.plist"))?
            .write_all(fixtures::INFO_PLIST.as_bytes())?;
        fs::File::create(path.join("Contents/_CodeSignature/CodeResources"))?
            .write_all(fixtures::CODE_RESOURCES.as_bytes())?;
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
        let status = Command::new("sudo")
            .arg("sh")
            .arg("-c")
            .arg(format!("mv {} {}", path.to_str().unwrap(), driver_path(&device.name)))
            .status()?;
        if status.success() {
            Ok(())
        } else {
            Err(Error::msg(format!("command failed with code {:?}", status.code())))
        }
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

    #[cfg(test)]
    mod test {
        use super::*;

        fn test_device_name() -> String {
            format!("test-{}", Uuid::new_v4().to_string()[..8])
        }

        #[test]
        fn restart_core_audio_should_work() {
            restart_core_audio().unwrap();
        }

        #[test]
        fn install_uninstall_should_work() {
            let name = test_device_name();
            // TODO: ensure device with this name does not already exist
            let device = Device{
                name,
            };
            install_device(&device).unwrap();
            restart_core_audio().unwrap();
            // TODO: verify device was installed correctly using cpal
            // TODO: test streaming with UDP/QUIC
            remove_device(&device.name);
            // TODO: ensure device is still streaming
            restart_core_audio().unwrap();
            // TODO: verify stream is stopped
            // TODO: verify device was removed correctly using cpal
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
