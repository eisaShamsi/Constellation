# THE ART DIRECTOR & TEAM'S FINAL VERDICT (v2) — MIG-105

**Function in hand:** the entity currently rendered as the "Universe Notes" section at the top of the sidebar's library list (`src/routes/+layout.svelte:7545-7580`), registered at index 0 of `.constellation/libraries.json` by `ensure_universe_notes_folder` (`src-tauri/src/universe.rs:252`).

**What this re-run had that the first run did not:** all FOUR designs (Designs 3 and 4 were never previously judged by anyone), the FULL spec §1–§7 (the designers had only worked from ~§5 onward), the recovered federation inspection, and four fresh judge verdicts. Every mechanism claim below that my ruling turns on was verified by me in the repo this session (`main @73d28bed`), including one direct factual contradiction between two judges, which I settled by reading the line myself.

---

## §7 ★ THE DELTA vs THE PRIOR VERDICT — read this first

The prior verdict was built from Designs 1–2 only and the spec's tail. With the full evidence in, here is exactly what changed and what survived.

### REVERSED (4 rulings)

**D-1. The mark. Prior: invent a new "nucleus" glyph (`kind='core'`) and recommend revisiting your 07-25 "no icon" steer. NOW: NO MARK — your steer stands, and we recommend it NOT be revisited.**
Why it flipped: both factual premises under the spec's (and our prior) mark argument are **verified false**. (a) "Every other section is marked" — the Bookmarks section header is shipped **unmarked plain text** (`+layout.svelte:7415`, `<div class="section-label">{$t('sidebar.bookmarks')}</div>`; the ⭐ is per-item at `:7422`). I read it. (b) "Reuse the Universe's existing mark, no new glyph invented" — there is no shipped Universe identity mark to reuse: the `◇` renders only inside a Style Setter preview card (`StyleSetter.svelte:1483`) **and is already the Template Studio's command-palette icon** (`+layout.svelte:2361`, `icon: '◇'` — I read the line; it is written as a unicode escape, which is why one judge's grep missed it); the shipped footer glyph is a chevron-pair whose only `onclick` is `showLibrarySwitcher = !showLibrarySwitcher` (`:7718-7719`) — its live meaning is "open the switcher," not "the head." Designs 3 and 4 — the two never previously judged — independently derived the no-mark conclusion from the settled primitive, and three of the four judges confirmed it. Under the locked design Five Acts becomes the head's **child**, so the "only unmarked sibling" comparison class dissolves entirely (Design 3's argument), and in this sidebar a glyph already means "a place you can enter" — the head is not a place, so **contrastive absence IS the mark** (Design 4's argument). The prior nucleus glyph was new pictorial vocabulary against an eleven-system prior-art sample containing zero decorative head-marks. Full reasoning in §6.

**D-2. The name. Prior: "Core" (`$t('core.title')`). NOW: "Core Organizer" (`$t('coreOrganizer.title')`) — your own phrase, verbatim.**
Why it flipped: the Boss-legibility judge is right that a one-word abstraction is more cryptic to the owner than the phrase the owner himself coined, and Design 3 demonstrated that "Core Organizer" translates as a **role** in all 15 locales (ar المُنظِّم الأساسي · fa سازمان‌دهندهٔ اصلی · he מארגן הליבה · ur بنیادی منتظم · zh 核心组织者 …), which removes the prior verdict's localization objection. The prior objection that "Organizer" is management vocabulary loses to a stronger rule: they are the Boss's own words for his own concept. "Brain" stays in the description, never the label. The i18n namespace becomes `coreOrganizer` (prior: `core`).

