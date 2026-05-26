//! MIG-056 §A — FederationWarning + FederationError types.
//!
//! ## skip_unavailable model
//!
//! Per Architect §5.2 (Boss-locked decision), cUniverse failures
//! surface as **non-fatal warnings** rather than fatal errors. A
//! missing / locked / corrupt / schema-drifted cUniverse is skipped;
//! the federated query continues with the available cUniverses; a
//! `FederationWarning` is appended to the `FederationContext`.
//!
//! `FederationError` is reserved for **parent-side** issues that
//! prevent federation from even starting (e.g., own DB missing,
//! `resolve_universe_libraries` fails, Mutex poisoned). These are
//! the only errors the caller needs to propagate.
//!
//! Authority chain (Architect §2):
//! - Elasticsearch CCS `skip_unavailable=true` model — official docs
//! - Lucene MultiSearcher — independent index failure handling
//! - DEVONthink production pattern — Boss confirms this matches his
//!   mental model.

use serde::Serialize;
use std::path::PathBuf;

/// Non-fatal warning emitted when a cUniverse can't participate in
/// federation. Collected in `FederationContext.warnings`, surfaced to
/// the frontend via §H's `federation_get_warnings` Tauri command.
#[derive(Debug, Clone, Serialize)]
pub struct FederationWarning {
    /// The cUniverse's root path (the universe directory, not its
    /// search.db file). String form for JSON-serialization to the
    /// frontend.
    pub cuniverse_path: String,

    /// Human-readable reason (e.g., "search.db missing",
    /// "locked by another process", "schema version 5 below floor 7").
    /// Surfaced verbatim in the frontend popup.
    pub reason: String,

    /// When the warning was emitted (Unix-seconds timestamp).
    /// Kept as i64 (not chrono::DateTime) to avoid serde noise; the
    /// frontend renders via locale-aware formatting.
    pub when_unix: i64,
}

impl FederationWarning {
    /// Build a warning with the current timestamp.
    pub fn new(cuniverse_path: PathBuf, reason: impl Into<String>) -> Self {
        let when_unix = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs() as i64)
            .unwrap_or(0);
        Self {
            cuniverse_path: cuniverse_path.to_string_lossy().into_owned(),
            reason: reason.into(),
            when_unix,
        }
    }
}

/// Parent-side federation errors. cUniverse-specific issues
/// (missing, locked, drifted, corrupt) become `FederationWarning`s
/// inside the `FederationContext` — NOT `FederationError`s.
#[derive(Debug)]
pub enum FederationError {
    /// `crate::universe::resolve_universe_libraries` failed. Usually
    /// means the parent universe's `universe.json` is missing or
    /// malformed — a more fundamental issue than any cUniverse
    /// failure.
    ResolveFailed(String),

    /// `SearchState.federation` Mutex was poisoned by a previous
    /// panic. Should be exceedingly rare; logged + returned but the
    /// app continues with an empty FederationContext.
    LockPoisoned,

    /// An unexpected SQL error during ATTACH / PRAGMA / DETACH on the
    /// MAIN connection. Per-cUniverse SQL errors become
    /// `FederationWarning`s in the context.
    SqlError(String),
}

impl std::fmt::Display for FederationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FederationError::ResolveFailed(msg) => {
                write!(f, "Failed to resolve universe libraries: {}", msg)
            }
            FederationError::LockPoisoned => write!(f, "Federation state Mutex was poisoned"),
            FederationError::SqlError(msg) => write!(f, "Federation SQL error: {}", msg),
        }
    }
}

impl std::error::Error for FederationError {}
