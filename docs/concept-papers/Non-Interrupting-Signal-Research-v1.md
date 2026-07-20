# The Non-Interrupting Signal — Research v1

**Function in hand:** how Constellation holds an observation ("this note has outgrown its shape") and
surfaces it **without breaking the writing stream** — and whether the Boss's **colour signal** is the
right mechanism.

**Rulings this serves** (both Boss, 2026-07-19):
> "Interruption is a creativity killer… the app should provide an uninterrupted stream for their creativity."
> "Don't interrupt, but notify… a subtle color signal will appear somewhere on the page to let them know they need to act when they are done writing."

**Method.** ~35 primary sources; PDFs pulled and text-extracted locally rather than trusting search
summaries. Where a search summary and the primary source disagreed, **the primary source won — and it
did disagree, twice.** Claims graded **[sourced]** / **[recalled]** / **[unknown]** throughout, per
the BASIC RULE. This pass exists because the prior historical research shipped confident filler.

---

## ★ 1. The verdict on the colour signal: SOUND — with one qualifier that decides the design

**Colour is a top-tier attentional channel.** Wolfe & Horowitz, *Five factors that guide attention in
visual search*, **Nature Human Behaviour 1:0058 (2017)**, Box 1 — the "undoubted guiding attributes"
are **Colour, Motion, Orientation, Size**. Of ~two dozen candidate attributes, colour is in the top
four. **[sourced]** *The instinct is right.*

**But pop-out is defined against a HOMOGENEOUS field, and ours is not.**
Adam, Patel, Rangan & Serences, *Journal of Cognition* 4(1):34 (2021) — **N=190, >210,000 trials**:
distractor heterogeneity substantially reduced colour-singleton capture, **F(1,92)=202.99, p<.001,
η²p=.69.** Heterogeneous displays force deliberate feature-search that overrides automatic capture.
**[sourced]**

**Checked against our own code:** `src/lib/editor/` already renders, *inside the text column*,
callout backgrounds and accent text, wikilink chips with coloured backgrounds, link colours,
per-link-type colours (`--ltc`), highlights, code colours and blockquote colours. Add the six link
**stage** colours (`stageEstablished` green · `stageFresh` cyan · `stageBirth` orange ·
`stageGrowing` purple · `stageAtRisk` yellow · `stageDormant` grey), library identity colours and
stratum. **The NotePane text column is a heterogeneous colour field.** A subtle new colour placed in
it will not pop out — and `stageAtRisk` yellow *already* means "act on this."

> **★ Therefore: the signal must live in a RESERVED, chromatically silent channel — not in the text.**

**And we already have the perfect one, switched off:** NotePane currently sets
`.cm-gutters { display: none }`. **The gutter is unused and therefore perfectly silent.** A narrow
gutter used *only* for shape signals gives the two properties the text column cannot — a homogeneous
field where one colour is the only colour, and uncrowded space (§4).

*Caution on this synthesis:* "a reserved channel works because it is homogeneous and uncrowded" is
**assembled reasoning** from Wolfe's pop-out definition + Adam et al. + Bouma's law. **No single paper
states it in that form.** Well-supported; not a citable finding. No published design rationale for
IDE gutter placement was located. **[unknown]**

---

## ★ 2. The finding that makes "notify without interrupting" actually possible

**Jonides & Yantis, *Perception & Psychophysics* 43:346–354 (1988)** (~987 citations): only
**abruptly onset** targets produced shallow search slopes. **Colour and luminance singletons did NOT
capture attention automatically.** **[sourced]**

Confirmed current — *Attentional Capture by Abrupt Onsets* (2025 review, PMC11908675): *"substantial
evidence supports the notion that abrupt onsets are more powerful at capturing attention compared to
color singletons,"* and colour singletons consistently do **not** capture like onsets even when
highly salient. **[sourced]**

> **The interruption is the MOMENT OF APPEARANCE, not the steady state. A colour already present
> cannot involuntarily capture attention.** This is precisely what the Boss's ruling requires, and it
> is why the design is not self-contradictory.

**[unknown] — the load-bearing gap:** whether a *slow fade-in* avoids capture while still reaching a
detectable state. The 2025 review contains **no gradual-onset data**. **Engineering position: do not
rely on ramp speed. Avoid the transient by TIMING** — introduce the signal during a writing pause.

