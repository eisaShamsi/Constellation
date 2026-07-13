# PJ-068 v3 — The Second Screen: Honest Duplication Audit (Art Director & Team)

**Date:** 2026-07-13 · **Status:** Team opinion delivered; awaiting Boss ruling → /migration.
**Trigger:** Boss (during PJ-090) — "What is the UI/UX team's (Art Director's) opinion, honestly? What else could the SS provide beyond the right sidebar or the main Task core plugin?"
**Method:** workflow `wf_043756ee-352` — Census (duplication map) → 3 Art-Director Options → 3 adversarial honesty Judges → Synthesis. Grounded in the PJ-068 / PJ-068-v2 concept papers + the live code.

---

Verification confirms the material — and confirms the honesty caveats. The 10-facet list in `SecondScreenCockpit.svelte` (lines 127-138) pulls its labels from the exact same translation keys as the main sidebar panels, and only `links` renders a real component; the other nine render the literal string "this facet is wired in the next pass" (lines 200-205). The Tasks toggle calls `toggleTaskReconciled` — a disk write — at two sites (1537, 1681). The library-wide tension report exists in memory but is only ever sliced per-note before display (line 462 comment + line 1421). And the left-dock Reviewer already owns Orphan/Fragile/Due lenses (lines 91-97), which means a naive "health board" would partly re-render something we already have.

Here is the team's honest verdict.

---

# The Art Director's Honest Read on the Second Screen

## 1. The honest verdict on the SS today

Eisa, your suspicion is correct, and I'm not going to soften it. **The Second Screen today is about 90% stubbed duplication wrapped around one real idea.**

The "cockpit" you see when a note is open is a tab bar of ten facets — Links, Properties, Structure, Tags, Sky View, Tasks, Knowledge Health, Provenance, Review Pulse, Source Review. That tab bar is a near-photocopy of your main window's right sidebar. We confirmed it in the code: the tabs literally borrow the *same names from the same translation keys* as the sidebar panels. So even the labels admit they're the same thing.

And it's worse than "the same thing," because **nine of the ten tabs are empty.** Click any of them and you get the text "this facet is wired in the next pass." Nothing is behind them. So today the SS shows you either (a) a copy of a sidebar panel you already have one glance away, or (b) nothing at all.

**What's genuine complement (keep):**
- **The link graph** (the Butterfly / Ledger / Orrery lenses). This is the *one* live facet and the *one* honest complement. It draws your note's living-link neighborhood as a spatial picture — twin-origin blooms, a balance-sheet of link lengths, an orbital map — at a size the 300-pixel sidebar physically cannot render. Honest caveat: the *data* underneath is the same backlinks data the sidebar already fetches. What's new is the *picture*, not the facts. It earns its place only as long as it stays a big-board visual, not "the sidebar's star tab, slightly bigger."
- **The Pin dial** (Follow / Pin / Locked). This is the only thing on the whole SS that a single screen physically cannot do: pin note A on screen 2 while you write note B on screen 1. Small, but genuinely second-screen-only.

**What's duplication (cut):**
- The **eight mirror facets** (Properties, Structure, Tags, Provenance, Review Pulse, Source Review, per-note Health, per-note Sky View) — each is a read-only copy of a sidebar panel. A copy minus editing is strictly worse than the original.
- The **Tasks facet + its write toggle** — the sharpest violation (see §3).
- The **dead and shadowed leftovers** — a legacy "editor panels" clone of the whole sidebar (unreachable), read-only copies of the note you're already editing, an OrgChart clone, and an orphaned companion for the disabled Constellation Map. All of it is either mirror code or unreachable code.

Blunt one-liner for you: **the facet tabs provide nothing beyond the right sidebar. The concept the SS was supposed to embody — the "Presenter Display" — is honored by exactly one facet and betrayed by the nine-tab mirror bar.**

## 2. What the SS should uniquely provide

**The concept in one line:** the Second Screen shows the *hidden dimensions around* the note you're writing — where it sits, how healthy its connections are, and where your thinking can go next — never a mirror of what the main screen already shows.

The true complements — the ones the main window *structurally cannot* put on screen beside your open note:

1. **The living-link graph as a full-glass board** *(exists today, keep and elevate).* Different **representation**: a spatial relational shape instead of a flat list. The main editor's canvas is full of your note's text; the sidebar's graph is a thumbnail. The SS is the only place this relationship-picture gets room to be *read as a shape* and clicked to navigate.

2. **A whole-corpus health board — but scoped honestly** *(build).* Different **altitude**: the sidebar is always about *one* note; nothing in the main window shows the health of your *whole* body of knowledge at once. This is the strongest "beyond the sidebar" answer — **with one honest correction.** Part of what a health board would show (notes due for review, orphans, fragile notes) *already exists* in the left-dock Reviewer. So the SS board must **lead with what the Reviewer does not have**: living-links that are *decaying* (they fade 5%/month unused), links going *dormant*, links whose confidence is *contested*, and — the cleanest verified gap — your *unresolved contradictions and tensions rendered for the whole corpus at once*. We confirmed the full tension report already sits in memory and is *never* shown whole; today it's sliced down to the single open note every time. That whole-view is genuinely absent from the main window, cheap to draw (it reads data we already keep), and glanceable from across the room — one click jumps the main window to the problem.

