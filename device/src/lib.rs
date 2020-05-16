use syslog::{Facility, Formatter3164};
use std::ffi::c_void;

#[no_mangle]
pub extern "C" fn rust_initialize_vad(vad: *const c_void) -> i32 {
    let formatter = Formatter3164 {
        facility: Facility::LOG_USER,
        hostname: None,
        process: "myprogram".into(),
        pid: 0,
    };

    match syslog::unix(formatter) {
        Err(e) => println!("impossible to connect to syslog: {:?}", e),
        Ok(mut writer) => {
            match std::fs::read_to_string("../Resources/config.yaml") {
                Ok(config) => writer.info(&config).unwrap(),
                Err(e) => writer.err(format!("failed to load config: {:?}, pwd={}", e, std::env::current_dir().unwrap().to_str().unwrap())).unwrap(),
            }
        }
    }

    0
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
