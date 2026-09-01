# Session Log — 2026-09-01 — THE DRAIN CYCLE

Previous: `SESSION-LOG-2026-08-31.md` (§1–§14c — PJ-433 + PJ-446 shipped and Boss-passed; the
whole-app sweep; PJ-454 panelled). Entry commit `67b8b13e`, branch `main`, synced (git pull:
already up to date). Orientation v4.28; ledger v2.09.

**The cycle's mandate (Boss-ruled 2026-08-31/09-01):** *fix the backlog, run NO new whole-app
hunt.* The last sweep proved the ledger is a working NET and a failing QUEUE — ~158 confirmed
findings invisible inside PJ-264 (≈100) and PJ-378 (58) — and spent most of its budget re-proving
known bugs. **First item: PJ-454.** Then unpack the two umbrella entries; then PJ-434, PJ-438.

---

## §1 — PJ-454 opened: SO#8 cross-check by RE-MEASURING, not by trusting yesterday's report

**Working on: the identity-stamping engine (`ensure_cid_cn`) and the template predicate that
guards it.**

**Concept (the horse):** *a template is a MOLD — identity and birth belong to the CAST.* The engine
that mints identity must itself refuse to stamp a mold, however that mold announces itself. The
function (the carriage) is the two-signal test and where it lives.

**SO#8 cross-check — the entry is one day old and was written by me, so the check that matters is
"is the defect still exactly as described in the CURRENT tree":**
- `isTemplatePath` (`store.ts:4875-4888`) — **still location-only**; grep for `kind` in its body
  returns nothing, only `templateFolder` / `startsWith` / `includes`.
- `ensure_cid_cn` (`canonical.rs:1449`) — **still unguarded**; grep for `template|kind` across its
  body returns nothing at all.
- The two-armed Rust predicate (`search.rs:4327-4332`) — still a LOCAL CLOSURE inside one
  function, shared with nobody.
All three confirmed. Nothing shipped overnight that changes the job.

**Universe enumeration, and an honest discrepancy carried forward:** the disk holds **NINE**
universe roots with `.constellation/universe.json` (8 under `E:\Constellation Universes`, plus
`E:\موسوعة عيسى`). **The panel reported scanning 13.** Rather than adopt either number, the
evidence workflow carries a dedicated reconciliation agent: find any universes the enumeration
missed (linked/child entries, nested roots, other locations) and state what 13 most plausibly
counted, marked as inference. **A number I cannot verify does not go in front of the Boss.**

## §2 — The fix: the Two-Signal Choke Point (written; tests pinned)

Per the panel's recommendation and the Boss's approval. **In `canonical.rs`, at the engine:**

- `frontmatter_declares_template(content)` — the SELF-DECLARED arm, **scoped to the leading `---`
  fence on purpose**. A note whose BODY mentions `kind: template` (this repo's own docs read
  exactly like that) must NOT be judged a mold, because the cost of a false positive is a real
  note that never receives an identity. Lenient about spacing, quoting, CRLF and value case;
  never about *where* it looks.
