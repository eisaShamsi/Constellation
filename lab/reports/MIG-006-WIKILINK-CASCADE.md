# MIG-006 — Wikilink Rename Cascade (Option 1: hard sync rewrite)

**Status**: Phase 1 complete (Architect). Phase 2 plan drafted — awaiting Build.
**Scope**: When a note is renamed, every source note containing `[[old_title]]` has that token rewritten on disk to `[[new_title]]`, preserving optional `|display` and `|link-type` annotations. Rewritten source files are re-indexed so `note_links.target_name` reflects the new target.
**Chosen option**: **1 — Hard cascade rewrite, sync, with the existing broken walker fixed.**

---

## Phase 1 — Architect (summary)

### The walker exists and is wired in but broken at the call site

`libraries.rs::update_links_on_rename` (L3502) and `update_links_recursive` (L3509) walk every `.md` under a library, replace `[[old]]` and `[[old|...]]` with their `new` equivalents, and write back. Frontend gates this on `appSettings.autoUpdateLinks` (default `true`) at `+layout.svelte:3795` after a successful `renameItem`.

### Three concrete defects

1. **Wrong `oldName` source** (`+layout.svelte:3788`): `oldPath.split(...).pop()?.replace('.md','')` yields the canonical filename stem (e.g. `20260424T063440Z_NOTE_531D`), never the human title (`§2 Round3`). The walker scans for `[[20260424T063440Z_NOTE_531D]]`, finds nothing, returns 0. **Cascade is silently dead for every canonical-filename note.**

2. **Watcher loop / open-editor race** (Invariant 14): the walker calls `fs::write` directly with no equivalent of the frontend's `recentWrites` map (`store.ts:151`). Each rewritten source bubbles up through the file watcher as an external edit; if a tab is open mid-autosave, you get LL-022-class overwrite races.

3. **Index drift** (Invariant 15): rewriting `[[old]] → [[new]]` in a source's body does NOT call `index_note` for that source. `note_links.target_name` still says `old` until the user touches that source again. MIG-004's alias-aware reads paper over this for inbound counts, but search/strata/maturity readouts that consult `target_name` directly stay stale.

### Wikilink syntax to preserve

From `extract_wikilinks` regex `\[\[([^\]|]+?)(?:\|[^\]]+?)?\]\]` and the livePreview parser:

- `[[Foo]]` — bare
- `[[Foo|display text]]` — display override
- `[[Foo|link-type]]` — typed-link annotation
- `[[Foo|alias|link-type]]` — both display override and link-type
- `![[Foo]]` — embed/transclude

Cascade rewrites the target name only — everything from the first `|` onward is preserved verbatim.

### Backfill scope

Pre-MIG-006 sources that already contain stale `[[old_title]]` won't auto-rewrite (the rename event has long since passed). MIG-004's `note_aliases` table (rows with `source='rename'`) is the authoritative list of historical (path, old_alias) pairs and drives an opt-in "rewrite stale wikilinks" command.

### Lessons from MIG-004

The alias table stays as a defense-in-depth read-side resolver for typos / partial / Arabic-normalized matches and for the brief window between rewrite-on-disk and reindex completion. **MIG-006 does not invalidate MIG-004; the two compose.**

---

## Hub-note threshold and async UX

- **Synchronous bound**: ≤ 100 sources rewritten in-process within the existing `update_links_on_rename` IPC.
- **Async path**: > 100 inbound. Rust spawns a blocking task and emits Tauri events `cascade:progress` `{ rename_id, processed, total, current_path }` every 25 files (or 250 ms, whichever first), `cascade:done` `{ rename_id, rewritten, failed: [{path, err}] }`, `cascade:cancelled`.
- **Pre-count** (cheap): a first-pass walk that only counts `content.contains("[[old]]") || content.contains("[[old|")` decides sync vs async before any write.
- **UX**: a toast with progress bar and Cancel button shows whenever cascade goes async. Cancellation flips an `AtomicBool` keyed by `rename_id`; the walker checks it between files. Files already rewritten stay rewritten — partial rewrites are valid intermediate state, not corruption.

## Watcher-loop strategy

Extend the recent-writes suppression to a **Rust-side `recent_writes: Mutex<HashMap<PathBuf, Instant>>`** in the same module that owns the file watcher. The cascade calls `mark_recent_write(path)` immediately before each `fs::write` and entries TTL out after 2.5 s. The watcher's "external edit" path checks this map first and skips emit when present.

