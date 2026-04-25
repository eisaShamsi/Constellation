# Session Log — 2026-04-25

## §112 — BUG-012 fix: title-leak via mountedFilePath staleness guard

**Commit**: `952f3c7`
**Files**: `src/lib/components/NotePane.svelte`, `src/lib/components/NoteEditor.svelte` (+20 / −5)

### What broke

Discovered during MIG-006 §1 testing. User attempted to replace
`Link me to [[§2 Round3]]` with `Link me to [[§2 Round3_v3]]` in
`§2 Round5`'s body. The editor jumped to the target (`§2 Round3_v3`),
but the target's frontmatter title was rewritten to the source's
(`§2 Round5`), and the target's prior titles (`§2 Round3`,
`§2 Round3_renamed`, `§2 Round3_v3`) ended up stamped into its
aliases. Two notes (`NOTE_531D` and `NOTE_EE1E`) ended up with the
same `id='§2 round5'`. Same class as BUG-002/003 (display-sync
race), but on the *write* path, not the display path.

### Root cause

When a `[[wikilink]]` click reuses the active tab in-place,
`openNoteTab` swaps `tab.path` / `tab.name` to the target, the
`{#key tab.id + '|' + tab.path}` block in `NoteEditor.svelte`
remounts `NotePane`, and the OLD `NotePane` runs `onDestroy`. A
final blur on the OLD title `<input>` fires `ontitlechange` —
but `ontitlechange(titleValue)` was emitted with no snapshot of
which file this title belonged to. `handleTitleChange` then read
the LIVE `tab.path` (now the TARGET) and called
`renameItem(target_path, source_title.md)`. In canonical mode
that runs `update_frontmatter_title` on the target file: title
overwrite + alias stamp.

`handleSave` / `handleFlush` already had a `mountedFilePath`
snapshot + `filePath !== tab.path` guard for exactly this race
on the body-write path. The title-write path lacked the matching
guard.

### Fix

- `ontitlechange` signature widened to `(newTitle, filePath)`.
  `NotePane.handleTitleBlur` now passes its `mountedFilePath`
  snapshot.
- `NoteEditor.handleTitleChange` bails when
  `filePath !== tab.path`. All file-path uses inside the handler
  read the snapshot, not the live `tab.path`, so even if the
  guard were ever loosened the rename target couldn't be
  misdirected.

### Verification

- `svelte-check`: 0 errors, 284 warnings (unchanged).
- User test: `[[§2 Round3]]` → `[[§2 Round3_v3]]` swap in
  `§2 Round5`. Editor jumps. Target title remains
  `§2 Round3_v3`, target aliases unchanged, source `§2 Round5`
  unchanged. **Pass.**
- Cosmetic follow-up logged: sidebar's "active item" highlight
  lags ~10s behind the navigation. Benign — no disk write, no
  reactive corruption — but worth a separate cleanup pass after
  MIG-006 closes.

### Why this matters for MIG-006

MIG-006 rewrites wikilinks across notes. Until this guard
landed, every `[[wikilink]]` typed during cascade testing risked
silently mutating the target's frontmatter title — turning a
benign rename test into a duplicate-id corruption that took
manual SQL surgery to recover from. With BUG-012 fixed, MIG-006
§1 verification can resume safely.

### Pending after this commit

- Resume MIG-006 §1 GUI verification.
- MIG-006 §2-§11.
- MIG-005 (alias-aware in-memory inbound consumers, deferred
  per user direction).
- MIG-002 §7-§10 (enrichment_worker drain loop).
- MIG-003 (human-name filenames).
- Cosmetic: sidebar active-item sync lag (~10s after wikilink
  navigation).

---

## §113 — MIG-006 §1 GUI verification + plan §3 expansion

**Files**: `lab/reports/MIG-006-WIKILINK-CASCADE.md` (plan update only — no code changes this commit)

### §1 verified working

After BUG-012 unblocked safe wikilink testing, ran §1 (lift `oldName`
from frontmatter) end-to-end through the GUI:

1. Source `§2 Round5` body edited to `Link me to [[§2 Round3_v5]]`,
   flushed via tab switch.
2. Target renamed via file tree: `§2 Round3_v5` → `§2 Round3_v6`.
3. Disk re-read confirmed body now `Link me to [[§2 Round3_v6]]`.

The walker received correct args (`old_name="§2 Round3_v5"`,
`new_name="§2 Round3_v6"`), visited every `.md` in the universe,
and rewrote the matching source — proving §1's `oldName` lift
fixed the call site. Verified via temporary Rust + JS
instrumentation (cascade-debug.log written to `%TEMP%`); both
instrumentation patches reverted before commit so the working tree
is back to the §111 state for app code.