**A computable pause detector, no ML needed.** Keystroke-logging convention: inter-key pauses
**below ~2000 ms** reflect transcription; **above ~2000 ms** reflect planning/revising, and pause
duration rises at larger linguistic boundaries. **[sourced, secondary]** So:
`inter-key interval > 2000 ms` **AND** cursor at a sentence/paragraph terminator. Compare Iqbal &
Bailey's statistical breakpoint models: **52–59% recall**, and 2–42% accuracy at classifying
breakpoint *type*. **A timer beats the models here.**

---

## 3. The interruption evidence — the popular version is wrong, but the Boss is right anyway

### "23 minutes to refocus" does not survive contact with the papers

Mark, Gudith & Klocke, *The Cost of Interrupted Work* (**CHI 2008**), read in full: **the figure does
not appear in it.** Actual results (N=48, interruptions every 2 min):

| Condition | Time on task | Errors |
|---|---|---|
| No interruption | **22.77 min** | 1.94 |
| Same-context interruption | **20.31 min** | 1.93 |
| Different-context interruption | **20.60 min** | 1.84 |

**Interrupted work finished FASTER (p<.05), with no quality difference.** What rose was stress
(6.92→9.46, p<.001), frustration, time pressure and effort. **[sourced]**

The nearest real figure — Mark, González & Harris (**CHI 2005**), 24 information workers: 77.2% of
interrupted work resumed same-day, mean **25 min 26 s, SD 54 min 48 s**. *The SD is more than double
the mean; the average is nearly meaningless as a typical value.* It measures **elapsed clock time
before returning to a topic** (during which workers handled ~2.26 other working spheres) — **not
cognitive recovery.** The popular "23 min" pairing traces to **interviews, not a peer-reviewed
measurement**; Mark's own 2023 book says "25 minutes." **The number is not even internally stable.**
**[sourced]**

**Real measured resumption lag:** Altmann & Trafton (CogSci 2004), N=96 — **3.8 s vs a 1.9 s baseline
≈ two seconds** of reorientation. **[sourced]**

### But creative work is a different regime — and this is where the ruling is vindicated

- **Foroughi, Werner, Nelson & Boehm-Davis, *Human Factors* 2014**, N=54, within-subjects **essay
  writing**: **quality dropped significantly** under interruption in both conditions (during
  outlining AND during writing). *Contrast Mark 2008's email task, where quality was unaffected.*
  **The quality cost appears when the task is GENERATIVE.** **[sourced]**
- **Keus van de Poll et al., *Applied Cognitive Psychology* 2016**, N=45/28, 30-second interruptions
  during story writing: writing speed collapsed to **1.25 chars/s vs 3.51 chars/s** —
  **F(1,44)=81.01, p<.001, η²=.65** (very large) — recovering within **10–15 s**. **[sourced]**
- Puranik, Koopman & Vough, *Journal of Management* 46 (2020), review of **247 publications**:
  findings on interruption frequency and performance are **inconsistent** across the literature.
  **[sourced]**

> **The Boss's ruling is correct for the reason he gave, even though the famous number is a myth.**
> Interrupting *generative* work costs output quality — an effect the office-task literature misses.

### Bailey & Konstan, *Computers in Human Behavior* 22(4) 2006, N=50
Interrupting **during** vs **between** tasks: **3–27% more time, twice the errors, 31–106% more
annoyance, twice the anxiety increase.** **[sourced]**

### Flow — do not build on it
Measurement literature contested (2022 *Frontiers in Psychology* critique of DFS-2/FSS-2 validity);
criticisms include self-report reliance and conflation with enjoyment/engagement. **No replication
record verified. [recalled/secondary] — no recommendation here rests on flow.**

---

## ★ 4. The single most design-relevant finding: return ≠ remember

**Ghibellini & Meier, *Humanities and Social Sciences Communications* 12:962 (2025)** — meta-analysis
of **59 publications**:

- **Zeigarnik effect (better memory for unfinished tasks): NOT SUPPORTED.** Weighted ratio **0.99**
  excluding Zeigarnik's own data (N=38). *"Lacks universal validity."*
- **Ovsiankina effect (tendency to RESUME unfinished tasks): SUPPORTED.** Weighted resumption rate
  **67.00%** (N=21), 66.79% excluding Ovsiankina's own data — well above chance.

