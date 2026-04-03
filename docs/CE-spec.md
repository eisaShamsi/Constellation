# Constellation Cognitive Engine — Complete Specification
**"Constellation does not manage knowledge. It helps the user know."**

Derived from: `docs/constellation_cognitive_engine_v2.1.pdf`
Roadmap index: `docs/cognitive-engine-roadmap.md`
Session log: `lab/reports/SESSION-LOG-YYYY-MM-DD.md`

---

## 0. Vision

The Cognitive Engine is a two-layer architecture that transforms Constellation from a note-taking app into a knowledge cognition instrument.

**Layer 1 — Structural Cognition** (12 tools, zero AI dependency):
Tools that work through data structure, graph topology, and metadata. Operate fully offline.

**Layer 2 — AI Discovery** (5 capabilities):
AI reads Layer 1's structures to find what the user cannot see from inside their own knowledge.

**Governing principle**: Complexity absorbed by the system, simplicity experienced by the user. Every tool must feel like thinking, not like operating software.

**Seven epistemological foundations** (from the Philosophy of Knowledge paper):
1. Knowledge is not information — value is in connections, not storage
2. Knowledge has a vertical dimension — 8-level hierarchy (Datum → Worldview)
3. Knowledge has a certainty dimension — ilm al-yaqin → haqq al-yaqin (invisible, structural)
4. Knowledge is organized by immutable principles — non-contradiction, causality, hierarchy
5. Knowledge has diverse sources — sensory, rational, transmitted, experimental, intuitive
6. Knowledge exists on a spectrum — received (from authority) vs. discovered (by user)
7. The essence of knowledge is understanding-generative apprehension — enables explain, predict, act

---

## 1. Foundation Inventory (Pre-existing, Do Not Re-build)

| Component | Location | Notes |
|---|---|---|
| GraphMind (Pixi.js force graph) | `src/lib/graph/graphEngine.ts` | 3 layouts, semantic engine, cluster detection |
| Wikilinks + Backlinks | `src/lib/libraries/store.ts`, Rust | Full bidirectional, cross-library, rename-update |
| YAML frontmatter + property system | `src/routes/+layout.svelte`, `src/lib/components/PropertyEditor.svelte` | 7 property types, auto type-detection |
| Tags + TagsPanel | `src/lib/components/TagsPanel.svelte` | Full scan, browse, filter |
| AI integration | `src-tauri/src/` (`ai_send_message`, `ai_validate_connection`) | Embeddings, semantic engine |
| FocusPane (= Fleeting capture) | `src/lib/components/FocusPane.svelte` | Maps to Externalization Stage 1 |
| Dataview / Bases query system | `src-tauri/src/` (`execute_dataview_query`) | Database-like queries |
| Full-text + property search | `src-tauri/src/` (`search_stars`, `search_by_property`) | Fast Rust-side |
| Tasks + Calendar | `src/lib/components/TasksPanel.svelte`, `CalendarPanel.svelte` | `scan_library_tasks` |

---

## 2. Build Rules (applies to every phase)

1. **Each phase has GO/NO-GO test plan** — user tests before next phase begins
2. **No re-testing passed phases** in subsequent phases
3. **Commit + tag milestone** after each passed phase
4. **Update session log (SO)** after every phase
5. **No IPC during typing** — zero `invoke()` calls in the typing path
6. **No AI dependency in Layer 1** — all 12 tools work offline, always
7. **No feature that makes the app slower** — every keystroke must remain instant
8. **No $effect loops** — use `$derived` for computed values
9. **Editor Parity Rule** — new editor features apply to all note views (except FocusPane)

---

## 3. UX Principles (applies to every phase)

| # | Principle | Implementation |
|---|---|---|
| P1 | Zero-configuration start | New user sees clean editor. No setup required. |
| P2 | Earned complexity | Tools reveal as library grows. Strata: 20+ notes. Tension Detector: 50+ linked notes. |
| P3 | Ambient indicators | Maturity = border colors. Strata = node sizes. Never pop-ups, never interruptions. |
| P4 | Optional depth | Typed links optional. Canvas quadrants optional. User chooses their depth. |
| P5 | One-action transitions | Promote canvas item: one click. Type a link: one pipe character. |
| P6 | RTL as native citizen | Every tool, view, and label works multidirectionally. |
| P7 | Invisible framework | The 20 frameworks are in the glass, not in the menu. Never explain them to the user. |

---

## LAYER 1 — STRUCTURAL COGNITION TOOLS

---

## Phase 1: Typed Links (الروابط الدلالية) ✅ COMPLETE

**Commit**: `d7edc6d` | **Date**: 2026-03-30

### Goal
Extend `[[wikilink]]` syntax to carry semantic meaning. The keystone feature — every downstream tool depends on it.

### Syntax
```
[[note|supports]]      — evidence for a claim         (blue  #4A9EFF)
[[note|contradicts]]   — tension / opposition          (red   #FF4A4A)
[[note|causes]]        — causal relationship           (orange #FF8C42)
[[note|exemplifies]]   — instance-of                  (green #4AFF88)
[[note|generalizes]]   — abstraction                  (purple #A44AFF)
[[note|derives-from]]  — provenance / source          (gold  #FFD700)
[[note|part-of]]       — compositional hierarchy      (gray  #AAAAAA)
```
Untyped links default to `associative`. Power users type; beginners never need to.

