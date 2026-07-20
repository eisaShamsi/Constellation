# Session Log — 2026-07-20

**Arc:** research-heavy day that ended in code. Two research programmes were run, adversarially
verified, and largely **overturned**; two Boss rulings reframed the design; MIG-101 opened and its
Phase A shipped after three rounds of Boss testing.

---

## 1. The exclusion ruling — religious books out of scope

**Boss:** *"Exclude all religious books."*

Applied mid-run to the media/containers research. Sacred/scriptural/liturgical texts are out as
design evidence. Two reasons: their layout answers questions a PKM never asks (recitation,
veneration, transmission-fidelity), and they had **already produced one factual error** — Qur'anic
mise-en-page generalised to all Arabic prose, which is what the earlier paragraph correction was.

Notably this is the **categorical** form of a fix I had proposed only as an instance (I had warned the
agent to watch for the same trap in the Hebrew material). Research-side Solve-the-Class.

**Commits:** `c5ae7fe1` (concept record re-grounded — headline, ceremony, addressability and the
fixed-frame caution all moved off religious exemplars), `db18ce9e` (Synthesis v2).

---

## 2. Synthesis v2 verified — and it did not survive well

`wf_b304b51b-5df` adversarially verified the four load-bearing claims. **One REFUTED outright, three
partly confirmed with their design recommendations removed or inverted.**

- **C4 REFUTED.** "Zones planned first, base text written last" is attested by nobody; the one secular
  manuscript with decisive internal evidence runs the other way. **"Shrink the host, never the
  annotation" lost its precedent entirely** — parchment was zero-sum, a scrolling viewport is not.
- **C2 INVERTED** — the claim I had reported to the Boss as a finding. Hypothes.is tries quote
  **last** (Range → Position → Quote); quote is the **validator**, not the address. Every historical
  success of lemma anchoring happened over a **frozen** host.
- **C3** causal claim retracted; what survives is **"do not gate capture on a head."**
- **C1** re-described; my own brief had asserted an interlinear zone that is the glossed *Bible's*
  hallmark — I did the exact "substitute a religious example and relabel it" move I had told the
  agent to refuse.

**Systemic finding:** all four leaned on a religious evidence base while presented as secular. The
exclusion did not merely trim the corpus — it revealed the layered-page evidence was
religious-dominant all along.

**Commit:** `695b05a5`.

---

## 3. Note-shape research on non-historical evidence

**Boss:** *"Now research the note shape concept without the historical framing."*

`wf_a88a4260-e4c` — six tracks (cognition · empirical PIM · product teardown · annotation-over-
editable-host · contemporary methods · type/schema theory), 36 load-bearing claims each adversarially
refuted before synthesis.

- **The premise holds for the artefact, not the mind.** Constraints demonstrably change what gets
  written (Gligorić ICWSM 2018, with a control arm); they do **not** improve memory or thinking —
  that failed two direct replications and a 77-effect-size meta-analysis. **Non-negotiable
  consequence: no shape feature may be pitched as "this helps you remember."**
- **The five dimensions are not peers.** Signal is structurally prior; extent is the most intuitive
  and worst-justified; attachment carries a measured 22–27% orphan rate.
- **Automatic graduation contra-indicated** by four unrelated fields. The split: **container may
  graduate automatically; KIND must be proposed.** Build order inverts — **byte-exact revert first**.
- The verification pass **executed the shipped library** and falsified the `MapMode.TrackDel` orphan
  prescription against `@codemirror/state` 6.5.4 in our own `node_modules`.

**Commit:** `7a163382`.

---

## 4. The Uninterrupted Stream — Boss ruling

**Boss:** *"Interruption is a creativity killer… the app should provide an uninterrupted stream."*
Then: *"Don't interrupt, but notify… a subtle color signal."*

Resolves graduation by **rejecting the axis I posed it on**. I framed it automatic-vs-deliberate; the
real variable is **WHEN**. A mid-stream prompt is a wizard compressed into one question — consent
obtained by interruption is a toll, not a conversation. **Silent action is dangerous; silent
observation is not.**

