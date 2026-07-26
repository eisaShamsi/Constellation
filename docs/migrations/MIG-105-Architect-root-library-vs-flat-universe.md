# MIG-105 — Architect (v2): The Core Organizer — the Universe's Head

**Phase 1 of 4 (`/migration`). Status: DESIGN LOCKED by Boss rulings 2026-07-26 — awaiting final
Boss approval of THIS document, then the Stage-0 prerequisites, then Phase 2 (Plan).**

Produced across two adversarial cycles on 2026-07-26:
- Cycle 1 (`wf_25a129dc-853`, 29 agents): 11 territory surveys → 4 option architects → 5 judges →
  8 adversarial attacks → completeness critic. Outcome: options A–D scored; D won 38/50.
- **Boss concept intervention (2026-07-26):** the *Brain / Core Organizer* concept re-founded the
  question — the entity is the Universe's HEAD, it remains, its scope is amended.
- Cycle 2 (`wf_8ade7478-139` + re-run `wf_e46ff8f5-60c`, 40 agents): the concept passed to the
  **Art Director & Team** (4 prior-art researchers → locked spec → 4 competing designs → 4 design
  judges → lead merge) and to the **Inspectors** (saved `safety-inspection` over the blast zone +
  3 concept-level inspections → Chief verdict). Companion documents:
  - `MIG-105-ArtDirector-verdict-Core-Organizer.md` (v2 — full-evidence re-run)
  - `MIG-105-Inspectors-verdict-Core-Organizer.md` (v2 — includes the federation inspection)

Every load-bearing number was re-verified by hand against the live universe and the live 2.0 GB
`search.db` before being written down here.

---

## Function in hand

> Working on: **the Universe's Core Organizer** — the entity today mis-implemented as the root
> library (`is_universe_notes`, index 0 of `.constellation/libraries.json`, `path` == the Universe
> root, written by `ensure_universe_notes_folder`, [universe.rs:252](../../src-tauri/src/universe.rs)).

## The concept (the horse) — Boss, verbatim, 2026-07-26

> "A universe in Constellation echo system is an organized structure, an entity of one's mind; it
> could act as an extension of one's reasoning. It is where cognition of knowledge occurs. Each
> universe has to have its own 'Brain' or 'Core Organizer', through which the universe will be
> structured, organized, controlled, and ruled. It is the head of the universe. It should include
> all vital and core files/contents that give the universe its identity. Without it, it will be
> headless and brainless, and it will be almost impossible to control or organize.
> Therefore, it should remain, it just need its scope amended."

The review's one-line justification, which beats every resolver argument below:
**an accident has no boundary; a head does.**

---

## §0. SO#8 cross-check — nothing here is already shipped

