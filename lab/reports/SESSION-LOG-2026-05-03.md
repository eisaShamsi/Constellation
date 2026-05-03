# Session Log — 2026-05-03

Continuation of yesterday's MIG-006 §3-redo testing arc. New file because the calendar date has rolled over from 2026-05-02. Stage 1-4 of the §3-redo test cycle PASSED yesterday; today picks up at Stage 5 prep, surfaces the breadcrumb-redundancy + Rule 8 drift items Boss flagged during testing, and lands the fixes.

---

## §136 — Stage breadcrumb redesign + handlePromote cascade gate

Boss observation during Stage 5 prep: the breadcrumb Stage dropdown duplicated the property panel — same control, two surfaces, no logical separation. Homework on commit history showed the original CE Phase 6 design (commit `87d21d7`, 2026-04-02 16:58) added Stage to the breadcrumb as a one-click `Promote →` *verb*, then commit `6cbe87c` (40 minutes later) silently refactored the verb into a property-selector dropdown ("no more typing exact values"). Boss's "not LOGICAL" critique was reading the post-refactor state correctly — the redundancy was real, but it was an artifact of the refactor losing the original verb-distinct intent, not of the original design.

§136 restores the verb-distinct design with one refinement (Boss decision): demotion is permitted, contradicting the CE-spec's "one-way" line. The CE-spec rule was an oversimplification — Luhmann himself revised notes; knowledge revision is a legitimate scholarly act. The visual treatment encodes the frequency asymmetry instead of the prohibition: Promote is the prominent canonical verb, demote is a subdued arrow-only affordance.

**Breadcrumb new shape**: `[← demote] [stage badge] [Promote →]`
- Promote → prominent (accent border, text + arrow)
- ← demote subdued (faint arrow, no border, tooltip-only label)
- At Fleeting, demote hidden (nowhere to go back)
- At Synthesis, promote hidden (nowhere to go forward)
- When no stage assigned, the strip is empty (initial assignment via property panel or FocusPane)

**Side fix — §134 audit drift the audit missed**: `NoteEditor.handlePromote` was the *other* surface bypassing the `isCascading` cascade gate (PropertyEditor.saveTabContent was the one caught in §134). It writes via `writeNote` directly using `freshBody()` (in-memory pre-cascade content). Added `if (isCascading(tab.path)) return` at the top.

After §136, all four edit surfaces share one consistent cascade gate:
- body save (handleSave) — gated since §132
- body flush (handleFlush) — gated since §132
- property save (saveTabContent) — gated since §134
- stage promote/demote (handlePromote) — gated this commit

**i18n**: `notePane.demote` added to all 15 locales. `notePane.promote` already existed from CE Phase 6.

**CE-spec.md Phase 6 updated** to reflect the bidirectional design, with the historical "one-way" line annotated.

**Orientation v1.29 created** alongside v1.28 documenting §135 + §136 per SO #6.

Commit: `90c1ea8`.

---

## §137 — Rename propagates to path-keyed reactive state (Rule 8)

Boss observation during Stage 5 testing: "we used to have the stage icon attached to the note title as a prefix… we want Constellation to do it instantly when the user promotes the note stage, demotes it, renames it, or re-renames it. That's why Constellation is unique and has its own prediction engine."

Audit confirmed Boss's instinct. Four reactive `Map<path, V>` `$state` in `+layout.svelte` drive the file-tree stage emoji, file-tree maturity dot, alias index, and search-hub link counts:
- `stageMap` — path → stage (CE Phase 6)
- `maturityMap` — path → maturity (CE Phase 3)
- `notePathToAliases` — path → alias[] (backlinks resolution)
- `searchLinkCounts` — path → {incoming} (search hub UI)

Promote/demote already kept them in sync via the `onStageChanged` callback chain. **Rename did not.** After a rename, the renamed file's old path stayed in every map as an orphan, and the new path had no entry — so the file tree showed the renamed note without its derived state until the next library scan. Direct violation of Rule 8 (Write-Time Derivation).

§137 closes the gap.

