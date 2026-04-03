# Constellation — Cognitive Engine Roadmap
**Quick reference index. Full specification: `docs/CE-spec.md`**

---

## Build Order

```
LAYER 1 — Structural Cognition (zero AI dependency)

  Phase 1:  Typed Links         ✅  [[note|link-type]] — keystone, unlocks all below
  Phase 2:  Knowledge Strata    ✅  auto-classify notes by abstraction level (8 levels)
  Phase 3:  Maturity Lifecycle  ✅  Seed → Sapling → Evergreen → Canonical
  Phase 4:  Tension Detector    🔲  contradictions, orphans, structural gaps
  Phase 5:  Provenance Chain    🔲  isnad-inspired source lineage
  Phase 6:  Externalization     🔲  fleeting → literature → permanent → synthesis
  Phase 7:  Review Pulse        🔲  spaced resurfacing + staleness scan
  Phase 8:  Trails              🔲  named ordered note sequences
  Phase 9:  Multi-Lens Views    🔲  multiple classification schemes, same content
  Phase 10: Expression Forge    🔲  synthesis workspace, output composition
  Phase 11: Sense-Making Canvas 🔲  pre-structural space (heaviest engineering)

LAYER 2 — AI Discovery (reads Layer 1 structures)

  Phase 12: Hidden Pattern Discovery   🔲  ghost links in GraphMind
  Phase 13: Blind Spot Detection       🔲  domain gaps via AI
  Phase 14: Cross-Domain Insights      🔲  cross-lens analogies
  Phase 15: Socratic Challenger        🔲  questions, never answers
  Phase 16: Worldview Synthesis        🔲  user's intellectual architecture map
```

---

## Progress Table

| Phase | Name | Status | Commit | Date |
|---|---|---|---|---|
| 1 | Typed Links | ✅ GO — user tested | `d7edc6d` | 2026-03-30 |
| 2 | Knowledge Strata | ✅ GO — user tested | `0f6d4bf` | 2026-04-02 |
| 3 | Maturity Lifecycle | ✅ GO — user tested | `5cf4283` | 2026-04-02 |
| 4 | Tension Detector | ⏳ Built — pending large library test | `88f8ddb` | 2026-04-02 |
| 5 | Provenance Chain | ✅ GO — user tested | `2de0c15` | 2026-04-02 |
| 6 | Externalization Engine | ✅ GO — user tested | `87d21d7` | 2026-04-02 |
| 7 | Review Pulse | ✅ GO — user tested | `b2bbed0` | 2026-04-02 |
| 8 | Trails | ✅ GO — user tested | `96d7f3e` | 2026-04-03 |
| 6 | Externalization Engine | 🔲 Not started | — | — |
| 7 | Review Pulse | 🔲 Not started | — | — |
| 8 | Trails | 🔲 Not started | — | — |
| 9 | Multi-Lens Views | 🔲 Not started | — | — |
| 10 | Expression Forge | 🔲 Not started | — | — |
| 11 | Sense-Making Canvas | 🔲 Not started | — | — |
| 12 | Hidden Pattern Discovery | 🔲 Not started | — | — |
| 13 | Blind Spot Detection | 🔲 Not started | — | — |
| 14 | Cross-Domain Insights | 🔲 Not started | — | — |
| 15 | Socratic Challenger | 🔲 Not started | — | — |
| 16 | Worldview Synthesis | 🔲 Not started | — | — |

---

## Key Dependencies

| Phase | Requires |
|---|---|
| 2 Knowledge Strata | Phase 1 (link types enrich signals) |
| 3 Maturity Lifecycle | Wikilinks (existing), file metadata (existing) |
| 4 Tension Detector | Phase 1 (`contradicts` type), Phase 3 (orphan severity) |
| 5 Provenance Chain | Phase 1 (`derives-from` type) |
| 6 Externalization | Frontmatter + FocusPane (both existing) |
| 7 Review Pulse | Phase 2 (priority), Phase 3 (staleness) |
| 8 Trails | GraphMind + Wikilinks (both existing) |
| 9 Multi-Lens Views | Tags + Dataview (both existing) |
| 10 Expression Forge | Phase 2, Phase 6, Phase 8 |
| 11 Sense-Making Canvas | NotePane + Frontmatter (both existing) |
| 12–16 (Layer 2) | All Layer 1 phases, AI integration (existing) |

---

## References

- **Full spec**: `docs/CE-spec.md` — architecture, test plans, GO/NO-GO criteria for all 16 phases
- **Source paper**: `docs/constellation_cognitive_engine_v2.1.pdf`
- **Session logs**: `lab/reports/SESSION-LOG-YYYY-MM-DD.md`
- **Lessons learned**: `docs/LESSONS-LEARNED.md`
