//! Listing users known to a platform.

use std::error::Error;

use crate::{BoxFuture, PlatformUser};

pub trait ListUsers: Send + Sync {
    fn list_users(&self) -> BoxFuture<'_, Result<Vec<PlatformUser>, Box<dyn Error + Send + Sync>>>;
}
