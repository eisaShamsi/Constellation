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