`src/lib/utils.ts` — new helper `migratePathKeyedMap<V>(map, oldPath, newPath): Map<string, V> | null`. Handles three cases atomically: direct file rename, folder-prefix rename (every key under the renamed folder rekeyed), and no-op canonical-file rename (returns `null` so the caller skips the store update entirely). Path normalisation matches the canonical key shape (forward-slash + lowercase).

`src/routes/+layout.svelte::handleRenameComplete` — captures the **effective path** returned by `renameItem` (correctly handles canonical-file renames where the disk path stays the same) and calls `migratePathKeyedMap` on each of the four maps. One Map allocation per map only when keys actually moved; identity-stable otherwise.

Commit: `de50ba3`.

---

## §138 — Stage + maturity load on library expand (deeper Rule 8 fix)

Boss tested §137 and reported: "the emoji is not visible, not before renaming or after it." The §137 path migration was correct but lit nothing because the upstream `stageMap` and `maturityMap` were both **empty on boot**.

Audit found the actual cause. `enrichNodesBackground` is the only path that populates these maps, and it was deliberately removed from the boot flow for boot-perf — see the "ZERO BOOT-TIME WALKS" comment at `+layout.svelte:2744-2757`. Before §138, the only triggers were the Sky View legend's `onRequestEnrichment` button, the Settings → Rebuild Index path, and the first-ever-launch modal. None of those fire on a normal boot, so the file tree never showed the stage emoji 🌱📖🔗✨ or the maturity dot ● — direct violation of Rule 8.