Also retired my own Bullet-Journal objection: BuJo migration is manual **and** non-interrupting, so
the ruling and the evidence agree.

**Commit:** `343f5de5`.

---

## 5. Non-interrupting signal research

Three threads. **The colour instinct is sound** (Wolfe & Horowitz 2017 — colour is a top-four guiding
attribute), with the constraints that decide the design:

- **Pop-out needs a homogeneous field**, and the NotePane text column is not one (verified in our own
  code). → **reserved silent channel**; the gutter is `display:none` and therefore perfectly silent.
- **Only ABRUPT ONSETS capture attention; colour singletons do not.** The interruption is the moment
  of appearance, not the steady state → introduce at a >2000 ms inter-key pause.
- **Crowding caps the vocabulary at ONE bit** — detection without identification.
- **The "23 minutes" figure is a myth** (absent from the paper; that study found interrupted work
  finished *faster*) — **but the ruling is right anyway**: for *generative* work, quality drops
  measurably (Foroughi 2014; writing speed 3.51 → 1.25 chars/s).
- **Return ≠ remember** — Zeigarnik refuted (ratio 0.99), Ovsiankina supported (67%). **The signal
  cannot be the durable path; the Reviewer is.**
- **The honest counter-case:** the closest prior art logged **186,480 suggestions displayed, 197
  followed (0.1%)**.
- Addendum corrected **my own** hue advice: not a hue axis — carry a **lightness** difference.
- **The accessibility escape hatch:** WCAG technique **G14** means the Reviewer entry frees the tint
  from the 3:1 floor. **One architectural decision satisfies discoverability AND accessibility.**

**Commits:** `d992d50c`, `4f290546`.

---

## 6. MIG-101 — Plan

**Commit:** `c946c5ec`. Six phases; every codebase fact verified from the repo first (two research
passes had already shipped filler). Boss ruled the sequencing: **shape + signal first**, Qusasah as
its own migration.

---

## 7. MIG-101 Phase A — BUILT + BOSS-VALIDATED

Three rounds of Boss testing; **each failure had a different cause**, which is itself the record.

### §A0 — a pre-existing content-integrity bug, found before writing a line of new code
`update_frontmatter_property` (shipped, used by **Bases cell editing today**) did
`content.lines()` … `join("\n")`, which on every property edit **silently converted CRLF files to LF
throughout — body included — and stripped the trailing newline.** On Windows that turns a one-field
edit into a whole-file diff under Git/Syncthing. Proven RED first (3 tests), then fixed **structurally**:
the frontmatter block is located by **byte offset** and only that region is rebuilt; the body is
spliced back verbatim and never split. Byte-exactness is now by construction.

Also found the **Rust suite was already red on `main`** — two `cache::` failures from test fixtures one
column (`created`) behind the production `note_links` schema. Fixed; suite green.

### §A1–A3 — shape, history, revert
`shape.rs` (new): closed vocabulary `scrap|page`, `shape_history`, module schema-versioning mirroring
`review.rs`. Menu items in NotePane ⋯, handler in NoteEditor, **i18n ×15**.

### Round 1 — "Step 2: nothing changed"
The write had **succeeded on disk** (verified by reading the bytes). What failed: shape was written
**around** the open note's model, which composes frontmatter from its **open-time byte-base** — so the
next save would have **silently erased it**. Not cosmetic: the content-divergence class MIG-076 exists
to prevent. **Fix: shape goes THROUGH the model**, the same door the Properties panel and the stage
promoter use. 8 vitest cases, including one that **documents the original fault deliberately**.

### Round 2 — undo oscillated page → scrap → page …
Undo applied the inverse **and recorded it as a new change**, so every undo became the next undo's
target. **Fix: undo CONSUMES a step** (an `undone` cursor); a new change truncates the undone branch.
3 Rust tests, one asserting undo **visits distinct states and terminates**.

