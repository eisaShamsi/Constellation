//! MIG-056 — Cross-Universe Federation module.
//!
//! Federates SQLite `search.db` files across the active universe and
//! its cUniverse children. Per Architect v1.0.
//!
//! ## Layered architecture
//!
//! - **failure.rs** — `FederationWarning` (skip_unavailable model) +
//!   `FederationError` (parent-side fatal errors). §A.
//! - **attach.rs** — boot-time ATTACH logic; user_version check;
//!   auto-migrate hook. §B (added in next commit).
//! - **migrate.rs** — auto-migration of schema-drifted cUniverses
//!   per §5.3. §C (highest-risk step; added in step C).
//! - **query.rs** — federated query helpers (per-schema SELECT
//!   builder, UNION ALL composer, outer ORDER BY / LIMIT). §D.
//! - **tests.rs** — `#[cfg(test)] mod tests` covering §A-§I.
//!
//! ## Public API (stable across the cascade)
//!
//! - `FederationContext` — per-boot federation state, stored in
//!   `SearchState.federation`. Holds attached cUniverses + warnings.
//! - `FederationWarning` — non-fatal cUniverse failure, surfaced to
//!   the frontend via the §H `federation_get_warnings` command.
//! - `FederationError` — parent-side fatal error type.
//!
//! ## Boss-locked decisions baked in
//!
//! - **§5.1** — Four consumers federate (lens / status bar /
//!   libraryStats / global search). Each adopts the context in
//!   §E/§F/§G respectively.
//! - **§5.2** — skip_unavailable model. cUniverse failures become
//!   warnings, not errors.
//! - **§5.3** — Auto-migrate cUniverses below the schema floor on
//!   first federated attach (§C ships the migrate helper with 4
//!   safeguards: lock check, backup, atomic txn, audit log).
//! - **§5.4** — ATTACH cap raised from SQLite default 10 to **25**
//!   at compile time. Enforced in §B's `attach_all`.

pub mod failure;

#[cfg(test)]
mod tests;

pub use failure::{FederationError, FederationWarning};

use std::path::PathBuf;

/// Per-boot federation state, stored in `SearchState.federation`.
///
/// Lifecycle:
/// - Created with `FederationContext::new()` (empty, not ready).
/// - Populated by `attach::attach_all` (§B) which:
///   - Reads `resolve_universe_libraries` to find cUniverses
///   - For each cUniverse: ATTACHes its search.db read-only,
///     enforces user_version floor (§C auto-migrate if drifted),
///     tunes cache_size
///   - Sets `ready = true` when done
///   - cUniverses that fail to attach become `warnings` entries
///     (skip_unavailable model)
/// - Read by federated query builders in §D-§G (`build_sql` for
///   lens; `get_all_library_stats` for status bar / sidebar; the
///   `constellation_search_*` family for global search).
/// - Invalidated on universe switch (resets to `new()` so the next
///   `attach_all` builds fresh state for the new universe).
///
/// Public methods are append-only / read-only — there's no
/// "remove an attached cUniverse mid-session" operation in v1.
/// Universe switches reset the whole context.
#[derive(Debug, Clone)]
pub struct FederationContext {
    /// Attached cUniverses. Each entry: `(schema_alias, cuniverse_root_path)`.
    /// `schema_alias` is the SQL identifier used in
    /// `cu0.note_meta`, `cu1.note_meta`, etc. — guaranteed alphanumeric
    /// per `attach.rs`'s alias generation rule.
    attached: Vec<(String, PathBuf)>,

    /// Non-fatal warnings. A cUniverse that's missing / locked /
    /// corrupt / schema-drifted produces a `FederationWarning` here
    /// and the federation continues without it (skip_unavailable
    /// model — Architect §5.2 / Agents 2 + 3).
    warnings: Vec<FederationWarning>,

    /// `false` until `attach_all` completes. Federated query consumers
    /// check this flag — if `false`, they fall back to active-universe-
    /// only behavior. Prevents races during the background-attach
    /// window (Architect §6.3).
    ready: bool,
}

impl FederationContext {
    /// Create an empty, not-ready context. Stored in `SearchState`
    /// before any boot work has been done.
    pub fn new() -> Self {
        Self {
            attached: Vec::new(),
            warnings: Vec::new(),
            ready: false,
        }
    }

    /// `true` once `attach_all` has finished and federated queries
    /// can use the context. Consumers fall back to active-only when
    /// this is `false`.
    pub fn is_ready(&self) -> bool {
        self.ready
    }

    /// Slice of `(schema_alias, cuniverse_root_path)` for cUniverses
    /// that successfully attached. The `schema_alias` strings are
    /// safe to interpolate into SQL (alphanumeric only).
    pub fn attached(&self) -> &[(String, PathBuf)] {
        &self.attached
    }

    /// Slice of warnings — cUniverses that didn't attach. Surfaced
    /// to the frontend via the §H `federation_get_warnings` command.
    pub fn warnings(&self) -> &[FederationWarning] {
        &self.warnings
    }

    /// Append an attached cUniverse. Called by `attach::attach_all`
    /// after a successful ATTACH + user_version check.
    pub fn add_attached(&mut self, alias: String, path: PathBuf) {
        self.attached.push((alias, path));
    }

    /// Append a warning. Called by `attach::attach_all` for every
    /// cUniverse that doesn't successfully attach (skip_unavailable
    /// model — Architect §5.2).
    pub fn warn(&mut self, path: PathBuf, reason: impl Into<String>) {
        self.warnings.push(FederationWarning::new(path, reason));
    }

    /// Mark the context as ready. Called once at the end of
    /// `attach::attach_all`.
    pub fn set_ready(&mut self, ready: bool) {
        self.ready = ready;
    }

    /// Clear the context — called on universe switch to reset state
    /// before the new universe's `attach_all` runs.
    #[allow(dead_code)] // Wired by §B (`attach_all`) + universe switch hook.
    pub fn reset(&mut self) {
        self.attached.clear();
        self.warnings.clear();
        self.ready = false;
    }
}

impl Default for FederationContext {
    fn default() -> Self {
        Self::new()
    }
}
