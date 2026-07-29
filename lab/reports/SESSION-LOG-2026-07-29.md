# Session Log — 2026-07-29 (session B)

Branch `main`, opened at `e4e793be` (clean). Continuing from the 2026-07-29 session-close
handover. **► Next action per PJ ledger v1.56: PJ-182.**

---

## §1 — PJ-182 verification (Reproduce-First) — COMPLETE

**Function in hand:** the frontmatter parser in the Properties write path —
`parseFrontmatter` / `composeFrontmatter` (`src/lib/libraries/store.ts`,
`src/lib/editor/yamlDoc.ts`), the projection that feeds both Properties panels and every
property write.

### §1.1 The filed defect is REAL, and reproduced by running the code

Evidence: `lab/reports/pj182-observation.txt` (JS), `…-observation2.txt` (adjacent shapes),
`…-observation3.txt` (post-rename validity). Probe:

```
---
cid_cn: ABCD
title: Imported Probe
tags:
- alpha
- beta
aliases:
- Old Name
- Older Name
stage: spark-seed
---
```

| Observation | Result |
|---|---|
| `store.parseFrontmatter` | `tags` → `{value:'', type:'list', listItems:[]}` — items **absent** |
| `yamlDoc.parseFrontmatterDoc` | `tags` → `['alpha','beta']` — **correct** |
| `buildFullContent` (tab.content cache) | emits `tags: ` / `aliases: ` — **both lists deleted** |
| add a tag, OPEN note | `tags:\n- alpha\n- beta` → `tags:\n  - gamma` — **alpha, beta gone** |
| add a tag, CLOSED note (`composeUpdatedContent`) | identical loss |
| re-parse of the damaged file | clean; `aliases` still empty → **still armed** |

### §1.2 TWO CORRECTIONS to the filed description

Both established by running the code, and both change how the bug must be described:

1. **The loss fires on a write to THAT KEY, not on "the next property write."** Editing a
   *neighbouring* key is safe: both sides of the compose diff project the list identically,
   so no diff is produced and the CST is left byte-perfect. The handover's "the next
   property write replaces the whole block" is too broad.
2. **The root is not "three regexes"; it is that Constellation has TWO frontmatter parsers
   and they disagree.** The hand-rolled one (`store.ts`) feeds the visible panel and the
   note model; the spec-compliant one (`yamlDoc.ts`, eemeli `yaml`) feeds the writer. The
   CST parser gets this input right. This is the same "two parsers, one rule, applied in
   only one of them" that `store.ts:1973` already names in-code for PJ-136.

### §1.3 The YAML rule (WA#5 cross-check)

A block sequence may be indented at the **same level as its parent mapping key** — valid
YAML 1.2. Verified empirically against the `yaml` library Constellation itself ships and
writes through (`parseDocument` → `Seq` of scalars, 0 errors). PyYAML is not installed on
this machine, so the handover's "what PyYAML emits" claim is **not** independently verified
here and is not repeated as fact.

---

## §2 — THE WHOLE-ECOSYSTEM SWEEP (the law, applied) — the scope is 20 surfaces, not one

