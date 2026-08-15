# SESSION LOG — 2026-08-11

## PJ-252 — the frontmatter block classifiers disagree, and adding a tag deletes the tags already there

**Status: CLOSED — Boss-validated over five test rounds (Stage 1, then Stage 2 Parts 1–4),
all passed. Committed after the pass, never before.**

Function in hand: **the frontmatter property write path** — `store.parseFrontmatter` (the
projection the Properties panel and the note model read) and `yamlDoc.composeFrontmatter`
(the byte-perfect CST write).

### The defect, and why there was a fourth one

Two classifiers answered the same question about the same bytes and disagreed:

- `store.parseFrontmatter` worked on LINES. `blockExtent`/`isYamlBlockChild` absorbed a `#`
  comment (or a wrapped continuation) into the block, then required every content line to be a
  bare `- item` before projecting an editable list. A comment fails that → READ-ONLY, empty
  `value`, no `listItems`.
- `yamlDoc.immutableBlockKeys` asked the `yaml` library, which attaches the comment as a
  comment and folds a wrapped item into ONE scalar → "all scalars" → not protected.

Every list mutator rebuilds from `p.listItems ?? (p.value ? split : [])` = `[]`, so the block
was spliced out and re-appended holding only the new tag. **Adding one tag deleted the tags
already there, from the `.md`, with no error and a clean re-parse afterwards.**

The fourth shape of one defect (2026-07-24 seq-of-maps, PJ-182 block scalars, PJ-136 nested
maps). It existed because each closure re-answered the question in a **second place**.

### The fix — one classifier

`yamlDoc.classifyDoc` / `classifyFrontmatterValues` is now the single answer to "what is this
key's value?" (`list` · `structured-list` · `block` · `scalar`), read by:

1. `store.parseFrontmatter` — the panel's projection, **including the list's item VALUES**;
2. `composeFrontmatter`'s `immutableBlockKeys` — the write path's refusal;
3. `projectProps` — a third classifier that had been answering it independently.

`composeFrontmatter` now parses the document ONCE (it previously parsed twice — the H1 gate
and `immutableBlockKeys`), so the classification and the refusal cannot come apart.

The refusal is stated **closed** (`WRITABLE_KINDS`), so a fifth `FmValueShape` added to the
union arrives REFUSED rather than silently writable.

`seqCarryingComments` carries an edited list's comments across the splice-and-append rewrite,
so making those lists editable does not trade a destroyed list for a destroyed comment.

### Two more shapes found by RUNNING the path while fixing it (both now fixed + pinned)

- An inline `- history   # why` was projected with the comment **inside the tag value**, and
  written back as the literal quoted tag `"history   # why"`.
- `whatever: [alpha, beta]` under a key `detectPropertyType` does not know was typed `text`
  valued `alpha, beta` — editing the note wrote the sequence back as that string.

### The regression this change nearly shipped — caught by the ui-inspector gate

Routing the projection through the library made **how the YAML text is extracted**
load-bearing for the first time. `parseFrontmatter`'s `rawYaml` is `yamlLines.join('\n')`; on a
CRLF note every line still ends in `\r`, and `join` puts a separator only BETWEEN elements, so
the **last line's `\r` was unterminated and the library read it as DATA**. The final property
of a Notepad-saved note came back as `snurfle\r` and was written as the quoted tag
`"snurfle\r"` — on an ordinary plain tags list, far commoner than any shape this PJ set out to
fix.

The ui-inspector found it by running the real chain and REJECTED the test. Its attribution
("pre-existing") was the one thing off: reproduced against HEAD, the same note round-trips
clean before this change. **It was mine.** Fixed by handing the classifier
`splitFrontmatter(content).yaml` — the exact bytes `composeFrontmatter` composes from — rather
than a second extraction of the same region. Two CRLF cases pinned.

### Verification

