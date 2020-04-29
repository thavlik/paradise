use std::{
    ops::DerefMut,
    time::Duration,
    thread,
};
use uuid::Uuid;

type Result<T> = std::result::Result<T, anyhow::Error>;

pub mod redis;
use crate::pool::redis::*;

///
pub trait PoolTrait {
    ///
    fn claim(&self, resource: Uuid, claimant: Uuid, expire: Option<Duration>) -> Result<Uuid>;

    ///
    fn release(&self, resource: Uuid) -> Result<()>;
}