**[sourced]**

> **People DO come back to unfinished things. They do NOT remember them better.**
> **A design may rely on a return tendency. It may NOT rely on the user remembering something you
> showed them once.** → The signal cannot be the durable path. The surface the user re-enters is.

---

## 5. Timing vs relevance — and relevance wins

**Boundary-vs-mid-task is robust on AFFECT and weak-to-absent on objective cost.** Read from the
papers, not the abstracts — this inverts the usual summary:

- **Adamczyk & Bailey (CHI 2004)**, N=16: interrupting at best moments cut **annoyance 56%**,
  frustration 49%, raised perceived respect 43% — but *"There were no significant effects on
  Resumption Lag."* **The timing effect was entirely subjective.** **[sourced]**
- **Iqbal & Bailey (CHI 2008)** field deployment: frustration by policy Medium **2.6** < Coarse 3.6 <
  Immediate 4.5 < Fine 5.5 (F(3,52)=6.2, p<.001) — **but NOT significant for programming**, the
  higher-load task. Reaction time 3.07 s vs 4.08 s, **p<0.056 — not significant.** Authors: *"Our
  results did not show that scheduling notifications at breakpoints affects users' resumption time."*
  Mean deferral to reach a breakpoint: **88.6 s.** **[sourced]**

**Relevance had a larger and cleaner effect than timing** — general-interest notifications caused
frustration μ=4.98 vs relevant μ=3.59, **F(1,109)=13.9, p<.001**. And the direction is
counter-intuitive and directly applicable:

> Relevant notifications belong at **Medium/Fine breakpoints — close to the activity.** When a
> *relevant* notification was delivered at a **Coarse** boundary, users **disliked** it: it pulled
> them back into a task they were deliberately leaving. **[sourced]**

**A proposal about the note being written right now is a relevant notification. The evidence says it
belongs NEAR the writing — not batched into a distant queue.**

**The resumption cue is the text itself.** Altmann & Trafton 2004: keeping the primary-task display
perceptually available reduced resumption lag (F(1,22)=5.7, p<.03 at 8 s). Mark's own informants:
*"a blinking cursor at the end of the last typed word can enable one to immediately reorient."*
**→ Never occlude the text. [sourced]**

---

## ⚠ 6. The honest counter-case: quiet usually fails

**This is the section that should worry us most. The evidence that quiet fails is stronger than the
evidence that quiet works.**

### The number that settles it
Rhodes' **Just-In-Time Information Retrieval** long-term logs (MIT thesis; 6 users, 3–7 months, 740
calendar days, 312 active days):

> **186,480 suggestions displayed. 197 followed. A 0.1% engagement rate.** ≈ two per week.

Fewer than **one-third** of experimental subjects viewed *any* Margin Notes suggestion. Rhodes reads
it positively — two valuable finds a week at near-zero cost — and flags the sample as self-selected.
**But as a measure of whether users engage a persistent, well-designed, non-intrusive peripheral
channel: 99.9% of what it offered was never touched.** **[sourced]**

*Counterweight, same source:* in the controlled task, JITIR users viewed **~3× as many documents**
(p<.01) and rated the agent **less distracting than a search engine**. Real value at a real cost.

**And Rhodes states our exact design tension, verbatim:**
> *"It must be non-intrusive. However, it cannot be so non-intrusive as to never be noticed."*

### Habituation is a property of the visual system, not of user diligence
- **Anderson, Kirwan, Jenkins, Eargle, Howard & Vance, CHI 2015** (10.1145/2702123.2702322), fMRI:
  **a dramatic drop in visual-processing activity after only the SECOND exposure**, decreasing
  further. Occurred for ordinary software notifications, not just security warnings. **[sourced]**
- **Vance, Jenkins, Anderson, Bjornn & Kirwan, *MIS Quarterly* 42(2), 2018**: adherence to standard
  warnings **substantially decreased over three weeks** in a real mobile app; **polymorphic**
  (varying-appearance) warnings substantially reduced habituation. **[sourced]**
- Habituation **generalises across similar-looking cues** — a family of similar markers habituates as
  a family. **Cue types are not attentionally independent.** **[sourced]**
