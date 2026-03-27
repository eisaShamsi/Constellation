# BLOCKING ISSUE #001: App-Wide Lag from +layout.svelte

## Status: OPEN
## Severity: CRITICAL
## Date: 2026-03-27

---

## Symptoms
- Title selection takes ~2s
- Enter key in title: 3s+ delay before cursor moves
- 10s+ freezes with no response
- Opening a note takes noticeable time to load content
- General UI unresponsiveness across all interactions

## Root Cause: `src/routes/+layout.svelte` (3873 lines)

The layout file is a monolithic component with **110+ reactive nodes** that create a cascading reactivity storm on every interaction.

### Critical Findings

| Issue | Location | Impact |
|---|---|---|
| `parseFrontmatter()` as `$derived` | Line 619 | Runs on EVERY keystroke — parses full YAML |
| `extractHeadings()` as `$derived` | Line 622 | Runs on EVERY keystroke — scans full document |
| `detectDir()` as `$derived` | Line 623 | Runs on EVERY keystroke — regex scan |
| `getBacklinks()` as `$derived` | Line 626-628 | Runs on EVERY keystroke — linear search all notes |
| `getOutgoingLinks()` as `$derived` | Line 646-651 | Runs on EVERY keystroke — linear search + map |
| Calendar scan `$effect` | Line 737-770 | `Promise.all()` on ALL libraries, nested loops |
| Font `$effect` DOM mutation | Line 808-906 | Heavy CSS string building + DOM writes |
| `idleTimer` not cleaned | Line 966 | Memory leak — never cleared in onDestroy |

### Scale of the Problem

- **77** `$state` variables
- **17** `$effect` blocks
- **19** `$derived` / `$derived.by` blocks
- **150+** reactive template references
- **3** `openTabs.update()` calls that cascade through all 17 effects

### The Chain Reaction

Every keystroke in the editor triggers:
```
keystroke → onchange → sidebarTab changes
  → parseFrontmatter() (full YAML parse)
  → extractHeadings() (full document scan)
  → detectDir() (regex scan)
  → getBacklinks() (all notes search)
  → getOutgoingLinks() (all notes search)
  → activeNoteTags (regex + Set)
  → template re-evaluates 150+ references
```

This is ~6 heavy computations on EVERY keystroke, with ZERO debouncing.

## Proposed Fix

### Phase A: Debounce the derived chain (immediate relief)
Convert the expensive `$derived` blocks (lines 619-676) into debounced `$effect` blocks that only recompute after a 500ms typing pause. The editor doesn't need real-time heading extraction or backlink updates while the user is typing.

### Phase B: Extract sidebar into its own component (structural fix)
The sidebar computations (headings, backlinks, outgoing links, tags) should live in a separate `<Sidebar>` component with its own reactive scope. This isolates the 3873-line layout from sidebar recomputation.

### Phase C: Lazy sidebar panels (optimization)
Only compute sidebar data for the currently visible panel. If the backlinks panel is collapsed, don't run `getBacklinks()`.

## Blocked Work
- eNotePane Phase 0 testing (user reported lag during skeleton test)
- All subsequent eNotePane phases

## Resolution Criteria
1. User can interact with the title instantly (< 100ms response)
2. User can type in the editor with zero perceptible lag
3. Tab switching feels instant
4. Environment Auditor (EA) passes
