# MIG-101 — Note Shape, Graduation, and the Quiet Signal
## `/migration` Phase 2 — the Plan

**Concept (the horse):** *A note's shape is a claim the note is making about itself. When the writing
stops matching the claim, the note should be able to say so — at a moment the writer chooses to
listen, never by interrupting them.*

**Function in hand:** the **shape field on a note**, the **observation that a note has outgrown it**,
and the **quiet gutter signal** that surfaces the observation without breaking the writing stream.

**Phase 1 (Architect) is already done** and lives in three verified research documents:
`Note-Shape-Concept-Research-v1.md` (six tracks, 36 load-bearing claims adversarially verified) ·
`Non-Interrupting-Signal-Research-v1.md` (+ addendum) · `Note-Shape-and-Template-Studio-Brainstorm.md`
(the durable concept record and Boss rulings).

---

## 0. Codebase facts this plan rests on — VERIFIED THIS SESSION, not assumed

Every one was read from the repo, because two research passes in this session shipped confident
filler that had to be retracted.

| Fact | Where | Status |
|---|---|---|
| `ReviewStatusPanel.svelte` exists; `reason: never_reviewed \| interval_due \| checkpoint \| dismissed` | `src/lib/components/ReviewStatusPanel.svelte:11` | ✅ verified |
| `review_schedule` is a **persisted** table — `(path PK, reason NOT NULL, due_days, stratum, last_reviewed)` | `src-tauri/src/review.rs:1264` | ✅ verified |
| Module schema versioning exists — `schema_versions (module, version, updated_at)` | `src-tauri/src/review.rs:1263` | ✅ verified |
| `.cm-gutters { display: none }` in NotePane — **the gutter is unused and chromatically silent** | `src/lib/components/NotePane.svelte:577` | ✅ verified |
| `note_meta` / `init_db` live in search.rs | `src-tauri/src/search.rs:2949` | ✅ verified |
| `update_note_property` is the frontmatter write path | `src-tauri/src/libraries.rs` | ✅ verified |
| Highest existing MIG = 100 → this is **MIG-101** | repo-wide grep | ✅ verified |

---

## 1. Scope — and the sequencing decision the Boss must rule on

### In scope for MIG-101
Shape as a **frontmatter field** · the **observation engine** (persisted, write-time) · the
**Reviewer route** · the **quiet gutter signal** · the **ramp** to a proposal · **container**
graduation (automatic + silent + reversible).

### Explicitly OUT of scope
- **Kind graduation as an automatic act.** Forbidden by the evidence (§2.3). It is a *proposal*, never
  an act.
- **Qusasah** — its own migration (proposed MIG-102). Far larger: an anchoring engine, sidecar
  storage, a rendering layer.
- **Template Studio** — MIG-103.

### ⚠ THE ONE DECISION THAT NEEDS A BOSS RULING BEFORE BUILD

**The brainstorm's D3 ruling said "SPLIT INTO THREE; Qusasah ships FIRST."** This plan puts
**shape+signal first instead.** Three reasons, offered for the Boss to accept or overturn:

1. **The evidence inverted the build order.** *"Build the byte-exact revert FIRST, because
   reversibility is what determines whether automatic is permissible at all."* Shape's revert
   infrastructure is small (a frontmatter field that never touches body). Qusasah's is large.
2. **Qusasah's engineering ground moved.** The v1 anchoring prescription was **empirically falsified
   against the shipped library** (§2.4). Its architecture is now specified but unbuilt and
   unprecedented — the research states plainly that *no product precedent was located* for annotation
   over an editable, externally-mutating local file. That is a research-grade risk; shape is not.
3. **Shape+signal is the smaller proof of the same architecture** — write-time observation, the
   Reviewer as durable path, the quiet channel. Qusasah will reuse all three.

**If the Boss prefers Qusasah first, this plan re-orders without rework** — Phases A–C are shared
infrastructure either way. *That is the point of the sequencing.*

---

## 2. The invariants this migration must not break

1. **The host body is never rewritten to change shape.** Shape changes *presentation and templating*,
   never content. **This is what makes revert byte-exact by construction** rather than by careful
   coding.
2. **Nothing blocks, modals, toasts, or steals focus during composition.** (Uninterrupted Stream
   ruling.)
3. **The passive signal carries exactly ONE state.** Capped by peripheral crowding — detection
   without identification — and by habituation generalising across cue families. Any second meaning
   goes on a **different channel**, never a second hue.
