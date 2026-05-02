# MIG-006 §3 redo — Architect

**Status**: Phase 1 (Architect) — awaiting Boss Phase 2 sign-off.
**Supersedes**: the §3 plan in `MIG-006-WIKILINK-CASCADE.md` lines 133-188 (commit `3c4732d`, reverted at `5afe0c2`).
**Anchored against**: [`docs/Rename-Function-Concept-Paper-v1.0.md`](docs/Rename-Function-Concept-Paper-v1.0.md).

---

## Why this exists

The original §3 plan documented "the existing prop-change handler (already used for second-screen sync)" as the mechanism that would update the CodeMirror doc when the cascade finished. **That handler did not exist.** The §115 commit invented one: a `$effect` in NotePane that watched the parent's `value` prop and dispatched a doc-replace transaction when it changed. That `$effect` raced with the `{#key tab.id+'|'+tab.path}` `onDestroy` on tab navigation and corrupted target body content with source body content. BUG-015. Reverted at §116.

The plan misled itself by referencing a fictional artifact. The redo binds itself to the Rename Function Concept Paper — a document that defines what Rename **is** before it specifies what §3 does. Every claim in this architect plan has to be checkable against the concept paper or against existing code.

The lesson the concept paper crystallises (Principle D6): **the rename + reload pipeline must NEVER use `$effect` to sync editor body content from a parent prop.** Two acceptable mechanisms — `{#key}` recreation, or imperative `view.dispatch` through a known ref. Never an `$effect`-driven dispatch.

---

## What's alive vs missing

| Step (per `MIG-006-WIKILINK-CASCADE.md`) | Status |
|---|---|
| §1 — `oldName` from frontmatter title | ✅ shipped, alive |
| §2 — walker correctness pass (regex; all 5 wikilink shapes) | ✅ shipped + 11 cascade tests |
| **§3 — open-editor coherence (a + b + c)** | ❌ REVERTED at `5afe0c2`. None of (a), (b), (c) is in code today. |
| §4 — reindex via `index_note` | ⏸ pending |
| §5 — `/simplify` checkpoint | ⏸ pending |
| §6 — pre-flight count + sync/async dispatch | ⏸ pending |
| §7 — progress + cancel + completion events | ⏸ pending |
| §8 — `/simplify` checkpoint | ⏸ pending |
| §9 — atomic per-file writes via tempfile | ⏸ pending |
| §10 — pre-MIG-006 backfill command | ⏸ pending |
| §11 — Phase 4 audit + closure | ⏸ pending |

This redo scopes **§3 only**. §4–§11 follow once §3 lands clean.

---

## The three changes that §3 must deliver

Mapped to the concept paper's Principle D2 (open editors are the most fragile state) and the three named failure modes:

| Sub-step | Closes failure mode | Concept-paper anchor |
|---|---|---|
| **(a) Flush-before-cascade.** Frontend helper `flushAllTabsInLibrary(libraryPath)` walks `openTabs`, picks tabs in the affected library, awaits each one's flush chain (debounced autosave → write → reindex). Called before `updateLinksOnRename`. | F2-pre-cascade-staleness | P4 (open-editor coherence), D2-pre-cascade-staleness |
| **(b) Reload-after-cascade.** Rust emits `cascade:rewrote { paths: Vec<String> }` after the walker completes. Frontend listener updates each affected open tab so its in-memory state matches the post-cascade disk content. **The reload mechanism is the design choice for this redo** — see "Reload mechanism options" below. | F2-post-cascade-stomp | P4 (open-editor coherence), D2-post-cascade-stomp, D6 (reactive coherence over reactive convenience) |
| **(c) Watcher suppression.** Rust `RECENT_WRITES: Lazy<Mutex<HashMap<PathBuf, Instant>>>` in a new `watcher_suppress` module. Two helpers — `mark(path)` and `was_recent(path) -> bool` — with a 2.5 s TTL. Cascade walker calls `mark` immediately before each `fs::write`. The watcher's external-edit emit path early-returns when `was_recent` is true. | F3-watcher-loop | P4 (open-editor coherence), D2-watcher-loop |

(a) and (c) are uncontroversial — the original §3 plan got them right; they didn't ship because the bundle reverted as one. **(b) is the design call this Architect proposal asks Boss to make.**