- **What resists it:** Bravo-Lillo et al., **SOUPS 2014** — for the two attractors that **forced the
  user to interact with the field containing the change**, habituation did **not** reduce response.
  **Visual salience decays with exposure. Required interaction does not.** **[sourced]**

### Peripheral placement is itself a risk
Pernice, NN/g eyetracking (2018), N=26: *"Legitimate content elements that have certain ad-like
characteristics are ignored, too."* One right-rail region at 25% of content area drew **1 of 132
content-area fixations (0.8%)** — 33× less attention than its size warranted. Triggers: **placement,
decorative treatment, ad-adjacency.** **[sourced]**

*Honest critique:* banner blindness has a real counter-record (Hervet et al. 2011 — most participants
**do** fixate ads). What survives is **fixation without encoding** — *arguably worse news:* the eyes
can land on the cue and it can still fail to register. **[secondary]**

### Deferred queues
Fitz et al., *Computers in Human Behavior* 101 (2019), randomised field experiment **N=237**:
batching **3×/day** improved attentiveness, productivity, mood and perceived control, and lowered
stress. **Critical negative result: the no-notification condition produced HIGHER anxiety and FoMO.
Silence is not free.** **[sourced]** · Queue-abandonment ("review debt") is well attested in
practitioner writing with **no peer-reviewed study found**. **[secondary]**

> **Verdict: the ruling on interruption is right; a pure-quiet design is NOT thereby validated.** It
> trades a measured, bounded, recoverable cost (~2 s resumption; 10–15 s writing-speed recovery;
> affective annoyance) for an **unbounded, unmeasured** one — the observation is never seen. **0.1%
> is what the quiet end looks like in production.**

---

## 7. Vision-science constraints on the signal

### Peripheral colour: the worry is overstated, and SIZE is the fix
Rosenholtz, *Capabilities and Limitations of Peripheral Vision* (Annual Review of Vision Science),
explicitly debunking the popular claim: *"humans are quite reasonable at peripheral color judgments,
so long as the patches are sufficiently large."* The standard cone-density figure exaggerates falloff
(it plots cones per **square** mm; density asymptotes ~4,000/mm²). **[sourced]**

Hansen, Pracejus & Gegenfurtner, *Journal of Vision* 9(4):26 (2009): chromatic detection persists to
**≥50° eccentricity**; **stimulus size is the critical parameter**; **red–green declines more steeply**
than luminance or blue–yellow. **[sourced]**

At normal desktop distance the whole screen is within ~±26°. **Peripheral colour loss is not the
binding constraint. Area compensates for eccentricity: a large low-saturation tint beats a small
saturated dot.**

### ★ Crowding caps the VOCABULARY at one bit
Rosenholtz: crowding, not acuity, is the dominant peripheral limit. **Bouma's law — critical spacing
≈ 0.4–0.5 × eccentricity** (Bouma 1970; Pelli et al. 2004). Radially-aligned flankers interfere ~2:1
more than tangential. And decisively:

> *"Under conditions of crowding, one does not generally have difficulty **detecting** a target"* —
> **crowding impairs IDENTIFICATION, not DETECTION.** **[sourced]**

> **A peripheral signal can reliably carry ONE bit — "something is here." It cannot carry a
> vocabulary.** Distinguishing *which* state requires foveating it. **This is vision science, not
> taste** — and it is reinforced by habituation generalising across similar cue families.

**Buildable spacing rule:** a signal at eccentricity E needs ~**0.45 × E** of empty space around it.
A gutter supplies that; an inline mark does not.

### Feature Integration Theory — cite the successor, not the textbook
Wolfe, *Forty years after feature integration theory* (2020), verbatim: *"if you asked almost any of
the researchers currently working on topics like visual search… about FIT, they would probably tell
you that the model was wrong."* The successor is **Guided Search** (GS6). **[sourced]**

### Accessibility is a hard floor
**WCAG 2.2 SC 1.4.1 Use of Color, Level A**, verbatim: *"Color is not used as the only visual means
of conveying information, **indicating an action, prompting a response**, or distinguishing a visual
element."* W3C confirms it applies to status indicators — the named clauses are exactly our case.
**A colour-only signal is a Level A failure.** **[sourced]**

