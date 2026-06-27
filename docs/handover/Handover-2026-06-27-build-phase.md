# Handover — 2026-06-27 — BUILD PHASE (next 4 candidates)

The 7-item backlog is complete. This handover scopes the **next build phase** across the four candidates the Boss named: **PJ-065, PJ-067, MIG-080, MIG-084.** Each was SO #8 cross-checked (4 parallel Explore agents, 2026-06-27) against the concept papers, the orientation v3.15 body, recent session logs, and the live code — **before** any build, per the rule that exists to stop the PJ-006 stale-entry trap.

## ⚠️ SO #8 verdicts up front
| Candidate | Verdict | Build? |
|---|---|---|
| **PJ-065** parent/TOC link | Concept RATIFIED, **build not started** | ✅ genuine `/migration` build |
| **PJ-067** Living Link v2 | Concept RATIFIED, **build not started** | ✅ genuine `/migration` build (Tension-first) |
| **MIG-080** right-click | Banked, **build not started** (but hand-rolled menus already exist to fold in) | ✅ genuine build (fold + the 4 menus) |
| **MIG-084** Rich Reviewer | **⚠️ ALREADY SHIPPED + Boss-validated 2026-06-23** (`HANDOVER-2026-06-23-mig084-complete.md`; `ReviewerView.svelte`/`ReviewStatusPanel.svelte` live) | ❌ NOT a build — only the **Reviewer Style Setter category** sub-item remains (small) |

**Do not re-build MIG-084.** The master-detail Rich Reviewer (6 lenses, diagnosis/prescription, priority engine `priorities.ts`, left-dock universe reviewer vs right-sidebar `ReviewStatusPanel` per MIG-080 §F) is live. The only open piece is a Reviewer **Style Setter text-resize category** (see below).

---

## PJ-065 — structural parent/TOC link type  (genuine `/migration` build)
**STATUS:** Concept ratified (Boss 2026-06-27, `docs/concept-papers/PJ-065-Parent-TOC-Link-Type-Concept-Paper.md`); build not started.
**SCOPE:** a distinct, **non-cognitive** link kind (the compositional spine — TOC/outline/chapter→scene). Schema (order column), write-path (frontmatter fold + auto-reverse + acyclicity), read-paths (TOC panel + breadcrumb), optional OrgChart extension. Crosses Rust↔Svelte → `/migration`.
**RULED:** distinct kind via a `structural:true` flag on the link-type registry (excluded from maturity/stratum/living-link apparatus, no weight/confidence/decay); both `parent:` and `contains:` directions but **materialize the reverse write-time**; multi-parent DAG with one **primary** spine; acyclic via write-time CTE guard; **store direct edges, compute closure on read** (LL-XXX rule); MVP = TOC panel + breadcrumb.
**OPEN (Architect rules):** the precise NAME + distinct color (must not collide with cognitive `part-of`); storage shape (extend `note_links` + nullable `seq` vs dedicated `note_toc` table — rec: extend); ordering mechanism (integer `seq` vs fractional/rebalanceable rank — note the ranked-dimensions memo); UI surface (new TOC panel vs OrgChart links-mode).
**GOTCHA — the one missing primitive:** `note_links` has **NO order column**; frontmatter YAML order is lost at the index boundary. That's the central thing PJ-065 adds. OrgChart today reads the filesystem (single-parent), not links. MIG-086 §F fold/reverse machinery exists — wire the auto-derive for the structural type.
**FIRST STEP:** open `/migration` Architect; Boss rules NAME+color, storage, ordering, UI surface. Don't schema-design before the name is ruled.

## PJ-067 — Living Link Relationship Model v2  (genuine `/migration` build, Tension-first)
**STATUS:** Concept ratified (Boss 2026-06-27, `PJ-067-...-v2-Concept-Paper.md` + `PJ-067-R4-Wargame-...md`); build not started. Dependency MIG-086 (frontmatter typed-links + dual-source fold) SHIPPED 2026-06-26.
**SCOPE (4 phases):** (1) **Dimension engine** — declarable per-type OWL-2 characteristics (symmetry/transitive/inverse/functional); typed backlinks both ends (store one, derive reverse — SKOS); transitive closure **on read** (recursive CTE, never stored). (2) **Tension vocabulary** — `undermines`/`undercuts` (children of `contradicts`), `qualifies`, `problematizes`/`answers`. (3) **Synthesis** — the n-ary `SYNTH` note-kind + `maps-to` analogy. (4) **Thematic family** — `used-for`/`prerequisite-of`/`precedes`/`near`. Tension-first per the paper.
**RULED:** R4 LOCKED — `problematizes/answers` = its own **interrogative family** (not a 9th act, not untyped); the canonical 8 + §7 order **FROZEN**; `part-of` transitive only within one Winston meronymy subtype; closure ON READ only (the LL-XXX OOM lesson); store-one-derive-reverse (SKOS).
**OPEN (Architect, R1/R2/R3/R5/R6):** R1 silent engine vs surfaced teaching vocabulary; R2 inverse label set + backlink styling; R3 thematic family scope (4 vs subset); R5 n-ary authorship (constituent-first vs synth-node-first) + analogy pairwise vs reified; R6 phasing.
**GOTCHA:** the n-ary `SYNTH` note-kind is **shared infra with analogy's mature reified form — build once, two consumers**. Registry one-level child-nesting already exists (MIG-067) for `undermines`/`undercuts`; the interrogative family is a parallel registry block, not nesting. Boot-perf gate (Rule 8) on the closure CTE over 7,600 notes.
**FIRST STEP:** `/migration` Architect for R1/R2/R3/R5/R6 → kick off Phase 1 (add `characteristics` to `LinkTypeDef` + registry merge + index wiring, silent-mode-only).

