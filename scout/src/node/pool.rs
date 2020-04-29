use uuid::Uuid;
use r2d2_redis::{r2d2, redis, RedisConnectionManager};

pub trait PoolTrait {
    fn reserve(&self, uid: Uuid) -> Result<(), ()>;
}

pub struct RedisPool {
    pool: r2d2::Pool<RedisConnectionManager>,
}

impl RedisPool {
    pub fn new() -> Self {
        let manager = RedisConnectionManager::new("redis://localhost").unwrap();
        let pool = r2d2::Pool::builder()
            .build(manager)
            .unwrap();
        Self {
            pool,
        }
    }
}

impl PoolTrait for RedisPool {
    fn reserve(&self, uid: Uuid) -> Result<(), ()> {
        Ok(())
    }
}
