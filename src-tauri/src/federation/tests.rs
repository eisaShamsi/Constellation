//! MIG-056 §A — Federation tests scaffold.
//!
//! §A ships the foundation types only. §B/§C/§D/§I extend this
//! module with the actual federation behavior tests (≥30 cases
//! per Plan §I).

#![cfg(test)]

use super::{FederationContext, FederationWarning};
use std::path::PathBuf;

#[test]
fn federation_context_starts_empty_and_not_ready() {
    let ctx = FederationContext::new();
    assert_eq!(ctx.attached().len(), 0);
    assert_eq!(ctx.warnings().len(), 0);
    assert!(!ctx.is_ready());
}

#[test]
fn federation_context_default_matches_new() {
    let a = FederationContext::new();
    let b = FederationContext::default();
    assert_eq!(a.attached().len(), b.attached().len());
    assert_eq!(a.warnings().len(), b.warnings().len());
    assert_eq!(a.is_ready(), b.is_ready());
}

#[test]
fn federation_context_add_attached_appends() {
    let mut ctx = FederationContext::new();
    ctx.add_attached("cu0".to_string(), PathBuf::from("/some/path"));
    ctx.add_attached("cu1".to_string(), PathBuf::from("/other/path"));
    assert_eq!(ctx.attached().len(), 2);
    assert_eq!(ctx.attached()[0].0, "cu0");
    assert_eq!(ctx.attached()[1].1, PathBuf::from("/other/path"));
}

#[test]
fn federation_context_warn_appends_and_skip_unavailable_model() {
    let mut ctx = FederationContext::new();
    ctx.warn(PathBuf::from("/missing/uni"), "search.db missing");
    ctx.warn(PathBuf::from("/locked/uni"), "locked by another process");
    assert_eq!(ctx.warnings().len(), 2);
    assert_eq!(ctx.warnings()[0].reason, "search.db missing");
    assert_eq!(ctx.warnings()[1].cuniverse_path, "/locked/uni");
}

#[test]
fn federation_context_set_ready_round_trips() {
    let mut ctx = FederationContext::new();
    assert!(!ctx.is_ready());
    ctx.set_ready(true);
    assert!(ctx.is_ready());
    ctx.set_ready(false);
    assert!(!ctx.is_ready());
}

#[test]
fn federation_context_reset_clears_all() {
    let mut ctx = FederationContext::new();
    ctx.add_attached("cu0".to_string(), PathBuf::from("/x"));
    ctx.warn(PathBuf::from("/y"), "test");
    ctx.set_ready(true);

    ctx.reset();

    assert_eq!(ctx.attached().len(), 0);
    assert_eq!(ctx.warnings().len(), 0);
    assert!(!ctx.is_ready());
}

#[test]
fn federation_warning_records_path_reason_and_timestamp() {
    let w = FederationWarning::new(
        PathBuf::from("E:/some/cuniverse"),
        "schema version 5 below floor 7",
    );
    assert_eq!(w.cuniverse_path, "E:/some/cuniverse");
    assert_eq!(w.reason, "schema version 5 below floor 7");
    assert!(w.when_unix > 0, "timestamp should be set");
}

#[test]
fn federation_warning_serializes_to_json() {
    // The frontend (`§H federation_get_warnings`) consumes the serde
    // JSON representation. Lock the shape so it doesn't drift silently.
    let w = FederationWarning::new(PathBuf::from("/x"), "missing");
    let json = serde_json::to_string(&w).unwrap();
    assert!(json.contains("\"cuniverse_path\":\"/x\""));
    assert!(json.contains("\"reason\":\"missing\""));
    assert!(json.contains("\"when_unix\":"));
}
