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