### Components Changed
| File | Change |
|---|---|
| `src-tauri/src/libraries.rs` | Parse `[[note\|type]]` directly; `KNOWN_LINK_TYPES` constant |
| `src/lib/libraries/store.ts` | `getBacklinks()` passes `linkType` through |
| `src/lib/components/BacklinksPanel.svelte` | Colored badge per link type |
| `src/lib/editor/livePreview.ts` | Typed links show note name in type color; hide `\|type` |
| `src/lib/editor/completions.ts` | `createTypedLinkCompletion()` — triggers on `[[note\|` |
| `src/lib/components/NotePane.svelte` | Wires `typedLinkCompletion` (highest priority) |
| `src/lib/graph/graphEngine.ts` | Type colors in normal + hover; `contradicts` = bidirectional; `causes` = thicker |

### Test Plan (18 tests — all passed)
1. `[[Philosophy of Knowledge|supports]]` — saves, reopens, link preserved ✓
2. `[[Tension|contradicts]]` — BacklinksPanel shows red "contradicts" badge ✓
3. `[[Source Book|derives-from]]` — GraphMind shows gold dotted line ✓
4. `[[plain link]]` (no pipe) — behaves exactly as before ✓
5. `[[note|foobar]]` (unknown type) — treated as `associative` silently ✓
6. Inside `[[` → type note name → type `|` → completion list appears ✓
7. Select `contradicts` → `]]` auto-closes → `[[note|contradicts]]` ✓
8. Press Escape during type completion → no link type inserted ✓
9. GraphMind: `contradicts` = red bidirectional ✓
10. GraphMind: `causes` = orange thicker arrow ✓
11. GraphMind: `derives-from` = gold dotted ✓
12. GraphMind: untyped = gray, no arrowhead ✓
13. BacklinksPanel: inbound `supports` → blue badge ✓
14. BacklinksPanel: mixed link types → each badge correct color ✓
15. `[[note|]]` (empty type) → treated as `associative` ✓
16. `[[note name with spaces|causes]]` → parses correctly ✓
17. `[[note#heading|derives-from]]` → target=note, type=derives-from ✓
18. Existing untyped links → zero regression ✓

---

## Phase 2: Knowledge Strata (طبقات المعرفة) ✅ COMPLETE

**Commit**: `0f6d4bf` | **Date**: 2026-04-02
**Depends on**: Phase 1 (link types enrich stratum signals)
**Unlocks**: Tension Detector (strata-aware orphans), Review Pulse (priority by stratum)

### Goal
Auto-classify every note into an 8-level knowledge hierarchy. No manual tagging. The system reads structural signals.

### 8-Level Hierarchy
| Level | Name | Structural Signals |
|---|---|---|
| 1 | Datum | ≤50 words, 0 links, raw fact |
| 2 | Information | 50–200 words, 0–1 links, single topic |
| 3 | Proposition | Single claim, ≤1 `derives-from` source |
| 4 | Concept | Links 3+ propositions, abstracts pattern; has `generalizes` links |
| 5 | Principle | Links 3+ concepts, states general rule; has `causes` or `supports` links |
| 6 | Theory | Map-of-Content; 8+ outgoing links to principles; `part-of` links from many |
| 7 | Paradigm | Referenced by 3+ theories; has very high betweenness centrality |
| 8 | Worldview | Highest centrality note; root of the deepest `derives-from` chain |

### Computation (Rust-side, never in JS)
Signal weights per note:
- `word_count`: short → low level, long → higher level
- `outgoing_link_count`: proxy for synthesis depth
- `inbound_link_count`: proxy for importance
- `link_types_present`: `generalizes`, `causes`, `supports` each add +1 level
- `betweenness_centrality_proxy`: notes with many different-source inbound links score higher

Formula: `stratum = clamp(base_from_word_count + link_bonus + type_bonus, 1, 8)`

No ML, no AI. Pure graph topology computation.

### New Rust Command
```rust
#[tauri::command]
pub fn compute_note_strata(library_path: String) -> Result<Vec<NoteStratum>, String>

pub struct NoteStratum {
    pub note_path: String,
    pub note_name: String,
    pub stratum: u8,        // 1–8
    pub word_count: usize,
    pub outgoing_links: usize,
    pub inbound_links: usize,
}
```

### GraphMind Visual
- Node **radius** = `base_radius + (stratum - 1) * 2.5`
- Node **glow intensity** = `0.0 + (stratum - 1) * 0.15` (alpha of glow halo)
- Propositions (3): small dim dots
- Theories (6): large luminous hubs
- User sees gravitational structure at a glance — no labels needed

### Bloom Cognitive Depth (secondary indicator)
Computed from edit history approximation:
- Note created but never linked → Remember (low)
- Note has outgoing links → Understand (medium)
- Note has `causes` or `generalizes` links to other notes → Analyze
- Note is referenced by synthesis-level notes → Create (high)
Shown as subtle ring inside node in GraphMind.

