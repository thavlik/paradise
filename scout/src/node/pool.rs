use std::ops::DerefMut;
use std::thread;
use r2d2_redis::{r2d2, redis, RedisConnectionManager};
use uuid::Uuid;

type Result<T> = std::result::Result<T, anyhow::Error>;

pub trait PoolTrait {
    fn reserve(&self, uid: Uuid) -> Result<()>;
}

pub struct RedisPool {
    pool: r2d2::Pool<RedisConnectionManager>,
}

impl RedisPool {
    pub fn new(redis_uri: &str) -> Result<Self> {
        let manager = RedisConnectionManager::new(redis_uri)?;
        let pool = r2d2::Pool::builder()
            .max_size(15)
            .build(manager)?;
        Ok(Self {
            pool,
        })
    }
}

impl PoolTrait for RedisPool {
    fn reserve(&self, uid: Uuid) -> Result<()> {
        let mut conn = self.pool.get()?;
        let reply = redis::cmd("PING").query::<String>(conn.deref_mut()).unwrap();
        Ok(())
    }
}
