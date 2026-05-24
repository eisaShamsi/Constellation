//! SHA-256 verification for downloaded model parts and the assembled
//! whole. Per Architect §3 invariant 8 (no model loads without verified
//! integrity) and §7 R-0b-4 (model-file integrity attack mitigation).

use sha2::{Digest, Sha256};
use std::io::Read;
use std::path::Path;

/// Streaming hex-SHA-256 of a file on disk. Reads in 1 MiB chunks; safe
/// for the multi-GiB assembled GGUF (peak memory = 1 MiB).
pub fn hex_sha256_of_file(path: &Path) -> Result<String, String> {
    let mut f = std::fs::File::open(path)
        .map_err(|e| format!("open {}: {e}", path.display()))?;
    let mut hasher = Sha256::new();
    let mut buf = vec![0u8; 1 << 20]; // 1 MiB
    loop {
        let n = f
            .read(&mut buf)
            .map_err(|e| format!("read {}: {e}", path.display()))?;
        if n == 0 {
            break;
        }
        hasher.update(&buf[..n]);
    }
    Ok(hex::encode(hasher.finalize()))
}

/// Verify a file's SHA-256 matches the expected hex string. Comparison
/// is case-insensitive (manifest emits lowercase; we compare lowercase
/// to be safe).
pub fn verify_file_sha256(path: &Path, expected_hex: &str) -> Result<(), String> {
    let actual = hex_sha256_of_file(path)?;
    if actual.eq_ignore_ascii_case(expected_hex) {
        Ok(())
    } else {
        Err(format!(
            "SHA-256 mismatch for {}: expected {expected_hex}, got {actual}",
            path.display()
        ))
    }
}

/// Verify a file's byte size matches.
pub fn verify_file_size(path: &Path, expected_bytes: u64) -> Result<(), String> {
    let meta = std::fs::metadata(path)
        .map_err(|e| format!("stat {}: {e}", path.display()))?;
    if meta.len() == expected_bytes {
        Ok(())
    } else {
        Err(format!(
            "size mismatch for {}: expected {expected_bytes} bytes, got {} bytes",
            path.display(),
            meta.len()
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    fn temp_with_bytes(bytes: &[u8]) -> tempfile::NamedTempFile {
        let mut f = tempfile::NamedTempFile::new().unwrap();
        f.write_all(bytes).unwrap();
        f.flush().unwrap();
        f
    }

    #[test]
    fn sha256_of_known_string() {
        let f = temp_with_bytes(b"hello world");
        let h = hex_sha256_of_file(f.path()).unwrap();
        // Well-known SHA-256 of "hello world".
        assert_eq!(
            h,
            "b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9"
        );
    }

    #[test]
    fn verify_accepts_matching_hash() {
        let f = temp_with_bytes(b"hello world");
        let r = verify_file_sha256(
            f.path(),
            "b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9",
        );
        assert!(r.is_ok());
    }

    #[test]
    fn verify_case_insensitive() {
        let f = temp_with_bytes(b"hello world");
        let r = verify_file_sha256(
            f.path(),
            "B94D27B9934D3E08A52E52D7DA7DABFAC484EFE37A5380EE9088F7ACE2EFCDE9",
        );
        assert!(r.is_ok());
    }

    #[test]
    fn verify_rejects_mismatched_hash() {
        let f = temp_with_bytes(b"hello world");
        let r = verify_file_sha256(f.path(), "deadbeef".repeat(8).as_str());
        assert!(r.is_err());
        assert!(r.unwrap_err().contains("SHA-256 mismatch"));
    }

    #[test]
    fn verify_size_matches() {
        let f = temp_with_bytes(b"hello world");
        assert!(verify_file_size(f.path(), 11).is_ok());
        assert!(verify_file_size(f.path(), 99).is_err());
    }
}