4. **The observation is data, not a signal.** Persisted, queryable, survivable. The tint is a *view*.
   (Rule 8, write-time derivation.)
5. **Conservation:** every note is in exactly one observation state at every read. Orphan-by-
   construction, never by a heuristic that can fail to fire. *(The Hypothes.is under-reporting bug:
   a channel that lies by omission is worse than no channel.)*
6. **Zero `invoke()` on the keystroke path.** Detection runs Rust-side, debounced, off the hot path.
7. **No shape feature may be described to the user as helping them remember or think better.** That
   warrant does not exist — it failed two direct replications and a 77-effect-size meta-analysis.
8. **i18n ×15 and RTL** on every user-facing string and position.

---

## 3. The phases

### PHASE A — Reversibility and the shape field *(the foundation; nothing else may land first)*

- **A1.** `shape` frontmatter field, written **only** through `update_note_property`. Values are a
  small closed vocabulary. Absent = unshaped, which is a valid and common state.
- **A2.** `shape_history` table — `(path, from_shape, to_shape, changed_at, changed_by)` where
  `changed_by ∈ {user, container_auto}`. Every change inspectable.
- **A3.** Revert: one gesture, restores the prior `shape` value. **Assert file bytes outside the
  frontmatter `shape:` line are unchanged.**

> **Verification (Boss-testable):** set a shape on a note, change it, revert it. The note's text is
> byte-identical to before. A test harness asserts this on a probe file; the Boss confirms on a real
> note that nothing in the body moved.

---

### PHASE B — The observation engine *(persisted, write-time, cheap)*

- **B1.** `shape_observation` table — `(path PK, observed_shape, evidence_json, confidence,
  first_seen, state)` where `state ∈ {open, acted, dismissed}`. Module-versioned via
  `schema_versions`, matching `review.rs`'s existing pattern.