Rejected alternatives: single suppression event (too coarse), block library-changed for a window (starves legitimate edits), call JS `markRecentWrite` from Rust (inverted control flow).

## Reindex strategy

**Per-source `index_note(conn, path, library_name)`** (already exposed via `constellation_search_reindex` at `search.rs:3064`). Slow but correct: it preserves frontmatter parsing, alias extraction, FTS5 sync, and the living-link traversal-counter snapshot at search.rs:1535 (Invariant 9 from MIG-004). Direct DELETE+INSERT into `note_links` would couple us to a schema we don't own.

Mitigation: reindex runs in batches of 25 with `busy_timeout=30s`, the same shape as `sky_backfill.rs`. The 1000-file ceiling reindexes in ~5–10 s on a warm SSD.

## Pre-MIG-006 backfill

- New Tauri command `cascade_backfill_stale_wikilinks(library_path)`.
- Reads `note_aliases WHERE source='rename'`. For each `(path, alias_lower)` row, resolves the current title for `path`, then runs the same cascade walker for `[[alias]] → [[currentTitle]]` across the library.
- **UI entry point**: Settings → Files → "Rewrite stale wikilinks…" button right under the existing `autoUpdateLinks` toggle (`SettingsModal.svelte:1420`). Click opens a confirmation showing `(path, old_alias, new_title)` rows; user can deselect any row before applying. Async-only path.

## Atomicity / partial failure

- Each `fs::write` is **per-file atomic** via `tempfile + persist` (write to `path.tmp`, fsync, rename over). On crash mid-cascade, no source is half-written.
- Cascade itself is **not** transactional across files. Failures collect into `cascade:done.failed[]`. Toast offers Retry; retry runs only the failed list.

---

## Phase 2 — Plan (11 steps)

| # | Step | /simplify? | Verify |
|---|------|-----------|--------|
| 1 | Lift `oldName` from frontmatter title, not filename | — | Cascade fires with `oldName="§2 Round3"` not the canonical stem |
| 2 | Walker correctness pass: regex-based, transclude-aware, link-type-preserving | — | Unit test covers all 5 wikilink shapes |
| 3 | Open-editor coherence: flush-before-cascade + reload-after-cascade + `recent_writes` watcher suppression | — | "Edit source, rename target without switching tabs" works; cascade rewrite survives the next autosave |
| 4 | Reindex each rewritten source via `index_note` | — | `note_links.target_name` shows new title post-cascade |
| 5 | `/simplify` checkpoint | ✔ | Empty diff or one focused refactor |
| 6 | Pre-flight count + sync/async dispatch | — | 200-inbound rename keeps UI responsive |
| 7 | Progress, cancellation, completion events + frontend toast | — | Cancel mid-cascade → already-rewritten stay rewritten, walker stops cleanly |
| 8 | `/simplify` checkpoint | ✔ | Visitor extracted, toast scope tightened |
| 9 | Atomic per-file writes via tempfile | — | Kill mid-cascade → every file fully old or fully new, never half-written |
| 10 | Pre-MIG-006 backfill command + Settings button | — | Vault with stale `[[old]]` body refs gets rewritten on demand |
| 11 | Phase 4 audit + closure | — | All invariants exercised; CLAUDE.md notes recent_writes + dispatcher primitives |

### `/simplify` checkpoints

Steps **5, 8** — after the writer/correctness lands, after the async path lands.

### Phase 4 audit trigger

Step 11 invokes `/migration` audit (invariant / drift / migration-path agents) plus a 200-file rename-in-Git-tracked-vault test for sync semantics.

---

### §1 — Lift `oldName` from frontmatter title, not filename

**What**
- `+layout.svelte::handleRenameComplete` (L3778): replace L3788 with a frontmatter-title lookup. New helper `getOldTitleForCascade(oldPath): Promise<string>` in `src/lib/libraries/store.ts` that (a) reads from the open tab if present, (b) falls back to a new lightweight Tauri `read_note_title(path)` command in `libraries.rs` that parses just the frontmatter of the file before it gets renamed. Call this **before** `renameItem`.

**Why** — Invariant 12. Without this, every other step is academic.

