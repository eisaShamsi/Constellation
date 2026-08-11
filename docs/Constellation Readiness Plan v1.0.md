# Constellation — Readiness Plan

**Version 1.0 | 2026-08-11** · Companion to `Constellation Readiness Review v1.0.md`.
Boss-commissioned: *"Put a readiness plan, by prioritizing all the PJs, and your verdict."*

> Every open PJ is ranked below. Seven new numbers (**PJ-262 … PJ-268**) are allocated here for
> readiness work the review surfaced that had no owner. Ranking is by **distance to publishable**,
> not by severity alone — a HIGH that no user will hit ranks below a MED that breaks the promise.

---

## ⚖️ BOSS RULING — 2026-08-11: M2 FIRST

> *"I want us to tackle 1 + 2 as a priority."* — Eisa, on the two categories that became **M2**:
> ① what is **actively corrupting or losing knowledge**, and ② what puts **notes in the wrong
> universe or makes the index lie about them**.

**The plan's sequence is amended: M2 runs FIRST.** M1 (**PJ-262**, the Living Link disk layer)
and the M0 scope ruling (**PJ-263**) are not cancelled — they are re-sequenced behind M2.

**My recommendation was M1 first; the Boss has ruled M2 first, and that is the decision.** The
argument for it is strong and I record it rather than re-litigate it: M2 is where a defect
reaches the user's `.md` files *today*, on ordinary input, in a product he is using daily. PJ-262
is a promise not yet kept; M2 is damage actively being done. Fixing what is breaking before
completing what is unbuilt is a defensible ordering, and it is his to make.

**One consequence stated once, then dropped:** PJ-262 remains the only *concept* failure in the
product, and every day of use adds more earned link data that lives nowhere durable. Nothing in
M2 reduces that. It should follow immediately behind M2 rather than drifting.

