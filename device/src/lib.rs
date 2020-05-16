use syslog::{Facility, Formatter3164};
use std::ffi::{c_void, CStr};
use std::path::PathBuf;
use std::os::raw::c_char;

#[no_mangle]
pub extern "C" fn rust_initialize_vad(vad: *const c_void, driver_name: *const c_char, driver_path: *const c_char) -> i32 {
    let driver_name = unsafe { CStr::from_ptr(driver_name) }.to_str().unwrap();
    let driver_path = PathBuf::from(unsafe { CStr::from_ptr(driver_path) }.to_str().unwrap());
    let formatter = Formatter3164 {
        facility: Facility::LOG_USER,
        hostname: None,
        process: driver_name.into(),
        pid: 0,
    };
    let mut writer = match syslog::unix(formatter) {
        Ok(writer) => writer,
        Err(e) => {
            println!("impossible to connect to syslog: {:?}", e);
            return 1;
        },
    };
    let config_path = driver_path.join("Contents/Resources/config.yaml");
    let config = match std::fs::read_to_string(&config_path) {
        Ok(config) => config,
        Err(e) => {
            writer.err(format!("failed to load config '{:?}': {:?}", &config_path, e));
            return 1;
        },
    };
    writer.info(&config);
    0
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
