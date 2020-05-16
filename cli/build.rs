use std::path::PathBuf;
use std::process::Command;

fn main() {
    let crate_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let configuration = match std::env::var("PROFILE").unwrap().as_str() {
        "debug" => "Debug",
        "release" => "Release",
        cfg => panic!("unknown configuration profile '{}'", cfg),
    };
    let platform_dir = PathBuf::from(crate_dir).join("../device/platform/macOS");
    let scheme = "ProxyAudioDevice";
    let target = "ProxyAudioDevice";
    let status = Command::new("bash")
        .arg("-c")
        .arg(format!("cd {} && xcodebuild -configuration {} -scheme {} -target {} -verbose", platform_dir.to_str().unwrap(), configuration, scheme, target))
        .status()
        .expect("failed to run command");
    if !status.success() {
        panic!(format!("xcodebuild failed with code {:?}", status.code()));
    }
}
