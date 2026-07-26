# Constellation Pending Jobs

**Version 1.50 | 2026-07-25**

> **What changed in v1.50** (**The PJ-140 [0] HIGH content-loss FIXED + Boss-validated (Backlinks "link it" — single content ownership + reindex + false-success surfaced), the 5-10s refresh lag fixed, and the Whole-Ecosystem no-reindex siblings closed. Ultracode**):
>
> ### ★ THE RE-PRIORITIZED BACKLOG
>
> **► NEXT ACTION — (1) MIG-105 Architect** (root library vs flat Universe — the data-model root cause behind the resolver class; Boss-directed, ready to run). **(2) MIG-104 remaining open questions** (durable earned-link home; location = `.constellation`, re-type-keeps-history = YES settled; review-state + coalesced-count questions left). **(3) The remaining PJ-140 backlog** (~37; the editor-lifecycle cluster is its own migration; the rest await a sequencing ruling). Then §1 use-side remainder · D4.
>
> **CLOSED THIS JOB (Boss-validated end-to-end — Stage 1 · timing re-test · open-note Stage 2 all PASS):**
> - **PJ-140 [0] — the HIGH content-loss** (`BacklinksPanel` "link it"). The old `linkMention` turned a plain-text mention into a `[[wikilink]]` via a raw `invoke('write_note')` with three silent failure modes: **open-model overwrite** (wrote behind an open note, whose next autosave erased the link + unsaved edits), **false success** (`catch {}` swallowed a failed write), **index divergence** (no reindex → backlink invisible until boot). Fixed with a shared store primitive `linkMentionInNote` on the proven `toggleTaskReconciled` body-edit shape — gate → **flush-the-open-model-or-abort** (never clobber) → mutate disk → model adopts → reindex; longest-root-wins resolution (nested-safe); body-scoped (no YAML corruption); **throws** on a real write failure, surfaced via the save-health banner. Reproduce-First `tests/pj-140/backlinksLinkMention.test.ts` (7 cases; T6 = open-dirty-locked ABORTS with no clobber). **Solve-the-Class: single content ownership.**
> - **The two index-divergence siblings** (`ExpressionForge` composition export · `SenseMakingCanvas` promote-to-note) — createNote-then-write-body-with-no-reindex, so the note kept its empty create-time content in the index. Each got a `reindexNote` (the Whole-Ecosystem no-reindex sweep, alongside `template_create` + `daily_template` in `+layout`). Were `wf_ae5d4d18` findings [1]/[2].
> - **The 5-10s "link it" refresh lag** (Boss-flagged). Investigated, not guessed: NOT the write, NOT the reindex (O(changed-edges), no re-embed) — **nothing bumped `perNoteRefreshNonce`**, so the backlink appeared only when an incidental trigger re-ran the panel effect. Fix: await the fast reindex → `onLinked` callback → `applyMentionLinkedLocally` bumps the nonce → both the Backlinks and Unlinked-mentions effects re-fetch at once. **Boss: "almost instant."**
>
> **NEWLY FILED:**
> - **PJ-147** *(hygiene · reuse)* — **consolidate the four inline longest-root-wins library resolvers** into one shared helper: `store.ts` `linkMentionInNote` (new, inline), `store.ts:4768` (PJ-088 sidecar-trash — and it LACKS the `+ '/'` boundary guard, a latent inconsistency), `store.ts` `deriveLibraryForPath`, and `+layout.svelte` `libraryForPath`. Extract `libraryStatsForPath` and route them through it (the 4768 boundary-guard change needs its own validation). Surfaced by the `/simplify` reuse pass.
> - **PJ-148** *(efficiency · altitude)* — **fold the body into `createNote(initialBody)`** at `ExpressionForge:143` and `SenseMakingCanvas:270`, which `createNote` an empty stub then `writeNote` the body then reindex (double index-pass + a stub window). The atomic single-call idiom is already proven at `+layout.svelte:4947`. Drops 2 IPC + the double index per export. Surfaced by the `/simplify` altitude/efficiency passes.
> - **Watch-item (not filed):** the flush-gate envelope (`markCascading` → `flushOpenTabOrAbort`-or-abort → mutate → `reloadTabsFromDisk` → `clearCascading`) is now hand-rolled in BOTH `toggleTaskReconciled` and `linkMentionInNote`. Extract `withOpenNoteFlushGate` on the **3rd** occurrence (LL-014 rule-of-three), not before.
>
> **STILL OPEN:** **PJ-145 / MIG-105** (root library vs flat Universe — Architect next) · **MIG-104** (durable earned-link home — 3 questions) · **PJ-140** (~37; editor-lifecycle cluster = own migration; rest await ruling) · **PJ-142** (bulk-accept end-to-end — Tauri mock harness) · **PJ-143** (`target_path` empty) · **PJ-144** (per-note scan reload) · **PJ-147** · **PJ-148** · **PJ-137** (one YAML authority) · **PJ-135** · **PJ-124** (inspection ignores `args.files` — struck again) · **PJ-132** (Sight flake) · PJ-125/126/127/128/129/131/133/138/139.
>
> ---

