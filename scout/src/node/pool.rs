use uuid::Uuid;
use r2d2_redis::{r2d2, redis, RedisConnectionManager};

type Result<T> = std::result::Result<T, anyhow::Error>;

pub trait PoolTrait {
    fn reserve(&self, uid: Uuid) -> Result<()>;
}

pub struct RedisPool {
    pool: r2d2::Pool<RedisConnectionManager>,
}

impl RedisPool {
    pub fn new(redis_uri: &str) -> Self {
        let manager = RedisConnectionManager::new(redis_uri).unwrap();
        let pool = r2d2::Pool::builder()
            .build(manager)
            .unwrap();
        Self {
            pool,
        }
    }
}

impl PoolTrait for RedisPool {
    fn reserve(&self, uid: Uuid) -> Result<()> {
        Ok(())
    }zs
}
