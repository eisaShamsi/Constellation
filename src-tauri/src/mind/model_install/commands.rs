//! Tauri IPC commands for model installation.
//!
//! Four commands ship in Phase 0b:
//! - `mind_install_model` — orchestrates the chunked download +
//!   verification + assembly for one model entry from `models.json`,
//!   streaming `DownloadProgress` events to the frontend through a
//!   typed `Channel<DownloadProgress>`.
//! - `mind_list_installed_models` — returns the per-user registry.
//! - `mind_active_model` — returns which model is currently active
//!   (the one `mind_start_turn` would load).
//! - `mind_set_active_model` — updates the active-model selection.
//!
//! Wired in `lib.rs:invoke_handler!` (Step E follow-up edit).

use tauri::{AppHandle, Manager};
use tauri::ipc::Channel;

use crate::mind::model_install::download::{
    append_to, download_part_to, fetch_release_manifest, part_url, DownloadProgress,
};
use crate::mind::model_install::manifest::{ModelEntry, ModelsCatalog};
use crate::mind::model_install::registry::{InstalledModel, InstalledModelsRegistry};
use crate::mind::model_install::verify::{verify_file_sha256, verify_file_size};

/// Read the bundled `models.json` Tauri resource.
fn load_catalog(app: &AppHandle) -> Result<ModelsCatalog, String> {
    let resource_dir = app
        .path()
        .resource_dir()
        .map_err(|e| format!("resource_dir: {e}"))?;
    let catalog_path = resource_dir.join("resources").join("models.json");
    let raw = std::fs::read_to_string(&catalog_path)
        .or_else(|_| {
            // Fallback for dev mode where the resource_dir layout differs.
            let dev = std::env::current_dir()
                .map_err(|e| format!("cwd: {e}"))?
                .join("src-tauri/resources/models.json");
            std::fs::read_to_string(&dev)
                .map_err(|e| format!("read models.json (dev): {e}"))
        })?;
    serde_json::from_str::<ModelsCatalog>(&raw)
        .map_err(|e| format!("parse models.json: {e}"))
}

/// Path under which Constellation stores installed models on disk.
fn models_dir(app: &AppHandle) -> Result<std::path::PathBuf, String> {
    let app_data = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("app_data_dir: {e}"))?;
    let dir = app_data.join("models");
    std::fs::create_dir_all(&dir)
        .map_err(|e| format!("create {}: {e}", dir.display()))?;
    Ok(dir)
}

fn app_data_dir(app: &AppHandle) -> Result<std::path::PathBuf, String> {
    app.path()
        .app_data_dir()
        .map_err(|e| format!("app_data_dir: {e}"))
}

fn now_unix() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