### Round 3 — undo went completely inert
**A migration bug, mine.** The `ALTER TABLE` adding `undone` lives inside `ensure_shape_schema`, which
was gated behind `!is_stamped` — and the Boss's table was **already stamped at v1** by the previous
build. The upgrade never ran; every query naming `undone` failed with *no such column*; and my
`.catch(() => {})` **swallowed it**, producing a broken feature with **no symptom at all**.
Diagnosed by reading the Boss's live `search.db` directly.

Three fixes, not one: **(1)** version bumped to 2; **(2)** the **class** removed — every entry point now
*upgrades if behind* instead of *bailing if not current*; **(3)** the swallowed error is gone.
v1 rows (the recorded oscillation) are discarded on upgrade — defect-era audit, not user intent.
**Proven against a COPY OF THE REAL DATABASE**: `no such column` before → column present, stamped v2,
query OK after.

### Verification at close
**Rust 1077 passed / 0 failed · frontend 8 passed · svelte-check 0 errors.**
**Boss test: Steps 2–6 PASS**, including Step 6 (set shape → type → switch away and back → shape survives).

---

## Standing-order notes

- **Reproduce-First earned its place three times today** — the disk bytes, the `search.db` schema, and
  the copy-of-the-real-DB upgrade proof. Every diagnosis came from reading the actual artifact, not
  from reasoning about the code.
- **Three rounds on one step.** Stated to the Boss that a fourth would be a signal to stop patching
  and re-examine the design rather than fix another instance (LL-014 discipline).

---

## 8. Safety inspection (pre-commit) — and a third app-killer

`wf_5ff846df-00c`. **Ran whole-app despite `args.files` — PJ-124 re-confirmed.**
**37 confirmed: 3 APP-KILLER · 19 HIGH · 12 MED · 3 LOW**, across 15 files.

### The one that was MINE
`NoteEditor.svelte` `applyShape` was the **only** write path in the component without
`if (readOnly) return` — its four siblings all carry it. The Index preview holds a SECOND model for a
path that may also be open in a real tab; a shape click there composes that stale body over the live
note, and because the receiving model is CLEAN the watcher **adopts** the revert instead of raising a
conflict sidecar. **Silent revert, on screen and on disk.** Display-not-Domain, broken by my code.
Fixed: guard before any mutation + menu hidden when read-only + `tests/mig-101/readOnlyWriteGuard.test.ts`.

> **Near-miss worth recording.** Proving that test red, my removal script silently didn't match —
> the file is CRLF and I searched with `\n`. **The exact line-ending class I had fixed hours earlier
> nearly let me ship an unproven test.** Redone with EOL detection: 2 failed → guard restored → green.

### The third app-killer, Boss-ruled and fixed in the same job
`yamlDoc.ts` `serializeLine` had **no `nested-object-list` branch**, so editing one row of a
structured `ikhtilāf` block flattened the whole block-seq to a scalar summary. On reopen the parser's
nested branch requires an EMPTY value, so it never fired — **every structured row gone from the `.md`,
silently.** The legacy `reconstructFrontmatter` serialized it correctly; the G4 swap dropped it.
Reproduced red-first (`tests/g4/nestedObjectListRoundtrip.test.ts`, 2 failed → 5 passed), fixed by
restoring the branch. **All 30 G4 tests pass — the compose path did not regress.**

### Untriaged
34 findings remain (**PJ-130**). Classes: silent-data-loss ×8, index-divergence ×7,
content-corruption ×4, content-loss ×4, cross-window-clobber ×3, false-success ×3, freeze-hang ×3.
`store.ts:2298` (second-screen dirty-birth) was **re-confirmed** by this sweep.

## Close

**Rust 1077/0 · frontend 570/0 (44 files) · svelte-check 0 errors.**
Orientation → **v3.61**. PJ ledger → **v1.40** (PJ-130, PJ-131 filed).
**Boss-tested:** Phase A Steps 2–6. **NOT Boss-tested:** the read-only guard and the yamlDoc fix —
Boss authorised the commit directly ("Fix the yamlDoc one, then commit").