**Verification** — Rename a canonical-filename note titled `§2 Round3` that's wikilinked from another note as `[[§2 Round3]]`. Confirm `update_links_on_rename` is invoked with `oldName="§2 Round3"` in the Tauri log.

**Risk mitigation** — Title lookup races with autosave. Solution: read the title BEFORE awaiting `renameItem`, store in local `const oldTitle`, pass that to the cascade.

**Rollback** — `git revert` — single commit, no schema impact.

---

### §2 — Walker correctness pass

**What**
- Rewrite `update_links_recursive` body in `libraries.rs:3509`. Replace `String::contains` + `replace` with a single compiled regex `\[\[(<old_escaped>)(\||\]\])` that captures the trailing delimiter. Apply via `Regex::replace_all` with a closure that emits `[[<new>]]` or `[[<new>|...`. Handles `[[Old]]`, `[[Old|display]]`, `[[Old|link-type]]`, `[[Old|alias|link-type]]`, and `![[Old]]`. Escape `old_name` with `regex::escape` so titles like `§2 Round3` work.

**Why** — Invariants 1, 4.

**Verification** — Unit test in `libraries.rs` covering all five wikilink shapes and the prefix-collision edge case (`Foo` vs `Foo Bar` — must not be cross-rewritten).

**Risk** — Arabic-normalized form mismatch. Mitigation: cascade uses the *raw human title* (pre-normalization), and the regex matches case-insensitively only for the ASCII range. Arabic stays exact-match.

**Rollback** — Revert single commit.

---

### §3 — Open-editor coherence (expanded)

**Why this is bigger than the original §3.** §1 verification (2026-04-25, BUG-013 diagnostic) exposed a class of failure the original §3 plan didn't cover. Three races, not one:

1. **Pre-cascade staleness.** Source tab is open and dirty; user renames the target before the source's debounced autosave fires. Walker reads disk → misses (or worse, rewrites a stale text the user has since edited).
2. **Post-cascade stomp.** Walker rewrites source on disk; the source tab's NEXT autosave (with its still-pre-cascade in-memory copy) overwrites the cascade's rewrite, silently undoing it.
3. **Watcher loop.** Walker's `fs::write` bubbles back through the file watcher as an "external edit," racing the editor's read-back.

The original §3 only addressed (3). All three must be solved or the cascade is unreliable whenever the source is open — i.e. for the most realistic usage pattern.

**What — three coordinated changes:**

**(a) Flush-before-cascade (frontend).** In `+layout.svelte::handleRenameComplete`, before calling `updateLinksOnRename`, force-flush every open tab in the affected library:

```typescript
import { flushAllTabsInLibrary } from '$lib/libraries/store';
// new helper: iterate openTabs, call doFlush() on each tab whose path
// startsWith(libraryPath). Awaits all writes + reindex.
await flushAllTabsInLibrary(lib.path);
await updateLinksOnRename(lib.path, oldName, newName);
```

`flushAllTabsInLibrary` is a new exported helper that walks `get(openTabs)`, picks tabs in the library, and resolves their `setWriteAhead → writeNote` chain. Awaits completion. After this returns, every dirty tab in the library is on disk and the walker reads a consistent state.

**(b) Reload-after-cascade (Rust → event → frontend).** Cascade's IPC return value already carries `count: u32`. Extend it to a struct:

```rust
pub struct CascadeResult {
    pub rewritten: Vec<String>,  // absolute paths
    pub failed: Vec<(String, String)>,
}
```

After the cascade completes, the Rust side emits a Tauri event `cascade:rewrote { paths: Vec<String> }`. Frontend listener in `+layout.svelte` receives it and, for each rewritten path that's currently in `openTabs`, re-reads its content from disk and patches the tab's in-memory `content` field while preserving `cursorPos` / `scrollTop` / `historyIndex`. The editor's `{#key}` binding doesn't change because tab.id/path stay the same; the new content propagates via the `body = $derived(parseFrontmatter(tab.content).body)` chain. Specifically, `NotePane`'s `value` prop changes, and the existing prop-change handler (already used for second-screen sync) updates the CM6 doc via `view.dispatch({ changes: { ... } })` while preserving the selection.

This eliminates race (2): after reload, the tab's in-memory content equals the post-cascade disk content, so the next autosave is a no-op (or merges new user typing on top of the rewrite, never reverts it).