---

## Reload mechanism options for (b)

Three options that satisfy D6 (no `$effect` on `value`/`editBody`). The fourth and fifth options from the previous turn (Disk-first reload via existing reactivity, Toast + manual reload) are excluded — the first is the BUG-015 path; the second fails the §3 verification test ("without clicking anywhere, the wikilink visibly updates").

### Option A — Tab-key invalidation (recreate)

On `cascade:rewrote { paths }`, for each affected open tab, the parent layout bumps the tab's `{#key}` value. Svelte destroys the existing NotePane and re-mounts it. The new mount reads fresh `tab.content` from disk and instantiates a fresh CodeMirror EditorView with the post-cascade content.

**Pros:**
- Smallest mechanism. Reuses existing `{#key}` patterns.
- No new ref-passing, no new lifecycle coordination.
- Strictly D6-compliant — no `$effect`, no imperative dispatch into a possibly-destroying view.

**Cons:**
- Cursor position lost (jumps to top of doc).
- Scroll position lost.
- Brief visual flicker (mount + paint).

**Trade-off framing:** the user just performed a deliberate rename. Losing cursor on the few open tabs that share the renamed-target's library is an acceptable cost for a deliberate cognitive action.

### Option B — Imperative dispatch via ref

NotePane exposes its `EditorView` to the parent via a ref callback prop (e.g. `bindEditor: (view: EditorView) => void`). On `cascade:rewrote`, the parent's listener calls `view.dispatch({ changes: { from: 0, to: view.state.doc.length, insert: newBody }, selection: view.state.selection })` directly on each affected tab's view. No `$effect`. Cursor/selection preserved by re-applying the existing `state.selection` after the doc replace.

**Pros:**
- Cursor and scroll preserved.
- Best UX.
- Strictly D6-compliant — imperative call, not `$effect`.

**Cons:**
- New ref-passing pattern from NotePane to layout.
- Lifecycle ordering must avoid dispatching into a view that's about to destroy. The cascade event fires after the cascade IPC returns; the `{#key}` chain is stable through the cascade unless the user navigates tabs mid-cascade. Concrete safety: gate the dispatch on `view.dom.isConnected` and silently skip if false.
- More moving parts to verify in audit.

**Trade-off framing:** if cursor preservation is part of the function's promise (it isn't, in v1.0 of the concept paper — P4 only requires "no typing in flight is lost, no stomp, no ghost characters"), this is the upgrade path.

### Option C — Programmatic close + reopen

On `cascade:rewrote`, for each affected open tab, programmatically close it and reopen with fresh disk content.

**Pros:**
- Simplest possible state machine.
- D6-compliant.

**Cons:**
- Tab order may shift.
- Focus jumps.
- Dirty-tab handling is awkward (would need a force-close that bypasses the unsaved-changes prompt, since flush-before-cascade already wrote it).
- Worst UX of the three.

**Rejected** — the UX cost is higher than Option A's cursor loss for no design gain.

---

## Recommendation

**Option A** for the §3 redo.

**Why:**
- It is the smallest mechanism that closes BUG-015's class of failure.
- It is strictly D6-compliant by construction (no `$effect`, no imperative dispatch into a view).
- It satisfies the §3 verification test from the original plan: "Without clicking anywhere, the editor's wikilink visibly updates to `[[<newTitle>]]`."
- The cursor/scroll loss is acceptable for a deliberate rename — the concept paper's P4 invariant requires *coherence*, not preservation.
- Option B is a strict superset; the work for A wastes nothing if Boss later requests cursor preservation.

**Open question for Boss:** is Option B preferred for v1, accepting the additional verification surface? Or is Option A acceptable for v1 with B as a future enhancement triggered by user feedback?

---

## Invariants the §3 redo must preserve

From the concept paper's P1–P8 plus the migration's existing invariants:

1. **P3 — Body content sacred.** The cascade's reload mechanism never mutates body content beyond what the walker wrote. Specifically: no `$effect`-driven body sync (D6).
2. **P4 — Open-editor coherence.** The §3 verification test from the original migration plan must pass. Verbatim: "Open `§2 Round5`. Edit body to `Link me to [[<targetCurrentTitle>]]`. Don't switch tabs. From the file tree (with focus still on `§2 Round5`'s editor), rename the target to a new title. **Without clicking anywhere**, the editor's wikilink visibly updates to `[[<newTitle>]]`. No second autosave reverts it."
3. **No regression in §1 (frontmatter-title) or §2 (walker correctness).** The redo is additive to those steps.
4. **No double-fire on rename.** Cascade runs exactly once per rename. The `cascade:rewrote` event fires exactly once.
5. **Watcher loop closed.** Cascade's `fs::write` does NOT bubble back as external-edit and re-trigger reload logic.
6. **Pre-cascade staleness eliminated.** Flush-before-cascade completes before walker reads disk. Concretely: every dirty tab in the affected library is on disk and the autosave-→-write-→-reindex chain has resolved.
7. **NotePane spec §2.6 unchanged.** No `$effect` is added that reads or writes `value` or `editBody`. Only allowed effects in NotePane stay: `dir` change (guarded by `prevDir`), font change (guarded by `prevFontKey`).

---

## Phase 2 plan (preview — formalise after Boss approves Option A or B)

If Boss approves Option A:

| Step | What ships | Verify |
|---|---|---|
| **§3-redo.1** | Frontend `flushAllTabsInLibrary(libraryPath)` helper in `store.ts`. Iterates `openTabs`, awaits each in-library tab's flush chain. | Unit test: dirty tab in library X, call helper, helper resolves only after every dirty tab in X is on disk. |
| **§3-redo.2** | Rust `watcher_suppress` module: `mark(path)` + `was_recent(path) -> bool`, 2.5 s TTL. Walker calls `mark` before each `fs::write`. Watcher's emit path early-returns when `was_recent`. | Test: cascade rewrites N files, watcher emits zero external-edit events for those N during the TTL window. |
| **§3-redo.3** | Rust `cascade:rewrote { paths: Vec<String> }` event emitted after walker completes. `update_links_on_rename` returns `CascadeResult { rewritten, failed }`. | Test: cascade rewrites 5 files, frontend listener receives `paths` array of length 5 with absolute paths. |
| **§3-redo.4** | Frontend `cascade:rewrote` listener in `+layout.svelte`. For each path in `openTabs` matching a rewritten path: re-read content from disk, update `tab.content`, **bump the tab's render key**. The NotePane destroys + remounts with fresh content. | The §3 verification test (above). Cursor jumps to top — accepted. |
| **§3-redo.5** | `handleRenameComplete` flow updated: `await flushAllTabsInLibrary(lib.path)` BEFORE `await updateLinksOnRename`. Input-block during the cascade window (between flush start and reload completion); `view.dispatch({ effects: EditorView.editable.of(false) })` for affected tabs. Released on `cascade:rewrote` or after a 5 s safety timeout. | Test: type into source tab while cascade runs — keystrokes blocked, no characters land in the swap window. |
| **§3-redo.6** | `/simplify` checkpoint. | Empty diff or one focused refactor commit. |
| **§3-redo.7** | Phase 4 audit (matches the original §11): invariant check (P1–P8), drift check (no new patterns the system doesn't know about — see LL-023), migration-path check (rename mid-cascade, partial cascade with failures, rename + Universe switch). | All audits pass. |

Step ordering matters. **§3-redo.1, .2, .3 are independent and can land in any order**. §3-redo.4 depends on .3. §3-redo.5 depends on .1 + .4. §3-redo.6 + .7 close the work.

If Boss approves Option B instead, §3-redo.4 changes:

| Step | Option B variant |
|---|---|
| §3-redo.4 (B) | NotePane exposes `EditorView` via `bindEditor` callback prop. Parent stores per-tab refs in a Map. On `cascade:rewrote`, listener calls `view.dispatch({ changes: { from: 0, to: view.state.doc.length, insert: newBody }, selection: view.state.selection })` for each path with a live ref. Skip if `view.dom.isConnected === false`. Then update `tab.content` to match. |

Cursor and scroll are preserved. Lifecycle safety added via `dom.isConnected` gate.

---

## Phase 1 sign-off question

Approve **Option A** (recreate via `{#key}`-bump — cursor lost, simplest mechanism) or **Option B** (imperative dispatch via ref — cursor preserved, larger surface) for the §3 redo? Then I write Phase 2 / Plan in full and we cascade through the build per Standing Order.
