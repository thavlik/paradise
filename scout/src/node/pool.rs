use std::{
    ops::DerefMut,
    time::Duration,
    thread,
};
use r2d2_redis::{
    r2d2,
    redis,
    RedisConnectionManager,
};
use uuid::Uuid;

type Result<T> = std::result::Result<T, anyhow::Error>;

///
pub trait PoolTrait {
    ///
    fn claim(&self, resource: Uuid, claimant: Uuid, expire: Option<Duration>) -> Result<Uuid>;

    ///
    fn release(&self, resource: Uuid) -> Result<()>;
}

///
pub struct RedisPool {
    pool: r2d2::Pool<RedisConnectionManager>,
}

impl RedisPool {
    pub fn new(redis_uri: &str, max_size: u32) -> Result<Self> {
        let manager = RedisConnectionManager::new(redis_uri)?;
        let pool = r2d2::Pool::builder()
            .max_size(max_size)
            .build(manager)?;
        Ok(Self {
            pool,
        })
    }
}

impl PoolTrait for RedisPool {
    fn claim(&self, resource: Uuid, claimant: Uuid, expire: Option<Duration>) -> Result<Uuid> {
        let mut conn = self.pool.get()?;
        // Generate a novel uid for the claim attempt
        let claim_uid = Uuid::new_v4();
        // TODO: write redis script to claim given uid
        let reply = redis::cmd("PING").query::<String>(conn.deref_mut())?;
        Ok(Uuid::new_v4())
    }

    fn release(&self, resource: Uuid) -> Result<()> {
        Ok(())
    }
}
