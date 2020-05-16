use syslog::{Facility, Formatter3164};

#[no_mangle]
pub extern "C" fn rust_function() {
    let formatter = Formatter3164 {
        facility: Facility::LOG_USER,
        hostname: None,
        process: "myprogram".into(),
        pid: 0,
    };

    match syslog::unix(formatter) {
        Err(e) => println!("impossible to connect to syslog: {:?}", e),
        Ok(mut writer) => {
            writer.err("hello world").expect("could not write error message");
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
