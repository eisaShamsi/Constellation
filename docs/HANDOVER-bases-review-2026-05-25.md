# HANDOVER — Constellation Bases review

**Created:** 2026-05-25 (end of MIG-046/047/048 revert session).
**Hand-off to:** next session.
**Boss directive (verbatim):** *"Next is to check where we stand regarding the Constellation Base. Prepare a handover to start the Base review in a new session."*

> **Read order for the next session:**
> 1. `docs/Constellation Orientation & Onboarding v2.34.md` — the canonical state (per CLAUDE.md SO #6).
> 2. **This file** — for the Bases-specific surface area + open questions.
> 3. Then begin the review per the scope outlined in §5 below.

---

## 1. Function in hand

**Working on: the Constellation Bases feature** — the structured table / card / list query view over markdown notes' YAML frontmatter, with `.base` files as the persistence format. Equivalent to Obsidian Bases / Dataview, but with both a visual builder (`.base` YAML) AND a DQL-style query language (`dataview.rs` reuses Bases' scan primitives).

This is a **review**, not a feature ship. The next session's job is to:
- Read the code with fresh eyes.
- Surface bugs, drift, perf cliffs, missing tests, doc/code mismatches.
- Recommend whether to leave Bases as-is, ship a polish migration, or queue a bigger redesign.

Eisa will scope the review further at the start of the new session.

## 2. Where we just came from (revert context)

Mind (MIG-046 + MIG-047 + MIG-048) was reverted today. `main` HEAD is `1b7ec9d1`. The local-LLM stack is gone; `ai/mod.rs` cloud bridge is the only LLM surface. **Bases were not touched by Mind at any point** — the Bases review is fully decoupled from anything Mind did.

If the new session needs the full revert context, read:
- `lab/reports/SESSION-LOG-2026-05-25.md` (today, both blocks).
- `docs/Constellation Orientation & Onboarding v2.34.md` (preamble has the revert headline).
- `docs/MoCh/MoCh-2026-05-25-{0700,1500}.md` (Boss conversation trail).

## 3. What Constellation Bases is (evidence-based)

Introduced **commit `c5b05f5c`** (2026-03-12): "Add Universe system, Bases, and major feature updates" — the MVP shipped with `.base` YAML, folder/tag/all source types, table view, filter + sort builders, cell editing in place, workspace-level Bases CRUD.

The grounding contract:
- **All local.** `.base` files live in `{universe}/.constellation/bases/` (workspace bases) or inside libraries (folder bases). Plain YAML on disk; the user owns them.
- **Read-write in place.** Editing a cell updates the note's YAML frontmatter directly via `update_frontmatter_property()`. No shadow DB.
- **No cache.** `query_base` does a live filesystem + frontmatter scan every call (this is the Rule 8 concern — see §4 below).

Spec doc: [`docs/BASES_MVP_SPEC.md`](BASES_MVP_SPEC.md).

### Data shape

```rust
// src-tauri/src/bases.rs:90–106
pub struct BaseDefinition {
    pub version: u32,
    pub name: String,
    pub source: BaseSource,       // folder | tag | all (+ optional path/tag; selected_vaults filter)
    pub columns: Vec<ColumnDef>,  // visible cols + widths; auto-detected if empty
    pub filters: Vec<FilterRule>, // is | is_not | contains | gt | lt | is_empty | is_not_empty
    pub sorts: Vec<SortRule>,     // property + asc | desc
    pub view: String,             // "table" | "card" | "list"
    pub direction: String,        // "auto" | "rtl" | "ltr"
}

// src-tauri/src/bases.rs:113–120
pub struct BaseRow {
    pub file_path: String,
    pub file_name: String,
    pub library_name: String,
    pub library_path: String,
    pub properties: HashMap<String, String>,
    pub modified: u64,
}
```

Frontend mirrors: `src/lib/bases/types.ts:1–55`.

### Tauri IPC surface (10 commands, all in `src-tauri/src/bases.rs`)

| Command | Line | What it does |
|---|---|---|
| `parse_base_file` | 360 | Load + parse `.base` YAML → `BaseDefinition` |
| `query_base` | 386 | Execute scan/filter/sort over a library; returns rows + auto-detected column names |
| `create_base` | 525 | Create a new `.base` in a library folder |
| `save_base_file` | 592 | Persist `BaseDefinition` changes (column reorder, filter/sort) |
| `update_note_property` | 603 | Cell edit → writes to note's frontmatter in place |
| `list_workspace_bases` | 717 | Enumerate all `.base` files in `{universe}/.constellation/bases/` |
| `create_workspace_base` | 755 | Create a workspace-level Base |
| `save_workspace_base` | 805 | Persist workspace Base definition |
| `delete_workspace_base` | 839 | Remove a `.base` file |
| `parse_workspace_base` | 861 | Load workspace `.base` |

> **Doc drift item:** Orientation v2.34 §4582 says "bases.rs — `.base` YAML CRUD. Live scans on `query_base`. **5 commands**." Actual count is **10**. The orientation needs a one-line correction. (Logged here so the review session can decide to fix it inline OR queue as a doc-drift commit.)

### UI mount points

| File | Lines | Role |
|---|---|---|
| `src/routes/+layout.svelte` | 4720–4732 | Left-sidebar collapsible "Bases" section listing workspace bases |
| `src/routes/+layout.svelte` | ~1051 | `handleNewBase()` wired from toolbar + context menu |
| `src/lib/components/BaseView.svelte` | 1–150 | Main view shell — source / filters / sorts editors + table/card/list switcher |
| `src/lib/components/BaseTableView.svelte` | full | Table rendering |
| `src/lib/components/BaseCardView.svelte` | full | Card rendering |
| `src/lib/components/BaseListView.svelte` | full | List rendering |
| `src/lib/components/BaseFilterBuilder.svelte` | full | Filter rule UI |
| `src/lib/components/BaseSortBuilder.svelte` | full | Sort rule UI |
| `src/lib/bases/types.ts` | 1–55 | TS mirror of Rust types |

