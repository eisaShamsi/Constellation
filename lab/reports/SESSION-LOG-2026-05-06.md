# Session log — 2026-05-06

## Top-line

Cascade through MIG-013 close-out → governance docs (Laws v1.3, Pending Jobs v1.1, NotePane Specs v1.0) → MIG-014 (Note-Stage Taxonomy) Phases 1A → 1C.5. First user-visible change of MIG-014 shipped; first Boss test surfaced a UX correction (datalist → custom dropdown) that landed as §1C.5. Awaiting retest before §1D.

## Commits landed today (chronological)

| Commit  | Phase / scope                                                 |
| ------- | ------------------------------------------------------------- |
| (prior) | MIG-013 §1A + §1B closed in earlier session                   |
| —       | Constellation Development Laws v1.0 → v1.3 authored           |
| —       | Constellation Pending Jobs v1.0 → v1.1 (stable ref numbers)   |
| —       | NotePane Specs v1.0 distilled from 121 commits                |
| `66001c4` | MIG-014 §1 Architect — Note-stage taxonomy                  |
| `1138aec` | MIG-014 §2 Plan — six-phase rollout                         |
| `c3b9454` | MIG-014 §1A — Rust schema + 5 Tauri commands                |
| `8a9ab3d` | MIG-014 §1B — frontend store + IPC wrappers                 |
| `17bf474` | MIG-014 §1C — PropertyEditor combobox v1 (native datalist)  |
| `57dceb2` | MoCh convention + first entry (`docs/MoCh/MoCh-2026-05-06-0900.md`) |
| `9973e65` | MIG-014 §1C.5 — replace native <datalist> with custom dropdown + NotePane Specs §3.5 doc-fix |

## Decisions made

1. CTSE Bridge Adapter switches index-time → query-time concept expansion (Option B). Locked into CLAUDE.md as Working Agreement #5 (Cross-check against proven methods).
2. Top-principal rules added: Predecessor Lookup, Stop-On-Correction, State the Function in Hand.
3. PJ-007 baseline = Living Link 6-stage + per-Universe `custom_stages`. Zettelkasten model rejected.
4. Pending Jobs use stable reference numbers (PJ-NNN) that never get reissued.
5. Law 2.6 hierarchy: Universe (itself a default Library via `is_universe_notes`) → Library → Folder → Note, with cUniverse as sibling federation.
6. **MoCh convention** — `docs/MoCh/MoCh-YYYY-MM-DD-HHMM.md` every ~3 hours. Stored as Standing Order #7 in CLAUDE.md.
7. **Eisa** as the conversational name (project-doc terminology unchanged).
8. **MIG-014 §1D scope correction** — breadcrumb stays as promote/demote arrows + badge, NO dropdown. The Plan I wrote was wrong; corrected mid-cascade. NotePane Specs §3.5 amended.
9. **§1C.5 mid-cascade addition** — native `<datalist>` ditched for a custom dropdown after first Boss test surfaced two-tier value/label rendering in WebView2.

## At-risk / open

- **MIG-014 §1C.5 retest pending** — MSI rebuild in flight. Boss to verify the new single-row dropdown works.
- §1D / §1E / §1F / §1G — pending in cascade.
- NSIS bundle race documented earlier; MSI is the primary installer used.
- PJ-014 — 13-locale `propertyEditor.stagePlaceholder` backfill queued.

## Known-broken (carried)

- `LinkLifecycle` dedupe in `store.ts:2298` — Option B approved 2026-05-01, deferred until post-CE.
- Pre-MIG-013 backups hit blocking v2 sentinel migration (project memory `mig013_v2_migration_blocking_boot`).

## Doc drift fixed today

- `NotePane Specs v1.0` §3.5 — corrected the dropdown claim (`6cbe87c` was experimental, undone at `90c1ea8`).

## Next decision point

Eisa's verdict on the §1C.5 retest. If clean, cascade to §1D (replace `stageOrder` + emoji map in NotePane.svelte breadcrumb). If not clean, narrow further before §1D.

---