## MIG-080 — right-click context menus  (genuine build)
**STATUS:** banked (Obsidian reference `Right-Click-Reference-Obsidian.md`, 2026-06-21), build not started. NOT greenfield — hand-rolled menus already exist (`FileTree.svelte`, `EditorContextMenu.svelte`, `contextMenuBuilder.ts`/`buildContextMenu()`; Backlinks/Outgoing/Sky/OrgChart/IndexPanel/Tabs have their own popovers).
**SCOPE:** a shared `<ContextMenu>` component + wire the **4 Obsidian target menus** (Note ~16 items · Folder ~14 · Link/selection ~20 · editor-empty ~14). Fold the existing hand-rolled handlers into the one builder.
**RULED:** Obsidian menus = the target (fidelity, with Constellation terminology/i18n/RTL adaptations); shared `buildContextMenu()` is the single source of truth; **the note list intentionally keeps the native WebView2 menu — NO custom right-click there** (the Item-1 correction, now in v3.15). File Tree gets menus.
**OPEN:** submenu nesting depth/style; icon vocabulary; whether the link menu binds to CM6 selection or the living-link widget; disabled-state rules (archive/read-only/multi-select).
**GOTCHA:** airtight WebView2 native-menu suppression (`preventDefault`+`stopPropagation`); RTL chevron flips + left-opening submenus (`detectDir()` exists); **64 menu items × 15 locales i18n with hard keys (no `$t||fallback`)**; verify CM6's `contextmenu` hook fires under WebView2.
**FIRST STEP:** `/migration`; architect the shared submenu component, fold File Tree first (verify en+ar+RTL), then unify the Backlinks/Outgoing popovers, editor+link menus last.

## MIG-084 — Rich Reviewer  (⚠️ SHIPPED — only the Style Setter sub-item remains)
**STATUS:** **COMPLETE + Boss-validated 2026-06-23.** Do not re-build. The remaining open item is the **Reviewer Style Setter category** (text-resize), not started.
**THE ONLY REMAINING WORK:** add a Reviewer text-resize Style Setter category, mirroring the Cataloger pattern (`--cat-scale` CSS var + Style Setter slider). Define `--reviewer-text-scale` over `.rv-detail`/`.rv-d-title`/`.rv-d-summary`/labels; 10 sliders (70–140%, default 100%); add the category + control labels to `styleSetter.labels.*` and localize ×15. Test the self-explanatory `whyNow` rows stay crisp at 70% and 140%.
**FIRST CLARIFY with Boss:** per-component scale (like Cataloger) or global Reviewer scale; which controls.

---

## Ready-to-paste next-session prompt

```
Constellation — session start. git pull; read git log -8; read docs/LESSONS-LEARNED.md;
read the highest-version docs/Constellation Orientation & Onboarding v*.md (v3.15);
read docs/handover/Handover-2026-06-27-build-phase.md (SO #8 already done for the 4 candidates).

The 7-item backlog is COMPLETE. Build phase. SO #8 verdicts (already cross-checked):
  • PJ-065 (parent/TOC link) — ratified, NOT built → genuine /migration build.
  • PJ-067 (Living Link v2) — ratified, NOT built → genuine /migration build, Tension-first.
  • MIG-080 (right-click) — banked, NOT built → genuine build (fold existing hand-rolled
    menus into a shared <ContextMenu> + the 4 Obsidian menus; note list stays NATIVE; 64
    items × 15 locales). 
  • MIG-084 (Rich Reviewer) — ⚠️ ALREADY SHIPPED 2026-06-23. DO NOT re-build. Only the
    Reviewer Style Setter text-resize category remains (small; mirror the Cataloger
    --cat-scale pattern).

Recommended order: PJ-065 → PJ-067 are the two real feature builds (both /migration:
Architect → Plan → Build → Audit; Boss rules the open decisions first). MIG-080 is a
self-contained build. The MIG-084 Style-Setter sub-item is a quick win.

Boss: which first? Ultracode is on — use Workflows for substantive work. One item at a
time; /migration for the cross-subsystem builds (PJ-065, PJ-067); staged Boss tests where
user-facing; full SO close-out (commit + session log + orientation v-bump + MoCh + handover)
before the next. Cross-check the BODY (not preambles) of orientation + session logs again
at the moment you start each, in case anything drifted.
```
