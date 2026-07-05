# The Navigator — Concept Paper (MIG-090 · Revision 2, concept-led)

**Date:** 2026-07-05 · **Status:** **RATIFIED (Boss, 2026-07-05)** — the horse ratified verbatim; **Form C (the blend: Workbench + Intent Bar front door)** chosen; the Search boundary confirmed (Search Hub = formulated-query power tool; Navigator = grammar-free intent + the held set); the entity-wide duplication debt filed as its own backlog entry (**PJ-069**, Pending Jobs v1.16). Next: Architect delta → Plan → build.
**Revision note:** Revision 1 locked the two-pane form on page one. The Boss corrected the order: *"let the concept (the horse) lead the design, not the other way around… easy, simple, yet state-of-the-art, and powerful enough to translate what is in the user's mind… avoid duplication among all Constellation's core plugins/functions. They all should complement each other as a whole entity."* This revision is rebuilt in that order: the whole entity → the gap → the horse → forms derived from it.
**Inputs:** whole-entity map `wf_1d470cb8-9e8` (40 canon + 42 live surfaces, both overlap ledgers) · frontier research `wf_50227623-e2e` · the Architect diagnosis (§§1–5 remain valid).

---

## 1. The whole entity — who owns what (condensed)

Every living surface, its ONE owned question (from its own concept paper / the live map):

| Axis | Surface → owned question |
|---|---|
| **The gate** | **Editor** — "what does this note say; let me change it." **Focus** — capture fast. **Tabs** — hold several open, pick the front one. |
| **Structure** | **File Explorer** — where things live. **OrgChart** — the whole Universe's hierarchy in one picture. **Structure panel** — where this note sits in the composition. |
| **Reaching** | **Quick Switcher** — jump by name. **Search Hub** — interrogate by content/vocabulary/connections (a query language). **Index** — the vocabulary of my thinking, term → notes. |
| **Around one note** | **Backlinks / Outgoing** — the link picture. **Tags tab** — this note's tags. **360°** — the full context + gaps. **Local Sky** — the neighborhood. **Suggested Connections** — the relatives it should link to (one list, five mounts — *the sanctioned reuse pattern*). |
| **The whole picture** | **Sky View** — the graph's shape. **CNS** — regions/bridges/silences. **CCS** — the flow of connections. **Knowledge Health** — vitality + curated worry-lists. **Dashboard** — the ambient overview + All-tags + recents. |
| **Attention** | **Reviewer (Review Pulse)** — "resurface a note at the right moment" (six lenses, system-driven). **Tasks/Open Loops** — unfinished knowledge work. **Calendar** — what did I think on this day. |
| **Sets & synthesis** | **Bases** — turn a set of notes into a live table (the user-configured lens; Boss ruling: tables are Bases). **Five Acts / Workspace Bases** — curated + user saved lenses. **Forge / Canvas** — assemble; hold the half-formed. |
| **Meta** | Cataloger (what kind is this), Federation, Second Screen (a display), Settings/Style Setter/Importer. |

**The entity's existing duplication debt** (found by the map; today's state, before any Navigator work): tag browsing in ~6 places · folder browsing in 4 · recents in 3 · orphan/fragile diagnostics in 5 · "hubs" rendered by 3 surfaces while CNS's paper claims it as "their one home" · note-list rendering hand-rolled in ~9 surfaces · two hand-rolled copies of the confidence menu. *These predate the Navigator question and deserve their own dedup backlog entry — the whole-entity law applies to the whole entity.*

## 2. The gap — what no organ owns

Walk the map against a user's actual day and one question has no owner:

- I know a note's name → Quick Switcher. I can phrase a query → Search Hub. I remember a term → Index. I want a defined table → Bases. The system thinks I should re-examine something → Reviewer picks *for* me. No note open → Dashboard shows ambient recents.
- **But: "the notes about X that I was forming last week — the thread I dropped — what I'm working WITH right now — what needs my hand next"** — the vague, real state a mind is actually in — has NO surface. Search demands a formulated query. Bases demand configuration. Reviewer decides by its own schedule. Tabs evaporate meaning (they hold windows, not intent). Recents are a shadow of it (time-ordered noise, not *my* working set).

The frontier research says nobody outside owns it either: the entire industry ranks by recency because recency is all they record; the working-set/triage lane ("process this set, advance each item") never fused with a notes tool. Constellation is uniquely fueled: it records traversal, stage, maturity, confidence, tension, review state — the *state of the work*, not just the time of the edit.