## End-of-day update — MIG-014 CLOSED

The §1A → §1D flat-list model was proven wrong in Boss test (custom emoji noise, long promote chain, three-surface state drift). Eisa pivoted twice — first with the "concept paper for Stages" instruction, then with the simplifying corrections (one custom term per Universe → then per-note only; one combobox; dash separator). The model that shipped is per-note dash-encoded.

### Additional commits today

| Commit | Phase / scope |
|---|---|
| `c59bdfb` | Stages Concept Paper v1.0 (2D matrix model — superseded) |
| `c7fcdd3` | MIG-014 §2 Plan v2 (multi-type matrix — superseded) |
| `738b086` | Concept Paper v1.1 + Plan v3 (one term, one control — partially superseded) |
| `5782f15` | **Stages Concept Paper v1.2 + Plan v4 (per-note dash-encoded — APPROVED)** |
| `2f58b8a` | MIG-014 §2A — Rust schema cleanup (drops CustomStage / custom_stages / 5 IPC) |
| `59ed95c` | MIG-014 §2B — frontend cleanup + new pure helpers (splitStage, stageLabel, nextStage, prevStage, single-arg lookupStageEmoji) |
| `432076c` | MIG-014 §2C — PropertyEditor 6-entry mode-flip combobox |
| `2c58bda` | MIG-014 §2D — NotePane breadcrumb chain walks within suffix |
| `bb7a6ef` | §2C+§2D fix — Properties sync + typed-fixed Enter |
| `e3a97a1` | **§2D fix — Law 2.7 (stage-as-parent architectural fix)** |
| `a50463c` | MIG-014 §2E — help + User Manual rewrite (en + ar) |
| `339d65b` | **MIG-014 §2F closes — three-agent audit + P0/P1 fixes + orientation v1.44** |
| (this) | PJ-007 → SHIPPED in Pending Jobs v1.1 + session log close-out |

### Two new top-principal lessons today

1. **Law 2.7 — Single source of truth: properties have one parent.** Triggered by three patches in a row failing to keep the three stage-display surfaces in sync. The architectural fix dropped local `$state` mirrors entirely and made every UI surface a `$derived` subfunction. Generalised from stage to every first-class property.
2. **`feedback_orientation_inline_with_commit.md`** — orientation v-bump lands IN THE SAME COMMIT as any SO #6 trigger; no batching, no waiting. Eisa had to ask "Why do I have to remind you?" — saved the lesson to never recur.

### State at end of day

- **MIG-014 (Note-Stage Taxonomy / PJ-007) — CLOSED.** Per-note dash-encoded model shipped through §2A → §2F.
- **Iteration record kept**: §1A → §1D commits stay in `main` per Eisa's call (don't rewrite history).
- **Documentation aligned**: orientation v1.40 → v1.44; Constellation Development Laws v1.0 → v1.4; Stages Concept Paper v1.0 → v1.2; Plan v1 → v4; NotePane Specs §3.5 corrected; Pending Jobs v1.1 with PJ-007 marked SHIPPED.
- **Audit clean**: three-agent audit (invariants / drift / migration-path) passed after P0/P1 close-out fixes. Audit report at `lab/reports/MIG-014-NOTE-STAGE-AUDIT.md`.
- **Memory grew**: five top-principal feedback memories now in place (`feedback_dont_make_things_up`, `feedback_secure_dont_muddle`, `feedback_tutorial_tests_and_cascade`, `feedback_minutes_of_chating`, `feedback_orientation_inline_with_commit`). Plus `project_mig014_audit_p2_p3_followups` for the six P2/P3 polish items.

### Open after close

- 6 P2/P3 items from the §2F audit (logged in memory; allocate PJ-NNN at next Pending Jobs bump).
- PJ-014 — 13-locale i18n + User Manual backfill (carried).
- Pre-existing carried items: LinkLifecycle dedupe (deferred until post-CE), pre-MIG-013 v2 sentinel migration mini-MIG.

### Tomorrow's first move

Whatever Eisa picks. MIG-014 closes today; the queue is open.