Per the Whole-Ecosystem Fix Law ("for a broad concern, spawn a parallel audit that finds
every surface exhaustively — do not enumerate from memory"), a 38-agent workflow swept five
modalities and then **adversarially refuted every candidate**. 33 verdicts returned;
**13 refuted, 20 confirmed**. Full register with per-finding evidence:
`lab/reports/pj182-confirmed-surfaces.md`.

**The single shared root cause:** eight independent sites answer *"is this line a
continuation of the previous key's block sequence?"* from **leading whitespace**, when
YAML's actual rule is *"the trimmed line begins with `-` followed by space or end-of-line."*
A mapping key can never begin with a dash. `search.rs::parse_frontmatter` has carried the
correct rule all along — in a comment that states it explicitly — and the nine surfaces that
handle this input correctly all use the trimmed-line test.

### §2.1 Confirmed — frontend (the READER is the broken half)

| Sev | Site | Defect |
|---|---|---|
| APP-KILLER | `store.ts:2009` | block-list branch requires indent — **the filed PJ-182** |
| APP-KILLER | `store.ts:1900` | ikhtilāf branch requires indent |
| APP-KILLER | `store.ts:1981` | nested-map branch requires indent; also swallows an indented **flow sequence** (`tags:\n  [a, b]`) |
| APP-KILLER | `store.ts:2048` | a **YAML comment** inside a block list demotes it to read-only |
| APP-KILLER | `store.ts:2087` | **block scalars** (`\|`, `>`, `\|-`) project as editable text whose value is the indicator char — editing destroys the block |
| HIGH | `store.ts:1882` | `- name: X` at column 0 is admitted as a **top-level key named `- name`** |
| APP-KILLER | `yamlDoc.ts:183` | the `STRUCTURED_LIST_KEYS` exemption removes the immutable-block protection for ikhtilāf |
| APP-KILLER | `noteModel.ts:157` | inherits the broken projection into the model (`props` + `base`) |
| APP-KILLER | `PropertyEditor.svelte:756` | `addTag` — the trigger |
| APP-KILLER | `PropertyEditor.svelte:795` | `addNestedRow` — "+ Add school" is reachable with zero rows shown |
| MED | `PropertyEditor.svelte:973` | `buildFullContent` writes the emptied lists into `tab.content` |

### §2.2 Confirmed — Rust (the READERS are fine; the WRITERS corrupt)

The index readers (`search.rs::parse_frontmatter`, `extract_aliases`) agree with the CST
parser and return the right items. So disk and index are correct **until a writer touches
the note** — at which point the `.md` becomes unparseable YAML and stays that way.

| Sev | Site | Defect |
|---|---|---|
| APP-KILLER | `libraries.rs:1767` `update_frontmatter_title` | **the rename cascade.** Renaming a note with a zero-indent `aliases:` list writes **invalid YAML** to disk |
| APP-KILLER | `bases.rs:541` `update_frontmatter_property` | a **Bases table cell edit** on a list column orphans the items → invalid YAML |
| APP-KILLER | `canonical.rs:292/299` `merge_frontmatter` | appends with a hardcoded 2-space indent → the last existing alias becomes `Older Name - "Injected"` |
| HIGH | `libraries.rs:1887` `set_frontmatter_parent` | PJ-065 "resolve contested parent" orphans the items. Its own comment declares this exact class fixed as an APP-KILLER on 2026-07-22 — **the fix only covered indented lists** |
| HIGH | `libraries.rs:899` `merge_initial_frontmatter` | same predicate |
| HIGH | `search.rs:5642` `extract_frontmatter_typed_links` | **typed links written as a zero-indent block list never reach `note_links`** — the Living Link graph silently loses those edges |
| MED | `link_types.rs:397` `structural_frontmatter_targets` | the PJ-065 structural-link guard **fails OPEN**, so a structural link counts as cognitive |
| MED/LOW | `bases.rs:471` `remove_frontmatter_property` | same predicate; latent only because shipped callers pass scalar keys |

### §2.3 Why 678 tests and 1,261 Rust tests are green

**Every in-repo fixture for every one of these sites uses two-space indentation.** Not one
test in either language exercises a zero-indent block sequence. Confirmed examples:
`bases.rs:852` (`tags:\n  - one`), `libraries.rs:7130` (`aliases:\n  - A`),
`search.rs:5723` (`supports:\n  - "[[Alpha]]"`), and every `tests/mig-103` and `tests/g4`
fixture. This is LL-037 rule 4 again — *the suite proves what it exercises.*

---

## §3 — REPRODUCTIONS LANDED (RED, both languages)

| File | State |
|---|---|
| `tests/pj-182/zeroIndentBlockList.test.ts` | **10 failed / 5 passed** — the 5 passing are the regression controls (indented forms + neighbour-edit safety), exactly as designed |
| `src-tauri/src/libraries.rs` `pj182_rename_keeps_a_zero_indent_alias_block_valid` | **RED** — "mixed indentation inside one block sequence is invalid YAML" |

No production code has been changed. `git diff` is the Rust test only.

---

## §4 — STATE OF STANDING (SO#5) — written because the scope changed materially

**(a) Verified-shipped & protected.** Everything in the 2026-07-29 handover §"State of
standing (a)": MIG-104 Slices 0–7 · PJ-174 AK-1/2/3 · MIG-107 §0–6 · PJ-178/179. Untouched
this session.

**(b) At-risk / in-flight.** Working tree carries ONLY: the two RED reproductions, the three
observation reports, and the confirmed-surface register. `PROPS_SINGLE_OWNERSHIP` retained
per Boss ruling.

**(c) Known-broken — CHANGED.** PJ-182 as filed was "small, proven, one function." It is in
fact **20 confirmed surfaces across two languages**, including **six APP-KILLERs not
previously known**, three of which convert a note's frontmatter into **unparseable YAML** on
ordinary gestures (rename a note · edit a Bases cell · resolve a contested parent), after
which every future property edit on that note is silently discarded forever.

**(d) Pending, not started.** PJ-181 · MIG-104 Slices 8–15 · MIG-105 Phase 2 · PJ-180 · the
rest of the v1.56 ledger.

**(e) Process.** PJ-166 did **not** strike this time — the sweep was deliberately authored as
a bespoke workflow rather than invoked as the `safety-inspection` skill, so the diff-scope
argument never arose. The per-build gate PJ-166 asks for still does not exist.

---

## §5 — THE SCOPE QUESTION PUT TO THE BOSS

`CLAUDE.md`'s Migration Rule: *"Any change that touches schema, core data flow, cross-surface
invariants, or multiple subsystems goes through the four-phase `/migration` workflow before
any code is written."* This change crosses Rust ↔ Svelte, read path ↔ write path, and touches
the rename cascade, Bases, canonicalization, the importer, the Living Link graph and both
Properties panels. By the letter of the rule it is a `/migration`.

Against that: it is one **reproduced, actively-destroying** content-loss bug, and the fix at
every site is the *same one-line predicate* — not an architecture change.

Ruling requested before any production line is written. Options put to the Boss:
1. **`/migration` (MIG-108), full four phases** — Architect → Plan → Build in slices → Audit.
2. **Fix the whole ecosystem now**, migration-grade discipline without the ceremony, one
   shared predicate + shared block-walker, landed in slices with a Boss test per slice.
3. **Fix the zero-indent class only**, and file the three *other* shapes found
   (block scalars · flow sequences · comments-in-block) as their own PJ entries.

### ► BOSS RULING (2026-07-29): **option 2 — fix the whole thing now**, in the five slices.

---

## §6 — THE BUILD

### §6.1 Slice 1 — the JS reader + the props contract

**`src/lib/libraries/store.ts`**
* New shared predicate **`isYamlSeqItem`** (exported) — *a line beginning with a dash is a
  sequence item at ANY indentation; a mapping key can never begin with one.* Plus
  `isYamlBlockChild`, `isYamlComment`, `isBlankLine`, `noCr`.
* The three divergent branch probes (`/^\s+-\s/`, `/^\s/`, `/^\s+-\s/`) are **replaced by
  ONE block-extent scan followed by one classification** — ikhtilāf / flat list /
  read-only. Three probes of one truth are three chances to disagree (LL-038 rule 5).
* The top-level-key guard now excludes sequence items, killing the phantom `- name` rows.
* A comment among block content demotes the block to read-only (matching the indented
  form); a run of comments ALONE no longer forms a block.

**The props contract** — `noteModel.ts` · `propsCommit.ts` · `noteSession.ts`
* Found in-pass (WA#6) and **indentation-independent**: `nestedObjects` — where a
  `nested-object-list`'s content actually lives — was compared by nothing, carried by no
  intent, and spread stale by `setPropValue`. **Every ikhtilāf edit was a silent no-op.**
* New **`sameNested`** beside `sameList`; `PropOp.set`, `IntentSink.setValue`,
  `editPropValue` and `setPropValue` all carry the rows; `touchedSince` and `plan` compare
  them.

**Verification.** `tests/pj-182/zeroIndentBlockList.test.ts` (15) +
`nestedObjectsIntent.test.ts` (7). RED proven **by removing the fix** (10 failed / 5 passed,
then 4 failed on the nested-objects half) — the tests that stayed green under removal are
exactly the indented controls, which is the intended result.

### §6.2 Slices 2 + 3 — the Rust writers and the Living Link graph

**New module `src-tauri/src/yaml_lines.rs`** — the Rust twin of the JS predicate, and the
single home for the rule: `is_seq_item` · `indent_of` · `is_top_level_key_line`.

| Site | Fix |
|---|---|
| `libraries.rs` `update_frontmatter_title` | dash-based item test **+ the appended alias now uses the block's OWN indent**. Also `t[1..]` instead of `t[2..]`, which would have PANICKED on a bare `-` |
| `libraries.rs` `set_frontmatter_parent` | dash-based item test — the 2026-07-22 APP-KILLER fix had only covered indented lists |
| `libraries.rs` `merge_initial_frontmatter` | `is_top_level_key_line` — a filtered key's zero-indent items no longer survive as an orphan sequence in a brand-new note |
| `bases.rs` `update_frontmatter_property` | `is_top_level_key_line` + dash-based skip — a Bases cell edit no longer orphans the old items |
| `bases.rs` `remove_frontmatter_property` | same |
| `canonical.rs` `merge_frontmatter` | appends at the block's own indent — no more `Older Name - "Injected"` |
| `search.rs` `extract_frontmatter_typed_links` | a column-0 item is a CONTINUATION — zero-indent typed links reach `note_links` again |
| `link_types.rs` `structural_frontmatter_targets` | same — the structural guard no longer fails open |

**Verification.** 10 new Rust tests. RED proven **by reverting the shared predicate to its
old indentation-based form: 8 failed, 2 passed — and the 2 that passed are precisely the
two indented controls.** (A first RED attempt via a scripted edit silently failed to apply
and reported all-green; caught by diffing the file. LL-037 rule 3, in practice.)

### §6.3 Slice 4 — the other three shapes

* **Block scalars** (`|`, `>`, `|-`, `>2`, …) — were projected as editable text valued
  `"|"`, and `buildFullContent` wrote `desc: "|"`, deleting the prose from the cache. Now
  read-only with bytes verbatim; `reconstructFrontmatter` re-emits the indicator.
* **Flow sequence on the next line** (`tags:` then `  [a, b]`) — was read-only, so the user
  could not edit their own tags while the CST parser read them fine. Now an editable list.
* **Comments in a block** — closed in Slice 1.

`tests/g4/composeUpdated.test.ts` had a case asserting the OLD loss as proof
(`expect(legacy).not.toContain('first line')`). That loss is now fixed at the source, so
the assertion was pinning a defect in place; it is rewritten to assert what is still true —
`buildFullContent` re-quotes and is therefore not byte-perfect, which is the actual reason
`composeUpdatedContent` exists. (Observed, not assumed: only the quoting style now differs.)

### §6.4 Gates

| Gate | Result |
|---|---|
| vitest | **62 files / 715 tests** (was 59/678) |
| Rust `cargo test --lib` | **1275 / 0** (was 1261), 11 ignored |
| svelte-check | **0 errors** (268 pre-existing CSS warnings) |
| Sight perf, SERIAL lane (PJ-172) | **31 / 31** |
| `tests/g4/frontmatterRoundtrip.test.ts` (permanently RED by design) | **unchanged** — 4 failed / 1 passed |

### §6.5 Slice 5a — `/simplify`, and the THREE defects it found in the fix itself

Four parallel reviewers (reuse · simplification · efficiency · altitude). The efficiency
review returned **no material regression** and explicitly recommended changing nothing
(the new regexes are literals, V8 compiles them once; `is_seq_item` is allocation-free and
short-circuited, ~2–3 ms across a full 7,600-note reindex). The other three found real
work, including **three defects introduced or left open by this very change** — all three
reproduced by running before being fixed:

1. **A block scalar was outside `immutableBlockKeys`** (`yamlDoc.ts`). Slice 4 gave it the
   read-only WIDGET; the half that actually protects bytes is the composer refusing the
   key, and its test was `isMap || seq-of-non-scalars`. A block scalar is `isScalar`
   (`BLOCK_LITERAL` / `BLOCK_FOLDED`), so it was in neither set. **Observed: a props array
   that merely OMITTED the row deleted `desc: |` and both prose lines from the file.**
2. **`nestedObjects` was threaded through three layers and stopped at the fourth.**
   `touchedSince`, `plan` and `setPropValue` all learned to carry the rows;
   `composeFrontmatter`'s own unchanged-check still decided from `value` — the display
   summary. **Observed: delete a row from an ikhtilāf block without the summary changing
   and the write is dropped.** A chain is exactly as strong as its last link.
3. **The Rust twin had no comment concept** (`bases.rs`). Widening the continuation-skip to
   a dash test meant a `#` line among the items ended the skip, and every item after it was
   emitted beneath the new scalar — orphaned, unparseable. The JS twin had always folded
   comments into the block; **only the Rust half was wrong.** LL-038 rule 4 exactly:
   widening a guard is a behaviour change for everything it drops. A seq-of-map's indented
   continuation line (`role: Y`) had the same hole.

**And the Whole-Ecosystem Fix Law caught me again.** The reuse review found **eight more
sites** of the same concern still hand-rolling `starts_with("- ")` — including
`search.rs::parse_frontmatter`, *the function whose comment the new module quotes as the
origin of the rule*, and the direct siblings of two functions that WERE routed
(`canonical.rs::remove_canonical_fields`, `libraries.rs::remove_frontmatter_contains_item`,
`sources/mod.rs` ×2, `bases.rs`'s own reader). All are now routed.

Also applied: `seq_item_value` / `yamlSeqItemValue` (a predicate without its extractor is a
trap — this change had already hand-patched a `t[2..]` panic because of it) · a
Unicode-consistent `is_indented` (the module shipped `is_seq_item` trimming Unicode
whitespace and `is_top_level_key_line` testing ASCII only, which would have re-opened the
2026-07-21 indented-`title:` shape for NBSP) · one `SetPropOpts` type instead of four hand-
widened copies · one `samePropRow` instead of five spellings of row equality (`propRow.ts`,
a leaf the composer can import without a cycle) · `blockExtent` · `unquote` · and the loop
now advances its cursor FIRST so no branch needs an `i = end - 1` off-by-one.

**Deferred and filed, not silently parked:** the Rust block-WALKER that would collapse the
eleven remaining per-site state machines; giving a block scalar its own `PropertyType` (it
currently borrows `nested-map` and is labelled "Nested map" to the user in 15 languages);
and PJ-137 — retiring the hand-rolled parser for the CST — which PJ-182 makes **strike six**.

**Gates after `/simplify`:** vitest **62 files / 716** · Rust **1277 / 0** · svelte-check
**0 errors** · Sight perf SERIAL **31 / 31**.

### §6.6 Slice 5b — the per-build safety inspection

Invoked **diff-scoped** with `args.files` (13 files). It returned `mode: "whole-app"`.
**PJ-166's EIGHTH strike** — 88 agents, ~10.8 M tokens, ~30 minutes, for a gate that was
asked to look at 13 files. Register: `lab/reports/SAFETY-INSPECTION-2026-07-29-pj182.md`.

**52 confirmed / 50 unique sites — 5 APP-KILLER · 10 HIGH · 29 MED · 7 LOW.** Triaged:

**Caused by this change → FIXED before the commit.** `sources/mod.rs` — *"the PJ-182 pass
routed only the block-skip half through `is_seq_item` and never routed the key-match half."*
The key match ran on the TRIMMED line, so an **indented** `sources:` — a nested map's key, or
a line of prose inside a block scalar — was matched as the note's own key and **deleted**.
Fixed at all four sites (both writers AND both readers, so they agree), RED-proven by
removing the guard. **This is the Whole-Ecosystem Fix Law catching me for the second time
inside the change made to obey it** — first the eight un-routed sites `/simplify` found, now
a routed site where I converted one of its two halves.

**Pre-existing, in files this change touched → filed, not fixed here.**
`yamlDoc.ts:311` (APP-KILLER — `composeFrontmatter`'s H1 branch silently discards *every*
property edit on a note whose YAML is malformed, and reports the save as successful;
`hasErrors` has zero consumers outside `yamlDoc.ts`), `yamlDoc.ts:362` (the CST
splice+append deletes YAML comments attached to the edited key — and the SET guard excludes
every list prop, so **adding a tag always takes that path**), `PropertyEditor.svelte:430`.

**The other 46 sites** are the whole-app register, unrelated to PJ-182 — headline
**APP-KILLER `PropertyEditor.svelte:974`**: the right-sidebar Properties panel is never
`{#key}`-remounted, so a pending 800 ms debounce that survives an in-place navigation writes
note A's properties onto note B, durably and silently. Filed as **PJ-187** (the register).

**Gates after the inspection fix:** vitest **62 files / 716** · Rust **1280 / 0** ·
svelte-check **0 errors** · Sight perf SERIAL **31 / 31**.

### §7 — PJ-181 (APP-KILLER): a merely-VIEWED note overwriting a newer external edit

**Function in hand:** the write-ahead recovery net — its restore on note open
(`store.ts::resolveNoteContent`) and its arbitration against the bytes just read from disk.

**SO#8 start-guard:** entry cross-checked against orientation v3.75 §preamble and the
2026-07-27/29 session logs. Fresh, never worked, names the function correctly.

#### §7.1 Reproduced, and the first reproduction was WRONG

Recipe V: view a note (type nothing) → close → the file is edited outside Constellation →
reopen. Driven through the REAL `openNoteTab` against a mocked IPC bridge (the
`tests/mig-076/reopenRecoveryClobber.test.ts` harness pattern).

**The first run of the second assertion PASSED — for a harness bug.** `flushIfDirty(id, ENV,
origin)` takes the save ENV as its second argument and I passed the path string, so `e.write`
was undefined, the save returned `write_failed`, and the test went green while the defect was
fully live. Caught by instrumenting rather than trusting it. **Measured with a real env:**

```
disk before flush : "...the original body\n\nEXTERNAL WORK"
flush result      : {"ok":true}                ← reported SUCCESS
disk after flush  : "...the original body"     ← the external edit is GONE
```

That is LL-037 rule 3 in miniature, inside the very session that wrote LL-039.

#### §7.2 The root — the net recorded WHAT it held and never WHY

The entry was `{ content, cursorPos, scrollTop }`. `resolveNoteContent` was asked to tell a
genuine recovery copy from a stale snapshot using information that was never written down,
and answered with `cid_cn` — the note's **identity**, which says nothing about its **version**.
Every check passed for an externally-edited note, so the stale view won the screen, the model
was born DIRTY (`markModelRecoveredFromNet`), and the first departure wrote it over the file.

Three premises, each verified by reading before any design:
1. `NoteEditor.svelte` `!needsDiskSave` branch stashes an entry even when nothing changed
   (`needsDiskSave` is NotePane's own dirty flag) — merely viewing leaves one;
2. nothing clears it for a CLOSED note;
3. the manual-open path marks the model dirty from it, unlike `restoreSessionTabs`, which
   passes `preserveNet` and seeds the TRUE disk baseline so a restored tab is born clean.

#### §7.3 The fix — the entry now says WHY it exists

`WriteAheadEntry` gains an optional `snapshot?: boolean`; `resolveNoteContent` rejects the net
when `wab.snapshot === true && diskContent !== wab.content`, taking the same path as the two
pre-existing rejections. Absent flag (every legacy localStorage entry) → treated as real work,
i.e. pre-PJ-181 behaviour, the direction that never discards the user's edits.

**A flag, not a copy of the baseline bytes — and that was a correction to my own first
version.** I first stored the baseline content itself. Then checked the store: the net's
localStorage blob is **never pruned and never capped**, and `setWriteAhead` swallows a quota
exception with an empty `catch`. Storing the baseline would have put a SECOND FULL COPY of
every viewed note's content in that blob — doubling the growth of an unbounded store whose
failure mode is silently ceasing to persist, which is the crash recovery it exists to provide.
For a snapshot the baseline IS the content, so a boolean carries the same information at no
size cost. *(The unbounded, uncapped, silently-failing WAB blob is pre-existing and now filed.)*

**LL-039 applied to this fix:** every `setWriteAhead` call site swept with no `grep -v` —
three live sites. Only the snapshot stash gets the flag; the two save-path stashes
(`NoteEditor`'s legacy branch and `standardSaveEnv.setNet`) hold work not yet on disk, where
omitting it is correct. `getWriteAhead` is read by exactly one consumer.

#### §7.4 Verification

RED-proven twice — once for the baseline version, once after the refactor to the flag — by
neutering `staleSnapshot`: **2 failed / 3 passed**, and the 3 that hold are the controls,
including the one that matters most: **Recipe S (PJ-102), where a genuine failed-save recovery
copy must still win and still be born dirty.** A fix that merely "prefers disk" fails that
control, and that failure would be the user's unsaved work.

| Gate | Result |
|---|---|
| vitest | **63 files / 721 tests** (was 62/716) |
| svelte-check | **0 errors** |
| Sight perf, SERIAL lane | **31 / 31** |
| Rust | untouched — frontend-only change |

**Boss-validated on the live Universe:** view → close → external append via the shell → reopen
shows BOTH paragraphs → tab switch and back → the externally-added paragraph is still on disk.

#### §7.5 The build's own inspection found an APP-KILLER **in this fix** — measured, before it shipped

A focused adversarial hunt over the PJ-181 diff (four lenses, every candidate refuted before
confirmation) returned one confirmed APP-KILLER, and it broke the very invariant the fix
existed to protect.

**`needsDiskSave` does not mean what I wrote in the comment.** It is NotePane's view-level
`dirty`, and `doSave()` clears it at **save-REQUEST** time (`NotePane.svelte:340`) — before the
write is attempted — and never restores it on failure. Its three assignments are: init false,
cleared on request, true on `docChanged`. So `!needsDiskSave` is ALSO true while a **failed or
in-flight** save's only copy is still unwritten.

The first version of the fix hard-coded `snapshot: true` in that branch. Consequence: after a
failed save (the documented `.md`-locked case) **any** teardown — a tab switch, the app-close
`beforeunload`, switching to Focus — re-stashed the user's ONLY copy flagged *already durable*,
and `resolveNoteContent`'s new branch then rejected it and **cleared it**. The fix would have
deleted precisely what the net exists to protect, with no error anywhere.

**My error, precisely:** I verified WHERE `needsDiskSave` came from and never checked what it
MEANT — I read the name and the neighbouring comment and inferred "durable". That is the
No-Guessing law, broken in the session that wrote LL-039 about this exact habit.

**Fixed:** the flag is now derived from the MODEL, which tracks durability (`markSaved` trails
the durable write) — `SINGLE_OWNERSHIP && !isModelDirty(tab.id)`. Under `SINGLE_OWNERSHIP=false`
there is no model, so the flag stays false = "real work" = pre-PJ-181 behaviour.

#### §7.6 Two of my three new tests were worthless when written

1. The Recipe-V flush case **passed for a harness bug** (a path string where a save ENV was
   expected → `e.write` undefined → `write_failed`), while the defect was fully live.
2. The new failed-save control **passed with the flag hard-coded to `true`** — it round-tripped
   through a reopen, and the round-trip re-stashed an UNFLAGGED entry that masked the flag
   entirely. It proved nothing about the thing it was written to prove.

The second is the sharper lesson and is now **LL-040**: *a test that exercises the CONSEQUENCE
cannot pin a DECISION whose whole purpose is that the consequence is ambiguous.* Recipe V and
the failed-save case are mechanically identical downstream — a snapshot-flagged entry differing
from disk — and differ only in which side is newer, which is exactly what the downstream code
cannot know. The surviving test asserts the **predicate at the point of decision**, and is
proven load-bearing: it fails against the old hard-coded flag and passes against the new one.

#### §7.7 A third correction — the localStorage blob

The first fix stored the baseline BYTES on each entry. Checking the store before committing:
the net's localStorage blob is **never pruned, never capped**, and `setWriteAhead` swallows a
quota exception with an empty `catch`. Storing the baseline would have put a SECOND FULL COPY
of every viewed note into that blob — doubling the growth of an unbounded store whose failure
mode is silently ceasing to persist, i.e. the crash recovery it exists to provide. Replaced
with a boolean: for a snapshot the baseline IS the content, so the flag carries the same
information at no size cost. *(The unbounded/uncapped/silently-failing WAB blob is
pre-existing — filed as PJ-188.)*

**Also filed (PJ-189):** entries written by the PREVIOUS build carry no flag and therefore keep
the pre-fix behaviour until the note is next opened-and-closed under the new build. Deliberate —
an unflagged entry is treated as real work, the direction that never discards — but it means
the fix self-heals per note rather than instantly.

**Gates after the correction:** vitest **63 files / 722 tests** · svelte-check **0 errors** ·
Sight perf SERIAL 31/31 · Rust untouched. **Boss re-validated** on the rebuilt binary, both the
external-edit recipe and the type-a-word-and-close-immediately case.

*(Boss test note: the tutorial handed over a `bash` `printf` command on a PowerShell box — the
third environment-unchecked instruction of the session, after two Bases columns that could not
be edited / could not display. The commands I run are verified; the ones I hand over were not.)*

### §6.7 Binary

`npm run build` then `cargo build --release` (clean on the first attempt). Chain verified by
mtime: `store.ts 12:26 → build/index.html 13:15 → constellation.exe 13:19`, and both bundles
grep-confirmed to carry the new parser. **Awaiting the Boss test — nothing is committed.**
