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

impl PoolTrait for MockPool {
    fn claim(&self, resource: Uuid, claimant: Uuid, expire: Option<SystemTime>) -> Result<Uuid> {
        Ok(Uuid::new_v4())
    }

    fn release(&self, resource: Uuid, claim: Uuid) -> Result<()> {
        Ok(())
    }
}