### Earned Complexity Rule
Strata computation and GraphMind visual activate only when library has 20+ notes. Below 20 notes: all nodes render identically (no noise for new users).

### Files to Change
| File | Change |
|---|---|
| `src-tauri/src/strata.rs` (NEW) | `compute_note_strata` command |
| `src-tauri/src/lib.rs` | Register new command |
| `src/lib/graph/graphEngine.ts` | Node radius + glow by stratum |
| `src/lib/graph/GraphMindView.svelte` | Fetch + pass strata data |
| `src/routes/+layout.svelte` | Call `compute_note_strata` on library load |

### Test Plan
1. Library with 5 notes (all unlinked) → all Level 1–2, GraphMind nodes all same size ✓
2. Add 20 notes → strata activate in GraphMind ✓
3. Create a note with 5+ outgoing `generalizes` links → stratum ≥ 5 ✓
4. Most-linked note has largest node in GraphMind ✓
5. Open 5000-word note + scroll → no lag (strata computed at load, not on scroll) ✓
6. Switch libraries → strata recompute correctly ✓
7. Rust command returns correct struct; zero JS parsing ✓

### GO/NO-GO
GO if tests 1–7 pass and no typing lag introduced.

---

## Phase 3: Maturity Lifecycle (دورة النضج) ✅ COMPLETE

**Commit**: `5cf4283` | **Date**: 2026-04-02
**Depends on**: Wikilinks (existing), file metadata (existing)
**Unlocks**: Review Pulse (staleness source), Tension Detector (orphan severity)

### Goal
Track note growth through 4 maturity states. No manual tagging. Computed from structural signals.

### 4 States
| State | Arabic | Signals | Visual |
|---|---|---|---|
| 🌱 Seed | بذرة | Newly created, 0 inbound links, edited ≤1 time | Faint dotted border in file tree |
| 🌿 Sapling | شتلة | Edited ≥2 times, 1–3 inbound links | Thin solid border, light green |
| 🌳 Evergreen | دائمة الخضرة | 3+ edit sessions, 4+ inbound links, referenced by others | Full border, rich green |
| ⭐ Canonical | مرجعية | Referenced by 10+ notes, last modified 30+ days ago | Golden border + star icon |

**Decay**: Evergreen note with no visits in 90+ days, while its tag-domain has active new notes → "wilting" state (subtle dimming in file tree). Signal: potential staleness.

### Signals Used
All from existing Rust commands — no new file writes:
- `get_file_metadata()` → `modified_time`, `created_time`, `size`
- `scan_library_links()` → inbound link count
- File size delta across two readings → proxy for "edited again"

### Visit Tracking
Lightweight: record last-opened note path + timestamp in a `.constellation/maturity.json` sidecar file per library (never inside .md files). Written only on tab close, not on every keystroke.

### New Rust Command
```rust
#[tauri::command]
pub fn compute_note_maturity(library_path: String, all_links: Vec<NoteLink>) -> Result<Vec<NoteMaturity>, String>

pub struct NoteMaturity {
    pub note_path: String,
    pub state: String,      // "seed" | "sapling" | "evergreen" | "canonical" | "wilting"
    pub inbound_count: usize,
    pub days_since_modified: u64,
}
```

### Visual Locations
1. **File tree**: colored left-border on note filename
2. **GraphMind**: subtle ring color around node (same 4 colors)
3. **NotePane tab**: tiny colored dot next to note name in tab bar

### Files to Change
| File | Change |
|---|---|
| `src-tauri/src/maturity.rs` (NEW) | `compute_note_maturity` + visit tracking |
| `src-tauri/src/lib.rs` | Register command |
| `src/lib/components/FileTree.svelte` | Colored border per maturity state |
| `src/lib/graph/graphEngine.ts` | Ring color around nodes |
| `src/routes/+layout.svelte` | Compute maturity on library load; refresh on tab close |

### Test Plan
1. Create new note → shows Seed in file tree ✓
2. Edit same note 3 times → advances to Sapling ✓
3. Note referenced by 4+ others → advances to Evergreen ✓
4. Canonical note: referenced by 10+, untouched 30+ days → golden star ✓
5. Wilting: Evergreen note untouched 90 days → dims in file tree ✓
6. Open 100-note library → file tree renders without lag ✓
7. Switch libraries → maturity states update correctly ✓

### GO/NO-GO
GO if tests 1–7 pass and file tree renders without visible delay.

---

## Phase 4: Tension Detector (كاشف التناقضات)

**Status**: 🔲 Not started
**Depends on**: Phase 1 (needs `contradicts` link type), Phase 3 (orphan severity)
**Unlocks**: Layer 2 tension analysis (Phase 12)

### Goal
Surface contradictions and knowledge gaps. Zero AI. Presented as a gentle "knowledge health" panel — not alarms, not judgments.

### 4 Detection Types
1. **Contradictions**: Notes linked with `|contradicts`, or notes sharing a topic tag with opposing typed links to a third note
2. **Orphan knowledge**: Notes with zero inbound links — disconnected from the understanding network
3. **Structural gaps**: Tag-groups (clusters of notes sharing ≥2 tags) with no wikilinks between them
4. **Single points of failure**: Concepts referenced by 5+ notes but deriving from only 1 source