## 3. The horse

> **The Navigator translates what is in the user's mind into the notes it refers to — and holds that working set while the user works it.**

Three properties, from the Boss's criteria:
- **Easy:** no query grammar (that's Search Hub's power tool), no configuration (that's Bases). Plain words and one-click intents in, notes out.
- **Simple:** ONE surface with two moments — *ask/pick* (translate the intent) and *hold/act* (the set stays, each note carries its next step). Nothing else.
- **State of the art and beyond:** intent translation over local semantic + lexical + state indexes (the e5 embeddings, FTS5, and the epistemic columns — all already persisted); working sets that survive restarts; suggestions the user ratifies, never automation that decides.

**The complementarity contract (the law applied):** the Navigator OWNS translation + the held set. It **composes, never re-implements**: text matching → the Search machinery; term pivots → the Index; tables → Bases; structure → the Explorer; review scheduling → the Reviewer (a "needs my hand" intent *reads* the Pulse's due list — one list, another mount, the Suggested-Connections pattern); acting on a note → the shared handlers every surface uses. Its row rendering should become the shared note-list primitive the map says nine surfaces hand-roll — the Navigator work *reduces* the entity's duplication instead of adding a tenth copy.

## 4. Three forms, derived from the horse (Boss picks)

**Form A — The Workbench (the desk).** A center surface holding *working sets*: I pick notes up (from anywhere — tree, search, a base, a link) or summon a set by intent; the set persists across sessions; each note in the set shows its standing and next step (stale? contested? unlinked?); one-key processing (open, advance stage, link, resolve, set down). Tabs stay the "open windows"; the Workbench is *what I'm working on*. — *Strongest on "hold/act" and the unclaimed working-set lane; intent-translation is its entry gesture.*

**Form B — The Intent Bar (ask-first).** One bar, summoned anywhere (like the Quick Switcher's grammar-free cousin): type plain words — "olive cultivation notes I touched recently", "contradictions I never resolved" — get the set, refine with one-click state chips (forming / stale / contested / orphaned / due), then act or hand off (→ open as tabs, → save as a Base, → send to Reviewer). Holds nothing between sessions. — *Strongest on "translate"; leans on Search/embeddings hardest; weakest on the working-set gap.*

**Form C — A ∘ B (the recommended blend).** The Workbench with the Intent Bar as its front door: ask or pick → a set materializes on the desk → it persists → each item carries its state and one-key verbs → done items are set down. Two moments, one surface, no third concept. — *Covers the full gap; the risk is scope — v1 must stay ruthlessly small (one desk, one bar, a handful of state chips, the shared verbs).*

All three: keyboard-first, ≤50 ms interactions on index reads only (the FAST contract from Revision 1 stands), 15-locale + RTL, mutation-event-live, and **zero new data domains**.

## 5. What this hands off (the duplication ledger from Revision 1's ideas)

- Vocabulary rail → **the Index panel owns terms** (the Navigator may *link* to it).
- Full-text search & query grammar → **Search Hub** (the Intent Bar delegates matching to the same engines; it adds translation, not a rival grammar).
- Tables, saved rule-views, columns → **Bases** (a Navigator set can be *saved as* a Base — an export, not a twin).
- Review scheduling & lenses → **the Reviewer** ("needs my hand" reads the same due list — another mount of one list).
- Hubs/orphans as *analytics* → CNS/Sky/KH keep the pictures; the Navigator only uses state as *chips on notes already in hand*.
- Batch operations → shared handlers (the earlier port ruling stands, whichever form wins).

## 6. Boss decisions

1. **Ratify (or edit) the horse** (§3).
2. **Pick the form:** A (Workbench) / B (Intent Bar) / **C (blend — recommended)** — or steer a different shape entirely; the horse survives any carriage.
3. **The Search boundary:** confirm the line — Search Hub = the formulated-query power tool; the Navigator = grammar-free intent + the held set. (The seam the map flags hardest.)
4. **The entity's pre-existing duplication debt** (§1): file as its own backlog entry (entity-wide dedup pass), separate from MIG-090?

## 7. Process

Ratification → Architect delta (form-specific: surfaces, IPCs — all reads from existing indexes; the old component's deletion inventory carries over) → Plan → build. The old Navigator keeps running untouched until the validated swap.
