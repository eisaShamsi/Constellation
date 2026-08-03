# Safety Inspection — 2026-08-03 (PJ-207 §1 per-build gate)

Invoked **diff-scoped** with `args.files = ["src-tauri/src/search.rs"]`; returned `mode: "whole-app"`
and scanned all 14 scopes. **PJ-166, tenth strike** — the per-build gate the standing order
requires still does not exist.

72 agents, ~9.1 M tokens, ~30 min. 58 candidates verified, **40 CONFIRMED / 15 refuted**.

## The result that gated the commit

**ZERO confirmed findings in `src-tauri/src/search.rs`** — the only file §1 touches.
The §1 diff is clean; every finding below is pre-existing.

## Confirmed sites, by file

- `+layout.svelte` — :1703, :1819, :6807, :9500
- `ConfidencePicker.svelte` — :61
- `NoteEditor.svelte` — :284
- `NotePane.svelte` — :362, :768, :783
- `PropertyEditor.svelte` — :974, :976, :1034
- `SecondScreenPage.svelte` — :138
- `bases.rs` — :796
- `canvas.rs` — :76
- `libraries.rs` — :398, :1295, :1298, :1299, :1418, :1893, :1903, :2038
- `linkTypeRegistry.ts` — :154
- `link_life_restore.rs` — :398
- `links_backfill.rs` — :447
- `mod.rs` — :442
- `query.rs` — :389
- `review_backfill.rs` — :182
- `sky_backfill.rs` — :456
- `store.ts` — :343, :346, :347, :1294, :1693, :2950, :3122, :4388, :7186
- `write_gate.rs` — :717

## Overlap with the 2026-08-02 triage register (already Boss-approved as "All 31")

| Inspection site | Triage item |
|---|---|
| `store.ts:7186` | **#11** — Universe switch leaves the previous universe's layouts loaded. **The inspection escalates this to APP-KILLER** and names a SECOND branch the triage missed: `loadWorkspaces` also refuses to adopt a *successful empty* read (`if (data.length > 0)`), contradicting its own comment. Collections, settings and property-types all got this reset; workspaces is the sibling that never did. |
| `libraries.rs:1418` | **#15** — folder rename freezes the window (both siblings were detached; this branch was not) |
| `PropertyEditor.svelte:974 / :1034` | **#8** — the 0.8 s property window writing onto a different note |
| `link_life_restore.rs:398` | **#23** — link bookkeeping disagrees after a restore |
| `write_gate.rs:717` | **#24** — rename writes the new title into the OLD file, then reports failure |
| `libraries.rs:398` | **#31** — a library register failure printed only to a console release builds discard |

## Not in the register — new

`libraries.rs:1295/1298/1299` (rename destination pre-delete destroying earned per-note data;
the justifying comment is factually false — `note_links` and `note_aliases` DO carry UNIQUE
constraints), `store.ts:2950/3122` (openNoteTab has no in-flight guard across an awaited disk
write), `store.ts:343/346/347` (the write-ahead net's unbounded synchronous localStorage blob,
quota exception swallowed — this is PJ-188), `NotePane.svelte:768/783`, `NoteEditor.svelte:284`,
`+layout.svelte:6807` (three closed-note writers emit nothing), `sky_backfill.rs:456`,
`review_backfill.rs:182`, `links_backfill.rs:447`, `bases.rs:796`, `canvas.rs:76`,
`query.rs:389`, `mod.rs:442`, `ConfidencePicker.svelte:61`, `linkTypeRegistry.ts:154`,
`SecondScreenPage.svelte:138`, `+layout.svelte:1703/1819/9500`, `store.ts:1294/1693/4388`.

**Triage owed** at the PJ-207 close, merged against the 31-item register and the 35-item
2026-08-03 prebuild feed. Full agent transcripts: `wf_bdb74b70-066`.