**D-3. `Maps\` and the "New Map…" verb are CUT from the head. Prior: `Maps\` created lazily with one creation verb. NOW: the head's content kinds are CLOSED to exactly three — one Charter, `Templates\`, `Five Acts\`.**
Why it flipped: the Form judge's finding against Design 2 applies to our prior ruling word for word — a folder that can hold arbitrarily many user MOC notes, with a creation verb on the head's own menu, is **the refill door re-armed one level down**. The head's distinguishing property versus the accident being removed is bounded-ness, and Design 3's rule is the right one: the refill question is not "how many" but "**which kinds**," closed in code. Your structure notes live in Libraries, where the migration proposal can file them.

**D-4. The Move picker. Prior: the head is not a destination, full stop. NOW: the head is not a *generic* destination, but exactly ONE labelled door remains — `_Core\Templates`.**
Why it flipped: Design 3 caught a regression nobody else saw (including our prior verdict and the spec's §5D): `<root>/Templates` is a reachable Move destination **today**, and "I wrote this note and now I want it to be a mold" is an ordinary action. Removing the root library without that door silently deletes a shipped capability. One door, labelled by function, no kind-inference.

### SURVIVED — with new corroboration (the load-bearing rulings)

**S-1. The entity REMAINS in `libraries.json`, re-typed `kind:"core"`, path amended to the leaf `<root>\_Core`.** This is the prior verdict's central mechanism, and it survives against both the spec's §1 ("no registry entry") and Design 3's no-entry-plus-token mechanism. Four reasons, now sharpened by the new evidence: (i) "it should remain, it just need its scope amended" is literally true only in this reading. (ii) The federation anchor survives **by construction** — `_Core`'s ancestor chain passes through the Universe root, so `unique_cuniverse_roots`' ancestor walk (`federation/attach.rs:71-118`) still lands; no replacement anchor is *required* (we adopt one anyway — N-1 below — so discovery is dual-assured). (iii) Design 3's own sharpest self-flagged risk — `deriveLibraryForPath` (`store.ts:2301-2313`) returning `libraryPath: ''` on a miss, which **silently kills at least seven surfaces** the moment a template mold is opened — cannot occur, because the head IS an entry the resolver finds. (iv) Design 3's honest count of its own alternative is **nine new owner-resolution seams** for thirteen notes; keeping the entry collapses those to roughly three (the kind-filter at registry consumers, the display helper, the Move-picker door). In a codebase where LL-014's strikes are spent and the §CB failure shape was live half-migrations, fewer new seams wins. The buildability judge's two hard critiques of this mechanism are both resolved in the lock: core entries **stay in `resolve_libraries_recursive`'s merge** (so the anchor claim and the merge ruling no longer contradict) and are filtered by `kind` at display surfaces; and the resolver wiring is explicit, not ambiguous.

**S-2. The primitive: a two-zone SECTION, first in the sidebar, replacing `:7545-7580` entirely.** The prior verdict converged on this shape from Designs 1–2; Design 3 — the top pick of both the concept judge and the Boss-legibility judge — independently converged on the same two-chamber shape. We now also adopt Design 3's best device: **the seam is stated in words, in the UI**, one `$t()` sentence per zone (see §2).

**S-3. The Zone-B boundary rule survives and FIXES the winning design's worst defect.** Prior rule: a door exists only for a vocabulary or registry that has **no other home** in the sidebar. Design 3's own seam test ("change it and every Library behaves differently") is violated by three of its own slots — Bases, Lenses, Collections do not rule (delete every `.base` and nothing behaves differently; two judges confirmed). Our five doors (Libraries · Linked Universes · Link types · Stages · Property types) all pass both tests. Bases, Collections/Bookmarks, Workspaces, Settings, Reviewer: OUT.

**S-4. Bookmarks and Bases stay top-level, untouched.** Design 3's burial of your daily-use Starred shelf three levels deep was its single biggest legibility defect (both relevant judges), and Design 2's frequency-argued exception was right. Sustained.

**S-5. The Charter: one, offered, never auto-created; clicking the section label opens it in NotePane** (the gate to cognition, your 2026-07-18 top-principal). Sustained — and we ADOPT Design 3's third migration destination: for your ~8-10 ambiguous root notes ("تجربتي", "عيسى المطور", "Eisa ALSHAMSI"…), the proposal offers "**make this my Universe's charter**" alongside "a Library" and "delete." Only a content-holding head can offer that.

**S-6. Zone A renders via the existing `FileTree` (repointed at `_Core`); `hideFiveActsFolder` is KEPT and repointed** (not deleted — deletion is only correct if the Five Acts section is also removed, which is scope creep into MIG-055 §F); the Five Acts top-level section is unchanged. Sustained.

**S-7. The colour ruling survives and is now doubly verified.** `buildLibraryColorMap` is positional (`colors.ts:12-16`) and built from the full `$libraries` (`+layout.svelte:1465` — I read both this session). The entry remains in that list, so index 0 stays occupied and **no library's colour changes**. (Design 4 verified the same mechanics from the other direction.)

**S-8. The Style Setter correction survives.** There is no `tree` *category* — the spec's §5C.12 and both original designs transcribed a preview grouping as a category. Ruling stands: add element `coreOrganizer` to the `interface` category and to `TREE_ELS`, chain `--ft-core-*` → `--ft-master-*` → `--interactive-accent`, delete the `data-style-target="library"` borrow (`:7547`).

**S-9. All structural refusals survive** (§2 "what it refuses"): not a write target (seven repoints to `appSettings.homeLibrary`, which ASKS when unset), not the trash (`store.ts:3846` repointed same-commit or delete throws at `:3853`), not a manifest, not the machine's state (`search.rs:8984` `current_version` untouched), not renameable/removable (refusal by absence), `notes_folder` no longer written but the flatten branch stays readable for multi-device sync (Design 1's argument, sustained), containment refusal added to `add_library` (`libraries.rs:382-416` checks only exact duplicates today), the self-violating heal (`universe.rs:839-848`) refuses-and-surfaces.

**S-10. Rulings R1–R5 and R7 and the `universes.json` blocker survive unchanged** (§5).

### NEW — adopted from Designs 3/4 and the judges (never in the prior verdict)

**N-1. The federation consistency fix is verified and adopted.** `resolve_child_universe_roots_recursive` exists at `universe.rs:528` with two production callers (`bases.rs:649`, `lens/system_notes.rs:186`) — I re-verified all three sites this session. `attach.rs:135` swaps to it, collapsing the two disagreeing notions of "the federated set" into the one you declared. With S-1's anchor-by-construction, discovery is dual-assured. The spec's "single largest structural risk" (§7.13) was overstated: it is a bounded one-call substitution.

**N-2. Judge-conflict adjudicated: the ◇ collision is REAL.** The buildability judge marked Design 1's claim "VERIFIED FALSE" after grepping for the literal glyph; the palette entry writes it as the escape `'◇'` (`+layout.svelte:2361` — read directly). Design 1 and the other two judges were right; the buildability judge's refutation is itself false. Consequence for the Plan: `◇` is live-ambiguous today (Template Studio palette + Style Setter Universe preview) and no design may stack a third meaning on it — which independently reinforces D-1's no-mark ruling.

**N-3. Four shipped defects on the exact surface under design, all verified this session, all fixed in-pass (WA#6 — fix what you discover):**
- `.v-chev` is never RTL-flipped — the file's only flips are `:10298` (`.tab-scroll-arrow`) and `:10624` (`.index-return-btn`); every collapsed library/cUniverse chevron points the wrong way in ar/fa/he/ur, against CLAUDE.md's explicit rule. One rule fixes it.
- "vault" terminology shipped in 10 of 15 locales inside `universe.manager.*` (verified: ja 保管庫, ru хранилища, pt Cofres, plus fa/he/hi/ko/tr/ur/zh) — against the standing "Library, never vault" rule, on the very surface (LibrarySwitcher) this migration touches.
- `.section-label`'s `text-transform: uppercase; letter-spacing: 0.04em` (`:10058`) breaks cursive joining in ar/fa/ur and is a no-op in 8 scripts — the zone captions use a new `.co-caption` (size/weight/opacity subordination), and the shipped Bookmarks caption is fixed in the same pass.
- `RECENT_CAPTURES_CONTENT` (`lens/system_notes.rs:47-77`) ships the Five Acts host body English-only as a Rust `const` — the Charter seed must be selected by app language; the existing const is fixed alongside.

**N-4. The carried flag now has EVIDENCE and becomes a filed PJ.** The Chief Inspector's guess about the missing inspection's topic was wrong (it was federation, now received) — but the underlying flag stands and this re-run proved it live: **no inspection has ever covered the appearance / naming / i18n / Style-Setter surface**, and the first two designs ever to work that surface found four shipped defects in one pass (N-3). File the appearance/i18n/Style-Setter inspection as its own PJ and run it before Phase 3. This flag is carried, not dropped.

**N-5. Two housekeeping discoveries for a Boss ruling, not silent action:** `.constellation/Constellation SV Test.db` — a **939,413,504-byte** orphan database (mtime Apr 23) sitting beside the live 2.0 GB `search.db` — plus `boot-perf.history.jsonl` (2.4 MB) and `diagnostics.log` (1.5 MB) growing unbounded. Deleting a user's file is your decision; filed as a PJ candidate in §5.

**N-6. A documentation-drift correction:** the spec and surveys cite `src/lib/i18n/locales/en.json`; there is no `locales/` subdirectory — the 15 files live directly in `src/lib/i18n/` (Design 3's catch, consistent with every grep I ran there).

---

## 1. WHAT WE THINK OF THE CONCEPT

**Unchanged from the prior verdict, and now stress-tested from four directions: it holds up, and it was the missing horse.**

The Architect's options A–D all asked what the entry should *own*; none asked what it is *for*. Your 2026-07-26 statement supplies the purpose, and the live data vindicates it better than any argument we constructed: the Universe root today holds ~35 development-debris notes, a `.trash` of 55 indexed rows, and one nested library — while the actual identity apparatus (`libraries.json`, `link-types.json`, `custom_stages`, the bases) sits invisible in `.constellation/`. **An accident has no boundary; a head does.** Design 4 put it best: the Brain is not a missing thing — it is an invisible thing wearing a Library's costume. And Dendron's recorded failure ("folks are unsure what it is for") is answered by the one input Dendron never had: you said what ours is for.

The full four-design bench also settled the concept's two ambiguities empirically:
- **"all vital and core files/contents" means BOTH the governing apparatus and your own identity writing, in two clearly separated zones.** The apparatus-only reading (Design 1) refuses the head any `.md` and its own author concedes "if he means notes, my design refuses his brief"; the minimal reading (Design 4) reduces "all vital and core" to name+created+version plus two vocabulary files and does not even surface `custom_stages` — the one file you authored that the app has never read and silently destroys (P4). The concept judge scored that reading 4/10 as "an excellent Stage 1, not a Brain." The two-zone reading carried both of your predicates — rule AND include — at full weight, and it won the two lenses closest to your seat.
- **07-25 governs 07-24, and the concept explains why:** in the entire surveyed field there is no system with both a note-owning root and a visible root identity object. 07-24 was a question; 07-26 answers it.

---

## 2. THE DESIGN — LOCKED

### What it IS

**The Core Organizer** — a first-class entity that **remains** in `.constellation/libraries.json` (same `id`, so `libraryAppearances` survives), with its scope amended in exactly four dimensions:

| | Today | Amended |
|---|---|---|
| **path** | the Universe root — physically contains every library under it | `<root>\_Core` — a **leaf** containing nothing but itself |
| **name** | the Universe's own name, re-pinned on every rename | the constant `_Core` — never tracks the Universe name again |
| **kind** | `is_universe_notes: true`, a flagged member | `kind: "core"` — a different class, absent from every library-facing surface |
| **contents** | 126 notes of debris + `.trash` + other libraries | the Universe's identity: one Charter, `Templates\`, `Five Acts\` — a **closed** kind-list |

The path amendment kills all three born defects at once: a leaf cannot enclose its siblings (Library-as-Folder, the Move-picker freeze and double-indexing become unreachable, not filtered); first-match and longest-match give the same answer; and the name stops being the most fragile identity in the system (there is no library-rename command anywhere; `rename_universe` was the only mutator and it updates nothing in the index).

### On disk

```
<Universe root>\
├── _Core\                    ← THE HEAD. kind:"core". Leaf. The only visible directory at the root.
│   ├── <Charter>.md          ← ONE, offered, never auto-created; pointer in universe.json → core.charter
│   ├── Templates\            ← moves in (12 molds; the 6 TS_test_* go to the debris proposal)
│   └── Five Acts\            ← moves in (1 host note); FIVE_ACTS_DIR + lister repoint together
├── .constellation\           ← UNCHANGED, hidden. The head RULES these; it never swallows them.
└── .trash\                   ← STAYS at the root; trashFolderScope='universe' resolves to the root
                                DIRECTORY (already authorised, libraries.rs:319-348)