### Earned Complexity Rule
Tension Detector activates only when library has 50+ linked notes. Below threshold: panel shows "Add more links to activate knowledge health monitoring."

### UI
- Right sidebar tab: "Health" (alongside Backlinks, Tags, Properties)
- Gentle accordion sections per tension type
- Each item: note name → click to open
- Severity indicator: low (gray), medium (amber), high (red dot)
- No pop-ups, no notifications, no interruptions

### New Rust Command
```rust
#[tauri::command]
pub fn detect_tensions(library_path: String, min_notes: usize) -> Result<TensionReport, String>

pub struct TensionReport {
    pub contradictions: Vec<TensionItem>,
    pub orphans: Vec<TensionItem>,
    pub structural_gaps: Vec<GapItem>,
    pub single_points: Vec<TensionItem>,
}
pub struct TensionItem {
    pub note_name: String,
    pub note_path: String,
    pub severity: String, // "low" | "medium" | "high"
    pub detail: String,
}
```

### Files to Change
| File | Change |
|---|---|
| `src-tauri/src/tension.rs` (NEW) | `detect_tensions` command |
| `src-tauri/src/lib.rs` | Register command |
| `src/lib/components/TensionPanel.svelte` (NEW) | Knowledge health sidebar panel |
| `src/routes/+layout.svelte` | Add "health" tab to right sidebar; wire data |

### Test Plan
1. Create two notes linked with `|contradicts` → both appear in Contradictions list ✓
2. Create isolated note (no links) → appears in Orphans ✓
3. Two tag-clusters with no cross-links → appear in Structural Gaps ✓
4. Note referenced by 6+ others, only one `derives-from` source → Single Point of Failure ✓
5. Library with 30 notes → panel shows "not enough links" message ✓
6. Library with 60+ linked notes → panel activates ✓
7. Click tension item → opens that note in editor ✓
8. No typing lag when panel is open ✓

### GO/NO-GO
GO if tests 1–8 pass.

---

## Phase 5: Provenance Chain (سلسلة الإسناد) ✅ COMPLETE

**Commit**: `2de0c15` | **Date**: 2026-04-02
**Depends on**: Phase 1 (`derives-from` typed link)
**Unlocks**: Layer 2 Blind Spot Detection (Phase 13 uses weak-provenance signals)

### Goal
Track source lineage for every knowledge claim. Inspired by the Islamic isnad tradition — history's most rigorous knowledge provenance system. Computational isnad: counts chain length without judging content.

### Mechanics
- Built entirely from `|derives-from` typed links (Phase 1)
- Any note can display its full ancestry: note → source → source's source → ... → primary
- **Received knowledge** (متلقّاة): chain traces to external source (book, paper, author) → cool blue glow in GraphMind
- **Discovered knowledge** (مُكتشَفة): chain originates with the user's own note → warm amber glow in GraphMind
- **Trust depth**: count of chain links. Direct primary source = trust depth 1. Fourth-hand summary = trust depth 4.
- The user never classifies notes as "received" or "discovered." The system detects structurally: if the root of the `derives-from` chain is a note with no further `derives-from` link and has an external URL or author property → received. Otherwise → discovered.

### V2.0 Visual Distinction (from paper)
- Received: cool color temperature (blue-tint ambient in GraphMind node glow)
- Discovered: warm color temperature (amber glow)
- Gradual, ambient — the user absorbs it without being told

### UI
- **NotePane sidebar panel**: "Provenance" tab showing ancestry chain as vertical tree
- Each ancestor: note name, trust depth badge, library-colored dot
- **GraphMind**: node glow hue shifts cool/warm based on origin type

### New Rust Command
```rust
#[tauri::command]
pub fn get_provenance_chain(note_path: String, all_links: Vec<NoteLink>, max_depth: usize) -> Result<ProvenanceChain, String>

pub struct ProvenanceChain {
    pub note_path: String,
    pub origin_type: String,   // "received" | "discovered" | "mixed"
    pub trust_depth: usize,    // 0 = primary source, higher = more steps removed
    pub ancestors: Vec<AncestorNode>,
}
pub struct AncestorNode {
    pub name: String,
    pub path: String,
    pub depth: usize,
    pub has_external_source: bool,
}
```

### Files to Change
| File | Change |
|---|---|
| `src-tauri/src/provenance.rs` (NEW) | `get_provenance_chain` command |
| `src-tauri/src/lib.rs` | Register command |
| `src/lib/components/ProvenancePanel.svelte` (NEW) | Ancestry chain UI |
| `src/lib/graph/graphEngine.ts` | Node glow cool/warm by origin type |
| `src/routes/+layout.svelte` | Wire ProvenancePanel to active note |

