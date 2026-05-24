//! Per-user installed-model registry. Lives at
//! `app_data_dir/installed_models.json`. The IPC commands
//! (commands.rs) read + write this to track which models the user has
//! installed and which one is currently active.
//!
//! Local-only. Never transmitted; never queried by any external service.

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct InstalledModelsRegistry {
    #[serde(default)]
    pub models: Vec<InstalledModel>,
    /// ID of the currently-active model, if any. `mind_start_turn`
    /// loads this one.
    #[serde(default)]
    pub active_model_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstalledModel {
    pub id: String,
    pub version: String,
    pub display_name: String,
    /// Absolute filesystem path to the assembled GGUF.
    pub file_path: String,
    /// Bytes on disk (after assembly).
    pub size_bytes: u64,
    /// Hex SHA-256 of the assembled file (matches catalog entry's
    /// `final_sha256` after install completes).
    pub sha256: String,
    /// Unix timestamp (seconds) when this install completed.
    pub installed_at_unix: u64,
}

impl InstalledModelsRegistry {
    /// Path of the registry file under the Tauri app-data directory.
    pub fn path_in(app_data_dir: &Path) -> PathBuf {
        app_data_dir.join("installed_models.json")
    }

    /// Load registry from disk; returns empty default if the file is
    /// missing (fresh install). Returns Err on parse failure (alerts
    /// the user to a corrupt registry rather than silently resetting).
    pub fn load_from(app_data_dir: &Path) -> Result<Self, String> {
        let p = Self::path_in(app_data_dir);
        if !p.exists() {
            return Ok(Self::default());
        }
        let raw = std::fs::read_to_string(&p)
            .map_err(|e| format!("read {}: {e}", p.display()))?;
        serde_json::from_str::<Self>(&raw)
            .map_err(|e| format!("parse registry {}: {e}", p.display()))
    }

    /// Persist registry to disk (creates app_data_dir if missing). Atomic-ish:
    /// writes to a `.tmp` sibling, then renames. Not perfectly crash-safe on
    /// Windows pre-NTFS-fixup; matches the existing tempfile pattern used
    /// elsewhere in the codebase (cataloger_reliability.json).
    pub fn save_to(&self, app_data_dir: &Path) -> Result<(), String> {
        std::fs::create_dir_all(app_data_dir)
            .map_err(|e| format!("create {}: {e}", app_data_dir.display()))?;
        let p = Self::path_in(app_data_dir);
        let tmp = p.with_extension("json.tmp");
        let raw = serde_json::to_string_pretty(self)
            .map_err(|e| format!("serialize registry: {e}"))?;
        std::fs::write(&tmp, raw)
            .map_err(|e| format!("write {}: {e}", tmp.display()))?;
        std::fs::rename(&tmp, &p)
            .map_err(|e| format!("rename {} -> {}: {e}", tmp.display(), p.display()))?;
        Ok(())
    }

    /// Insert or replace a model entry by id. Sets `active_model_id` if
    /// no model was active before (first install convenience).
    pub fn upsert(&mut self, model: InstalledModel) {
        let id = model.id.clone();
        if let Some(idx) = self.models.iter().position(|m| m.id == id) {
            self.models[idx] = model;
        } else {
            self.models.push(model);
        }
        if self.active_model_id.is_none() {
            self.active_model_id = Some(id);
        }
    }

    /// Look up by id.
    pub fn get(&self, id: &str) -> Option<&InstalledModel> {
        self.models.iter().find(|m| m.id == id)
    }

    /// Set the active model. Returns Err if the id isn't installed.
    pub fn set_active(&mut self, id: &str) -> Result<(), String> {
        if !self.models.iter().any(|m| m.id == id) {
            return Err(format!("model not installed: {id}"));
        }
        self.active_model_id = Some(id.to_string());
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn fixture_model(id: &str) -> InstalledModel {
        InstalledModel {
            id: id.into(),
            version: "v1".into(),
            display_name: format!("{id} display"),
            file_path: format!("/tmp/{id}.gguf"),
            size_bytes: 100,
            sha256: "deadbeef".into(),
            installed_at_unix: 1_716_500_000,
        }
    }

    #[test]
    fn fresh_registry_is_empty() {
        let td = TempDir::new().unwrap();
        let r = InstalledModelsRegistry::load_from(td.path()).unwrap();
        assert!(r.models.is_empty());
        assert!(r.active_model_id.is_none());
    }

    #[test]
    fn save_then_load_round_trips() {
        let td = TempDir::new().unwrap();
        let mut r = InstalledModelsRegistry::default();
        r.upsert(fixture_model("fanar-1-9b-q4km"));
        r.save_to(td.path()).unwrap();

        let r2 = InstalledModelsRegistry::load_from(td.path()).unwrap();
        assert_eq!(r2.models.len(), 1);
        assert_eq!(r2.models[0].id, "fanar-1-9b-q4km");
        assert_eq!(r2.active_model_id.as_deref(), Some("fanar-1-9b-q4km"));
    }

    #[test]
    fn upsert_replaces_existing() {
        let mut r = InstalledModelsRegistry::default();
        r.upsert(fixture_model("fanar-1-9b-q4km"));
        let mut second = fixture_model("fanar-1-9b-q4km");
        second.size_bytes = 999;
        r.upsert(second);
        assert_eq!(r.models.len(), 1);
        assert_eq!(r.models[0].size_bytes, 999);
    }

    #[test]
    fn upsert_first_install_sets_active() {
        let mut r = InstalledModelsRegistry::default();
        r.upsert(fixture_model("fanar-1-9b-q4km"));
        assert_eq!(r.active_model_id.as_deref(), Some("fanar-1-9b-q4km"));
        // Second install does NOT change active.
        r.upsert(fixture_model("other-model"));
        assert_eq!(r.active_model_id.as_deref(), Some("fanar-1-9b-q4km"));
    }

    #[test]
    fn set_active_rejects_uninstalled_id() {
        let mut r = InstalledModelsRegistry::default();
        r.upsert(fixture_model("fanar-1-9b-q4km"));
        assert!(r.set_active("not-installed").is_err());
        assert!(r.set_active("fanar-1-9b-q4km").is_ok());
    }
}
