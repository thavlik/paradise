#[macro_use]
extern crate log;

use std::ffi::{c_void, CStr};
use std::path::PathBuf;
use std::os::raw::c_char;
use syslog::{Facility, Formatter3164, BasicLogger};
use log::{SetLoggerError, LevelFilter};
use anyhow::{Result, Error};

fn init_logger() -> Result<()> {
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
    Ok(())
}

#[no_mangle]
pub extern "C" fn rust_initialize_vad(driver_name: *const c_char, driver_path: *const c_char) -> i32 {
    if init_logger().is_err() {
        return 1;
    }

    warn!("hello from a macro, getting driver name");

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
    warn!("{}", &config);
    0
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