> **Resolution without adding noise: make POSITION the redundant channel.** A signal that only ever
> appears in one dedicated location is identifiable by *where it is* — position is free redundancy,
> unlike a glyph, which is louder by construction.

---

## 8. The recommendation

**Concept (the horse):** *the note's shape is a claim the note is making about itself; when the
writing stops matching the claim, the note should be able to say so — at a moment the writer chooses
to listen.*

### Where the observation LIVES — not in the signal
Persist it as data (Rule 8, write-time derivation): a row per note carrying the observed shape, **the
evidence that triggered it** ("9 dated entries appended over 6 weeks"), a confidence score, first-seen
timestamp, and state (`open` / `acted` / `dismissed`). **The signal is a VIEW of that row, never its
storage.** Survivable, queryable, reversible.

**Route it into the Reviewer that already exists.** `ReviewStatusPanel.svelte` already carries a
per-note `reason` (`never_reviewed` / `interval_due` / `checkpoint` / `dismissed`), `due_days` and
`days_overdue`. **A shape observation is a new `reason`, not a new subsystem** — satisfying the
no-duplication rule, and putting the observation in a surface the user **enters deliberately**, which
§4 says is the only durable path.

### The SIGNAL — one bit, reserved channel, introduced at a pause
1. **Channel: a dedicated gutter, not the text column.** Currently `display: none` → perfectly silent.
2. **Property: a large, low-saturation tint — not a small saturated dot.** Size preserves peripheral
   chromatic detection. A soft vertical band beside the affected region beats a bright pip.
3. **Hue: blue–yellow axis, never red–green.** R–G falls off most steeply *and* fails for the common
   deficiencies. **Avoid red regardless — red is the alarm register and this is not an alarm.**
4. **Onset: introduce during a detected writing pause** (>2000 ms inter-key at a sentence/paragraph
   boundary), **never mid-burst.** Do not rely on fade duration; rely on *when*.
5. **Redundancy via position + a glyph on approach** — WCAG 1.4.1 satisfied by the reserved location,
   plus a shape rendered at readable size once pointer/caret comes near. Resting state = tint;
   identified state = fully non-colour-dependent.
6. **Vocabulary: exactly ONE passive state.** Capped by crowding physiology, not taste. If graduation,
   anchor-uncertainty and review-due all get gutter colours, none is identifiable peripherally and
   habituation generalises across the family. **Everything else is read foveally, in the panel.**
7. **Ramp, don't dialog** (Rhodes' ramping interface): tint → hover/caret reveals a one-line statement
   of what was observed → click opens the proposal with evidence → act or dismiss. Bail-out at every
   stage; no stage above ~2 s (Miller's two-second rule). *Progressive disclosure, never a wizard.*
8. **Show the EVIDENCE, never just the conclusion** — *"9 dated entries appended over 6 weeks"*, not
   *"this looks like a journal."* The Constellation Way, and what the relevance finding predicts will
   be tolerated.
9. **Suppress below a confidence threshold** — Smart Compose's discipline (p90 latency <60 ms;
   confidence-thresholded triggering; **shows nothing when not confident**). Precision over recall.
   **A proposal that fires on ambiguous evidence habituates the channel and costs it permanently.**

### Reversibility
**Shape is a frontmatter field, changed by explicit user action, and nothing else.** Never rewrite
body content to "convert" a note. Changing shape changes how a note is *presented and templated*, not
what it *contains* — so revert is a single field edit with zero content risk. Log it; keep the
observation row at `state: acted` so history is inspectable.

### The counter-case, honoured — two backstops
- **The Reviewer is the durable path.** Ovsiankina (67% resumption) says users return to unfinished
  things they own; the Zeigarnik null says they will **not** remember unprompted.
- **Cap the queue; let observations age out to QUIET, not to GONE.** Unbounded accumulation is the
  attested failure mode; the Fitz et al. null warns that full silence has its own cost.

### What NOT to do
- **Not in the text column** (heterogeneity, η²p=.69) · **No modal/toast with abrupt onset while
  typing** (the one proven involuntary-capture mechanism) · **No colour vocabulary** (crowding) ·
- **Never colour alone** (WCAG 1.4.1 Level A) · **Never red** · **Never animate, pulse or flash**
  (motion is an undoubted guiding attribute — it *will* capture, which the ruling forbids) ·
