# Session Log — 2026-06-24

## MIG-086 §C — the one-click typed-link action (THE save-path step)

**Function in hand:** the **Link** button inside `<RelatedCandidates>` (the "Connect to:" list in the
Reviewer's orphan/fragile detail). §A (`suggest_related_notes` backend) and §B (the read-only list)
shipped + Boss-validated; the Link button is currently INERT. §C makes it act.

**Concept (the horse):** a suggestion is a *diagnosis*; the typed link is the *act*. Clicking **Link**
on a related note always asks "what kind of relationship?" (the 8-type picker), then turns the answer
into a typed Living Link — born as `[[type::Target]]` body text, one deliberate connection at a time.

### Pre-build investigation (two parallel Explore agents — facts verified against live code)

**A. Link direction (CONFIRMED, and it inverts the naïve reading).** An orphan O is defined by
`incoming_count = 0` (`review.rs:337`). To de-orphan O, a note must link *to* O. So the wikilink
`[[type::O]]` is written into the **candidate's** body (candidate = source, orphan = target), NOT into
the orphan. Reindexing the candidate runs `maintain_incoming_after_save` (`search.rs:1214`) which bumps
**O's** `incoming_count` → O leaves the orphan lens on the Reviewer's next `get_due_notes`.
⇒ `addLinkToNote(candidatePath, type, orphanName)`.

**B. C-4 is automatic.** `index_note` (`search.rs:5164`) hard-codes `confidence='hypothesis'`,
`weight 1.0`, `status 'active'` for every body-derived link. No explicit stamping needed — a
suggestion-born link enters as `hypothesis` by construction. ✔ C-4.

**C. Target string.** `index_note` resolves `[[type::target]]` by folded `LOWER(name)` against
`note_meta.name`. The orphan's Reviewer `note_name` *is* `note_meta.name`, so passing the orphan's
`note_name` as the target resolves correctly (Unicode fold made consistent by MIG-085 §B.0).

**D. Typed-link parsing.** `[[type::target]]` → `link_type=type`, `target=fold(target)`; `associative`
accepted as the null/default type; the 8 seeds + custom types all valid (`search.rs:4504`).

### Predecessor → Replacement (top principal — written BEFORE any code edit)

- **Predecessor:** the Reviewer's orphan-only primary **"🔗 Connect"** button
  (`ReviewerView.svelte`, current line ~410: `onclick={() => onNoteClick?.(n.note_path, n.note_name)}`).
  Introduced by the MIG-084 rich-Reviewer redesign. It does NOT connect — it merely **opens the note in
  the editor**, which is the *identical* action already offered by the **"↗ Open in editor"** hand-off
  (current line ~422, same `onNoteClick`). It is the "inert Connect button" Boss-ruling 3 (Plan §1)
  targets ("remove the inert Reviewer Connect button"); the Plan's `:397` line ref predates MIG-084.
- **Replacement — in the same place (Boss-approved §9b ruling 3):** the `<RelatedCandidates>` Link
  action, already mounted directly under the prescription card (`ReviewerView.svelte:348–355`, §B). The
  orphan prescription string stays as the *diagnosis* above the new *action* block.
- **Cut:** the orphan-specific primary "🔗 Connect" button. **Kept:** "↗ Open in editor" hand-off
  (covers opening), "🗄️ Mark standalone" (the orphan's dismiss verb). Net for an orphan: connect via the
  inline Link buttons, declare standalone, or open in editor — no redundant punt-to-editor button.
- This is logged for the §C Boss test to confirm, since the Reviewer was redesigned after the Plan.

### Plan-detail CORRECTION (genuine architectural surprise — surfaced, not ambushed)

Plan §C says the open-note branch goes "through `composeNoteModel`+`saveTabContent` … with body-appended
`[[T::Tgt]]`." **That is wrong for a *body* change.** Under SINGLE_OWNERSHIP, `saveTabContent`
(`store.ts:613`) IGNORES its `body` param and composes the disk body from the **noteModel** alone — it
only writes *props* into the model (`editNoteProps`). That is exactly why the tag analog (`addTagToNote`)
works: tags are props. A body append through `saveTabContent` would write nothing. Worse, mutating the
model's body behind the live NotePane editor (the body owner, which pushes its rope to the model on every
keystroke) would be **clobbered** on the next edit — the BUG-015 class.

**Correct mechanism (honors the deeper invariant the Plan was protecting):** the proven precedent for
programmatically editing a possibly-open note's body is **`toggleTaskReconciled`** (`store.ts:533`,
MIG-082 §A.3) / the rename cascade: `markCascading` (silences the open editor's autosave) → flush the
model's unsaved edits to disk first via `saveNoteSession` (a direct write that bypasses the cascade
gate) → append on disk (`readNote`→`parseFrontmatter`→`writeNote`) → `reindexNote` → `reloadTabsFromDisk`
(re-seeds the model + bumps `reloadVersion` so the `{#key}` REMOUNTS NotePane with the appended content).
NEVER a blind disk write behind a live model. `addLinkToNote` is built as the structural twin of
`toggleTaskReconciled`, handling open + closed sources uniformly.

