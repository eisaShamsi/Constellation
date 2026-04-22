# Knowledge Formulation

## What It Is

Constellation is not a note-taking app. It is a **Knowledge Formulation** system — a tool designed to help you BUILD understanding, not just store information.

The difference:
- **Knowledge Management**: "Where did I put that note?"
- **Knowledge Formulation**: "What can I BUILD from what I know?"

## The Living Link

In Constellation, a link between two notes is not a dead pointer. It is a **living connection** that carries eight properties:

| Property | What it is |
|---|---|
| **Type** | What kind of relationship? (supports, contradicts, causes, exemplifies, generalizes, derives-from, part-of) |
| **Direction** | Source → target; backlinks flow the other way. |
| **Annotation** | Why does this connection exist? Your reasoning at the moment of linking. |
| **Weight** | How significant? Starts at 1.0, grows logarithmically with each traversal. |
| **Confidence** | How certain? Hypothesis → Evidence → Established → Contested. |
| **Created** | When the link was born. |
| **Last Traversed** | When you last followed it. Drives stale detection. |
| **Traversal Count** | How many times you've followed it. Drives the tier gradient. |

## The Seven Link Types

| Type | What It Means | When to Use |
|------|---------------|-------------|
| **supports** | A provides evidence for B | "This data confirms my thesis" |
| **contradicts** | A challenges or opposes B | "This finding conflicts with my assumption" |
| **causes** | A leads to or produces B | "This event triggered that outcome" |
| **exemplifies** | A is a concrete instance of B | "This is a real example of that concept" |
| **generalizes** | A abstracts a pattern from B | "This principle emerges from those specifics" |
| **derives-from** | A originates from B | "This idea came from that source" |
| **part-of** | A is a component of B | "This chapter belongs to that book" |

Anything else (including no type at all) falls back to the untyped default, `relates`.

## The Five Acts of Knowledge Creation

1. **Observation** — You encounter something and capture it in a note. No links yet.
2. **Connection** — You realize this relates to something you already know. You create a typed link.
3. **Tension** — You discover a contradiction. You create a `contradicts` link. This tension is where real thinking begins.
4. **Synthesis** — You resolve the tension with a new understanding. A `generalizes` link captures this higher-level insight.
5. **Conviction** — Evidence accumulates over time. Your links strengthen through use. Confidence upgrades from hypothesis to established.

---

# Tutorials

Every Living Link function, step by step.

## Tutorial 1 — Your first typed link

**What it does**: Creates a wikilink that carries a type (the kind of relationship) so Constellation knows *why* two notes are connected.

**Steps**:
1. Open any note in the editor.
2. Place the cursor where you want the link.
3. Type: `[[supports::Mughal Empire]]` (two square brackets, the type, two colons, the target note name, two square brackets).
4. Save. In Live Preview, the link renders as the target's name with a small **supports** badge beside it in Backlinks / Outgoing panels.

**Recognized types**: `supports`, `contradicts`, `causes`, `exemplifies`, `generalizes`, `derives-from`, `part-of`. Anything else is parsed as an untyped `relates` link.

---

## Tutorial 2 — Adding an annotation (the "why")

**What it does**: Attaches your reasoning to a link. The annotation is the single most valuable piece of link data — it's the *why*.

**Steps**:
1. Write a typed link as in Tutorial 1.
2. Add a single pipe `|` after the target, followed by your reasoning: `[[supports::Mughal Empire|Babur launched his 1526 invasion from Kabul]]`.
3. Save.
4. Open the target note (Mughal Empire) and look at its right-sidebar → Backlinks tab. Under the source note's context excerpt, you'll see your annotation in italic purple quotation marks.

**Rule**: Only one pipe is allowed per link. Don't write `[[target|display|annotation]]` — the parser treats everything after the first pipe as one string.

**Tip**: Write annotations as if you're explaining the connection to a future reader (including future-you). "Because the timeline fits" is more useful six months later than "see above."

---

## Tutorial 3 — Following a link and watching the tier grow

**What it does**: Every time you click a wikilink, Constellation counts the traversal. The ×N chip and tier color reflect how worn the path is.

**Steps**:
1. In any note, click a wikilink. The target opens in a new tab or the current tab depending on your setting.
2. Return to the source note. The wikilink now shows a small `×1` chip beside it.
3. Click the same link 2 more times. Chip shows `×3` and changes color (moves from *emerging* to *established* tier).
4. Click it 7 more times (total 10). Chip becomes solid purple — **load-bearing**. This is a worn path in your thinking.

**Tier thresholds**:

| Traversals | Tier | Visual |
|---|---|---|
| 1–2 | emerging | faint tint |
| 3–9 | established | stronger tint |
| 10+ | load-bearing | solid fill, white text |
| 90+ days since last traversal | stale | amber |

---

## Tutorial 4 — Reading confidence vs tier

**Tier** (visual) and **Confidence** (truth-stance) are related but distinct:

- **Tier** is earned passively — it comes from how often you use the link.
- **Confidence** is your epistemic stance — how sure you are the connection is true.

By default, confidence auto-promotes with traversal:
- ×1–2 → Hypothesis
- ×3–9 → Evidence
- ×10+ → Established
- User-set `Contested` is preserved and never overridden by the auto-rule.

To see the confidence level, **right-click** a link row in Backlinks or Outgoing Links — the current level is highlighted in the menu.

---

## Tutorial 5 — Contesting or force-promoting confidence