**(c) Watcher suppression (Rust).** Add `pub static RECENT_WRITES: Lazy<Mutex<HashMap<PathBuf, Instant>>>` in a new `watcher_suppress.rs` module. Two helpers: `mark(path)` and `was_recent(path) -> bool` with 2500 ms TTL. Cascade walker calls `mark` immediately before each `fs::write`. The watcher's emit path early-returns when `was_recent` is true, so the cascade's rewrites don't bubble back as "external" change events that would re-trigger our own reload logic.

**Input-block during cascade.** Between flush start and reload completion, the affected tabs' editors set `view.dispatch({ effects: EditorView.editable.of(false) })` to block keystrokes. Released on `cascade:rewrote` (or after a 5 s timeout for safety). Cascade is a deliberate user action; a brief input block is acceptable, and prevents any keystroke landing in the flush→write→reload window from being lost.

**Why** — Invariants 5, 14, plus the new "open-editor coherence" requirement.

**Verification (the test that has to pass):**
1. Open `§2 Round5`. Edit body to `Link me to [[<targetCurrentTitle>]]`. Don't switch tabs.
2. From the file tree (with focus still on `§2 Round5`'s editor), rename the target to a new title.
3. **Without clicking anywhere**, the editor's wikilink visibly updates to `[[<newTitle>]]`. Cursor and scroll position unchanged. No second autosave reverts it.
4. Close and reopen Constellation. Confirm the rewrite persisted.

**Risk**
- **Flush failure mid-batch**: one tab's flush throws → cascade aborts, surfaces error. No partial damage (renameItem already ran, but cascade gracefully short-circuits with toast: "Rename succeeded but link cascade was skipped — retry?").
- **Reload races user typing**: covered by input-block.
- **TTL too short for slow disks**: 2500 ms covers two debounced autosave cycles + safety margin; bench on a 7600-note universe before locking the constant.

**Rollback** — Revert the §3 commit. (a), (b), (c) ship as one cohesive commit since they only make sense together.

---

### §4 — Reindex each rewritten source via `index_note`

**What**
- Inside `update_links_recursive`, after a successful `fs::write`, push the path into a `Vec<PathBuf> rewritten_paths`. After the walk completes, the outer `update_links_on_rename` opens a single connection and calls `index_note(conn, path, library_name)` for each, in batches of 25 wrapped in `BEGIN/COMMIT`.

**Why** — Invariant 15.

**Verification** — After rename + cascade, `SELECT target_name FROM note_links WHERE source_path = '<rewritten>'` shows `new_title`.

**Risk** — `library_name` not currently passed. Mitigation: extend the IPC signature to `update_links_on_rename(library_path, library_name, old_name, new_name)`.

**Rollback** — Revert; alias-aware reads from MIG-004 keep correctness intact.

---

### §5 — `/simplify` checkpoint

**What** — Audit §1–§4. Look for:
- Duplicate logic between original and rewritten walker.
- Whether `read_note_title` overlaps with existing frontmatter parsers (`bases.rs:134`, `search.rs:1326`).
- Whether `recent_writes` belongs in `lib.rs` or its own file.

**Verification** — `/simplify` produces an empty diff or one focused refactor commit.

---

### §6 — Pre-flight count + sync/async dispatch

**What**
- New helper `count_link_occurrences(library_path, old_name) -> u32`. `update_links_on_rename` becomes a thin dispatcher: ≤100 → existing in-process flow; >100 → spawn a blocking task with a fresh `rename_id: String`, return immediately with `{ mode: "async", rename_id, total }`. A Rust `AtomicBool` per `rename_id` lives in a module-level `Mutex<HashMap<String, Arc<AtomicBool>>>` for cancellation.

**Why** — Invariants 3, 10.

**Verification** — Rename a synthetic 200-inbound note; UI stays responsive; toast appears with progress.

**Risk** — Blocking thread starves Tauri's runtime under repeated cascades. Mitigation: serialize cascades through a single mpsc — only one cascade walks at a time.

**Rollback** — Revert; cascade is sync-only.

---

### §7 — Progress, cancellation, completion events + frontend toast

**What**
- Tauri events `cascade:progress`, `cascade:done`, `cascade:cancelled`. New `CascadeToast.svelte` subscribes via `listen()` in `+layout.svelte`. Cancel button invokes `cancel_cascade(rename_id)` which flips the `AtomicBool`. Done shows file count; failures show "Show details".

**Why** — Invariants 3, 10.

**Verification** — Cancel mid-cascade — already-rewritten files stay rewritten, walker stops cleanly.

**Risk** — Leaking `AtomicBool`s. Mitigation: drop the entry in `cascade:done` and `cascade:cancelled` paths.

**Rollback** — Revert; sync path still works for ≤100.

---

### §8 — `/simplify` checkpoint

**What** — Audit §6–§7. Specifically: should `count_link_occurrences` and the rewriter share their walk? (Yes — extract a single visitor that takes a `Mode::Count | Mode::Rewrite` enum.) Is the toast doing too much?

**Verification** — Empty diff or one refactor commit.

---

### §9 — Atomic per-file writes via tempfile

**What**
- Replace `fs::write(&path, updated)` with tempfile + atomic rename (`atomicwrites` crate or hand-rolled `path.tmp` + `fs::rename`).

**Why** — Invariants 5, 7.

**Verification** — Kill the process mid-cascade; inspect on disk — every file is either fully old or fully new, none half-written.

**Risk** — Atomic rename on Windows fails if another process holds the file. Mitigation: catch error, add to `failed[]`, continue.

**Rollback** — Revert to direct `fs::write`.

---

### §10 — Pre-MIG-006 backfill command

**What**
- New Tauri command `cascade_backfill_stale_wikilinks(library_path) -> BackfillReport`. Implementation:
  1. `SELECT path, alias_lower FROM note_aliases WHERE source='rename'`.
  2. For each row, resolve the current human title for `path`.
  3. If `alias_lower != normalize(current_title)`, run the §2 walker for `[[alias]] → [[current_title]]`.
  4. Reindex via §4.
- **UI**: Settings → Files, "Rewrite stale wikilinks…" button under the `autoUpdateLinks` toggle. Always async.

**Why** — Invariant 16.

**Verification** — Vault with known stale `[[old]]` body reference (alias row exists, body still says old). Run backfill. Body now says `[[new]]`; `note_links.target_name = new`.

**Risk** — Huge backfill time for old vaults. Mitigation: opt-in and async; user sees progress and can cancel.

**Rollback** — Revert; alias-aware reads from MIG-004 still answer correctly.

---

### §11 — Phase 4 audit + lessons forward

**What** — Final commit:
- README/CLAUDE.md updates noting MIG-006 is live, the threshold, the cancellation contract, and the backfill command.
- Audit pass: confirm Invariants 1, 4, 5, 7, 10, 12, 14, 15, 16 are exercised.
- Lessons forward to MIG-005 (queued): `recent_writes` Rust map (§3) and cascade dispatcher (§6) are load-bearing primitives MIG-005 will reuse.
- Lessons forward to MIG-003: §1's frontmatter-title lookup remains correct; once filenames *are* the human title, lookup becomes a no-op for new notes but still required for legacy canonical-named notes.

**Verification** — Phase 4 audit passes; this doc updated with "Status: Build complete".

**Rollback** — Docs revert only.

---

## Lessons forward

**To MIG-005**:
- Reuse the Rust `recent_writes` map (§3) — any future multi-file mutation must mark before write.
- Reuse the cascade dispatcher (§6) — sync ≤100, async with progress events above. Threshold may differ per migration.
- Reuse the per-file atomic write (§9). Don't ship multi-file mutations without tempfile semantics.

**To MIG-003** (human-name filenames):
- §1's frontmatter-title lookup remains correct: for legacy canonical files, the title isn't the filename. Don't simplify §1 away when MIG-003 lands.
- MIG-003's rename will trigger MIG-006 cascade automatically — verify the `oldName` derivation still does the right thing when filename and title coincide.

## Critical files for implementation

- `src-tauri/src/libraries.rs` (walker, dispatcher, backfill command)
- `src-tauri/src/search.rs` (extract_wikilinks reference, index_note for §4 reindex)
- `src-tauri/src/lib.rs` (file-watcher emit path, RECENT_WRITES global)
- `src/routes/+layout.svelte` (oldName fix at L3788, cascade toast subscription)
- `src/lib/libraries/store.ts` (recentWrites parity, getOldTitleForCascade helper, updateLinksOnRename signature)

**Next**: Phase 3 — Build, starting with §1.
