# The Note Constellation — Radial Note View (Concept Brief for Design Review)

*A focused concept for the second screen's note-focus view. Prepared as the brief for a UX + concept-artist aesthetic review. Part of the PJ-068 v2 Knowledge Cockpit (`PJ-068-v2-Second-Screen-Knowledge-Cockpit-Concept-Paper.md`).*

---

## 1. What it is

When a note is open in Constellation's **main window** (a clean writing space), the **second screen** — a *read-only companion display* on a second monitor — shows that note's **whole world in one view**: a **radial link-graph** with the open note at the **center**, and **every note connected to it** orbiting around it.

It is not an editor and never writes. It is the *"what surrounds this idea?"* instrument — the note seen not as text, but as a living node in a field of relationships. You glance at it while you write; you click a node and the *main* window navigates there.

## 2. The core visual idea

- **The open note sits at the center** — a single calm anchor (its title; a hint of its state).
- **Backlinks radiate to the LEFT** (*what points here*), **outgoing links radiate to the RIGHT** (*where it points*). This left/right split is the primary read: what feeds this idea vs. what it feeds.
- **Every link is shown** — like the Sky View graph, nothing hidden behind a "+N more." A note with 3 links and a note with 200 links are both fully present, arranged clearly around a full ring.
- **Each orbiting node is a linked note**, encoded by the relationship, not just position:
  - **Color = the typed relationship** — Constellation's cognitive vocabulary of 8 living-link types (+ a neutral default): **supports · contradicts · causes · exemplifies · generalizes · derives-from · part-of · supersedes · associative**. A contradiction *looks* different from a support at a glance.
  - **Size = the link's living weight** — a load-bearing, often-traversed relationship reads heavier than a fresh one; a dormant/decaying link fades.
- **Dense but manageable.** The ring stays clean because the nodes are small and quiet by default; **hovering a node reveals** its note name + relationship (and a spoke back to the center); **clicking it navigates** the main window there.
- **Facet tabs across the top** — the note's other dimensions in one place: Properties · Backlinks · Structure · Tags · Sky View · Tasks · Knowledge Health · Provenance · Review Pulse · 360.3D · Source Review. The radial graph is the default "Links" view; the tabs open the rest. *"Everything about this note, in one view."*
- **A coupling dial** (Follow / Peek / Pin) sits with the note anchor: the view follows the main window's open note, previews on hover, or pins one note as a fixed reference while you roam.

## 3. Why radial / why "Constellation"

The product is named **Constellation** — a Personal Knowledge *Formulation* system whose founding metaphor is *"an extension of the mind."* Knowledge here is not stored, it is **connected, challenged, synthesized**. Links are **living vessels** with type, weight, confidence, and a lifecycle (Spark → Birth → Growth → Maturity → Dormancy → Renewal/Archival). The radial view makes a single idea's **constellation of relationships** literally visible — a small celestial chart of one note's place in the mind. The aesthetic should feel like that: a **calm, luminous star-map of an idea**, not a busy diagram.

## 4. The semantic content the aesthetic must serve

The beauty must carry meaning, never decorate emptily (a house design rule — *Form-Aligns-To-Purpose*):
- The **left/right** axis (feeds-this vs. fed-by-this) must be instantly legible.
- The **9 relationship colors** must be distinguishable yet harmonious, and readable in **both light and dark** themes.
- **Weight** (node size/opacity) should read as "how alive is this bond."
- **Tensions** (contradicts links) are the most cognitively valuable — real thinking lives there — and may deserve to *draw the eye*.
- The **center note** is the subject; everything else is context and must not overpower it.

## 5. Hard constraints

- **Read-only, companion-monitor surface** — it can be more ambient / visually rich than a working editor, and it has the *whole screen* to breathe. But it must never distract from or duplicate the main window's editing.
- **Theme-aware** — must be beautiful in both light and dark.
- **Multilingual + RTL** — note titles can be Arabic, Hebrew, Persian, Chinese, mixed scripts (the center note in the current build is `الجزائر`).
- **Any scale** — must look considered whether the note has **0, 3, 30, or 200+** links.
- **Fast** — Craig Mod's *"speed is a proxy for quality"*; kepano's *files-you-control*. No stutter; renders from already-persisted data.

## 6. Current state (what exists now — to be critiqued)

A first implementation renders: the center note as a small rounded rectangle; all backlinks as small filled dots on the left semicircle and all outgoing as filled dots on the right; dots colored by relationship type and sized by lifecycle tier; hover enlarges a dot and shows its name + a spoke; a color legend below; facet tabs on top. Honest but plain — filled dots on two arcs, functional more than beautiful.

## 7. The Boss's aesthetic target

A **calm, elegant full ring** of small, evenly-spaced circles around the centered note — dense yet uncluttered, with a clear sense of arcs/regions and generous whitespace, the facet tabs quiet across the top. Clean, considered, *beautiful* — the kind of view you'd be happy to leave open on a second monitor all day.

## 8. The question for the review panel

**Focus: the aesthetic.** How should this radial note-constellation *look and feel* to be genuinely beautiful, calm, and on-brand for "Constellation" — while keeping every relationship legible at a glance and at any scale? Concrete recommendations wanted on: composition & the center treatment; node form (filled vs. outline, rings, glow, size/opacity encoding); the 9-color relationship palette (harmonious + distinguishable, light & dark); the left/right and arc structure; typography & labels; whitespace & density at 200+ nodes; motion & hover; how tensions draw the eye; and the overall mood/metaphor (star-map / orrery / mandala / …). Name precedents worth stealing from.
