mod error;

use error::{Error, Result};

#[derive(Clone, Debug)]
pub struct Ctx {
    user_id: i64,
}

impl Ctx {
    pub fn new(user_id: i64) -> Result<Self> {
        if user_id == 0 {
            return Err(Error::CtxCannotNewRootCtx);
        }
        Ok(Self { user_id })
    }

    pub fn root_ctx() -> Self {
        Self { user_id: 0 }
    }

    pub fn user_id(&self) -> i64 {
        self.user_id
    }
}
