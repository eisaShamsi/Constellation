//! Model installation infrastructure for Constellation Mind.
//!
//! MIG-047 Phase 0b Step E. Provides the Rust side of the first-launch
//! model-download UX:
//!
//! 1. Reads the bundled `models.json` catalog (Tauri resource) listing
//!    the installable models with their URLs + final SHA-256.
//! 2. On `mind_install_model`, fetches the per-release `manifest.json`,
//!    downloads each split-chunk in order with progress events,
//!    verifies per-part and final SHA-256, assembles into a single
//!    GGUF in `app_data_dir/models/`.
//! 3. Maintains a per-user `installed_models.json` registry (id,
//!    version, file path, installed timestamp, active flag).
//! 4. Exposes `mind_list_installed_models`, `mind_active_model`, and
//!    `mind_set_active_model` so the Settings → Mind UI (Step F) and
//!    the `mind_start_turn` refactor (Step G) can discover what's
//!    installed and pick which one to use for the next turn.
//!
//! Per Architect §4 D + §10.5 Q3 override: distribution is GitHub
//! Releases with file-splitting; this module does NOT call any
//! Constellation-operated cloud service. All HTTP fetches target
//! `https://github.com` or `https://objects.githubusercontent.com`
//! (GitHub's CDN-served asset URLs).
//!
//! Per Architect §3 invariant 7 (Local-First / no exfiltration): the
//! installed-models registry is local-only. No telemetry of which
//! models a user installs is sent anywhere.

pub mod commands;
pub mod download;
pub mod manifest;
pub mod registry;
pub mod verify;

pub use commands::{
    mind_active_model, mind_install_model, mind_list_installed_models, mind_set_active_model,
};
pub use download::DownloadProgress;
pub use manifest::{ModelEntry, ModelsCatalog, ReleaseManifest, ReleasePart};
pub use registry::{InstalledModel, InstalledModelsRegistry};
