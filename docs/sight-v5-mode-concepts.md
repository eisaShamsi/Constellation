# Constellation Sight v5 — Mode Concepts Deep-Dive

**Version 0.1 | 2026-05-13 — work in progress.**
**Author**: Eisa Alshamsi · drafted with Claude.
**Companion to**: `Constellation-Sight-Concept-Paper-v3.1.md` (the design contract). At MIG-024 §N close-out this document folds into Concept Paper v3.2 as the authoritative §6 (the modes) expansion.

> **Purpose**: each Sight v5 mode gets a deep articulation across 8 dimensions. The act of writing this surfaces what's missing in the implementation and produces a fine-tune checklist per mode. By the time all 7 modes are locked, Sight v5's design intent is fully codified and the `§N` polish backlog is itemized.

---

## §0 · What this document IS (and is NOT)

**IS**: the canonical articulation of what each of Sight v5's 7 modes means, why it exists, what it shows, what each visual variable encodes within it, the exact placement algorithm, and what the current build is missing.

**NOT**: the visual-foundation Concept Paper (that's `Constellation-Sight-Concept-Paper-v3.1.md`). This doc deepens §6 of that paper.

**NOT**: a tutorial for end users. The User Manual + help topic do that. This doc is for designers / implementers / Claude sessions / future contributors who need to know *why each mode exists and what it should be doing*.

---

## §1 · Global visual invariants (true across all 7 modes)

Six visual variables encode the universe. **Four are constant across mode toggles** (the load-bearing invariants from Concept Paper §7); **two change with the active mode**.

### §1.1 The four constants — same in every mode

| # | Variable | Encodes | Levels | Source data |
|---|---|---|---|---|
| 1 | **Radial position** (center → rim) | **Strata** — your epistemic elevation | L8 Worldview at the central pole, then L7 Perspective · L6 Framework · L5 Theory · L4 Hypothesis · L3 Opinion · L2 Fact · L1 Datum at the rim | `note_meta.stratum` (computed by sky_nodes triggers per MIG-002) |
| 2 | **Size** | **Maturity** — how developed the note is | seed (1.5 px) · sapling (2.5) · evergreen (3.5) · canonical (5) · wilting (2, greyed) | `note_meta.maturity` |
| 3 | **Brightness / alpha** | **Confidence** — how settled the claims are | hypothesis (0.45) · evidence (0.7) · established (1.0); contested overrides to 0.85 | derived from `note_links.confidence` (per-note primary) |
| 4 | **Color** | **State** | ink black `#1a1a1a` (default) · red `#a83232` (contested — has unresolved inbound `contradicts` link) | `note_links.link_type` + `note_links.confidence` |

### §1.2 The two mode-specific variables

| # | Variable | Encodes | Mode-dependent? |
|---|---|---|---|
| 5 | **Angular position** (around the rim) | The active mode's wedge basis | YES — the mode toggle's primary job |
| 6 | **Grouping / clustering** | Emergent dense regions at (wedge × stratum). *"I have lots of this stratum's notes in this organizational/temporal/etc. bucket."* | YES — meaning depends on mode |

### §1.3 What does center-vs-rim mean? — invariant

| Position | Stratum | Meaning |
|---|---|---|
| Central pole | L8 Worldview | Your most synthesized comprehensive stances |
| Near pole | L7 Perspective | Insights, wisdom (*baṣīrah / ḥikmah*) |
| Mid-inner | L6 Framework | Models, laws, doctrines |
| Mid | L5 Theory | Integrated explanatory systems |
| Mid-outer | L4 Hypothesis | Tentative propositions awaiting evidence |
| Outer | L3 Opinion | Probable claims (*ẓann*) |
| Near rim | L2 Fact | Verified propositions |
| At rim | L1 Datum | Raw material — pre-conceptual data |

A dense star at the pole = a worldview note (rare; weighty). A faint sapling at the rim = a recent data clip (lots; lightweight). **The radial axis IS the cognitive depth axis**, regardless of which mode is active.

---

## §2 · The hollow semantic — incompleteness made visible

A star is rendered **hollow** (parchment interior + colored stroke) when it qualifies for one of four "incompleteness" types. The frame color tells you *which* incompleteness, without consuming additional screen real-estate.

### §2.1 Four hollow types + frame colors

| Hollow type | Frame color | Hex | Meaning |
|---|---|---|---|
| **Missing data for the active mode** | gold | `#c9a227` | This note isn't classified on the dimension you're currently looking through. Toggle to a different mode and the star may render solid. |
| **No links** (graph orphan) | blue ink | `#2a4a8c` | The note has zero incoming AND zero outgoing typed links. Invisible to the link-graph; cannot participate in the network. |
| **Missing properties** | amber | `#c9831f` | Zero entries in YAML frontmatter — no `tags`, no `held_by`, no `stratum`, nothing. The note is body-only. |
| **Missing content** | gray | `#888888` | Body length < 50 characters. An empty placeholder. |

**Reserved**: red `#a83232` is the *contested* signal only — never a frame color. Solid red dot = state; nothing else uses red.

### §2.2 Priority cascade — when multiple gaps apply

A single note can hit several incompleteness types at once. Render **one** frame color per the cascade (highest first):

1. **Missing data for the active mode** (gold) — wins because the user is *actively looking through* this dimension right now
2. **No links** (blue ink) — most severe structural defect; the note is invisible to the link-graph
3. **Missing properties** (amber) — affects classification quality across all modes
4. **Missing content** (gray) — least diagnostic; an empty note might be a valid placeholder

So a star that's hollow with a **gold frame in mode P** means "you haven't sourced me." Toggle to mode L; if that same star also has no outgoing links, it now renders with a **blue-ink frame** (mode-data is still flagged but the link-orphan trumps it). The frame color *follows what's most diagnostic right now*.

### §2.3 Thresholds (Eisa-confirmed 2026-05-13)

| Threshold | Value | Notes |
|---|---|---|
| `bodyChars < N` → "missing content" | **N = 50** | A note shorter than 50 chars is treated as a placeholder. |
| "Missing properties" | **Zero entries in frontmatter** | Even one frontmatter key (e.g. `tags: [foo]`) counts as having properties. |
| "No links" | **Zero in + zero out** | Either direction with at least one link disqualifies. |
| "Missing data for active mode" | per-mode `hasModeDataFor(note, mode)` | See each mode's section. |

---

## §3 · Universal calculation algorithm

Every star's placement reduces to one function. The only mode-specific pieces are `bucketKeyFor(note, mode)` and `hasModeDataFor(note, mode)`; everything else is invariant.

```
function placeStar(note, mode, scope_id):

    # ─── 1. Filter (D-V3 scope) ───────────────────────────────────
    if not visibleUnderScope(note, scope_id):
        skip

    # ─── 2. Radial position (mode-invariant) ──────────────────────
    if note.stratum is null:
        skip                                  # unstratified notes hidden
    radius = bandCenterRadius(note.stratum, domeRadius)

    # ─── 3. Angular position (mode-specific) ──────────────────────
    bucket_key = bucketKeyFor(note, mode)
    wedge      = lookupWedge(bucket_key, mode)
    jitter     = fnv1a32(note.path) / 2^32    # deterministic [0, 1)
    azimuth    = wedge.azimuthStart + (wedge.azimuthEnd - wedge.azimuthStart) * jitter

    # ─── 4. Size (mode-invariant) ─────────────────────────────────
    size = { seed: 1.5, sapling: 2.5, evergreen: 3.5, canonical: 5, wilting: 2 }[note.maturity]

    # ─── 5. Brightness / alpha (mode-invariant) ───────────────────
    if note.contested:
        alpha = 0.85
    elif note.confidence ≥ 0.95:
        alpha = 1.0           # established
    elif note.confidence ≥ 0.6:
        alpha = 0.7           # evidence
    else:
        alpha = 0.45          # hypothesis (default)

    # ─── 6. Color (mode-invariant) ────────────────────────────────
    color = note.contested ? '#a83232' : '#1a1a1a'

    # ─── 7. Fill style + frame color (per §2 cascade) ─────────────
    gaps = []
    if not hasModeDataFor(note, mode):
        gaps.push('#c9a227')  # mode-data (gold)
    if note.linkInCount == 0 and note.linkOutCount == 0:
        gaps.push('#2a4a8c')  # no-links (blue ink)
    if note.frontmatterKeyCount == 0:
        gaps.push('#c9831f')  # no-props (amber)
    if note.bodyChars < 50:
        gaps.push('#888888')  # no-content (gray)

    fill        = (gaps.length == 0) ? 'solid' : 'hollow'
    frame_color = (gaps.length == 0) ? null    : gaps[0]    # priority cascade

    # ─── 8. Render ────────────────────────────────────────────────
    x = domeCenterX + cos(azimuth) * radius
    y = domeCenterY + sin(azimuth) * radius
    if fill == 'solid':
        drawCircle(x, y, size, color, alpha)
    else:
        drawHollowCircle(x, y, size, frame_color, alpha)
```

**Performance note**: `gaps.length == 0` for the vast majority of notes in a healthy universe — solid render is the common path. Per-frame cost stays at the §11.1 budget.

---

## §4 · Mode R: Regions

### §4.1 Core function

Wedges by Library — sized proportional to note count, largest first. Each star stays at its stratum band; only its angular wedge changes. Reveals the per-library distribution across the 8 strata at a glance.

For Eisa's Trial-7600 universe: BIOLOGY · PHYSICS · HISTORY · PHILOSOPHY · LITERATURE · MUSIC · COMPUTER SCIENCE · ARCHITECTURE · LINGUISTICS · EARTH SCIENCES · FILM · RELIGION & COMPARATIVE TRADITIONS · علوم عربية · جغرافيا · أدب وتراث · تاريخ عربي وإسلامي · EISA COGNITIVE KNOWLEDGE.

### §4.2 Role within the Epistemic Contents Concept

A Library is your **top-down organizational layer** — your chosen taxonomy of where things go. The Universal Epistemic Content Taxonomy (5 branches × 11 sources) is the **bottom-up scholarly scaffold** — what each note IS, civilizationally.

Mode R is the **bridge between them**. It asks: *"Within my chosen organizational scheme, where does the epistemic content actually live across the strata?"*

A library titled "BIOLOGY" that's almost entirely L1-L2 reveals you have lots of raw biology data but no synthesis — even though you THOUGHT of biology as a developed area. A library titled "PHILOSOPHY" weighted heavily L7-L8 reveals you're doing your synthetic work there. The library names tell your story; Mode R shows whether the strata BACK that story up.

### §4.3 How it helps users understand their cognitive knowledge state

Mode R answers questions no other surface in Constellation can:

- *Which library carries the most cognitive weight?* (biggest wedge)
- *Which libraries are stuck at raw-material collection?* (L1-L2 dominant, no L4+ presence)
- *Which libraries do I synthesize in?* (L5-L8 presence)
- *Which libraries are PRETENDING to be developed?* (named for high-level concepts but populated only with low-stratum notes)
- *Where is my organizational hierarchy lying to me about my actual thinking?*

The "pretending" question is the killer. Most PKM users name folders aspirationally — "Strategic Thinking", "Worldview Development" — and then dump raw clips into them. Mode R makes the gap between aspiration and reality visible.

### §4.4 Uniqueness vs. other PKM systems

| Tool | What it does for libraries | What it does NOT do |
|---|---|---|
| **Obsidian** Graph view | Note-to-note edges, flat | No library aggregation; no stratum dimension |
| **Logseq** sidebar | Folder tree as a list | No visual topology; no cognitive depth |
| **Roam Research** | No folder concept | N/A — flat namespace |
| **Notion** | Workspace pages | No graph; no strata; no library-level visualization |
| **InfraNodus** | Auto-detected Louvain communities | User can't define the communities; they emerge from word co-occurrence — not from how the user organizes |
| **Constellation Mode R** | USER-DEFINED libraries × strata distribution | First PKM that shows "where in MY organizational scheme is each cognitive layer?" |

The differentiator: every other tool makes you choose between **your organization** (folders/libraries) and **structural insight** (graph/clusters). Mode R fuses them — your organization IS the angular axis; structural insight (strata) IS the radial axis. Both readings, one image.

### §4.5 Importance among Constellation core functions

- **Sky View** — live nervous-system topology (force-directed bubble graph)
- **Constellation Map** — file-tree containment as a sunburst (where files LIVE)
- **Search Hub** — point query (where does X EXIST?)
- **Index Panel** — term vocabulary (which terms appear AND where?)
- **Sight modes other than R** — re-cut wedges by Time / Confidence / Stage / Act / Provenance / Link Type

**Mode R's role specifically**: while the Map shows you what's IN each library (containment), Mode R shows you what KIND OF THINKING is in each library (cognitive distribution). The Map answers "where do my files live?" — Mode R answers "where does my reasoning live?" Different questions; complementary surfaces.

Mode R is Sight v5's **lowest-cognitive-load entry point** — every user knows their own library names. It's the "first click" for a new user; it's why R is the default first-launch mode (Concept Paper §6.1).

### §4.6 Visual encoding within Mode R

| Variable | What it means in R |
|---|---|
| Radial position | Strata (invariant — see §1.3) |
| **Angular position** (mode-specific) | **Library** the note belongs to. Wedges sized proportional to library note count, sorted largest-first starting at top (12 o'clock). |
| Size | Maturity (invariant) |
| Brightness / alpha | Confidence (invariant) |
| Color | State (invariant — ink default, red contested) |
| **Fill / frame** | Solid by default; hollow with `#c9a227` gold frame is impossible in R (every note has a library — `hasModeDataFor` always true). Hollow with blue-ink / amber / gray frame still appears for the other gap types. |
| **Grouping** (emergent) | A dense band along one stratum within a library wedge = "this library has many notes at this cognitive depth." A library wedge that's tall (spans many strata bands) = balanced cognitive depth. A library wedge that's narrow-radial-band = single-depth concentration. |

### §4.7 Calculation algorithm — Mode R specifics

```
bucketKeyFor(note, R):
    return note.libraryName ?? '(no library)'

hasModeDataFor(note, R):
    return note.libraryName != null      # always true in practice

wedgeBuckets(notes, R):
    counts = Map of libraryName → note count, computed across all visible notes
    sorted = sort counts descending by count
    total  = sum of counts
    cursor = -π/2                                          # start at top (12 o'clock)
    wedges = []
    for (key, count) in sorted:
        span = (count / total) * 2π                        # count-proportional
        wedges.push({
            key:            key,
            label:          key,                            # library name as-is
            count:          count,
            azimuthStart:   cursor,
            azimuthEnd:     cursor + span
        })
        cursor += span
    return wedges
```

The library wedge order is **deterministic** per Universe snapshot. Adding a note doesn't reorder wedges unless it changes the largest-first ranking.

### §4.8 Implementation gaps / fine-tune checklist

| Feature | Status | Note |
|---|---|---|
| Wedges sized proportional to library note count | ✅ shipped | |
| Library names rendered on the rim | ✅ shipped (fix-3) | |
| Stars positioned within their library wedge | ✅ shipped | |
| Stars stay at the same stratum band across mode toggles | ✅ shipped | |
| Hover-tooltip on a star shows note title | ✅ shipped | |
| Hover-tooltip on a WEDGE (not a star) shows library size + per-stratum breakdown | ❌ **missing** — fine-tune candidate for §N |
| Click on a wedge auto-flips scope to L (Library) and filters to that library | ❌ **missing** — fine-tune candidate for §N |
| Wedge ordering: largest-first only | ⚠️ shipped this way | No alphabetical / user-toggleable sort yet |
| Empty libraries (zero notes) suppressed | ✅ automatic (count=0 → zero-width wedge) | |
| Mixed Arabic + Latin library names render correctly on the rim | ✅ shipped (`dir="auto"`) | |
| Library color tinting (instead of all stars ink-black) | ❌ **deliberately not** | Concept Paper §5.3: color reserved for state (contested = red); library color would conflict |
| Hollow rendering + 4 frame colors per §2 | ❌ **not yet implemented** — ships in §N polish or earlier |

---

## §5 · Mode L: Link Types

### §5.1 Core function

Wedges by **dominant outgoing typed-link kind**. Ten uniform wedges at 36° each, in canonical order — *supports · contradicts · causes · exemplifies · generalizes · derives-from · part-of · associative · supersedes · untyped*. Each note is placed in the wedge of its single most-frequent outgoing link type; ties broken alphabetically. Notes with zero outgoing links land in the **untyped** wedge.

### §5.2 Role within the Epistemic Contents Concept

The 9 typed-link kinds are Constellation's **cognitive vocabulary** (per CLAUDE.md Architecture Principles + the Living Link Architecture). They aren't decorative tags — each maps to a specific epistemological move:

| Link kind | Cognitive move | Civilizational anchor |
|---|---|---|
| **supports** | Warrant / justification | *taṣdīq* adjuncts; evidential reasoning |
| **contradicts** | Counter-position / dialectic | *ikhtilāf*; antithesis |
| **causes** | Causal claim | *ʿillah* (efficient cause); *qānūn al-sababiyyah* |
| **exemplifies** | Instance-of (particular under universal) | *al-juzʾiyy taḥta al-kullī* |
| **generalizes** | Inductive ascent (particular → universal) | *al-istiqrāʾ* |
| **derives-from** | Ancestry / lineage / logical derivation | *ijtihād* chain; *al-naẓar al-burhānī* |
| **part-of** | Mereology / structural composition | *al-juzʾ wa-l-kull* |
| **associative** | Looser semantic kinship (no formal claim) | *al-iqtirān al-dhihnī* (mental association) |
| **supersedes** | This replaces an older claim | *al-nāsikh wa-l-mansūkh* |

Mode L asks: **"What kind of reasoning predominates in my universe?"** Each user has a cognitive signature — a CONNECTOR (mostly supports / derives-from), a DIALECTICAL THINKER (contradicts-heavy), a CAUSAL EXPLAINER (causes), a TAXONOMIST (part-of / generalizes / exemplifies), or a COLLECTOR (associative-dominant). Mode L makes that signature visible.

### §5.3 How it helps users understand their cognitive knowledge state

The diagnostic questions Mode L answers:

- *What reasoning move do I make most?* (largest wedge)
- *Which moves am I AVOIDING?* (empty or near-empty wedges)
- *Where in the strata do my different reasoning moves cluster?*
  - If `supports` is heavy at **L4–L5**: you're warranting hypotheses + theories (healthy)
  - If `supports` is heavy at **L1–L2**: you're piling citations onto raw data — collecting, not synthesizing
  - If `contradicts` is dominant at **L7–L8**: you're a dialectical worldview-builder (rare and valuable)
  - If `derives-from` is sparse at **L6–L8**: your high-stratum notes float free without traceable lineage
- *Is my linking disciplined?* The **untyped wedge size** is a direct measure of how much of your linking is undisciplined — every untyped link is a missed cognitive-vocabulary opportunity.
- *Is my universe a* **monoculture** *or a balanced mind?* A balanced mind shows non-trivial presence across many wedges. A monoculture concentrates 80%+ in one wedge.

The most diagnostic single read: **are your L7–L8 stars solid in `derives-from`, or hollow with a gold frame?** If solid: your worldview has lineage. If hollow: your worldview is unanchored conviction.

### §5.4 Uniqueness vs. other PKM systems

| Tool | What it does for link semantics | What it does NOT do |
|---|---|---|
| **Obsidian** | Wikilinks are UNTYPED — every `[[X]]` is identical | Cannot distinguish reasoning moves; cannot aggregate by type |
| **Logseq** | Untyped wikilinks + page-references | Same gap as Obsidian |
| **Roam Research** | Block references; backlinks; no type | Linking is structural, never semantic |
| **Notion** | Database relations have type — but only via schemas, not free-text linking | Cannot type a link inline in prose |
| **InfraNodus** | Edges are word co-occurrences (frequency) | No notion of typed semantic moves |
| **Tinderbox** | Supports labeled links between notes | No aggregate visualization of the cognitive-move distribution |
| **Academic citation tools** (Zotero / Citavi) | Distinguish "cites" / "refutes" / "extends" | Specific to scholarly papers; no PKM-grade integration with personal notes |
| **Constellation Mode L** | Every note's PRIMARY reasoning move is angularly visible across the entire universe | — |

Constellation is the **first PKM** to make the cognitive vocabulary itself a navigable axis. The closest analogue is academic citation analysis ("this paper cites X to refute Y"), but applied to *personal* knowledge formation.

### §5.5 Importance among Constellation core functions

- **Sky View** shows the network ALIVE — but doesn't aggregate by link type.
- **Map** shows containment — orthogonal to reasoning-move analysis.
- **Search Hub** finds specific links — point query, not aggregate.
- **Backlinks / Outgoing Links panels** show one note's edges at a time — local, not global.
- **Mode L specifically** — the **only surface** that aggregates the user's cognitive moves at universe scale. The Living Link Architecture invested heavily in typing edges (link.rs, the 9 link kinds, MIG-022 §A.2 `supersedes`); Mode L is the surface that makes that investment LEGIBLE.

Without Mode L, the typed-link infrastructure has no aggregate diagnostic — users would type their links carefully but never see what those types collectively SAY about their thinking. **Mode L is what closes the Living Link loop.**

It's the natural **second click** after R: once you've seen *where* your thinking lives (R), the next question is *what kind of thinking is it* (L).

### §5.6 Visual encoding within Mode L

| Variable | What it means in L |
|---|---|
| Radial position | Strata (invariant — see §1.3) |
| **Angular position** (mode-specific) | **Dominant outgoing link type.** 10 uniform wedges at 36° each, in canonical order starting at top (12 o'clock): `supports → contradicts → causes → exemplifies → generalizes → derives-from → part-of → associative → supersedes → untyped`. Order matches the Living Link cognitive-vocabulary canon. |
| Size | Maturity (invariant) |
| Brightness / alpha | Confidence (invariant) |
| Color | State (invariant — ink default, red contested) |
| **Fill / frame** | Solid by default. Hollow with **gold frame** (`#c9a227`) for stars in the `untyped` wedge — they're missing data for the active mode (no typed outgoing links). Hollow with blue-ink / amber / gray for the other gap types per §2 cascade. |
| **Grouping** (emergent) | A wedge that has stars across all strata = "this reasoning move pervades my thinking at every depth." A wedge concentrated at one stratum = "I use this move only at this depth." A nearly-empty wedge = "I rarely make this kind of move." The `untyped` wedge size is the discipline gauge. |

**Connector-line note**: in Mode L, the typed-link connector colors (green/red/gold/blue/slate-blue/gray per §5.4 of the Concept Paper) reinforce the wedge basis — a green line typically connects two notes both in the `supports`/`derives-from` wedge. This is intentional visual coherence: the wedge shows where each star LIVES; the connector colors show how stars RELATE. Both channels carry the same vocabulary.

### §5.7 Calculation algorithm — Mode L specifics

```
bucketKeyFor(note, L):
    return note.dominantLinkType ?? 'untyped'

hasModeDataFor(note, L):
    return note.dominantLinkType != null
    # i.e. solid only if at least one outgoing typed link exists.

wedgeBuckets(notes, L):
    # Canonical order — matches the Living Link cognitive vocabulary
    # (CLAUDE.md Architecture Principles, expanded to 9 + untyped).
    order = ['supports', 'contradicts', 'causes', 'exemplifies', 'generalizes',
             'derives-from', 'part-of', 'associative', 'supersedes', 'untyped']
    counts = Map of linkType → note count, computed across all visible notes
    span   = 2π / 10                                      # uniform 36° each
    cursor = -π/2                                          # start at top (12 o'clock)
    wedges = []
    for key in order:
        wedges.push({
            key:            key,
            label:          $t(`linkTypes.${key}`),       # locale-aware
            count:          counts.get(key) ?? 0,
            azimuthStart:   cursor,
            azimuthEnd:     cursor + span
        })
        cursor += span
    return wedges
```

**Why uniform spans (not count-proportional like R)**: the canonical order is a *vocabulary*, not a frequency. Even an empty `supersedes` wedge needs to be visible at the same angular position every time so the user builds spatial memory: "supersedes lives at ~252°." Count-proportional would shift the wedge angles every render — destroying the muscle memory the four-constants invariant exists to protect.

The `untyped` wedge is INTENTIONALLY at the end (rim position 324°–360°). It's the visible to-do list — its size shrinks as the user types more of their links.

`dominantLinkType` is computed at backfill time in §2 via `(SELECT link_type FROM note_links WHERE source_path = nm.path GROUP BY link_type ORDER BY COUNT(*) DESC LIMIT 1)`. Ties (rare on real data) resolve by SQLite's natural row order at Layer 1 — stable per snapshot, but **arbitrary**.

**Tie-breaking principle** (Eisa-confirmed 2026-05-13): *the cognition analysis should help the user to decide.* The system does NOT silently pick a winner via some "prefer cognitively-significant moves" heuristic — that would inject the system's value judgment into the user's cognitive signature. Instead, ties become a **Layer 2 diagnostic surface**: when N notes have tied dominant link types, the diagnostic findings card surfaces them as "**N notes have ambiguous primary link kind — resolve to refine your reasoning signature**" with a click-through to a tie-resolution interface (likely a side-panel chip set, mirroring the CECE Sibling Disambiguation pattern from MIG-021v3).

This pattern — *system surfaces ambiguity; user resolves it* — is the Layer 2/3/4 contract throughout Sight v5. Mode L's ties are the first concrete instance.

### §5.8 Implementation gaps / fine-tune checklist

| Feature | Status | Note |
|---|---|---|
| 10 wedges in canonical order | ✅ shipped | |
| `dominantLinkType` populated by §2 backfill | ✅ shipped | |
| Stars positioned by dominant link type | ✅ shipped | |
| Rim labels show all 10 link kinds (locale-aware via `linkTypes.*` i18n keys) | ✅ shipped (fix-3) | |
| Connector-line colors per typed kind | ✅ shipped (§5 of Concept Paper §5.4) | |
| Hollow rendering for `untyped` wedge (gold frame per §2 cascade) | ❌ **missing** — fine-tune candidate for §N |
| Wedge-hover tooltip: "supports — N notes — strata breakdown L1: x, L2: y, …" | ❌ **missing** — pending §N |
| Wedge-click → filter visible stars to ONLY that link type's notes | ❌ **missing** — pending §N |
| Brighten ONLY the active-wedge link's connector lines | **REJECTED** (Eisa, 2026-05-13) — keep all 9 typed-link colors visible regardless of mode; the connector colors are a stable second channel that shouldn't shift under mode toggles |
| `untyped` wedge CTA: "Type some of your links via the Outgoing panel →" | **DEFERRED** (Eisa, 2026-05-13) — pending decision; not blocking. Symmetric to mode P's Unsourced CTA if approved later |
| Tie-breaking on `dominantLinkType` — when a note has equal counts of multiple kinds | **Layer 2 diagnostic, NOT Layer 1 silent fix** (Eisa, 2026-05-13). The cognition analysis surfaces ties to the user via the Findings card; user resolves via a tie-resolution panel (mirrors CECE Sibling Disambiguation). At Layer 1, the SQLite natural-order arbitrary-but-stable resolution is acceptable as a placeholder — the user is told *which* notes are tied so they can fix them, not silently nudged |
| When the user adds a `supersedes` link, the wedge should highlight briefly to show "this just changed" | ❌ open design polish — pending §N |

---

---

## §6 · Mode T: Time

### §6.1 Core function

Wedges by **creation month** (0..11). Twelve uniform wedges at 30° each, in calendar order starting at JAN at top (12 o'clock), proceeding clockwise. Each star is placed in the wedge of `note.createdMonth`. Notes with no `created_at` (rare) fall in a "no-month" bucket and render hollow with the gold frame per §2.

The wedge corresponding to **today's month** (May for 2026-05) is subtly gold-tinted (`#c9a227 @ 0.05 alpha`) — the "you are here" temporal anchor. This tint is mode-T-only (gated per fix-4); other modes don't carry it.

### §6.2 Role within the Epistemic Contents Concept

Time is the **temporal axis of cognitive development** — *when* did each insight, datum, hypothesis, theory emerge in your epistemic life. Across the surveyed civilizational traditions:

| Tradition | Time-and-knowledge framing |
|---|---|
| **Greek** (Aristotle, *Posterior Analytics*) | Knowledge develops temporally — *epistēmē* arises through demonstration, which itself unfolds in stages |
| **Sunni Islamic** (*ʿilm al-tārīkh* + Ibn Khaldūn's *Muqaddimah*) | Intellectual history flows in *aṭwār* (epochs) — knowledge is a civilizational arc |
| **Indian** (*pramāṇa-vāda* + *smṛti* as a *pramāṇa*) | Memory (recall of previously cognized content) is itself a means of knowledge — time is a *pramāṇa*-bearing dimension |
| **Western 20th-c.** (Polanyi, *Personal Knowledge*) | Tacit knowledge accumulates over time before becoming explicit — temporal incubation is part of knowing |

Mode T sits at the intersection of the **Universal Epistemic Content Taxonomy** (what kind of content) and the **Five Acts of Knowledge Creation** (Observation → Connection → Tension → Synthesis → Conviction — implicit temporal ordering). It asks: **"When did each layer of my knowing crystallize?"** A heavy L7-L8 wedge in March 2024 means "March was when I locked in worldview-tier thinking." A drought of L5+ stars in recent months means "I haven't synthesized in a while."

The companion data substrate is MIG-022 §B's `note_state_history`. Mode T as it ships in MIG-024 uses `note_meta.created_at` (when the note was *born*). Mode T's Layer 2 evolution will overlay STATE-CHANGE timestamps from the history table — *when did this star migrate from L4 to L5?* — for trajectory analysis. Creation-time today; trajectory-time tomorrow.

### §6.3 How it helps users understand their cognitive knowledge state

The diagnostic questions Mode T answers:

- *When am I most cognitively productive?* (heaviest month wedge)
- *Are there cognitive droughts?* (empty wedges = months you produced nothing)
- *Are RECENT months producing high-stratum notes (synthesis), or only L1-L2 (raw collection)?*
- *Did a particular month produce a worldview-tier insight?* (a single star at L7-L8 in a specific month wedge = "that month was when I locked in that worldview")
- *Is my output decelerating?* (recent months sparse vs older months heavy → cognitive cooling)
- *Is my output accelerating in synthesis?* (recent months heavy at L4-L8 = synthesis bloom)

The cognitive-signature reads Mode T surfaces:

| Pattern | What it means |
|---|---|
| **Burst** — heavy production during certain months only | Sabbatical / deadline-driven thinker |
| **Steady drip** — even distribution across months | Consistent daily-thinking habit |
| **Drought + harvest** — long quiet periods + sudden spikes | Insight-cycle thinker |
| **Decay tail** — heavy old months, sparse recent | Cognitive productivity declining; intellectual cooling |
| **Inverted-stratum recency** — recent months L1-L2 heavy, older months L4-L8 heavy | You're collecting raw material *now* but synthesized *then* — possibly preparing for the next synthesis bloom, possibly stuck |

The most diagnostic SINGLE READ: **are the current month + last 2-3 months populated at L4+?** If yes: actively synthesizing. If only L1-L2 in recent months: collection mode. If empty in recent months: cognitive winter.

### §6.4 Uniqueness vs. other PKM systems

| Tool | What it does for time | What it does NOT do |
|---|---|---|
| **Obsidian** + Daily Notes plugin | A page per day; calendar sidebar | No aggregate temporal view; no stratum × time matrix |
| **Logseq** | Daily Journal as primary org method | Calendar but not stratum-aware; no cognitive-depth-over-time view |
| **Roam Research** | Daily pages central; calendar surface | Same gap as Logseq |
| **Notion** | Timeline / calendar view in databases | Database-scoped; not aggregate-universe; no cognitive depth |
| **Tana** | Daily nodes; date supertags | Similar to Roam — no stratum dimension |
| **DEVONthink** | Notes have dates; date-based smart folders | No aggregate "see your cognitive year at a glance" surface |
| **Constellation Mode T** | **Temporal axis × strata distribution** — when did each cognitive depth emerge across the year? | — |

Constellation is the **first PKM** to show "*WHEN* did each cognitive depth emerge in my universe?" The closest analogue is a researcher's own intellectual diary — but Mode T renders that diary *visually* without forcing the user to scroll through thousands of timestamps.

### §6.5 Importance among Constellation core functions

- **Sky View · Map · Search Hub · Index** — none have aggregate temporal visualization.
- **MIG-022 §B `note_state_history`** — the Rust data substrate (schema + trigger + backfill + query API shipped). Mode T is the **first consumer** of this substrate's temporal data, even at MIG-024 it only reads `created_at`. Layer 2 (MIG-025) will read state-change history for trajectory analysis.
- **Five Acts × Mode T composition** — a high-value Layer 2 diagnostic: are your OLDER months mostly Observation/Connection notes, and NEWER months Synthesis/Conviction? That suggests cognitive maturation. Are NEWER months still mostly Observation? Stuck in collection mode.
- Mode T is the natural **third click** after R (where) and L (what kind) — answering "**when**." It's the temporal closure of the cognitive-portrait triad.

Without Mode T, Constellation's heavy investment in temporal data — the MIG-022 §B history infrastructure, the `created` and `last_traversed` Living Link properties, the MIG-014 lifecycle stages — has no aggregate visual surface. **Mode T is the closure for the temporal axis the way Mode L is the closure for the typed-link axis.**

### §6.6 Visual encoding within Mode T

| Variable | What it means in T |
|---|---|
| Radial position | Strata (invariant — see §1.3) |
| **Angular position** (mode-specific) | **Creation month.** Twelve uniform 30° wedges in calendar order. JAN at top (12 o'clock), then FEB → MAR → APR → MAY → JUN → JUL → AUG → SEP → OCT → NOV → DEC clockwise. Locale-aware month names via `Intl.DateTimeFormat` (handles all 15 locales — JAN renders as "Jan" in en, "يناير" in ar, "1月" in ja, etc.). |
| Size | Maturity (invariant) |
| Brightness / alpha | Confidence (invariant) |
| Color | State (invariant — ink default, red contested) |
| **Fill / frame** | Solid by default. Hollow with gold frame for `createdMonth == null` (very rare). Other gap types per §2 cascade. |
| **Special tint** | The wedge corresponding to **today's month** is gold-tinted at 0.05 alpha. Mode-T-only — other modes don't carry it (per fix-4). The "you are here" temporal anchor. |
| **Grouping** (emergent) | A heavy wedge at one month = "productive month." Stars stacked along one stratum band within a wedge = "I made N notes at this depth in that month." Empty wedges = drought months. Concentration in recent months = active period; concentration in older months = past intensity. |

**Year information is collapsed** in this design — May 2024 and May 2026 land in the same MAY wedge. For a 1–3 year universe (typical Constellation user), this is acceptable; for a 10-year universe the conflation loses granularity. Year-disambiguation is a **Layer 2 enhancement** (year-stacking within wedge via jitter, or year-range slider) — not part of MIG-024.

### §6.7 Calculation algorithm — Mode T specifics

```
bucketKeyFor(note, T):
    return note.createdMonth != null ? String(note.createdMonth) : '?'

hasModeDataFor(note, T):
    return note.createdMonth != null

wedgeBuckets(notes, T, locale):
    fmt    = Intl.DateTimeFormat(locale, { month: 'short' })
    span   = 2π / 12                                          # uniform 30° each
    cursor = -π/2                                              # start at top (JAN)
    wedges = []
    for m in 0..11:
        wedges.push({
            key:            String(m),
            label:          fmt.format(new Date(Date.UTC(2024, m, 15))),
            count:          # count of visible notes where createdMonth == m
            azimuthStart:   cursor,
            azimuthEnd:     cursor + span
        })
        cursor += span
    return wedges

# Special: render the today-month wedge tint
todayWedge():
    m = (new Date()).getMonth()                               # 0..11
    return {
        startAngle: m * π/6 - π/2,
        endAngle:   (m+1) * π/6 - π/2,
        fillStyle:  'rgba(201, 162, 39, 0.05)'                # gold @ 0.05 alpha
    }
    # Drawn ONLY when activeMode === 'T' (fix-4).
```

**Why uniform spans (not count-proportional)**: same reasoning as Mode L — months are a *fixed-cardinality vocabulary* (the calendar), and the user must build spatial memory ("MAY is at 4 o'clock"). Count-proportional would shift wedges every render — losing the muscle memory.

**Year-stacking within wedge** (future Layer 2 enhancement): the deterministic FNV jitter currently spreads stars within a wedge. A future variant could replace `jitter = fnv1a32(path)/2^32` with `jitter = (year - minYear) / (maxYear - minYear)` — putting older years near one edge of the wedge and newer years at the other. Visually: each month wedge becomes a mini-timeline of *that month across years*. Open design call for MIG-025; not in MIG-024 scope.

`createdMonth` is computed at backfill time in §2 via `CAST(strftime('%m', nm.created_at, 'unixepoch') AS INTEGER) - 1`. Stable per snapshot; doesn't change unless the user manually edits `created_at` in frontmatter (rare).

### §6.8 Implementation gaps / fine-tune checklist

| Feature | Status | Note |
|---|---|---|
| 12 month wedges, JAN at top, clockwise | ✅ shipped | |
| Stars positioned by `note_meta.created_at` month | ✅ shipped | |
| Locale-aware month names via `Intl.DateTimeFormat` | ✅ shipped (fix-3) | |
| Today-month gold tint, gated to T mode only | ✅ shipped (fix-4) | |
| Hollow rendering for `createdMonth == null` (gold frame) | ❌ pending §N — depends on hollow-rendering implementation |
| Wedge-hover tooltip: "MAY — N notes — strata breakdown L1: x, L2: y, …" | ❌ pending §N |
| Wedge-click → filter visible stars to that month only | ❌ pending §N |
| **Year-disambiguation within wedge** (year-stacking via jitter, OR year-range slider) | ❌ Layer 2 enhancement — out of MIG-024 scope. Acceptable for ≤3-year universes; problematic for 10+-year universes. |
| Multiple calendar systems on the rim (Hijri / Solar Hijri / Hebrew per v3 paper §7.2) | ❌ deferred — Concept Paper v3.1 carried only Gregorian; multi-calendar support is a future MIG. |
| Today-marker at the exact rim position (a small ring or arrow at today's date, not just the month wedge tint) | ❌ open design polish — pending §N |
| Year-range filter: "last 12 months" / "last 3 years" / "all time" | ❌ Layer 2 — pending MIG-025 |
| Trajectory overlay: read `note_state_history` to show stars that MIGRATED strata over time (e.g., a Theory note that started as a Hypothesis 6 months ago) | ❌ Layer 2 — pending MIG-025; this is the high-value MIG-022 §B consumer |
| Five-Acts × Time composition: "your older months are mostly Observation; your recent months are mostly Synthesis" | ❌ Layer 2 — pending MIG-025 diagnostic findings card |
| Cognitive-cooling alert: "You haven't produced an L4+ note in N months" | ❌ Layer 2 / Layer 3 — pending MIG-025 / MIG-026 |

---

---

## §7 · Mode C: Confidence

### §7.1 Core function

Wedges by **dominant per-note link confidence**. Four uniform wedges at 90° each, in canonical order from top: **hypothesis → evidence → established → contested**. Each star is placed in the wedge corresponding to its most-frequent outgoing link confidence level. Notes with no outgoing typed links default to the `hypothesis` wedge AND render hollow with a gold frame (missing-data-for-mode).

The contested wedge is special: a note is contested if it has any inbound `contradicts` link with non-archived status — regardless of its other-confidence-level outgoing links. Contested OVERRIDES the dominant-confidence calculation. Once a note is contested, it lives in the contested wedge until the dispute is resolved or archived.

### §7.2 Role within the Epistemic Contents Concept

Confidence is the **certainty axis** of cognitive content. It maps directly to **Branch 4 (Epistemic States)** of the Universal Epistemic Content Taxonomy — the most universally calibrated branch across the surveyed traditions:

| Tradition | Confidence-states scale |
|---|---|
| **Greek** (Plato, *Theaetetus*) | *doxa* → *epistēmē* — opinion → knowledge; *aporia* (productive doubt) as a starting state |
| **Sunni Islamic** *kalām* + *uṣūl al-fiqh* (Al-Jurjānī, Al-Ghazālī) | *jahl murakkab* → *jahl basīṭ* → *wahm* → *shakk* → *ẓann* → *iʿtiqād* → *ʿilm* → *yaqīn* (compound ignorance → simple ignorance → illusion → doubt → opinion → belief → knowledge → certainty) — the 8-tier graded scale |
| **Indian** *pramāṇa-vāda* | *prama* (true cognition) vs *aprama* (false/doubtful); *saṃśaya* (doubt) as a defective state requiring resolution |
| **Modern Western** | Bayesian credence; degrees of belief; reliabilism; the JTB (justified true belief) analysis |

Constellation's 4-tier confidence enum (`hypothesis · evidence · established · contested`) is a **deliberate condensation** of the Sunni 8-tier graded scale, calibrated to what's diagnostically useful in PKM. The mapping isn't 1:1:

| Constellation tier | Sunni anchor | Practical meaning |
|---|---|---|
| **hypothesis** | *ẓann* (opinion / probable belief) | Tentative claim; awaiting evidence |
| **evidence** | between *ẓann* and *ʿilm* | Warranted claim; multiple independent supports |
| **established** | *ʿilm* / approaching *yaqīn* | Settled within the user's epistemic frame |
| **contested** | (disputed state) | Orthogonal modifier — can hit ANY tier; a worldview can be contested just as a hypothesis can |

The orthogonality of `contested` is the design tension worth noting: in the taxonomy, "contested" is a *modifier* (a hypothesis can be contested; a worldview can be contested), not a 4th tier. Constellation collapses this into a 4th wedge for visualization simplicity — accepting the trade-off that contested-tier-ambiguity is invisible in Mode C (a contested L7 worldview and a contested L1 datum land in the same wedge but at different radii). The radial axis preserves the stratum information; the dispute information lives on the angular axis.

### §7.3 How it helps users understand their cognitive knowledge state

The diagnostic questions Mode C answers:

- *What's my confidence distribution?* (which wedge is largest)
- *Am I a TENTATIVE thinker* (hypothesis-heavy) *or a CONFIDENT asserter* (established-heavy)?
- *Are my high-stratum notes (L7–L8 worldview) at established confidence — or are they HYPOTHESES I'm treating as worldview?*
- *Are my L1–L2 facts established? They should be.*
- *How much of my universe is contested?* (red wedge size)
- *Do my contested notes cluster at any stratum?*

The cognitive-signature reads Mode C surfaces:

| Pattern | What it means |
|---|---|
| **Hypothesis-dominant + high-stratum tilt** (L7-L8 stars in hypothesis wedge) | Intellectual OVERCONFIDENCE — claiming worldview-tier conviction with hypothesis-tier evidence. **Diagnostic-critical.** |
| **Hypothesis-dominant + low-stratum balance** (L1-L4 in hypothesis; L5+ in evidence/established) | Healthy maturation arc — claims consolidate as they ascend strata |
| **Established-dominant universally** | Either fully settled thinker OR un-self-critical asserter; only deeper analysis can tell |
| **Contested-heavy at L4-L5** | Active dialectical work — *ikhtilāf* engagement; you're THINKING THROUGH disputes |
| **Contested-heavy at L7-L8** | Worldview-tier disputes — your foundational stances are unsettled. May be productive (philosophical inquiry) or destabilizing (existential drift) |
| **Contested-heavy at L1-L2** | You're disputing FACTS — usually wasted dispute energy unless deliberately auditing source reliability |

The most diagnostic SINGLE READ: **the mismatch between stratum and confidence**. High-stratum notes (L6–L8) at hypothesis confidence is the sign of *intellectual overreach*. Low-stratum notes (L1–L2) at contested is the sign of *misallocated dispute energy*. Mode C makes both visible at a glance.

### §7.4 Uniqueness vs. other PKM systems

| Tool | What it does for confidence | What it does NOT do |
|---|---|---|
| **Obsidian** | No native confidence concept | Cannot represent epistemic certainty |
| **Logseq** | No native concept; tags can simulate | No graph-aware confidence |
| **Roam** | No native concept | Same gap |
| **Notion** | Database properties can model confidence | Not graph-aware; no aggregate visualization |
| **InfraNodus** | Edges have weights | Weight ≠ confidence; no epistemic distinction |
| **Bayesian / formal-probability tools** (Pyro, Stan, etc.) | Confidence intervals + credences | For models, not personal knowledge |
| **GRADE** (medical-evidence framework) | Per-claim confidence ratings | Specific to systematic reviews; not PKM |
| **Citation managers** (Zotero, Citavi) | Citation type | No per-claim confidence |
| **Constellation Mode C** | Per-note dominant confidence × strata distribution at universe scale | First PKM to surface confidence as a navigable cognitive-portrait axis |

The closest cross-domain analogue is **academic systematic-review tools** (GRADE: *Grading of Recommendations Assessment, Development and Evaluation*) — but GRADE operates on individual clinical claims, not personal knowledge graphs. Constellation Mode C is the **first PKM-grade aggregation of epistemic confidence** at universe scale.

### §7.5 Importance among Constellation core functions

- Confidence has been a first-class column in `note_links` since the Living Link Architecture (CLAUDE.md "four confidence levels: hypothesis → evidence → established → contested").
- **Mode C is the ONLY surface** that aggregates per-note confidence at universe scale. Backlinks / Outgoing Links panels show one note's edge confidences at a time — local, not global.
- **The Layer 2 composite Mode C × Mode T × Mode A**: *"your recent months show many hypothesis-tier notes at L1-L2 stratum produced via Observation acts — you're in active CONJECTURE phase; expect synthesis in 3-6 months as evidence accrues"*. This is the kind of life-cognitive narrative no PKM has ever offered.
- **Mode C × the contested signal** is uniquely Constellation's: contested isn't a user-set tag but a *graph-emergent* state (computed from inbound `contradicts` link presence). Mode C shows the contested-cluster at a glance — the universe's *dispute terrain*.

Mode C is the natural **fourth click** after R (where) → L (what kind) → T (when) → **C (how certain)**. The confidence axis is the *settledness* dimension of the cognitive portrait.

### §7.6 Visual encoding within Mode C

| Variable | What it means in C |
|---|---|
| Radial position | Strata (invariant — see §1.3) |
| **Angular position** (mode-specific) | **Dominant per-note confidence.** Four uniform 90° wedges in canonical order from top (12 o'clock), clockwise: `hypothesis → evidence → established → contested`. Order follows the *shakk → ẓann → ʿilm → yaqīn* progression with contested as the dispute-modifier at the end. |
| Size | Maturity (invariant) |
| Brightness / alpha | Confidence (invariant). **Note: tautological reinforcement in this mode.** Stars in the hypothesis wedge are dim (0.45); stars in the established wedge are bright (1.0). Both wedge-position AND brightness tell the same story — intentional double-encoding. The wedge is *categorical* (which tier?); the brightness is *visual* (how settled does it FEEL?). |
| Color | State (invariant — ink default, **red contested**). The contested wedge will appear entirely red — third reinforcing channel. Three-way tautology (wedge + alpha + color) is unique to Mode C and is the *most visually emphatic* mode. |
| **Fill / frame** | Solid by default. Hollow with **gold frame** (`#c9a227`) for stars with `confidenceAlpha == null` AND `not contested` — i.e., notes with no outgoing typed links to derive confidence from. Other gap types per §2 cascade. |
| **Grouping** (emergent) | Stars stacked along one stratum band within a confidence wedge = "I have N notes at this depth at this confidence tier." Cross-wedge stratum-mismatch is the diagnostic gold (see §7.3). |

The triple-tautology (angular + alpha + color all encoding confidence) is Mode C's distinctive read. In every OTHER mode, the alpha + color invariants carry second-channel information; in Mode C they reinforce the primary axis. This makes Mode C the *most visually saturated* — the hypothesis quadrant is the dimmest sky region; the contested quadrant is the reddest. The user reads the universe's certainty distribution before consciously parsing the wedges.

### §7.7 Calculation algorithm — Mode C specifics

```
bucketKeyFor(note, C):
    if note.contested:
        return 'contested'             # contested OVERRIDES dominant-conf
    elif note.confidenceAlpha is null:
        return 'hypothesis'            # default for notes with no outgoing links
    elif note.confidenceAlpha >= 0.95:
        return 'established'
    elif note.confidenceAlpha >= 0.6:
        return 'evidence'
    else:
        return 'hypothesis'

hasModeDataFor(note, C):
    return note.confidenceAlpha != null OR note.contested
    # i.e. solid only if the note has actual confidence data
    # (either an outgoing typed link OR an inbound contradicts).

wedgeBuckets(notes, C):
    order = ['hypothesis', 'evidence', 'established', 'contested']
    span = 2π / 4                                              # uniform 90° each
    cursor = -π/2                                              # start at top
    wedges = []
    for key in order:
        wedges.push({
            key:            key,
            label:          $t(`sight.v5.field.${key}`),      # locale-aware
            count:          # count of visible notes whose confidenceBucket == key
            azimuthStart:   cursor,
            azimuthEnd:     cursor + span
        })
        cursor += span
    return wedges
```

**Backfill computation** (currently NOT in §2 — see §7.8 critical gap): the dominant `confidenceAlpha` per note should be derived at backfill time from `note_links`:

```sql
(SELECT
    CASE confidence
        WHEN 'established' THEN 1.0
        WHEN 'evidence'    THEN 0.7
        WHEN 'contested'   THEN 0.85
        ELSE 0.45
    END
 FROM note_links nl3
 WHERE nl3.source_path = nm.path
 GROUP BY confidence
 ORDER BY COUNT(*) DESC
 LIMIT 1) AS confidence_alpha
```

Tie-breaking (e.g. equal counts of `evidence` + `established`) follows the same **ambiguity-surfacing pattern** as Mode L (§5.7): Layer 1 picks SQLite natural order; Layer 2 surfaces the tie via the Findings card; user resolves via a side-panel chip set.

### §7.8 Implementation gaps / fine-tune checklist

| Feature | Status | Note |
|---|---|---|
| 4 wedges in canonical order (hypothesis → evidence → established → contested) | ✅ shipped (geometry) | Wedges render correctly but are not yet POPULATED with data — see next row |
| **`note_meta.confidenceAlpha` populated by §2 backfill** | ❌ **CRITICAL GAP — fix-7 candidate** | The §2 backfill currently writes `NULL AS confidence_alpha`. Result: every star defaults to `hypothesis` bucket + alpha 0.45. The hypothesis wedge is artificially full; evidence/established wedges artificially empty. Mode C is currently *non-functional* on real data despite the chrome rendering. The fix: replace the NULL placeholder with the SQL aggregation above. ½-day work. |
| `contested` flag computed from inbound `contradicts` links | ✅ shipped in §2 backfill | `EXISTS (SELECT 1 FROM note_links WHERE target_path = nm.path AND link_type = 'contradicts' AND confidence != 'archived')` — works correctly thanks to fix-1's `idx_link_target_path` index |
| Star alpha encoded by confidence | ✅ shipped (`alphaForConfidence` in render.ts) | But operates on the NULL data → defaults all stars to 0.45 until fix-7 |
| Red coloring for contested | ✅ shipped | |
| Hollow rendering for missing-confidence stars (gold frame) | ❌ pending §N — depends on hollow-rendering implementation |
| Wedge-hover tooltip: "established — N notes — strata breakdown L1: x, L2: y, …" | ❌ pending §N |
| Wedge-click → filter visible stars to that confidence tier only | ❌ pending §N |
| **Stratum-confidence mismatch alert** (Layer 2 diagnostic): "you have N notes at L7-L8 with hypothesis-tier confidence — intellectual overreach risk" | ❌ Layer 2 — pending MIG-025. **Highest-value diagnostic in Mode C.** |
| **Stratum-confidence mismatch alert (low end)**: "you have N notes at L1-L2 in contested — wasted dispute energy?" | ❌ Layer 2 — pending MIG-025 |
| Confidence trajectory: "your `established` notes were `hypothesis` 6 months ago — your synthesis is consolidating" | ❌ Layer 2 — pending MIG-025; needs `note_state_history` integration |
| Confidence-tie surfacing (Mode L pattern reapplied): notes with equal counts of `evidence` + `established` get surfaced for user resolution | ❌ Layer 2 — pending MIG-025; same ambiguity-surfacing pattern from Mode L |
| Bayesian credence integration: surface "your `evidence` notes have credence 0.6-0.95; established are 0.95+" — for users who want quantitative reading | ❌ open design — possibly Layer 2 polish or future MIG |
| Cognitive-overreach coaching: when high-stratum notes accumulate at hypothesis confidence, prompt "would you like help locking these down with evidence?" | ❌ Layer 4 — pending MIG-027 |

---

---

## §8 · Mode S: Stages

### §8.1 Core function

Wedges by **dominant lifecycle stage**. Six uniform wedges at 60° each, in canonical lifecycle order from top: **Spark → Birth → Growth → Maturity → Dormancy → Archival**, clockwise. Each star is placed in the wedge of its `note_meta.stage` value (extracted from `properties_json` at backfill time). Notes with no `stage` set fall to the `Spark` bucket at Layer 1 AND render hollow with the gold frame per §2 (missing-data-for-mode).

### §8.2 Role within the Epistemic Contents Concept

Stage is the **developmental axis** of cognitive content — *where in its development arc does this note live?* Distinct from Time (T mode = when the note was *born*) and from Confidence (C mode = how *certain* the claims are).

| Stage | Meaning | Cognitive position |
|---|---|---|
| **Spark** | Idea just captured; minimal body; not yet linked | The seed moment — *al-bidāyah* / *dunamis* (potentiality) |
| **Birth** | Actively forming; gaining body; first links emerging | The germinative phase |
| **Growth** | Being developed; substantial body; multiple link relationships | Active cultivation — *al-tarbiyah al-fikriyyah* |
| **Maturity** | Fully developed; settled body; rich link network | Actualized — *energeia* (Aristotle) |
| **Dormancy** | Alive but not actively touched; awaiting reactivation | Latent state — *al-mukmunah* |
| **Archival** | Explicitly archived; preserved but inactive | Curated past — preserved heritage |

The 6-stage arc is shipped as a first-class `note_meta.stage` column (MIG-014), parallel to the Living Link Architecture's link-lifecycle (per CLAUDE.md "Links follow a lifecycle: Spark → Birth → Growth → Maturity → Dormancy → Renewal/Archival"). Mode S surfaces the note-side of that lifecycle.

Cross-civilizational anchors:

| Tradition | Lifecycle/development framing |
|---|---|
| **Aristotelian** | *dunamis / energeia* — potentiality vs actuality; Spark is *dunamis*, Maturity is *energeia* |
| **Sunni Islamic** *baḥth* methodology | *baḥth* (initial inquiry) → *naẓar* (sustained investigation) → *taqrīr* (formulation) → *taḥrīr* (refinement) — the investigative life cycle |
| **Indian *bhāvanā*** | Cultivation through meditation stages — concepts mature through deliberate development |
| **Modern systems thinking** | Software development lifecycle (SDLC); product lifecycle; biological organism lifecycle |

Mode S asks: **"Where in the developmental arc does my universe live?"** — the *maturation distribution* of cognitive work.

### §8.3 How it helps users understand their cognitive knowledge state

The diagnostic questions Mode S answers:

- *Where does my universe live developmentally?* (largest wedge)
- *Am I a STARTER (Spark-heavy) or a FINISHER (Maturity-heavy)?*
- *How much of my universe is Dormant?* (notes alive but untouched — yellow alarm)
- *Where in the strata do my Maturity notes cluster?* (Maturity at L7-L8 = synthesized worldview; Maturity at L1-L2 = curated fact base)
- *Are my Spark notes accumulating without progressing?* (a growing Spark wedge over time = idea hoarding)

The cognitive-signature reads Mode S surfaces:

| Pattern | What it means |
|---|---|
| **Spark-heavy + low Maturity** | Idea hoarder — lots of seeds, little cultivation. Constellation works against this; Mode S makes the imbalance visible |
| **Birth-heavy** | Active germinator — many notes recently formed but not yet matured |
| **Growth-heavy** | Active cultivator — substantial notes being developed; the healthy productive state |
| **Maturity-heavy** | Settled thinker — consolidated knowledge base; risk: stagnation without new Spark/Birth flow |
| **Dormancy-heavy** | Stagnant universe — nothing being touched; cognitive atrophy signal |
| **Archival-heavy** | Curator/archivist — explicitly preserving past work; legitimate role for some users |

Most diagnostic SINGLE READ: **the Spark-to-Maturity ratio combined with stratum distribution**. A 10:1 Spark:Maturity ratio means most ideas die in infancy. A 1:10 ratio means very little new generative work entering the system. Healthy ratio depends on user-role (curator vs creator). The CONCERNING pattern is **Dormancy concentrated at L7-L8** — your foundational worldview-tier thinking has been untouched. May need re-examination given changing context.

### §8.4 Uniqueness vs. other PKM systems

| Tool | What it does for lifecycle | What it does NOT do |
|---|---|---|
| **Obsidian** | No lifecycle concept; plugins (Note Refactor, Archiver) exist | No first-class developmental arc; no aggregate visualization |
| **Logseq** | No lifecycle | Same gap |
| **Roam Research** | No lifecycle | Same gap |
| **Notion** | Status fields (To Do, In Progress, Done) via database properties | Not graph-aware; not aggregate-universe; not cognitive |
| **Tana** | Status supertags | Tag-based; not first-class; no aggregate view |
| **Andy Matuschak's "evergreen notes"** | Informal stage concept (seedling → budding → evergreen) | Convention only; no enforcement; no aggregation |
| **Roam-style daily-notes** | Date-tracking but not stage-tracking | Time ≠ stage |
| **Software project boards** (Trello, Linear) | Kanban statuses | For projects, not knowledge pieces |
| **Constellation Mode S** | Lifecycle-stage × strata distribution at universe scale | First PKM to make note-development arc a navigable cognitive axis |

The closest cross-domain analogue is **software project Kanban** (Backlog → In Progress → Done) — but applied to *individual knowledge pieces* rather than tasks. Constellation Mode S is the **first PKM-grade lifecycle visualization** at universe scale.

### §8.5 Importance among Constellation core functions

- MIG-014 shipped lifecycle stage as a first-class `note_meta.stage` column.
- The `stage` is editable per-note via the Properties panel (PropertyEditor) — already shipped pre-MIG-024.
- **Mode S is the ONLY surface** that aggregates stage at universe scale. Properties panel shows one note's stage at a time; Mode S shows the whole-universe distribution.
- **Mode S × Mode T composition** (Layer 2): "*your old creation-time months are mostly Maturity stage; recent months are mostly Spark — your idea factory is producing but not finishing*". Powerful diagnostic.
- **Mode S × Mode A composition** (Layer 2): "*Acts of Observation tend to be Spark-stage; Acts of Synthesis tend to be Growth+; Acts of Conviction tend to be Maturity+*" — a healthy lifecycle-acts mapping should hold; deviations are diagnostic signals.
- **Mode S × Mode L composition** (Layer 2): "*your `derives-from`-heavy notes cluster at Maturity (good — derived knowledge consolidates); your `contradicts`-heavy notes cluster at Birth (you're actively engaging disputes)*".

Mode S is the natural **fifth click** after R (where) → L (what kind) → T (when) → C (how certain) → **S (how developed)**.

### §8.6 Visual encoding within Mode S

| Variable | What it means in S |
|---|---|
| Radial position | Strata (invariant — see §1.3) |
| **Angular position** (mode-specific) | **Dominant lifecycle stage.** Six uniform 60° wedges in canonical order from top (12 o'clock), clockwise: `Spark → Birth → Growth → Maturity → Dormancy → Archival`. Order matches the natural developmental arc. |
| Size | **Maturity (the SIZE encoding — note: NOT the same as the "Maturity" stage wedge!).** See §8.6.1 below. |
| Brightness / alpha | Confidence (invariant) |
| Color | State (invariant — ink default, red contested) |
| **Fill / frame** | Solid by default. Hollow with gold frame for stars with `stage == null`. Other gap types per §2 cascade. |
| **Wedge background** (Mode S only — Eisa-confirmed) | **Spark · Birth · Growth · Maturity** wedges → standard parchment background (`#faf6e8`). **Dormancy · Archival** wedges → muted weathered cream (`#f0e9cb`) to visually encode "less alive." This is *categorical data encoding*, not decoration — distinguishes the active half of the lifecycle from the inert half at first glance. |
| **Grouping** (emergent) | Heavy at one stage = "this is where most of my universe lives developmentally." Stratum-cluster within stage wedge: "my Maturity notes cluster at L4-L5 (theory-tier maturity)" or "my Spark notes cluster at L1-L2 (raw idea seeds)." Combined Dormancy + Archival area = the inert portion of the universe — visually muted via the wedge background per row above. |

### §8.6.1 Critical naming clarification — "Maturity" as size vs stage

Constellation has **two distinct concepts** that both use the word "Maturity," and they MUST not be conflated:

| Concept | Source field | Type | What it represents |
|---|---|---|---|
| **Maturity (size encoding)** | `note_meta.maturity` enum | `seed · sapling · evergreen · canonical · wilting` | **Network importance** / link density. Computed by sky_nodes triggers from inbound link counts + age. ENCODED AS STAR SIZE (1.5 → 5 px). |
| **Maturity (stage wedge)** | `note_meta.stage` value `'Maturity'` | One of 6 stage strings | **Developmental phase** of the note's lifecycle. User-set via Properties panel. ENCODED AS ANGULAR WEDGE in Mode S. |

A star CAN be at **stage = Spark** (recent idea, undeveloped) AND **size = evergreen** (lots of inbound links because many notes reference it). This is a real signal: "*a Spark-stage note that's already a network hub*" = an idea you captured recently that other notes already build on. Conversely: **stage = Maturity** + **size = wilting** = "*a fully-developed note that nobody links to anymore*" — possibly stale, possibly orphaned.

**The two-axis mismatch is itself diagnostic** — Layer 2 should surface notable mismatches as findings.

### §8.7 Calculation algorithm — Mode S specifics

```
bucketKeyFor(note, S):
    return note.stage ?? 'Spark'        # default for unstaged

hasModeDataFor(note, S):
    return note.stage != null
    # Solid only if user has explicitly set the stage; unstaged
    # notes render hollow with gold frame even though they default
    # to Spark wedge for placement.

wedgeBuckets(notes, S):
    order = ['Spark', 'Birth', 'Growth', 'Maturity', 'Dormancy', 'Archival']
    span = 2π / 6                                              # uniform 60° each
    cursor = -π/2                                              # start at top
    wedges = []
    for key in order:
        wedges.push({
            key:            key,
            label:          $t(`stage.${key.toLowerCase()}`) || key,   # locale-aware
            count:          # count of visible notes whose stage == key
            azimuthStart:   cursor,
            azimuthEnd:     cursor + span
        })
        cursor += span
    return wedges
```

**Backfill**: `note.stage` is computed at §2 backfill via `json_extract(nm.properties_json, '$.stage')`. This already works correctly (unlike Mode C's `confidence_alpha` NULL bug — Mode S has no equivalent gap). Stage is user-set in PropertyEditor; saved to frontmatter as a string value.

**Why uniform spans (not count-proportional)**: same reasoning as Modes L + T + C — the 6 stages are a *fixed-cardinality vocabulary* (the lifecycle), and the user must build spatial memory ("Maturity is at 4 o'clock"). Count-proportional would shift wedges every render — destroying muscle memory across mode toggles.

### §8.8 Implementation gaps / fine-tune checklist

| Feature | Status | Note |
|---|---|---|
| 6 wedges in canonical lifecycle order (Spark → Archival) | ✅ shipped (geometry) | |
| Stars positioned by `note_meta.stage` | ✅ shipped | Uses `json_extract(properties_json, '$.stage')` per §2 backfill |
| `stage` extracted via json_extract in §2 backfill | ✅ shipped | |
| PropertyEditor allows users to set `stage` | ✅ shipped (pre-MIG-024) | |
| Hollow rendering for unstaged notes (gold frame) | ❌ pending §N — depends on hollow-rendering implementation |
| Wedge-hover tooltip: "Maturity — N notes — strata breakdown" | ❌ pending §N |
| Wedge-click → filter visible stars to that stage only | ❌ pending §N |
| Locale-aware stage label rendering (currently raw English `Spark`/`Birth`/etc.) | ⚠️ partial | Verify `stage.*` i18n keys exist + are populated across 15 locales — likely a gap |
| **Spark-to-Maturity ratio diagnostic** (Layer 2): "you have N Sparks for every Maturity — possibly idea hoarding" | ❌ Layer 2 — pending MIG-025 |
| **Dormancy at high-strata alert** (Layer 2): "N L7-L8 notes have been Dormant > 6 months — your worldview tier is stagnant" | ❌ Layer 2 — pending MIG-025 |
| **Stage × Time composition** (Layer 2): "*your old months produce Maturity; recent months produce Spark*" | ❌ Layer 2 — pending MIG-025 |
| **Stage × Acts composition** (Layer 2): "*Observation acts cluster at Spark; Synthesis acts cluster at Growth+*" | ❌ Layer 2 — pending MIG-025 |
| **Stage × Maturity-size mismatch alert** (Layer 2): "N stars are stage=Maturity but size=wilting (mature notes that nobody links to anymore)" — see §8.6.1 | ❌ Layer 2 — pending MIG-025; the two-axis mismatch is a unique diagnostic |
| **Stage-progression coaching** (Layer 4): "N notes at Birth haven't progressed in months — would you like help developing one?" | ❌ Layer 4 — pending MIG-027 |
| **Dormancy resurrection coaching** (Layer 4): "want help re-examining these old worldview notes?" | ❌ Layer 4 — pending MIG-027 |
| Archival workflow integration: clicking an Archival-wedge note shows "view archival history / restore" | ❌ open design — pending §N |
| **Muted wedge background for Dormancy + Archival** (Eisa-confirmed 2026-05-13) | ❌ pending fix-N. Proposed fill: `#f0e9cb` (~5–7% darker than parchment `#faf6e8`; reads as "weathered" without being a dark stain). Drawn as a pie-slice from dome center to rim, BEHIND the strata-band rings + stars. Encodes "less alive" categorically — not decoration; honors Concept Paper §5.6 because it carries data semantic. |

---

---

## §9 · Mode A: Acts

### §9.1 Core function

Wedges by **which Act produced the note**. Six uniform wedges at 60° each, in canonical Five-Acts order from top: **Observation → Connection → Tension → Synthesis → Conviction → Unacted**, clockwise. Each star is placed in the wedge of its `note_meta.actsPrimary` value (extracted from `properties_json.act` at backfill). Notes with no act tag fall to the `Unacted` wedge AND render hollow with the gold frame per §2.

### §9.2 Role within the Epistemic Contents Concept

The Five Acts are Constellation's **methodological vocabulary** for *how* knowledge is created — the cognitive operations themselves. Per CLAUDE.md and the canonical knowledge-formulation philosophy:

> *The Five Acts of Knowledge Creation: Observation → Connection → Tension → Synthesis → Conviction.*

This isn't a workflow; it's the cognitive **arc every piece of knowledge traverses**. Each Act maps to a specific operation:

| Act | Operation | Civilizational anchor |
|---|---|---|
| **Observation** | Noticing, capturing raw input | *al-mushāhadah*; Aristotelian *aisthēsis*; Polanyi's subsidiary awareness |
| **Connection** | Relating one thing to another | *al-rabṭ*; Hume's association; the linking move |
| **Tension** | Identifying contradiction / dialectic | *al-tanāquḍ* / *al-ikhtilāf*; Hegelian antithesis; Socratic *aporia* |
| **Synthesis** | Integrating into higher-order construct | *al-tarkīb*; Hegelian aufhebung; Aristotelian universal-from-particulars |
| **Conviction** | Settling into stable belief | *al-iʿtiqād al-mustaqirr*; Polanyi's focal awareness; Wittgensteinian "bedrock" |

Cross-civilizational arc parallels:

| Tradition | Equivalent arc |
|---|---|
| **Aristotelian inductive arc** | sensation → memory → experience → universal → first principles |
| **Sunni *uṣūl al-fiqh*** | *istiqrāʾ* (induction) → *istidlāl* (inference) → *ijtihād* (independent reasoning) → *taqrīr* (formulation) |
| **Western analytic** | data → patterns → anomalies → theory → settled belief |
| **Hegelian dialectic** | thesis → antithesis → synthesis (Tension and Synthesis Acts directly named) |
| **Polanyi's *Personal Knowledge*** | from tacit subsidiary awareness through articulation to focal explicit conviction |

Mode A asks: **"Which cognitive ACT produced each note in my universe?"** — the *methodological signature* of your thinking.

**Distinct from the other modes**:
- Stage (S) tells WHERE in the developmental arc the note lives (Spark → Maturity)
- Time (T) tells WHEN the note was born
- Confidence (C) tells HOW certain the claims are
- **Acts (A) tells WHICH cognitive operation produced it**

A note can be at Maturity stage (developed), Established confidence (settled), but produced via an Observation Act (a thoroughly-recorded fact you observed). Acts is orthogonal to Stage and Confidence and Time.

### §9.3 How it helps users understand their cognitive knowledge state

The diagnostic questions Mode A answers:

- *What cognitive Acts dominate my universe?* (largest wedge)
- *Am I an OBSERVER, a CONNECTOR, a DIALECTICAL THINKER, a SYNTHESIZER, or a CONVICTION-MAKER?*
- *Am I AVOIDING Tension?* (small Tension wedge = no dialectic engagement; intellectual comfort zone)
- *Where in the strata do my Synthesis acts cluster?* (Synthesis at L5+ = healthy; Synthesis at L1-L2 = misclassified or premature)
- *Are my Convictions backed by Synthesis?* (Conviction-heavy without proportional Synthesis = leap-of-faith convictions)
- *Are my Observations turning into Connections?* (lots of Observation, few Connection = hoarding without weaving)

The cognitive-signature reads Mode A surfaces:

| Pattern | What it means |
|---|---|
| **Observation-heavy** | Clipping / research / capture phase; collecting raw material |
| **Connection-heavy** | Active linker; building the network; weaving |
| **Tension-heavy** | Dialectical thinker; engaging contradictions; productive disputation |
| **Synthesis-heavy** | Integrator; building higher-order structures; *al-mufakkir al-tarkībī* |
| **Conviction-heavy** | Settled-belief asserter; potentially past the productive inquiry phase |
| **Unacted-heavy** | Methodologically untracked; user not engaging Constellation's cognitive vocabulary — most common state for new users; shrinks as they tag Acts |

**Most diagnostic SINGLE READ — the FLOW pattern**: do you have a healthy *Observation → Connection → Tension → Synthesis → Conviction* distribution? Or are you stuck at one Act?

The most common cognitive deficit Mode A surfaces is the **bottleneck at Tension** — users with heavy Observation + Connection but a near-empty Tension wedge are *avoiding contradictions*. They're noticing things and linking them but never engaging the dialectical move that produces deeper synthesis. Mode A makes that avoidance visible at a glance.

The second most common: **Conviction without Synthesis** — heavy Conviction with low Synthesis means belief without integration; possibly leap-of-faith asserting.

### §9.4 Uniqueness vs. other PKM systems

| Tool | What it does for cognitive operations | What it does NOT do |
|---|---|---|
| **Obsidian** | No methodological taxonomy | Cannot represent cognitive operations |
| **Logseq** | No methodological taxonomy | Same gap |
| **Roam** | No methodological taxonomy | Same gap |
| **Notion** | Status fields can simulate | Not first-class; not graph-aware |
| **Tinderbox** | Agents can mark notes | Not cognitive-act-specific; manual setup |
| **PARA method** (Tiago Forte) | Projects / Areas / Resources / Archives | Workflow categories, not cognitive operations |
| **Zettelkasten** | Literature / fleeting / permanent notes | 3 categories; not graph-aware; not methodological |
| **Research methodology taxonomies** (qualitative/quantitative; observational/experimental) | Research-type categorization | For researchers, not personal knowledge; no PKM integration |
| **Constellation Mode A** | Cognitive-act × strata distribution at universe scale | First PKM to make the *methodology of knowledge creation* a navigable axis |

The closest cross-domain analogue is **research methodology classification** — but those are research-type taxonomies (qualitative vs experimental), not cognitive-operation taxonomies, and they don't operate over personal knowledge graphs. Constellation Mode A is the **first PKM to surface the user's methodological signature** at universe scale.

### §9.5 Importance among Constellation core functions

- The Five Acts are **foundational to Constellation's Knowledge Formulation philosophy** (per `docs/CONSTELLATION-KNOWLEDGE-FORMULATION.md` and CLAUDE.md). They aren't a feature — they're a load-bearing concept.
- Concept Paper v3.1 §6 notes Acts as "*per-note act tag (CE Layer 2 — partial)*" — meaning the taxonomy ships but data population on real universes is incomplete. Most users (including Eisa) will see a heavy Unacted wedge initially, shrinking as they tag.
- **Mode A is the ONLY surface that aggregates Acts at universe scale.** Single-note Act is editable in PropertyEditor; no other view shows the universe-wide distribution.
- **Mode A × every other mode composition** is uniquely powerful (Layer 2 territory):
  - **× Mode T**: *maturation arc* — older months heavy in Observation, newer in Synthesis/Conviction
  - **× Mode S**: *lifecycle alignment* — Observation should tend Spark; Synthesis should tend Growth+
  - **× Mode C**: *confidence trajectory* — Observation Acts often hypothesis; Conviction Acts often established
  - **× Mode L**: *link-type pattern* — Connection Acts use `supports`/`derives-from`; Tension Acts use `contradicts`; Synthesis Acts use `generalizes`/`exemplifies`
  - **× Mode P**: *source pattern* — Observation Acts come from perception/testimony; Synthesis Acts from inference/postulation

Mode A is the natural **sixth click** after R (where) → L (what kind) → T (when) → C (how certain) → S (how developed) → **A (which cognitive operation)**.

### §9.6 Visual encoding within Mode A

| Variable | What it means in A |
|---|---|
| Radial position | Strata (invariant — see §1.3) |
| **Angular position** (mode-specific) | **Dominant Act.** Six uniform 60° wedges in canonical Five-Acts order from top (12 o'clock), clockwise: `Observation → Connection → Tension → Synthesis → Conviction → Unacted`. Order matches the natural cognitive arc with Unacted at the end (parallel to Mode L's `untyped` placement). |
| Size | Maturity (invariant) |
| Brightness / alpha | Confidence (invariant) |
| Color | State (invariant — ink default, red contested) |
| **Fill / frame** | Solid by default. Hollow with gold frame for stars in the `Unacted` wedge (notes with no Act tag). Other gap types per §2 cascade. |
| **Wedge background** | All wedges standard parchment (`#faf6e8`) — **no muted treatment for Unacted**. The Unacted wedge encodes *missing data*, which the hollow stars already signal at the star level. Muting the wedge would be double-encoding the same signal. (Distinct from Mode S's Dormancy + Archival muting, which encodes *less-alive lifecycle states* — full semantic states, not absence-of-data.) |
| **Grouping** (emergent) | Heavy at one Act = your dominant cognitive operation. Stratum-cluster within an Act wedge: "Synthesis at L5+ (healthy)" or "Synthesis scattered everywhere (over-claiming)." Empty Tension wedge = you avoid contradiction (alarm signal). |

### §9.7 Calculation algorithm — Mode A specifics

```
bucketKeyFor(note, A):
    return note.actsPrimary ?? 'Unacted'

hasModeDataFor(note, A):
    return note.actsPrimary != null
    # Solid only if user has explicitly tagged the note with an Act.

wedgeBuckets(notes, A):
    order = ['Observation', 'Connection', 'Tension', 'Synthesis', 'Conviction', 'Unacted']
    span = 2π / 6                                              # uniform 60° each
    cursor = -π/2                                              # start at top
    wedges = []
    for key in order:
        wedges.push({
            key:            key,
            label:          $t(`acts.${key.toLowerCase()}`) || key,   # locale-aware
            count:          # count of visible notes whose actsPrimary == key
            azimuthStart:   cursor,
            azimuthEnd:     cursor + span
        })
        cursor += span
    return wedges
```

**Backfill**: `note.actsPrimary` is computed at §2 backfill via `json_extract(nm.properties_json, '$.act')`. **Verify the frontmatter field name** is `act` (singular) — if PropertyEditor writes `acts` (plural) or `action` or another variant, the backfill won't pick it up. (See §9.8 polish checklist.)

**Why uniform spans + canonical order**: Five Acts are a fixed-cardinality vocabulary. Spatial memory: "Tension is at 4 o'clock; Synthesis at 6." Same reasoning as Modes L / T / C / S.

**Why Unacted at the END (not at top)**: parallel to Mode L's `untyped` placement — the missing-data wedge sits at the natural "off the arc" position so the active-vocabulary wedges retain their cognitive ordering.

### §9.8 Implementation gaps / fine-tune checklist

| Feature | Status | Note |
|---|---|---|
| 6 wedges in canonical Five-Acts order | ✅ shipped (geometry) | |
| Stars positioned by `note_meta.actsPrimary` | ✅ shipped | Uses `json_extract(properties_json, '$.act')` per §2 backfill |
| **Verify frontmatter field name consistency** (`act` vs `acts` vs `action`) | ⚠️ **flag for verification** | Backfill uses `'$.act'`. PropertyEditor write convention needs cross-check; if mismatch, all notes will appear Unacted regardless of user input |
| Hollow rendering for Unacted notes (gold frame) | ❌ pending §N |
| `acts.*` i18n keys exist + populated across 15 locales | ⚠️ **likely gap** — same suspicion as Mode S's `stage.*` keys; verify at §N close-out |
| **Acts data is sparse on real universes** (per Concept Paper §6: "CE Layer 2 — partial") | ⚠️ **content gap, not code gap** | Most notes won't have `act` set initially — Unacted wedge will dominate. Solution: (a) encourage tagging via PropertyEditor docs; (b) add an Acts classifier in CECE Layer 2 (MIG-025) |
| **Acts classifier in CECE** (Layer 2): CECE could propose Acts for Unacted notes based on note content + link patterns + author Act tags | ❌ Layer 2 — pending MIG-025 |
| Wedge-hover tooltip per-stratum breakdown | ❌ pending §N |
| Wedge-click → filter | ❌ pending §N |
| **Acts × Time composition** (Layer 2): maturation arc — "*your old months are mostly Observation; recent months are mostly Synthesis — healthy maturation*" | ❌ Layer 2 — pending MIG-025 |
| **Acts × Stage composition** (Layer 2): lifecycle health — "*your Observation acts cluster at Spark (correct); your Synthesis acts cluster at Growth+ (correct)*" | ❌ Layer 2 — pending MIG-025 |
| **Acts × Confidence composition** (Layer 2): conviction integrity — "*your Conviction acts have established confidence (good); your Conviction acts have hypothesis confidence (leap-of-faith asserting)*" | ❌ Layer 2 — pending MIG-025 |
| **Tension-avoidance alert** (Layer 2 / 3): "*Your universe has < 5% Tension acts — you may be avoiding dialectical engagement*" | ❌ Layer 2 / 3 — pending MIG-025 / 026 |
| **Observation-Connection-bottleneck alert** (Layer 2): "*High Observation, low Connection — you collect but don't link*" | ❌ Layer 2 — pending MIG-025 |
| **Cognitive flow coaching** (Layer 4): "*Want help moving these Observation notes to Connection by drafting some links?*" | ❌ Layer 4 — pending MIG-027 |
| **Conviction-without-Synthesis coaching** (Layer 4): "*Your Convictions outpace your Synthesis — would you like to revisit the underlying Synthesis chain?*" | ❌ Layer 4 — pending MIG-027 |

---

---

## §10 · Mode P: Provenance

### §10.1 Core function

Wedges by **primary horizontal-axis source family** from CECE's live taxonomy. Twelve uniform wedges at 30° each, in canonical Universal Epistemic Content Taxonomy order from top: **perception → inference → testimony → mass-transmission → comparison → postulation → non-apprehension → memory → innate-disposition → inspiration → revelation → unsourced**, clockwise. Each star is placed in the wedge of its `note_meta.sourcesPrimary` family (CECE writes leaf IDs like `testimony/authoritative`; Mode P aggregates to the top-level family). Notes with no source classification fall to the `unsourced` wedge AND render hollow with the gold frame per §2.

When >70% of the visible universe is unsourced (typical for fresh installs before classification), the dome center renders the **empty-state CTA card** ("*Classify via Source Review →*") shipped in fix-3.

### §10.2 Role within the Epistemic Contents Concept

Mode P is **the mode that introduces the new dimension Sight v5 brings to Constellation**. Per Concept Paper v3.1 §4.3:

> *"The horizontal axis (the 11 sources / masādir al-maʿrifah / pramāṇa) is not yet tracked in Constellation today — except where the MIG-021v3 CECE cascade has already started populating it. Sight v5 lifts it into a per-note frontmatter field and as the seventh Sight mode (P — Provenance)."*

The 11 source families are the **horizontal axis** of the Universal Epistemic Content Taxonomy — the *masādir al-maʿrifah* / *pramāṇa* convergence distilled from cross-civilizational epistemology. Each source represents a distinct **EPISTEMIC ORIGIN**: how the content was *acquired or produced* cognitively.

Civilizational provenance of each source family:

| Family | Sunni anchor | Indian anchor | Other anchors | Recognition |
|---|---|---|---|---|
| **perception** | الحِسّ | *pratyakṣa* | Greek *aisthēsis* · Mohist *qīn zhī* | Universal |
| **inference** | العَقل | *anumāna* | Greek logic | Universal |
| **testimony** | الخَبَر | *śabda* | Mohist *shuō zhī* | Sunni, Indian (Nyāya/Mīmāṃsā), Mohist, Confucian; rejected as primary by Cārvāka, Dignāga's Buddhists, Greek Pyrrhonists |
| **mass-transmission** | التَّواتُر | (no equivalent) | — | **Distinctively Sunni** *uṣūl al-fiqh* — convergent independent reports too numerous to collude on falsehood |
| **comparison** | القياس | *upamāna* | — | Nyāya, Mīmāṃsā, Vedānta, Sunni jurisprudence (*qiyās*); subsumed under inference by Buddhists |
| **postulation** | الاستنباط الافتراضي | *arthāpatti* | Modern philosophy of science (IBE) | Mīmāṃsā, Vedānta, modern PoS |
| **non-apprehension** | عَدَم الإدراك | *anupalabdhi* | — | *Bhāṭṭa Mīmāṃsā*, *Advaita Vedānta* |
| **memory** | الذاكرة | *smṛti* | All major traditions | Universal |
| **innate-disposition** | الفِطرة | (no single equivalent) | Greek *nous* · Mencian *liángzhī* | Sunni, Greek, Confucian, Mencian |
| **inspiration** | الإلهام / الكَشْف | (varies) | Sufi-influenced; certain Buddhist | Some Sunni Sufi authors, Daoist; rejected by Mohists + most analytic philosophers |
| **revelation** | الوحي | (cf. *āgama*) | Christian, Jewish | Sunni Islam, Judaism, Mīmāṃsā (Vedas), Christianity; rejected by Cārvāka + secular analytic |

Mode P asks: **"What KIND of knowing produced each note in my universe?"** — the *epistemic-origin signature*. This is the **most civilizationally-conscious** mode in Sight v5; it's where the Universal Epistemic Content Taxonomy's pluralism becomes most directly user-facing.

**Distinct from every other mode**: R/L/T/C/S/A all describe the cognitive *work on* content. P describes the **epistemic origin of the content itself**. You can have an Observation Act (the cognitive operation) producing a note from inspiration source (the epistemic origin) — orthogonal axes.

### §10.3 How it helps users understand their cognitive knowledge state

The diagnostic questions Mode P answers — most epistemologically rich of all 7 modes:

- *What epistemic sources dominate my universe?* (largest wedge)
- *Am I a TESTIMONY-heavy thinker (mostly quoted authorities) or an INFERENCE-heavy thinker (mostly your own reasoning)?*
- *Where in the strata do my different sources cluster?*
  - testimony at **L1-L2** = "I quote authorities for raw facts" — good citation hygiene
  - testimony at **L7-L8** = "my worldview is built on quoted authority" — fine for tradition-grounded thinkers; derivative for creative thinkers
  - inference at **L7-L8** = "I derive my worldview through my own reasoning" — sign of independent thinker
  - revelation at any stratum = "religiously-anchored content" — categorically distinct epistemic class
- *Are there sources I never use?*
  - Never `non-apprehension` → you don't reason from absence; missing a methodological tool
  - Never `mass-transmission` → you don't engage Sunni *uṣūl* even if Muslim; methodology gap
  - Never `postulation` → you don't do Inference to Best Explanation; methodologically restricted
- *Is my source distribution methodologically diverse, or a monoculture?*

The cognitive-signature reads Mode P surfaces — **the most epistemologically rich of all the modes**:

| Pattern | Reads as |
|---|---|
| **Perception-heavy** | Empiricist; observational; experiential thinker |
| **Inference-heavy** | Rationalist; deductive; analytical thinker |
| **Testimony-heavy** | Traditional; authority-grounded thinker |
| **Mass-transmission-heavy** | Sunni-classical; *uṣūlī*-trained thinker |
| **Comparison-heavy** | Analogical; case-based thinker |
| **Postulation-heavy** | Abductive; theoretical; explanation-seeking thinker |
| **Non-apprehension-heavy** | Absence-reasoning; *via negativa* thinker (rare) |
| **Memory-heavy** | Retrospective; archive-grounded thinker |
| **Innate-disposition-heavy** | First-principles; intuition-trusting thinker |
| **Inspiration-heavy** | Mystical; Sufi-influenced thinker |
| **Revelation-heavy** | Scripturally-grounded thinker |
| **Unsourced-heavy** | Epistemic provenance untracked — the initial state for any new universe |

Most diagnostic SINGLE READ: **the methodology pluralism**. A balanced mind shows non-trivial presence across MANY source families. A monoculture shows 80%+ in one. The cross-civilizationally-pluralistic taxonomy makes this LEGIBLE in a way no other framework can — your distribution can be compared against multiple intellectual traditions, not just one.

### §10.4 Uniqueness vs. other PKM systems

| Tool | What it does for source provenance | What it does NOT do |
|---|---|---|
| **Obsidian** | No source/provenance concept | Cannot represent epistemic origin |
| **Logseq** | No source concept | Same gap |
| **Roam** | No source concept | Same gap |
| **Notion** | Database relations can simulate | Not first-class; not graph-aware; not civilizational |
| **Zotero / Citavi** | Academic source citations | Only for academic papers; not personal knowledge; not classification by epistemic origin type |
| **Hadith management tools** (specialized Islamic scholarship) | Have *isnād* chains | Closest Islamic-tradition analogue; no aggregate visualization; no cross-civilizational integration |
| **Bayesian belief networks** | Have source priors | For probabilistic models, not personal knowledge |
| **Constellation Mode P** | 11-source × strata distribution at universe scale, with cross-civilizational epistemological grounding | **First PKM to surface epistemic provenance as a navigable cognitive axis with cross-civilizational methodology** |

The closest analogues are **academic systematic-review source-classification** (peer-reviewed / gray literature / expert opinion) and **Islamic *isnād* chain analysis** — but neither operates over personal knowledge graphs, neither aggregates, and neither integrates cross-civilizationally. **Mode P is genuinely unprecedented.**

### §10.5 Importance among Constellation core functions

Mode P is the **single most architecturally-loaded mode in Sight v5**. It's the mode that justifies — at the visualization layer — the entire MIG-021/021v2/021v3 CECE shipment (~3 weeks of coordinated work), the Universal Epistemic Content Taxonomy doc, the bilingual EN/AR materials, the multi-civilizational research Eisa commissioned. **Without Mode P, all that infrastructure has no aggregate visualization surface.**

- CECE's 6-cataloger ensemble produces the source classifications that feed Mode P
- Source Review panel surfaces classifications one note at a time; **Mode P shows the universe-wide result**
- The 11-source taxonomy is fully populated across CECE's live `horizontal_taxonomy.rs` (~53 nodes including leaf IDs) + bilingual labels in 15 locales
- The empty-state CTA (fix-3 / D-V6.α) loops back to Source Review — every dot you classify shrinks the Unsourced wedge

**Mode P × every other mode composition is the deepest analytical layer Sight v5 offers** (Layer 2 territory):

- **× Mode A**: source × cognitive-operation matrix (Observation→perception/testimony; Synthesis→inference/postulation; etc.)
- **× Mode T**: source trajectory ("*your recent months are heavy in testimony — you've been reading; your older months are heavy in inference — you were synthesizing your own thinking*")
- **× Mode C**: source-confidence relationship (testimony often hypothesis or evidence; revelation often established or *yaqīn*)
- **× Mode S**: source-stage (testimony tends Spark — citations stay raw; inference matures through Growth → Maturity)
- **× Mode L**: source-link pattern (testimony notes use `derives-from`; inference notes use `supports`/`generalizes`; **`contradicts` from inference engaging with testimony makes the cross-civilizational dialectic visible**)

**Mode P is the seventh and last click — the deepest layer of cognitive-portrait analysis Sight v5 offers.** It's the mode that makes Constellation a *civilizationally-pluralistic* PKM, not just a graph-with-typed-links.

### §10.6 Visual encoding within Mode P

| Variable | What it means in P |
|---|---|
| Radial position | Strata (invariant — see §1.3) |
| **Angular position** (mode-specific) | **Dominant source family.** Twelve uniform 30° wedges in canonical Universal Taxonomy order from top (12 o'clock), clockwise: `perception → inference → testimony → mass-transmission → comparison → postulation → non-apprehension → memory → innate-disposition → inspiration → revelation → unsourced`. Order matches §III of `epistemic-content-taxonomy.md`. |
| Size | Maturity (invariant) |
| Brightness / alpha | Confidence (invariant) |
| Color | State (invariant — ink default, red contested) |
| **Fill / frame** | Solid by default. Hollow with gold frame for stars in the `unsourced` wedge. Other gap types per §2 cascade. |
| **Wedge background** | All 12 wedges standard parchment (`#faf6e8`) — **no muted treatment for any source family**. Civilizational pluralism principle (Concept Paper §3.1 design principle 4): we don't editorialize on which sources are more or less "alive." Visually treating revelation/inspiration/innate-disposition as different from inference/perception would betray Mode P's design intent. Only Unsourced gets the hollow signal at the star level. |
| **Empty-state CTA** | When >70% of visible universe is unsourced, render the centered card: *"Most of your universe is unsourced. Provenance mode reveals the shape of your epistemic content once notes are classified."* + *"Classify via Source Review →"* button. Already shipped (fix-3). |
| **Grouping** (emergent) | Heavy at one source = your dominant epistemic source. Stratum-cluster within a source wedge = "*my testimony notes cluster at L1-L2 (citation pile) vs L7-L8 (worldview-by-quoted-authority)*." Empty wedges = methodology gaps in your epistemic toolkit. |

### §10.7 Calculation algorithm — Mode P specifics

```
SOURCE_FAMILIES = [
    'perception', 'inference', 'testimony', 'mass-transmission',
    'comparison', 'postulation', 'non-apprehension', 'memory',
    'innate-disposition', 'inspiration', 'revelation', 'unsourced'
]

bucketKeyFor(note, P):
    return sourceFamily(note.sourcesPrimary)

sourceFamily(primary):
    if primary == null: return 'unsourced'
    # CECE writes leaf IDs in 'family/leaf' shape (e.g. 'testimony/authoritative').
    # Mode P aggregates to the top-level family.
    slash = primary.indexOf('/')
    return slash > 0 ? primary.slice(0, slash) : primary

hasModeDataFor(note, P):
    return note.sourcesPrimary != null
    # Solid only if CECE (or the user) has classified the source.

wedgeBuckets(notes, P):
    span = 2π / 12                                            # uniform 30° each
    cursor = -π/2                                              # start at top
    wedges = []
    for key in SOURCE_FAMILIES:
        wedges.push({
            key:            key,
            label:          $t(`sources.label.${key}`) || key,   # locale-aware (15 locales shipped MIG-021v3 + MIG-022 §E.3)
            count:          # count of visible notes whose source family == key
            azimuthStart:   cursor,
            azimuthEnd:     cursor + span
        })
        cursor += span
    return wedges
```

**Backfill**: `note.sourcesPrimary` is computed at §2 backfill via `json_extract(nm.sources, '$[0]')` — the first element of the JSON array in `note_meta.sources`. Already shipped correctly in MIG-024 §2.

**Family extraction** (`sourceFamily`): CECE writes leaf IDs like `testimony/authoritative`; Mode P aggregates to the top-level family `testimony`. Slash-prefix split is the family extraction. Drill-down to leaf IDs is a future enhancement (see §10.8).

**i18n**: source labels come from CECE's 15-locale shipped i18n (`sources.label.*` keys, populated in MIG-021v3 V3-§10.D + extended in MIG-022 §E.3). **Already complete across all 15 locales** — Mode P is the i18n-most-complete mode.

**Why uniform spans + canonical Universal Taxonomy order**: same reasoning as the other modes — fixed-cardinality vocabulary, spatial-memory invariant. The taxonomy's §III canonical order is preserved so users build muscle memory ("revelation is at 11 o'clock").

### §10.8 Implementation gaps / fine-tune checklist

| Feature | Status | Note |
|---|---|---|
| 12 wedges in canonical Universal Taxonomy order | ✅ shipped (geometry) | |
| Stars positioned by `note_meta.sourcesPrimary` family | ✅ shipped | Uses `json_extract(nm.sources, '$[0]')` per §2 backfill, then slash-prefix family extraction |
| `sources.label.*` i18n keys populated across 15 locales | ✅ shipped (MIG-021v3 + MIG-022 §E.3) | The i18n-most-complete mode |
| Source Review panel + 6-cataloger ensemble + Sibling Disambiguation | ✅ shipped (MIG-021v3) | Full classification workflow already in production |
| Empty-state CTA when >70% unsourced | ✅ shipped (fix-3) | |
| Hollow rendering for Unsourced notes (gold frame) | ❌ pending §N — depends on hollow-rendering implementation |
| Wedge-hover tooltip per-stratum breakdown | ❌ pending §N |
| Wedge-click → filter visible stars to that source family only | ❌ pending §N |
| **Drill-down to leaf IDs**: clicking `testimony` wedge shows leaf distribution (e.g., authoritative / common / scholarly / journalistic) | ❌ open design — pending §N or Layer 2 |
| **Source × Stratum pluralism diagnostic** (Layer 2): "*your universe shows balance across 8 source families × 6 strata — high methodological pluralism*" | ❌ Layer 2 — pending MIG-025 |
| **Methodology gap alert** (Layer 2): "*you have 0 notes from non-apprehension or postulation — you may be missing absence-reasoning or IBE in your methodological toolkit*" | ❌ Layer 2 — pending MIG-025 |
| **Source-monoculture alert** (Layer 2): "*85% of your notes come from testimony — you primarily quote authorities*" | ❌ Layer 2 — pending MIG-025 |
| **Cross-civilizational signature analysis** (Layer 2): "*your distribution matches the Sunni *uṣūl al-fiqh*-heavy thinker pattern (testimony + mass-transmission + qiyās dominant)*" — **uniquely Constellation; no other PKM offers this** | ❌ Layer 2 — pending MIG-025; high-value diagnostic |
| **Source × Stratum mismatch alert** (Layer 2): "*your testimony notes cluster at L7-L8 — your worldview is largely quoted authority. Healthy for tradition-grounded thinkers; derivative for creative thinkers — your call.*" | ❌ Layer 2 — pending MIG-025 |
| **CECE auto-suggest acceleration**: integrate background classifier scan progress into Mode P so users see the unsourced wedge shrinking in real-time as CECE classifies | ❌ pending §N — UX polish around the Source Review handoff |
| **Source-acquisition coaching** (Layer 4): "*want help adding inference-derived notes to broaden your epistemic base?*" | ❌ Layer 4 — pending MIG-027 |
| **Civilizational-tradition-matching coaching** (Layer 4): "*your signature aligns with the *Ishrāqī* tradition (high innate-disposition + inspiration + perception). Want to explore methodology?*" | ❌ Layer 4 — pending MIG-027; profoundly novel Layer 4 feature |

---

---

## §11 · Cross-mode polish backlog (compiled 2026-05-13)

All open items from §4–§10 organized by **ship-target** — which MIG / phase delivers each piece. This is the actionable backlog for MIG-024 close-out and the four-MIG Layer cascade beyond.

### §11.1 fix-N candidates — ship NOW before MIG-024 §N close-out

These are real bugs or Eisa-confirmed quick fixes that shouldn't wait for the next MIG.

| ID | Item | Source | Effort |
|---|---|---|---|
| **fix-7** | `note_meta.confidenceAlpha` population in §2 backfill — currently hardcoded `NULL`, breaking Mode C | §7.8 (P0 — Mode C non-functional) | ½ day — replace `NULL AS confidence_alpha` with the SQL aggregation in §7.7 |
| **fix-8** | Hollow rendering + 4 frame colors per §2 (touches every mode's missing-data state) | §2 + §4–§10 (every mode flags this) | 1 day — update `drawStars` in render.ts to handle the cascade; change in one place lights up across all 7 modes |
| **fix-9** | Muted wedge background for Mode S Dormancy + Archival (`#f0e9cb`) | §8.6 + §8.8 (Eisa-confirmed) | ½ day — add `bgFill` to wedge struct; render pie-slice behind strata bands |
| **fix-10** | Verify frontmatter field name consistency for Mode A (`act` vs `acts` vs `action`) — if PropertyEditor writes a different key than backfill reads, ALL notes appear Unacted | §9.8 (verification + potential one-line fix) | ½ day verification + ½ day fix if mismatch |

### §11.2 §N inline polish — small UI items, ship before MIG-024 closes

UI items that don't introduce new computational layers but tighten the visible surface. Bundle these into the §N close-out commit.

**Cross-mode — apply to every mode**:
- Wedge-hover tooltip: "{wedge_label} — N notes — strata breakdown L1: x, L2: y, …"
- Wedge-click → auto-flip scope to that wedge's filter (R-mode: scope to that library; T-mode: scope to that month; etc.)
- Locale label verification: confirm `linkTypes.*` (Mode L), `stage.*` (Mode S), `acts.*` (Mode A) i18n keys are populated across 15 locales — `sources.label.*` (Mode P) already verified shipped. The Mode R library names use raw user-set strings (no i18n needed); Mode T month names use `Intl.DateTimeFormat` (auto-locale); Mode C uses `sight.v5.field.*` (verified shipped fix-3).

**Per mode**:
- **Mode R**: optional alphabetical wedge sort toggle in Settings (currently largest-first only)
- **Mode T**: today-marker beyond month tint (small ring or arrow at today's exact date on the rim)
- **Mode L**: when user adds a `supersedes` typed link, the supersedes wedge highlights briefly
- **Mode P**: drill-down — clicking a source-family wedge shows the leaf-ID distribution within (testimony → authoritative / common / scholarly / journalistic). Could also be Layer 2.
- **Mode P**: integrate background classifier scan progress into the empty-state — users see the unsourced wedge shrinking in real-time as CECE classifies
- **Mode S**: archival workflow integration — clicking an Archival-wedge note shows "view archival history / restore to Growth" action

### §11.3 Layer 2 diagnostic findings — MIG-025

Layer 2 ships the **diagnostic engine** that reads aggregations across the layout cache + the per-mode wedge counts and surfaces plain-language findings in the Findings side panel. Per Concept Paper v3.1, this is where Sight v5 stops being a pure visualizer and becomes a diagnostic instrument.

**Cross-cutting infrastructure**:
- **Ambiguity-surfacing pattern** — every mode where Layer 1 placement requires arbitrary tie-breaking → Layer 2 surfaces via the Findings card → user resolves via side-panel chips mirroring CECE Sibling Disambiguation. First instance from Mode L (§5.7); reapplied to Mode C confidence ties (§7.8). Standardize the resolution surface so it's reusable across modes.

**Per mode**:

| Mode | Layer 2 diagnostic findings |
|---|---|
| **T** | Year-disambiguation within wedge (year-stacking via jitter); year-range filter ("last 12 months / 3 years / all time"); trajectory overlay reading `note_state_history` (the high-value MIG-022 §B Rust foundation consumer); Five-Acts × Time composition; cognitive-cooling alert |
| **C** | **Stratum-confidence mismatch alert (high-end)**: "*N L7-L8 notes at hypothesis confidence — intellectual overreach risk*" (highest-value Mode C diagnostic); stratum-confidence mismatch alert (low-end): "*N L1-L2 notes contested — wasted dispute energy*"; confidence trajectory from `note_state_history` ("*your established notes were hypothesis 6 months ago*"); confidence-tie surfacing (per ambiguity pattern); Bayesian credence integration |
| **S** | Spark-to-Maturity ratio diagnostic; Dormancy-at-high-strata alert; Stage × Time composition; Stage × Acts composition; Stage × Maturity-size mismatch (the §8.6.1 two-axis dual diagnostic) |
| **A** | **Acts classifier in CECE** (the data-population enabler — Mode A is currently sparse on real universes); Acts × Time composition (maturation arc); Acts × Stage composition (lifecycle health); Acts × Confidence composition (conviction integrity); Tension-avoidance alert; Observation-Connection-bottleneck alert |
| **P** | Source × Stratum pluralism diagnostic; methodology gap alert ("*0 notes from non-apprehension or postulation*"); source-monoculture alert ("*85% testimony*"); **cross-civilizational signature analysis** ("*your distribution matches the Sunni *uṣūl al-fiqh*-heavy thinker pattern*") — uniquely Constellation; no other PKM offers this; Source × Stratum mismatch alert |

**The 5 highest-value Layer 2 findings** (across all modes — these are the ones users will most often act on):
1. **Mode C** stratum-confidence mismatch (intellectual overreach)
2. **Mode S** Dormancy-at-high-strata (worldview stagnation)
3. **Mode A** Tension-avoidance (intellectual comfort zone)
4. **Mode P** cross-civilizational signature (uniquely Constellation)
5. **Mode T** trajectory ("*your synthesis is consolidating / your output is cooling*")

### §11.4 Layer 3 recommendation engine — MIG-026

Layer 3 wires Qwen3-1.7B + GBNF grammar (the V3-§7.b llama.cpp work) to convert Layer 2 findings into specific named recommendations. Each Layer 2 finding becomes a "Recommend" button → grammar-constrained LLM output naming specific notes/actions.

**Pattern** (cross-mode):
- Click "Recommend" on any Layer 2 finding → Qwen3 generates structured recommendation: `{ findings: [...], actions: [{ note_path, action_kind, rationale }] }`
- GBNF grammar guarantees valid JSON output (no parse failures)
- Recommendations cached per (universe-snapshot-hash + finding-id)

**Examples per mode** (from the Layer 2 findings above):
- **Mode C** "47 stalled hypotheses; here's the 3 most actionable to promote with evidence"
- **Mode S** "5 worldview-tier notes have been Dormant for >6 months — here's the 3 most worth re-examining given recent context"
- **Mode A** "Your Tension wedge is empty; here's 3 contradiction-pairs in your library worth engaging"
- **Mode P** "You have 1,200 testimony notes but only 40 inference notes — here's 5 testimony notes worth deriving inference from"
- **Mode T** "Your output cooled in February; here's 3 March-March-time notes worth reactivating"

### §11.5 Layer 4 coaching — MIG-027

Layer 4 is the conversational LLM mode — the user picks a Layer 3 recommendation and Qwen3 walks them through executing it via Constellation-aware actions (open note, create link, propose stratum promotion, etc.).

**Per mode**:

| Mode | Layer 4 coaching |
|---|---|
| **T** | Cognitive-cooling coaching ("*want to revive these dormant tracks together?*") |
| **C** | Cognitive-overreach coaching ("*your L7 worldview notes are at hypothesis confidence — want help locking them down with evidence chains?*") |
| **S** | Stage-progression coaching ("*N notes at Birth haven't progressed in months — would you like to develop one?*"); Dormancy resurrection coaching |
| **A** | Cognitive flow coaching ("*Want help moving these Observation notes to Connection by drafting some links?*"); Conviction-without-Synthesis coaching ("*Your Convictions outpace your Synthesis — would you like to revisit the underlying Synthesis chain?*") |
| **P** | Source-acquisition coaching ("*want help adding inference-derived notes to broaden your epistemic base?*"); **civilizational-tradition-matching coaching** ("*your signature aligns with the Ishrāqī tradition (high innate-disposition + inspiration + perception). Want to explore the methodology?*") — **profoundly novel Layer 4 feature; uniquely Constellation** |

### §11.6 Future MIGs (post-MIG-027)

Items that don't fit the four-Layer framework or push beyond Sight v5 entirely:

- **Multiple calendar systems on Mode T rim** (Hijri / Solar Hijri / Hebrew per Concept Paper v3.1 §7.2) — needs broader calendar-systems infrastructure across Constellation, not just Sight
- **Mode-switch 600 ms ease animation per-star angular interpolation** (currently snaps; per Plan §5 noted as fix-N if needed) — polish item; may be deferred indefinitely if snap is acceptable on Eisa-test
- **Brighten ONLY the active-wedge link's connector lines** in Mode L — REJECTED per Eisa 2026-05-13; not in any future ship
- **Mode L `untyped` wedge CTA** — DEFERRED per Eisa 2026-05-13; pending decision

### §11.7 Polish-vs-Layer breakdown summary

| Bucket | Count | Where |
|---|---|---|
| **fix-N candidates** (ship NOW, before MIG-024 §N closes) | 4 (fix-7 / 8 / 9 / 10) | §11.1 |
| **§N inline polish** (small UI; bundle into §N close-out commit) | ~20 items | §11.2 |
| **Layer 2 diagnostic findings** (MIG-025) | ~25 distinct findings across 7 modes + ambiguity-surfacing pattern | §11.3 |
| **Layer 3 recommendations** (MIG-026) | ~5+ per Layer 2 finding (auto-derived) | §11.4 |
| **Layer 4 coaching surfaces** (MIG-027) | ~10 distinct coaching flows | §11.5 |
| **Future MIGs** (post-027) | 1 architectural (calendar systems) + 2 deferred-or-rejected | §11.6 |

---

## §12 · Document close-out + Concept Paper v3.2 fold-in

**This document is COMPLETE as of 2026-05-13** — all 7 modes locked across 8 dimensions each; all open items compiled into §11 with ship-targets.

**At MIG-024 §N close-out**: this document folds into `Constellation-Sight-Concept-Paper-v3.2.md` as the canonical §6 (modes) expansion. The fold-in:

1. Sections §0 (what this doc is) and §12 (this section) are absorbed into the Concept Paper preamble
2. Sections §1–§3 (global invariants, hollow semantic, calculation algorithm) become Concept Paper §5.x additions (visual encoding) and §6.x additions (calculation)
3. Sections §4–§10 (the 7 modes) replace Concept Paper §6's per-mode table with full 8-dimension articulations
4. Section §11 (cross-mode polish backlog) becomes the §N close-out commit's polish manifest + the seed for MIG-025/026/027 Architect docs

After fold-in, this standalone document moves to `docs/historical/sight-v5-mode-concepts-v0.1.md` per SO #6. Concept Paper v3.2 carries the canonical content forward.

---

**End of v0.1 — fully drafted; awaiting MIG-024 §N close-out for Concept Paper v3.2 fold-in.**
