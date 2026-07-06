# PJ-069 — The Whole-Entity Deduplication Pass — Concept Paper

**Date:** 2026-07-06 · **Status:** **Concept + shape RATIFIED (Boss, 2026-07-06).** Horse framing = **"one home per capability."** Priority = **answer-duplication first.** Scope = **the 7 filed clusters + the 9 newly-found cross-cluster families.** First step = **the dead-code cull.** Awaiting approval of the Step 0 plan (§9a) before any code.

Concept-paper-first per the bring-up method. This paper states the horse, presents a freshly re-verified map (the 2026-07-05 counts are stale — re-audited today), names the method (already proven twice on `main`), and surfaces the rulings needed before any `/migration`.

**Basis:** SO#8 cross-check + adversarial re-audit — workflow `wf_2ae0f8c0-d59` (18 agents: 3 context readers, 7 cluster finders, 7 adversarial verifiers, 1 completeness critic; ~2.3 M tokens). Every count below was verified against the live tree this session, not carried from the map.

---

## 0. Why this paper exists (the horse must precede the carriage)

The Boss filed PJ-069 with a law, not a feature request:

> *"We have to avoid duplication among all Constellation's core plugins/functions. They all should complement each other as a whole entity."* — Boss, 2026-07-05

A deduplication pass is a *function* (the carriage). Before touching it we must state its *concept* (the horse) — the one thing it is for. If the horse is only "delete copied code," the pass is a janitorial chore and will be sequenced last forever. It is not a chore. It is load-bearing to what Constellation *is*.

---

## 1. The horse *(ratified — "one home per capability")*

> **Every capability in Constellation has one home — one owner that computes or renders it — and every other surface mounts that home instead of re-implementing it. The whole entity is one system of single-owned capabilities, not a federation of copies.**

Why "one home" and not "tidy up copies": copies do not stay identical. They drift. And in a *knowledge-formulation* tool, a drifted copy is not just a maintenance cost — it becomes **a different answer to the same question**, and the user sees the contradiction. The re-audit found this is not hypothetical. Today, in shipped code:

- **"Is this note an orphan?"** has **five mutually-inconsistent live answers.** `inbound==0 && words>20` (Reviewer, 360°, Tension) · `total-degree==0` (Sky View, CNS) · `in==0 AND out==0` (Search filter + the new Collections "Unlinked" chip) · `degree<2` (Cataloger) · `not-in-sky-set` (editor gesture gate). The same note is an orphan on one surface and not on another.
- **"How connected is this hub?"** The CNS *Hubs* register — the surface whose own concept paper says *"this is their one home"* — reads a `GROUP BY target_name` count (`search.rs:7258`) that is **not** the alias-aware, DISTINCT-source, archive-excluding `note_meta.incoming_count` that the Backlinks badge shows. **The owner surface can display a different hub number than the badge on the same note.** Six live in-degree substrates compute this "same" number.
- **"What are my tags, and how many?"** The main Dashboard reads the write-time `tag_counts` summary; the Second Screen re-walks the filesystem with a *verbatim clone* of the Dashboard's code and a different tag definition (frontmatter-only, misses inline `#hashtags`). Two windows of one app, two tag readings.

Constellation calls its search *"a diagnostic instrument for intellectual life."* An instrument whose dials disagree is broken — not untidy, **broken**. That is the real cost of a missing single home: not maintenance hours, but **a fractured answer** — the user asks the entity one question and gets contradictory replies depending on which organ they open. Giving each capability one home is what keeps the entity's answers coherent.

The carriage (what we will actually build): per-capability ownership — **one home + sanctioned mounts** — landed via `/migration`, dead copies deleted, and the seeds already shipped (`NoteRow`/`NoteList`, `searchFold`, `switcherRank`, `ConfidencePicker`, `RelatedCandidates`) adopted so the pass *reduces* the entity's surface count instead of adding to it.

This sits directly under the top-principal **Concept Before Function** and is the whole-entity expression of **Knowledge Formulation, not Management**.

---

## 2. Two kinds of duplication (the frame that sets priority)

The audit makes one distinction that should govern sequencing. Not all duplication is equal:

**A. Answer-duplication (semantic — the dangerous class).** The same cognitive question is *computed differently* in different places, so the answers diverge and the user sees the contradiction. This is where "the instrument gives one reading" is already violated. Clusters: **orphan/fragile · hubs · tags** (definition drift). Fixing these is not cleanup — it is *correcting wrong readings the user can see today*.

**B. Form-duplication (mechanical — the latent class).** The same *rendering or formatting* is hand-rolled repeatedly. The answer is the same; the code is copied. No contradiction yet — but every copy is where the *next* answer-contradiction hatches (e.g. CCSView hard-codes the confidence colours, so the moment the user recolours Confidence in the Style Setter, CCS silently disagrees). Clusters: **note-lists · confidence (already fixed) · recents · folders** (mostly), plus the cross-cutting families in §7.