**Version 1.49 | 2026-07-25**

> **What changed in v1.49** (**A 14+5 PJ-140 Rust remediation shipped & Boss-validated · the Whole-Ecosystem file-tree fix (PJ-141 closed) + self-heal backfill · the library-icon system · TWO new top-principal LAWS. Per-build inspection: no regression. Ultracode**):
>
> ### ★ THE RE-PRIORITIZED BACKLOG
>
> **► NEXT ACTION — (1) FIX the PJ-140 HIGH content-loss** (`BacklinksPanel.svelte:181` `linkMention` — a "link it" click writes a `[[wikilink]]` via raw `invoke('write_note')` bypassing the open model, error swallowed, no reindex: open-model overwrite + false-success + index divergence, all silent). It is the single HIGH left in the register and is fixable now — own build + Boss test. **(2) MIG-105 Architect** (root library vs flat Universe — the data-model root cause behind the resolver class this job patched at the surface; Boss-directed, ready). **(3) MIG-104 remaining open questions** (location = `.constellation`, re-type-keeps-history = YES; questions left on review-state + coalesced counts). Then §1 use-side remainder · D4.
>
> **CLOSED THIS JOB (Boss-validated on the running binary, 18:32):**
> - **The Whole-Ecosystem file-tree fix** (`libraries.rs` / `search.rs` / `tasks.rs` + 33 frontend sites) — one concern ("enumerate the file tree, honoring Library ≠ Folder"), **every** surface. Two shared helpers `nested_library_paths` / `is_nested_library`; 13 walkers now honor the exclude set; per-file library attribution (longest-root-wins) replaces fixed-name stamping so a nested library no longer reports **0 notes**. Frontend `libraryForPath` at 33 sites → **PJ-141 CLOSED**. Move-picker filter no longer hides a library whose note lives at its own root. Born of the nested-library bug + **THE WHOLE-ECOSYSTEM FIX LAW**.
> - **Self-heal `library_attribution_backfill.rs`** (new) — versioned/batched/off-main-thread column re-write correcting mis-attributed `note_meta.library_name`; stamps only after a completeness check. Wired in `ensure_search_db_ready`.
> - **PJ-140 Rust remediation — 14 numbered + 5 unnumbered findings** (per-build inspection `wf_ae5d4d18`: ZERO new). Path-integrity via `migrate_note_db_paths`: #2 folder-rename cascade · #16 move-migrates · #3 folder-delete purges · #17 delete purges aliases+embedding · #18 gate_rename dest-guard. Freeze/leak: #27/#28 name_lower seek · #42/#60 async · #57 WAL-daemon newest-wins · #61 perf_trace bounded · #43 copy skips junctions. Silent-failure: #53 propagate Err · #33 nested-universe consolidation gated. Durability (atomic_write/corrupt-aside): libraries.json · review-pulse.json (was a silent default() losing all history) · collections.json. +9 tests.
> - **The library-icon system** (planet → library building, Boss "D") + Style Setter Icon-size control + toolbar `:global` fix + Move colour + glyph enlargement. Boss-validated.
> - **TWO new top-principal LAWS** — The Whole-Ecosystem Fix Law · No Guessing — Investigate to Build Awareness.
>
> **NEWLY FILED / GROWN:** PJ-140 ~40 (14+5 closed); gains the `wf_ae5d4d18` entries incl. **[0] HIGH `BacklinksPanel.svelte:181`**. **PJ-146** *(docs)* — translated help dirs are a partial subset.
>
> **STILL OPEN:** PJ-140 (~40) · PJ-145/MIG-105 · MIG-104 · PJ-142 · PJ-143 · PJ-144 · PJ-137 · PJ-135 · PJ-124 · PJ-132 · PJ-125–139.
>
> ---

*(Versions 1.48 and earlier: see `Constellation Pending Jobs v1.49.md` and its predecessors — the trail is durable, never overwritten.)*
