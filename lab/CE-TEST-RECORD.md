# Cognitive Engine — Test Record

Comprehensive history of all CE phases: implementation, tests conducted, and results.

---

## Phase 1: Typed Links (الروابط الدلالية)

**Commit:** `d7edc6d` | **Date:** 2026-03-30 | **Status:** ✅ PASS

### Test Plan (18 tests — ALL PASSED)
| # | Test | Result |
|---|------|--------|
| 1 | `[[Philosophy of Knowledge\|supports]]` saves/reopens | ✅ |
| 2 | `[[Tension\|contradicts]]` → BacklinksPanel red badge | ✅ |
| 3 | `[[Source Book\|derives-from]]` → GraphMind gold dotted | ✅ |
| 4 | `[[plain link]]` (no pipe) → normal behavior | ✅ |
| 5 | `[[note\|foobar]]` (unknown type) → treated as associative | ✅ |
| 6 | `[[` → type name → `\|` → completion appears | ✅ |
| 7 | Select `contradicts` → `]]` auto-closes | ✅ |
| 8 | Escape during type completion → no type inserted | ✅ |
| 9 | GraphMind: `contradicts` = red bidirectional | ✅ |
| 10 | GraphMind: `causes` = orange thicker arrow | ✅ |
| 11 | GraphMind: `derives-from` = gold dotted | ✅ |
| 12 | GraphMind: untyped = gray, no arrowhead | ✅ |
| 13 | BacklinksPanel: inbound `supports` → blue badge | ✅ |
| 14 | BacklinksPanel: mixed types → each badge correct | ✅ |
| 15 | `[[note\|]]` (empty type) → associative | ✅ |
| 16 | `[[note name with spaces\|causes]]` → parses correctly | ✅ |
| 17 | `[[note#heading\|derives-from]]` → target=note, type=derives-from | ✅ |
| 18 | Existing untyped links → zero regression | ✅ |

---

## Phase 2: Knowledge Strata (طبقات المعرفة)

**Commit:** `0f6d4bf` | **Date:** 2026-04-02 | **Status:** ✅ PASS

### Test Plan (7 tests)
| # | Test | Result |
|---|------|--------|
| 1 | Library with 5 unlinked notes → all Level 1–2, same-size nodes | ✅ |
| 2 | 20+ notes → strata activate in GraphMind | ✅ |
| 3 | Note with 5+ outgoing `generalizes` → stratum ≥ 5 | ✅ |
| 4 | Most-linked note has largest node | ✅ |
| 5 | Open 5000-word note + scroll → no lag | ✅ |
| 6 | Switch libraries → strata recompute | ✅ |
| 7 | Rust command returns correct struct | ✅ |

### Post-test fix
- Stratum halo changed to complementary color (180° HSL rotation) for visibility

---

## Phase 3: Maturity Lifecycle (دورة النضج)

**Commit:** `5cf4283` | **Date:** 2026-04-02 | **Status:** ✅ PASS

### Test Plan (7 tests)
| # | Test | Result |
|---|------|--------|
| 1 | New note → Seed (no border) in file tree | ✅ |
| 2 | Note with 2+ inbound → Sapling (light green) | ✅ |
| 3 | Note with 4+ inbound, 7+ days old → Evergreen (rich green) | ✅ |
| 4 | Note with 10+ inbound, 30+ days untouched → Canonical (gold) | ✅ |
| 5 | Evergreen untouched 90 days → wilting (dimmed) | — not yet testable |
| 6 | 100-note library → file tree renders without lag | ✅ |
| 7 | Switch libraries → maturity updates | ✅ |

### Visual locations confirmed
- File tree: colored left border ✅
- Star View: maturity ring ✅
- Tab bar: colored dot ✅

---

## Phase 4: Tension Detector (كاشف التناقضات)

**Commit:** `88f8ddb` | **Date:** 2026-04-02 | **Status:** ⏳ BUILT — pending large library test

### Test Plan (8 tests)
| # | Test | Result |
|---|------|--------|
| 1 | Library <50 linked notes → "not enough links" message | ✅ |
| 2 | Library 50+ linked → panel activates with 4 sections | ⏳ pending |
| 3 | Notes linked with `\|contradicts` → Contradictions list | ⏳ pending |
| 4 | Isolated note → Orphans list | ⏳ pending |
| 5 | Tag-clusters with no cross-links → Structural Gaps | ⏳ pending |
| 6 | Note with 6+ inbound, 1 source → Single Point of Failure | ⏳ pending |
| 7 | Click tension item → opens note | ⏳ pending |
| 8 | No typing lag with panel open | ⏳ pending |

### Notes
- User building a larger test library to reach 50+ linked notes threshold
- Test 1 passed with "5 / 50 linked notes" message

---

## Phase 5: Provenance Chain (سلسلة الإسناد)

**Status:** 🔲 Planning

### Test Plan (7 tests — from CE spec)
| # | Test | Result |
|---|------|--------|
| 1 | Chain: A →derives-from→ B →derives-from→ Source → depth=2 | ⏳ |
| 2 | Note with no derives-from → depth=0, origin=discovered | ⏳ |
| 3 | External source (has url: property) → origin=received | ⏳ |
| 4 | GraphMind: received → blue glow; discovered → amber glow | ⏳ |
| 5 | ProvenancePanel: ancestor chain as tree, clickable | ⏳ |
| 6 | Circular derives-from → handled (max_depth cap) | ⏳ |
| 7 | No typing lag with panel open | ⏳ |

---

## Phases 6–16: Not yet started

| Phase | Name | Status |
|-------|------|--------|
| 6 | Externalization Engine | 🔲 |
| 7 | Review Pulse | 🔲 |
| 8 | Trails | 🔲 |
| 9 | Multi-Lens Views | 🔲 |
| 10 | Expression Forge | 🔲 |
| 11 | Sense-Making Canvas | 🔲 |
| 12 | Hidden Pattern Discovery | 🔲 |
| 13 | Blind Spot Detection | 🔲 |
| 14 | Cross-Domain Insights | 🔲 |
| 15 | Socratic Challenger | 🔲 |
| 16 | Worldview Synthesis | 🔲 |

---

## Non-CE Tests

### NotePane Regression (R1–R13)
**Date:** 2026-03-31 → 2026-04-02 | **Status:** ✅ ALL 86 PASS
- Full results in `lab/NOTEPANE-REGRESSION-2026-03-31.md`

### Toolbar Enhancement
**Date:** 2026-04-02 | **Status:** ✅ PASS
- Underline, Subscript, Superscript, Alignment, Clear Formatting, Find/Replace
- RTL-aware direction flip, table toolbar RTL
- Toggle on/off button

### Universe Portability
**Date:** 2026-04-02 | **Status:** ✅ PASS
- Flat Obsidian-style structure (no nesting)
- Auto-migration from nested to flat
- Portable: move to USB/new drive → paths auto-fix on activation

### FocusPane Multilingual
**Date:** 2026-04-01 | **Status:** ✅ PASS
- bidiPlugin per-line direction detection
- Plain hairline cursor
- Language-First by Design principle

### Typewriter Font Theme
**Date:** 2026-04-01 | **Status:** ✅ PASS
- 8 fonts bundled (Special Elite, Courier Prime, Miriam Libre, PT Mono, etc.)
- Settings toggle: Default / Typewriter