**Proposed priority: answer-duplication first** (it is producing wrong readings now), then form-duplication (it prevents future wrong readings). Confidence is the proof the method works — it is already at zero (§3, §4).

---

## 3. The re-verified map — the seven clusters, today

The 2026-07-05 map (`wf_1d470cb8-9e8`) counted the pre-existing debt as *tags×6, folders×4, recents×3, orphan/fragile×5, hubs 3-way, note-lists×9, confidence-menu×2*. It was drawn the **same day** the Navigator was deleted and predates MIG-092/093. Re-audited counts:

| Cluster | Kind | Map | **Today** | The single open ruling |
|---|---|---|---|---|
| **orphan / fragile** | **A — answer** | 5 | **9 surfaces / 12 definition sites / 5 definitions** | **Boss DEFINITION ruling required** — one concept + one predicate, or several *named* concepts ("orphan" vs "unlinked")? |
| **hubs** | **A — answer** | 3 | **6 in-degree substrates** (owner surface disagrees with badge) | Ratify "the Backlinks number **is** the hub number" (repoint to `note_meta.incoming_count`)? |
| **tags** | **A — answer** | 6 | **6 live** (recomposed — different six) + 3 dead-in-tree; 5 divergent tag definitions | Fate of the hierarchical tag-**tree** paradigm (orphaned `TagsPanel.svelte`): **delete** or **revive** as the shared `TagTree`? |
| **note-lists** | **B — form** | 9 | **26 live hand-rolled** (primitive shipped, **1** adopter) | The **exemption boundary** — do command-palette rows / table rows / tree leaves / card grids count, or are they sanctioned different paradigms? |
| **folders** | **B — form** | 4 | **5 live** + federation grouping hand-rolled at **7 sites** | OrgChart's data source — repoint to the shared tree, or ratify `map.rs` as its sanctioned command? |
| **recents** | **B — form** | 3 | **2 hand-rolls** (3 sanctioned mounts ledgered separately) | None — mechanical. |
| **confidence** | **B — form** | 2 | **0** — already resolved (the exemplar) | None — done. Reference it. |

**Three corrections the paper must carry (verified against live source this session):**

1. **"Sight is disabled" is over-broad.** CNS v2 (`ConstellationSight2` + `SightPanel`) is **LIVE core** — `SIGHT_V2_ENABLED = true` (`engine.ts:131`), `constellationSight` defaults true (`store.ts:4299`), mounted at `+layout.svelte:6306`. **Disabled-in-tree = Sight v3/v4/v6/v7 engines + the Constellation *Map* frontend (`ConstellationMap.svelte`) only.** The blanket memory line needs this nuance — several surfaces we would otherwise dismiss as "dormant" are live.
2. **`map.rs`'s hierarchy builder is LIVE**, via the default-on OrgChart. The only reachable OrgChart runs `constellation_map_universe` (`OrgChart.svelte:449/:802`); its `read_library_tree` path is the *embedded* branch, and the `'skyview'` sidebar mode that would mount it is **never assigned** — dead code. So OrgChart's hand-rolled frontend federation is dead, and `map.rs`'s second hierarchy builder + its own inbound-count map are live duplication.
3. **Confidence is already at zero.** The shared `ConfidencePicker.svelte` shipped **2026-06-29** (`fa98bf6b`, MIG-077 §F) — *six days before the map was drawn*. Both named "copies" (Backlinks, Outgoing) are sanctioned mounts with ~8 lines of positioning glue each. The map's `×2` was stale at draw time.

---

## 4. The method — one owner + sanctioned mounts (already proven twice)

This is not a new pattern to invent. The entity already contains three worked examples:

- **MIG-092 (Bookmarks + Workbench → Collections).** The adversarial dedup pass caught a *Bookmarks* feature the whole-entity map itself had **missed**. The Boss ruled **UNIFY** → Collections became the one hand-picked-set mechanism; Bookmarks = the pinned "Starred" collection with **two sanctioned mounts** (Search-Hub tab + sidebar). This *reduced* the entity's surface count. **This is the template.**
- **MIG-077 §F (`ConfidencePicker`).** Two byte-identical inline confidence menus → one self-contained shared component; hosts keep only positioning glue. The confidence cluster is *already* the finished state.
- **`RelatedCandidates` — "one list, N mounts"** (the sanctioned-reuse *model*, MIG-086): one "Connect to:" surface, **6 mounts across 5 hosts** (`+layout`, GraphMindView, Inspector360 ×2, ReviewerView, TensionPanel), each passing only host context. *(Note: the project prose "one list, five mounts" is now numerically stale — cite 6/5.)*

