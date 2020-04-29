use super::*;
use std::ops::DerefMut;
use r2d2_redis::{
    r2d2,
    redis,
    RedisConnectionManager,
};

///
pub struct RedisPool {
    pool: r2d2::Pool<RedisConnectionManager>,
}

impl RedisPool {
    pub fn new(redis_uri: &str) -> Result<Self> {
        let manager = RedisConnectionManager::new(redis_uri)?;
        /// https://docs.rs/r2d2/0.8.8/r2d2/struct.Builder.html
        let pool = r2d2::Pool::builder()
            .max_size(32)
            .build(manager)?;
        Ok(Self {
            pool,
        })
    }
}

impl PoolTrait for RedisPool {
    fn claim(&self, resource: Uuid, claimant: Uuid, expire: Option<SystemTime>) -> Result<Uuid> {
        let mut conn = self.pool.get()?;
        {
            let mut pubsub = conn.deref_mut().as_pubsub();
            pubsub.subscribe("paradise")?;
            loop {
                let msg = pubsub.get_message()?;
                let payload : String = msg.get_payload()?;
                println!("channel '{}': {}", msg.get_channel_name(), payload);
            }
        }

        // Generate a novel uid for the claim attempt
        let claim = Claim {
            uid: Uuid::new_v4(),
            resource,
            claimant,
            expire: None,
        };

        // TODO: write redis script to claim given uid
        let mut cmd = redis::cmd("SET");
        if let Some(expire) = expire {
            //cmd = *cmd.arg("PX")
            //    .arg(expire.as_millis().to_string())
        }
        let _: () = cmd
            .arg("NX")
            .query::<()>(conn.deref_mut())?;
        // TODO: announce claim over redis

        redis::cmd("PUBLISH")
            .arg("paradise")
            .arg("")
            .query(conn.deref_mut())?;
        Ok(Uuid::new_v4())
    }

    fn release(&self, resource: Uuid) -> Result<()> {
        let mut conn = self.pool.get()?;
        Ok(())
    }
}
