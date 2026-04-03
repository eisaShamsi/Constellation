# Cognitive Engine

The Cognitive Engine transforms Constellation from a note-taking app into a knowledge cognition instrument. It doesn't manage your notes — it helps you *know*.

> "The quantity of your data and information doesn't matter. It is NOT about how many references/sources you keep or store; it is about how you formulate your KNOWLEDGE from them, and how to link all of it into one meaningful awareness."

---

## Philosophy

The Cognitive Engine rests on seven epistemological foundations:

1. **Knowledge is not information.** Storing facts is easy. Understanding what they mean, how they relate, and why they matter — that is knowledge. The CE treats notes as knowledge artifacts, not data containers.

2. **Value lives in connections.** A note in isolation is a dead letter. The moment you link it to another note — and specify *why* — it becomes part of a living knowledge network.

3. **Knowledge has a vertical dimension.** Not all notes are equal. A quick fact sits at a different intellectual altitude than a synthesized theory. The CE measures this through an 8-level hierarchy (Knowledge Strata), so you can see at a glance where your thinking is deep and where it is shallow.

4. **Knowledge grows and decays.** Notes are not static objects. They mature from raw captures into authoritative references — and they can wilt from neglect. The CE tracks this lifecycle automatically.

5. **Healthy knowledge is coherent.** It is not enough to have many connected notes. Your knowledge network should be free of unacknowledged contradictions, orphaned insights, and fragile single-source foundations. The CE surfaces these tensions.

6. **Provenance matters.** Knowing *where* your knowledge comes from is as important as the knowledge itself. Is this your own synthesis? Or is it fourth-hand from a source you never verified? The CE traces every chain of derivation.

7. **The framework is invisible.** You should never feel like you are "using" the Cognitive Engine. It works silently in the background, reading the structure of your notes and surfacing insights through subtle visual cues. You absorb its benefits naturally, through the act of writing and linking.

---

## Feature 1: Typed Links

### What it is

In most note-taking apps, a link is a link. It says "these two notes are connected" — nothing more. Typed Links add *semantic meaning* to every connection. Instead of just linking Note A to Note B, you can specify *how* they relate: does A support B? Contradict it? Cause it? Generalize it?

The syntax is simple. Inside a standard wikilink, add a pipe character followed by the relationship type:

```
[[Note Name|supports]]
```

### Why it matters

Without typed links, all connections look identical. Your knowledge graph becomes a web of undifferentiated lines — you can see that things are related, but not *how*. With typed links, you can distinguish "this evidence supports that claim" from "this evidence contradicts that claim." This distinction is the foundation of structured thinking.

When you look at your Star View graph, typed links become colored edges. Red lines show contradictions. Blue lines show support. Gold lines trace provenance. At a glance, you can read the intellectual topology of your knowledge.

### How to use it

**Step 1: Create a regular wikilink**

Type `[[` followed by a note name and `]]`:

```
See [[Climate Change]] for more details.
```

**Step 2: Add a type**

Place a pipe `|` after the note name, then type the relationship:

```
This finding [[Climate Change|supports]] the warming hypothesis.
```

**Step 3: Use autocomplete**

When you type `[[note name|`, a dropdown appears showing all available link types. Select one to insert it.

### The 7 link types