- `templates_dir_for_note(file_path)` — the LOCATION arm with no AppHandle, mirroring
  `search.rs::templates_dir_for_db`: walk up to the universe root, read `templateFolder` from that
  universe's settings, resolve through the shared `resolve_templates_dir_for_root`. Best-effort by
  design — a note outside any universe resolves to `None` and the frontmatter arm carries the
  guarantee there (the same posture `templates_dir_for_db`'s own comment records).
- `is_template_file(file_path, content)` — **either arm is sufficient**, with the reasoning
  recorded at the site: location alone missed all 102 measured molds (none sat in the configured
  folder); self-declaration alone would have protected none of them either (they are Obsidian-era
  molds Constellation never marked). Both, OR'd, at the one engine — because **ten paths can reach
  it and a guard at two of them is what produced the 102.**
- `ensure_cid_cn` returns a mold **byte-identical**: no mint, **and no legacy `cid:` → `cid_cn:`
  migration** (migrating is still maintaining an identity on a file that must not hold one), and
  no write at all — so the command's `updated != content` check stays false and **a template costs
  nothing on open**.

**Four tests**, including the dangerous direction: `pj454_a_note_that_merely_mentions_templates_is_not_a_mold`
(body prose, no frontmatter, unterminated fence, `template_kind:` near-miss, `kind: note`), plus
both arms end-to-end on a real universe layout, plus the engine guarantee (mold unchanged AND
unwritten; legacy-`cid:` mold not migrated; **ordinary note still stamped and still written**).

**Frontend contract rewritten, not the code:** `isTemplatePath`'s doc now states that it is **no
longer the guarantee** — it is a cheap early-out that only saves an IPC round-trip, the Rust side
refuses regardless, and **"do not add a second arm here"**, because a duplicated predicate is the
drift the Whole-Ecosystem Fix Law exists to prevent.

**Tests: 4/4 green** (`cargo test --lib canonical` — 42 passed, 0 failed).

### §2b — A performance defect I put in my own fix, caught before it shipped

The first version tested for a template BEFORE the already-stamped early-out. `is_template_file`'s
location arm **walks parent directories and reads the universe's `settings.json`** — so that
ordering put filesystem work on **every open of every note**, since the overwhelmingly common case
is a note that already carries an identity and needs nothing done. **This is PJ-446's lesson, on
PJ-446's exact path, twenty-four hours later** — and it was mine, not inherited.

Reordered so the cheap check runs first: already-stamped → return; then the template test; then
legacy migration; then mint. The answer is identical either way (a stamped file is returned
unchanged by both branches), so the ordering is pure cost. The reasoning is recorded at the site
with the PJ-446 reference, because the next reader will otherwise "tidy" the guard back to the
top where it reads more naturally.

## §3 — The approval list (in flight, READ-ONLY)

Workflow `wf_a6e967d3-1fe`: one agent per universe + the reconciliation agent + a composer, all
under the ledger brief — three arms reported **separately** (under-templates-folder /
self-declared / name-heuristic-only, the third a **review list, never a repair candidate**), and
for every candidate: path, stamp + its embedded date, count of `earned.jsonl` records keyed on that
`cid_cn`, count of `note_links.target_cid_cn` pointing at it, and the `note_meta` row's `kind` +
`cid_cn`. **The exclusion rule is the point of the whole exercise:** a candidate with earned
records or inbound identity links **is not a mold** — it is a note that was treated as one, and it
is excluded and surfaced separately, because stripping identity from a real note silently severs
its earned reading history with nothing shown on screen.

Nothing is repaired until the Boss approves that list, and then snapshot-first.

---

## §4 — THE 102 DOES NOT REPRODUCE. I reported a false number to the Boss; here is the correction

**The evidence workflow walked 18,901 files across nine universes and found ZERO stamped molds by
the app's two definitions** — and, correctly, **refused to reconcile the gap**: *"I do not know how
the figure of 102 was produced… A repair authorised against a number nobody can reproduce is the
worst outcome available here."* It was right to refuse, and it also recorded something valuable:
in four universes a real note carries the word `template` as a LENS NAME in its properties, with
zero earned history and zero inbound links — **the exclusion rule would not have protected them**,
so a word-matching repair tool would have silently stripped four real notes.

**I then measured it myself, because two of my own gates disagreed and neither number could stand.**

**The truth is in neither report. 67 files** carry a template placeholder (`{{…}}`) in their
frontmatter **and** an identity stamp:

| Universe | Stamped molds |
|---|---|
| Eisa Universe | 50 |
| موسوعة عيسى | 8 |
| Eisa Cognitive Knowledge | 5 |
| MIG108 Rehearsal | 4 |

Read directly to confirm, not inferred:
`الكون المعرفي\x\Templates\Base 1 Template (up, related, created).md` →
`created: "{{date}}"` immediately above `cid_cn: 20251229T125213Z_NOTE_9949`.

**Why each count was wrong.** The panel OVER-counted: it treated every stamped file in a
template-named folder as damaged. The evidence sweep UNDER-counted: it recognised only the app's
two definitions, so real molds the app cannot see registered as "correctly stamped, do not touch."
**Both were measuring; neither was measuring the thing.**

### §4b — The consequence that matters more than the count: MY FIX DOES NOT COVER HIS DATA

**None of those 67 files would be protected by the Two-Signal Choke Point.** They do not declare
`kind: template`, and they do not sit in the configured folder — they live in `القوالب`,
`قوالب العرب`, `قوالب الفكر والتمعن`, `الكون المعرفي\x\Templates`, `قوالب سجل الخليج`, and more.

**His vault organises templates PER DOMAIN, in several folders, in Arabic. Constellation's setting
holds exactly ONE templates folder.** That structural mismatch — not a missing arm on a guard — is
the defect underneath PJ-454. The fix remains correct and necessary; it is simply **not sufficient
for his actual organisation**, and I would have told him it was.

**Not repaired on this signal.** `{{` in frontmatter is a strong indicator, not a definition — a
note *about* Templater syntax would match it, and stripping identity from a real note is the one
harm here that is silent and permanent. The revised picture goes back to the panel before any
ruling request.

## §5 — The diff inspection found TWO defects in the code I had just written. Both mine, both fixed

Standing order (per-build, diff-scoped) on `canonical.rs` + `store.ts`. It did not take my word
for anything — **it compiled my function and ran it**:

1. **MED, silent-data-loss —** `frontmatter_declares_template` tested `kind: template` on the RAW
   line with no root-key check, so an **INDENTED** occurrence returned `true`: a nested map child,
   a block scalar's contents, a tab-indented line. A frontmatter cheat-sheet note (`example: |` /
   `  kind: template`) or a Templater config map would have been judged a mold and **permanently
   denied an identity** — silently, on every open, and **the boot healer could not have repaired
   it**, because ITS exemption reads the ROOT `kind` and would not have matched. Zero live
   incidence (10,921 files walked) but reachable by legal YAML at any time.
   **This is the exact false-positive direction I had written a test for and congratulated myself
   on.** I scoped the search to the frontmatter fence and forgot to scope it to top-level keys —
   in a file where three neighbouring functions already call `yaml_lines::is_top_level_key_line`
   for precisely this, and which was swept for this same indentation-is-data class on 2026-08-11.
   **Fixed:** the helper now gates the test.
