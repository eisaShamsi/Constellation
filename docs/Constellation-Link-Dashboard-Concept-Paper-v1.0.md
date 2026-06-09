# Constellation Link Dashboard — Concept Paper v1.0

> **⚠️ SUPERSEDED 2026-06-09 by `Constellation-Circulatory-System-Concept-Paper-v1.0.md`.** This was the
> first draft. Cross-checking it against the other core-plugin concept papers (Cognitive-Engine One-Picture,
> CNS, the Base, the Cataloger, 360.3D, Sight Subsystem) revealed two errors it makes: (1) it conflates the
> authoring-side *"Link Dashboard"* archive panel with the diagnostic instrument; (2) it tries to own
> *topology* (Most-Connected / hubs) that belongs to **CNS**. The instrument the project actually lacks is the
> **circulatory** complement to CNS — `weight / decay / dormancy / lifecycle / confidence-flow / archive` — now
> defined as **CCS (Constellation Circulatory System / الجهاز الدوري)**. Preserved here as the historical first
> attempt; read CCS for the canonical concept.

**Drafted 2026-06-09.** Status: **first concept of record.** Before this paper there was no concept, rule,
help article, or design doc for the Link Dashboard — only one passing line in the orientation ("Archive =
soft-delete, reversible via the Link Dashboard's Archived tab"). It grew as a right-sidebar panel without a
specification. This paper exists to give it one, for the Owner to ratify, **before** any `/migration` re-homes
or extends it.

Throughout, **FACT** marks what the code does today; **PROPOSAL** marks what this paper argues it *should*
be. The two are kept separate on purpose (BASIC RULE — nothing here is invented as if it already existed).

---

## 1. What the Link Dashboard is

**PROPOSAL.** The Link Dashboard is **Constellation's diagnostic instrument for the Living-Link graph at the
library- and universe-wide scale** — the one surface where you survey, interrogate, and curate **the links
themselves**, as a population, rather than the links of any single note.

If NotePane is where you *write* knowledge and Sky View is where you *see* its shape, the Link Dashboard is
where you **tend the connective tissue**: what is load-bearing, what is wearing thin, what is decaying, what
is broken, what is isolated, and what has been retired. It is the cockpit of the Living Link Architecture.

In the language of Constellation's Five Acts (Observation → Connection → Tension → Synthesis → Conviction),
the Link Dashboard is the instrument of **Connection at the macro scale** — not making one link, but reading
the health of all of them.

---

## 2. Why it exists

Constellation treats links as **first-class knowledge objects** — "living vessels" carrying eight properties
(Type · Direction · Annotation · Weight · Confidence · Created · Last-Traversed · Traversal-Count), four
confidence levels (hypothesis → evidence → established → contested), and a lifecycle (Spark → Birth → Growth →
Maturity → Dormancy → Renewal/Archival). **FACT** — this architecture is shipped (orientation §4.4; lifecycle
commands in `search.rs`).

But a population of living objects with weight, decay, and a lifecycle is **invisible and untended** unless
something lets you see it whole. The existing note-scoped surfaces don't:

- **Backlinks / Outgoing panels** answer "what links *this* note?" — one note at a time.
- **360.3D** answers "where does *this* note sit?" — one note, deeply.
- **Sky View** shows the graph *spatially* but doesn't rank, age, or curate links.

Nothing answers the **universe-wide, link-population questions**: *Which connections carry my thinking? Which
are decaying from disuse? Which are broken? Which notes are stranded?* That gap is what the Link Dashboard
fills. Without it, the Living Link Architecture is a write-time investment no one can read back at scale.

---

## 3. The questions it answers

**FACT** — the current panel ships **seven sections**, each a library-wide query:

| Section | The question | Living-Link property it reads |
|---|---|---|
| **Most Connected** | Which notes are hubs? | link count per note |
| **Most Traveled** | Which paths do I actually walk? | Traversal-Count |
| **Stale** | Which links are decaying from disuse? | Last-Traversed → effective weight decay |
| **Cross-Library** | Which links bridge libraries? | source/target library (federation) |
| **Broken** | Which links point at nothing? | unresolved target |
| **Orphans** | Which notes are stranded? | zero links |
| **Archived** | What did I retire (and can restore)? | archived flag (soft-delete) |

**PROPOSAL** — a *complete* instrument for the Living-Link property set would also answer (candidate future
sections, not built today):

- **Confidence distribution** — how much of my graph is hypothesis vs evidence vs established vs contested?
- **Typed-link distribution** — the balance of the eight cognitive acts (supports / contradicts / causes /
  exemplifies / generalizes / derives-from / part-of / supersedes) across the universe.
- **Tension** — where do `contradicts` links cluster? (today a separate surface; arguably belongs in view.)
- **Newly born / load-bearing transitions** — links crossing confidence thresholds (≥3 → evidence, ≥10 →
  established).

These are deferred to the design/owner discussion (§11), not asserted as scope.

---

## 4. The outputs of reading it (what you leave with)

Like 360.3D's three reads, the Link Dashboard should leave the user holding a small number of clear takeaways:

- **LOAD-BEARING** — "these connections carry my thinking" (Most Connected + Most Traveled). Protect them.
- **EROSION** — "these are decaying or already broken" (Stale + Broken). Repair, re-traverse, or let go.
- **ISOLATION** — "these notes are stranded" (Orphans). Connect them, or accept their solitude.
- **CURATION** — "this is what I've retired, and I can bring it back" (Archived). Reversible pruning.

If a section doesn't serve one of these reads, it shouldn't be on the instrument (Form-Aligns-To-Purpose).

---

## 5. Design principles

1. **Population, not note.** Every view is library/universe-wide. The moment a view needs an "active note" to
   make sense, it belongs in a note-context panel, not here.
2. **Read-then-act.** Each diagnosed link/note is actionable in place — open it, traverse it, set its
   confidence, archive/restore it — without leaving the instrument.
3. **Reversible curation.** Nothing is destroyed. Archive is soft-delete; every action is undoable (the
   Living Link Architecture's "archival, not deletion" rule).
4. **Write-time truth.** The Dashboard *reads* derived state; it never recomputes the universe on open
   (Performance Rule 8). Counts, decay, and rankings come from the maintained `note_links` index.
5. **Form-Aligns-To-Purpose.** Each section earns its place by answering one of the §4 reads. No filler tabs.
6. **Federation-aware.** In a federated universe it reads across cUniverse libraries (the Cross-Library
   section is the seed of this).

---

## 6. What the Link Dashboard is NOT

This is the heart of the Owner's tension — defining it by contrast with its siblings:

- **NOT the Backlinks / Outgoing panels.** Those are **note-scoped** ("links of *this* note"), and rightly
  live in the right sidebar. The Dashboard is the **population** view. They are complementary, not the same.
- **NOT 360.3D.** 360.3D is one note's full position; the Dashboard is the whole graph's link health.
- **NOT Sky View.** Sky View is the **spatial** rendering of the graph (bubbles, force layout). The Dashboard
  is the **tabular, ranked, curatable** rendering — lists you act on, not a picture you pan.
- **NOT the Index.** The Index browses **terms/vocabulary**; the Dashboard browses **links/connections**.
- **NOT the Cataloger.** The Cataloger classifies notes by **epistemic source & content type**; the Dashboard
  diagnoses **connections**, not classifications.
- **NOT the Constellation Map.** The Map is the **hierarchy** (library → folder → note sunburst); the
  Dashboard is the **link lattice** across that hierarchy.

It is the **only** surface whose subject is the *link as a population* with lifecycle and health.

---

## 7. Its home — the architectural rule

**PROPOSAL (the rule).** The Link Dashboard is a **first-class, full-width surface** — a left-dock Core
Plug-in (like the Cataloger) or a full-page overlay (like 360.3D's full-window mode). **It is never a
note-context side panel.**

**FACT** — today it violates this: it is mounted **only** as a ~300px right-sidebar tab (`rightSidebarTab ===
'links'`), the same strip that holds *this-note* properties and backlinks. A universe-wide instrument crammed
into a note-context strip is the mismatch the Owner flagged. Seven ranked, actionable lists do not fit, and
the location wrongly implies a note scope the data never had.

The right sidebar may still keep a **lightweight, note-scoped** "links of this note" affordance (Backlinks /
Outgoing already do this). The **Dashboard** graduates out.

---

## 8. What it reads and writes (contracts)

**Reads (FACT):** the `note_links` SQLite index (source/target/type/weight/confidence/traversal/last-traversed),
note metadata, and resolution state (broken = unresolved target). All derived/maintained at write time — no
filesystem walk on open.

**Writes (FACT):** through the existing Living-Link lifecycle commands (`search.rs`): `_link_traverse`
(bumps weight + traversal-count), `_link_set_confidence`, `_link_archive` / `_unarchive`, `_link_dormant`,
`_link_decay`. The Dashboard is a **view + action layer** over these; it introduces no new write path.

This matters for the `/migration`: re-homing the Dashboard is a **frontend relocation** (mount point + layout),
**not** a schema or data-flow change — the contracts above are unchanged.

---

## 9. Architectural invariants

- **I1 — Write-time derivation.** The Dashboard never recomputes the link graph on open; it reads the
  maintained index (Perf Rule 8). First-time population, if ever needed, runs in the background.
- **I2 — Reversibility.** Every action is undoable; archive is soft-delete (no hard deletion of links).
- **I3 — No new write path.** It composes the existing lifecycle commands; it doesn't fork link storage.
- **I4 — Note panels stay note-scoped.** Promoting the Dashboard does not absorb or break Backlinks/Outgoing.
- **I5 — Federation-transparent.** Reads honor the active universe's federated library set.
- **I6 — Locale + theme aware.** Full i18n (all 15 locales) + Style-Setter theming, like every core surface.

---

## 10. Current state → target state

**Today (FACT):** `LinkDashboard.svelte` (377 lines), 7 sections, library-wide data (`allLinks` + `allNotes`,
no `activeNotePath`), reached **only** via the right-sidebar Links tab. Undocumented. Localized as
`linkDashboard.*`. Just made openable without an active note (MIG-007 hub fix) — which surfaced that the
*home*, not the gating, is the real issue.

**Target (PROPOSAL):** a first-class surface per §7, with the §3 sections (current seven, plus owner-approved
additions), the §4 reads as its spine, the §5 principles as its discipline. Delivered via a `/migration`
(Architect → Plan → Build → Audit) that is **frontend-only** (relocation + layout + any new sections that
read existing data).

---

## 11. The questions for the Owner

These are the decisions this paper needs ratified before a `/migration` opens:

1. **Home.** Left-dock **Core Plug-in** (peer of the Cataloger — persistent dock button, full page) — or
   **full-page overlay** (peer of 360.3D's full-window — opened from a ribbon/command)? *(Recommendation:
   left-dock Core Plug-in — it's a place you return to, like the Cataloger.)*
2. **Section set.** Keep the current seven as-is? Add any of the candidates in §3 (confidence distribution,
   typed-link distribution, tension, threshold transitions)? Drop any that don't serve a §4 read?
3. **The right-sidebar remnant.** Remove the right-sidebar Links tab entirely once the Dashboard graduates,
   or keep a slimmed note-scoped "links of this note" there?
4. **Name.** Keep "Link Dashboard," or give it an evocative Constellation name in the family of Sky View /
   Sight / Cataloger / 360.3D? *(Optional — the code/i18n use `linkDashboard`; a rename has i18n cost.)*
5. **The MIG-007 hub button.** Until this ships, the Links **Settings** tab's "Open Link Dashboard" button
   opens the current side panel. Keep it (harmless), or re-point/remove it pending the new home?

Once these are answered, this becomes v1.1 (ratified), and a `/migration` Architect doc follows.