Verified against orientation v3.68 §2/§3/§6.6 (bodies) and the 07-24/07-25 session logs. Everything
shipped 24–25 July was symptom-level and says so in its own words (`read_library_tree` exclusion:
"Touches NO data model — that is MIG-105"; the Whole-Ecosystem sweep: "the data-model root cause …
remains MIG-105"). The data model is untouched.

---

## §1. The root cause, stated precisely

`ensure_universe_notes_folder` runs on **every** universe activation and, when no entry carries the
flag, inserts at index 0 ([universe.rs:404-409](../../src-tauri/src/universe.rs)):

```rust
libs.insert(0, LibraryInfo {
    id: format!("universe_notes_{}", uuid_simple()),
    name: meta.name.clone(),          // ← name == the Universe's own name
    path: root_path_str,              // ← path == the Universe ROOT directory
    is_universe_notes: true,
    canonical_mode: "native".to_string(),
});
```

Three independent defects are born on those three lines:

1. **`path` == the Universe root** ⇒ the entry physically contains every other library under the
   root; every naive recursive walk swallows them (Library-as-Folder, Move-picker freeze,
   double-index, 0-note counts).
2. **`insert(0, …)`** ⇒ every first-match `starts_with` resolver returns it for every path in the
   Universe (the four 07-24 resolver bugs + the frontend rename-cascade twin).
3. **`name` == the Universe name** ⇒ `note_meta.library_name` (a NAME — there is no `library_id`
   column in any of the 36 tables) is coupled to the Universe's display name.

Three activation-time healers re-assert this shape every boot (universe.rs:835-838, :1097-1103,
:803-807), so it cannot be hand-edited away.

**Two invisible powers the entry carries today, both by accident of its path, neither by design**
(Inspectors v2): it is the **trash anchor** for the active delete path (`store.ts:3846/:3853` —
deleting the entry breaks every delete on the Boss's live settings), and it is the **load-bearing
federation anchor** (`attach.rs:71-118` discovers cUniverses by walking library-path ancestry;
كون عيسى federates through its Brain **alone** — its registry holds exactly one entry). Options that
removed the entry (B, C) would have silently broken deletes AND severed federation. **The Boss's
"it should remain" is proven right twice over.**

---

## §2. The territory — verified live data

**Live universe:** `E:\Constellation Universes\Eisa Cognitive Knowledge` (search.db 2.0 GB, active).

| | |
|---|---|
| Registered libraries | **20** (1 root-flagged + 18 external + 1 nested-at-root) |
| Libraries physically outside the Universe root | 18 of 20 |
| Notes indexed | **7,816** — 0 blank `library_name`, 0 phantom rows (verified) |
| Owned by the root entry | **126** |
| …loose at the root | 44 indexed (**45 on disk** — `Testing opened note.md` has no index row, P6) |
| …`.trash` | 55 · `Templates` 12 · `Daily Notes` 4 · `MIG-005 Test` 4 · `Replication Bug` 4 · `Imported` 2 · `Five Acts` 1 |
| Root-entry link edges | 147, of which **139 un-earned** (would silently reset `created` on any reindex) |
| Federation | **Eisa Universe** is the parent (`children` = [كون عيسى, Eisa Cognitive Knowledge]); ECK's own `children` = `[]`; **three** flagged entries coexist in the live federated set |
| Nested-at-root libraries, live | 3 universes (`Creating new library` 0 notes; `EisaLibraryTest`; `التصوير`) |

Also at the root: `EisaTest.canvas` (no index row — the mover must read the **disk**, not the index).

---

## §3. THE LOCKED DESIGN *(Boss-ruled 2026-07-26; mechanisms from AD v2, safety gates from Inspectors v2)*

### 3.1 What it IS

**The Core Organizer** — the Universe's head. It **remains** in `.constellation/libraries.json`
(same `id`, so per-library appearance survives) — **stored like a library, never presented as one**
(Boss-accepted). Its scope is amended in four dimensions:

| | Today | Amended |
|---|---|---|
| **path** | the Universe root (contains everything) | `<root>\_Core` — a **leaf** containing only itself |
| **name** | the Universe's own name, re-pinned every rename | **per-universe-unique stored name** (see 3.6); display is always the localized label |
| **kind** | `is_universe_notes: true`, a flagged library | `kind: "core"` — a different class, absent from every library-facing surface |
| **contents** | 126 notes of debris + `.trash` + other libraries | the Universe's identity — a **closed** kind-list (3.3) |

Why the entry (not a token, not a dock button, not deletion): "remain… amended" is literally true
only in this reading; the federation anchor survives **by construction** (`_Core`'s ancestor chain
passes through the root); the trash anchor survives; and the alternative costs ~9 new
owner-resolution seams where keeping the entry costs ~3.

**The structural rule that kills the bug class** (from option D, retained): **no registered library
path may be an ancestor of another registered library path.** With `_Core` a leaf and containment
refused in `add_library`, `nested_library_paths` returns ∅ forever, first-match ≡ longest-root-wins,
and every walker — including the ~19 that never received an exclude set — is correct **by
construction**. The Library-as-Folder family becomes unreachable, not filtered.

### 3.2 Name and label — **Boss-ruled**

- **Entity label: "Core Organizer"** — the Boss's own phrase, verbatim; translates as a *role* in
  all 15 locales (المُنظِّم الأساسي · سازمان‌دهندهٔ اصلی · מארגן הליבה · بنیادی منتظم · 核心组织者 …).
  i18n namespace `coreOrganizer`; "Brain" appears in the description, never the label.
- **"Root Library" / "System Library" — REJECTED** (Boss accepted the reasoning): "Root Library" is
  the name of the disease being cured; "System Library" mislabels the head as a member of the
  category it rules and frames the user's own charter/MOC as machine territory.
- **"Core MOC" — the MOC goes INSIDE, not on the label** (Boss accepted): a MOC maps; the head also
  *rules* (the concept's four verbs). The name carries the role; the front page carries the map.

### 3.3 Contents — a CLOSED kind-list — **Boss-ruled**

```
<Universe root>\
├── _Core\                    ← THE HEAD. kind:"core". Leaf. The only visible directory at the root.
│   ├── <Universe MOC>.md     ← ONE front-page note: charter + Map of Content in one document
│   │                           ("what this Universe is, and where everything lives").
│   │                           OFFERED, never auto-created; pointer in universe.json → core.moc;
│   │                           never-overwrite / delete-to-restore contract; seeded in app language.
│   ├── Templates\            ← moves in (the real molds; the 6 TS_test_* debris go to §4)
│   └── Five Acts\            ← moves in (1 host note); FIVE_ACTS_DIR repoints creator+lister together
├── .constellation\           ← UNCHANGED, hidden. The head RULES these; it never swallows them.
└── .trash\                   ← stays at the root (universe-scope trash; R3 pending on the 55 rows)
```

Nothing else can ever be created inside the head — **no New Note, no New Folder, no Maps\ folder**
(an open folder with a creation verb is the refill door re-armed one level down; bounded-ness is
what distinguishes a head from the accident being removed). The Universe MOC is one bounded note.
`_Core` is an ASCII code constant — never localized, never user-settable (directory paths are
per-universe-unique by construction, so the constant is safe on disk; the NAME rule is 3.6).

### 3.4 How it appears — **Boss-ruled: NO icon**

A **section, first in the sidebar**, above cUniverse and library rows, replacing the special-case
"Universe Notes" block (`+layout.svelte:7545-7580`) entirely. **No icon** — the Boss's 2026-07-25
steer stands, re-affirmed 2026-07-26 with the review's stronger reason: in this sidebar a glyph
means *"a place you can enter"* (building = library, planet = cUniverse); the head is not a place —
it rules the places — so its unmarked state is **contrastive** and reads "not one of those."
`LibraryIcon`'s `kind='root'` is deleted from the union with **no replacement kind**. No count badge
(126 was the defect made visible). Clicking the section label opens the **Universe MOC in NotePane**
(the gate to cognition). Two zones, each explained by one on-screen `$t()` sentence:

- **Zone A — its own notes** ("These are this Universe's own notes — its front page and its molds.
  Changing them changes what it *says*, not how it *works*."): the existing `FileTree`, repointed at
  `_Core`. Containment here is true on disk, so chevron and indent are honest. With no MOC yet, the
  first row reads *"Write this Universe's front page…"*.
- **Zone B — what it rules** ("Change anything here and every Library in this Universe behaves
  differently."): door rows — **Libraries · Linked Universes · Link types · Stages · Property
  types** — each passing BOTH tests (it rules; it has no other sidebar home), each reading its live
  value from the small JSON it describes (no walk, no Rule-8 exposure), rendered only when
  non-empty, handing off to its owning surface. Stages and Property types are read-only in v1 and
  say so (no editor exists for either — verified). Bases and Bookmarks stay top-level, untouched.

Style Setter: new element `coreOrganizer` in the `interface` category + `TREE_ELS`, chain
`--ft-core-*` → `--ft-master-*` → `--interactive-accent`; the `data-style-target="library"` borrow
is deleted.

### 3.5 What it refuses — **Boss-ruled: the root holds no user notes, ever**

> "The user cannot have notes in the root library. If they create a new note, they either have to
> add it to the existing ones or create a new library for that reason." — Eisa, 2026-07-26

This is the **total amendment** the Inspectors made a hard condition (a partial amendment — anything
left ownerless at the root — is *worse than doing nothing*: eleven silent-skip sites, a fabricated
name at `search.rs:6435-6436`, a self-heal that stamps success over rows it cannot fix).

1. **Not a write target.** All seven creation paths (`New Note :4269`, Quick Capture `:4395`, New
   Folder `:4458`, Daily Notes `:4758`, quick-switcher create `:5466`, Reviewer scope `:7981`,
   `ImporterModal.svelte:21`) repoint to `appSettings.homeLibrary` — which **asks** when unset (The
   Constellation Way; never a silent default). A fresh Universe's first note triggers "name your
   first Library."
2. **Not a Library**: absent from `$libraryStats`, `$libraryCount`, Library Manager, Switcher,
   Picker, Bases query list, Dashboard grid, OrgChart, `map.rs:555`. The status-bar count becomes
   honest (decrements by one).
3. **Not removable, not renameable** — refusal by absence (no Remove row, no rename command). The
   silent re-insert (`universe.rs:403-414`) and the unguarded Remove (`LibraryManager.svelte:91`)
   both dissolve structurally.
4. **Not a filesystem ancestor of any library** — `add_library` gains the containment refusal
   (today it checks only exact-path duplicates, `libraries.rs:394`).
5. **Not index 0 of anything that matters** — the surviving first-match resolvers become harmless
   AND are still fixed (P8, WA#6); callerless `get_library_mode` deleted.
6. **Not a generic Move destination** — exactly ONE labelled door remains: `_Core\Templates`
   ("make this note a mold" is a shipped capability that a blanket refusal would silently delete).
7. **Not the trash, not an inbox, not a manifest, not the machine's state** (`search.db`,
   session/boot/diagnostics untouched; `search.rs:8984` `current_version` untouched).
8. **`notes_folder` is never written again** (it arms the flatten landmine via `rename_universe:1008`)
   but stays readable for multi-device sync. The scope carrier is the entry's own `path` + `kind`.

### 3.6 The reserved-name discipline and federation (PR-F3) — constraint locked, token format = Plan

Library identity in this system IS a name (name-keyed joins at `libraries.rs:659/:774/:811`; no id
columns anywhere). **Therefore: the stored `name` of every Core entry must remain per-universe-unique;
the user-facing label is always `$t('coreOrganizer.title')` via one shared `libraryLabel()` display
helper keyed on `kind`/name** (the `LibraryIcon` one-component discipline; guard test added to
`vitest.config.ts` `test.include` — P9). A shared stored token (`_Core` as the *name*) across N
federated heads would merge them into one phantom library in every name-keyed join — an app-killer
(Inspectors PR-F3, mechanism hand-verified). Recommended token format for the Plan:
`"_Core — <Universe name>"` frozen at migration time (unique, visibly a system token, survives a
later universe rename without re-pinning). Federation staging (PR-F1/F2/F4): core entries stay in
`resolve_libraries_recursive`'s merge, filtered by `kind` at display surfaces; `attach.rs:135` swaps
to the manifest-driven `resolve_child_universe_roots` (verified: exists, two production callers)
with a FederationWarning for undiscoverable children; an explicit "the ACTIVE universe's Core"
accessor replaces all five first-match `.find(is_universe_notes)` sites; a per-universe Brain-shape
version is stamped and checked at attach.

### 3.7 Re-attribution (non-negotiable)

A direct SQL `UPDATE`, never a reindex (`index_note`'s fast path compares the library name at
`search.rs:6313`; a rebuild re-INSERTs every edge with `created = now` — 139 of 147 edges would
silently reset). `note_links.library_name` must be UPDATEd explicitly (no trigger exists for it).

---

## §4. THE MIGRATION OF THE EXISTING ROOT CONTENT — **Boss-ruled 2026-07-26**

> "Regarding the loose folders and notes that exist today, we will move them all under the
> **'Eisa Test'** Library." — Eisa, 2026-07-26

| Content at the root today | Destination (ruled) |
|---|---|
| 45 loose `.md` (44 indexed + the invisible `Testing opened note.md`) + `EisaTest.canvas` | **Eisa Test** (`E:\Cognitive Knowledge\Eisa Test`, id `library_18b8aa76588135405360`, 78 notes today) |
| `Daily Notes\` (4) · `Imported\` (2) · `MIG-005 Test\` (4) · `Replication Bug\` (4) | **Eisa Test** (folders move whole) |
| `Templates\` (12 molds; incl. 6 `TS_test_*`) | **`_Core\Templates`** — the 6 `TS_test_*` debris molds move with the folder; the Boss can delete them there |
| `Five Acts\` (1 host note) | **`_Core\Five Acts`** |
| `.trash\` (55 indexed rows) | **stays at the root** (universe-scope trash); the 55 rows are ruling R3 |
| `Creating new library\` (registered, nested, 0 notes) | grandfathered; containment rule refuses new nesting |

Arithmetic (verified): 126 root-owned rows = 55 `.trash` + 71 moving — of the 71, **13 → `_Core`**
(12 Templates + 1 Five Acts) and **58 → Eisa Test** (44 + 4 + 4 + 4 + 2), plus the unindexed note
and the canvas by the disk walker. Eisa Test grows ≈ 78 → ≈ 138 notes.

Execution rules: the mover reads the **disk**, never the index (P6); the extended 11-table path
cascade (P1) is a hard prerequisite; move-first → verify-root-clean → flip-registry-last ordering
(no unowned instant); `migrate_note_db_paths` runs per moved file; re-attribution per §3.7;
settings that hold root paths (`templateFolder` — absolute on disk today, `dailyNoteFolder`,
collections/workspaces/review-pulse path keys, `property-types.json`'s library-name key carrying
the Boss's `domain`/`ikhtilāf` typings) are rewritten in the same journaled pass; every root-writer
repoints in the same commit (`lens/system_notes.rs:94-99` Five Acts every boot; `get_templates_dir`;
`undo_adopt_kind`); a migration report lists every move (File-Over-App: this ruling is the explicit
user action; the report is its record).

Honest note, stated once: "Eisa Test" will then hold both test debris and the ~8-10 real knowledge
notes (`Eisa ALSHAMSI`, `المعرفة العربية`, …). That is the Boss's call and freely reversible later
with ordinary Move; the migration proposal screen still shows the full manifest before executing,
and the "make this my Universe's front page" offer remains available for any of them afterwards.

---

## §5. Prerequisites — nothing above is planned until these land

**Stage 0 — live defects (all exist today, independent of MIG-105; WA#6):**
P1 path cascade covers 5 of 11 path-bearing tables (every rename/move orphans history — 1,312
`note_state_history` rows for the root notes alone) · P2 `PRAGMA foreign_keys` enabled only inside
one `#[test]` · P3 reconcile self-heal failing silently at scale (1,577 unexplained "relocate
deferred"; the error is discarded at `reconcile.rs:192` — Reproduce-First applies to the self-heal)
· P4 `UniverseMeta` has no catch-all: `custom_stages` (the Boss's own 🏷️ stage — zero readers,
zero writers, verified) is destroyed by rename/attach/detach · P5 17 root notes lack `cid_cn` ·
P6 the invisible root note · P7 second-screen fixed-name walker (`collect_notes_names_recursive`)
mislabelling nested libraries' notes · P8 three surviving first-match resolvers + callerless
`get_library_mode` · P9 the vitest `include` allow-list.

**Tier-1 (Inspectors v2):** PR-1 kill the three path-healers + version-gate `universe.json` ·
PR-2 one explicit "no owner" representation, as a refusal · PR-F1 federation anchor preserved
(satisfied by §3.1) + manifest-driven discovery · PR-F2 active-universe Core accessor at the five
`.find()` sites · PR-F3 per-universe-unique stored name (§3.6) · PR-F4 Brain-shape version checked
at attach · PR-5 UPDATE-not-reindex (§3.7).

**Tier-2:** PR-6 root-writer + stored-settings repoints · PR-7 scope carrier never `notes_folder`;
`#[serde(flatten)]` on `UniverseMeta` · PR-8 "it should remain" as a code guard · PR-F5 non-active
universes read-only (or index-follows) · PR-F6 rule on the 620 parent-held child rows · PR-9
three-state load contract (Loaded | Absent | Unreadable) for every head file.

**Found in passing, filed for fixes/rulings:** sidebar chevrons never RTL-flip; "vault" shipped in
10 locales (`universe.manager.*`); `.section-label` uppercase breaks Arabic cursive joining;
`RECENT_CAPTURES_CONTENT` English-only; the 939 MB orphan `Constellation SV Test.db` + unbounded
logs (R8); the un-inspected appearance/i18n/Style-Setter surface (its own inspection before Phase 3).

---

## §6. Invariants that must not break

1. Every existing note keeps an owner **at every instant**, including half-applied states.
2. On-screen === disk === index for every note (Editor-Surface Gate).
3. `search.db` is the system of record for earned link data — re-attribution by UPDATE only.
4. **Library ≠ Folder — and Head ≠ Library — enforced by the data model**, not remembered by walkers.
5. Longest-root-wins stays canonical; after the fix first-match cannot differ.
6. Federation survives a partial upgrade (per-universe version marker, attach-time check).
7. No regression in boot, indexing, or IPC on the 7,816-note universe.
8. Cross-platform paths (casing, NFC/NFD, separators) — Windows now, macOS later.
9. Reversible, with pre-flight copies of `libraries.json` + `universe.json`.
10. Absent is a fact; unreadable is an unknown; **registered-but-missing is surfaced, never healed
    silently**.
11. **The amendment is TOTAL** (no ownerless region ever exists) **and FEDERATION-STAGED** (no mixed
    old/new shapes readable in one federated view).

---

## §7. Rulings

### RULED (Boss, 2026-07-26)

| # | Ruling |
|---|---|
| Concept | The head **remains**; scope amended. It is the Universe's Brain / Core Organizer. |
| Operative steer | 07-25 governs ("nothing under the root but cUniverses and Libraries"); the concept is *why* it is right. |
| Name | **"Core Organizer"** (label; localized). "Root/System Library" and "Core MOC" rejected with accepted reasoning. |
| Icon | **None.** The 07-25 steer stands; `kind='root'` deleted, no replacement. |
| Front page | The **Universe MOC** — one note, charter + map in one document, inside `_Core`, offered never auto-created. |
| Contents | Closed kinds: Universe MOC + `Templates\` + `Five Acts\`. Nothing else, ever. |
| Root rule | **Total**: no user notes at the root; every new note requires a home Library (ask when unset; first note in a fresh Universe creates the first Library). |
| Loose content | **Everything moves to "Eisa Test"** (except Templates/Five Acts → `_Core`; `.trash` stays). |

### OPEN — next Boss decisions (with recommendations)

| # | Question | Recommendation |
|---|---|---|
| R3 | The 55 `.trash` index rows | Purge index-only — but only after the Plan reads the delete/watcher path end-to-end to prove the leak source is closed |
| R4 | Stage 0 now, before anything else? | **Yes** — every item is a live defect; the migration cannot be built on them |
| R5 | MIG-104 (durable earned-link data) before MIG-105? | **Yes** — every link-data risk becomes recoverable instead of permanent |
| R7 | `sight_v3` cache (7,636 rows keyed by library paths) | Purge in the same transaction; accept one recompute |
| R8 | The 939 MB orphan `Constellation SV Test.db` + uncapped logs | Delete the orphan, cap the logs — as its own PJ |
| Token | Exact stored-name format for Core entries (§3.6 constraint is locked) | `"_Core — <Universe name>"` frozen at migration; confirm at Plan approval |

**Step-1 blocker (unchanged):** the app-data registry `universes.json` (Roaming) lists only
كون عيسى while ECK is demonstrably the universe in use — instrument `save_registry` and read the
answer before the migration names its target.

---

## §8. The reproduction harness (required before any code)

Three layers on proven project patterns (`tests/mig-076/runtimeHarness.test.ts`,
`tests/pj-140/backlinksLinkMention.test.ts`):

1. **One fixture, every resolver, one assertion** — a TempDir universe in the live shape (root
   entry, nested registered library, name-prefix siblings, `.trash`, one unindexed file): *every
   enumerator that reports a file reports it under the owner `library_name_for_path` returns,
   exactly once.* RED today (`collect_notes_names_recursive`, `get_library_mode`).
2. **The frontend twin** — `libraryForPath` / `libraryIdForPath` / `deriveLibraryForPath` agree
   with Rust; `buildUniverseFolderEntries` yields no duplicate key **with the `seen` dedupe
   removed**; the `libraryLabel` guard test (no surface renders a raw core name).
3. **Migration recipes, killed at every step boundary** — after each kill+resume: every `.md` has
   exactly one owner; no index row points at a missing path; no row in any of the six un-cascaded
   tables lacks its `note_meta` row; `note_links.library_name == note_meta.library_name`.
   New recipes for the rulings: root-create refusal (every creation path), fresh-universe
   first-note → first-Library flow, the Eisa-Test bulk move (interrupt at worst moment), the
   `_Core\Templates` Move door, three-federated-Brains name-join integrity, headless-universe
   detection (`link_library_as_universe`).

Every new test file must be added to `vitest.config.ts` `test.include` (P9) or it silently never runs.

---

## §9. Ledger & references

To file at the next PJ bump (SO#9): Stage-0 items P1–P9 as individual entries where not already
filed · PR-F1–F6 · the appearance/i18n inspection · the N-3 defect batch (RTL chevrons, "vault"
locales, cursive captions, English-only const) · R8 (orphan DB + log caps) · the registry
instrumentation blocker.

Companion documents: `MIG-105-ArtDirector-verdict-Core-Organizer.md` (design evidence, item-by-item
inventory ruling, deletions list) · `MIG-105-Inspectors-verdict-Core-Organizer.md` (safety verdict,
prerequisites, unenforceable-invariants audit) · `MIG-105-root-library-vs-flat-universe.md` (the
original logged concept + evidence trail, 2026-07-24).

**Next step:** Boss approval of this document → R3/R4/R5/R7/R8 + token rulings → Stage 0 →
`/migration` Phase 2 (Plan), presented for approval before any code.
