//! Listing users known to a platform.

use exn::Exn;

use crate::{BoxFuture, HarmonyError, PlatformUser};

pub trait ListUsers: Send + Sync {
    fn list_users(&self) -> BoxFuture<'_, Result<Vec<PlatformUser>, Exn<HarmonyError>>>;
}
