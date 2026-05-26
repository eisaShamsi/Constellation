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

/// MIG-056 §C — Auto-migrate errors.
///
/// Returned by `federation::migrate::run_migrations_on` per Architect
/// §5.3. The caller (`attach::attach_all`) turns each variant into a
/// `FederationWarning` (skip_unavailable model) so the user gets a
/// clear explanation of which cUniverse couldn't be brought into
/// federation and why.
#[derive(Debug)]
pub enum MigrationError {
    /// Another process holds an exclusive lock on the cUniverse's
    /// `search.db`. Per Architect §9.3 risk 1 — happens when the
    /// user has the cUniverse open in another Constellation window.
    /// Mitigation: skip + tell the user to close it.
    CUniverseLocked,

    /// Backup copy (search.db → search.db.pre-mig-056.bak) failed.
    /// Usually permission denied or disk full. Per Architect §9.3
    /// risk 2 — we don't proceed without a backup.
    BackupFailed(String),

    /// `crate::search::init_db` returned an error against the
    /// cUniverse's `search.db`. The pre-migration backup has been
    /// restored. Per Architect §9.3 risk 2 — atomic via the backup
    /// restore. Original migration error message in the variant.
    MigrationFailed(String),

    /// Audit log write to `{parent}/.constellation/federation-audit.log`
    /// failed (e.g., parent dir read-only). The migration succeeded
    /// or failed-and-restored normally; we just couldn't log it.
    /// Returned as an error so the caller knows to surface a warning.
    AuditLogFailed(String),

    /// Catastrophic: migration failed AND backup couldn't be
    /// restored. The cUniverse's `search.db` is in an indeterminate
    /// state. The backup at `search.db.pre-mig-056.bak` is still
    /// present (manual recovery possible). Per Architect §9.3
    /// risk 2 — escalates from MigrationFailed when restore also
    /// fails.
    BackupRestoreFailed {
        migration_error: String,
        restore_error: String,
    },
}

impl std::fmt::Display for MigrationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MigrationError::CUniverseLocked => write!(
                f,
                "cUniverse is open in another Constellation window — close it to enable federation"
            ),
            MigrationError::BackupFailed(msg) => {
                write!(f, "pre-migration backup failed (skipped to be safe): {}", msg)
            }
            MigrationError::MigrationFailed(msg) => {
                write!(f, "auto-migration failed; backup restored: {}", msg)
            }
            MigrationError::AuditLogFailed(msg) => {
                write!(f, "audit log write failed: {}", msg)
            }
            MigrationError::BackupRestoreFailed {
                migration_error,
                restore_error,
            } => write!(
                f,
                "CATASTROPHIC: migration failed ({}) AND backup restore failed ({}); manual recovery from .pre-mig-056.bak needed",
                migration_error, restore_error
            ),
        }
    }
}

impl std::error::Error for MigrationError {}
