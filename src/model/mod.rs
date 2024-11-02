mod base;
mod error;
mod store;
pub mod task;
pub mod user;

pub use self::error::{Error, Result};
use store::{new_db_pool, Db};

#[derive(Clone)]
pub struct ModelManager {
    db: Db,
}

impl ModelManager {
    pub async fn new() -> Result<Self> {
        let db = new_db_pool().await?;

        Ok(Self { db })
    }

    pub(in crate::model) fn db(&self) -> &Db {
        &self.db
    }
}