§138 adds a third trigger: when the user expands a library in the sidebar (`toggleLibrary`, first-expand only), fire `scan_note_stages` and `compute_note_maturity` for that library and merge results into the reactive `stageMap` / `maturityMap`. Fire-and-forget (the expand isn't blocked); maps are reactive `$state` so the file tree re-renders when each scan returns; mutation guard so no spurious reactivity on no-op merges.

Combined with §137, all four mutations Boss called out — promote, demote, rename, re-rename — now keep the file-tree indicators in lockstep with disk reality on a fresh boot, no manual rebuild required.

The architectural ideal — `stage` / `maturity` as columns in `note_meta` SQLite, populated by `index_note` on every write, queried at boot — remains queued (it's the MIG-006 §4 shape). §138 is the bridge: metadata is gathered eagerly per library on user demand, no schema changes required.

Commit: `be9df91`.

---

## Production binary build (in flight at log-write time)

Boss directive: "Let's work on a binary version. Build it." Production `tauri build` kicked off in the background to produce a release-mode bundle + installer. Verification builds (vite production frontend + cargo check) both passed clean before kicking off; svelte-check shows only the pre-existing deferred `LinkLifecycle 'fresh'` error.

Output destination (per Tauri default): `src-tauri/target/release/bundle/`. Will report binary path + size to Boss when build completes.

---

## §139 — Three production-binary bugs Boss caught

Boss installed the §138 production binary and reported three bugs from real-world testing:

1. **RTL arrow inversion** — Promote → / ← demote arrows had hard-coded characters, reading inverted in Arabic / Hebrew / Persian / Urdu note context. Fix: swap based on `dir === 'rtl'`.
2. **Folder children no emoji / dot** — `<svelte:self>` recursion in `FileTree.svelte:102` was missing `stageMap` + `maturityMap` from the prop list. Notes inside any folder rendered with default empty maps. Fix: pass both to the recursive call.
3. **Promote / add-Stage didn't update file-tree** — chain (handlePromote → onStageChanged → stageMap reassign) looked correct but file tree didn't re-render. Root cause: `$state(new Map())` + reassign-to-fresh-Map pattern has a Svelte 5 reactivity quirk visible specifically through the child-reads-via-prop path. Fix: switch `stageMap` and `maturityMap` to `SvelteMap` (Svelte 5's explicitly-reactive Map subclass). Mutations are reactive at the operation level — no reassign-to-force-identity dance needed. New helper `migratePathKeyedMapInPlace<V>` in utils.ts for the §137 site. Six existing call sites refactored to direct `.set()` / `.delete()`.

Commit: `d99476e`.

## Production binary builds (×2)

- **First build (after §138)**: `tauri build` produced MSI + NSIS + raw exe. All three artifacts in `src-tauri/target/release/bundle/`. Boss installed and tested.
- **Second build (after §139)**: same command. MSI + raw exe rebuilt cleanly. NSIS bundling failed at the very last step with `os error 32` — the NSIS `setup.exe` output path was locked by another process (probably Windows Defender or the previously-installed Constellation holding the file open). MSI is the artifact Boss used; raw exe also fresh. NSIS in `bundle/nsis/` is stale (still §138).

## Stage 5–6 — Boss test cycle closure

**Stage 5 — PropertyEditor + handlePromote cascade gate (§134 + §136)** — PASS. Verified that during a wikilink rename cascade window, frontmatter property edits and stage promote/demote don't stomp the cascade rewrite. Pre-cascade properties survive; post-cascade property editing still works (gate clears properly); cascade rewrites land cleanly without disturbing existing properties.

**Stage 6 — Spam-rename refcount (§135)** — PASS. Verified that two renames fired in rapid succession in the same library don't pop each other's cascade gates. Both cascades complete cleanly; all source bodies updated correctly; no bodies show mixed/corrupt content.

## MIG-006 §3 redo: fully closed

Every Concept Paper failure mode verified end-to-end by Boss test cycle:

| Stage | Failure mode | Status |
|---|---|---|
| 1 | Basic cascade (closed-note rewrite) | ✓ |
| 2 | F2 — open-editor coherence (headline win) | ✓ |
| 3 | F2 — pre-cascade staleness | ✓ |
| 4 | F3 — watcher-loop + multi-source perf | ✓ |
| 5 | F2 — post-cascade-stomp at all 4 edit surfaces | ✓ |
| 6 | F4 — spam-rename refcount race | ✓ |

Plus all four user-visible follow-ups (§136–§139) PASSED in production-binary testing.

## §140 — Cross-note content corruption via stale writeAheadBuffer

Boss reported a serious data corruption bug during normal Constellation use: "Sometimes, when switching between notes after renaming or creating notes, I discover that a note replicates its contents, title, and cid_cn into another note. The victim note keeps its title in the file tree, but when I click it, it shows the culprit note (title, content, and properties)."

Investigation pinpointed `writeAheadBuffer` (in-memory `Map<filePath, V>` + `localStorage` backup that survives app restarts). When a note is flushed, the editor's content is stashed under its file path so a later `openNoteTab` can substitute it for a disk read. **`renameItem` / `moveItem` / `deleteItem` migrate `openTabs.path` correctly but never touched the buffer.** When a path was reused after a rename or delete (trivial with human-named notes), `openNoteTab` hit the stale buffer entry and loaded the OLD note's content (cid_cn, title, body) into the new tab. The file tree kept showing the new note's correct title (driven by `display_title` from disk frontmatter — disk was correct) while the tab held the old note's content (in-memory only, until the user typed and triggered a `handleSave` that committed the corruption to disk).

Direct violation of Rule 8 (Write-Time Derivation) — same gap §137 closed for stage/maturity, except corruption-class severity (the BUG-015 lineage).

Three-part fix:
- New helpers `migratePathKeyedAuxStateOnRename` and `clearPathKeyedAuxStateOnDelete` in `store.ts` migrate / drop wab + recentWrites entries (in-memory + localStorage backup), with folder-prefix support for folder rename / delete.
- Wired into `renameItem`, `moveItem`, `deleteItem`.
- Defense-in-depth in `openNoteTab`: when a wab entry hits, also read disk and compare the `cid_cn` signature; on mismatch, prefer disk and clear the stale buffer. Self-healing for users with stale localStorage from prior sessions before the fix landed.

Boss tested the §140 production binary and confirmed PASS — reproduction (rename Foo v2 → Foo v3, create new Foo v2, click new Foo v2) now correctly shows the new note's content with its own cid_cn.

Commit: `2d40ccf`.

## §141 — /simplify checkpoint over §137-§140

Three review agents (reuse / quality / efficiency) walked the §137-§140 diff. Aggregated findings shipped as fixes:

**Reuse:**
- `normalizePathKey(p)` exported from `src/lib/utils.ts`. The `(p) => p.replace(/\\/g, '/').toLowerCase()` function was duplicated 7+ times across utils, store, +layout. Single source of truth for the path-key contract used by every reactive Map.
- `WAB_LS_KEY = 'constellation-wab'` constant in store.ts (was hard-coded in 5 places).
- Single `walkAuxStatePaths` walker shared by the §140 rename + delete helpers. Walker passes the ORIGINAL key to the decide callback so folder-rename suffix preservation works on case-mixed Windows paths.

**Quality:**
- `openNoteTab`'s wab/disk choice extracted to `resolveNoteContent(filePath)` helper. Returns `{content, cursorPos, scrollTop}`: when wab is stale, drops the wab cursor/scroll too (subtle correctness improvement the §140 inline code missed).
- `handleStageChanged(path, stage)` hoisted in +layout.svelte (3-line callback was inlined twice).

**Efficiency:**
- `extractCidCn` regex bounded to the first `---…---` frontmatter block (was scanning full body — material win for large notes).

**Style:**
- Stripped `// §139:` / `// §140:` inline anchor comments where they narrated what the code obviously does. Kept docstrings on function declarations.

Skipped: `migratePathKeyedMap` vs `migratePathKeyedMapInPlace` unification (two-API surface stays for clarity), toggleLibrary scan-merge unification (shapes differ enough), `<svelte:self>` 11-prop spread (explicit forwarding IS the documentation).

Commit: `42e9693`. Production rebuild kicks off after this commit.

## §142 (MIG-006 §4) — Reindex rewritten sources after cascade

Original gap from §3-redo Stage 1: after rename, Outgoing Links panel kept showing the OLD target name (`foo`, lowercased) because the body cascade didn't trigger reindex of the affected source notes. `note_meta.outgoing_links_json` and `note_links.target_name` stayed stale until the user touched the source again.

§142 plugs the Rust side: after `update_links_recursive` returns, the IPC opens the search-state connection and calls `reindex_single_note` for each rewritten path. Per-call transactions (`index_note` already wraps in `BEGIN IMMEDIATE`/COMMIT). Best-effort: per-file failures logged + skipped — the cascade rewrite is on disk, alias-aware reads from MIG-004 keep correctness intact, the IPC must not fail back over a reindex glitch.

IPC signature change: `update_links_on_rename(library_path, library_name, old_name, new_name)` — added `library_name` parameter (required by `index_note`). TS wrapper + +layout caller updated.

Commit: `d40e587`.

## §143 — Targeted in-place update of allLibraryLinks (the almost-fix)

§142 fixed Rust SQLite, but Boss tested and reported "Nothing changed" — Outgoing Links panel still showed stale name. Diagnosis: `allLibraryLinks` is a frontend `$state<NoteLink[]>` loaded ONCE at boot via `cache_boot_snapshot_graph` and never re-fetched. SQLite is correct; the in-memory snapshot the panel reads is stale.

§143 attempted a targeted in-place update: walk allLibraryLinks for entries where `source_path ∈ result.rewritten` AND `target.toLowerCase() === oldName.toLowerCase()`, rewrite `target` to `newName.toLowerCase()`. Same shape as §137's path-keyed migration applied to the link's target field.

But Boss tested again and reported "Nothing changed" on the §143 binary too. Root cause: after several renames in a single session (Hub v4 → v5 → v6 → v7 → v8), the in-memory state held `hub v4` from the boot snapshot, while subsequent renames passed `Hub v6` / `Hub v7` as `oldName`. The targeted match never fired — the in-memory state had drifted further than any single rename's `oldName`. So §143 walked allLibraryLinks correctly but skipped every entry.

Commit: `d119201`. Superseded by §144 (kept in history for archaeology).

## §144 — Re-fetch graph snapshot after cascade

Replaced §143's targeted update with the simpler drift-resistant fix: after `await updateLinksOnRename` returns, call `cache_boot_snapshot_graph` and replace `allLibraryLinks` + `notePathToAliases` wholesale. Catches not just the just-rewritten target but ANY drift accumulated in the session.

Cost: same as boot's graph fetch — `low-millis` per the cache.rs comments on the reference 7600-note Universe. Acceptable because rename is already a multi-step user-initiated action.

Boss tested PASS — Outgoing Links panel now updates immediately after rename. Closes the original Stage 1 observation.

Commit: `dcd5490`.

## Side discovery during §144 testing — pre-§140 corruption + Unlinked Mentions alias bleed

While testing §144 Boss observed two non-§144 issues:

1. **Tab title / content / cid_cn mismatch**: SourceA test note had `title: Hub v6` in frontmatter AND a duplicate `cid_cn` matching Hub v8. This is the §140 corruption class but the file was already corrupted from BEFORE §140's fix landed — §140 prevents NEW path-reuse contamination but can't retroactively heal already-corrupted files. Boss self-healed by delete + recreate. Logged for future sessions: existing libraries may carry pre-§140 cid_cn collisions; need manual recovery or a one-time scrub utility (queued).

2. **Unlinked Mentions panel matches frontmatter alias entries**: the scanner reads full file content (frontmatter + body) so YAML alias entries (`- "Hub v6"` from rename history) surface as "unlinked mentions". Should split on the closing `---` fence. Logged in project memory `project_unlinked_mentions_alias_bleed.md`. Pair with `project_unlinked_mentions_double_count.md` in a single Unlinked Mentions cleanup MIG.

## MIG-008 Phase 1 — Create-Dialog Standardization (Architect)

Boss directive 2026-05-03: "Whenever I created a folder it is created in the respective location under the name 'New Folder'. It shouldn't work this way. What I want it to do is to follow the standard way of any file system. A popup dialog box should emerge to name the new folder and to choose the location. Same thing should happen when creating new note, base or library."

Inventory (via Explore agent) found four inconsistent create flows:
- **Folder** — auto-named "New Folder", rejects on collision, no dialog
- **Note** — auto-named "Untitled", auto-increments on collision (Rust), no dialog
- **Base** — auto-named "Untitled Base", auto-increments via 100-iter frontend loop, has its own `NewBaseDialog`
- **Library** — auto-named "My Library", rejects on collision, has folder picker only

Architect plan defined 11 invariants (I1–I11): single dialog entry point, pre-filled + pre-selected default name, location shown vs picked, Esc/Enter parity with OS dialogs, validation, post-create UX preserved, RTL, a11y, kind-driven specialization, i18n in 15 locales.

Three options enumerated:
- **(A)** Single shared `<CreateItemDialog>` component, kind-driven, modal — recommended
- **(B)** Inline tree-row input (Finder-style) — doesn't fit Library/Base
- **(C)** Modal + template/properties picker — overkill

**Boss approved Option A.** Cascading through Phase 2 Build per Plan Approval = Build Approval.

Build plan: 8 steps (§Build.1–.8) — build component → wire 4 affordances → drop orphaned handlers → /simplify → audit. Each step pauses for Boss-testable verification clause.

Architect doc: `lab/reports/MIG-008-CREATE-DIALOG-ARCHITECT.md`. Commit: `22839d4`.

Out of scope for MIG-008: filename-collision popup (separate `project_rename_collision_popup_wanted.md`), template/properties picker, inline-tree gesture, default-frontmatter changes.

## MIG-008 §Build.1–§Build.6 (§145–§150)

Cascaded through the architect plan:
- §145 — CreateItemDialog component + i18n keys (en + ar)
- §146 — Wire New Folder (right-click + library-toolbar)
- §147 — Wire New Note (right-click + library-toolbar + command palette)
- §148 — Wire New Base (workspace + folder-context); kind-specific extras snippet for library multi-select
- §149 — Wire New Library + new Rust IPC `create_new_library_at(parent_path, name)` (async per §152)
- §150 — Orphan sweep: removed 5 state vars + 2 functions + welcome-screen inline form; deleted `NewBaseDialog.svelte`

## §151 — Boss-flagged context-menu gaps

After Boss tested the §150 binary, two findings:
1. Folder right-click was missing "New Base" — added.
2. Library / Universe row right-click fell through to browser-default menu — wired `oncontextmenu` on all three library-header sites with a slim menu (New Note / New Folder / New Base; Rename + Delete suppressed at library level).

Boss tested §151 binary: 4/4 PASS.

## §152 (Build.7) — /simplify checkpoint

Three review agents (reuse / quality / efficiency) walked §145–§151. Tier 1+2+3 fixes shipped + four Boss-approved adds:

**Closure-blockers**: i18n backfill 13 locales (de/es/fa/fr/he/hi/ja/ko/pt/ru/tr/ur/zh); `create_new_library_at` switched to `(async)`; IME composition guard in dialog keydown handler.

**Component cleanups**: `KIND_LABELS` lookup table (replaces 2 switch statements); dropped `defaultName` prop + `lastOpenState + $effect` (re-mount on each `{#if}` open is enough); `parseFrontmatter` replaces hand-rolled regex (which was missing `cid_cn`); `baseSelectedSet` $derived for O(1) lookup.

**Boss-approved adds**: right-click "New note" applies folder templates uniformly via shared `createNoteWithTemplate` helper; `/libraries` route migrated to CreateItemDialog (last surface using inline-input pattern); path-traversal hardening on `create_new_library` and `create_new_library_at` via `sanitize_name`.

**Style**: removed NewBaseDialog tombstone comment.

Production rebuild + Boss verified all four §152-specific scenarios PASS.

## §Build.8 — Phase 4 Audit

`lab/reports/MIG-008-AUDIT.md` documents the audit. All 11 invariants (I1–I11) verified PASS against shipped code + Boss tests. No unintended drift. Migration path: no action needed (pure UX change). Code reduction: net negative LOC (shared component absorbed enough variation to come out smaller than the four pre-MIG-008 inconsistent flows combined).

## SO #2 — Help files + User Manual

Updated per Boss directive:
- `docs/help.uConstellation.World/Notes Management/Notes Management.md` — Elements toolbar table updated; new "The Create dialog" section with full UX walkthrough.
- `docs/help.uConstellation.World/Libraries/Library management.md` — "Adding a library" rewritten to distinguish Create-new vs Link-existing.
- `docs/User Manual.md` — §2 (Managing Libraries) and §3 (Creating a Note) rewritten.
- `docs/help.ar/User Manual.md` — same updates in Arabic (Boss's daily language).

13 other locale User Manuals queued via `project_user_manual_13_locales_backfill.md`.

## MIG-008 closed

Project memory `project_create_dialog_standardize.md` marked SHIPPED. Orientation v1.30 → v1.31 with the closure callout.

## Pending after MIG-008

- **Standard OS-style create dialog for Folder / Note / Base / Library** (Boss directive 2026-05-03). Currently auto-creates with default names ("New Folder", "Untitled") and expects in-place rename. Should behave like Explorer / Finder: modal with name input + location picker + Cancel/Create. Applies to all four create surfaces. Logged in project memory as `project_create_dialog_standardize.md`. Likely composes with the planned filename-collision popup (`project_rename_collision_popup_wanted.md`).
- **MIG-006 §4–§11**: reindex via `index_note` (closes the stale `outgoing_links_json` gap Boss surfaced in Stage 1 — Outgoing Links panel still shows old target names after a cascade), sync/async dispatch + progress events (P6 — hub-rename UX), atomic per-file writes via tempfile (P5 — kill-mid-cascade integrity), pre-MIG-006 backfill command for stale wikilinks. **§4 is the natural next item if Boss wants to continue the rename-cascade arc.**
- CE Phase 9 Path B / MIG-010 scale.
- store.ts:1850 LinkLifecycle Option B fix (deferred until post-CE).
- Other 13 locales — backfill §120 inspector360 keys.
- NSIS bundling lock (re-run when the file lock releases, or skip if MSI is the canonical distribution).
- "Maturity for new notes" — `'seed'` is filtered out of the dot display by design; if Boss wants seeds to show a small dot (e.g. grey), that's a separate UX call.
