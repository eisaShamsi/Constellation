# HANDOVER — 2026-08-11, PJ-249 close

**Read in this order:** this file → `docs/Constellation Orientation & Onboarding v3.95.md`
(the "What changed in v3.95" preamble, then §3–§4) → `docs/Constellation Pending Jobs v1.79.md`
(► Next action) → `lab/reports/SESSION-LOG-2026-08-10.md` §6d–§6h.

**Working directory:** `E:\مشاريع كلاود\Constellation`, branch `main`. Never read or edit
anything under `.claude/worktrees` — it is a session-spawn artefact, not the project.

---

## 1. State of the tree

| | |
|---|---|
| HEAD | `904bccbc` — *PJ-249 §6f–§6h — the index the migration built had never been used* |
| Rust | **1,442 passed / 0 failed** |
| vitest | **927 passed + 3 expected-fail** (82 files) |
| svelte-check | **0 errors** (268 warnings, pre-existing) |
| release binary | `src-tauri/target/release/constellation.exe`, **2026-08-10 22:35**, Boss-validated |
| working tree | clean at handover except the docs written with this file |

**The 3 "expected fail" are deliberate.** They are PJ-252's pinned reproduction in
`tests/pj-249/yamlCommentInSeqDestroysBlock.test.ts`, written as `it.fails`. **Green means the
defect is still live.** The moment PJ-252 is fixed they turn RED and must be flipped to `it`.

---

## 2. PJ-249 — CLOSED, Boss-validated

**Measured on the Boss's real universe, end to end: `~50,000 ms → 216 ms` per rename.**

```
seek-query 20 ms | freshness-map 2,730 rows 2 ms
freshness-net-done (1 suspects) [382 dirs, 2,109 .md (0 unknown, 1 drifted)
                                 | read_dir 16 ms | metadata 0 ms] at 71 ms
rewrite-done at 216 ms
```

### The thing worth carrying forward

The migration's headline number was **false for a week**, and every test was green the whole
time. The seek shipped, the Boss's second rename measured 44 ms, and it looked done.
`EXPLAIN QUERY PLAN` on his live database said otherwise:

```
SCAN note_links USING INDEX idx_link_source      <-- a full scan of 31,368 rows
sqlite_stat1:  idx_link_target_base -> 31367 31367
real cardinality: 3.8
```

**The index had never been used.** The statistic was collected while `target_base` was all
NULL — true then — and falsified by the §4 back-fill that filled it. The 44 ms was that same
full scan served from the OS page cache; the first rename of each session paid **2,579 ms**.

Fixed **structurally, not statistically**: the index carries `source_path` so it *covers* the
query, verified to hold with `sqlite_stat1` deleted entirely. A scoped `ANALYZE` also runs, so
the false statistic is not left in the database for the next query written against the column.

### What is now shared, and must stay shared

- **`search::ensure_index_shape(conn, name, expected, create_sql)`** — reads `pragma_index_info`,
  drops on drift, creates. Used by `target_base_backfill::widen_seek_index` **and**
  `link_boot_index`. `CREATE INDEX IF NOT EXISTS` keys on the NAME: a changed column list is a
  silent no-op for every existing user. **Background only** — never call it from `init_db`.
- **`link_boot_index::BOOT_LINK_COLUMNS`** — one constant feeding the boot query (`cache.rs`),
  the covering index, and the tests. **Its ORDER is load-bearing and pinned by a test**:
  `cache::read_links_in_schema` reads positionally (`row.get(0)`…`row.get(11)`), so a reorder
  silently feeds every link's annotation into its confidence.

---

## 3. ► NEXT ACTION — PJ-252 (APP-KILLER, reproduced, unfixed)

**Adding a tag to a note whose frontmatter list carries a comment line, or an item wrapped
across two lines, DELETES the entries already in that list, from the `.md`, with no error.**

Two classifiers disagree about whether the block is safe to edit:

- `store.parseFrontmatter` (`store.ts:2582`) is **LINE**-based — `blockExtent`/`isYamlBlockChild`
  absorb the comment, then require every content line to be a bare `- item`. A comment fails
  that → projected READ-ONLY with an **empty `value`** and no `listItems`.
- `yamlDoc.immutableBlockKeys` (`yamlDoc.ts:225`) asks the **`yaml` library** — comment attached
  as a comment, wrapped item folded to one scalar → "all scalars" → **not protected**.

The mutator then rebuilds from `p.listItems ?? (p.value ? split : [])` = `[]`. Same route
destroys a typed-link block via `addTypedLinkToProps` (`store.ts:1468`).

**Exposure measured, not assumed: 1 of 10,077 live notes** across both universes — a probe
note, on `authors`. Urgent by class, not by blast radius. **Do not alarm the Boss about it;
he has already been told the number.**

**The fix is ONE shared predicate.** This is the *fourth* shape of a block the write path must
refuse (2026-07-24 closed seq-of-maps; PJ-182 closed block scalars), and it is open precisely
because each closure re-answered the question in a second place. `yamlDoc`'s own comment states
the standard: *"refusing here means the block survives however the panel behaves."*

Start by running the pinned reproduction — it is already written.

---

## 4. BLOCKED — needs a Boss ruling before any code