### Test Plan
1. Chain: Note A `→|derives-from→` Note B `→|derives-from→` External Source → trust depth = 2 ✓
2. Note with no `derives-from` links → trust depth = 0, origin = "discovered" ✓
3. External source note (has `url:` property) → chain terminates, origin = "received" ✓
4. GraphMind: received note → cool blue glow; discovered note → warm amber glow ✓
5. ProvenancePanel: shows ancestor chain as tree, clickable ✓
6. Circular `derives-from` chain → handled gracefully (max_depth cap) ✓
7. No typing lag when ProvenancePanel is open ✓

### GO/NO-GO
GO if tests 1–7 pass.

---

## Phase 6: Externalization Engine (محرك التجسيد) ✅ COMPLETE

**Commit**: `87d21d7` | **Date**: 2026-04-02
**Depends on**: Frontmatter system (existing), FocusPane (existing), Phase 3 (Maturity alignment)

### Goal
Progressive formalization pipeline: fleeting → literature → permanent → synthesis. FocusPane already IS the Fleeting stage. This phase makes the pipeline explicit via frontmatter + UI affordances.

### 4 Stages
| Stage | Arabic | What it is | Current App Hook |
|---|---|---|---|
| Fleeting | عابرة | Quick capture, no structure | FocusPane (already exists) |
| Literature | مرجعية | Rewritten from source, has `source:` property | NotePane + PropertyEditor |
| Permanent | دائمة | Atomic idea, linked to graph, one note = one idea | NotePane |
| Synthesis | تركيبية | Combines multiple permanent notes into new insight | NotePane + Expression Forge (Phase 10) |

### Frontmatter Property
Stage stored as: `stage: fleeting | literature | permanent | synthesis`
Default: no `stage:` property = unclassified (backward compatible).

### UI
- **File tree**: subtle stage icon next to note name (🌱 fleeting, 📖 literature, 🔗 permanent, ✨ synthesis)
- **NotePane header**: stage indicator + one-click promote button ("Promote to Literature →")
- **FocusPane**: "Promote to NotePane as Permanent" button on save
- Stage progression is one-way: fleeting → literature → permanent → synthesis (no demotion needed)
- Not mandatory. Notes without a stage are just notes.

### Alignment with Maturity (Phase 3)
Stage and Maturity are independent but correlated:
- Fleeting notes are usually Seeds
- Permanent notes that get many links become Evergreen
- Synthesis notes tend toward Canonical
The system never forces this alignment — it emerges naturally.

### Files to Change
| File | Change |
|---|---|
| `src/lib/components/FileTree.svelte` | Stage icon per note |
| `src/lib/components/NotePane.svelte` | Stage indicator + promote button in header |
| `src/lib/components/FocusPane.svelte` | "Promote to Permanent" on save |
| `src/routes/+layout.svelte` | Read `stage:` from frontmatter; pass to components |

### Test Plan
1. Add `stage: fleeting` to frontmatter → fleeting icon in file tree ✓
2. Click "Promote to Literature" in NotePane → frontmatter updates to `stage: literature` ✓
3. Literature note without `source:` property → gentle prompt to add source ✓
4. FocusPane → "Promote to Permanent" → new note created in NotePane with `stage: permanent` ✓
5. Note without `stage:` → no icon, no prompt (backward compatible) ✓
6. Stage icon renders without layout shift in file tree ✓

### GO/NO-GO
GO if tests 1–6 pass.

---

## Phase 7: Review Pulse (نبض المراجعة)

**Status**: 🔲 Not started
**Depends on**: Phase 2 (Strata for priority weighting), Phase 3 (Maturity/decay detection)

### Goal
Spaced resurfacing and staleness monitoring. Not flashcards — these are knowledge revisit prompts that ask: "Still relevant? Link it? Archive it?"

### 3 Modes
1. **Spaced Resurfacing**: Notes never revisited, queued at expanding intervals (1 day → 3 → 7 → 14 → 30). Priority weighted by Strata (higher strata = reviewed sooner).
2. **Staleness Scan**: Evergreen/Canonical notes untouched while their tag-domain has active new notes. Sourced from Phase 3 decay detection.
3. **Mental Model Checkpoints**: Notes tagged `#assumption` or `#model` surface periodically (every 30 days) with: "Do you still hold this view? Has anything changed?"

### UI
- **Command palette command**: "Review due notes" → opens Review Pulse panel
- **Gentle badge** on right sidebar tab when notes are due (count only, no notification)
- Each review item: note name, last visited, stratum level, one-click open
- Actions per item: ✓ Reviewed | 🔗 Link it | 🗄️ Archive | 👁️ Snooze 7 days

### Storage
Review schedule stored in `.constellation/review-pulse.json` per library:
```json
{
  "last_reviewed": { "note-path.md": "2026-03-01" },
  "snoozed": { "note-path.md": "2026-04-07" }
}
```
Never written to .md files.

### Files to Change
| File | Change |
|---|---|
| `src-tauri/src/review.rs` (NEW) | Due-note computation, schedule R/W |
| `src-tauri/src/lib.rs` | Register commands |
| `src/lib/components/ReviewPulsePanel.svelte` (NEW) | Review queue UI |
| `src/routes/+layout.svelte` | Badge count; wire panel |

