//! Designed error type for Harmony.
//!
//! `HarmonyError` is a flat struct categorized by what the caller can do
//! (kind) and whether it is safe to retry (status). Internal enums are
//! not public: callers interact via `is_xxx()` query methods.

use core::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ErrorKind {
    ConnectionFailed,
    SendFailed,
    DiscoveryFailed,
    ConfigInvalid,
    Internal,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ErrorStatus {
    Permanent,
    Temporary,
}

#[derive(Debug)]
pub struct HarmonyError {
    kind: ErrorKind,
    status: ErrorStatus,
    message: String,
}

impl HarmonyError {
    pub fn connection(message: impl Into<String>) -> Self {
        Self {
            kind: ErrorKind::ConnectionFailed,
            status: ErrorStatus::Temporary,
            message: message.into(),
        }
    }

    pub fn send(message: impl Into<String>) -> Self {
        Self {
            kind: ErrorKind::SendFailed,
            status: ErrorStatus::Temporary,
            message: message.into(),
        }
    }

    pub fn discovery(message: impl Into<String>) -> Self {
        Self {
            kind: ErrorKind::DiscoveryFailed,
            status: ErrorStatus::Temporary,
            message: message.into(),
        }
    }

    pub fn config(message: impl Into<String>) -> Self {
        Self {
            kind: ErrorKind::ConfigInvalid,
            status: ErrorStatus::Permanent,
            message: message.into(),
        }
    }

    pub fn internal(message: impl Into<String>) -> Self {
        Self {
            kind: ErrorKind::Internal,
            status: ErrorStatus::Permanent,
            message: message.into(),
        }
    }

    #[must_use]
    pub const fn temporary(mut self) -> Self {
        self.status = ErrorStatus::Temporary;
        self
    }

    #[must_use]
    pub const fn permanent(mut self) -> Self {
        self.status = ErrorStatus::Permanent;
        self
    }

    #[must_use]
    pub const fn is_connection_failure(&self) -> bool {
        matches!(self.kind, ErrorKind::ConnectionFailed)
    }

    #[must_use]
    pub const fn is_send_failure(&self) -> bool {
        matches!(self.kind, ErrorKind::SendFailed)
    }

    #[must_use]
    pub const fn is_discovery_failure(&self) -> bool {
        matches!(self.kind, ErrorKind::DiscoveryFailed)
    }

    #[must_use]
    pub const fn is_config_invalid(&self) -> bool {
        matches!(self.kind, ErrorKind::ConfigInvalid)
    }

    #[must_use]
    pub const fn is_internal(&self) -> bool {
        matches!(self.kind, ErrorKind::Internal)
    }

    #[must_use]
    pub const fn is_temporary(&self) -> bool {
        matches!(self.status, ErrorStatus::Temporary)
    }

    #[must_use]
    pub fn message(&self) -> &str {
        &self.message
    }
}

impl fmt::Display for HarmonyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let status = if self.is_temporary() {
            "temporary"
        } else {
            "permanent"
        };
        write!(f, "{} ({})", self.message, status)
    }
}

impl std::error::Error for HarmonyError {}
