# Constellation — Readiness Review

**Version 1.0 | 2026-08-11** · Boss-commissioned: *"a wholistic review/evaluation of Constellation, to make sure it is READY."*

> **Method.** Every claim below was read out of the repository today — source, config, the six
> whole-app sweep registers, the concept papers, the bring-up charter, the boot-perf gate.
> Nothing is recalled. Where I could not establish something, it says so.

---

## 1. The concept — CONFIRMED, and it is unusually well-formed

Constellation's concept is not vague, not retrofitted, and not drifting. It is stated in one
sentence and everything else descends from it:

> **Personal Knowledge *Formulation*, not Management.**
> Management asks *"where did I put that?"* · Formulation asks *"what can I BUILD from what I know?"*
> — `docs/CONSTELLATION-KNOWLEDGE-FORMULATION.md:13-17`

Its purpose: help **one mind connect, challenge, synthesize and build understanding.** Storage
volume is explicitly not the goal; *awareness of connections* is.

**The organising spine — The Five Acts of Knowledge Creation:**

> Observation → Connection → Tension → Synthesis → **Conviction**

Every function must advance at least one Act toward Conviction (a held, defended position).
That is a real test, not a slogan: the concept-paper template gates each function on
*"which Act does it serve?"* and says plainly — *if none, it doesn't belong.*

**The five load-bearing principles**, each stated as a constraint rather than an aspiration:

| | |
|---|---|
| **File Over App** | `.md` + YAML on disk is the source of truth; the app is a window. No proprietary format, no silent writes |
| **Local-First** | all data on device, no telemetry, no account, works fully offline. Sync is the user's choice |
| **The Living Link Architecture** | links are first-class objects with 8 properties, 4 confidence levels, weight earned through use, every operation reversible |
| **Constraint as Design** + **Form-Aligns-To-Purpose** | every feature justifies its existence; every part of every feature justifies its presence |
| **Language-First** | 15 languages simultaneously, 4 RTL first-class, per-line bidi — architectural, not a setting |

**Verdict on the concept: it is sound, coherent, and genuinely differentiated.** The Five Acts
pipeline and the Living Link vocabulary (supports · contradicts · causes · exemplifies ·
generalizes · derives-from · part-of · supersedes) are a real epistemic position, not a feature
list. No mainstream PKM tool asserts this. **This is the product's actual moat.**

---

## 2. Does the built application achieve it? — Five findings, honestly

### 2.1 ✅ The foundations are real, not claimed

File-Over-App, Local-First and Language-First are **implemented, not aspirational**. Notes are
plain `.md`; the Universe is a portable directory; there is no network dependency; 15 locale
files are parity-guarded by `scripts/i18n-parity.mjs` plus a vitest gate. The editor's per-line
bidi is a core CM6 extension. These are the hardest things to retrofit and they were not
retrofitted.

### 2.2 ❌ The Living Link Architecture is HALF-BUILT — and it is the concept's centrepiece

This is the most important finding in this review.

The concept says links are first-class objects whose **weight, confidence, traversal count and
archival state** are durable, and that **every link operation is reversible**. Verified in the
tree today:

- **No code writes a LINK file.** There is no disk layer.
- `traversal_count`, `weight`, `last_traversed`, `confidence` promotions and `status='archived'`
  exist **only in `search.db`** (`note_links`).
- Recomputable from the `.md` files: a link's existence, type, target, annotation. **Everything
  the user *earned* is not.**

Two consequences that cut against the founding principles:

1. **File Over App is violated for the half of the link that matters.** The `.md` is not the
   source of truth for weight/confidence/traversal — the index is.
2. **"Every link operation must be reversible" is currently false.** Because the wikilink stays
   in the note, rebuilding the index **resurrects every archived link as active**, silently
   reversing the user's decision.

`CLAUDE.md` states this plainly and dates it (verified 2026-07-24, re-verified today): *"Do not
cite this section as evidence that link data is durable on disk. It is not."* Closing it is
Boss-directed and `/migration`-sized.

**Implication for publishing: `search.db` is currently a system of record, not a cache.** If a
user deletes it — a wholly reasonable act for something named like an index — they lose earned
knowledge that no backup of their `.md` files can restore. **This is a ship blocker for a
product whose headline promise is "your files are yours."**

### 2.3 ⚠️ The bring-up program that was supposed to certify readiness was never completed

The Boss's own method (2026-06-15) was: disable everything except the editor, then **re-enable
one function at a time, each behind a concept paper whose §10 acceptance checklist must pass.**

Read today: **28 concept papers exist and are well-written.** But their Status lines say —

