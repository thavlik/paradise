use super::*;

pub const RESERVE_LUA: &'static str = include_str!("../../../lua/reserve.lua");

#[test]
fn load() {
    let mut client = redis::Client::open("redis://127.0.0.1:6379").unwrap();
    let hash: String = redis::cmd("SCRIPT")
        .arg("LOAD")
        .arg(RESERVE_LUA)
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
        .arg(RESERVE_LUA)
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
        .arg(RESERVE_LUA)
        .arg(1)
        .arg(&key)
        .arg(&claim)
        .query(&mut client)
        .unwrap();
    assert!(reserved);
    let reserved: bool = redis::cmd("EVAL")
        .arg(RESERVE_LUA)
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
        .arg(RESERVE_LUA)
        .arg(1)
        .arg(&key)
        .arg(&claim)
        .query::<bool>(&mut client)
        .unwrap_err();
    assert_eq!(err.code().unwrap(), "BadClaim");
}