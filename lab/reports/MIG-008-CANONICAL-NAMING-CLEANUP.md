# MIG-008 — Canonical Naming Cleanup

**Owner**: Eisa ALSHAMSI
**Migration lead**: Claude
**Opened**: 2026-04-27
**Status**: Phase 2 — Plan (awaiting approval)

---

## Phase 1 — Architect

### The user-visible problem

When a note has a canonical filename (e.g. `20260426T140737Z_NOTE_E561.md`), several panels and views display the canonical id as the note's label instead of the human-readable frontmatter title (e.g. "Apple Tree Fruit"). Users can't recognize their notes in those surfaces.

### Why this happens

Constellation has TWO ways a surface can produce a note label:

1. **Read from SQLite** (`note_meta.name`, `sky_nodes.name`). These are populated at index time by [`search.rs::index_note:1665-1670`](src-tauri/src/search.rs:1665), which already prefers the frontmatter `title:` field with file_stem fallback. ✅ Working correctly today.

2. **Scan the filesystem directly** and derive the label from `path.file_stem()`. ❌ This is the bug class — every Cognitive Engine Layer 1 phase scanner that walks the library tree currently does this without consulting frontmatter.

### Surfaces affected (Phase 1 inventory — verified by grep)

| # | Surface | File:Line | Currently shows |
|---|---|---|---|
| 1 | Constellation Map (universe + library variants) | [`map.rs:422`](src-tauri/src/map.rs:422), [`map.rs:508`](src-tauri/src/map.rs:508) | canonical filename in tooltips |
| 2 | 360° Inspector — note name | [`inspector360.rs:88`](src-tauri/src/inspector360.rs:88), [`:266`](src-tauri/src/inspector360.rs:266) | canonical filename in note + neighbors |
| 3 | 360° Inspector — trail name | [`inspector360.rs:431`](src-tauri/src/inspector360.rs:431) | canonical filename in trail rows |
| 4 | Knowledge Strata scanner | [`strata.rs:182`](src-tauri/src/strata.rs:182) | strata results keyed/labeled by filename |
| 5 | Maturity Lifecycle scanner | [`maturity.rs:157`](src-tauri/src/maturity.rs:157) | maturity results labeled by filename |
| 6 | Provenance Chain scanner | [`provenance.rs:61`](src-tauri/src/provenance.rs:61), [`:158`](src-tauri/src/provenance.rs:158) | provenance chain labels |
| 7 | Review Pulse scanner | [`review.rs:208`](src-tauri/src/review.rs:208) | due-note labels |
| 8 | Multi-Lens scanner | [`lenses.rs:186`](src-tauri/src/lenses.rs:186), [`:219`](src-tauri/src/lenses.rs:219) | lens grouping labels |
| 9 | Tasks scanner | [`tasks.rs:243`](src-tauri/src/tasks.rs:243), [`:285`](src-tauri/src/tasks.rs:285), [`:383`](src-tauri/src/tasks.rs:383) | task source labels |
| 10 | Canvas list | [`canvas.rs:115`](src-tauri/src/canvas.rs:115) | canvas item labels |
| 11 | Bases query rows | [`bases.rs:725`](src-tauri/src/bases.rs:725) | base table row labels |
| 12 | Tension Detector (likely — needs verification) | [`tension.rs`](src-tauri/src/tension.rs) | contradiction labels |

Plus **already fixed** in §90: [`libraries.rs::scan_unlinked_recursive:1766`](src-tauri/src/libraries.rs:1766) — Unlinked Mentions panel.

### Invariants this migration must not break

1. **Wikilink resolution.** Wikilinks resolve against `note_meta.name` (which is already the title). The migration must not change that. **Path 2 only touches scanners' display strings, not the resolution path. Safe.**
2. **Cache keys / lookup ids.** Internal use of `path.file_stem()` for deduplication, cache keys, and indexing must remain. Don't change those — only change USER-VISIBLE label strings.
3. **Sky View / Backlinks / Search results.** These read from `note_meta` / `sky_nodes` and already work. Don't touch them.
4. **Performance**: each scanner already reads the file content for word_count / link extraction / etc. The frontmatter-title extraction is a sub-millisecond regex scan over the same string. No I/O added.
5. **No schema migration**. No reindex. No trigger changes. Path 1 is already correct in production.

### Risks considered