**Sequence as amended:** **M2 → M3 → M1 → M0/M4 → M5 → M6 → M7.**
*(M3 moved up with M2 because PJ-264's triage can only reorder M2's own contents — doing it late
means re-doing M2's ranking.)*

---

## 0. What "READY" means — the exit criteria

Constellation is publishable when all seven hold:

| # | Criterion | Today |
|---|---|---|
| 1 | The product's own promise is true: `.md` files are the source of truth for **everything the user creates or earns** | ❌ earned link data lives only in `search.db` |
| 2 | No known-live app-killer | ✅ **met** — all three closed |
| 3 | No open HIGH that can corrupt a `.md` or misplace a note | ❌ ~2 families open |
| 4 | The backlog is honest — every confirmed finding has a number and a group | ❌ ~100 unnumbered |
| 5 | Every shipped surface is certified against its concept paper, or switched off | ❌ ~25 uncertified |
| 6 | The app survives being killed, and its memory ceiling is known | ❌ 2 of 5 boot criteria unmet |
| 7 | It builds, signs, notarizes and launches on macOS | ❌ never built |

**Two of seven met.**

---

## 1. The plan — seven milestones, in dependency order

### M0 · DECIDE — four rulings, blocking (Boss only, ~1 session of conversation)

Nothing below can be sequenced correctly until these are ruled. They cost the Boss thinking, not
engineering time, and three of them gate other people's work.

| PJ | Ruling needed | Blocks |
|---|---|---|
| **PJ-263** 🆕 | **Scope: what is in v1.0?** Certify the ~25 uncertified surfaces, or cut them | sizes M4, M6 and most of M2 |
| **PJ-224** | Does the ordinary search box federate? | **PJ-207 §13**, which has been blocked for weeks |
| **PJ-253** | Case-fold in the rename cascade — changes which links get rewritten on disk | M2 |
| **PJ-219** | The user-action write class design | M2 |
| **PJ-260** | Mixed line endings in Rust-written frontmatter | M7 (trivial either way) |

> **PJ-263 is the highest-leverage decision available in the entire project.** Nine
> visualisation surfaces exist and four are already off. Constraint as Design — *"every feature
> must justify its existence"* — points at a smaller v1.0. Every item in M4 and much of M2 is
> priced by this answer.

---

### M1 · MAKE THE PROMISE TRUE — the Living Link disk layer

| rank | PJ | why it is first |
|---|---|---|
| **1** | **PJ-262** 🆕 *(APP-KILLER-class · `/migration`-sized · Boss-directed 2026-07-24)* | The only **concept** failure in the product |

**The problem, verified in the tree today:** no LINK file is written anywhere. A link's
`weight`, `confidence`, `traversal_count`, `last_traversed` and `status='archived'` exist only in
`note_links` inside `search.db`. Consequences: **File Over App is violated for the earned half of
every link**, and *"every link operation must be reversible"* is **false** — rebuilding the index
resurrects every archived link, silently reversing the user's decision.

**Why first:** every day of use adds more earned data that lives nowhere durable, and a user
deleting a file called "index" is a reasonable act that currently destroys knowledge. It also
un-blocks M5 — kill-mid-index recovery only matters this much *because* the database is
irreplaceable.

**Shape:** full `/migration` — Architect → Plan → Build → Audit. Touches the write path, the
indexer, rename/move, and sync. `CLAUDE.md` must be amended in the same commit that lands it, and
only then.

---

### M2 · PROTECT THE USER'S FILES — the two families that reach disk

These are ranked above every other HIGH because they damage `.md` files or put notes in the wrong
universe. **20 PJs.**

**② Frontmatter writers that emit unparseable YAML** — output is a sequence with no key, which is
the precondition for every *later* property edit on that note vanishing silently.

| rank | PJ | |
|---|---|---|
| 2 | **PJ-234 + PJ-240** | the blank-line block-drop; three writers still on the old rule while the corrected predicate exists and is used in one file only |
| 3 | **PJ-258** | `listItemsOf` splits on a raw comma instead of the quote-aware splitter |
| 4 | *(from the registers, unnumbered — see PJ-264)* | `update_frontmatter_title` orphans a block-scalar title on rename; backslash escaping in four writers |

**③ The federation boundary — "One Universe, One Location"**

| rank | PJ | |
|---|---|---|
| 5 | **PJ-235 + PJ-254** | `move_item` can physically move a note **into** a linked universe; every rename/move/create tail files a linked universe's note into *this* index |
| 6 | **PJ-255** | six detached DB tails with no generation guard across a universe switch |
| 7 | **PJ-244 · 245 · 246** | three back-fills with the same gap — **one shared helper for all nine**, per the Whole-Ecosystem Fix Law |

**④ Recovery and false-success — the nets that may not catch you**

| rank | PJ | |
|---|---|---|
| 8 | **PJ-236 · 237 · 238** | unbounded net blob · raw-string path match · recovery-blind vacate guard |
| 9 | **PJ-242 · 243** | `Ok({})` on metadata failure, and the frontend latches that as a real read |
| 10 | **PJ-241** | write + reindex + `return true` on frontmatter that never changed |
| 11 | **PJ-247** | a timeout that cannot time out |
| 12 | **PJ-239** | `note_embeddings` non-partial UNIQUE — the shape `sky_nodes` just shed |
| 13 | **PJ-251** | retyping a link never recomputes the target's incoming aggregates |
| 14 | **PJ-222** | the `collect_md_paths` boundary |
| 15 | **PJ-248** Group-1 members (items 3–5, 9, 11) | |

---

### M3 · MAKE THE BACKLOG HONEST

| rank | PJ | |
|---|---|---|
| **16** | **PJ-264** 🆕 *(process · blocks confidence in every other rank)* | **~100 of 177 confirmed sweep findings have never been given a number.** De-duplicate across six registers, drop what is already fixed, number and group the remainder |

**This could reorder everything above it.** Until it is done, the ranking in M2 is the best
sequence over the findings we have *numbered*, not over the findings we *have*. It is read-and-sort
work, not building — one focused session.

> Sequencing note: M3 could equally run **before** M2. I put it second because M2's two families
> are already known, already dangerous, and already understood; but if the Boss prefers certainty
> before effort, swapping M2 and M3 is defensible and costs nothing.

---

### M4 · CERTIFY OR CUT THE SURFACE AREA

| rank | PJ | |
|---|---|---|
| 17 | **PJ-263** (execution half; the ruling is M0) | finish each surviving function's concept-paper §10 checklist, switch off what does not make v1.0 |
| 18 | **PJ-207 §13** | unblocked once PJ-224 is ruled |
| 19 | **PJ-227** | a linked universe's phantom rows exempt from dead-row removal (9 live rows) |
| 20 | **PJ-220** | the `{name:}` workflow form + args delivery |

The concept papers already name each surface's debts — Sky View's hardcoded English strings and
hand-rolled context menu, Review Pulse's Rule-8 filesystem re-walk, Search Hub's missing gate and
unmeasured query latency. **This milestone is mostly finishing work that is already specified.**

---

### M5 · SURVIVE THE WORST

| rank | PJ | |
|---|---|---|
| 21 | **PJ-265** 🆕 | **Kill-mid-index recovery** — boot ship-gate criterion 5, *not implemented*. No duplicate notes, no WAL corruption after a force-quit or power loss |
| 22 | **PJ-268** 🆕 | **Backup & recovery system** — Boss-wanted 2026-06-21; concept paper already written (`docs/concept-papers/Backup-System-Concept-Paper.md`). The worst-case safety net a PKF owes its user |
| 23 | **PJ-266** 🆕 | **Idle RSS ≤ 350 MB** — criterion 3, never measured |
| 24 | **PJ-257** | `props_reparse` fails every boot and re-arms forever over 2 rows, silently |
| 25 | **PJ-233** | the registry names a different universe than the app demonstrably runs |
| 26 | **PJ-110** | localStorage durability — `constellation-wab` needs a durability design, not a JSON file |

**PJ-265 and PJ-268 become far less frightening once M1 lands**, because the database stops being
irreplaceable. That is the dependency argument for M1 first, stated in reverse.

---

### M6 · macOS

| rank | PJ | |
|---|---|---|
| 27 | **PJ-267** 🆕 | build → sign → notarize → smoke-test on Apple silicon |

Sub-tasks, all verified as the actual gaps today: CI is `windows-latest` only · `bundle.macOS` is
**null** (no identity, entitlements, `minimumSystemVersion`, notarization) · the app has **never
been launched on macOS** · ONNX Runtime / bundled SQLite / `memmap2` / the watcher unverified on
Apple silicon · NFD filename normalisation untested · the RTL paragraph-direction gesture
(`paragraphDir.ts:195-205`) needs a Cmd keymap.

**Deliberately after M4.** Porting a feature set you are about to cut is wasted work.

---

### M7 · DEBT, POLISH, DOCS

| rank | PJ | |
|---|---|---|
| 28 | **PJ-256** | no back-fill re-collects its table's statistics — the class behind PJ-249's false headline |
| 29 | **PJ-250** | boot-shaped refresh used as an incremental update |
| 30 | **PJ-259** | the PJ-252 altitude residue: property-TYPE is still answered in two places; block extent still line-decided |
| 31 | **PJ-226 · 225** | ≈24 walkers on `path.is_dir()` · 9 hand-rolled `mtime_secs` copies |
| 32 | **PJ-248** items 13–14 · **PJ-172** Sight timing flakes · **PJ-260** mixed EOL · **PJ-261** doc-drift |
| 33 | Doc-drift watch | translated manuals partial; the PJ-252 manual paragraph is English-only |

---

## 2. Newly allocated — PJ-262 … PJ-268

| PJ | Title | Group |
|---|---|---|
| **PJ-262** | Living Link disk layer — make `search.db` disposable again | 1 · Charter · `/migration` |
| **PJ-263** | Surface-area certification or cut — the v1.0 scope ruling + execution | 3 · **Boss ruling** |
| **PJ-264** | Triage the ~100 unnumbered sweep findings across six registers | 1 · process |
| **PJ-265** | Kill-mid-index recovery (boot criterion 5) | 1 |
| **PJ-266** | Idle RSS measurement (boot criterion 3) | 2 |
| **PJ-267** | macOS: build, sign, notarize, smoke-test | 3 |
| **PJ-268** | Backup & recovery system (concept paper exists) | 3 |

---

## 3. VERDICT

**Constellation is NOT ready to publish. It is closer than its backlog suggests, and further than
its polish suggests.**

**What is genuinely finished** — and this is more than most projects at this stage: the concept,
and it is a *good* concept that no competitor asserts. The file format and portability. The
editor and its content-ownership model. Multilingual and RTL, built in from the ground up rather
than bolted on. Boot performance on a 7,600-note universe. An engineering discipline — 941 + 1445
tests, adversarial audits, an authoring/gating agent pair — that this session watched catch an
app-killer-class regression every one of those tests had missed. **And zero known-live
app-killers**, which is earned.

**What is not finished is not spread thin — it is concentrated in one failure and one decision.**

**The failure:** the Living Link Architecture is half-built. The product's headline promise —
*your files are yours* — is not true of the thing that makes Constellation Constellation. That is
**PJ-262**, and it should start now.

**The decision:** thirty functions ship from a one-person project, ~25 of them never certified by
the Boss's own acceptance program. **PJ-263.** Constraint as Design already contains the answer;
it just has not been applied to the product as a whole the way it is applied to every feature
inside it.

Everything else — 59 HIGH findings, two unmet boot criteria, macOS — is ordinary engineering that
this project has repeatedly proven it can execute well. None of it is conceptually hard. It is
priced by PJ-263 and made safe by PJ-262.

**My recommendation, in one line:** rule PJ-263 this week, start PJ-262 immediately, and do not
touch macOS until both are settled.

**Rough shape, assuming the current pace:** M0 is days. M1 is a full `/migration`. M2+M3 together
are the bulk of the remaining engineering. M4 is priced by the ruling. M5–M7 follow.
**I will not put a date on it** — this project's own history (a "44 ms" that was really 2,579 ms,
a headline number false for a week) is the argument against estimates that have not been measured.
