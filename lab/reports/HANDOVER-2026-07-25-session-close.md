# Handover — 2026-07-25 session close

**Read FIRST, in this order:** `docs/Constellation Orientation & Onboarding v3.68.md` (the
canonical fluency doc) → `docs/Constellation Pending Jobs v1.50.md` (the living backlog +
► Next action) → this file. Then `CLAUDE.md` (two NEW laws added today — see below).

**Branch:** `main` @ **`384e8fca`** (pushed to `origin/main`). Working tree **clean**.
**Gates green:** Rust **1161/0** (unchanged — no Rust edits after the file-tree batch) ·
svelte-check **0 errors** · vitest **623/623**.
**Last release binary built:** `src-tauri/target/release/constellation.exe` @ **20:26** today
(has the backlinks HIGH fix minus one behavior-identical `/simplify` cleanup that landed after —
so the binary is behaviorally current; rebuild only if you need the source-exact build).

---

## What shipped today (07-25) — four commits

1. **`bc8091e3`** — search.db made un-deletable (schema gate renames aside, never deletes;
   absent ≠ stale) · archive-reversal APP-KILLER fixed · the hour-long Cataloger scan de-frozen ·
   sidebar library duplication fixed · CLAUDE.md link-durability corrected · MIG-104/105 logged.
   *(Logged in `SESSION-LOG-2026-07-24-B.md`; ledger v1.48; orientation v3.66.)*
2. **`332f566d`** — the **Whole-Ecosystem file-tree fix** (Library ≠ Folder at 13 Rust walkers +
   33 frontend resolver sites → PJ-141 closed) + self-heal `library_attribution_backfill.rs` +
   a **14+5 PJ-140 Rust remediation** + the **library-icon system** (planet → building) +
   **TWO new top-principal LAWS**. *(Session log 07-25; ledger v1.49; orientation v3.67.)*
3. **`375771bc`** — doc stamp (commit hash into the 07-25 session log).
4. **`384e8fca`** — **PJ-140 [0] HIGH fixed**: Backlinks "link it" content-loss (single content
   ownership via `linkMentionInNote`) + reindex + the 5-10s→instant refresh + the Whole-Ecosystem
   no-reindex siblings. *(Session log Phase D; ledger v1.50; orientation v3.68; MoCh.)*

**Two new LAWS in CLAUDE.md + memory (Boss-dictated 2026-07-25):**
- **The Whole-Ecosystem Fix Law** — fix the whole concern across EVERY surface, one pass, shared
  helper so they can't drift. (`feedback_whole_ecosystem_fix_law.md`)
- **No Guessing — Investigate to Build Awareness** — read/run/query for facts; never theorize to
  build awareness. (`feedback_no_guessing_investigate.md`)

---

## State of standing (SO#5)

### (a) Verified-shipped & protected
- The whole-ecosystem file-tree consistency (nested libraries show once, appear in Move, report
  correct note counts) — Boss-validated; self-heal backfill runs on boot.
- The PJ-140 Rust remediation (14 numbered + 5 durability findings) — per-build inspection clean.
- The library-icon system (building glyph, resizable, all surfaces) — Boss-validated.
- The Backlinks "link it" HIGH — content-integrity-safe, reindexed, instant — Boss-validated
  end-to-end (Stage 1 · timing · open-note Stage 2). Reproduce-First test = 7 cases green.

### (b) At-risk / in-flight / uncommitted
- **None uncommitted** — working tree is clean.