| function | "Enabled in bring-up" | "Budget met" |
|---|---|---|
| 01 Note Editor | **yes (core)** | ✓ measured |
| 06 Search Hub | **no** | — not measured |
| 12 Sky View | **no** | — not measured |
| 22 Review Pulse | **no** | — not measured; Rule-8 violation named |

**Meanwhile the shipped defaults turn nearly everything ON** (`store.ts:7012`): search, index,
backlinks, outgoing, tags, skyView, orgChart, quickSwitcher, commandPalette, secondScreen,
inspector360, cece, ccs, aiSkills, workspaces, templates, dailyNotes — all `true`. Only
`constellationMap: false` and `semanticSearch: false` are off, plus Sight v3/v4/v6/v7 held off
by compile-time consts (`SIGHT_V2_ENABLED = true`, the rest `false`).

**So the app ships with ~25 functions enabled that the acceptance program never certified.**
That is not the same as saying they are broken — most demonstrably work, and the Boss uses them
daily. It means **"READY" has never actually been measured per-function**, and the papers
themselves name specific unmet debts (missing shared context menus, hardcoded English strings
in Sky View, a Rule-8 filesystem re-walk in Review Pulse).

### 2.4 ⚠️ Two of five boot ship-gate criteria are unmet, one of them structurally

| # | Criterion | Status |
|---|---|---|
| 1 | UI visible ≤ 2.5 s | ✅ ~870 ms |
| 2 | Responsive ≤ 6 s | ✅ 811 ms |
| 3 | Idle RSS ≤ 350 MB | 🔲 **never measured** |
| 4 | Stat-sweep 50 changed files ≤ 3 s | 🟡 detection + repair shipped; formal harness open |
| 5 | **Kill-mid-index recovery** (no duplicate notes, no WAL corruption) | 🔲 **not implemented** |

Criterion 5 is the serious one for publication. A user force-quitting or losing power during an
index pass is ordinary; that path has never been tested. It sits directly on top of §2.2 — the
database that would be damaged is the one holding the non-recomputable link data.

### 2.5 ✅ The engineering discipline is genuinely exceptional — and is the reason this is fixable

941 vitest + 1445 Rust cases. An adversarial safety-inspection workflow where **every candidate
is refuted before it is confirmed**. A test-authoring/test-gating agent pair whose default
verdict is REJECTED. A ledger, an orientation doc, session logs, and a lessons-learned register.

This is not decoration. **In this session alone the gate caught an APP-KILLER-class regression
that all 941 green tests missed.** Very few one-person projects have anything like it.

---

## 3. Surfaces and core plug-ins — the inventory

30 functions across 6 phases (`docs/concept-papers/00-MASTER`):

**Core spine (always on):** Note Editor (NotePane + FocusPane) · File Tree · Tab Bar ·
Properties panel · Outline panel.
**Search + Index:** Search Hub · Index panel (Term Browser) · Quick Switcher.
**Graph & relations:** Backlinks · Outgoing Links · Tags panel · Local Sky.
**Visualisation:** Sky View (PIXI) · Constellation Map (D3 — **off**) · Constellation Sight
(v2 on; v3/v4/v6/v7 **off**) · OrgChart · Inspector360.
**Knowledge curation:** The Cataloger (CECE) · CCS · Knowledge Health · Tasks · Calendar ·
Review Pulse · Tension/Provenance/Source Review · Expression Forge · Sense-Making Canvas.
**Infra & federation:** Federation (cUniverse) · Second Screen · Five Acts notes · Workspace
Bases · Style Setter · Command Palette · Settings · Importer · Universe Manager · Quick Capture.

**Observation — this is a very large surface area for a v0.1.0 one-person product.** Constraint
as Design says every feature must justify its existence. Nine visualisation surfaces exist, four
of which are switched off. For a *publishable* release, the question is not whether each is
good; it is whether each is **finished, certified and supportable**. That is a scoping decision
only the Boss can make, and it is the single highest-leverage decision available.

---

## 4. App-killer bugs — measured, not estimated

Across the **six whole-app sweep registers** (`lab/reports/sweeps/`):

| severity | confirmed rows |
|---|---|
| **APP-KILLER** | **3** |
| HIGH | 59 |
| MED | 79 |
| LOW | 36 |
| **total** | **177** |

### The three APP-KILLERs are ALL CLOSED

| finding | status |
|---|---|
| `yamlDoc.ts:225` — adding a tag deletes the tags already there | **CLOSED today** (PJ-252, Boss-validated) |
| `NoteEditor.svelte:415` — recovery net stamped durable, then deleted | **CLOSED** (PJ-207 §15; fix comment in the current tree) |
| `sources/mod.rs:503` — raw `starts_with("---")` fence detection | **CLOSED** (PJ-207 §15, `fence_offset`) |

**There is no known-live app-killer in Constellation today.** That is a real and earned
statement, and it is the strongest single fact in this review.