```

`_Core` is an ASCII code constant, never localized, never user-settable (a language-switch rename would orphan every absolute path in five stores, and the path cascade covers only 5 of 11 tables today — P1). Gate: `store.ts:3637 isTemplatePath`'s cid-fence, `appSettings.templateFolder` and `resolve_templates_dir`'s default move in the **same commit** as the folder, or the next template edit injects an identity line into a mold.

### How it appears

A **section, first**, above the cUniverse rows (`:7583`) and library rows (`:7641`), replacing `:7545-7580` entirely. **No mark** (§6). No count badge (126 was the defect made visible: 55 unreachable `.trash` rows + ~35 debris). No Libraries-ruled badge either — the status bar already renders `$tn('plurals.libraries', …)` at `:9685`, and a badge whose removal costs nothing is decoration by our own test. Label: `$t('coreOrganizer.title')` — a translated string, never data (today's raw un-`$t()`'d `.name` at `:7553` is a live full-localization violation this closes).

Expanded — two zones, each explained **in the UI itself** by one `$t()` sentence (Design 3's device, the best legibility idea in the packet):

**Zone A — ITS OWN NOTES.** Caption: *"These are this Universe's own notes — its charter and its molds. Changing them changes what it says, not how it works."* The existing `FileTree`, repointed at `_Core`. Containment here is true on disk, so the chevron and indent are honest. When no charter exists, the first row is *"Write this Universe's charter…"* — one click creates it at a name you confirm, with `init_at`'s never-overwrite / delete-to-restore contract (`lens/system_notes.rs:110-118`), seeded in the app language.

**Zone B — WHAT IT RULES.** Caption: *"Change anything here and every Library in this Universe behaves differently."* Door rows in the `section-label` grammar (no chevron — nothing false about containment): **Libraries · Linked Universes · Link types · Stages · Property types**. Every door passes two tests at once: the blast-radius test (it rules) and the no-other-home test (nothing else in the sidebar owns it). Each reads its live value from the small JSON it describes (largest ~4.5 KB — no walk, no `scan_*`, no Rule-8 exposure), renders only when its subject is non-empty (no "Linked Universes (0)" row — a Universe with zero cUniverses is complete), and hands off to its owner: Library Manager, Universe panel, link-types editor. **Stages and Property types are read-only in v1 and say so on their face** — no editor exists for either (verified; `propertyTypeRegistry.ts` is a store, not a surface). Zero duplication (2026-07-05 ruling).

### What the user can do

Open (label → Charter in NotePane) · expand · write the Charter (once, offered) · **New template** (Template Studio's own verb, not a second implementation) · Reveal in Explorer · Style · a command-palette entry. **Nothing else can be created in it.** No New note / New folder / New base (today's `:6037-6040` root branch dies), no `Maps\`, no inbox.

### What it refuses

All twelve refusals of the prior verdict stand — not a Library (absent from `$libraryStats`, `$libraryCount`, the Manager, Switcher, Picker, Bases query list, Dashboard grid, OrgChart, `map.rs:555`); not removable or renameable (refusal by absence — today's silent re-insert at `universe.rs:403-414` with the unguarded Remove at `LibraryManager.svelte:91-93` is the Dendron anti-pattern exactly); not index 0 of anything that matters (the six first-match resolvers become harmless AND are still fixed, P8/WA#6; callerless `get_library_mode` deleted); not a default write target (seven repoints to `homeLibrary`, which asks when unset); not the trash; not a manifest; not the machine's state; not `notes_folder`-written; not an open content set. Amended per D-4: not a generic Move destination, but `_Core\Templates` remains one labelled door.

### The reserved-name discipline

`note_meta.library_name` is a NAME with no id column in any of the 36 tables, so `_Core` is a reserved value for the head's ~14 notes. **One shared display helper** — `libraryLabel(name)` mapping `_Core` → `$t('coreOrganizer.title')`, everything else passthrough — applied at every raw-name render site, using **Design 2's 19-site census as the checklist** (`NotePane.svelte:1450` breadcrumb first; I verified that sample), following the `LibraryIcon` one-component discipline, plus a guard test that **must be added to `vitest.config.ts` `test.include`** or it silently never runs (P9). If a site is ever missed, the leak reads `_Core` — unmistakably a system token, not a plausible-but-wrong library name.

### Federation

Core entries (this Universe's and every child's) **stay in `resolve_libraries_recursive`'s merge** and are filtered by `kind` at display surfaces — so the ancestor-walk anchor holds by construction. Additionally, `attach.rs:135` swaps to the manifest-driven `resolve_child_universe_roots_recursive` (`universe.rs:528`; verified, two production callers) — discovery reads the federation you declared instead of guessing it from paths, and attach/Bases/Five Acts stop disagreeing about the federated set. Child universes' `_Core` notes are **not surfaced in v1** (a child's charter shown read-only per-universe is a later, separate decision). Parent and child may be on different binaries; the reserved constant makes that safe.

### Re-attribution (non-negotiable, both verified)

A direct `UPDATE`, never a reindex — `index_note`'s fast path compares `o_lib == &library_name` (`search.rs:6313`), so a rebuild re-INSERTs every edge with `created = now`; of the head's 147 edges, 139 are un-earned and would silently reset. And `note_links.library_name` must be UPDATEd explicitly — nothing in the codebase ever writes that column and, unlike `sky_nodes` (trigger `note_meta_sky_au`), it has no trigger.

---

## 3. THE ITEM-BY-ITEM RULING ON THE LIVE INVENTORY

| Item (verified live) | Ruling |
|---|---|
| `.constellation\` | Unmoved, unswallowed. The head rules it via Zone-B doors. The empty `.constellation\templates\` is **deleted** (a privileged empty directory beside a working pointer is a trap). |
| `custom_stages` — live `[{name:"concept", emoji:"🏷️"}]` | Modelled + rescued + surfaced. `UniverseMeta` gains the field AND `#[serde(flatten)]` (P4 — without it your stage dies on the next rename/attach re-serialize). Zone-B door, read-only in v1. **The head is its first reader in the app's history** — zero readers, zero writers today (Design 4 re-verified). |
| `libraries.json` (20 entries) | Zone-B door → Library Manager. The head's own entry re-typed per §2. |
| `universe.json.children[]` (live `[]`) | Zone-B door → Universe panel. Renders only when non-empty. |
| `link-types.json` (3: `contains`, `parent`, your `inspires`) | Stays; Zone-B door → link-types editor. |
| `property-types.json` (4 library-name keys) | Stays; Zone-B door, read-only v1. **Blocker:** the key `"Eisa Cognitive Knowledge"` carries your `domain`/`ikhtilāf` typings and must be rekeyed to the destination Library in the same commit or they silently stop applying (`propertyTypeRegistry.ts` keys by library name). |
| `bases\` (3) + `lenses.json` (2 builtin) | **Stay top-level, untouched.** No Core door (they have a home, and a base does not rule — deleting one changes no behavior). Design 2's and Design 3's absorption both rejected. |
| `bookmarks.json` / the Starred shelf | **Stays top-level, untouched.** Design 3's burial rejected — it is your daily surface. |
| `collections.json` / `workspaces.json` / `settings.json` / `review-pulse.json` | Not slots. Owned by their own surfaces; `collections.json`'s three `libraryName` values rekeyed in the move commit. |
| `Templates\` (12) | Moves into `_Core\Templates` (setting + fence + default in one commit — Gate 1). MIG-TPL §1's line is hidden-vs-visible, and `_Core\Templates` is visible and user-owned, so this honours it. The 6 `TS_test_*` go to the debris proposal. |
| `Five Acts\` (1 host note) | Moves into `_Core\Five Acts` — one constant (`FIVE_ACTS_DIR`, `system_notes.rs:36`) moves creator and lister together. The top-level section is unchanged; `hideFiveActsFolder` is kept and repointed; the `:7450` label/colour source repoints off `universeNotesStats`. |
| `.trash\` (55 indexed rows) | Stays at the root; resolver repointed same-commit (`store.ts:3846`/`:3853`). The 55 rows: ruling R3. |
| The ~35 debris notes + `MIG-005 Test\` + `Replication Bug\` | Leave the root; never enter the Core. Evidence-first proposal, grouping rule visible, nothing preselected; destination or delete is your call (R2). The app never deletes and never auto-files. |
| The ~8-10 real notes (`Eisa ALSHAMSI`, `المعرفة العربية`, `تجربتي`, `عيسى المطور`, …) | Shown with full evidence (`cid_cn` NULLs said on the row — 17 of 126 are; incoming links; **earned** link data; Collection/Base/review/workspace membership). Three destinations: a Library · delete · **"make this my Universe's charter"** (at most one). We don't know which they are, and the app doesn't either — it asks. |
| `Daily Notes\` (4) · `Imported\` (2) · `EisaTest.canvas` | To a Library. A journal and a staging area are content, not identity; `openDailyNote` (`:4758`) and `ImporterModal.svelte:21` repoint. The canvas moves by the **disk** walker (no `note_meta` row). |
| `Creating new library\` (registered, nested, 0 notes) | Grandfathered, flagged, refused going forward by the containment rule in `add_library`. |
| `Testing opened note.md` (P6 — on disk, no index row) | Proof the mover reads the disk, not the index. |
| `Constellation SV Test.db` (939 MB, Apr 23) + unbounded logs | Not the head's content. **Your ruling** (§5 R8) — we do not delete a user's file as a side effect. |

Arithmetic closes: 126 root-owned rows = 55 `.trash` + 71 that follow their files (44 indexed loose + 12 + 4 + 4 + 4 + 2 + 1).

---

## 4. WHAT WE ARE DELETING

**Frontend — `+layout.svelte`:** the whole `:7545-7580` block (raw `.name` label `:7553`, the only count badge `:7554-7556`, the `data-style-target="library"` borrow `:7547`, the no-icon comment `:7551-7552`); `:1461`'s `universeNotesStats` accessor (the `:1460` filter becomes `kind === 'library'`); the `iconKind='root'` branch `:6371-6382`; the `seen` dedupe `:6370-6373` stops being load-bearing (containment refused ⇒ the harness proves it with the dedupe removed); the root context-menu branch `:6037-6040`; six `$libraries[0]`/`universeNotesStats` write targets `:4269`, `:4395`, `:4458`, `:4758`, `:5466`, `:7981`.

**Components:** `LibraryIcon.svelte:20` — `kind='root'` deleted from the union, **no replacement kind added** (the head renders no icon anywhere); the empty branch `:49` goes; `LibraryPicker.svelte:40-45`, `DashboardView.svelte:181`, `MoveDialog.svelte:92` root blocks; `LibraryManager.svelte:77-93` — the head leaves the list and the unguarded Remove hazard dissolves structurally; `ImporterModal.svelte:21`; `OrgChart.svelte:1179`'s inline book glyph repointed at `LibraryIcon` (the drift `LibraryIcon` was created to prevent).

**Rust:** `universe.rs:403-414` insert + the healers `:803-807`, `:835-838`, `:1097-1103` repointed at `_Core`; the name re-pins `:972-975`, `:990-993` deleted; `:839-848` refuses-and-surfaces; `libraries.rs:6-12` — `is_universe_notes` replaced by `kind` with read-side promotion of the legacy flag (28 Rust + 12-literal frontend sites; three independent `.find()` accessors — `store.ts:207`, `+layout.svelte:1461`, `DashboardView.svelte:94`); `libraries.rs:436` `get_library_mode` (zero callers, prefix bug) deleted; `attach.rs:135` swapped to `universe.rs:528`.

**i18n / CSS / docs:** the `universeNotes` namespace retired across all 15 locales (zero call sites — verified); the "vault" strings in `universe.manager.*` corrected in 10 locales; `.v-chev` gains its RTL flip; `.section-label`'s Latin-only uppercase+tracking replaced for captions; `RECENT_CAPTURES_CONTENT` localized; `docs/User Manual.md:292-298` + all 14 translations ("named after the universe") rewritten in the same commit (SO#2); spec citations to `src/lib/i18n/locales/` corrected to `src/lib/i18n/`.

---

## 5. WHAT WE COULD NOT SETTLE — YOUR RULINGS, AS CHOICES

**R1 — Does the head hold your own writing?** (a) *Recommended:* yes — one Charter plus the Templates/Five Acts molds, closed kinds, nothing else, `_Core` the one visible directory at the root (a narrow, argued departure from 07-25's letter — the app's own boot code regrows `Templates\`/`Five Acts\` at the root today, so the literal end-state requires repointing those writers anyway, and `_Core` is where the repoint lands). (b) Governs only — the root is literally clean, but the head has no cognition in it and "an extension of one's reasoning" goes unfulfilled.

**R2 — The 45 loose files and test folders.** Per group or per file: destination Library · delete · (for at most one) "make this my Universe's charter." **Sequencing consequence, stated plainly:** the path amendment can land only after your R2 rulings empty the root of unowned content — with the entry's path at `_Core`, a loose root file would have no owner. The proposal screen is where you rule; deferring R2 defers the scope amendment itself. (If you want relief before ruling, Design 4's rename-and-separate interim exists — but it ships a residue library and re-churns the sidebar twice; we recommend ruling once, moving once.)

**R3 — The 55 `.trash` index rows.** (a) *Recommended:* purge, index-only, no file touched — they are polluting `sky_nodes` (54) and `review_schedule` (60): the Reviewer is scheduling deleted notes today. Caveat owed: nobody has established how they got in; the Plan reads the delete/watcher path end to end first, or the purge is deferred and we say so.

**R4 — Stage 0 first (P1 path-cascade 5-of-11, P3 the 1,577 silent "relocate deferred" lines, P4 serde flatten)?** (a) *Recommended:* yes — moving 13+ files before P1 orphans 1,312 `note_state_history` rows, 126 `note_body`, 103 `note_summaries`, 44 `sight_v3_layout` for exactly these notes.

**R5 — MIG-104 (durable earned-link data on disk) first?** (a) *Recommended:* yes — every link-data risk becomes recoverable instead of permanent.

**R7 — `sight_v3` cache.** The head's path changes, so `library_set_hash` (paths-only — verified) orphans all 7,636 cached layout rows. (a) *Recommended:* purge the three tables in the same transaction, accept one recompute. Decided, not discovered.

**R8 *(new)* — the 939 MB `Constellation SV Test.db` orphan** (+ the unbounded `boot-perf.history.jsonl` / `diagnostics.log`). (a) Delete the orphan and cap the logs (a PJ, not a MIG-105 side effect). (b) Leave as is.

**The step-1 blocker, unchanged:** `C:\Users\ealsh\AppData\Roaming\world.uconstellation.app\universes.json` lists only `كون عيسى` as its single entry and `active_id`, while ECK is demonstrably in use today. The Plan's first task is to instrument `save_registry`'s write path and read the answer — the migration cannot name its target Universe on a registry known to disagree with reality.

*(The prior R6 — the mark — is no longer a ruling request; the team's answer is §6. Overrule us if you want a face on it.)*

---

## 6. ON THE "NO ICON" STEER — FINAL RECOMMENDATION

**Keep it. Do not revisit it. This reverses our prior recommendation, and the reversal is evidence-driven, not taste.**

The prior verdict asked you to revisit the steer and accept a new nucleus glyph. That recommendation rested on the spec's two premises, and on re-verification **both are false**: the "every other row is marked" consistency argument misread the shipped markup (the Bookmarks section header at `:7415` is plain unmarked text — a shipped, Boss-accepted, unmarked system section already exists), and "the Universe's existing mark" does not exist as shipped identity vocabulary (the `◇` lives in a Style Setter preview and *already means Template Studio* in the command palette via `'◇'` at `:2361`; the footer chevron-pair's one shipped meaning is "open the switcher"). Adopting any candidate would be inventing product vocabulary while claiming reuse, or stacking a third meaning on an already-ambiguous glyph.

And the positive case for absence is stronger than the case for any mark: in this sidebar a glyph is the grammar of **"a place you can enter"** (every library row emits the building, every cUniverse row the planet, nothing else in the tree block is marked). The head is not a place — it is the thing that rules the places — so its unmarked state is **contrastive**: it reads as "not one of those," which is precisely the statement MIG-105 exists to make. Under the locked design Five Acts becomes the head's child, so the sibling-consistency argument dissolves structurally. The head's marks are all structural and all already in the design: first position, its own zone, its own kind, a role label in your own words, absence from every library surface, the `_` prefix on disk. Eleven surveyed systems; zero decorative head-marks. Your steer was right — and it is now right for a better reason than the one it was given.

The one code change the ruling still requires: `kind='root'` is deleted from `LibraryIcon` — a kind that renders nothing looks like a working option and produces an invisible element — with **no replacement kind**, because the head renders no icon anywhere.

---

**Where the team disagreed, ruled in one line each.** Design 3 was right about the two-zone shape, the in-UI seam sentences, the closed-kinds rule, the charter-as-first-line answer to Dendron, the third migration destination, the Move-picker Templates door, and the `.section-label` cursive defect — and wrong to bury Bookmarks, wrong to put Bases/Collections in a "Rules" chamber its own test refutes, and wrong to pay nine seams for a token when keeping the entry costs three. Design 4 was right about the no-mark grammar, the dead divider, the vault strings, the `.v-chev` flip, the colour and sight-cache mechanics, and "a scope defined by subtraction is a residue" — and wrong to shrink the head to a settings shortcut that opens the switcher, and wrong to leave your own `custom_stages` unsurfaced after verifying it was orphaned. Design 1 was right about the federation swap (verified), `custom_stages` as the head's first reader, and the ◇ collision (verified against a judge's refutation) — and wrong to move the head off the surface you were looking at. Design 2 was right about NotePane as where a head is thought in, the Bookmarks exception, and the 19-site census — and wrong to invent a glyph and open by asking you to reverse a ruling. The Art Director's own spec was wrong on both mark premises, wrong about the Style Setter `tree` category, self-contradictory on FileTree, overstated the federation risk, and silently deleted the Templates Move destination. The buildability judge was wrong about Design 1's ◇ claim. The prior verdict — ours — was wrong on the mark, the name, the Maps folder, and the wholesale Move refusal, and it said so above, first.
