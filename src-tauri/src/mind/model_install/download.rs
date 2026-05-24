//! Streamed chunked download for split-file model assets.
//!
//! Each model lives in a GitHub Release as ~1.7 GiB chunks (per
//! Architect §4 D). The download flow:
//! 1. Fetch the per-release `manifest.json` (small JSON).
//! 2. For each part listed in the manifest, download to a temp file
//!    with progress events flowing to the frontend.
//! 3. Verify the per-part SHA-256 before moving on.
//! 4. Append each verified part to the assembled GGUF in a single pass.
//! 5. Verify the assembled file's SHA-256 against the manifest's
//!    `final_sha256` AND against the catalog entry's `final_sha256`
//!    (cross-check to catch a tampered manifest).
//!
//! Frontend progress shape: `DownloadProgress` events stream through
//! a `tauri::ipc::Channel<DownloadProgress>` (same primitive
//! MIG-046 §C set up for `StreamEvent`).

use serde::{Deserialize, Serialize};

use crate::mind::model_install::manifest::{ModelEntry, ReleaseManifest, ReleasePart};

/// Progress event for one install. Mirrors the model_install flow stages
/// so the frontend can render meaningful UI per phase.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "stage", rename_all = "snake_case")]
pub enum DownloadProgress {
    /// Manifest fetch in flight (small; usually <1s).
    FetchingManifest { model_id: String },
    /// Downloading a specific part. `bytes_done` is for THIS PART,
    /// not the whole; `bytes_total` is the part's total.
    /// `part_index` is 1-based for UI.
    DownloadingPart {
        model_id: String,
        part_index: u32,
        total_parts: u32,
        part_name: String,
        bytes_done: u64,
        bytes_total: u64,
    },
    /// A part finished + verified.
    PartVerified {
        model_id: String,
        part_index: u32,
        total_parts: u32,
    },
    /// Assembling the parts into the final GGUF.
    Assembling { model_id: String },
    /// Verifying the assembled file's final SHA-256.
    VerifyingFinal { model_id: String },
    /// Installation complete; the model is registered.
    Done {
        model_id: String,
        final_path: String,
        final_size_bytes: u64,
    },
    /// Installation failed. The Tauri command also returns Err; this
    /// event lets the UI show the failure reason inline.
    Failed { model_id: String, error: String },
}

/// Fetch the per-release manifest JSON from the catalog entry's URL.
pub async fn fetch_release_manifest(entry: &ModelEntry) -> Result<ReleaseManifest, String> {
    let client = reqwest::Client::builder()
        .user_agent("Constellation-Mind/0.1 (+https://github.com/eisaShamsi/Constellation)")
        .build()
        .map_err(|e| format!("build http client: {e}"))?;

    let resp = client
        .get(&entry.manifest_url)
        .send()
        .await
        .map_err(|e| format!("GET {}: {e}", entry.manifest_url))?
        .error_for_status()
        .map_err(|e| format!("manifest HTTP error: {e}"))?;

    let bytes = resp
        .bytes()
        .await
        .map_err(|e| format!("read manifest body: {e}"))?;

    serde_json::from_slice::<ReleaseManifest>(&bytes)
        .map_err(|e| format!("parse manifest JSON: {e}"))
}

/// Download one part to a temp file with chunk-level progress callbacks.
/// Returns the local path of the downloaded file (caller is responsible
/// for cleanup on failure paths).
pub async fn download_part_to<F>(
    part_url: &str,
    dest_path: &std::path::Path,
    mut on_chunk: F,
) -> Result<(), String>
where
    F: FnMut(u64, u64),
{
    use tokio::io::AsyncWriteExt;

    let client = reqwest::Client::builder()
        .user_agent("Constellation-Mind/0.1 (+https://github.com/eisaShamsi/Constellation)")
        .build()
        .map_err(|e| format!("build http client: {e}"))?;

    let mut resp = client
        .get(part_url)
        .send()
        .await
        .map_err(|e| format!("GET {part_url}: {e}"))?
        .error_for_status()
        .map_err(|e| format!("part HTTP error: {e}"))?;

    let total = resp.content_length().unwrap_or(0);
    let mut file = tokio::fs::File::create(dest_path)
        .await
        .map_err(|e| format!("create {}: {e}", dest_path.display()))?;

    let mut downloaded: u64 = 0;
    while let Some(chunk) = resp
        .chunk()
        .await
        .map_err(|e| format!("read chunk: {e}"))?
    {
        file.write_all(&chunk)
            .await
            .map_err(|e| format!("write chunk: {e}"))?;
        downloaded += chunk.len() as u64;
        on_chunk(downloaded, total);
    }
    file.flush()
        .await
        .map_err(|e| format!("flush {}: {e}", dest_path.display()))?;
    Ok(())
}

/// Compute the per-part download URL given the manifest URL and a part.
pub fn part_url(manifest_url: &str, part: &ReleasePart) -> String {
    ReleaseManifest::part_url(manifest_url, part)
}

/// Append the bytes of `src` onto `dst` (open in append mode). Used to
/// assemble the final GGUF from verified parts.
pub async fn append_to(dst: &std::path::Path, src: &std::path::Path) -> Result<(), String> {
    use tokio::io::AsyncWriteExt;

    let mut src_file = tokio::fs::File::open(src)
        .await
        .map_err(|e| format!("open src {}: {e}", src.display()))?;
    let mut dst_file = tokio::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(dst)
        .await
        .map_err(|e| format!("open dst {}: {e}", dst.display()))?;

    let mut buf = vec![0u8; 4 << 20]; // 4 MiB
    loop {
        use tokio::io::AsyncReadExt;
        let n = src_file
            .read(&mut buf)
            .await
            .map_err(|e| format!("read src: {e}"))?;
        if n == 0 {
            break;
        }
        dst_file
            .write_all(&buf[..n])
            .await
            .map_err(|e| format!("write dst: {e}"))?;
    }
    dst_file
        .flush()
        .await
        .map_err(|e| format!("flush dst: {e}"))?;
    Ok(())
}
