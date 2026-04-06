//! Listing users known to a platform.

use {exn::Exn, std::sync::Arc};

use crate::{BoxFuture, HarmonyError, PlatformUser};

pub trait ListUsers: Send + Sync {
    fn list_users(&self) -> BoxFuture<'_, Result<Vec<PlatformUser>, Exn<HarmonyError>>>;
}

impl<T: ListUsers> ListUsers for Arc<T> {
    fn list_users(&self) -> BoxFuture<'_, Result<Vec<PlatformUser>, Exn<HarmonyError>>> {
        (**self).list_users()
    }
}
