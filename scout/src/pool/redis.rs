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
    release_hash: String,
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
            release_hash: format!(""),
        })
    }
}

impl PoolTrait for RedisPool {
    fn claim(&self, resource: Uuid, claimant: Uuid, expire: Option<SystemTime>) -> Result<Uuid> {
        let mut conn = self.pool.get()?;
        {
            let mut pubsub = conn.deref_mut().as_pubsub();
            pubsub.subscribe("claim")?;
            loop {
                let payload = pubsub.get_message()?;
                let payload: Vec<u8> = payload.get_payload()?;
                let payload: Claim = bincode::deserialize(&payload[..])?;
                // TODO: propogate writes
            }
        }

        // Generate a novel UUID for the claim
        let uid = Uuid::new_v4();

        // Attempt to claim the resource in redis
        // TODO: write redis tests for these commands
        let mut cmd = redis::cmd("SET")
            .arg(resource.as_bytes())
            .arg(uid.as_bytes());
        if let Some(expire) = expire {
            cmd = cmd.arg("PX")
                .arg(expire.duration_since(SystemTime::now())?
                    .as_millis()
                    .to_string());
        }
        let _: () = cmd
            .arg("NX")
            .query::<()>(conn.deref_mut())?;

        // TODO: save claim info to persistent storage

        // Announce the claim over redis
        let encoded: Vec<u8> = bincode::serialize(&Claim {
            uid,
            resource,
            claimant,
            expire,
        }).unwrap();
        redis::cmd("PUBLISH")
            .arg("claim")
            .arg(encoded)
            .query(conn.deref_mut())?;

        Ok(uid)
    }

    fn release(&self, resource: Uuid, claim: Uuid) -> Result<()> {
        let mut conn = self.pool.get()?;

        // TODO: delete the redis key if the claim uid matches
        redis::cmd("EVALSHA")
            .arg(&self.release_hash)
            .arg("1") // num_keys
            .arg(resource.as_bytes())
            .arg(claim.as_bytes())
            .query(conn.deref_mut())?;

        // Announce that the resource has been released over redis
        redis::cmd("PUBLISH")
            .arg("release")
            .arg(resource.as_bytes())
            .query(conn.deref_mut())?;

        Ok(())
    }
}