- **PJ-253** — the cascade's two halves disagree about **case**. The seek folds
  (`target_base_of` → `fold_match_key`), so `[[meeting notes]]` IS found when renaming
  "Meeting Notes"; `cascade_pattern` (`libraries.rs:7205`) matches literally via `regex::escape`,
  so the link is read and **left naming a title nothing owns**, and the rename reports success.
  **Not a PJ-249 regression** — the old walk used the same regex. Needs a ruling because the fix
  changes **which links get rewritten on disk**.
- **PJ-207 §13** — still GATED on **PJ-224**. Unchanged. Do not start it.
- **MIG-109 / MIG-110** — do not start.

---

## 5. Open, filed, not yet worked

`docs/Constellation Pending Jobs v1.79.md` is authoritative. Headlines:

- **PJ-254 + PJ-235** — the **federation-boundary family**: every rename/move/create tail
  resolves its library through `load_all_libraries` (the FEDERATED resolver), so touching a
  linked universe's note files it into THIS universe's index; and `move_item` can physically
  move a note **into** a linked universe. The file states the contract against itself at
  `libraries.rs:266-269`. Same family as the Ctrl+N picker bug fixed 2026-08-10. **One rule:
  a universe-wide list is right for RESOLVING a name and wrong for CHOOSING where to write.**
- **PJ-255** — six detached DB tails with no generation guard across a universe switch (folds
  into PJ-244/245/246's helper).
- **PJ-256** — **no back-fill in the app re-collects statistics for the table it fills.** The
  only two `ANALYZE` sites run at the *start* of their pass. PJ-249 fixed the one instance that
  was measurably mis-planned; 14 back-filled columns are unindexed today and the first index
  added to any of them lands on a statistic nothing refreshes.
- **PJ-257** — `props_reparse` **fails on every boot and re-arms forever** over 2 rows. Seen
  live in the Boss's log twice.
- **Two whole-app sweep registers, 54 confirmed findings**, durable at
  `lab/reports/sweeps/SWEEP-2026-08-10-fourth-whole-app.json` (25) and
  `SWEEP-2026-08-11-fifth-whole-app.json` (29). **Most are not yet individually numbered** —
  that triage is itself a ledger job.

---

## 6. Standing constraints that bit THIS session

- **Never describe the app without looking at it.** Every Boss-facing test goes
  `tutorial-auditor` → `ui-inspector`, default verdict REJECTED. **This session's tutorial was
  rejected three times.** One rejection saved a whole round-trip: my draft told the Boss to
  watch the file-tree row, which updates *before* the slow step — he would have reported a pass
  on a test that could not fail.
- **No guessing — investigate.** I named a culprit before measuring it **three times** this
  session (the freshness net twice, the database connection once). Every one was plausible;
  every one was wrong; the instrumentation found the truth each time. One of those wrong
  guesses shipped a fix that made the symptom worse.
- **Verify the binary before every test.** Caught a stale binary once tonight by grepping the
  `.exe` for a new diagnostic string.
- **`npm run build` before `cargo build --release`** — cargo alone re-embeds a stale frontend.
- **Never touch the live database** under `E:\Constellation Universes`. Work on a byte copy.
- **The registry `universes.json` is unreliable** (PJ-233) — it lists only `كون عيسى` while the
  app demonstrably runs `Eisa Universe`. To find the active universe, read the freshest
  `.constellation/diagnostics.log` by mtime, or its own `boot-perf.latest.json`.
- **The Boss tests and passes every build BEFORE commit.**
- Shell escaping: write Python to a `.py` file with the Write tool. Heredocs mangle `\u{0}`
  and backslashes — that cost a wasted build tonight.

---

## 7. Ready-to-paste next-session prompt

```
Read, in order:
  1. lab/reports/HANDOVER-2026-08-11-pj249-close.md
  2. docs/Constellation Orientation & Onboarding v3.95.md  (the "What changed in v3.95"
     preamble, then §3 Architecture and §4.x — the BODY, not just preambles)
  3. docs/Constellation Pending Jobs v1.79.md  (the "► Next action" line)

Then: git pull origin main, and git log --oneline -5.

Working directory is E:\مشاريع كلاود\Constellation on branch main. Never read or edit
anything under .claude/worktrees.

Function in hand: PJ-252 — the frontmatter block classifiers disagree, and adding a tag
DELETES the tags already in the list. APP-KILLER, reproduced, unfixed. Its reproduction is
already committed at tests/pj-249/yamlCommentInSeqDestroysBlock.test.ts as three `it.fails`
tests — green today BECAUSE the defect is live; they turn RED when the fix lands and must
then be flipped to `it`.

Run that test first and read its failure. Then fix it as ONE SHARED PREDICATE used by both
store.parseFrontmatter and yamlDoc.immutableBlockKeys — not a fourth patch. This is the
fourth shape of this same defect (2026-07-24 closed seq-of-maps, PJ-182 closed block
scalars); each previous closure re-answered the question in a second place, which is why
there is a fourth. Make the two classifiers unable to disagree.

Exposure is measured at 1 of 10,077 live notes, so it is urgent by class, not blast radius.
The Boss already knows the number — do not re-alarm him.

Before it reaches him: diff-scoped safety-inspection, /simplify, and the Boss test through
tutorial-auditor -> ui-inspector (default verdict REJECTED). He tests and passes BEFORE the
commit.

Do NOT start: PJ-207 §13 (gated on a PJ-224 ruling), MIG-109, MIG-110.
PJ-253 (the cascade's case-fold miss) is BLOCKED awaiting the Boss's ruling — it changes
which links get rewritten on disk. Ask for that ruling when the moment is right; do not
build it unasked.
```
