use super::*;

///
pub struct MockPool {
}

impl MockPool {
    pub fn new() -> Self {
        Self {
        }
    }
}

impl PoolTrait for RedisPool {
    fn claim(&self, resource: Uuid, claimant: Uuid, expire: Option<Duration>) -> Result<Uuid> {
        Ok(Uuid::new_v4())
    }

    fn release(&self, resource: Uuid) -> Result<()> {
        Ok(())
    }
}
