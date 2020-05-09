use super::*;


#[cfg(test)]
mod reserve {
    use super::*;

    const SCRIPT: &'static str = include_str!("../../lua/reserve.lua");

    #[test]
    fn load() {
        let mut client = redis::Client::open("redis://127.0.0.1:6379").unwrap();
        let hash: String = redis::cmd("SCRIPT")
            .arg("LOAD")
            .arg(SCRIPT)
            .query(&mut client)
            .unwrap();
        assert_eq!(hash.len(), 40);
    }

    #[test]
    fn freshly_reserved() {
        let resource = Uuid::new_v4().to_string();
        let claim = Uuid::new_v4().to_string();
        let key = rk::claim(&resource);
        let mut client = redis::Client::open("redis://127.0.0.1:6379").unwrap();
        let reserved: bool = redis::cmd("EVAL")
            .arg(SCRIPT)
            .arg(1)
            .arg(&key)
            .arg(&claim)
            .query(&mut client)
            .unwrap();
        assert!(reserved);
    }

    #[test]
    fn already_reserved() {
        let resource = Uuid::new_v4().to_string();
        let claim = Uuid::new_v4().to_string();
        let key = rk::claim(&resource);
        let mut client = redis::Client::open("redis://127.0.0.1:6379").unwrap();
        let reserved: bool = redis::cmd("EVAL")
            .arg(SCRIPT)
            .arg(1)
            .arg(&key)
            .arg(&claim)
            .query(&mut client)
            .unwrap();
        assert!(reserved);
        let reserved: bool = redis::cmd("EVAL")
            .arg(SCRIPT)
            .arg(1)
            .arg(&key)
            .arg(&claim)
            .query(&mut client)
            .unwrap();
        assert!(!reserved);
    }

    #[test]
    fn err_bad_claim() {
        let resource = Uuid::new_v4().to_string();
        let claim = Uuid::new_v4().to_string();
        let key = rk::claim(&resource);
        let mut client = redis::Client::open("redis://127.0.0.1:6379").unwrap();
        let _: () = redis::cmd("SET")
            .arg(&key)
            .arg("foobarbaz")
            .query(&mut client)
            .unwrap();
        let err = redis::cmd("EVAL")
            .arg(SCRIPT)
            .arg(1)
            .arg(&key)
            .arg(&claim)
            .query::<bool>(&mut client)
            .unwrap_err();
        assert_eq!(err.code().unwrap(), "BadClaim");
    }
}

#[cfg(test)]
mod release {
    use super::*;

    const SCRIPT: &'static str = include_str!("../../lua/release.lua");

    #[test]
    fn load() {
        let mut client = redis::Client::open("redis://127.0.0.1:6379").unwrap();
        let hash: String = redis::cmd("SCRIPT")
            .arg("LOAD")
            .arg(SCRIPT)
            .query(&mut client)
            .unwrap();
        assert_eq!(hash.len(), 40);
    }

    #[test]
    fn ok() {
        let resource = Uuid::new_v4().to_string();
        let claim = Uuid::new_v4().to_string();
        let key = rk::claim(&resource);
        let mut client = redis::Client::open("redis://127.0.0.1:6379").unwrap();
        let _: () = redis::cmd("SET")
            .arg(&key)
            .arg(&claim)
            .query(&mut client)
            .unwrap();
        let _: () = redis::cmd("EVAL")
            .arg(SCRIPT)
            .arg(1)
            .arg(&key)
            .arg(&claim)
            .query(&mut client)
            .unwrap();
        let result: Option<String> = redis::cmd("GET")
            .arg(&key)
            .query(&mut client)
            .unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn err_no_claim() {
        let resource = Uuid::new_v4().to_string();
        let claim = Uuid::new_v4().to_string();
        let key = rk::claim(&resource);
        let mut client = redis::Client::open("redis://127.0.0.1:6379").unwrap();
        let err = redis::cmd("EVAL")
            .arg(SCRIPT)
            .arg(1)
            .arg(&key)
            .arg(&claim)
            .query::<()>(&mut client)
            .unwrap_err();
        assert_eq!(err.code().unwrap(), "NoClaim");
    }

    #[test]
    fn err_bad_claim() {
        let resource = Uuid::new_v4().to_string();
        let claim = Uuid::new_v4().to_string();
        let key = rk::claim(&resource);
        let mut client = redis::Client::open("redis://127.0.0.1:6379").unwrap();
        let _: () = redis::cmd("SET")
            .arg(&key)
            .arg("foobarbaz")
            .query(&mut client)
            .unwrap();
        let err = redis::cmd("EVAL")
            .arg(SCRIPT)
            .arg(1)
            .arg(&key)
            .arg(&claim)
            .query::<()>(&mut client)
            .unwrap_err();
        assert_eq!(err.code().unwrap(), "BadClaim");
    }
}