| gate | result |
|---|---|
| vitest | **941 passed / 0 failed** (82 files). The 3 `it.fails` are now `it`, plus 10 new. |
| Rust | **1445 passed / 0 failed** (1442 + 3 new `inject_cid_cn`) |
| svelte-check | **0 errors**, 268 warnings (baseline unchanged) |
| release binary | `src-tauri/target/release/constellation.exe` **2026-08-11 10:11** — sources → `build/` 10:07 → exe 10:11 (the mtime chain is the check; Tauri compresses embedded assets, so grepping the `.exe` for a marker string finds nothing either way — verified with a control string) |
| Boss test | Stage 1 + Stage 2 Parts 1–4, **all PASSED**, evidence returned as screenshots + Notepad bytes |
| `/simplify` | 4 agents; 8 findings applied, 3 skipped with reasons (below) |
| safety-inspection | ran twice; **54 confirmed findings, none in this diff** |
| Boss test | `tutorial-auditor` → `ui-inspector`: **REJECTED ×2, APPROVED on the third** |

**Perf, measured (20,000 calls, 20-line frontmatter):** `parseFrontmatter` 7.2 µs → 110 µs,
essentially all of it the one `parseDocument`. All 26 call sites read: no library walk, not on
the keystroke path, loops bounded (restored tabs, the split view's 2–4 companions, the
~40-note multi-select tag add — each iteration of which already awaits a read and a write).
`composeFrontmatter` gave one parse back in the same change.

### `/simplify` — skipped, with reasons

- **Route `serializeLine`'s list fallback through `propRow.listItemsOf`** — skipped: it adds a
  `.filter(Boolean)`, a behavior change on the write path. Vindicated the same hour — the
  safety inspection confirmed a HIGH in `listItemsOf` itself (`propRow.ts:101`, raw-comma split
  instead of the quote-aware `splitFlowSeqItems`).
- **Delete `projectProps` / `parseFrontmatterDoc`'s projection as production-dead** — out of
  scope; filed instead.
- **Exhaustive `switch` + `never` on every `FmValueShape` consumer** — took the cheaper
  structural half instead (the closed `WRITABLE_KINDS` refusal above).

### The blank-line family — surfaced by the Boss's own test, two independent causes

The Boss's Stage-1 Notepad screenshot showed a blank line between `---` and `title:`. Two
separate writers were producing one symptom:

1. **`canonical.rs::ensure_cid_cn`** — re-emitted the frontmatter slice under a fresh `---\n`
   while the slice itself BEGAN with that newline. Fires on the one pass that injects a note's
   identity, i.e. the first time any note is ever opened, so it was the normal state of every
   hand-authored note Constellation had opened. Third sibling of what PJ-207 §15 fixed in
   `update_frontmatter_title` and `set_frontmatter_parent` — the only one it had not reached,
   and flagged as LOW by the morning's whole-app sweep before the Boss ever saw it. Split out
   as a pure `inject_cid_cn` so the shape is testable without disk; 3 Rust tests.
   **Boss ruling, mid-edit: `cid_cn` is Constellation's note-categorisation system and is NOT a
   bug.** Correct — the fix moves a newline; every `cid_cn:` emission is byte-identical and
   `generate_canonical` is untouched. Stopped on the correction, listed the diff, got "keep it,
   but don't mess with the cid_cn naming system", and verified that claim against the diff
   before proceeding.
2. **`composeFrontmatter`** — splicing the block's FIRST key empties the CST residue, and
   `+= eol` on an EMPTY string invented the line. **Pre-existing**: measured at HEAD,
   byte-identical output. My first fix for it was wrong AND its comment claimed something
   untrue — that a user-typed blank would be preserved. It would not: at that point a
   user-typed blank and a splice-left blank are indistinguishable, both simply gone from the
   residue, and the old code preserved the user's only by the same accident that fabricated
   one. The user's blank is now restored from `rawYaml`, the file's own bytes. Both cases
   pinned.

### The Boss test — five rounds, three gate rejections, two real catches

