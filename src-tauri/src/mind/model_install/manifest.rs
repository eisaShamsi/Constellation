//! Manifest types: the bundled `models.json` catalog and the per-release
//! `manifest.json` produced by the model-pipeline workflow (Step A).

use serde::{Deserialize, Serialize};

// ─── models.json — the bundled catalog of installable models ────────

/// Top-level shape of `src-tauri/resources/models.json`.
///
/// Fields prefixed with `$` are documentation aids (`$schema_version`,
/// `$doc`, `$todo`, `$comment_sha256`); serde's `#[serde(default)]` +
/// `#[serde(other)]` would let us ignore them, but since we use `serde_json`
/// directly to parse, unknown fields are dropped silently.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelsCatalog {
    pub models: Vec<ModelEntry>,
}

/// One installable model in the catalog.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelEntry {
    pub id: String,
    pub version: String,
    pub display_name: String,
    #[serde(default)]
    pub display_name_ar: String,
    pub description: String,
    #[serde(default)]
    pub description_ar: String,
    pub model_family: String,
    pub quantization: String,
    pub language_focus: Vec<String>,
    pub context_window: u32,
    pub license: String,
    pub license_notes_url: String,
    pub manifest_url: String,
    pub release_url: String,
    /// Hex-encoded SHA-256 of the concatenated (un-split) GGUF.
    /// Starts with `TBD-` before the first model-pipeline workflow run;
    /// `install` refuses to proceed while this is the case (avoids
    /// shipping an unverifiable download).
    pub final_sha256: String,
    /// Total byte size of the concatenated GGUF; 0 when sha256 starts with `TBD-`.
    pub final_size_bytes: u64,
}

impl ModelEntry {
    /// True if this entry is fully populated and safe to install.
    pub fn is_ready_to_install(&self) -> bool {
        !self.final_sha256.starts_with("TBD") && self.final_size_bytes > 0
    }
}

// ─── manifest.json — the per-release manifest published alongside parts ──

/// Shape of the `manifest.json` the model-pipeline workflow uploads
/// alongside each split-chunk. Downloaded at install time to discover
/// the parts list + per-part hashes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReleaseManifest {
    pub name: String,
    pub version: String,
    pub display_name: String,
    pub model_family: String,
    pub quantization: String,
    pub upstream_repo: String,
    #[serde(default)]
    pub upstream_base_model: String,
    pub language_focus: Vec<String>,
    pub context_window: u32,
    pub license: String,
    #[serde(default)]
    pub license_notes: String,
    pub release_tag: String,
    pub release_url: String,
    pub final_sha256: String,
    pub final_size_bytes: u64,
    pub parts: Vec<ReleasePart>,
    #[serde(default)]
    pub llama_cpp_tag: String,
    #[serde(default)]
    pub produced_by: String,
}

/// One split-chunk in the per-release manifest.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReleasePart {
    /// The leaf filename of the part (e.g. `fanar-1-9b-q4km.gguf.part-aa`).
    pub name: String,
    /// Hex-encoded SHA-256 of this part's bytes.
    pub sha256: String,
    pub size_bytes: u64,
}

impl ReleaseManifest {
    /// Build the absolute download URL for a part by replacing the
    /// leaf `manifest.json` in `manifest_url` with the part's name.
    ///
    /// Both URLs share the same GH Release-asset prefix:
    /// `https://github.com/<owner>/<repo>/releases/download/<tag>/<filename>`.
    pub fn part_url(manifest_url: &str, part: &ReleasePart) -> String {
        match manifest_url.rsplit_once('/') {
            Some((prefix, _leaf)) => format!("{prefix}/{}", part.name),
            None => part.name.clone(), // pathological; would fail at fetch
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn models_catalog_parses_bundled_json() {
        // The bundled resource itself is the spec; round-trip parse.
        let raw = include_str!("../../../resources/models.json");
        let parsed: ModelsCatalog =
            serde_json::from_str(raw).expect("models.json round-trips");
        assert!(!parsed.models.is_empty());
        // First model is Fanar.
        let fanar = &parsed.models[0];
        assert_eq!(fanar.id, "fanar-1-9b-q4km");
        assert_eq!(fanar.quantization, "Q4_K_M");
        assert!(fanar.language_focus.contains(&"ar".to_string()));
    }

    #[test]
    fn model_entry_not_ready_while_sha_is_tbd() {
        let raw = include_str!("../../../resources/models.json");
        let parsed: ModelsCatalog = serde_json::from_str(raw).unwrap();
        let fanar = &parsed.models[0];
        // Before first workflow run, sha256 starts with "TBD-".
        assert!(fanar.final_sha256.starts_with("TBD"));
        assert!(!fanar.is_ready_to_install());
    }

    #[test]
    fn release_manifest_round_trips() {
        let m = ReleaseManifest {
            name: "fanar-1-9b-q4km".into(),
            version: "v1".into(),
            display_name: "Fanar 1.9B (Q4_K_M)".into(),
            model_family: "fanar".into(),
            quantization: "Q4_K_M".into(),
            upstream_repo: "https://huggingface.co/QCRI/Fanar-1-9B-Instruct".into(),
            upstream_base_model: "google/gemma-2-9b".into(),
            language_focus: vec!["ar".into(), "en".into()],
            context_window: 8192,
            license: "Apache-2.0".into(),
            license_notes: "Continued pretraining of google/gemma-2-9b".into(),
            release_tag: "models/fanar-1-9b-q4km-v1".into(),
            release_url: "https://github.com/x/y/releases/tag/models/fanar-1-9b-q4km-v1".into(),
            final_sha256: "abc123".into(),
            final_size_bytes: 5_234_567_890,
            parts: vec![
                ReleasePart {
                    name: "fanar-1-9b-q4km.gguf.part-aa".into(),
                    sha256: "aa11".into(),
                    size_bytes: 1_782_579_200,
                },
                ReleasePart {
                    name: "fanar-1-9b-q4km.gguf.part-ab".into(),
                    sha256: "bb22".into(),
                    size_bytes: 1_782_579_200,
                },
            ],
            llama_cpp_tag: "b6285".into(),
            produced_by: "Constellation model-pipeline workflow".into(),
        };
        let s = serde_json::to_string(&m).unwrap();
        let back: ReleaseManifest = serde_json::from_str(&s).unwrap();
        assert_eq!(back.parts.len(), 2);
        assert_eq!(back.parts[0].name, "fanar-1-9b-q4km.gguf.part-aa");
    }

    #[test]
    fn part_url_replaces_leaf() {
        let manifest_url =
            "https://github.com/x/y/releases/download/models/fanar-1-9b-q4km-v1/manifest.json";
        let part = ReleasePart {
            name: "fanar-1-9b-q4km.gguf.part-aa".into(),
            sha256: "x".into(),
            size_bytes: 1,
        };
        let url = ReleaseManifest::part_url(manifest_url, &part);
        assert_eq!(
            url,
            "https://github.com/x/y/releases/download/models/fanar-1-9b-q4km-v1/fanar-1-9b-q4km.gguf.part-aa"
        );
    }
}