/// Install one model: download all parts, verify, assemble, register.
#[tauri::command]
pub async fn mind_install_model(
    app: AppHandle,
    model_id: String,
    on_progress: Channel<DownloadProgress>,
) -> Result<(), String> {
    let catalog = load_catalog(&app)?;
    let entry: ModelEntry = catalog
        .models
        .iter()
        .find(|m| m.id == model_id)
        .cloned()
        .ok_or_else(|| format!("unknown model id: {model_id}"))?;

    if !entry.is_ready_to_install() {
        let err = format!(
            "model '{model_id}' is not yet ready to install — final_sha256 \
             still TBD (the model-pipeline workflow has not run + populated \
             the catalog yet)."
        );
        let _ = on_progress.send(DownloadProgress::Failed {
            model_id: model_id.clone(),
            error: err.clone(),
        });
        return Err(err);
    }

    let _ = on_progress.send(DownloadProgress::FetchingManifest {
        model_id: model_id.clone(),
    });

    let manifest = match fetch_release_manifest(&entry).await {
        Ok(m) => m,
        Err(e) => {
            let _ = on_progress.send(DownloadProgress::Failed {
                model_id: model_id.clone(),
                error: e.clone(),
            });
            return Err(e);
        }
    };

    // Cross-check the manifest's final_sha256 against the bundled catalog.
    if !manifest
        .final_sha256
        .eq_ignore_ascii_case(&entry.final_sha256)
    {
        let err = format!(
            "manifest final_sha256 {} does not match catalog entry {} for {model_id}",
            manifest.final_sha256, entry.final_sha256
        );
        let _ = on_progress.send(DownloadProgress::Failed {
            model_id: model_id.clone(),
            error: err.clone(),
        });
        return Err(err);
    }

    let dir = models_dir(&app)?;
    let final_path = dir.join(format!("{}-{}.gguf", entry.id, entry.version));
    let tmp_dir = dir.join(format!(".tmp-{}-{}", entry.id, entry.version));
    std::fs::create_dir_all(&tmp_dir)
        .map_err(|e| format!("mkdir {}: {e}", tmp_dir.display()))?;

    // If the final file already exists with the right hash, skip the download.
    if final_path.exists()
        && verify_file_sha256(&final_path, &entry.final_sha256).is_ok()
        && verify_file_size(&final_path, entry.final_size_bytes).is_ok()
    {
        let _ = on_progress.send(DownloadProgress::Done {
            model_id: model_id.clone(),
            final_path: final_path.to_string_lossy().to_string(),
            final_size_bytes: entry.final_size_bytes,
        });
        // Even on early-exit, refresh the registry entry.
        register_installed(&app, &entry, &final_path)?;
        return Ok(());
    }

    // Otherwise: clean up any prior assembly + download fresh.
    if final_path.exists() {
        let _ = std::fs::remove_file(&final_path);
    }

    let total_parts = manifest.parts.len() as u32;
    for (idx, part) in manifest.parts.iter().enumerate() {
        let part_index = (idx + 1) as u32;
        let part_path = tmp_dir.join(&part.name);
        let url = part_url(&entry.manifest_url, part);

        // Stream this part, emitting progress events as bytes arrive.
        let model_id_for_progress = model_id.clone();
        let part_name_for_progress = part.name.clone();
        let on_progress_clone = on_progress.clone();
        let total_parts_for_progress = total_parts;
        let result = download_part_to(&url, &part_path, move |bytes_done, bytes_total| {
            let _ = on_progress_clone.send(DownloadProgress::DownloadingPart {
                model_id: model_id_for_progress.clone(),
                part_index,
                total_parts: total_parts_for_progress,
                part_name: part_name_for_progress.clone(),
                bytes_done,
                bytes_total,
            });
        })
        .await;

        if let Err(e) = result {
            let _ = on_progress.send(DownloadProgress::Failed {
                model_id: model_id.clone(),
                error: e.clone(),
            });
            // Clean up partial files.
            let _ = std::fs::remove_dir_all(&tmp_dir);
            return Err(e);
        }

        // Verify the part's hash + size immediately.
        if let Err(e) = verify_file_size(&part_path, part.size_bytes) {
            let _ = on_progress.send(DownloadProgress::Failed {
                model_id: model_id.clone(),
                error: e.clone(),
            });
            let _ = std::fs::remove_dir_all(&tmp_dir);
            return Err(e);
        }
        if let Err(e) = verify_file_sha256(&part_path, &part.sha256) {
            let _ = on_progress.send(DownloadProgress::Failed {
                model_id: model_id.clone(),
                error: e.clone(),
            });
            let _ = std::fs::remove_dir_all(&tmp_dir);
            return Err(e);
        }

        let _ = on_progress.send(DownloadProgress::PartVerified {
            model_id: model_id.clone(),
            part_index,
            total_parts,
        });
    }

    // All parts verified. Assemble.
    let _ = on_progress.send(DownloadProgress::Assembling {
        model_id: model_id.clone(),
    });
    // Make sure the destination doesn't already exist (append would otherwise add).
    if final_path.exists() {
        let _ = std::fs::remove_file(&final_path);
    }
    for part in &manifest.parts {
        let part_path = tmp_dir.join(&part.name);
        if let Err(e) = append_to(&final_path, &part_path).await {
            let _ = on_progress.send(DownloadProgress::Failed {
                model_id: model_id.clone(),
                error: e.clone(),
            });
            let _ = std::fs::remove_file(&final_path);
            let _ = std::fs::remove_dir_all(&tmp_dir);
            return Err(e);
        }
    }

    // Verify the assembled whole against BOTH the manifest AND the catalog.
    let _ = on_progress.send(DownloadProgress::VerifyingFinal {
        model_id: model_id.clone(),
    });
    if let Err(e) = verify_file_size(&final_path, entry.final_size_bytes) {
        let _ = on_progress.send(DownloadProgress::Failed {
            model_id: model_id.clone(),
            error: e.clone(),
        });
        let _ = std::fs::remove_file(&final_path);
        let _ = std::fs::remove_dir_all(&tmp_dir);
        return Err(e);
    }
    if let Err(e) = verify_file_sha256(&final_path, &entry.final_sha256) {
        let _ = on_progress.send(DownloadProgress::Failed {
            model_id: model_id.clone(),
            error: e.clone(),
        });
        let _ = std::fs::remove_file(&final_path);
        let _ = std::fs::remove_dir_all(&tmp_dir);
        return Err(e);
    }

    // Cleanup tmp dir; register the install.
    let _ = std::fs::remove_dir_all(&tmp_dir);
    register_installed(&app, &entry, &final_path)?;

    let _ = on_progress.send(DownloadProgress::Done {
        model_id: model_id.clone(),
        final_path: final_path.to_string_lossy().to_string(),
        final_size_bytes: entry.final_size_bytes,
    });
    Ok(())
}

fn register_installed(
    app: &AppHandle,
    entry: &ModelEntry,
    final_path: &std::path::Path,
) -> Result<(), String> {
    let app_data = app_data_dir(app)?;
    let mut reg = InstalledModelsRegistry::load_from(&app_data)?;
    reg.upsert(InstalledModel {
        id: entry.id.clone(),
        version: entry.version.clone(),
        display_name: entry.display_name.clone(),
        file_path: final_path.to_string_lossy().to_string(),
        size_bytes: entry.final_size_bytes,
        sha256: entry.final_sha256.clone(),
        installed_at_unix: now_unix(),
    });
    reg.save_to(&app_data)?;
    Ok(())
}

/// Return the bundled `models.json` catalog so the Settings UI can list
/// installable models with their display names + sizes + license info.
#[tauri::command]
pub async fn mind_list_catalog(app: AppHandle) -> Result<ModelsCatalog, String> {
    load_catalog(&app)
}

#[tauri::command]
pub async fn mind_list_installed_models(app: AppHandle) -> Result<InstalledModelsRegistry, String> {
    InstalledModelsRegistry::load_from(&app_data_dir(&app)?)
}

#[tauri::command]
pub async fn mind_active_model(app: AppHandle) -> Result<Option<InstalledModel>, String> {
    let reg = InstalledModelsRegistry::load_from(&app_data_dir(&app)?)?;
    let id = match reg.active_model_id.as_deref() {
        Some(id) => id.to_string(),
        None => return Ok(None),
    };
    Ok(reg.get(&id).cloned())
}

#[tauri::command]
pub async fn mind_set_active_model(app: AppHandle, model_id: String) -> Result<(), String> {
    let app_data = app_data_dir(&app)?;
    let mut reg = InstalledModelsRegistry::load_from(&app_data)?;
    reg.set_active(&model_id)?;
    reg.save_to(&app_data)
}