**What it does**: Sometimes you disagree with the auto-promotion. Maybe a link has ×20 traversals but you've since found evidence contradicting it — you want to mark it **contested**. Or you just created a link and already have strong evidence — you want to jump straight to **established**.

**Steps**:
1. Open the right sidebar → **Backlinks** tab (or Outgoing Links).
2. Find the link you want to re-tag.
3. **Right-click** the link row (left-click navigates). A popover appears at your cursor.
4. Choose one: **Hypothesis**, **Evidence**, **Established**, or **Contested**. The current level is highlighted.
5. The popover closes. The change is persisted immediately.

**Verifying**: Right-click the same row again. The new level is highlighted.

**Why it matters**: Confidence is searchable. Once you mark something `contested`, you can find all your contested connections in one query and revisit them.

---

## Tutorial 6 — Archiving a link (soft delete, reversible)

**What it does**: Removes a link from Backlinks, Outgoing, Most-Traveled, and Stale views without deleting the underlying data. The link is preserved in history and can be restored at any time.

**Steps**:
1. Right-click the link row in Backlinks or Outgoing Links.
2. In the popover, below the four confidence options, click **Archive link**.
3. The row disappears from the panel immediately.

**Under the hood**: The DB row is marked `status='archived'` and its weight is zeroed. Traversal count and confidence are preserved so you don't lose history.

---

## Tutorial 7 — Restoring an archived link

**What it does**: Brings an archived link back to life. Traversal count and confidence are restored; weight resets to 1.0 (the link has to earn its weight again).

**Steps**:
1. Open the right-sidebar **Link Dashboard** (share-2 icon, last in the tab row).
2. Click the **Archived** tab (last tab, rightmost).
3. The list of archived links appears. Source names are italic and muted to signal inactivity.
4. Click the circular-arrow button at the end of the row you want to restore.
5. The row disappears from Archived. Open the source note — the link is back in Outgoing Links.

---

## Tutorial 8 — Running the confidence back-fill (one-shot)

**What it does**: Promotes existing links whose traversal count already crossed a tier threshold but never went through the auto-promotion rule (for example, links that aged before confidence auto-promotion shipped).

**Steps**:
1. Open **Settings** → **Appearance** → scroll to **Living Link Lifecycle**.
2. Find the **Back-fill link confidence** row.
3. Click **Run back-fill**. The button shows "Running…" briefly.
4. When done, a line appears in accent color: *Promoted N link(s) (→evidence: X, →established: Y).*

**Safe to re-run**: Idempotent. Never downgrades. Preserves user-set `contested`.

---

## Tutorial 9 — Tuning decay (half-life)

**What it does**: Controls how fast a link's effective weight halves when you don't use it. Affects sort order in Backlinks / Outgoing / Most-Traveled — not the raw data.

**Steps**:
1. Open **Settings** → **Appearance** → **Living Link Lifecycle**.
2. **Apply weight decay to link sorts** toggle — turn off if you want raw traversal count only.
3. **Decay half-life** slider — drag between 7 and 365 days. Lower = faster drop-off. Default 60 days.

**Guideline**:
- 30 days — aggressive. "What's alive in my thinking right now?"
- 60 days — balanced default.
- 120 days — gentle. Good for slow-paced research vaults.
- 365 days — nearly off. Weight is effectively permanent.

---

## Tutorial 10 — The Link Dashboard

**How to open**: Right sidebar → share-2 icon (last tab in the row).

The Dashboard has seven tabs. Each answers a specific diagnostic question about your knowledge network:

| Tab | Question it answers |
|---|---|
| **Most Connected** | Which notes have the most links in or out? Your thinking hubs. |
| **Most Traveled** | Which links have you actually walked most often? (Not the same as "most connected" — a much-traveled link is a *worn path*.) |
| **Stale** | Which links haven't been touched in 90+ days? Candidates to revisit, update, or retire. |
| **Cross-Library** | Which links cross library boundaries? Your federation in action. |
| **Broken** | Which links point to notes that don't exist? Targets to create or wikilinks to fix. |
| **Orphans** | Which notes have zero links? Isolated cells in need of integration. |
| **Archived** | Which links did you soft-delete? Listed newest-first with a one-click restore button. |

---

## Tutorial 11 — Searching your knowledge network

Constellation's search is a diagnostic instrument. These queries work in any of the 15 supported languages:

| Query | What it returns |
|---|---|
| `supports [[Democracy]]` | Notes that `supports`-link to *Democracy*. |
| `contradicts [[My Thesis]]` | Counter-evidence against your thesis. |
| `causes [[Climate Change]]` | Notes `causes`-linked to climate change. |
| `derives-from [[Ancient Philosophy]]` | Your ideas that trace back to this source. |
| `orphans` | Isolated notes (no incoming or outgoing links). |

---

## Field Guide: When to use which type

- **supports / contradicts** — the workhorse pair. Use these when you're building or challenging an argument.
- **causes** — reserve for actual causal claims, not mere correlation. If you're unsure, use `supports`.
- **exemplifies** — "here's a concrete case of that abstract idea."
- **generalizes** — the reverse. "Here's the pattern these specifics share."
- **derives-from** — intellectual lineage. Who did you learn this from? What's the ancestor idea?
- **part-of** — structural membership. A chapter is `part-of` a book. A concept is `part-of` a framework.

---

## Keyboard shortcuts

- **Right-click a link row** (in Backlinks or Outgoing) — opens the confidence/archive popover.
- **Ctrl/Cmd-click a wikilink** — opens the target in a new tab.
- **Middle-click a wikilink** — same as Ctrl-click.