## 4. Known concerns + drift (from the orientation v2.34 reading)

These are already documented in `Constellation Orientation & Onboarding v2.34.md` (and v2.33 below it). The review session does not need to re-discover them.

1. **Rule 8 (write-time derivation) violation** — `bases.rs::query_base` is read-time live scan. Orientation §4976 "CE Rule 8 audit-pending" lists Bases as one of three subsystems (Bases / dataview / lenses / Constellation Map) that still scan on every read instead of maintaining a derived view at write time. **This is the headline concern.** On a 7,600-note Universe (Eisa's scale) the query cost is the question.
2. **No column count drift caught in orientation** — Bases IPC surface has grown from 5 → 10 commands without orientation §4582 being updated.
3. **dataview.rs reuses bases scan primitives** (orientation §4581) — coupling the review needs to weigh: are the two languages (visual builder vs DQL) both worth maintaining, or should one supersede the other?
4. **BootBundle includes `workspace_bases`** (orientation §4571 + §4839) — the boot path eagerly loads workspace base definitions. Worth confirming this is cheap (just YAML parse, not the live scan).

## 5. Suggested review scope (the new session should refine with Eisa)

The "Base review" task is open-scoped right now. Five candidate framings:

- **Framing A — Performance audit.** Does `query_base` scale to a 10k-note Universe? Time `query_base` on Eisa's actual data; identify the slow path; propose either incremental indexing or a write-time derived `bases_cache` table.
- **Framing B — Rule 8 compliance migration.** Treat Bases as the next migration target. Architect → Plan → Build → Audit a `bases_cache` SQLite table updated by triggers on `note_meta` writes; switch `query_base` to a cheap SQL lookup. Same shape as MIG-013 / MIG-027 / MIG-040.
- **Framing C — Feature gaps audit.** What's missing vs Obsidian Bases? GroupBy? Multi-library cross-search? Aggregations (count/sum)? RTL-correct table headers on Arabic universes? Calendar view? Cards-by-tag? Make a punch list.
- **Framing D — UX review.** Open BaseView on a real Universe, walk through every interaction, surface friction points. Specifically the 5 open questions in §6 below.
- **Framing E — Code-quality + test coverage.** `bases.rs` is ~880 lines with how many unit tests? Frontend components have how much svelte-check coverage? Tighten before any feature work.

Reasonable default: **start with B (Rule 8 migration architect doc)** — it's the highest-leverage move and matches the `/migration` discipline already in flight for other subsystems. But Eisa picks.

## 6. Open questions a reviewer would want answered

From the Explore-agent territory map (verbatim — not invented):

1. **How does the "all" source type handle millions of notes?** Is there pagination, or does `query_base` materialize every row into memory? (Likely in-memory `Vec`; worth checking if Eisa's universe scales beyond 10k notes.)
2. **Cell edit conflicts:** when two tabs edit the same note property, which write wins? No locking visible in `update_note_property()` — known race?
3. **Property type coercion:** `FilterRule` operators like `gt` parse values as `f64`, but `properties` are all `String`. What happens if a cell contains "apple"? Silent no-match?
4. **Workspace bases inheritance:** can a child universe (cUniverse) inherit parent workspace bases, or are they strictly per-universe? The Universe spec says "child universes with recursive vault resolution" but bases seem isolated.
5. **RTL correctness:** has the table/card/list view been tested in an RTL universe with bidirectional column headers (Arabic property names + English values, and vice versa)?

## 7. Pointers for the new session

| Need | Path |
|---|---|
| Canonical state of the world | `docs/Constellation Orientation & Onboarding v2.34.md` |
| Bases spec | `docs/BASES_MVP_SPEC.md` |
| Rust impl | `src-tauri/src/bases.rs` |
| Frontend impl | `src/lib/components/BaseView.svelte` + `Base{Table,Card,List}View.svelte` + `Base{Filter,Sort}Builder.svelte` |
| TS types | `src/lib/bases/types.ts` |
| Boot integration | `src-tauri/src/boot_bundle.rs` (`workspace_bases` field) |
| Adjacent system | `src-tauri/src/dataview.rs` (DQL — reuses Bases scan primitives) |
| Today's session | `lab/reports/SESSION-LOG-2026-05-25.md` |
| Today's conversation trail | `docs/MoCh/MoCh-2026-05-25-{0700,1500}.md` |
| Rule 8 (write-time) policy | `CLAUDE.md` §Performance Rules → Rule 8 |
| /migration workflow | `.claude/skills/migration.md` |

## 8. First actions for the new session

1. **State the function in hand** (per CLAUDE.md top principal): "Working on: the Constellation Bases review."
2. **Read** `docs/Constellation Orientation & Onboarding v2.34.md` end-to-end (~5 min). Then this handover.
3. **Ask Eisa which framing** (§5 A–E) — or his own framing. Don't start architect work without the scope locked.
4. **If a migration framing is chosen**, follow the standard /migration workflow: Architect → Plan → Build → Audit → PCS.
5. **If a review-only framing**, deliver a one-doc finding report with file:line evidence + recommendation, no code changes.

---

*End of handover. Total ~600 words of grounded evidence. The new session can start the Bases review without re-discovering the territory.*
