# CE Phase 1 — Typed Links (الروابط الدلالية)
**Cognitive Engine — Layer 1, Tool 3**
**Status**: 🔲 Not started

---

## Goal

Extend the existing `[[wikilink]]` syntax to carry semantic meaning via a pipe character. This is the keystone feature — every downstream Cognitive Engine tool depends on it.

```
[[note|supports]]      — evidential relationship
[[note|contradicts]]   — tension (triggers Tension Detector)
[[note|causes]]        — causal (directional arrow in GraphMind)
[[note|exemplifies]]   — instance-of relationship
[[note|generalizes]]   — abstraction relationship
[[note|derives-from]]  — provenance (feeds Provenance Chain)
[[note|part-of]]       — compositional hierarchy
```

Untyped links default to `associative`. Power users type links; beginners never need to.

---

## Epistemological Basis

From the Cognitive Engine paper: *"Connectivism (Siemens) holds that the connections between nodes constitute knowledge. These principles demand that links carry meaning, not just association."*

The principle of non-contradiction, causality, identity, and hierarchy are not optional frameworks — they are the logic of thought itself.

---

## Scope

### In Scope
- Rust: parse `[[note|link-type]]` in `scan_library_links` → populate `link_type` field
- NotePane: autocomplete `|` after note name inside `[[...]]` suggests link types
- GraphMind: render each link type with distinct color + arrowhead + dash pattern
- BacklinksPanel: show link type badge next to each backlink
- livePreview.ts: render typed wikilinks with link-type color hint in editor

### Out of Scope
- No AI in this phase
- No validation that link types are "correct" — user decides
- No UI for managing link types globally (future: Multi-Lens Views)

---

## Link Type Catalogue

| Type | Symbol | Meaning | GraphMind Color | Arrow |
|---|---|---|---|---|
| `associative` | (default) | General connection | Gray `#888` | None |
| `supports` | 🔵 | Evidence for a claim | Blue `#4A9EFF` | →  |
| `contradicts` | 🔴 | Tension, opposition | Red `#FF4A4A` | ↔ (bidirectional) |
| `causes` | 🟠 | Causal relationship | Orange `#FF8C42` | → (thicker) |
| `exemplifies` | 🟢 | Instance-of | Green `#4AFF88` | → (dashed) |
| `generalizes` | 🟣 | Abstraction | Purple `#A44AFF` | → (upward) |
| `derives-from` | 🟡 | Provenance/source | Gold `#FFD700` | → (dotted) |
| `part-of` | ⚪ | Compositional | Light gray `#AAAAAA` | → (thin) |

---

## Architecture

### Rust — `src-tauri/src/links.rs`

The existing `NoteLink` struct already has a `link_type` field. The parser needs to extract it:

**Current**: `[[note name]]` → `NoteLink { target: "note name", link_type: "associative" }`
**After**: `[[note name|causes]]` → `NoteLink { target: "note name", link_type: "causes" }`

Parser logic (in `scan_library_links`):
1. When a `[[...]]` is found, split on `|`
2. First part = target note name
3. Second part (if present) = link type
4. Validate against known types; unknown types default to `associative`
5. Fragment handling: `[[note#heading|causes]]` → target `note`, fragment `heading`, type `causes`

### TypeScript — `src/lib/editor/completions.ts`

Add typed link completion triggered by `|` when cursor is inside `[[...|`:
1. Detect cursor is inside `[[...` and user typed `|`
2. Offer completion list: supports, contradicts, causes, exemplifies, generalizes, derives-from, part-of
3. Each option shows description + color swatch
4. Pressing Tab/Enter inserts link type and closes `]]`

### GraphMind — `src/lib/components/GraphMindView.svelte` + `graphEngine.ts`

GraphMind already supports semantic links with `linkType` on the `StarLink` interface. The engine already has rendering code. Updates needed:
1. Map the 7 typed link types to the color/style table above in `graphEngine.ts`
2. `contradicts` links: render as bidirectional (both arrowheads)
3. `causes` links: thicker stroke weight
4. `derives-from` links: dotted line
5. `exemplifies` links: dashed line
6. Legend panel: show link type color guide (optional, togglable)

### BacklinksPanel — `src/lib/components/BacklinksPanel.svelte`

1. Show link type as a colored badge next to each backlink entry
2. Badge uses the same color from the Link Type Catalogue
3. Group backlinks by link type (optional secondary view)

### livePreview.ts — `src/lib/editor/livePreview.ts`

When rendering `[[note|causes]]` in the editor:
1. Strip the `|causes` from the displayed text (show only note name)
2. Apply a colored underline matching the link type color
3. This is a visual-only change — file content unchanged

---

## Test Plan

### Parse Tests (manual)
1. Type `[[Philosophy of Knowledge|supports]]` in a note → save → reopen → link preserved ✓
2. Type `[[Tension|contradicts]]` → open BacklinksPanel on "Tension" → shows red "contradicts" badge ✓
3. Type `[[Source Book|derives-from]]` → GraphMind → gold dotted line from this note to "Source Book" ✓
4. Type `[[plain link]]` (no pipe) → behaves exactly as before ✓
5. Type `[[unknown|foobar]]` → treated as `associative` silently ✓

### Autocomplete Tests
6. Inside `[[` → type a note name → type `|` → completion list appears ✓
7. Select `contradicts` → `]]` auto-closes → result: `[[note|contradicts]]` ✓
8. Press Escape during type completion → no link type inserted, `|` remains ✓

### GraphMind Tests
9. Open GraphMind → `contradicts` link renders red bidirectional ✓
10. `causes` link renders orange thicker arrow ✓
11. `derives-from` link renders gold dotted ✓
12. Untyped link renders gray, no arrowhead ✓

### BacklinksPanel Tests
13. Open a note that has inbound `supports` link → badge shows blue "supports" ✓
14. Open a note with mixed link types → each badge color matches type ✓

### Edge Cases
15. `[[note|]]` (empty type) → treated as `associative` ✓
16. `[[note name with spaces|causes]]` → parses correctly ✓
17. `[[note#heading|derives-from]]` → target=note, fragment=heading, type=derives-from ✓
18. Existing notes with no typed links → zero regression, all open and save normally ✓

---

## Passing Criteria (GO/NO-GO)

**GO** if tests 1–18 all pass.
**NO-GO** if:
- Any existing untyped wikilink breaks (regression)
- Typing lag introduced in NotePane
- GraphMind crashes or renders incorrectly

---

## Files Changed

| File | Change Type |
|---|---|
| `src-tauri/src/links.rs` (or equivalent) | Modify parser |
| `src/lib/editor/completions.ts` | Add typed link completions |
| `src/lib/editor/livePreview.ts` | Color-coded underline for typed links |
| `src/lib/components/GraphMindView.svelte` | Pass link types to engine |
| `src/lib/components/graphEngine.ts` | Render typed links with color/style |
| `src/lib/components/BacklinksPanel.svelte` | Show link type badges |

---

## Session Log Target

After passing: update `lab/reports/SESSION-LOG-2026-03-30.md` with phase results.
Update `docs/cognitive-engine-roadmap.md` progress table: Phase 1 → ✅ Done.
