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

## Pending after §139

- **MIG-006 §4–§11**: reindex via `index_note` (closes the stale `outgoing_links_json` gap Boss surfaced in Stage 1 — Outgoing Links panel still shows old target names after a cascade), sync/async dispatch + progress events (P6 — hub-rename UX), atomic per-file writes via tempfile (P5 — kill-mid-cascade integrity), pre-MIG-006 backfill command for stale wikilinks. **§4 is the natural next item.**
- CE Phase 9 Path B / MIG-010 scale.
- store.ts:1850 LinkLifecycle Option B fix (deferred until post-CE).
- Other 13 locales — backfill §120 inspector360 keys.
- NSIS bundling lock (re-run when the file lock releases, or skip if MSI is the canonical distribution).
- "Maturity for new notes" — `'seed'` is filtered out of the dot display by design; if Boss wants seeds to show a small dot (e.g. grey), that's a separate UX call.
