# MIG-086 — Phase 2 BUILD PLAN
## Suggested-Relatedness → One-Click Typed Link, Everywhere

**Doc:** `docs/MIG-086-Plan.md` (this file — net-new)
**Architect:** `docs/MIG-086-Architect-Reviewer-Link-Suggestions.md` (approved)
**Branch / root:** `main` @ `E:\مشاريع كلاود\Constellation`

---

## 1. Concept (the horse) + Boss decisions baked in

**Concept (one line):** *When a note is link-poor (orphan, fragile) — or whenever the user is staring at a note's link surface — Constellation answers "what could this connect to?" by surfacing the BM25 most-related existing notes, and turns the answer into a typed Living Link in one click.* The suggestion is a diagnosis; the typed link is the act. Suggest = read-time-cheap, query-time-on-demand (CLAUDE.md Rule 8 — never per-keystroke).

**Boss decisions locked (2026-06-23), baked into every step below:**
1. **Always ask the type.** Clicking **Link** on any suggested note ALWAYS opens the 8-type picker. Orphan pre-selects `associative`; fragile pre-selects `derives-from`. (No standalone picker exists — findings §"Link-Type Picker" — so §C builds a small one on the existing registry + `LinkTypePill`.)
2. **Everywhere now.** Build one reusable `<RelatedCandidates>` and wire it into every core link-dealing surface (Reviewer detail, NotePane right-sidebar, Backlinks/Outgoing branch, 360 Inspector) — NOT Reviewer-only.
3. **5 candidates.** Fragile reuses the same list; its picker pre-sets `derives-from` and shows a "shore it up" heading. Remove the inert Reviewer **Connect** button (`ReviewerView.svelte:397`).

---

## 1b. Concept-derived invariants (Concept-Before-Function — Boss-gated 2026-06-23)

The concept was validated against the Knowledge-Formulation principles: the suggester does the
*lookup*, never the *formulation*; the user keeps the two cognitive acts — **judging** that a candidate
is really related, and **deciding what the relationship means** (the type). These invariants are what
keep the feature *formulation*, not link-spraying *management*. They are not optional polish; a step
that violates one violates the concept.

- **C-1 · NO bulk accept — ever.** There is exactly one **Link** button per candidate, and it ALWAYS
  routes through the type picker. No "Link all", no "accept all suggestions", no multi-select. Each
  connection is one deliberate, typed act. *(This is the single most load-bearing constraint — a
  bulk-accept would re-introduce the exact quantity-over-quality trap the type-picker exists to prevent.
  No future step may add one.)*
- **C-2 · Always show the *why*.** The shared-term chips + snippet are mandatory, not decorative — a
  connection the user can't see the basis for isn't one they can judge. If §A can't return shared
  terms for a candidate, that candidate is not shown.