### (c) Known-broken / open defects
- **PJ-140 backlog (~37)** — no HIGH remains (the [0] HIGH closed today). The register from
  `wf_45def36d` (today's per-build sweep, whole-app via PJ-124): 6 MED + 2 LOW pre-existing,
  incl. `NoteEditor.svelte:249` handleSave re-entrancy autosave-drop (editor-lifecycle cluster —
  **its own migration**), `search.rs:8513` archive-doesn't-recompute-target-incoming,
  `universe.rs:117` atomic_write fixed-temp-name race, `search.rs:9648` reindex Ok-on-None,
  `libraries.rs:5600` collect_library_notes sync-freeze, `store.ts:1923/1530/5937`. These await
  the Boss's sequencing ruling; none is an APP-KILLER.

### (d) Pending, not started
- **MIG-105 Architect** (root library vs flat Universe) — Boss-directed, the ► Next action. The
  data-model root cause behind the whole resolver/file-tree class. `docs/migrations/MIG-105-*.md`
  logged (concept + evidence + 3 options); the Architect workflow has NOT run.
- **MIG-104** (durable earned-link home / Earned-Life Ledger) — Boss settled location
  (`.constellation`) + re-type-keeps-history (YES); 3 open questions remain.
- **PJ-147** (consolidate the 4 inline longest-root library resolvers — one lacks the boundary
  guard) · **PJ-148** (`createNote(initialBody)` at ExpressionForge/SenseMakingCanvas to drop the
  stub-then-reindex double-pass) — both filed today by `/simplify`.
- PJ-142 (bulk-accept end-to-end — needs a Tauri mock harness) · PJ-143 (`note_links.target_path`
  empty on all rows) · PJ-144 (per-note scan reload) · PJ-137 (one YAML authority) · PJ-135 ·
  PJ-132 (Sight flake) · MIG-103 remaining phases (§1 use-side remainder · D4 · §2 · §3 · §5–7).

### (e) Doc drift
- **PJ-146** — the translated help dirs (`help.ar`, `help.de`, … ×14) are a PARTIAL subset of
  `help.uConstellation.World`, not a 1:1 mirror (most lack a "Libraries" topic). Today's
  user-facing notes (library icon, icon-size control) landed in the English canonical help +
  User Manual only. A full 14-language help sync is its own job.
- **PJ-124** — the `safety-inspection` workflow ignores `args.files` and runs whole-app every
  time (struck again today). Diff-scoped inspection is not actually diff-scoped until fixed.

---

## Watch-items / gotchas for the next session
- **Build verification:** the real SvelteKit app chunks live in `build/_app/immutable/chunks/`,
  NOT `build/assets/` (that holds icon/worker assets). Grep the former to confirm a frontend
  change is bundled. Note: production build strips dev write-journal `origin` label strings and
  `console.*` args — grep for a **surviving** string (a throw message quasi, an i18n key), or
  check the store chunk's content-hash filename changed.
- **`cargo test --release | tail` truncates** the unit-test result line (only the doc-test
  summary survives the tail). Redirect full output to a file and grep `test result:`.
- **The flush-gate envelope** (`markCascading`→`flushOpenTabOrAbort`-or-abort→mutate→
  `reloadTabsFromDisk`→`clearCascading`) is now duplicated in `toggleTaskReconciled` +
  `linkMentionInNote`. Extract `withOpenNoteFlushGate` on the **3rd** occurrence (LL-014), not before.
- **Mandatory Boss test before commit** stands (memory `feedback_boss_test_every_build_mandatory`).

---

## ► READY-TO-PASTE NEXT-SESSION PROMPT

```
Continue Constellation on branch main (@384e8fca, clean). First: git pull; read
docs/Constellation Orientation & Onboarding v3.68.md, then docs/Constellation Pending Jobs
v1.50.md (► Next action), then lab/reports/HANDOVER-2026-07-25-session-close.md. Note the two
new laws in CLAUDE.md (Whole-Ecosystem Fix + No-Guessing).

Then start the ► Next action: the MIG-105 Architect phase — root library vs flat Universe (the
data-model root cause behind the resolver/file-tree class we've been patching at the surface:
the root library claiming the Universe path at index 0). docs/migrations/MIG-105-*.md has the
concept + evidence + 3 options. Run the /migration Architect: map the territory, enumerate the
options with speed/effort/risk, list the invariants that must not break (esp. the universe_notes
root-as-library contract, the longest-root-wins resolvers, and the Move/indexer/watcher walkers
that PJ-141 + the Whole-Ecosystem fix just brought into line). Present the Architect doc for my
approval before any Plan or code. Cross-check MIG-105 against orientation §body + the 07-25
session logs first (SO#8) — confirm it isn't already partly shipped.
```