### Build (this commit)

- **NEW** `src/lib/components/LinkTypePicker.svelte` — a small 8-type picker on `linkTypeRegistry`
  (`topLevelLinkTypes()` + prepended `associative`) rendering each as a `LinkTypePill`; pre-selects
  `defaultType`; fixed-overlay popover (interaction copied from the confidence popover,
  `OutgoingLinksPanel.svelte:253`).
- **`RelatedCandidates.svelte`** — wired the Link button: opens the picker → on choose,
  `addLinkToNote(candidate.path, type, noteName)` → optimistic-remove the connected candidate +
  `onConnected`. New prop `noteName` (the in-hand note's display name = the link target).
- **NEW** `store.ts::addLinkToNote(sourcePath, linkType, target)` — the headless, content-integrity-safe
  body-append (twin of `toggleTaskReconciled`). Single-writer: writes body `[[type::target]]` only,
  never `note_links` (index_note derives it). **NO bulk "Link all"** (concept invariant C-1).
- **`ReviewerView.svelte`** — pass `noteName={n.note_name}` + `onConnected` (reload + re-seat
  selection, mirroring `act()`); removed the orphan "🔗 Connect" primary button (Predecessor above).
- **i18n (en only this commit; ×15 in §E):** `reviewer.pickTypeTitle`.

**Verification:** `svelte-check` green → full 8-point Editor-Surface Gate in the reproduction harness →
Boss tutorial (Plan §6). NO note_links writer; default confidence `hypothesis` (C-4, automatic).

### Adversarial review of the §C diff (workflow `wf_84fdf194-c5c`, 4 dimensions × independent verify)

10 raw findings → **5 confirmed, 0 uncertain, 5 refuted**. Per "Fix what you discover" (WA#6), **all 5
fixed before the Boss test** — none deferred:

1. **P2 — lossy frontmatter round-trip (File-Over-App breach).** `addLinkToNote` re-serialized the
   candidate's frontmatter via `parseFrontmatter`→`buildFullContent` (drops YAML comments/blank lines,
   re-quotes, can't represent block scalars) — and the candidate is almost always a CLOSED third-party
   note the user never opened. **Fix:** append the wikilink to the **raw `content`** at EOF (no parse) —
   every frontmatter+body byte preserved. *(Verified `content === prefix + body` exactly via
   parseFrontmatter's `lines.slice(endIndex+1).join('\n')`; raw-append also handles the
   no-trailing-newline edge the reviewer's slice suggestion would have glued — strictly safer.)*
2. **P2 — reindex failure stranded the model behind disk.** For an OPEN source, if `reindexNote` threw,
   `reloadTabsFromDisk` was skipped → model held the old body → next autosave clobbered the link.
   **Fix:** run `reloadTabsFromDisk` BEFORE reindex (model adopts the append regardless), and make
   reindex non-fatal (try/catch — it's a derived view that self-heals).
3. **P3 — stale picker across a note switch (latent).** **Fix:** reset `picker`/`connecting` in the
   `notePath` `$effect` (host-independent — protects the §D hosts too).
4. **P3 — picker hid custom CHILD link types.** Used `topLevelLinkTypes()`; the editor's `[[`
   autocomplete uses the full list. **Fix:** `getLinkTypes()` (full resolved list, children in place).
5. **P3 — `pickTypeTitle` missing in 14 locales.** **Fix:** added native translations to all 14
   (ar/de/es/fa/fr/he/hi/ja/ko/pt/ru/tr/ur/zh) — all 15 locales now carry the key and parse.

Refuted (correctly): direction is right; no $effect loop; C-1..C-5 honored; the open-note
markCascading→flush→reload pattern matches the `toggleTaskReconciled` precedent.

**Post-fix:** `svelte-check` 0 errors (my files add 0 warnings); MIG-076/067/084 suites 51/51 green.

### Boss test round 1 — two findings (fixed before re-test)

**Point 4 — picker truncated off the right edge.** The `<LinkTypePicker>` opened at the click point;
the Link buttons sit at the pane's right edge, so the menu ran off-screen and the type labels were
clipped. **Fix:** measure the menu after mount and viewport-clamp it (`bind:this` + an `$effect` that
shifts it fully on-screen, opening leftward/upward as needed); hidden until placed (no flash). Also a
`max-width` guard. (`LinkTypePicker.svelte`.)

**Point 3 — "show ALL related notes, sequenced by closeness," not a 5-cap.** §A had THREE caps: the
return clamp (`unwrap_or(5).min(20)` — the visible "5"), the BM25 pool (`LIMIT 60`), and a
`processed >= 15` cap that existed only because computing each candidate's shared-term *why* re-tokenized
its whole body (the Rule-8 cost). **Fix (the right one — make the why cheap, then uncap):** rewrite §A
step 5 to detect shared terms via the **FTS index** (≤12 term-membership lookups against the candidate
rowid set) instead of re-tokenizing each candidate. That removes the per-candidate tokenization, so the
`processed` cap is gone and the WHOLE BM25 pool is surfaced, still ranked closest-first; return clamp +
default raised to the pool size (60). Snippet now reads a 400-char body prefix (bounded). Frontend:
`suggestRelatedNotes` called with no limit (→ all); the list gets `max-height:60vh; overflow-y:auto` so
a long list scrolls in place (`RelatedCandidates.svelte`). The 60-candidate BM25 pool is the relatedness
ceiling (a perf-safe bound, not a UI cap) — effectively "all" for a link-poor note.
  - **Rule 8 measured (live 7,660-note universe, `rehearse_live_suggestions --ignored`):** 477 ms
    (Geological clock) / 1000 ms (Edinburgh's High Kirk) / 1383 ms (السلطان محمد الفاتح) — within the
    prior §A budget (0.5–2.3 s for the biggest notes); on-demand only (Rule 1/3). No regression; the
    cheaper FTS-membership path offsets processing the full pool. §A unit tests 3/3 green.

**Predecessor → Replacement (point 3 touches shipped §A):** `suggest_related_impl` step-5 shared-term
computation — was per-candidate `tokenize_tf` (re-tokenize name+body, cap `processed=15`); replaced
**in place** by FTS index term-membership (`term_hits` map). Same result semantics (a candidate is shown
iff it shares ≥1 distinctive query term — C-2), cheaper, uncapped. Return clamp `min(20)`→`min(60)`,
default `5`→`60`. No schema change; the §A unit-test contract (self/already-linked/no-shared/empty/limit)
is preserved and green.

## MIG-086 — Boss design discovery → FOLD typed-links-in-frontmatter (2026-06-24)

**Trigger (Boss, after §C Stage-1 PASS):** a typed link appended as dangling `[[type::target]]` body text
"without context is illogical." Led to a whole-ecosystem question: should typed links live in frontmatter
**Properties**?

**Research (workflow `wf_ce7372d6-d02`, 5 agents + fact-check):** Obsidian Properties (links now in graph +
backlinks natively, 2026), Dataview inline fields, **Breadcrumbs** (declare-one-direction + auto-implied
reverse — the precedent), Logseq/Roam/Tana, RDF/property-graph KR theory. Codebase scan confirmed a
**doc-drift**: the CLAUDE.md "Living Link = first-class `LINK` file on disk" does NOT exist in code — links
are 100% body-`[[type::target]]`-derived into `note_links`; rich props live only in that table.

**Boss rulings (2026-06-24):**
1. **Continue MIG-086 §C and FOLD the frontmatter-typed-links approach into this migration.**
2. Frontmatter **shape = type-as-property** (`supports:\n  - "[[X]]"`) — chosen over a structured
   `links:` object-list.
3. A **brand-new frontmatter-structured link type** (parent/child TOC; authors/screenwriters) → **future
   job PJ-065** (research/Concept-Paper-first; depends on this fold's foundation). Filed in Pending Jobs v1.14.

**Outcome:** §C's body-append is superseded. New Architect (`docs/MIG-086-Architect-Frontmatter-Typed-Links.md`)
+ Plan Part 2 (`MIG-086-Plan.md`, phases §F1–§F4 + §D + §E). Decisions D1–D6 + invariants 1–8 recorded in
the Architect. **§C remains UNCOMMITTED in the working tree** pending the fold (the §C UI — picker, wiring,
button removal — is kept; only the write mechanism changes from body-append to frontmatter props path).
Awaiting Boss Plan-approval before building §F1.

### Plan APPROVED 2026-06-24 → cascading §F1 → §E (pause at §F2 + §D)

**§F1 — `index_note` reads frontmatter typed-links (dual-source) — DONE.** New
`extract_frontmatter_typed_links` + `emit_frontmatter_links` (search.rs) parse type-as-property links
(scalar / inline-array / block-list, block-aware like `extract_aliases`); a known-type key → its real
type, a non-type key → `associative` (byte-for-byte back-compat with the prior full-content scan). The
body parser now runs on the stripped `body`; `index_note` merges body+frontmatter links, dedup on
`(type::target)`. **7 new unit tests + link-parser (6) + suggest (3) all green.**

**§F2 — connect writes the frontmatter property (props path) — DONE.** `addLinkToNote` rewritten: the
link is now declared as `<type>: ["[[target]]"]` in the SOURCE note's frontmatter via the proven PROPS
save path (OPEN → `composeNoteModel`+`saveTabContent`; CLOSED → `readNote`+add-prop+`writeNote`+`reindex`)
— exactly `addTagToNote`. The §C body-append + cascade machinery is GONE; the body is never touched, so the
BUG-015 surface is eliminated. New `addTypedLinkToProps` (dedup, list semantics). `reconstructFrontmatter`
now quotes list items via `quoteIfNeeded` (so `[[X]]` → valid YAML `"[[X]]"`; round-trip byte-stable since
the block-list parser strips quotes on read; also fixes pre-existing special-char list items). svelte-check
0 errors / 0 new warnings; mig-076/067/084 51/51 green.

**§F3 — frontmatter links survive rename — DONE (no cascade change needed).** `rewrite_wikilinks_in_text`
already runs `replace_all` over RAW file content, so a quoted frontmatter `"[[Old]]"` is rewritten in place
to `"[[New]]"` (quotes preserved); the post-rename reindex re-derives `note_links`. Locked with a new
cascade test (`frontmatter_typed_link_rewrites_on_rename`); **12 cascade tests green.**

**§F4 (display polish) deferred to after the §F2 Boss test** — the PropertyEditor already renders the new
`<type>:` list property, so the connection is visible/testable now; the type-pill + clickable-target polish
lands next. **Rebuilt frontend + binary for the §F2 Boss test.**

### §F2 Boss test round 1 — PASS (functional) + a ~2-minute latency finding → fixed

**Functional PASS (Boss screenshots):** connecting orphan "Note Regression Test" wrote
`supports:\n  - "[[Note Regression Test]]"` into the candidate "Machine learning" frontmatter; ALL other
properties (title, tags, maturity, stage, source, attribution, cid_cn…) + the entire body intact. The
dangling-link problem is gone; content-integrity holds. The concept is proven.

**Boss finding:** the connect took **~2 minutes**. Root-caused via a 4-agent diagnosis workflow
(`wf_9d58a4b6-d73`): `index_note` DELETE+re-INSERTs ALL of the candidate's `note_links` edges; each edge
fires the sky stratum/maturity triggers (`STRATUM_SQL_EXPR`/`MATURITY_SQL_EXPR`, search.rs:183-317) whose
`COUNT(DISTINCT source_path)` subqueries scan the 233,995-row table. "Machine learning" is a link-dense
Wikipedia import → O(edges × big-subquery) ≈ 2 min. **PRE-EXISTING** (every link-dense *save* pays it,
hidden because saves reindex fire-and-forget); §F2 exposed it by `await`ing the reindex. (Embedding ruled
out — semantic search off by default + that call is non-blocking.)

**MIG-086 fix (this commit):** make the connect non-blocking, matching the proven save pattern —
(1) `addLinkToNote` (closed path) now **fire-and-forgets** the reindex (the frontmatter write, the source of
truth, is still awaited); (2) `ReviewerView.refreshAfterConnect` is now **optimistic** — it drops the
in-hand note's current lens row (`selected.reason`-scoped) for instant feedback instead of blocking on a
get_due_notes reload (which would still show the row until the background reindex finishes). The DB
reconciles on the next Reviewer load.

**Root cause filed as PJ-066** (P1 perf, SKY/MIG-079 domain, /migration + measurement) — the sky-trigger
reindex storm; not folded into MIG-086 (separate subsystem). Surfaced to Boss for a priority ruling
(WA#6 exception: genuinely needs a separate migration). svelte-check 0 errors after the fix.

### §F2 Boss test round 2 — Check 1 PASS (instant), Check 3 PASS, Check 2 fixed

- **Check 1 (timing): PASS** — connect is now instant (non-blocking + optimistic).
- **Check 3 (rename): PASS** — frontmatter link follows the rename ("took a while" = the PJ-066 reindex
  the cascade pays; pre-existing, Boss ruled to fix separately).
- **Check 2 (open-note display): fixed.** When the suggestion is ALREADY OPEN in a tab, the new property
  didn't show until reopen (the props save path updates model+disk but doesn't re-render an open note's
  PropertyEditor). Fix: `addLinkToNote` open path now calls `reloadTabsFromDisk([sourcePath])` after
  `saveTabContent` — re-seeds + `{#key}`-remounts the open tab from the just-written disk (cheap: one file
  read, no reindex; the proven `toggleTaskReconciled` refresh pattern). svelte-check 0 errors; rebuilt.

**Boss ruling: finish MIG-086 first** (PJ-066 = its own later Sky/perf migration). **§F4 (PropertyEditor
type-pill + clickable target) folded into §E** — the frontmatter link already renders as a list-item chip
(functional); the clickable/pill polish groups with the §E polish pass. NEXT: §D (wire `<RelatedCandidates>`
into the 4 other hosts) → pause at §D Boss test → §E.

## Relationship-typology exploration → PJ-067 (Concept-Paper-first); MIG-086 finishes on current model

Boss-driven open-ended exploration of the link vocabulary. Workflow `wf_f97e9d18-518` (5 sourced digs +
fact-check) mapped the territory: a typed link varies on DIMENSIONS (symmetry / **transitivity** /
**inverse-converse** / **arity binary↔n-ary** / cardinality / **taxonomic↔thematic**) and FAMILIES (7
covered; uncharted: thematic/functional, **analogy/structure-mapping**, **n-ary synthesis**,
argument-attack **undercuts/undermines**, **problematizes/answers**, qualifies). Research doc:
`docs/Living-Link-Relationship-Typology-Research-2026-06-24.md`. Two load-bearing flags: rename
`complements`→**`co-completes`** (lexical "complementarity" = the OPPOSITE); "together form an idea" is
genuinely **n-ary** (a synthesis node), not a pairwise symmetric link.

**Boss ruling 2026-06-24:** *finish MIG-086 on the current 8 + directional/symmetric model; Concept-Paper
the typology.* → **PJ-067** filed (Living Link Relationship Model v2; Concept-Paper-first). The symmetric
tier + `co-completes` + ALL new dimensions/families live in PJ-067, NOT MIG-086.

### STATE OF STANDING (SO #5 snapshot — after the major exploration/decision)

**Verified-shipped + protected (committed, on `main`):** MIG-085 §B; MIG-086 §A (`suggest_related_notes`)
+ §B (`<RelatedCandidates>` read-only). Binary in use predates today.

**Built + verified this session, UNCOMMITTED in the working tree (the frontmatter fold):**
- §F1 — `index_note` dual-source typed-links (body + frontmatter type-as-property); 7 tests + no regression.
- §F2 — connect writes the frontmatter property (props path); **Boss-validated PASS** (clean frontmatter
  link, full content-integrity). Latency root-caused (sky-trigger storm, PJ-066) → connect made non-blocking
  + optimistic Reviewer. Check 2 (open-note refresh) fixed via `reloadTabsFromDisk` — **rebuilt, pending
  Boss re-verify**.
- §F3 — frontmatter links survive rename (cascade already content-wide); locked with a test.
- Picker/UI from §C kept; ReviewerView orphan "Connect" button removed.
- Docs: Architect (frontmatter fold) + Plan Part 2; Pending Jobs v1.14 (PJ-065/066/067); typology research
  doc; this session log. Binary at 18:21 has §F1–§F3 + all fixes.

**Known-broken / at-risk:** PJ-066 (link-dense reindex ~2 min, background, pre-existing — Boss deferred).

**COMMITTED `dc6056a4` (2026-06-24)** — MIG-086 §F1–§F3 + §A uncap + §C UI, Boss-validated (Check 2
re-verified PASS → §F2 fully closed). 26 files. The frontmatter fold is secured on `main`. (The regenerated
`_manifests.generated.ts` was deliberately left unstaged — unrelated Sight build artifact.)

### SESSION CLOSE — PCS (2026-06-24, 20:14)
- **Commit + Push:** code milestone `dc6056a4`; SO artifacts committed + pushed at close.
- **SO docs:** Orientation **v3.05** (new file; preamble captures the frontmatter fold + the LINK-file
  doc-drift correction + PJ-065/066/067 + the typology — BODY §4.x/§8/§12 reconciliation deferred to the
  §E ship-time full v-bump); **MoCh** `docs/MoCh/MoCh-2026-06-24-1100.md` (conversational trace); **Handover**
  `lab/reports/HANDOVER-2026-06-24-mig086-frontmatter-fold.md` (+ ready-to-paste next-session prompt).
- **Help / User Manual:** DEFERRED to §E (per the plan — MIG-086 not fully shipped; §D pending; documenting
  a half-wired feature would mislead). Stated explicitly, not silently skipped.
- **Next session:** MIG-086 §D (4 hosts) → §E. Then PJ-067 (typology Concept Paper), PJ-066 (sky perf), PJ-065.

**Pending, not started:** §D (4 hosts), §E (i18n×15, RTL, /simplify, help/UM, Orientation v-bump, MoCh,
mark shipped). Then PJ-067 (typology Concept Paper), PJ-065, PJ-066.

### §D plan (current model) — direction is host-set (user In/Out/Both → PJ-067)
`<RelatedCandidates>` gains a host-set `direction: 'inbound' | 'outbound'` prop; `choose()` routes
`addLinkToNote` accordingly (inbound = suggestion→in-hand/de-orphan; outbound = in-hand→suggestion).
- **Diagnostic surfaces → inbound:** Reviewer orphan/fragile (existing), 360 Inspector + Health/TensionPanel
  (gated on orphan/SPOF/missing-link-types).
- **General surfaces → outbound:** NotePane sidebar Backlinks tab (always), Sky View per-node menu.
- libraryPath plumbing gaps to close: Inspector360 (both mounts) + TensionPanel (prop) + GraphMindView
  (derive from libraryName via allNotes). Mount map captured this turn.