### But 59 distinct HIGH sites remain open

Concentrated in four themes, all of which touch the user's files or their index:

1. **Frontmatter writers that corrupt on ordinary input** — the blank-line block-drop
   (`bases.rs:584`, `libraries.rs:2341`), a block-scalar title orphaned by rename
   (`libraries.rs:2254`), backslash escaping. Output is *unparseable YAML*, which is the
   precondition for every later property edit on that note vanishing silently.
2. **The federation boundary** — `move_item` can physically move a note **into a linked
   universe**; every rename/move/create tail files a linked universe's note into *this*
   universe's index. Contradicts "One Universe, One Location" (MIG-108).
3. **Universe-switch races** — six detached DB tails with no generation guard write the
   departing universe's data into the newly-opened one's database.
4. **Empty-success readers** — `universe.rs` returns `Ok({})` on metadata failure and the
   frontend latches it as real.

**And roughly 100 of the 177 findings have never been given a PJ number.** They exist only as
JSON. They are not de-duplicated across runs, so the true distinct count is unknown — and
establishing it is itself an unstarted job.

---

## 5. macOS readiness — NOT READY, but closer than expected

**What is already right** (better than the standing rule feared):

- **Platform-specific shell operations are properly `#[cfg]`-gated** — `constellation_show_in_folder`
  and `open_path` have real macOS arms (`open -R`, `open`), as does the Arabic RSS backend.
- **Keyboard exposure is small.** Only **3** sites read `ctrlKey` without `metaKey`; **29** handle
  both. The real gap is the RTL paragraph-direction gesture (`paragraphDir.ts:195-205`, PJ-106
  §B4), which is explicitly Ctrl-based and will need a Cmd keymap.
- **No hardcoded drive letters or backslash paths in application code** — every hit is in tests
  or doc comments.
- Tauri v2 is cross-platform by construction; `bundle.targets` is `"all"`.

**What blocks it:**

| | |
|---|---|
| **Never built** | CI is `runs-on: windows-latest` **only**. No macOS build has ever been produced |
| **Never run** | No macOS smoke test, ever. Zero evidence the app launches on macOS |
| **No bundle config** | `bundle.macOS` is **null** — no signing identity, no entitlements, no `minimumSystemVersion`, no notarization. An unsigned `.app` on modern macOS is blocked by Gatekeeper |
| **Unverified natively** | ONNX Runtime (`ort`), bundled SQLite, `memmap2`, and the file watcher all need Apple-silicon verification. None has been attempted |
| **Unicode normalisation** | macOS HFS+/APFS decomposes filenames (NFD). The rename cascade handles NFC; NFD arrival is untested |

**Honest estimate: macOS is a genuine work item, not a checkbox** — but the app code is mostly
platform-neutral, so the cost is in build/sign/notarize/test infrastructure rather than a
rewrite. The standing "consider macOS in every decision" rule has done its job.

---

## 6. Verdict — is Constellation READY?

**Not yet. And the gap is narrower than the list above makes it look, because it is concentrated
in four specific things rather than spread across the product.**

### What is genuinely ready
The concept · the file format and portability · the editor and its content-ownership model ·
multilingual/RTL · the test and audit discipline · boot performance on a 7,600-note universe ·
**zero known-live app-killers.**

### The four things standing between here and publishable

**① Make `search.db` disposable again — the Living Link disk layer.**
Until earned link data is durable in the `.md` files, Constellation's headline promise
("your files are yours") is not true of the thing that makes it Constellation. This is
`/migration`-sized and it is the highest-value work available.

**② Certify the surface area, or cut it.**
~25 functions ship enabled that the bring-up program never certified. Either finish the
per-function acceptance checklists, or decide what a v1.0 actually contains and switch the rest
off. Constraint as Design points at the second answer.

**③ Close the frontmatter-writer family and the federation boundary.**
The 59 HIGH findings are not evenly dangerous — these two themes are where the user's *files*
and *universe placement* are at risk. And triage the ~100 unnumbered findings so the backlog is
honest.

**④ Survive being killed.** Boot criterion 5 (kill-mid-index recovery) is unimplemented, and
criterion 3 (idle memory) unmeasured.

### And then, separately: macOS
Build → sign → notarize → smoke-test on Apple silicon. Best done **after** ① and ② so the
platform work targets a settled feature set.

### My recommendation on sequencing

**① before everything else.** It is the only item that is a *concept* failure rather than an
engineering defect, and every day of use adds more earned link data that lives nowhere durable.
② is the decision that determines how much ③ and macOS actually cost — so it should be made
early even if executed later.

**Constellation is a good product with an unusually strong idea and unusually strong
engineering discipline, and it is roughly one migration plus one scoping decision away from
being publishable.** It is not there today.