- **B2.** The detector runs **on the existing index write path** (Rule 8) — never a `scan_*` command,
  never a boot rebuild. Evidence is **structural and countable** ("N dated headings appended over M
  weeks"), never a semantic guess.
- **B3.** **Confidence threshold — suppress below it entirely.** Smart Compose's discipline: show
  nothing when not confident. *A proposal that fires on ambiguous evidence habituates the channel and
  costs it permanently.*
- **B4.** Conservation check at read: `open + acted + dismissed == total observations`.
- **B5.** First-time back-fill runs **in the background after paint**, with status-bar progress, and
  is **resumable**.

> **Verification:** on a large Universe (7,600+ notes), boot time and typing latency are unchanged,
> measured before/after. A note that genuinely accreted dated entries produces exactly one open
> observation carrying its evidence; an ordinary note produces none.

---

### PHASE C — The Reviewer route *(the durable path — AND what makes the tint legal)*

**This must land BEFORE Phase D.** Two independent reasons, and they are the strongest structural
finding of the research:

- **Discoverability.** People *return* to surfaces they own (Ovsiankina, 67%) but do **not** remember
  what they were shown once (Zeigarnik, refuted — weighted ratio 0.99). The tint cannot be the
  durable path.
- **Accessibility.** WCAG **G14** — if the state is also available in text somewhere reachable, the
  tint becomes a *redundant enhancement*, exempt from the 3:1 contrast floor. **The Reviewer entry is
  the G14 text equivalent. Without Phase C, a genuinely subtle tint is a Level A failure.**

- **C1.** New `reason` value `shape_outgrown` on `review_schedule`. **Additive** — existing reasons
  untouched.
- **C2.** `ReviewStatusPanel` renders it **showing the evidence, never the conclusion**:
  *"9 dated entries appended over 6 weeks"* — **not** *"this looks like a journal."*
- **C3.** i18n ×15 for every new string; `detectDir` on user content; RTL layout verified.

> **Verification (Boss-testable):** a note with an open observation appears in the Reviewer with its
> evidence stated in plain language, in every UI language, correctly laid out in Arabic.

---

### PHASE D — The quiet signal *(only after C)*

- **D1.** Enable a **dedicated gutter** in NotePane, used **exclusively** for the shape signal — the
  reserved silent channel. *(FocusPane is excluded: plain text only, per the Editor Parity exception.)*
- **D2.** **Visual spec, evidence-bound:**
  - a **large, low-saturation tint** — area compensates for eccentricity; a soft band beats a bright pip
  - **a lightness difference as well as a hue difference** — *not* a hue-axis choice *(corrects my own
    earlier "blue–yellow axis" advice: Apple names blue-vs-orange confusable, Okabe & Ito name
    blue-vs-violet, and blue–yellow is the axis tritan loses)*
  - **never red** — the reserved alarm register
  - **no motion, no pulse, no flash** — motion is an undoubted guiding attribute; it *will* capture,
    which the ruling forbids. *(Note the precedent: IEC 60601-1-8 defines its LOWEST alarm priority as
    steady and non-flashing.)*
  - **no salience escalation over time** — habituation collapses response after the *second* exposure.
    **Escalate PLACEMENT (into the Reviewer), never brightness.**
- **D3.** **Onset — the interruption risk lives here, not in the steady state.** Introduce during a
  detected pause: **inter-key interval > 2000 ms AND cursor at a sentence/paragraph terminator.**
  Never mid-burst. Fade in (~1 s) as a second-order measure. *Timing is the evidence-backed lever;
  fading is the design-practice lever. Rely on timing.*
- **D4.** **RTL:** the mark itself does **not** mirror (no directional semantics); **its position
  must**, via logical properties (`inset-inline-start`), never physical `left`/`right`.
- **D5.** Style Setter category, per the existing add-an-element recipe.

> **Verification (Boss-testable):** type continuously in a note that crosses the threshold — **nothing
> appears while you are typing.** Pause; the band appears softly in the gutter. It is present but
> ignorable. In Arabic it sits on the correct edge. Type a burst of 10 characters in both NotePane
> and FocusPane: no lag.

---

### PHASE E — The ramp *(progressive disclosure, never a wizard)*

Staged, with bail-out at every stage, **no stage costing more than ~2 seconds**:
**tint → hover/caret reveals a one-line statement of what was observed → click opens the proposal with
its evidence → act or dismiss.**

- **E1.** Dismiss is **durable** — the observation does not re-fire on the same evidence.
- **E2.** Ignored observations **age out to QUIET, not to GONE.** Unbounded accumulation is the
  attested failure mode of deferred queues; full silence has its own measured cost.
- **E3.** Accepting is **cheaper than dismissing** — the shipped pattern across every non-interrupting
  proposal examined.

> **Verification (Boss-testable):** the signal can be walked past indefinitely with no penalty;
> hovering explains it in one line; acting changes only the `shape` field; dismissing silences it for
> that evidence and it does not come back.

---

### PHASE F — Container graduation *(automatic + silent + reversible — last)*

Only after A–E prove the reversibility and the channel.

- **F1.** A note that outgrows its **container** gains page affordances **automatically and silently**.
  **No judgement is made, nothing is reclassified, nothing is lost.** This is exactly the Boss's
  sentence — *the container must not block him.*
- **F2.** **KIND is never changed automatically.** It is proposed through E, with evidence shown.

> The split is the whole finding: **structural inference may change how a note BEHAVES; it may never
> silently change what a note IS.** *(Excel inferring type from cell structure corrupted >30% of a
> literature by 2020 and the vendor's fix was not a smarter classifier — it was showing a notification
> before the conversion. A silent action became a marked one.)*

---

## 4. Per-build discipline (Standing Orders)

Every phase: **Boss tests and passes BEFORE commit** (top standing order) · diff-scoped
`safety-inspection` over changed files · `/simplify` on the diff · session-log entry · **SO#9 PJ-ledger
reconciliation in the same commit** · orientation v-bump when a trigger fires.

**Phase 4 (Audit)** at close: invariants · drift · migration path (first boot, schema mismatch,
mid-backfill interrupt, rollback).

---

## 5. What this plan deliberately does NOT claim

- **No cognitive benefit.** Shape changes the artefact, not the mind.
- **No historical warrant.** The manuscript grounding was verified and collapsed.
- **The quiet channel may simply not be noticed.** The closest prior art logged **186,480 suggestions
  displayed, 197 followed — 0.1%.** Phase C exists *because* of this, and it is why the Reviewer, not
  the tint, is the durable path. **This risk is mitigated, not eliminated.**
- **Whether users act on the proposal at all can only be answered by shipping it and watching.**
  Instrument which observations are acted on, dismissed, or ignored — and be willing to cut the
  feature if the answer is "none."

---

*MIG-101 Plan, 2026-07-19. Awaiting Boss approval on §1's sequencing decision. Plan approval = build
approval; the cascade then runs to the Phase-A verification clause.*