### Test Plan
1. Note never opened → appears in Spaced Resurfacing queue after 1 day ✓
2. Snooze note → disappears for 7 days ✓
3. Mark as "Reviewed" → removed from queue; interval doubles ✓
4. Note tagged `#assumption` → appears in Mental Model Checkpoints after 30 days ✓
5. Staleness Scan: Evergreen note + domain has new notes → appears in staleness list ✓
6. Higher-strata notes prioritized in queue ✓
7. Queue renders without lag ✓

### GO/NO-GO
GO if tests 1–7 pass.

---

## Phase 8: Trails (المسارات) ✅ COMPLETE

**Commit**: `96d7f3e` | **Date**: 2026-04-03
**Depends on**: GraphMind (existing), Wikilinks (existing)

### Goal
Named, ordered sequences of notes. First-class objects in the knowledge graph. A Trail tells a story, traces an argument, or records a research journey.

### Data Format
Trail = `.trail.md` file in the library root (or `trails/` folder):
```markdown
---
trail: true
title: "Path to Understanding Causality"
description: "From basic observations to causal theory"
---
[[Observation Note 1]]
[[Observation Note 2]]
[[Causal Pattern Note]]
[[Principle of Causality]]
[[Theory of Causal Systems]]
```

### UI
- **Sequential navigation**: previous/next buttons in NotePane when inside a trail
- **Trail picker**: command palette → "Open Trail" → lists all `.trail.md` files
- **GraphMind overlay**: trail rendered as a colored path over the graph (distinct from regular links)
- **Playback mode**: full-screen note-by-note presentation (like a slideshow, but with branch-and-return)
- **Trail indicator**: NotePane header shows "Trail: [name] — Note 3 of 7"

### Relationship with Expression Forge (Phase 10)
A Trail can be exported as the structural backbone of a written article via Expression Forge.

### Files to Change
| File | Change |
|---|---|
| `src-tauri/src/trails.rs` (NEW) | Trail CRUD: list, read, create, update |
| `src-tauri/src/lib.rs` | Register commands |
| `src/lib/components/TrailNavigator.svelte` (NEW) | Prev/next nav + trail indicator |
| `src/lib/components/NotePane.svelte` | Show TrailNavigator when active trail |
| `src/lib/graph/graphEngine.ts` | Trail path overlay rendering |
| `src/lib/graph/GraphMindView.svelte` | Expose trail toggle |

### Test Plan
1. Create `.trail.md` with 5 note links → trail appears in trail picker ✓
2. Open trail → NotePane shows "Trail: X — Note 1 of 5" with prev/next ✓
3. Clicking "Next" opens next note in trail ✓
4. Last note: "Next" wraps or disables ✓
5. GraphMind: trail notes connected by colored path overlay ✓
6. Playback mode: fullscreen, note-by-note ✓
7. Trails work with RTL notes ✓

### GO/NO-GO
GO if tests 1–7 pass.

---

## Phase 9: Multi-Lens Views (العدسات المتعددة) ✅ COMPLETE

**Commit**: `4b72c0c` | **Date**: 2026-04-03
**Depends on**: Tags (existing), Dataview/Bases (existing)

### Goal
Same library content viewed through multiple independent classification schemes. No note duplication. No note movement. Switch lenses from sidebar.

### Mechanics
- Each lens = a named tag-hierarchy or metadata-query (extends existing Bases/Dataview system)
- Lenses defined in `.constellation/lenses.json` per universe
- Switch lenses from sidebar toggle → file tree reorganizes
- Multilingual lens: RTL concept ↔ LTR concept pairs side by side

### Lens Definition (JSON)
```json
{
  "lenses": [
    {
      "name": "By Topic",
      "type": "tag-hierarchy",
      "root_tags": ["#philosophy", "#science", "#practice"]
    },
    {
      "name": "By Stage",
      "type": "property-query",
      "property": "stage",
      "values": ["fleeting", "literature", "permanent", "synthesis"]
    },
    {
      "name": "By Certainty",
      "type": "property-query",
      "property": "certainty",
      "values": ["exploratory", "provisional", "established"]
    }
  ]
}
```

### UI
- Sidebar toggle: lens switcher (dropdown or tabs)
- File tree reorganizes to reflect active lens
- Notes can appear in multiple lens-groups (no duplication)
- The current folder structure remains the "default lens"

### Files to Change
| File | Change |
|---|---|
| `src-tauri/src/lenses.rs` (NEW) | Lens CRUD; apply lens to file tree |
| `src-tauri/src/lib.rs` | Register commands |
| `src/lib/components/LensSwitcher.svelte` (NEW) | Lens picker UI |
| `src/lib/components/FileTree.svelte` | Lens-aware rendering mode |
| `src/routes/+layout.svelte` | Active lens state |

### Test Plan
1. Create "By Stage" lens → file tree reorganizes into stage groups ✓
2. Note without `stage:` → appears in "Unclassified" group ✓
3. Switch back to default lens → original folder structure restored ✓
4. Note appears in both "Philosophy" and "Science" lens groups ✓
5. Create lens for multilingual pairs → Arabic and English concept shown side by side ✓
6. Lens persists across app restarts ✓

### GO/NO-GO
GO if tests 1–6 pass.

---

## Phase 10: Expression Forge (مصنع التعبير)