- **C-3 · Invitational framing, not assertion.** Copy reads as a *suggestion to evaluate* ("Suggested
  connections" / "could this connect to…"), never "these ARE related, confirm." The user is the one
  who decides the relationship is real.
- **C-4 · Suggestion-born links enter as `hypothesis` confidence** (the lowest rung of the Living-Link
  confidence ladder hypothesis → evidence → established → contested) — a proposed relationship to be
  earned, not asserted as established. **§A/§C check:** confirm `index_note`'s body-derived `note_links`
  default confidence is `hypothesis`; if it isn't, the concept requires the connect path to stamp it.
- **C-5 · Only *unconnected* relatives, and an honest empty state.** Already-linked targets are excluded
  (§A anti-join); when nothing clears the relatedness bar, say so ("No strong matches yet") — never
  fabricate a relative (BASIC RULE). The feature heals a *diagnosed* gap (orphan/fragile / SPOF), it
  does not pad already-rich notes.

---

## 2. Surface list — concrete hosts for `<RelatedCandidates>`

**FINAL set (concept-validated against all ~21 left-dock surfaces, Boss-ruled 2026-06-23 — workflow
`wf_ada99394`).** A surface hosts the feature ONLY if its cognitive job is *the connections of one note
in hand*; browse/visualize/aggregate/other-unit surfaces were EXCLUDED (table in §2b).

| # | Surface | File : slot point | Note-context source | Gate |
|---|---------|-------------------|---------------------|------|
| 1 | **Reviewer detail** (orphan + fragile lenses) | `ReviewerView.svelte` — after `.rv-d-rx` card, before `.rv-d-prio-box` (between **342 ↔ 344**); **remove inert Connect button line 397** | `n.note_path` + `libraryPath` prop (line 35) | `isOrphan(n)` (215) `‖` `n.reason==='fragile'` |
| 2 | **NotePane right-sidebar — Backlinks tab** | `+layout.svelte` `'backlinks'` branch — new `.rs-section` **after line 7483** (below Outgoing) | `sidebarTab.path` / `sidebarTab.libraryPath` | always (note-scoped); fire-once on note change |
| 3 | **360° Inspector — BOTH mounts** (the Boss's example; bullseye concept match) | `Inspector360.svelte` — footer block between the matrix (`:445`) and HUD (`:456`); **plumb `libraryPath` into props** (absent today) at BOTH mount sites — **full-page** (`+layout.svelte:6576`, `compact={false}`) AND **right-rail** (`:7554`, `compact={true}`) | `note_path`/`note_name` + new `libraryPath` | `data.is_orphan ‖ data.single_point_of_failure ‖ data.missing_link_types` |
| 4 | **Note "Health" tab (TensionPanel)** — the strongest fit; already lists THIS note's orphan/contradiction/SPOF/gap as clickable rows | `TensionPanel.svelte` — beside the orphan rows (`:162–183`); mounted at `+layout.svelte:7527` (the note-scoped Health tab) | the open note's path/libraryPath | the note is flagged orphan / SPOF / in-a-gap |
| 5 | **Sky View — per-node right-click MENU ITEM only** (NOT the canvas — Form-Aligns-To-Purpose: the canvas visualizes, the menu acts) | `GraphMindView.svelte` — new "Suggest connections…" item in the per-node context menu (`:1153`, per-node via `graphEngine.ts:1791`, carries `{id,name,path,libraryName}`) | per-node (right-clicked node) | always available on a node; opens the same flow in a popover/modal |

**Reuse, not host (no new render):**
- `LinkTypePill.svelte` — the type chips in §C's picker and any displayed type. Self-contained (own font/dir/colour) per [feedback_self_contained_components].
- `linkTypeRegistry.ts` — `getLinkTypes()` / `topLevelLinkTypes()` / `linkTypesStore` feed the picker; **prepend `associative` manually** (excluded from `SEED_IDS`).

### 2b. Left-dock plugins EXCLUDED (concept-validated — stated so the audit knows each was considered)
- **Knowledge Health Dashboard** — browse-only whole-universe *vital-signs* read instrument; rows are inert (no per-note detail/handoff). It doesn't even hold the orphan/SPOF list — that's TensionPanel (surface #4). Inventing a per-note drill-down here would bolt a different surface onto it.
- **Sky View canvas** — visualization; click already means open-and-leave. Only its per-node *menu* hosts (surface #5).
- **Search Hub** (finder — "where is X", not one note's relatives), **Index** (unit = the term/lemma, not the note), **Organization Chart** (folder containment, not links), **Calendar** (dates), **Tasks** (to-dos), **Cataloger/CECE** (sources/citations), **CCS** (whole-system flow visualization), **Constellation Sight** (whole-universe lens; → Wings).
- **DISABLED:** Constellation Map (MIG-038 → Wings), Sight v3/v4/v6/v7 (dev flags off).

**Other excluded (component-level):**
- **NotePane CM6 editor** (`NotePane.svelte`) — Rule 1/Rule 3 forbid `invoke()` on the keystroke hot path. The NotePane affordance lives in the right-sidebar (surface #2), not in the editor.
- **`LinkTypesEditor.svelte`** — Settings *vocabulary* config, not a per-note link creator. Reuse its data source only.
- **The confidence popover** (BacklinksPanel 336–353 / OutgoingLinksPanel 253–270) — edits *confidence*, not type; it is the *interaction template* §C copies, not a host.

> **Note on surface #2 scope:** the Backlinks tab already stacks BacklinksPanel + OutgoingLinksPanel (`+layout.svelte:7454–7483`). Adding the new `.rs-section` here gives BOTH the Backlinks and Outgoing diagnosis→action loop one shared "Suggested connections" block — no separate Outgoing host needed. A dedicated "Related" tab is **out of scope for this MIG** (would touch the tab strip ~7335, `NOTE_SCOPED_TABS` line 395, `validTabs` restore 7697, placement `$effect` ~1746 — defer unless Boss asks).

---

## 3. Phased steps (each = one commit, each with a verification clause)

### §A — Backend: `suggest_related_notes` (BM25 "More Like This")
**Touches:** `src-tauri/src/libraries.rs` (new command next to `read_cooccurring_terms`), `src-tauri/src/lib.rs:460ish` (register).
**Does NOT touch:** any write path, `note_links`, `index_note`. Pure read.

- New command `suggest_related_notes(library_path, note_path, limit) -> Vec<RelatedCandidate>`.
- New struct `RelatedCandidate { note_path, note_name, score, shared_terms: Vec<String>, snippet }` (serde camelCase).
- Implement the architect §2 BM25 MLT SQL over `notes_fts`: take the source note's top-weighted FTS terms, build a disjunctive `MATCH`, rank by `bm25(notes_fts)`, **exclude self** (`note_path != ?source`), **exclude already-linked** targets (anti-join `note_links WHERE source_path = ?source` — read-only), cap at `limit`. Return matched shared terms + a short snippet.
- **Stopword/empty guard:** if the source yields no content terms after stopword stripping → return `[]` (never a full-table scan).
- Register in `lib.rs` invoke handler.

**Verify (`cargo test`, fixture Universe):**
1. Planted relative ranks **#1** for a source sharing a rare term.
2. **Self excluded** (source never appears in its own results).
3. **Already-linked excluded** (a target already in `note_links` for the source is absent).
4. **All-stopword / empty source → `[]`** (no panic, no scan).
5. `limit` honored (≤5 rows).

---

### §B — Shared `<RelatedCandidates>` component (read-only) + store wrapper
**Touches:** `src/lib/components/RelatedCandidates.svelte` (net-new), `src/lib/libraries/store.ts` (`suggestRelatedNotes` wrapper next to `readCooccurringTerms` @ **2958**), ONE host for the Boss test (Reviewer detail, surface #1).
**Does NOT touch:** any write path. Display + fetch only.

- `suggestRelatedNotes(libraryPath, notePath, limit=5)` → `invoke('suggest_related_notes', …)` (store.ts ~2958).
- `<RelatedCandidates>` **props:** `notePath`, `notePersistPath`/`libraryPath`, `defaultType` (`'associative'|'derives-from'`), `heading` (for the fragile "shore it up" variant), `onConnected?` callback. **Display per candidate:** ranked note name (dir="auto") + shared-term chips + snippet + a **Link** button (inert in §B — wired in §C).
- **Fire-once guard:** fetch in an `$effect` keyed on `notePath` only (re-query when the note changes, NOT on every render — Risk §5). Empty/loading/error states are honest (empty = "No strong matches yet").
- Mount in **Reviewer detail** between lines 342↔344, gated `isOrphan(n) ‖ n.reason==='fragile'`, passing the right `defaultType` + heading.

**Verify (Boss-testable, read-only):** Open Reviewer → orphan lens → select an orphan with content. The detail pane shows up to 5 ranked related notes with shared-term chips and a snippet under the prescription. The **Link** button is visible but does nothing yet. Fragile lens shows the same list under a "shore it up" heading. No console errors; selecting a different note re-queries (chips change), not on idle re-render.

---

### §C — One-click **Link** action (the type-picker + headless append)
**Touches:** `src/lib/components/LinkTypePicker.svelte` (net-new small picker), `src/lib/components/RelatedCandidates.svelte` (wire the button), a new headless helper `addLinkToNote(...)` (in `src/lib/reviewer/` or co-located store util — see invariants §4), `ReviewerView.svelte` (refresh via `act()`-pattern + remove inert Connect button line 397).

- **`<LinkTypePicker>`** — built on the registry (no reusable picker exists, findings confirm). Enumerate `topLevelLinkTypes()` **+ prepended `associative`**; render each as a `LinkTypePill`; `defaultType` prop pre-selects (orphan→`associative`, fragile→`derives-from`); emits chosen type. Interaction shape copied from the confidence popover (`OutgoingLinksPanel.svelte:253–270`): fixed overlay + button-per-option.
- **Click flow:** Link button → open `<LinkTypePicker>` (pre-selected) → on choose type `T`, target `Tgt`:
  - **`addLinkToNote(sourcePath, T, Tgt)`** — clone the proven open/closed branch from `addTagToNote` (`+layout.svelte:5248–5269`): if the source note is **open** → `composeNoteModel` (identity-guarded; refuse on `!ok`) → `saveTabContent` with body-appended `[[T::Tgt]]`; if **closed** → `readNote` → `parseFrontmatter` → `writeNote(path, buildFullContent(props, body + '\n[[T::Tgt]]'), 'reviewer_connect')` → `reindexNote(path, lib.name)`.
  - **Single-writer invariant:** writes ONLY body `[[T::Tgt]]` text; NEVER touches `note_links` (derived by `index_note`).
- **Refresh:** after success, re-fetch via the Reviewer `act()` in-place pattern (lines 217–227) → `get_due_notes` re-runs → connected orphan's `incoming_count`→≥1 → leaves the orphan lens (write-time derivation confirmed, `review.rs:337`).
- **Remove** inert Connect button (`ReviewerView.svelte:397`) per ruling 3.

**Verify:** Boss tutorial in §6.

---

### §D — Wire `<RelatedCandidates>` into the remaining 4 hosts (Boss-ruled set)
Landable as one commit per surface if preferred (§D.2–§D.5); each verifiable independently.

- **Surface #2 — NotePane right-sidebar (Backlinks tab):** add `.rs-section` after `+layout.svelte:7483` inside the `'backlinks'` branch — "Suggested connections" for `sidebarTab.path`/`sidebarTab.libraryPath`, `defaultType='associative'`. Fire-once on note change.
- **Surface #3 — 360° Inspector, BOTH mounts:** mount `<RelatedCandidates>` in the footer block between the matrix (`:445`) and HUD (`:456`), gated `data.is_orphan ‖ data.single_point_of_failure ‖ data.missing_link_types`; **plumb `libraryPath`** into Inspector360 props at BOTH mount sites (`+layout.svelte:6576` full-page + `7554` right-rail); reuse its `onNoteClick` contract; `defaultType='derives-from'` when SPOF else `'associative'`.
- **Surface #4 — Note "Health" tab (TensionPanel):** mount beside the orphan/SPOF/gap rows (`TensionPanel.svelte:162–183`; tab at `+layout.svelte:7527`) for the open note; gated on the note being flagged orphan/SPOF/in-a-gap; `defaultType='derives-from'` for SPOF/fragile else `'associative'`. The strongest concept fit — the diagnosis row gets its action inline.
- **Surface #5 — Sky View per-node right-click menu:** add a **"Suggest connections…"** item to the per-node context menu (`GraphMindView.svelte:1153`, per-node via `graphEngine.ts:1791`, node carries `{id,name,path,libraryName}`). NOT on the canvas. Clicking opens `<RelatedCandidates>` for that node in a small popover/modal (reuse the component; the menu supplies notePath/libraryName). `defaultType='associative'`.

**Verify each surface:** §6 multi-surface tutorial (each host: suggestions appear on the right gate, the one-click typed connect works, the connected note's flag clears).

---

### §E — i18n, RTL, gate, simplify, SO
**Touches:** all 15 locale files (`en, de, es, fr, pt, it… ` per CLAUDE.md set: ar, de, en, es, fa, fr, he, hi, ja, ko, pt, ru, tr, ur, zh), docs.

- **i18n ×15:** the ~8 new keys (component heading, "Suggested connections", "shore it up", "No strong matches yet", "Link", picker title, shared-term label, snippet aria). Picker reuses existing `linkTypes.<id>` keys (`en.json:3231–3242`) — do NOT add new type keys.
- **RTL:** `dir="auto"` on note names/snippets; chips/picker flip per `detectDir()`; chevrons/overlay anchoring mirror in RTL.
- **`/simplify`** on the final diff (SO #4).
- **Editor-Surface Gate (top principal):** §C's `addLinkToNote` touches the save/compose path → run the **full 8-point gate** in the reproduction harness AND Boss test. Critical items for this MIG: (2) Focus enter/exit no spurious write; (3)/(4) tab-switch teardown incl. *while in Focus*; (6) **linked-probe rename pair** (A links B, the connect must not corrupt either identity); plus **on-screen===disk** after every connect. The open-note path MUST go through `composeNoteModel`+`saveTabContent` (never a blind disk write behind an open model — `+layout.svelte:5243–5247` invariant).
- **SO docs (same commit set):** session log (`§A…§E` commit hashes), Orientation v-bump (new surface + new command + picker), help files + User Manual ×15, MoCh, mark MIG-086 shipped.

**Verify:** `svelte-check` + `cargo test` green; gate harness all-green; Boss confirms RTL + all 15 locales render.

---

## 4. Reuse + invariants

- **Picker:** none exists (only the CM6 `[[`-autocomplete in `completions.ts:108–117`, not openable as UI) → **build a small `<LinkTypePicker>`** on `linkTypeRegistry` + `LinkTypePill`, interaction copied from the confidence popover. Prepend `associative`.
- **Living Link single-writer:** there is NO `create_link` Rust command. A link is born only as `[[type::Target]]` body text → `writeNote`/`saveTabContent` → `constellation_search_reindex` → `index_note` DELETE+INSERTs `note_links` (`search.rs:5152`). `<RelatedCandidates>`/`addLinkToNote` write body text only — never `note_links`.
- **Rule 8 / Rule 1+3 (query-time, on-demand):** `suggestRelatedNotes` fires on detail-open / note-change / panel-open only — **never per-keystroke**, never from a CM6 ViewPlugin. Fire-once guard per host.
- **Headless append helper:** does NOT exist (`addTagToNote` is the proven tag analog; `src/lib/reviewer/` has zero link-write wiring). **Build `addLinkToNote`** by cloning `addTagToNote`'s open/closed branch (`+layout.svelte:5248–5269`), swapping prop-mutation for body-append, reusing the `reindexNote` tail. Pass `origin='reviewer_connect'` for the write journal.
- **Predecessor → Replacement (top principal):** the inert Reviewer **Connect** button (`ReviewerView.svelte:397`) → replaced **in the same place** by the `<RelatedCandidates>` Link action (Boss-approved §9b ruling 3). The orphan prescription string (`reviewer.rx.orphan`, line 246) stays as the *diagnosis* above the new *action* block.

---

## 5. Risks + mitigations

- **False positives (weak BM25 matches).** Mitigate: honest "No strong matches yet" empty state; show shared-term chips + snippet so the user *sees why* before linking. Optional score floor in §A SQL.
- **Re-query storm (fetch on every render).** Mitigate: per-host fire-once `$effect` keyed on `notePath` only; loading/empty/error states; cancel/ignore stale responses on rapid note switching.
- **Reindex-after-connect refresh.** Reindex maintains the **source** note's outgoing edges + targets' `incoming_count` (`maintain_incoming_after_save`, `search.rs:1214`). Pass the **source path** (the note receiving the wikilink) to `reindexNote`. Reviewer re-fetches via `act()`/`get_due_notes`; sidebar/360 re-fetch on the next note-context change.
- **Performance on 7,600+ notes.** BM25 MLT is a single bounded FTS5 `MATCH` (read-only, indexed) capped at `limit` — no full scan. Stopword guard prevents degenerate queries. **Measure before/after boot + a 7,600-note suggest call** (CLAUDE.md Rule 8 hard constraint) before committing §A.
- **Open-note write corruption** (the BUG-015/§CB class). Mitigate: `addLinkToNote` open branch goes through `composeNoteModel`+`saveTabContent` (identity-guarded), never a blind disk write behind an open model; §E runs the full Editor-Surface Gate incl. the linked-probe rename pair.

---

## 6. Boss test (Testing-Instructions Rule)

### §C — connect an orphan from the Reviewer

**What this is & why it matters.** The Reviewer is your knowledge triage desk. An "orphan" is a note nothing links to — it's intellectually stranded. Until now the Reviewer only *told* you to connect it (the old grey Connect button did nothing). Now it *shows you* the 5 notes most related to it and lets you make a real, typed connection in one click. A typed link isn't just "see also" — it records *how* two ideas relate (supports, causes, derives-from…), which is the heart of Constellation.

**Walk-through (pre-state → action → post-state):**
1. **Open the Reviewer** — left dock, click the Reviewer icon. *You see the master list of due notes.*
2. **Pick the orphan lens** — click **🔗 orphan**. *The list filters to notes nothing links to.*
3. **Select an orphan with real content** (>20 words). *The detail pane opens: prescription card on top, then a new **"Suggested connections"** block listing up to 5 related notes — each with its title, a few shared-word chips, and a one-line snippet.*
4. **Click "Link"** on the most relevant suggestion. *A small type picker opens, with **associative** already highlighted (the sensible default for "these relate").*
5. **Choose a type** — e.g. click **supports** if the orphan supports that note. *The picker closes; you briefly see the list refresh.*
6. **Post-state:** the orphan you just connected **drops out of the orphan list** (it now has an incoming link). Open it and its target — the target's body now contains `[[supports::<orphan title>]]`, and both files are intact.

**Failure modes:** If the note stays in the orphan list after linking → the incoming-count refresh didn't fire (report it). If the *wrong* note's body changed → identity-guard breach (stop, report — this is the gate's job to catch). If the picker doesn't open or shows no types → registry wiring broke.

**Fragile variant:** pick the **⚠️ fragile** lens instead — same 5-note list, but under a **"shore it up"** heading, and the picker pre-selects **derives-from** (a fragile note needs a supporting source).

### §D — the same affordance everywhere

**What this is & why it matters.** The exact same "suggest + one-click typed link" tool now lives wherever you deal with a note's connections — not just in the Reviewer. Same component, same behavior, so you never relearn it.

**Walk-through:**
1. **NotePane sidebar.** Open any note → right sidebar → **Backlinks** tab. *Below "Linked/Unlinked mentions" and "Outgoing", you now see **"Suggested connections"*** for the open note. Click **Link** on a suggestion → pick a type → *the link is written into the open note's body and the Outgoing list updates.* Type something in the editor first to confirm there's **no lag** (the suggestion never fires while you type).
2. **360° Inspector.** Open a note → 360 Inspector. If the note is an orphan or a single-point-of-failure, *below the stratification matrix you see the same **Suggested connections** block.* Connect one → *the matrix's orphan/SPOF flag clears on the next open.*

**Failure modes:** If typing in the editor stutters when the sidebar suggestion is visible → a fetch is firing on keystroke (Rule 1/3 breach — report). If the Inspector block appears on a well-connected note → the `is_orphan ‖ SPOF` gate is wrong. If a connect in one surface doesn't show in another → the reindex/refresh path didn't run.

---

**Commit boundaries:** §A · §B · §C · §D · §E (five commits). Each lands with its verification clause green before the next. Plan-approval = build-approval: cascade §A→§E, pausing only at the §B, §C, and §D Boss-testable verification clauses.

---

# PART 2 — Frontmatter Fold (Boss-ratified 2026-06-24)

Architect: `docs/MIG-086-Architect-Frontmatter-Typed-Links.md`. **Supersedes §C's body-append.** §A + §B
stay shipped. The connect now writes a **type-as-property** frontmatter link (D1); `index_note` reads typed
links from **both** body and frontmatter (D2, dual-source, non-destructive); `note_links` stays the single
index + earned-property store (D3). The §C UI work (`<LinkTypePicker>`, the RelatedCandidates picker wiring,
optimistic removal, `onConnected`, the Reviewer "Connect"-button removal) is **kept** — only the *write
mechanism* changes.

### §F1 — Backend: `index_note` reads frontmatter type-as-property links (dual-source)
**Touches:** `src-tauri/src/search.rs` (`extract_typed_links` / `index_note`). **Does NOT touch** any write path.
- Parse the note's frontmatter: for each property whose **key is a known link type** (registry: the 8 seeds
  + `associative` + custom) and whose value(s) are quoted `"[[wikilink]]"` (single or YAML list), emit a
  `TypedLink { link_type = key, target, annotation: "" }`.
- Merge with the body-derived links; **dedup on `(source, target, link_type)`** (the `note_links` UNIQUE
  index already enforces — ensure the INSERT path treats a body+frontmatter duplicate as one row, no error).
- Reuse `fold_match_key` for the target; `associative` accepted; unknown keys ignored (they're ordinary
  frontmatter, e.g. `tags`, `created`).

**Verify (`cargo test`, fixture):** (1) a note with `supports:\n  - "[[X]]"` frontmatter → one `note_links`
row (type=supports, target folded, confidence `hypothesis`). (2) Body `[[derives-from::Y]]` still indexes.
(3) The SAME `type::target` in body AND frontmatter → exactly ONE row. (4) The existing §A suggest tests +
all prior link tests stay green.

### §F2 — Rewrite `addLinkToNote` to write the frontmatter property (props save path)
**Touches:** `src/lib/libraries/store.ts` (`addLinkToNote`), `RelatedCandidates.svelte` (call shape unchanged).
- Add `"[[<targetName>]]"` to the source note's frontmatter property named by the chosen type (create the key
  as a list if absent; append + dedup if present).
- **OPEN source:** `composeNoteModel` (identity-guard; refuse on `!ok`) → `editNoteProps`(updated props) →
  `saveTabContent`. **CLOSED source:** `readNote` → `parseFrontmatter` → add to props → `buildFullContent`
  → `writeNote('reviewer_connect')` → `reindexNote`. **Remove** the body-append + `markCascading` /
  `reloadTabsFromDisk` dance — the props path is BUG-015-safe by construction (it's exactly `addTagToNote`).
- Confidence default `hypothesis` still comes from `index_note` (C-4, automatic). NO `note_links` writer.

**Verify (Boss test + reproduction harness):** the FULL **8-point Editor-Surface Gate** on the props path;
connect → the source note's frontmatter gains `type:\n  - "[[orphan]]"`; orphan leaves the orphan lens;
**on-screen === disk** after every connect; the **linked-probe rename pair** (A links B in frontmatter,
rename B, both identities intact). Boss tutorial per the Testing-Instructions Rule.

### §F3 — Rename cascade rewrites frontmatter wikilinks
**Touches:** `src-tauri/src/libraries.rs` (`update_links_on_rename`) + the JS cascade reload.
- Extend the walker to rewrite quoted `"[[oldname]]"` / `"[[oldname|alias]]"` inside frontmatter typed-link
  properties, not just body wikilinks (invariant D6 — frontmatter links must survive rename).

**Verify:** rename a note that is a frontmatter link target → the source's frontmatter link updates; reindex
reflects the new target; linked-probe pair intact; no body corruption.

### §F4 — Editor display of frontmatter typed-links
**Touches:** `PropertyEditor.svelte` (+ NotePane sidebar as needed).
- Render a type-as-property link with a self-contained `LinkTypePill` + a clickable target (opens the note).

**Verify:** a note with frontmatter typed-links shows the type pill + clickable target in its properties;
the target's Backlinks/Outgoing show the relationship; clicking opens the target.

### §D — Wire `<RelatedCandidates>` into the remaining 4 hosts
As Part 1 §D, but the connect now writes frontmatter. Resolve the **direction** question here: for
non-orphan hosts (NotePane sidebar / Sky node), default the declared link to **in-hand note → suggestion**;
confirm with Boss at the §D test (orphan/fragile stay suggestion → orphan, required to de-orphan).

### §E — i18n ×15, RTL, /simplify, docs, gate, SO
As Part 1 §E, plus: document **frontmatter typed-links** as a user-facing feature in help + User Manual ×15;
Orientation v-bump (the frontmatter-link model + the doc-drift correction that no `LINK` file exists);
mark MIG-086 shipped.

**Commit boundaries (Part 2):** §F1 · §F2 · §F3 · §F4 · §D · §E. Plan-approval = build-approval: cascade,
pausing at the **§F2** (connect) and **§D** Boss tests. §C's body-append (`addLinkToNote` v1) is replaced by
§F2 and never shipped.