- **A scanner doesn't already read the file content**: it'd need to add a `fs::read_to_string`. Per surface, verify content is already read.
- **Frontmatter parser edge cases**: `extract_frontmatter_title()` returns `None` if title is missing or malformed. Helper falls back to file_stem. No worse than today.
- **Internationalization**: titles can contain Arabic / Hebrew / Cyrillic / etc. `extract_frontmatter_title` returns the raw string; downstream rendering already handles these. No risk.
- **Frontmatter title with quotes**: `title: "Apple (Fruit)"` quoted form. `extract_frontmatter_title` already trims quotes (verified in `libraries.rs:916`). No risk.

---

## Phase 2 — Plan (awaiting approval)

### Step 0 — Shared helper

Add `pub(crate) fn note_display_name(path: &Path, content: &str) -> String` to [`libraries.rs`](src-tauri/src/libraries.rs):

```rust
/// User-visible display name for a note. Prefers the frontmatter
/// `title:` field; falls back to file stem when title is missing.
/// Caller passes already-read file content (every scanner reads
/// content for other reasons — word_count, links, etc.).
pub(crate) fn note_display_name(path: &Path, content: &str) -> String {
    extract_frontmatter_title(content)
        .unwrap_or_else(|| path.file_stem()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_default())
}
```

**Verification**: `cargo check` clean.
**User-testable**: No (foundation only).

### Step 1 — `map.rs` (Constellation Map)

Patch the two `path.file_stem()` call sites in [`map.rs`](src-tauri/src/map.rs) so the Map's tooltips and labels use the human title.

**Verification**: User opens the Constellation Map, hovers an arc segment for any canonical note, sees the human title (not `20260426T...`). Existing test set (`MIG-005 Test/`) is sufficient — six canonical notes already there.
**User-testable**: Yes — Stage 1.

### Step 2 — `inspector360.rs` (360° Inspector)

Three call sites. Same pattern.

**Verification**: User opens the 360° Inspector on a canonical note, sees the human title in the center AND neighbor / trail rows.
**User-testable**: Yes — Stage 2.

### Step 3 — `strata.rs`, `maturity.rs`, `provenance.rs`

Three CE Layer 1 phases sharing identical scanner shape. Same patch pattern in each.

**Verification**: Open Knowledge Health (strata + maturity surface) on a canonical note. Open Provenance panel. All show human titles.
**User-testable**: Yes — Stage 3 (one combined verification).

### Step 4 — `review.rs`, `lenses.rs`, `tasks.rs`, `canvas.rs`, `bases.rs`

Five more scanners. Same patch pattern.

**Verification**: Open Review Pulse, Multi-Lens, Tasks panel, a Canvas, and a Base query. Any canonical-named source displays human title.
**User-testable**: Yes — Stage 4 (combined).

### Step 5 — `tension.rs` (verify + patch if needed)

Confirmed by grep: tension scanner exists. If it derives labels from filenames, apply same patch.

**Verification**: Tension panel shows human titles.
**User-testable**: Yes — Stage 5 (small).

### Step 6 — Sweep audit

Re-grep `path.file_stem()` across `src-tauri/src/`. For every remaining hit, classify Category A (internal/correct) vs Category B (label leak). For B, patch with the helper. For A, document why it's left as-is in a code comment.

**Verification**: Final grep produces only Category A hits, each with a comment justifying its presence.
**User-testable**: No (code-review checkpoint).

### Step 7 — Phase 4 audit

Three parallel agents per the migration skill:
- **Invariant Check**: confirm wikilink resolution, cache_boot_snapshot_*, Sky View, Backlinks, Outgoing Links, search all still work.
- **Drift Check**: any new code path that derives a label from a filename without going through the helper?
- **Migration Path Check**: cold-boot, schema readiness, watcher reindex, second-screen sync — all unaffected.

**Verification**: All three agents return clean.
**User-testable**: No (audit only).

---

## Cumulative scope

- 1 helper added (`note_display_name`)
- ~14 scanner sites patched across 11 Rust files
- 0 schema changes
- 0 trigger changes
- 0 frontend changes
- 0 IPC contract changes
- 0 user data migration

Each per-surface change is small, uniform, and low-risk. Total effort estimate: ~30 min of code, 15 min of build, plus user verification rounds.

---

## Approval requested

User approval requested on:

1. **The collapsed scope** (Path 2 only — Path 1 already in production).
2. **The 6-stage cascade** with user testing after Steps 1, 2, 3, 4, 5.
3. **Skipping a frontend rebuild for Steps 0 / 6 / 7** (no user-visible change in those steps).

Once approved, build cascades per Plan-Approval-Equals-Build-Approval.