**Status**: 🔲 Not started
**Depends on**: Phase 2 (Strata for suggestions), Phase 8 (Trails as backbone), Phase 6 (Synthesis stage notes)

### Goal
Synthesis workspace where the user composes output by assembling notes. The point where the knowledge cycle completes: capture → cognition → **expression**.

### UI Layout
Split-pane workspace:
- **Left**: note browser (filtered by Strata, filtered by community lens)
- **Right**: composition canvas — drag notes in as blocks, rearrange, write connective text
- **Structural suggestions**: graph-proximity notes suggested in left panel (pure topology, zero AI)
- **Trail integration**: existing trails importable as starting backbone

### Key Behaviors
- Dragging a note into the canvas embeds its content as a collapsible block
- User writes transitions/annotations between blocks
- Export: Markdown file combining selected notes + user text
- Socratic Challenger (Phase 15) hooks here when AI is enabled

### Files to Change
| File | Change |
|---|---|
| `src/lib/components/ExpressionForge.svelte` (NEW) | Full synthesis workspace |
| `src/routes/+layout.svelte` | Forge as a new workspace view mode |

### Test Plan
1. Open Expression Forge → left panel shows notes filtered by strata ✓
2. Drag note into canvas → note content appears as collapsible block ✓
3. Rearrange blocks by drag ✓
4. Graph-proximity suggestions update as notes are added to canvas ✓
5. Import a Trail as backbone → trail notes pre-loaded in canvas order ✓
6. Export → produces valid Markdown file ✓
7. RTL notes render correctly in canvas ✓

### GO/NO-GO
GO if tests 1–7 pass.

---

## Phase 11: Sense-Making Canvas (لوحة الإدراك)

**Status**: 🔲 Not started
**Depends on**: Frontmatter system (existing), NotePane (existing)
**Note**: Most engineering-intensive phase. Scoped for last in Layer 1.

### Goal
A pre-structural space for capturing ambiguous, half-formed, contradictory signals BEFORE they become structured notes. This is where Constellation differs most from competitors: it explicitly supports the pre-knowledge phase.

### UI
- Infinite spatial canvas (pan + zoom)
- **Items**: text snippets, wikilinks, images, free-form text
- **Four optional Cynefin quadrants**: Clear, Complicated, Complex, Chaotic
- Drag items between quadrants as understanding evolves
- **Promote**: one-click promotes a canvas item to a proper note in NotePane, carrying canvas context as frontmatter metadata

### Data Format
Canvas stored as `.canvas` file (JSON):
```json
{
  "items": [
    { "id": "1", "type": "text", "x": 100, "y": 200, "content": "Half-formed idea about...", "quadrant": "complex" },
    { "id": "2", "type": "link", "x": 300, "y": 150, "target": "[[Note Name]]", "quadrant": null }
  ]
}
```

### Files to Change
| File | Change |
|---|---|
| `src/lib/components/SenseMakingCanvas.svelte` (NEW) | Infinite canvas with Cynefin quadrants |
| `src-tauri/src/canvas.rs` (NEW) | Canvas CRUD (`.canvas` files) |
| `src-tauri/src/lib.rs` | Register commands |
| `src/routes/+layout.svelte` | Canvas as a tab type alongside notes |

### Test Plan
1. Create canvas → blank infinite canvas renders ✓
2. Add text item → draggable snippet ✓
3. Drag item to "Complex" quadrant → item snaps to quadrant ✓
4. Promote item to note → new note opens with canvas context in frontmatter ✓
5. Canvas persists across app restarts ✓
6. Pan/zoom performance: 50+ items, smooth 60fps ✓
7. RTL text items render correctly ✓

### GO/NO-GO
GO if tests 1–7 pass.

---

## LAYER 2 — AI DISCOVERY LAYER

*Layer 2 activates after Layer 1 establishes a rich structural foundation.*
*Governing principle: AI does not replace cognition. It extends perception.*

**Local-first policy decision**: Layer 2 features use the existing `ai_send_message` Tauri command. For on-device privacy, users may configure a local LLM via app settings. Cloud AI is opt-in only, never default.

---

## Phase 12: Hidden Pattern Discovery

**Status**: 🔲 Not started
**Depends on**: Phase 1 (Typed Links), Phase 2 (Strata), GraphMind semantic engine (existing)

### Goal
Find thematic patterns the user hasn't explicitly linked. Surfaces as "ghost links" — dashed translucent lines in GraphMind suggesting: "These notes may be related. Explore?"

### Mechanics
- Uses existing `semanticEngine.ts` (embedding-based similarity, already in codebase)
- Ghost links appear for note pairs with similarity above threshold but no existing wikilink
- Ghost links are distinct from both regular links (solid) and semantic Phase 2 links (indigo dashed)
- Ghost links: thinner, lighter, with `?` label on hover

### Files to Change
| File | Change |
|---|---|
| `src/lib/graph/semanticEngine.ts` | Filter out already-linked pairs; expose ghost link API |
| `src/lib/graph/graphEngine.ts` | Ghost link rendering (lighter, thinner, `?` label) |
| `src/lib/graph/GraphMindView.svelte` | Toggle for ghost link visibility |