| Type | Color | Meaning | Example |
|------|-------|---------|---------|
| `supports` | Blue (#4A9EFF) | Evidence for a claim | "This study [[Global Warming\|supports]] the CO2 hypothesis" |
| `contradicts` | Red (#FF4A4A) | Tension or opposition | "However, this data [[Previous Study\|contradicts]] the timeline" |
| `causes` | Orange (#FF8C42) | Causal relationship | "Deforestation [[Soil Erosion\|causes]] downstream flooding" |
| `exemplifies` | Green (#4AFF88) | Instance of a concept | "The 2008 crisis [[Financial Bubbles\|exemplifies]] herd behavior" |
| `generalizes` | Purple (#A44AFF) | Abstraction | "This pattern [[Complex Systems\|generalizes]] across domains" |
| `derives-from` | Gold (#FFD700) | Source lineage | "My analysis [[Original Paper\|derives-from]] the 2019 dataset" |
| `part-of` | Gray (#AAAAAA) | Compositional hierarchy | "Chapter 3 [[Book Project\|part-of]] the main argument" |

### Where you see it

- **Editor**: When your cursor moves away from a typed link, it renders in the type's color. A `supports` link appears blue; a `contradicts` link appears red.
- **Backlinks Panel**: Each backlink that uses a typed link shows a small colored badge with the link type name (e.g., a blue "supports" badge).
- **Star View**: Link edges between nodes are colored by type. Red edges for contradictions, gold for derivation chains, and so on.

### Tips

- **You don't have to type every link.** Plain `[[link]]` works perfectly — it defaults to an "associative" connection. Only add types when the relationship is meaningful to you.
- **Start with two types.** If you are new to typed links, begin with just `supports` and `contradicts`. These two alone transform how you see your knowledge.
- **Unknown types are safe.** If you type `[[note|foobar]]`, Constellation treats it as an associative link with "foobar" as the display text. Nothing breaks.
- **Autocomplete helps.** You do not need to memorize all 7 types. Type `[[note|` and the autocomplete dropdown appears with all options and their descriptions.

---

## Feature 2: Knowledge Strata

### What it is

Knowledge Strata classifies every note in your library into one of 8 intellectual levels, from raw facts at the bottom to worldview-shaping paradigms at the top:

| Level | Name | Description |
|-------|------|-------------|
| 1 | **Datum** | A raw fact. 50 words or fewer, no links. A phone number, a date, a single observation. |
| 2 | **Information** | A single topic. 50-200 words, 0-1 links. A paragraph explaining one thing. |
| 3 | **Proposition** | A developed thought. 200+ words or 2+ outgoing links. An argument or analysis. |
| 4 | **Concept** | A node in your thinking. Links to 3+ notes, uses `generalizes` links. Connects ideas. |
| 5 | **Principle** | A rule or pattern. Links to 3+ concepts, uses `causes` or `supports` links. Explains why things work. |
| 6 | **Theory** | A map of content. 8+ outgoing links, many `part-of` inbound. An MOC (Map of Content) that organizes a domain. |
| 7 | **Paradigm** | A framework referenced by 3+ high-level notes. A lens through which you interpret many things. |
| 8 | **Worldview** | The highest centrality in your graph. The deepest root of your longest derivation chains. Your foundational assumptions. |

### Why it matters

Not all notes are equal, and pretending they are hides the vertical dimension of your knowledge. A quick fact you jotted down is fundamentally different from a theory you synthesized over months. Strata reveals this difference visually — you can see at a glance which ideas are deep and which are surface-level.

This is not a judgment. Level 1 notes are essential — they are the raw material. But if your entire library sits at levels 1-2, you have data without understanding. If you see clusters at levels 5-7, you know your thinking has matured in that area.

### How it works (automatic — no action needed)

Constellation computes stratum automatically from structural signals. There is no manual tagging, no configuration, no AI. The system reads your notes and their connections:

**Base level from word count:**
- 50 words or fewer = Level 1 (Datum)
- 50-200 words = Level 2 (Information)
- 200+ words = Level 3 (Proposition)

**Bonuses from structure (each adds +1 level):**
- 3 or more outgoing links
- 5 or more inbound links
- Uses `generalizes` typed links
- Uses `causes` or `supports` typed links
- Referenced by 3 or more unique source notes

The final level is the base plus all earned bonuses, clamped to a maximum of 8.

### Where you see it

- **Star View**: Higher-level notes appear as **larger nodes**. Level 1-2 notes are small dots. Level 5+ notes are visibly bigger. Level 4+ notes also gain a complementary-colored glow halo — a soft ring of light that makes them stand out.
- The visual contrast tells you immediately: big glowing nodes are your synthesis hubs; small dots are your raw facts.

### Earned Complexity

Strata sizing only activates when your library has **20 or more notes**. Below that threshold, all nodes appear the same size. This prevents visual noise for new users or small libraries — you only see the complexity when there is enough structure to make it meaningful.

### Tutorial: See your knowledge depth

1. Open **Star View** (the graph icon in the left sidebar).
2. Look for the largest nodes — these are your highest-level notes (Concept and above).
3. Notice the glow halo around them — the complementary color makes them pop against any background.
4. Zoom in to see the smaller dots — these are your Datum and Information-level notes.
5. Ask yourself: where is my thinking deep? Where is it still surface-level? The visual difference answers this question instantly.

---

## Feature 3: Maturity Lifecycle

### What it is

Every note in your library has a lifecycle. It begins as a raw capture (a Seed), grows as you connect it to other notes (Sapling), matures into an established reference (Evergreen), and can eventually become an authoritative anchor in your knowledge (Canonical). If you neglect it long enough, it wilts — a gentle signal that it may need revisiting.

### Why it matters

Notes are not static. A note you wrote yesterday and a note you wrote three years ago — both refined, both heavily linked — are at very different stages of intellectual maturity. Maturity tracking shows you which notes are well-established pillars of your thinking and which are still rough sketches waiting to be developed.

It also protects against knowledge rot. When an Evergreen note goes untouched for 90+ days, it dims to a "Wilting" state. This is not punishment — it is a gentle nudge that this knowledge might benefit from a fresh look.

### The 5 states

| State | Visual | Criteria |
|-------|--------|----------|
| Seed | No indicator | 0 inbound links, modified within 1 day of creation |
| Sapling | Light green border (#4ade80) | 1-3 inbound links, OR modified 2+ days after creation |
| Evergreen | Rich green border (#16a34a) | 4+ inbound links AND created 7+ days ago |
| Canonical | Gold border (#f59e0b) | 10+ inbound links AND last modified 30+ days ago (stable, authoritative) |
| Wilting | Faded green border (40% opacity) | Evergreen-level connections (4+ inbound, 7+ days old) but untouched for 90+ days |

### How it works (automatic — no action needed)

Constellation computes maturity from two signals:

1. **Inbound link count** — How many other notes link *to* this note? More inbound links means more of your knowledge depends on this note.
2. **File age and activity** — When was the note created? When was it last modified? A note with many connections that has been stable for months is more mature than one you just wrote.

There is no manual tagging. The states are computed every time you open your library.

### Where you see it

- **File tree**: A colored left border appears on note names. Light green = Sapling, rich green = Evergreen, gold = Canonical, faded green = Wilting. Seeds have no border.
- **Star View**: A colored ring appears around nodes matching their maturity state.
- **Tab bar**: A small colored dot appears before the note title, matching the maturity color.

### Tutorial: Track your note growth

1. Look at your **file tree** in the left sidebar.
2. Notes with a **light green** left border are Saplings — growing, starting to connect.
3. Notes with a **rich green** border are Evergreen — well-established in your knowledge network.
4. Notes with a **gold** border are Canonical — your authoritative references, stable and heavily linked.
5. Notes with a **faded green** border are Wilting — still structurally connected but untouched for a long time.
6. Notes with **no border** are Seeds — new ideas waiting to grow.
7. In Star View, look for the colored rings around nodes to see the same information spatially.

---

## Feature 4: Tension Detector

### What it is

The Tension Detector analyzes your knowledge network for structural problems that you cannot see by reading individual notes. It surfaces four types of issues:

1. **Contradictions** — pairs of notes you have explicitly linked with `|contradicts`
2. **Orphan Notes** — notes with meaningful content but zero inbound links
3. **Structural Gaps** — groups of notes sharing tags but not linking to each other
4. **Single Points of Failure** — notes referenced by 5+ others but deriving from only 1 source

### Why it matters

Healthy knowledge is not just connected — it is coherent. You can have hundreds of linked notes and still harbor unacknowledged contradictions, orphaned insights that never got integrated, clusters of related ideas that never found each other, and entire chains of reasoning that rest on a single unchecked source.

The Tension Detector makes these invisible problems visible. It does not tell you what to *think* — it shows you where your knowledge structure has weaknesses, so you can decide what to do about them.

### The 4 detection types in detail

**Contradictions**

These are notes you have linked with `[[note|contradicts]]`. The Tension Detector collects them in one place so you can see all acknowledged tensions at a glance. Contradictions are not errors — they are signs of intellectual honesty. A knowledge base with zero contradictions is either very small or suspiciously tidy.

Each contradiction is marked as **high severity** because it represents an unresolved intellectual tension.

**Orphan Notes**

An orphan is a note with meaningful content (more than 20 words) but zero inbound links. No other note references it. It exists in your library but is disconnected from your knowledge network.

Severity is based on word count:
- **High**: 500+ words — a substantial note that nobody links to. This is likely a missed connection.
- **Medium**: 100-500 words — a moderate note worth integrating.
- **Low**: 20-100 words — a small note, possibly a stub waiting for development.

Orphans are sorted by severity, with the most substantial disconnected notes shown first.

**Structural Gaps**

A structural gap occurs when multiple notes share the same tag but do not link to each other. For example, if you have 5 notes tagged `#epistemology` but none of them contain wikilinks to any of the others, that is a structural gap — topically related ideas that are structurally isolated.

This detection helps you find connections you may have overlooked. The notes are already about the same topic (you tagged them that way), but your knowledge graph does not reflect this relationship.

**Single Points of Failure**

A single point of failure is a note that 5 or more other notes depend on (via inbound links) but that derives from only 1 source (1 or fewer `derives-from` links). If that single source turns out to be wrong or incomplete, every note that depends on it is compromised.

This is not a call to distrust your sources — it is a prompt to consider whether important foundational notes should have more than one supporting source.

### Earned Complexity

The Tension Detector activates only when your library has **50 or more linked notes**. Below that threshold, the panel shows a progress counter (e.g., "32 / 50 linked notes") instead of detection results. This prevents overwhelming new users with health metrics before there is enough structure to analyze meaningfully.

### Where you see it

- **Right sidebar**: The Health tab, marked with a pulse icon. Click any item to open that note directly.
- Each detection type is shown in a collapsible section with a count badge.
- Severity is indicated by colored dots: red for high, amber for medium, gray for low.

### Tutorial: Check your knowledge health

1. Open the **right sidebar** (if it is not already visible).
2. Click the **Health tab** — the pulse icon.
3. If you have fewer than 50 linked notes, you will see a progress counter. Keep building your library.
4. Once you pass the threshold, you will see 4 collapsible sections:
   - **Contradictions** — Review these for intellectual depth. Do you understand why these notes conflict?
   - **Orphan Notes** — Consider linking the high-severity ones into your network. Where do they belong?
   - **Structural Gaps** — Look at the tag clusters and ask: should these notes reference each other?
   - **Single Points of Failure** — For your most-referenced notes, consider adding more source diversity.

---

## Feature 5: Provenance Chain

### What it is

The Provenance Chain traces the source lineage of every knowledge claim in your library. When you write a note that builds on another note, which itself is based on a book, which cites a paper — the Provenance Chain maps that entire derivation path.

This feature is inspired by the Islamic *isnad* tradition (chain of narration), the oldest and most rigorous system of knowledge provenance in human history. In hadith scholarship, every claim carries a chain of narrators tracing back to the original source. The chain's integrity determines the claim's reliability.

In Constellation, this is computational *isnad*: it counts the chain length and identifies the origin without judging the content. You decide what the provenance means.

### Why it matters

Knowing *where* your knowledge comes from is as important as the knowledge itself. Consider these scenarios:

- You have a note stating a bold claim. Is it your own insight, or did you read it somewhere? If you read it, where? And where did *that* source get it?
- You are writing an essay and want to cite your sources. Can you trace the chain from your synthesis back to the primary evidence?
- You discover that a foundational source was retracted or discredited. Which of your notes are affected?

Provenance answers all of these questions by following `|derives-from` links backward through your note network.

### Two types of knowledge origin

**Received (blue indicator)** — The chain eventually traces to a note with external source markers in its frontmatter. This means the knowledge came from outside your own thinking — a book, paper, website, or other reference. External source markers are: `url`, `author`, `source`, `doi`, `isbn`, or `reference` in the YAML frontmatter.

**Discovered (amber indicator)** — The chain ends at one of your own notes with no external attribution. This is your own insight, your own synthesis. You are the originator.

**Mixed (purple indicator)** — The chain branches and some roots are external while others are your own. This is common for synthesis notes that combine external evidence with personal insight.

### Trust Depth

Trust depth is the number of steps between your current note and the primary source:

- **Depth 0** — You *are* the primary source. No `derives-from` chain exists; this is your original note.
- **Depth 1** — One step from the source. Your note derives directly from the original.
- **Depth 2** — Two steps removed. Your note derives from a note that derives from the source.
- **Depth 4** — Fourth-hand knowledge. Four links in the derivation chain.

Higher depth is not inherently bad, but it is worth being aware of. The longer the chain, the more opportunities for information to be transformed, simplified, or distorted.

### Where you see it

- **Right sidebar**: The Provenance tab (crosshair icon). It shows the full ancestor chain as a visual tree, with each ancestor indented by depth level. Blue dots indicate external sources; amber dots indicate your own notes.
- **Star View**: Received notes have a subtle blue glow; discovered notes have an amber glow (when origin data is available).

### Tutorial: Trace your sources

**Step 1: Create a source note**

Create a note called "Original Research Paper" with this frontmatter:

```yaml
---
author: Jane Smith
url: https://example.com/paper
doi: 10.1234/example
---

The paper argues that X leads to Y under conditions Z.
```

**Step 2: Create an intermediate note**

Create a note called "My Reading Notes" and include:

```
Based on the findings in [[Original Research Paper|derives-from]],
I think the key insight is that X and Y are correlated when Z is present.
```

**Step 3: Create a synthesis note**

Create a note called "My Theory of XYZ" and include:

```
Building on [[My Reading Notes|derives-from]], I propose that
the X-Y relationship also applies in domain W.
```

**Step 4: View the provenance chain**

Open "My Theory of XYZ" and click the **Provenance tab** (crosshair icon) in the right sidebar. You will see:

```
My Theory of XYZ
  -> My Reading Notes (depth 1)
     -> Original Research Paper (depth 2, "external" badge)
```

The origin badge shows **Received** in blue — this theory traces back to an external source. The trust depth is 2, meaning you are two derivation steps from the primary evidence.

**Step 5: Compare with a discovered note**

Create a note called "My Original Insight" with no `derives-from` links and no external source markers in the frontmatter. Check its Provenance tab — it will show **Discovered** in amber with depth 0. This is knowledge that originates from you.

### External source markers

The Provenance Chain recognizes these frontmatter keys as indicators of external sourcing:

| Key | Purpose | Example |
|-----|---------|---------|
| `url` | Web source | `url: https://example.com` |
| `author` | Human author | `author: Jane Smith` |
| `source` | General source reference | `source: Annual Report 2024` |
| `doi` | Digital Object Identifier | `doi: 10.1234/example.5678` |
| `isbn` | Book identifier | `isbn: 978-0-13-468599-1` |
| `reference` | General citation | `reference: Smith et al., 2024` |

Any note containing one or more of these keys with a non-empty value is treated as an external source — a terminal node in the provenance chain.

---

## Feature 6: Externalization Engine (محرك التجسيد)

### What it is

A progressive formalization pipeline that tracks how your notes mature from raw captures to crystallized insights. Every note can be assigned one of four stages:

| Stage | Icon | Meaning |
|-------|------|---------|
| Fleeting | 🌱 | Quick capture, passing thought |
| Literature | 📖 | Rewritten from a source in your own words |
| Permanent | 🔗 | Atomic idea, one concept, connected to your graph |
| Synthesis | ✨ | Original insight combining multiple permanent notes |

### Why it matters

Most apps treat all notes equally. The Externalization Engine makes the distinction visible — you can see at a glance how much of your library is raw capture versus genuine understanding.

### How to use it

**Setting a stage:**
- In the breadcrumb bar (above the editor), use the stage dropdown to select a stage
- Or expand Properties and use the stage dropdown there
- Both sync instantly with the file tree

**Promoting a note:**
- Change the dropdown from one stage to the next
- In Focus mode, click "🔗 Promote to Permanent" at the bottom

**Removing a stage:**
- Select "— Stage —" from the dropdown to remove the stage entirely

### Where you see it

- **Breadcrumb bar**: dropdown with emoji + stage name
- **Properties panel**: dropdown when `stage` property exists
- **File tree**: emoji icon next to the note name
- **Focus mode footer**: "Promote to Permanent" button

### Tips

- Stages are completely optional — notes without a stage work normally
- Start by marking your most important notes as Permanent or Synthesis
- Use Fleeting for quick captures in Focus mode

---

## Feature 7: Review Pulse

### What it is

Review Pulse is a spaced resurfacing system that brings notes back to your attention at expanding intervals. Instead of letting knowledge fade after you write it, Review Pulse nudges you to revisit notes on a schedule: 1 day, then 3 days, then 7, then 14, then 30 days after last review. Each review is a checkpoint — a moment to confirm that you still understand what you wrote and that it still holds true.

Review Pulse also monitors notes tagged with `#assumption` or `#model` as mental model checkpoints. These are ideas that shape how you interpret everything else, so they deserve deliberate periodic re-examination.

Finally, Review Pulse maintains a "Never Reviewed" queue — notes that have existed in your library but have never been explicitly reviewed. These are the ideas you captured and forgot.

### Why it matters

Knowledge decays without revisitation. You write a note today, and in three weeks you have forgotten not just the details but that the note exists at all. Spaced repetition is the most well-established technique in cognitive science for fighting this decay. Review Pulse applies this principle not to flashcards but to your actual notes — the knowledge artifacts you created yourself.

The mental model checkpoint feature is especially important. Your assumptions and mental models are the invisible lenses through which you see everything. If an assumption becomes outdated and you never notice, every conclusion built on top of it is compromised. Review Pulse ensures your foundations get regular inspection.

### How to use it

**Step 1: Open the Review Pulse panel**

Click the **Review Pulse** tab in the left sidebar. You will see three sections:

- **Due for Review** — notes whose next review date has arrived, sorted by urgency
- **Mental Model Checkpoints** — notes tagged `#assumption` or `#model` that are due for re-examination
- **Never Reviewed** — notes that have never been explicitly reviewed

**Step 2: Review a note**

Click any note in the list to open it. Read through it. Then choose one of three actions:

- **Reviewed** (checkmark) — confirms you have re-read the note. The next review is scheduled at the next interval in the sequence (1 → 3 → 7 → 14 → 30 days).
- **Snooze 7d** (eye icon) — pushes the note back by 7 days without advancing the interval. Use this when you do not have time to review properly right now.
- **Dismiss** (archive icon) — removes the note from the review queue entirely. Use this for notes you no longer need to revisit.

**Step 3: Use the Command Palette**

Open the Command Palette and type "Review due notes" to jump directly to notes that are due for review.

### Where you see it

- **Left sidebar**: The Review Pulse tab shows all pending reviews with a badge count indicating how many notes are due.
- **Sidebar badge**: A number badge appears on the Review Pulse tab icon when reviews are pending, so you always know at a glance.
- **Command Palette**: "Review due notes" command for quick access.

### Tips

- Make reviewing a daily habit. Open the Review Pulse tab each morning and spend a few minutes with your due notes. The intervals are designed so this never takes long.
- Tag your core beliefs and working assumptions with `#assumption` or `#model` so they appear in the Mental Model Checkpoints section. These are the notes most worth revisiting.
- Use Snooze sparingly. If you snooze a note repeatedly, consider whether it belongs in your review queue at all — Dismiss might be more honest.
- The Never Reviewed section is a powerful discovery tool. Notes you have never reviewed are notes you have never truly integrated into your thinking.

---

## Feature 8: Trails

### What it is

Trails are named, ordered sequences of notes — like chapters in a book or stops on a guided tour through your knowledge. When you create a trail, you define a specific path through your library: "read this note first, then this one, then this one." Each note in a trail knows its position and provides navigation to the previous and next note in the sequence.

A trail is defined by adding `trail: true` to a note's frontmatter. The note's content then serves as the trail definition — an ordered list of wikilinks that form the sequence.

### Why it matters

Knowledge is not always a web. Sometimes it is a path. When you learn a new subject, you follow a sequence: fundamentals first, then intermediate concepts, then advanced applications. When you explain something to someone else, you choose an order. Trails let you capture that order explicitly.

Without trails, your library is a non-linear web of connections — powerful for exploration, but unhelpful when you need to say "start here and follow this path." Trails add the linear dimension back, giving you the ability to create guided tours, reading sequences, learning paths, and argument progressions through your existing notes.

### How to use it

**Step 1: Create a trail note**

Create a new note and add `trail: true` to the frontmatter:

```yaml
---
trail: true
---
```

**Step 2: Define the sequence**

In the note body, list wikilinks in the order you want them to be followed:

```markdown
# Introduction to Epistemology

1. [[What is Knowledge]]
2. [[The Problem of Justification]]
3. [[Foundationalism vs Coherentism]]
4. [[The Gettier Problem]]
5. [[Virtue Epistemology]]
```

Each wikilink becomes a stop on the trail. The order in the note defines the order of the trail.

**Step 3: Navigate the trail**

When you open any note that belongs to a trail, the breadcrumb bar shows a trail indicator with the trail name. Arrow buttons let you navigate to the previous and next note in the sequence without leaving the editor.

**Step 4: Open a trail from the Command Palette**

Open the Command Palette and type "Open Trail" to see a list of all trails in your library. Select one to open the trail definition note.

### Where you see it

- **Breadcrumb bar**: When viewing a note that belongs to a trail, a trail indicator appears showing the trail name and position (e.g., "Epistemology 3/5"). Previous and next navigation arrows let you move along the trail.
- **Trail definition note**: The note with `trail: true` in frontmatter serves as the trail's table of contents.
- **Command Palette**: "Open Trail" command lists all trails for quick access.

### Tips

- Use trails for onboarding sequences: create a "Start Here" trail that walks a newcomer through the most important notes in your library.
- Use trails for argument construction: lay out the steps of a complex argument in order, so you can walk through the reasoning from premise to conclusion.
- A note can belong to multiple trails. The breadcrumb shows whichever trail you navigated from.
- Trail notes themselves can be linked to from anywhere. They serve as curated entry points into specific topics or narratives in your knowledge.

---

## The Big Picture

These 8 features work together as a unified system:

1. **Typed Links** give your connections meaning — not just "related" but "supports," "contradicts," "derives-from."
2. **Knowledge Strata** reveals the depth of your understanding — from raw facts to synthesized paradigms.
3. **Maturity Lifecycle** tracks how your notes grow over time — from fresh seeds to authoritative references.
4. **Tension Detector** surfaces what is disconnected, contradictory, or fragile — the structural weaknesses in your thinking.
5. **Provenance Chain** traces where your knowledge comes from — external evidence or personal insight, and how many steps removed.
6. **Externalization Engine** makes the formalization journey visible — from fleeting capture to synthesized insight.
7. **Review Pulse** ensures you revisit what you know — spaced resurfacing that fights knowledge decay and keeps your mental models honest.
8. **Trails** add the linear dimension — named ordered sequences that turn your web of knowledge into guided paths.

Together, they answer the question every knowledge worker eventually asks: *Do I truly understand what I think I understand?*

The Cognitive Engine does not answer that question for you. It gives you the structural awareness to answer it yourself.

### How it all connects

- When you add a **typed link**, that link feeds into **Strata** (certain types like `generalizes` and `supports` boost your note's level), **Maturity** (each inbound link brings a note closer to Evergreen), **Tension** (a `contradicts` link registers as a detected contradiction), and **Provenance** (a `derives-from` link extends the source chain).
- When you set a **stage** on a note, the Externalization Engine tracks its formalization progress. A Fleeting note promoted to Permanent signals that a raw idea has been refined into a standalone concept.
- When **Review Pulse** resurfaces a note, you have the opportunity to add new links, update stale content, or re-evaluate assumptions — feeding back into every other CE feature.
- When you create a **Trail**, you impose narrative order on your knowledge web, making it possible to walk through complex ideas step by step.
- A single action — typing `[[note|derives-from]]` — simultaneously enriches all eight systems. You do not need to think about the CE explicitly. Just write, link, and think. The engine observes and reflects.

### Earned Complexity

The Cognitive Engine follows a principle of **earned complexity**: features reveal themselves only when your library has enough structure to make them meaningful.

- **Star View strata sizing**: activates at 20+ notes
- **Tension Detector**: activates at 50+ linked notes
- **Provenance Chain**: always available (even a single `derives-from` link creates a chain)
- **Maturity colors**: always visible in the file tree and tabs

Below these thresholds, the UI stays clean and simple. As your library grows, the CE grows with it.

---

## Coming Soon (Layer 2)

The current Cognitive Engine is **Layer 1** — structural intelligence built from your note topology. No AI is involved. Every computation reads wikilinks, word counts, frontmatter, and file metadata. Nothing more.

Future phases will add **Layer 2** — AI-powered discovery on top of these structural tools:

- **Hidden pattern discovery** — find conceptual overlaps between notes that are not explicitly linked
- **Blind spot detection** — identify topics you reference frequently but never develop deeply
- **Cross-domain insights** — surface unexpected connections between different areas of your knowledge
- **Socratic questioning** — prompt you with questions that challenge your assumptions
- **Worldview synthesis** — map your deepest assumptions and how they shape your thinking

Layer 2 requires no separate setup. It will read the structures you have already built with Layer 1 — your typed links, strata levels, maturity states, and provenance chains. Everything you do today with the CE prepares the foundation for deeper intelligence tomorrow.