### BUG-013 — open-editor cascade race (separate from §1 itself)

User-reported during verification: cascade fails when the source
note is open and the user renames the target without first
switching tabs. Three races identified:

1. **Pre-cascade staleness** — source's in-memory edits not yet
   flushed to disk; walker reads stale text.
2. **Post-cascade stomp** — walker rewrites disk; source tab's
   next autosave (with pre-cascade in-memory text) overwrites
   the cascade's update, silently undoing it.
3. **Watcher loop** — walker's `fs::write` bubbles back through
   the file watcher as an external edit, racing the editor's
   read-back.

The original MIG-006 §3 plan ("Rust-side `recent_writes`
suppression for the file watcher") only addressed (3). All
three must be solved or the cascade is unreliable for the
realistic usage pattern (user is reading/editing a note,
realises the linked target needs a rename, renames it).

### Plan update — §3 expanded

`MIG-006-WIKILINK-CASCADE.md` updated:

- §3 row in Phase 2 plan table widened to:
  *"Open-editor coherence: flush-before-cascade +
  reload-after-cascade + `recent_writes` watcher suppression"*.
- §3 detailed section rewritten to specify three coordinated
  pieces:
  - **(a) Flush-before-cascade (frontend)**: new helper
    `flushAllTabsInLibrary(libraryPath)` in
    `src/lib/libraries/store.ts`, awaited in
    `+layout.svelte::handleRenameComplete` before
    `updateLinksOnRename`.
  - **(b) Reload-after-cascade (Rust→event→frontend)**:
    `update_links_on_rename` returns `CascadeResult` with
    `rewritten: Vec<String>`; emits `cascade:rewrote { paths }`
    Tauri event; frontend listener re-reads each rewritten
    path's content into the open tab's `content` field while
    preserving cursor/scroll/history.
  - **(c) Watcher suppression (Rust)**: original §3 content
    moved here unchanged.
  - **Input block during cascade**: editor `editable.of(false)`
    between flush-start and reload-complete (5 s safety
    timeout) so no keystroke is lost in the flush→write→reload
    window.
- Verification clause now: "open source, rename target without
  clicking away, wikilink updates in-place, cursor/scroll
  preserved, no autosave reverts it."
- All three pieces ship as one cohesive commit — they only make
  sense together.

### Why this matters

The cascade as it stands today (post-§1) works *if the user
remembers to switch away from the source before renaming the
target*. That's an unacceptable UX rule — it's the kind of
hidden constraint that turns a feature into a bug report.
Expanded §3 makes the cascade fire reliably regardless of which
tab the user happens to be looking at.

### Pending after this commit

- MIG-006 §2 (walker correctness pass — regex, transcludes,
  link-type preservation).
- MIG-006 §3 expanded (per the rewritten plan).
- MIG-006 §4-§11 unchanged.
- BUG-014 logged: `NOTE_EE1E` has an orphaned `cid_cn` value
  pointing at `NOTE_531D` from BUG-012's manual recovery —
  cosmetic only (cid_cn is never used as an identity key
  outside legacy migrations) but should be cleaned up.
- Cosmetic: relocate "Auto-update links on rename" toggle from
  Sky View & Links to Knowledge Management.
- Cosmetic: sidebar active-item sync lag.

---

## §114 — MIG-006 §2: regex-based walker correctness pass

**Commit**: `37ee40d`
**Files**: `src-tauri/src/libraries.rs` (+142 / −15)

Replaced `String::contains` + `replace` in `update_links_recursive`
with a single compiled regex `\[\[(escaped_old)(\]\]|\|)`. Trailing
delimiter (`]]` or `|`) captured and re-emitted, preserving every
tail (`|display`, `|link-type`, `|alias|link-type`). Embeds
(`![[X]]`) flow through naturally because the regex anchors on
`[[` — the leading `!` is never matched. `regex::escape(&old_name)`
keeps titles with metacharacters safe (`§2 Round3`, `Foo (bar)`,
`a.b`). Regex compiled once per cascade, threaded down into the
recursion.

11 unit tests pinned in `cascade_walker_tests` covering all five
wikilink shapes, prefix collisions (`Foo` vs `Foo Bar`),
metachars-in-title, Arabic titles, the §1 verification corpus.
All pass. GUI-untested for transcludes / typed-link tails on real
notes (covered only by unit tests).

---

## §115 — MIG-006 §3 expanded: open-editor coherence — INTRODUCED BUG-015

**Commit**: `3c4732d` (on `main`)
**Status**: ⚠️ **Carries data-corruption vector. Do not pull this build.**

Three coordinated changes shipped as one commit:
1. `RECENT_WRITES` watcher suppression in `watcher.rs` (2500 ms TTL).
2. `update_links_on_rename` returns `CascadeResult { rewritten,
   failed }` and emits `cascade:rewrote { paths }` Tauri event.
3. Frontend: `flushAllEditorsInLibrary` registry, `cascadeInProgress`
   input-block, `cascade:rewrote` listener that reloads each
   rewritten path's `tab.content`, **and a value-prop → doc sync
   `$effect` in NotePane** that dispatched a CM6 doc-replace
   transaction whenever the parent's `body = $derived(parseFrontmatter(tab.content)).body`
   changed without a tab-path change.

The value-sync `$effect` raced with `{#key tab.id+'|'+tab.path}`
onDestroy during ordinary tab navigation. When the user clicked
from source to target in the file tree, `tab.content` updated to
target's content; reactivity propagated `body` to target_body;
the OLD source NotePane's value-sync $effect fired BEFORE its
onDestroy, replacing its own CM6 doc with target_body; then
{#key} ran destroy → doFlush() read the swapped doc → handleFlush
wrote target_body to source path (or vice versa depending on
direction). The bail guard `filePath !== tab.path` did not catch
it because the captured `mountedFilePath` matched the snapshot
but the doc had been swapped between source and target via the
$effect.

Real symptom observed: typing `qux` in source `§2 Round5`, then
clicking target in tree, then renaming target via right-click →
both files end up with identical bodies on disk. Confirmed via
`%TEMP%\bug015.log` instrumentation.

---

## §STATE-OF-STANDING — 2026-04-25 evening

This record is mandated by Standing Order #5 (CLAUDE.md). Future
sessions reading this should rely on this snapshot as the
authoritative state at this point in time.

### A — Top principal rule installed

`CLAUDE.md` Working Agreement #4: **never ship changes whose
architectural blast radius hasn't been validated.** Before any
code change touching write paths / lifecycle / reactivity / IPC
contract: spawn parallel review agents (Explore, Plan,
code-reviewer) to map call graph + consumers + invariants, write
the impact in advance, surface unmappable risk to the user and
stop. Speed never overrides preservation. The MIG-006
§3-expanded → BUG-015 incident is the canonical violation this
rule prevents. Memory entry: `feedback_secure_dont_muddle.md`.

### B — Verified-shipped & protected (do not muddle with)

- **Cognitive Engine Layer 1** Phases 1–11 (typed links, strata,
  maturity, tension built-pending-large-test, provenance,
  externalization, review pulse, trails, multi-lens views,
  expression forge, sense-making canvas).
- **Living Link Architecture** P0–P5 (dual-layer storage, 7
  cognitive operators in 15 locales, traversal tracking,
  confidence levels, weight + 6-stage lifecycle, formulation
  queries, knowledge-health dashboard, archive/unarchive,
  annotation display, lifecycle decay knob).
- **Constellation Arabic Engine** M3 / M3-baker / M5 / M6 / M7 /
  M8 / M8b / M8c / M9 / M10 / M11-infra / M12 / M12-detect /
  M12-bench / M13 / M14 — all shipped + tested.
- **M11-data v1 + v2 Producer** — v2 reached 20 K target per user
  (499 shards in `lab/m11-data/concepts/`).
- **MIG-001 Sky View Write-Time Derivation** — closed.
- **MIG-004 Alias-Aware Resolution** — closed.
- **MIG-006 §1 (oldName from frontmatter)** — verified.
- **§112 BUG-012 fix (title-leak)** — verified.
- **§114 MIG-006 §2 (regex walker)** — shipped, 11 unit tests
  pass, GUI edge cases untested.
- **Boot Performance** Criteria 1 + 2 — verified production
  (`5cb4f94` boot bundle, 1 s paint / 8 s hydrated).
- **Panel Placement Tier 1 + 1b** — shipped.
- **WTD audit closures** — Sky View, Backlinks/Outgoing, Tag
  browser.

### C — At-risk / in-flight / uncommitted

- **`§115 / 3c4732d` is on `main` and carries BUG-015.** Anyone
  pulling `main` inherits the corruption vector.
- **Worktree (`E:\مشاريع كلاود\Constellation\.claude\worktrees\frosty-stonebraker-75c9bf`)
  has uncommitted strip changes** that remove the value-sync
  `$effect`, `cascade:rewrote` listener, and BUG-015
  instrumentation. Untracked files: `src/lib/editor/flushRegistry.ts`,
  `lab/forensics/BUG-015-*-snapshot.md`, `dev/`. The strip
  preserves the watcher suppression, flush registry, and
  `cascadeInProgress` input-block as the safe pieces of §115.
- **The strip is unverified** — never built, never user-tested.

### D — Known-broken (bugs identified, not yet fixed)

- **BUG-013** open-editor cascade race — re-opened by BUG-015's
  strip; effectively unresolved. The "rename target while
  source is visible" scenario will require switching tabs first
  for the cascade to be reliable.
- **BUG-014** orphaned `cid_cn` in `NOTE_EE1E` (collateral from
  BUG-012's manual recovery). Cosmetic.
- **BUG-015** target-body corruption from §115's value-sync
  `$effect`. Root-caused via `bug015.log`. Strip exists in
  worktree but uncommitted.
- **Title-heading rename gap**: `NoteEditor.handleTitleChange`
  does not call `updateLinksOnRename`. Only file-tree rename
  triggers the cascade.
- **Cosmetic**: "Auto-update links on rename" toggle is in Sky
  View & Links section (should be Knowledge Management).
- **Cosmetic**: sidebar active-item highlight lags ~10 s after
  wikilink navigation.

### E — Real disk corruption (collateral from BUG-015)

- `E:\Constellation Universes\Eisa Cognitive Knowledge\20260424T063440Z_NOTE_531D.md`
  (target "§2 Round3_vEisa2") — body replaced with source's
  body. No pre-corruption snapshot.
- `E:\Constellation Universes\Eisa Cognitive Knowledge\20260424T092445Z_NOTE_EE1E.md`
  (source "§2 Round5") — orphan `cid_cn` from BUG-012 manual fix
  (cosmetic).
- User's `Source Note v1` / `Target Note v1` test notes — same
  corruption from BUG-015 reproduction. Can be deleted.

Forensic snapshots saved to `lab/forensics/`.

### F — Pending / not started

**Cognitive Engine Layer 3 (5 phases):** 12 Hidden Pattern
Discovery / 13 Blind Spot Detection / 14 Cross-Domain Insights
/ 15 Socratic Challenger / 16 Worldview Synthesis.

**Migrations:** MIG-002 §7–§10 (enrichment_worker drain loop +
derives-from triggers + frontend swap + audit). MIG-003
human-name filenames (not started; canonical-stem readability
pain reported by user). MIG-005 alias-aware in-memory inbound
consumers (deferred from MIG-004 audit 4B-1, 4B-2). MIG-006
§3 expanded redo / §4–§11.

**Boot performance:** Criterion 4 (post-UI sync sweep),
Criterion 5 (kill-mid-index recovery), stats persistence,
Settings → Rebuild Index button, CHANGELOG entry, version bump
to 0.4.0, help docs, release-run boot-perf trace, Settings →
Debug Boot Performance scorecard UI.

**Panel Placement:** Tier 2 (drag-and-drop), Tier 3 (detachable
multi-window), functional walkthrough.

**WTD audit follow-ons:** Sight dashboard (ConstellationSight2
recomputes on each toggleLens), sidebar star counts, Map.

**Arabic Engine perf follow-ons** (queued from M9): string-intern
`pattern_label`, mmap FST bytes, trim per-call `Arc::clone` on
`ACTIVE_STORE`. Aspirational throughput ≥ 200 K w/s (today
~130 K), aspirational cache ≤ 10 MiB at 7K-root scale (projected
~90 MiB today).

**M11-data follow-ons:** synonyms (sense-tagged in-language
near-equivalents), domains (science / philosophy / arts / Islamic
studies / medicine packs).

**Misc:** `__navTrace` instrumentation dev-gate, isolated
throttle stress-test helper, RTL alignment verification on
Arabic docx, "note as organism" editor redesign (design-only).

### G — Documentation drift

- `docs/cognitive-engine-roadmap.md` Phases 6–11 listed twice
  with conflicting status (clean rows on top, "🔲 Not started"
  rows below).
- `lab/m11-data/README.md:3` Status field still says "v2
  (in-flight, scaling toward 20K)" — corpus has reached 20K.
- No central project-wide tracker. Status spread across
  cognitive-engine-roadmap, per-MIG plans, per-component READMEs,
  per-session logs. Recommend `lab/reports/STATUS.md` as the
  one-stop pointer.

### H — Direction set by user this session

- Top principal rule installed.
- "Fix the essential first" — interpreted as: **address
  Section C (BUG-015 / `3c4732d` on main) and Section E
  (corrupted notes) before any new work.**
- All other priorities (Section F) deferred until C + E are
  clean.
- Standing Order #5 added (this record) to make state-of-standing
  snapshots a recurring discipline, not an ad-hoc rescue.