### Test Plan
1. Two thematically related but unlinked notes → ghost link appears in GraphMind ✓
2. Ghost link distinguishable from regular links and semantic links ✓
3. Hover ghost link → shows similarity score + "Explore?" prompt ✓
4. Click "Create link" on ghost link → inserts wikilink in source note ✓
5. Ghost links toggle-able in GraphMind settings ✓

---

## Phase 13: Blind Spot Detection

**Status**: 🔲 Not started
**Depends on**: Phase 2 (Strata), Phase 5 (Provenance), AI integration (existing)

### Goal
AI examines the knowledge graph and identifies domains where the user has notes but significant conceptual gaps.

### Mechanics
- AI receives: tag clusters, note titles, stratum distribution (no note content for privacy)
- AI identifies: "Your systems engineering knowledge has no coverage of X. Explore?"
- Presented as a gentle "Knowledge Gaps" section in the Tension Panel (Phase 4) or separate tab
- User can dismiss or investigate each gap suggestion

### Privacy
- Only note titles + graph structure sent to AI (not content, unless user opts in)
- Local LLM mode: content can be sent safely (stays on device)

---

## Phase 14: Cross-Domain Insight Generation

**Status**: 🔲 Not started
**Depends on**: Phase 9 (Multi-Lens Views / Community Lenses), AI integration (existing)

### Goal
AI reads notes from different community lenses and proposes cross-domain analogies. Operates in the user's Zone of Proximal Development (Vygotsky): scaffolding connections just beyond current reach.

### UI
- "Insights" panel (separate from Tension Detector)
- Each insight: "Your notes on [Lens A: topic] share structural patterns with your notes on [Lens B: topic]. Explore connection?"
- Accept → creates a cross-domain note with both lens tags + `|generalizes` link suggestion

---

## Phase 15: Socratic Challenger

**Status**: 🔲 Not started
**Depends on**: Phase 10 (Expression Forge), AI integration (existing)

### Goal
When user writes synthesis notes or works in Expression Forge, AI reads the draft and asks challenging questions. Never provides answers — deepens thinking.

### Mechanics
- Triggered manually (button: "Challenge this") or automatically on Synthesis-stage notes (opt-in)
- AI reads draft + connected notes (via typed links)
- Generates 2–3 Socratic questions:
  - "You state X, but your note Y suggests the opposite. How do you reconcile this?"
  - "What would need to be true for this claim to fail?"
  - "You haven't addressed [gap]. Is that intentional?"
- Questions appear in a non-intrusive sidebar panel, not inline

---

## Phase 16: Worldview Synthesis

**Status**: 🔲 Not started
**Depends on**: All Layer 1 phases complete, AI integration (existing)

### Goal
AI reads the entire knowledge graph and generates a "Worldview Map": the user's deepest beliefs, frameworks, and organizing principles — extracted from patterns in notes.

### Output
- Visual: a small graph of the user's top-level organizing concepts (Paradigm/Worldview level Strata)
- Text: narrative summary of intellectual architecture — "Your knowledge appears organized around these core frameworks: [A], [B], [C]"
- Reveals: paradigms the user operates within, assumptions taken for granted, questions never asked

### Privacy
This is the most sensitive Phase 2 feature. Requires explicit user consent. Content is sent to AI.
Local LLM strongly recommended.

---

## 4. Progress Table

| Phase | Name | Status | Commit | Date | Test Results |
|---|---|---|---|---|---|
| 1 | Typed Links | ✅ GO | `d7edc6d` | 2026-03-30 | 18/18 tests passed |
| 2 | Knowledge Strata | 🔲 Not started | — | — | — |
| 3 | Maturity Lifecycle | 🔲 Not started | — | — | — |
| 4 | Tension Detector | 🔲 Not started | — | — | — |
| 5 | Provenance Chain | 🔲 Not started | — | — | — |
| 6 | Externalization Engine | 🔲 Not started | — | — | — |
| 7 | Review Pulse | 🔲 Not started | — | — | — |
| 8 | Trails | 🔲 Not started | — | — | — |
| 9 | Multi-Lens Views | 🔲 Not started | — | — | — |
| 10 | Expression Forge | 🔲 Not started | — | — | — |
| 11 | Sense-Making Canvas | 🔲 Not started | — | — | — |
| 12 | Hidden Pattern Discovery | 🔲 Not started | — | — | — |
| 13 | Blind Spot Detection | 🔲 Not started | — | — | — |
| 14 | Cross-Domain Insights | 🔲 Not started | — | — | — |
| 15 | Socratic Challenger | 🔲 Not started | — | — | — |
| 16 | Worldview Synthesis | 🔲 Not started | — | — | — |

---

## 5. Session Pickup Protocol

If a session clears, the next session should:
1. `git pull origin main`
2. `git log --oneline -5` — find where we left off
3. Read this file — find the last ✅ phase
4. Read the corresponding session log: `lab/reports/SESSION-LOG-YYYY-MM-DD.md`
5. If last phase was user-tested and passed → proceed to next phase
6. If last phase was committed but not user-tested → conduct the test first