| round | verdict | what it caught |
|---|---|---|
| Stage 1 draft | REJECTED | Properties opens COLLAPSED — the test asked the Boss to read chips that would not be on screen |
| Stage 1 v2 | REJECTED | **the CRLF regression**, found by running the real chain |
| Stage 1 v3 | APPROVED | — |
| Stage 2 P1 | REJECTED | "reopen in Notepad" is ambiguous; Notepad shows a stale buffer and has no Reload |
| Stage 2 P1 v2 · P2 · P3+P4 | APPROVED | — |

Two Boss-side observations worth recording, both of which were *my* wording, not defects:

- **"Shouldn't the title be pj252-test-1?"** — `deriveTabName` (`store.ts:3230`) takes the
  display name from the frontmatter `title:` when present; the sidebar shows the filename. Both
  correct and by design. My test text had him type a `title:` that did not match the filename.
  Filed **PJ-261**: the function's own comment says "for canonical files", but it applies to
  every file.
- **"Step 1.1 is missing the comment"** — he was reading the Properties panel, which
  structurally cannot render a YAML comment. My wording sent him to the panel and the file in
  one breath. Read the file's bytes myself rather than sending him back; the comment was there.
  Fixed the wording for Parts 2–4, and the inspector now reviews for that failure mode.

### Filed, not fixed here

- The altitude review's survivors: `detectPropertyType` (`store.ts:199-236`) and
  `PropertyEditor.svelte:469-485` still independently answer "is this key a list" for
  property-TYPE purposes — a different question from YAML node kind, and changing it changes
  behavior. **Block EXTENT** is also still line-decided (`blockExtent`), so kind can no longer
  disagree but extent still can.
- The retained line-based fallback in `parseFrontmatter` runs ONLY when the YAML does not
  parse, where `composeFrontmatter`'s H1 branch re-emits the bytes verbatim and makes no
  structural edit — safe by construction, and stated as such in the code.
- **Two whole-app safety sweeps, 54 confirmed findings**, none in this diff. Both runs ignored
  the `args.files` diff-scoping and went whole-app; the frontmatter write path was covered in
  full either way. Registers to be triaged into the ledger.

---

## STATE OF STANDING — 2026-08-11, at the Boss's request (SO #5)