2. **LOW —** the location arm used `Path::starts_with`, which is **case-sensitive**, while the
   frontend predicate it is documented as backstopping compares case-insensitively. The setting is
   a free-text field, and typing `templates` against an on-disk `Templates` is invisible on
   Windows — so the arm could silently miss, **and my own new comment in `store.ts` ("the Rust side
   still refuses") would have been false.** A durable comment recording a guarantee the code does
   not provide is worse than no comment. **Fixed:** case-insensitive comparison on normalised
   separators, matching `isTemplatePath`.

**Two new tests**, each written to fail against the version I wrote an hour earlier:
`pj454_an_indented_kind_template_does_not_make_a_note_a_mold` (nested map, block scalar, tab,
nested-quoted, seq item — plus the root key still counting) and
`pj454_location_arm_is_case_insensitive` (lowercase setting, capitalised folder on disk).

**Both fixes verified: 6/6 PJ-454 tests green** (`cargo test --lib canonical` — 44 passed, 0
failed). Full suite re-running.

**The lesson worth keeping, stated against myself.** I wrote a test *specifically* for the
false-positive direction and said so out loud — then shipped the exact false positive I had named,
because I scoped the search to the frontmatter fence and forgot to scope it to root keys. **Naming
a hazard is not the same as guarding it.** The gate that caught it did not reason about my code;
it *compiled and ran* it. That is the difference between a review and a measurement, and it is the
second time in two days that "verify by executing, not by reading" was what actually found the
defect (the first was the mold count itself).

## §6 — The revised panel (`wf_c25d4845-90f`)

Convened on the corrected picture rather than asking the Boss to rule against another unverified
number. It measures his real template topology (every template-looking folder in the four affected
universes, with per-folder placeholder and stamp counts) and researches multi-folder template
support in Obsidian/Templater/Logseq/Dendron, then judges six candidate fixes — multi-folder
setting · folder-name convention · mark-his-molds · placeholder signal · consolidate-by-hand ·
combinations — through three lenses: false-positives/irreversibility, file-over-app fidelity to
HIS organisation, and scope/cost inside a DRAIN cycle.

**The discriminating question given to the topology agent**, because it decides the whole design:
*are there placeholder files OUTSIDE template folders, and template-folder files WITHOUT
placeholders?* Every candidate is also tested against the four real notes that carry `template`
as a **lens name** with zero earned history and zero inbound links — the ones an
earned-history exclusion rule would **not** have protected.