- **Never escalate salience over time** — habituation decays after the *second* exposure; escalation
  buys a few exposures then costs the channel. **Escalate PLACEMENT (into the Reviewer) instead.**
- **Never convert the note automatically.**

---

## 9. Silent action vs silent observation — the hypothesis holds, but the real variable is different

**The canonical case, with verified numbers:**
- Ziemann, Eren & El-Osta, *Genome Biology* 17:177 (2016): **19.6%** of publications across 18
  journals with supplementary Excel gene lists carried autocorrect-corrupted gene names.
- Abeysooriya et al., *PLOS Computational Biology* 17(7):e1008984 (2021): **30.9% (3,436/11,117)**,
  2014–2020. **The problem got WORSE.**
- HGNC renamed **27 genes** by 2020 (SEPT1→SEPTIN1, MARCH1→MARCHF1). **The scientific community
  changed its nomenclature to accommodate a spreadsheet's silent behaviour.** **[sourced]**

**Why it was damaging is a three-way conjunction:** the conversion was **silent**, **destructive in
place** (the original string is gone), and **plausible-looking** (a date is a valid cell value, so
nothing downstream flagged it).

**★ Microsoft's eventual fix is the strongest available evidence for the principle:** an opt-out
under File > Options > Data — **and a notification shown before the conversion takes place.**
**The vendor's remediation was to convert a SILENT ACTION into a MARKED ACTION.** **[sourced]**

**But "silent action bad / silent observation safe" is not quite right.** Autosave, background
indexing and crash recovery are silent *actions* and are welcomed. The distinguishing variables, in
order: **(1) does it destroy user data irreversibly?** (Excel yes; autosave no — it preserves)
**(2) is it detectable after the fact?** (Excel no — the corrupt value is plausible) **(3) does the
user's mental model predict it?** (nobody expects a gene symbol to become a date).

Silent observation is safe primarily because it satisfies (1) trivially. **Honest exception:** silent
*classification* later acted upon, or discovered only when it surfaces, is a trust event.
**[recalled — reasoning, not a finding.]**

> **The rule adopted: observe silently · act only on explicit instruction · make every acted change a
> single reversible field.**

---

## 10. What could not be established

- **[unknown]** No study puts interruption cost and noticing benefit **on the same axes** for one
  interface. The IRC model (McCrickard et al., *TOCHI* 10(4) 2003 — Interruption/Reaction/
  Comprehension, eight genres; **the ambient corner is defined by giving up REACTION**) supplies a
  coordinate system and claims equations; the equations were not read and no external validation was
  found.
- **[unknown] — most load-bearing:** whether a **gradual/ramped onset** avoids capture while reaching
  a detectable steady state. No gradual-onset data in the 2025 review. *Hence: avoid the transient by
  timing, not by ramp speed.*
- **[unknown]** Published design rationale for IDE gutter / overview-ruler placement. VS Code's
  overview ruler with dedicated error/warning/info/find colours is verifiably a **reserved strip**,
  but no document explaining the reasoning was found. *(VS Code's own UX guidance does state:
  "Respect the user's attention by only sending notifications when absolutely necessary" — and if the
  user doesn't truly need it, "consider to not show anything and relax." JetBrains ships a severity
  level literally called **"No highlighting (fix available)"**.)* **[sourced]**
- **[unknown]** Interruption duration and quality rubric in Foroughi et al. 2014 (paywalled).
- **[unknown]** Whether the 2025 Zeigarnik/Ovsiankina meta-analysis performed publication-bias
  analysis.
- **Not collected:** alarm human-factors literature (IEC 60601-1-8 priority/colour coding; FAA AC
  25.1322-1) on signal-vocabulary limits, and cross-cultural colour-emotion + RTL mirroring guidance.
  **§7's accessibility section rests on WCAG 1.4.1 only; the "one passive state" cap is derived from
  crowding physiology, not from alarm design.** That literature would corroborate or sharpen it and
  **is worth collecting before build.**
- **Flow** — no replication record verified; no recommendation rests on it.

---

*Research v1, 2026-07-19. Serves the Uninterrupted Stream ruling and the colour-signal
specification. Companion to `Note-Shape-Concept-Research-v1.md` (six-track concept research) and
`Note-Shape-and-Template-Studio-Brainstorm.md` (the durable concept record).*