Written before answering, per Standing Order #5. Sources: `docs/Constellation Pending Jobs
v1.80.md` (the live backlog), the six whole-app sweep registers in `lab/reports/sweeps/`, and
`git log`. Nothing below is recalled — every count was read or computed from the repo today.

### (a) Verified-shipped and protected

- **PJ-252 CLOSED** (`de951dfd`, pushed) — Boss-validated over five rounds. Frontmatter now has
  ONE classifier; the write path's refusal is stated closed; comments survive an edited list.
- **PJ-249 CLOSED** (2026-08-11, prior session) — rename ~50 s → 216 ms.
- Gates at this moment: vitest **941/0** · Rust **1445/0** · svelte-check **0 errors** · release
  binary **10:11** Boss-validated.
- Protection in place: 941 vitest + 1445 Rust cases, the `tutorial-auditor` → `ui-inspector`
  gate (which caught an APP-KILLER-class regression this session that all 941 tests missed), and
  the `safety-inspection` workflow.

### (b) At-risk / in-flight / uncommitted

**Nothing uncommitted.** Working tree clean at `de951dfd` apart from this record and the ledger
group-placement fix for PJ-259/260/261 (they were named in the v1.80 preamble but omitted from
the five-groups section — my own error, corrected in the same pass that found it).

### (c) Known-broken — open, with a numbered owner

**26 open PJ numbers** in Group 1–2 filed since the sweeps began, plus 11 carried older items.
Full categorisation delivered to the Boss in chat; the ledger's five groups remain authoritative.

### (d) THE HEADLINE RISK — pending, not started, and not individually numbered

**Six whole-app sweep registers hold 177 confirmed findings. Only about two dozen have ever
been given a PJ number.**

| register | confirmed | triaged into PJs? |
|---|---|---|
| `SWEEP-2026-08-09-second` | 32 | largely → PJ-234…PJ-251 |
| `SWEEP-2026-08-10-third` | 37 | largely → PJ-234…PJ-251 |
| `SWEEP-2026-08-10-fourth` | 25 | **headline items only** (PJ-252/254/255) |
| `SWEEP-2026-08-11-fifth` | 29 | **headline items only** |
| `SWEEP-2026-08-11-sixth` | 30 | **headline item only** (PJ-258) |
| `SWEEP-2026-08-11-seventh` | 24 | **headline item only** |

Roughly **100 adversarially-confirmed findings exist only as JSON**, outside the backlog. They
are NOT de-duplicated across runs — the same defect recurs in multiple sweeps, so the true
distinct count is lower and unknown. **Establishing that number is itself the job**, and it is
the completeness net SO #9 exists to protect. Two ledger versions have now deferred it.

### (e) Documentation drift

- The User Manual's PJ-252 paragraph is **English only**; the 14 translations were not swept.
- **PJ-261** — `deriveTabName`'s comment says "for canonical files"; it applies to every file.
- Translated manuals remain partial and drift in vocabulary (carried Group-5 watch).
- Orientation **v3.96** and ledger **v1.80** are current as of this commit.

### Blocked on a Boss ruling — cannot proceed without him

| | needs |
|---|---|
| **PJ-253** | which links a rename rewrites on disk (case-fold) |
| **PJ-207 §13** | gated on **PJ-224** — whether the ordinary search box federates |
| **PJ-260** | mixed line endings in Rust-written frontmatter |
| **PJ-219** | the user-action write class awaits its design ruling |

---

## PJ-234 + PJ-240 — CLOSED, Boss-validated

**Boss ruling that set the sequence:** *"I want us to tackle 1 + 2 as a priority."* — the two
readiness categories (① actively corrupting/losing knowledge, ② notes misplaced or the index
lying). The readiness plan is amended to **M2 → M3 → M1**; PJ-262 and PJ-263 re-sequenced, not
cancelled. My recommendation had been M1 first; recorded once and dropped.

### The defect

`is_block_value_line` = *"a `- item` or an indented line"* is **false for a blank line**. Every
writer that used it to drop a replaced list stopped at the first blank and emitted the remaining
items under the new scalar — a sequence with no key. Unparseable YAML, which is the precondition
for every later property edit on that note being silently discarded.

### Reproduce-first

RED at every site before any fix. First run: `topics: gamma` followed by an orphaned `- beta`.

### FOUR surfaces, not three

The Whole-Ecosystem sweep for the three known sites found a fourth:
`merge_initial_frontmatter` dropped a template's filtered identity key but KEPT its list items,
so a note created **from a template** was born with unparseable frontmatter. It had been flagged
MED and unnumbered in the seventh sweep. Closed in the same pass.

**And the wrong predicate is DELETED.** `is_block_value_line` had zero callers afterwards;
removed from `yaml_lines.rs` with a comment in its place. Five drop loops in Rust, one rule.

### What the gate caught

Three rounds. Two rejections were real:
1. My "what BROKEN looks like" illustration omitted a **blank line** the pre-fix code actually
   emits — the inspector reverted the diff, ran it, and diffed the bytes.
2. My own correction introduced a forward promise ("you'll see the three topics in Step 5") that
   Steps 4–5 make impossible. Mine, not the auditor's.

### PJ-269 — found by the gate, filed not fixed

Verifying the test, the inspector ran the chain and got `{"topics": ""}` for a note holding three
items. Confirmed by reading: `search.rs::parse_frontmatter` skips every sequence item, so
`properties_json` records an empty string for **every** block list (`tags` is special-cased).
Base tables, lens queries and filters all show blank. **A third frontmatter parser** —
`bases.rs::parse_frontmatter` joins the items, `search.rs` does not. Filed as PJ-269; it is a
read/indexer change needing a reindex, so it earns its own pass.

### Gates

Rust **1452/0** (7 new) · vitest **941/0** · binary **14:46** · Boss-validated Stage 1.

### Process slips, recorded

- **I edited the committed v1.80 ledger in place**, which SO#9 forbids. Corrected: v1.80 restored
  to its committed state, v1.81 carries the delta.
- **The binary was stale when I first checked it** — the `ui-inspector` had reverted `bases.rs`
  and `yaml_lines.rs` to run the pre-fix comparison, leaving them newer than the `.exe`. Verified
  the fix survived the restore (1452/0) and rebuilt before sending anything. The standing
  "verify the binary before testing" rule is what caught it.

---

## PJ-235 + PJ-254 — INTERIM, disposition ruled by PANEL at the Boss's direction

**The Boss accepted the `/migration` request and mandated: "the inspectors and auditor choose"
what happens to the interim diff.** Three independent reviewers, same neutral brief, no
preference disclosed. **Verdict 2–1: COMMIT AS INTERIM**, blocking conditions attached.

| reviewer | verdict | decisive point |
|---|---|---|
| A | Option 1 + 4 conditions | the startup rename path is HEAD's largest live hazard; reverting reinstates it |
| B | Option 1 + 4 conditions | no walk boundary was narrowed in the final diff (proved set-equality); carrying 260 uncommitted lines through a migration is the worst option |
| C (sceptic) | Option 2 + carve-out | seven false/contradictory comments; the guard had zero tests; two tails mislabel foreign rows |

**All blocking conditions executed before commit:**
1. Pure `require_own_library_in` + a wiring test that turns RED if the foreign-root check is
   deleted (C's sharpest finding: the earlier tests exercised only the primitives).
2. The FALSE comment corrected at 3 sites — `auto_canonicalize_all` does NOT run at startup
   (no caller anywhere; the startup path is `repair_external_libraries_on_startup`). The
   correction is dated in the comment itself.
3. `invalidate_libraries_cache()` added to `add_child_universe` + `remove_child_universe` —
   without it the foreign set stays empty for the whole session in which a universe is linked.
4. Honest scope: `move_item`'s comment states the SOURCE side is unguarded; the two silent
   `None` reindex arms now `diag_log`.

**Residue filed as PJ-270…PJ-275** (ledger v1.82); PJ-235/PJ-254 marked PARTIAL, the
`/migration` is the closer. Architect input banked:
`docs/migrations/PJ-235-federation-boundary/ARCHITECT-INPUT-federated-write-sites.md` —
a verified enumeration of **22 federated write sites** (21 live, 1 latent), including
`constellation_search_reindex` trusting a frontend-supplied library name at ~20 call sites.

**Tutorial pipeline: REJECTED ×2, APPROVED round 3.** Round 1's rejection was materially
important: the auditor had concluded no reachable linked universe exists — the inspector read
`Eisa Universe/.constellation/universe.json` and found it links BOTH other universes, making
the headline observable demonstrable after all. Round 2 caught the planet-icon placement
(universe row, not library rows), a label case ("New Note"), and a false "no templates"
claim (ECK has 11).

**Gates:** Rust **1456/0** · vitest **941/0** · svelte-check **0 errors** · binary **06:46**
(the 06:05 binary was proven stale by an incremental rebuild — the freshness check earned its
keep again). Boss test SENT; commit gated on his pass.

**Boss test result (2026-08-12): Part A PASS · Part B PASS.** And at the pass, a ruling that
reframes the migration: *"Why can't I move files to/from cUniverses? I would like to be able
to do that."* Filed **PJ-276** — deliberate cross-universe move with full both-sides
bookkeeping — as the migration's headline requirement. The goal is no longer "seal the
boundary"; it is "seal the SILENT crossings, and build a proper door." PJ-270's defect framing
is subsumed into PJ-276's correctness case. Ledger v1.82 carries the ruling verbatim.

---

## BOSS RULING (2026-08-12) — FULL CROSS-UNIVERSE OPERATIONS. The federation contract is REFRAMED.

> "I want to be able to conduct full functions/operations between universes. You have to ask
> yourself, why did I design Constellation to have a cUniverse(s) if I wasn't planning to have
> full access and/or operations among them? If it is kept as-is today, just to (read) and not
> able to (write), then why bother to include other universes (as cUniverses) in the first
> place? That's why Constellation are unique." — Eisa, 2026-08-12

This SUPERSEDES the read-only federation assumption (MIG-065 §J "a write must never be
authorized onto a read-only cUniverse"; the "reads but never writes" phrasing in the interim
guards). It is CONSISTENT with the Boss's own 2026-07-05 ruling "It is ONE universe" (every
name resolver spans federated libraries) — the read-only contract was the implementation's
assumption, never his design.

**What stands:** the interim walls (commit 7921e593) remain correct until each operation is
made safe — the defect was never the crossing, it was the SILENT crossing with broken
bookkeeping (earned link/review data stranded in the source universe's search.db, ghost rows,
wrong-universe attribution). PJ-276 (the move door) is subsumed into the larger goal.

**The migration's goal, final form:** full cross-universe functions/operations — move, and the
rest of the operation surface — each with correct both-sides bookkeeping. The Architect phase
maps the operation classes, the per-universe database model, the earned-data transfer payload,
and the design options, and must also surface which PRIOR RULINGS this direction supersedes so
the Boss can confirm the repeals explicitly (incl. its interaction with the pending PJ-224
search-federation ruling).

---

## MIG-111 — Architect phase CLOSED (2026-08-12)

Allocated MIG-111 (109/110 already taken by Search-Aerial and Tabs-in-Every-View concepts).
12-agent mapping workflow: 6 subsystem maps, 3 independent design options, 3 adversarial
attacks — 1.8M tokens, every claim file:line-verified. Deliverables:

- `docs/migrations/PJ-235-federation-boundary/MIG-111-ARCHITECT.md` — the Phase-1 document:
  concept (Boss ruling verbatim) → territory → options+verdicts → Option A's 7 blocking
  conditions → invariants → the prior-rulings reconciliation table → 5 Phase-2 decision points.
- `MIG-111-ARCHITECT-EVIDENCE.md` (196 KB) — the full maps/options/attacks, durable.

Verdict: **Option A (route-to-owner) recommended, viable-with-conditions.** C proved
structurally impossible on FTS5 grounds (BM25 statistics are index-global — one universe's
registration would change another's ranking). B carries two app-killer cross-process holes.

The attack passes also surfaced that three PRE-EXISTING hazards become critical under any
write-capable federation: link_life's process-local ledger lock, the WAL false-negative
`is_cuniverse_open_elsewhere` probe, and federation/migrate.rs's fs::copy backup/restore.
All scheduled as Plan conditions.

**Awaiting the Boss's five Phase-2 decisions (ARCHITECT.md §6). No build until Plan approval.**

---

## BOSS RULING (2026-08-12, second) — THE CONCEPT IS NAMED: UNIVERSE OF UNIVERSES

> "When I designed Constellation to include a child Universe (Linked Universe), it was the
> concept and philosophy of a Universe of Universes, if the user would like to have all their
> knowledge under one umbrella. Accordingly, there should be full interactions among those
> linked universes, seemingly [seamlessly]. Meaning, we are NOT going to create an
> old-fashioned ad hoc system. You should think, design, and build it out of the box. You have
> to be creative and smart. What matters eventually is that you achieve what others think is
> impossible to achieve, design-, programming-, or coding-wise. You should uphold
> Constellation's philosophy: simple yet powerful." — Eisa, 2026-08-12

Design consequences adopted:
- **Seamless, not border-control.** The Phase-2 §6 decision 2 option set (marked section +
  confirmation vs separate command) is WITHDRAWN as framed — both were doors. The umbrella is
  one space; pickers list it whole, the planet mark is identity information, not a warning;
  no confirmation ceremony on routine operations.
- **Not ad-hoc = one choke point.** The design centrepiece is the UNIVERSE ROUTER: one layer
  under every operation that resolves ownership and supplies the home universe's full context
  (DB handle, link vocabulary, locks). The 22-site register is not 22 guards — it is 22
  callers of one router. This also closes the panel's structural finding that
  reindex_single_note trusts its callers.
- §6 decisions resolved by this ruling: (1) repeals CONFIRMED by the ruling's own terms;
  (2) seamless as above; (4) two-instance refusal stays (invisible safety machinery);
  (5) PJ-224 read as YES — one umbrella means search spans it by default — flagged for veto
  rather than assumed silently. (3) wave order goes to the Plan with a routing-first proposal.

---

## MIG-111 — CONCEPT PANEL (Boss-mandated) — VALIDATED 5/5

The Boss: "Let the inspectors, auditors, UX & UI, plus the Art Director & team, to validate
this concept." Five chairs, independent, blind to each other; synthesis merged.

**VERDICT: THE CONCEPT STANDS — VALIDATED-WITH-REQUIREMENTS, 5/5 chairs, 0 rejections.**
37 binding requirements (R1–R37), 3 chair conflicts requiring Boss rulings (C1 ceremonies,
C2 locked-universe presentation, C3 tab identity). Banked in full at
docs/migrations/PJ-235-federation-boundary/MIG-111-CONCEPT-PANEL.md.

The finding that reframes everything, from the SAFETY chair of all places: **"the status quo
is not the safe baseline — it is the most dangerous state."** The 22-site register shows
today's "read-only" federation already crosses with broken bookkeeping; rejecting the concept
would preserve a documented silent-failure surface. And the philosopher chair produced the
uniqueness line the Plan will carry: **"sovereignty with seamlessness: each corpus keeps its
own truth; the mind over them is one."**

Hard pre-conditions before any door opens: R1 (five unguarded writers on-boundary),
R3 (vocabulary threading proven red→green), R5 (a real OS lock — the WAL probe certified
insufficient), R7 (every crash window red→green), R11 (live-WAL fs::copy banned, fixed FIRST),
R35 (the PJ-262 sequencing question to the Boss), R36 (repeals ratified item by item).

Prior-art (WA#5, completed in the parallel workflow): **no shipping PKM product does this.**
Obsidian: cross-vault links "will remain impossible"; Notion: cross-workspace move is a lossy
copy that breaks links and history; DEVONthink comes closest (UUID item links, metadata
travels) but name-links cannot cross databases and clones silently become copies; Tana
federates search/references but schemas do not travel and moves lose supertag context.
"Universe of Universes with full seamless operations" is genuinely unprecedented in the field.

---

## MIG-111 — PLAN DELIVERED for Boss approval (2026-08-12)

Prior-art (WA#5): uniqueness CONFIRMED — no shipping PKM product federates independent bases
with full seamless cross-base operations. Engineering prior-art: the Router assembles proven
parts (SQLite backup API, OS lock files, IMAP-MOVE two-phase semantics) in an unprecedented
shape. Two adversarial attacks on the drafts: SOUND-WITH-AMENDMENTS ×2; all amendments folded
into MIG-111-PLAN.md (H1 vocabulary-maintenance values, H2 fail-closed owner resolution,
H3 longest-match fast path, H4 readable lock metadata, B2 refusal-before-input, B3 prep off
the save tail, B4 routing-before-doors).

Plan: Phase 0 foundations (R11 fs::copy ban FIRST · R5 real owner lock · ledger lock · R1
five writers on-boundary · the §0.5 Boss gate of seven rulings) → Phase 1 the Router
(seamless linked-note editing; the H1 harness; Class-D kill; the Place Line; whole-ecosystem
identity) → Phase 2 the transfer engine (journaled two-phase move; cid re-key; genuine Undo)
→ Phase 3 full ops + cross-universe link healing (the migration's acceptance test) → Phase 4
the diagnostic umbrella + repeals executed. Every phase carries its R-numbers; measurement +
Boss-journey gates per wave (R33/R34).

Ledger v1.84. Awaiting: Plan approval + C1/C2/C3/R31/R35/R36/PJ-224.

---

## MIG-111 §0.5 — SEVEN RULINGS TAKEN (2026-08-12). PLAN APPROVED. BUILD BEGINS.

1 Plan approved · 2 C1 Undo-not-ceremony · 3 C2 persistent quiet line · 4 C3 delegated to the
Art Director & team · 5 "Linked Universe" adopted (PJ-277 filed for the renaming sweep;
user-visible scope; code identifiers excluded, recorded not silent) · 6 R35 YES — PJ-262
ships before Phase 2, inside MIG-111's sequence · 7 R36 details requested → delivered in
chat, ratification pending · 8 PJ-224 RULED YES — the search box federates; PJ-207 §13
un-gated, folds into Phase 4.

Phase 0.1 (live-WAL fs::copy ban) begins now.

---

## BOSS RULING (2026-08-12) — ITEM 7 RESOLVED AS "LINK MODE": THE USER CHOOSES

> "Let the user choose between reading only or reading and writing. Either way, it should be
> implementable." — Eisa, 2026-08-12

The repeals table is RATIFIED-AS-AMENDED by this ruling. The amendment:

- ① MIG-056's read-only contract is NOT repealed — it is **demoted from contract to MODE**.
  Every linked universe carries a user-chosen mode: **Read only** (today's behaviour, kept
  first-class) or **Read & write** (the Universe-of-Universes full-operations behaviour).
- ④ The interim guards become PERMANENT for read-only-mode links (with honest wording:
  "linked read-only — you can change this in the Universe Manager"), and dissolve per-door
  only for read-write links.
- ② concept paper describes both modes; ③ (automatic writes never cross on their own) is
  unchanged and additionally enforces read-only mode; ⑤–⑧ unchanged; ⑨ PJ-224 unaffected
  (search is READ — it federates in both modes).

Design consequences (into MIG-111-PLAN.md):
- `resolve_owner` returns the link MODE with ownership; the Router refuses writes to
  read-only-mode links with the user's own choice quoted back.
- The federation manifest's `children` entries gain a mode (schema evolution from bare path
  strings). **Existing links default to Read only** — conservative, preserves today's
  behaviour; the user upgrades deliberately. **New links ask at link time** — the consent
  moment (Tana's "allow content from" precedent, from the prior-art sweep).
- Universe Manager shows and toggles the mode per link.
- Both modes are first-class forever ("either way, it should be implementable") — read-only
  is never a stub or a deprecation path.

---

## MIG-111 Phase 0.2 — COMMITTED: the per-universe OWNER LOCK (R5)

New module `universe_lock.rs`: OS file lock (`LockFileEx`/`flock` via fs4) held for the whole
active session — sees an IDLE holder (the retired probe's certified false negative) and dies
with the process (stale locks impossible by construction). Two-file shape per attack H4: the
zero-byte `owner.lock` is the truth; `owner.info.json` (never locked) supplies the WHO for
refusal messages. Identity = canonicalized root. Wired at all five activation sites;
`is_cuniverse_open_elsewhere` now consults the owner lock first, keeping the SQLite probe
only for non-Constellation tools. Phase 0 policy: RECORD, don't enforce — Phase 1.4 flips it.

Verification per the Plan's clause: the TWO-PROCESS test — a real spawned child holds the
lock idle; the old probe reports not-held (pinned RED forever), the owner lock reports held
with the child's pid; child exit releases. Rust **1462/0** (4 new). Inspection: ninth
whole-app register banked (33 confirmed — ZERO in this diff; two pre-existing
migrate_legacy_data findings join the triage pile). vitest/svelte-check unaffected (no
frontend files in the diff).