The rule the method encodes: a shared component is not enough — the pill saga taught that a shared component still drifts per host, so the owner must be **self-contained** (owns its font/dir/size/colour/IPC), and mounts pass only context. `NoteRow` (self-contained per-title RTL), `LinkTypePill` (self-contained badge), and `ConfidencePicker` already embody this.

---

## 5. The seeds already in hand (build on these, never add a tenth copy)

| Seed | File | Owns | Adoption today |
|---|---|---|---|
| `NoteRow` / `NoteList` | `src/lib/components/NoteRow.svelte`, `NoteList.svelte` | the one note row (per-title RTL, row-height contract over `VirtualList`) | **1 of 26** (CollectionsPanel only) |
| `searchFold` | `src/lib/searchFold.ts` | the one frontend Arabic/Latin fold (Rust-parity documented) | QuickSwitcher, switcherRank, IndexPanel |
| `switcherRank` | `src/lib/switcherRank.ts` | banded title ranking | QuickSwitcher only (autocompletes still hand-roll) |
| `ConfidencePicker` | `src/lib/components/ConfidencePicker.svelte` | the confidence menu + IPC | Backlinks, Outgoing (complete) |
| `RelatedCandidates` | `src/lib/components/RelatedCandidates.svelte` | the "Connect to:" list (the remedy for orphans) | 6 mounts / 5 hosts (complete) |
| `ContextMenu` + `contextMenuBuilder` | `src/lib/components/ContextMenu.svelte` | the shared right-click menu | most surfaces (one rogue: `EditorContextMenu`) |
| `recentNotes` | `src/lib/libraries/recentNotes.ts` | the recents MRU (self-declared "one source of truth") | Dashboard, QuickSwitcher, +layout write hook |
| `tag_counts` (Rust) | `src-tauri/src/tag_counts.rs` | write-time tag counts (the Rule-8 answer) | Dashboard; SS + Rust fs-walks bypass it |
| `note_meta.incoming_count` (Rust) | write-time, alias-aware, DISTINCT-source | the canonical in-degree | Reviewer, Backlinks, Collections; 6 substrates bypass it |

---

## 6. Dead code — delete, do not consolidate (confirmed unreachable this session)

These are not consolidation targets; they are removals that shrink the audit surface for every cluster:

- `TagsPanel.svelte` — the only hierarchical tag-tree builder left, but **zero imports** across `src/`. (Its fate is the tags-cluster ruling: delete, or revive as the shared `TagTree`.)
- `NoteGrid.svelte` — imported at `+layout.svelte:95`, **never mounted** (`<NoteGrid` appears nowhere).
- `store.ts::timeAgo` — sole consumer is the orphaned `/libraries` route.
- `/libraries` route + `NavBar.svelte` — reachable only via each other; `NavBar` imported by nothing. Legacy that escaped the MIG-091 §retire sweep.
- `lenses.rs::apply_tag_lens` — registered IPC, **zero frontend callers** (the 2026-05-09 "delete `lenses.rs`" decision is still unexecuted). Compounding: `SettingsModal` still offers UI to *create* tag-hierarchy lenses that can never render.
- `bases.rs::scan_by_tag` / `query_base` — unregistered, retired by MIG-065 §I.
- `map.rs::constellation_map_data` — registered zombie, zero frontend invokers.
- `boot_bundle.rs:93-95` — still fetches a `bookmarks` value nothing reads (the logged trivial follow-up).

---

## 7. Cross-cluster families the 2026-07-05 map never listed — **IN SCOPE (Boss-ruled 2026-07-06)**

The re-audit surfaced nine more duplication families, **each evidenced with file:line**, that no cluster in the original PJ-069 owns. The Boss ruled these **into this same pass** (scope = 7 + 9). Each becomes its own owner ruling under the same method:

1. **Arabic fold / normalization (cross-IPC)** — `searchFold.ts` is the declared owner, yet `store.ts:2233` (`normalizeArabicLight`), `NotePane.svelte:561`, `BaseTab.svelte:430` duplicate it frontend-side, and **six overlapping folds** live on the Rust side. JS↔Rust parity is a "keep in sync" comment, not test-pinned. *(Answer-class — folds that diverge miss different notes.)*
2. **Relative-time formatting** — `fmtTraversed` is **byte-identical** in Backlinks and Outgoing, **and hard-codes English "today/yesterday/Nd ago"** — an i18n-rule violation. Plus `searchHistory.relativeTime`, `store.timeAgo` (dead), `TasksPanel` day-math.
3. **Library-colour dot chip** — **seven** hand-rolled `.lib-dot` variants, three different colour-resolution paths, `#7c3aed` fallback repeated ≥5×.
4. **cUniverse federation grouping** — the `getChildUniverses` + `read_child_universe_libraries` grouping hand-rolled at **7 sites** (5 live); the boot-perf doc already flags the repeated reads. Directly governed by the "It is ONE universe" ruling.
5. **Name-autocomplete matching/ranking** — wikilink/heading/command completions (`completions.ts`, `CodeMirrorEditor`, `GraphMindView`, `CommandPalette`) each hand-roll title matching; none use the shipped `switcherRank`/`searchFold` — the exact problem MIG-093 already solved.
6. **Context-menu renderer duo** — `ContextMenu.svelte` (canonical) vs `EditorContextMenu.svelte` (own positioning/click-outside/markup).
7. **Search-history / generic MRU** — two parallel search-history stores (localStorage `searchHistory.ts` vs SQLite Index-filter history) + `EmojiIconPicker`'s own MRU; same pattern-family as `recentNotes`.
8. **Link-type label resolution** — `LinkTypePill` owns the `linkTypes.${id}` i18n lookup, yet three of its own consumers re-implement it beside the mount.
9. **Note-open dispatch duality** — direct `openNoteTab` vs the `constellation:open-note` CustomEvent bus. *Likely a sanctioned module-boundary event bus* — but the owner + "which mechanism a new surface uses" should be declared so it doesn't drift.

---

## 8. The Boss decisions

**Ratified 2026-07-06:** horse = "one home per capability" (§1) · priority = answer-duplication first (§2) · scope = 7 clusters + 9 families (§3, §7) · first step = the dead-code cull (§9a).

**Still to be ruled — inside each cluster's own `/migration` Architect doc** (the natural place; none blocks Step 0):
- **orphan/fragile** — one concept + one predicate, or several *named* concepts ("orphan" vs "unlinked")? *(the one genuinely open definition question — gates Step 1)*
- **tags** — `TagsPanel.svelte` (orphaned tag-tree): delete, or revive as the shared `TagTree`?
- **hubs** — ratify "the Backlinks number **is** the hub number" (repoint the 6 substrates to `note_meta.incoming_count`)?
- **note-lists** — the exemption boundary (palette rows / table rows / tree leaves / card grids in or out)?
- **folders** — OrgChart's data source: repoint to the shared tree, or ratify `map.rs`?

**Housekeeping (do now, with ratification):** correct the PJ-069 Pending Jobs entry → **v1.17** (four stale parts: dead surfaces in the counts; the obsolete "coordinate with MIG-090" clause; the overstated confidence-menu×2; the events it predates) with the recomposed counts from §3.

---

## 9. Proposed sequencing (a sketch — pending ratification, not a commitment)

Each **answer-duplication** cluster is a genuine subsystem-crossing change (Rust ↔ Svelte, definition ↔ every consumer) → each goes through `/migration` (Architect → Plan → Build → Audit). The **form-duplication** clusters can batch behind the shipped seeds.

- **Step 0 — Dead-code cull** (§6): one reviewed cleanup commit; no behaviour change; shrinks every subsequent audit.
- **Step 1 — Orphan/fragile** (answer, highest contradiction): needs the Boss definition ruling first, then one shared Rust predicate helper over `note_meta` write-time columns, consumed by Reviewer/360/Tension/Search/Collections. `/migration`.
- **Step 2 — Hubs** (answer): ratify "Backlinks number = hub number"; repoint the six substrates to `note_meta.incoming_count`; the CNS register and the badge finally agree. `/migration`.
- **Step 3 — Tags** (answer): rule the tag-tree fate; Dashboard = the browse owner on `tag_counts`; Second Screen mounts DashboardView (display-not-domain); Rust fs-walks retire to the index. `/migration`.
- **Step 4 — Note-lists** (form, biggest count): rule the exemption boundary; adopt `NoteRow`/`NoteList` across the live surfaces, **starting with the same-data duplicate** (the sidebar Starred list vs CollectionsPanel — same data, two renderings). Batched.
- **Step 5 — Folders + the §7 form families** (recents, relative-time, library-dot, federation grouping): batched behind their owners.
- **Confidence** — already done; the reference exemplar. Residue (CCSView hard-coded hexes; triplicated i18n labels) folds into Step 4/the Sight cleanup.

Every step obeys the hard constraint: **no regression to boot time, typing latency, or IPC responsiveness**, measured on the 7,600+-note universe before commit. The consolidations are Rule-8 wins (fewer read-time re-walks), so the pass should *improve* perf, not cost it.

---

## 10. Process

Ratification of this paper → per-cluster Architect deltas (form-specific) → Plans → builds, each landable as one commit with a verification clause, `/simplify` on each diff, the audit trio at the end of each `/migration`. The old surfaces keep running until each validated swap. Standing Orders (session log, MoCh, Orientation v-bump on ship, help/manual) apply throughout.

**This session's deliverable ends at this paper.** Awaiting the §8 rulings before any code.