3. **The Pin/Locked reference** *(exists, keep).* Different **focus**: hold your *source* note on screen 2 while you write its *synthesis* on screen 1. A one-window app can't do this. This is the Synthesis Act made ergonomic.

4. **A whole-Universe map across time — the "Estimation Map"** *(build last).* Different **dimension**: your Sky View shows the graph as it is *now*; it has no past→present→future axis. A zoomed-all-the-way-out view of your knowledge across time — what was created when, which links matured, what's decaying or coming due next — is main-window-impossible. It's the marquee idea, but it's also the least-defined and most research-heavy. It's the *destination*, not the first step.

## 3. The Tasks ruling

**Should the SS show a task checklist you can tick? No. Never.** And this answers PJ-090 directly.

**On PJ-090: cut the toggle. Do NOT ship the cross-window-broadcast fix.** The SS's Tasks toggle writes to your `.md` file on disk. That breaks the single settled law of the Second Screen — *it is read-only, always* (Display-not-Domain). It's also the exact PJ-090 bug: the SS writes, the main window doesn't hear about it, so the main window reverts it. The proposed broadcast patch would make an *illegal write work* — which entrenches the violation instead of removing it. The right fix is to **delete the checkbox.** Ticking tasks is the main-window Tasks plugin's one job; duplicating it on a display is wrong twice over (a duplicate *and* a write).

**Should tasks appear on the SS at all? Yes — but as a signal, never a list:**
- As a **health signal** on the whole-corpus board: "this note carries 3 overdue commitments → at-risk." An urgency flag, not a checklist.
- As a **point on the time map**: task due-dates projected onto the future layer of the Estimation Map.
- When a task *is* your focus, show **the knowledge it points at** (its source notes and their living links), read-only — which, honestly, largely folds into the link graph the SS already draws.

So: tasks *yes*, checklist *never*, toggle *never*, the write *deleted not repaired*.

## 4. Net-new ideas worth your consideration

The genuinely new complements the main window can't host, ranked by how confident we are they aren't secretly duplicates:

- **Whole-corpus tension / contradiction board** (highest confidence — verified absent from main): every unresolved "contradicts," every load-bearing link that's decaying, every contested-confidence relationship, across your whole knowledge — the seeds of your next Synthesis, on one screen. This directly serves the Tension Act.
- **Living-link health at the edge level**: link decay, dormancy, traversal-count and confidence shown *on the connections themselves* in the graph — the "living link as a living object," which no flat panel can do.
- **Cross-note "knowledge diff"** (strongest fresh idea from the options): when two notes are open, render the *space between them* — the links directly joining A and B, their shared neighbors, their common vs. divergent tags. The main split view shows two note *bodies*; nothing renders the *relationship* between them. This is Connection and Tension made visible.
- **The time map** (marquee, but build last): your whole Universe laid out across past→present→future.

Two honesty flags so we don't over-promise: "unlinked mentions / connections you haven't made yet" is **not** net-new — the main window's Backlinks panel already surfaces those, so that idea would be a duplicate. And a "Five Acts compass" for the focus note is interesting but speculative — it needs a check that the signal is actually derivable from stored link data before we'd promise it.

## 5. Recommended next step

**One honest recommendation: re-conceive the SS around 2–3 true complements. Do it in two moves.**

**Move 1 — Cut, now (no ambiguity, high confidence).** Delete the nine stub mirror tabs and the tab bar itself, the Tasks write-toggle, the read-only note copies, the OrgChart clone, and the dead/orphaned companions. Keep the link graph and the Pin dial. This alone removes the PJ-090 bug, kills the duplication you correctly spotted, and honors the concept — *before* we build anything new. It's the safe, obvious first cut regardless of which direction we take next.

**Move 2 — Build the whole-corpus health/tension board first**, scoped to the verified gap (link decay/dormancy, contested confidence, and the whole-corpus tension render the main window never shows), with tasks folded in as an urgency signal, not a list. It's the strongest "beyond the sidebar" surface, it reads data we already keep (cheap, no re-scan), and it passes the no-duplication test by construction because it's *whole-corpus, not per-note*. The time map is real but research-heavy — it's the finale, not the opener.

Where the team split, honestly: two of our three reviewers favored building the full three-zone cockpit (health board → decision/graph zone → time map); one favored the minimalist "one screen, one job — just the graph at full glass" and sending the whole-corpus views back to the main window. The disagreement is really about *ambition vs. restraint*, and both agree on the same cuts and the same Tasks ruling. Our recommendation threads them: **cut hard now, then build the one verified whole-corpus surface — and only expand to the time map if it proves out.**

This needs your ruling — full three-zone cockpit, or minimalist graph-only — and then it goes through `/migration` (it crosses the SS ↔ main-window sync boundary and touches a write path, so it's not a one-file fix). But **Move 1, the cut, can and should proceed on your word alone.**