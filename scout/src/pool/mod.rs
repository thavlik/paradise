use std::{
    time::SystemTime,
};
use uuid::Uuid;
use serde::{Serialize, Deserialize};

type Result<T> = std::result::Result<T, anyhow::Error>;

pub mod mock;
pub mod redis;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Claim {
    pub uid: Uuid,
    pub resource: Uuid,
    pub claimant: Uuid,
    pub expire: Option<SystemTime>,
}

///
pub trait PoolTrait {
    ///
    fn claim(&self, resource: Uuid, claimant: Uuid, expire: Option<SystemTime>) -> Result<Uuid>;

    ///
    fn release(&self, resource: Uuid, claim: Uuid) -> Result<()>;
}
