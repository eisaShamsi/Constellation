# Knowledge Formulation

## What It Is

Constellation is not a note-taking app. It is a **Knowledge Formulation** system — a tool designed to help you BUILD understanding, not just store information.

The difference:
- **Knowledge Management**: "Where did I put that note?"
- **Knowledge Formulation**: "What can I BUILD from what I know?"

## The Living Link

In Constellation, a link between two notes is not a dead pointer. It is a **living connection** that:

- Has a **type** — what kind of relationship? (supports, contradicts, causes...)
- Has an **annotation** — why does this connection exist?
- Has a **weight** — how significant is this connection? (grows with use)
- Has a **confidence** — how certain is this? (hypothesis → evidence → established)
- Has a **history** — when was it created? When last used? How often?

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

## The Five Acts of Knowledge Creation

### Act I: Observation
You encounter something — a fact, an idea, a quote. You capture it in a note. No links yet.

### Act II: Connection
You realize this relates to something you already know. You create a typed link: "This supports that."

### Act III: Tension
You discover a contradiction. Something challenges your existing understanding. You create a `contradicts` link. This tension is where real thinking begins.

### Act IV: Synthesis
You think deeply about the tension and arrive at a new understanding that resolves it. A `generalizes` link captures this higher-level insight.

### Act V: Conviction
Over time, evidence accumulates. Your links strengthen through use. Confidence upgrades from hypothesis to established. The idea becomes part of how you see the world.

## How to Create Links

### Basic Link
```
[[Note Name]]
```

### Typed Link
```
[[supports::Note Name]]
```

### Typed Link with Annotation
```
[[supports::Note Name|My field data confirms this]]
```

## Confidence Levels

| Level | Meaning |
|-------|---------|
| **Hypothesis** | "I think this might be true" — early stage, no evidence yet |
| **Evidence** | "I have some basis for this" — data, a paper, an observation |
| **Established** | "This is well-supported" — multiple sources, tested |
| **Contested** | "This is actively debated" — contradicting evidence exists |

## Link Lifecycle

Links are alive. They grow with use and fade with neglect:

1. **Birth** — You create the link. Weight starts at 1.0.
2. **Growth** — Each time you follow the link, its weight increases.
3. **Maturity** — High weight, high confidence, regularly used. This is a core connection in your thinking.
4. **Dormancy** — You haven't used this link in months. Weight slowly decays.
5. **Renewal** — You rediscover the connection. Weight jumps back.
6. **Archival** — You decide this connection is no longer relevant. It's preserved in history but hidden from active search.

## Searching Your Knowledge

The search engine lets you query your link network:

- `supports [[Democracy]]` — What evidence supports this idea?
- `contradicts [[My Thesis]]` — What challenges my thesis?
- `causes [[Climate Change]]` — What leads to this outcome?
- `derives-from [[Ancient Philosophy]]` — What ideas came from this source?
- `orphans` — Which notes have no connections? (isolated cells)

All search operators work in your language — Arabic, French, Japanese, and 12 others.
