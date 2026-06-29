# Handover — 2026-06-29 — PJ-065 SHIPPED (structural / parent-TOC link)

PJ-065 is **built, validated, and closed out**. This handover records the shipped state and scopes the next build phase (the three remaining candidates from the 2026-06-27 build-phase handover, minus PJ-065).

## What shipped this session (all on `main`)
The **Structural / Parent-TOC link type** — a new, NON-cognitive link kind (the compositional spine: Book → Part → Chapter → Scene), excluded from every cognitive surface. End-to-end, Boss-validated:
- **Author** — `parent` / `contains` preset properties in the Property editor, authored as `list` chips with bracket-guarded auto-`[[ ]]` wrap (type a name, get a link).
- **View** — the **Structure** panel (`StructuralOutlinePanel.svelte`, right sidebar after Backlinks): ancestor breadcrumb + whole-work outline, a Whole-work ⇄ This-note focus toggle, you-are-here highlight, virtualized ≥50 rows.
- **Guard** — single-parent (child's own `parent` wins; strict tree; `contains` overruled on conflict) + acyclicity (visited-set cut, ↻ marker).
- **Resolve** — contested rows show **Keep** / **Move here** → `resolve_structural_conflict` (gate_write → reindex → cascade:rewrote; 6 tests).
- **Exclusion** — `link_types::structural_not_in_clause` / `is_structural_type` / `STRUCTURAL_SEED_IDS=['parent','contains']`; Phase-4 audit (3 agents) confirmed exclusion on every surface + closed 2 P1 leaks (sky_backfill, extract_wikilinks) + P3 hardening.
- **Bonus #1** — the app-wide **24 s rename → instant** (deleted a dead `UPDATE note_links SET target_path` full-scan; target_path is NULL by design).
- **Bonus #2** — a latent **Sight bug** (`sight_v6.rs`): 6 subqueries + the link-set SELECT matched the always-NULL target_path; migrated to `(target_path OR target_name_lower)`. Verified live (ISBN inbound 0→5359). Sight is a disabled Wing → no live impact.

**Verification:** cargo test 998/0 (+ new sight_v6 + structural tests), svelte-check 0 errors, every Boss test PASS. Orientation bumped to **v3.16**.

## Open follow-ups (none blocking)
- **SV/OC Phase-2 concept stub** — banked at `docs/concept-papers/PJ-065-Phase2-SV-OC-Visualization-Concept-Stub.md` (how structural links could render on Sky View / Outgoing-Connections). For later; not started.
- **Reviewed-and-kept (NOT defects):** the frontmatter split/rebuild preamble is duplicated across 3 `libraries.rs` helpers (each has a deliberate variation — `update_frontmatter_title` intentionally omits the blank-line guard the field-removers need); `children_of` does one `parent_of` resolve per `contains`-child (bounded by TOC size). Both consciously kept per the /simplify review; revisit only if a 4th site appears or a very wide `contains:` list shows up.

## Next build candidates (SO #8 cross-check still valid from 2026-06-27; re-check the BODY of orientation v3.16 + recent session logs at the moment you start each)
| Candidate | Verdict | Build? |
|---|---|---|
| **PJ-067** Living Link Relationship Model v2 | Concept ratified 2026-06-27, **not built** | ✅ genuine `/migration` build, **Tension-first** (dimension engine → tension vocab → synthesis → thematic family). Dep MIG-086 shipped. |
| **MIG-080** right-click context menus | Banked, **not built** (hand-rolled menus exist to fold in) | ✅ genuine build — shared `<ContextMenu>` + the 4 Obsidian menus; **note list stays NATIVE WebView2**; 64 items × 15 locales. |
| **MIG-084** Rich Reviewer | **SHIPPED + Boss-validated 2026-06-23 — do NOT re-build** | ❌ only the **Reviewer Style Setter text-resize category** remains (small; mirror the Cataloger `--cat-scale` pattern; clarify per-component vs global with Boss first). |
| **Backup & Recovery system** | Boss-wanted (2026-06-21), concept paper exists | After MIG-080, via `/migration` + WA#5. |

---

## Ready-to-paste next-session prompt

```
Constellation — session start. git pull; read git log -10; read docs/LESSONS-LEARNED.md;
read the highest-version docs/Constellation Orientation & Onboarding v*.md (now v3.16 — its
preamble covers PJ-065 + the rename-perf + sight_v6 fixes); read
docs/handover/Handover-2026-06-29-PJ-065-shipped.md.

PJ-065 (structural / parent-TOC link) is SHIPPED + closed out. Build phase continues.
SO #8 verdicts (re-check the BODY of orientation v3.16 + recent session logs before starting):
  • PJ-067 (Living Link v2) — ratified, NOT built → genuine /migration build, Tension-first.
  • MIG-080 (right-click) — banked, NOT built → build (shared <ContextMenu> + 4 Obsidian menus;
    note list stays NATIVE; 64 items × 15 locales).
  • MIG-084 (Rich Reviewer) — ⚠ ALREADY SHIPPED 2026-06-23. DO NOT re-build. Only the Reviewer
    Style Setter text-resize category remains (small; mirror Cataloger --cat-scale).
  • Backup & Recovery — Boss-wanted; after MIG-080, via /migration + WA#5.

Recommended: PJ-067 is the next big feature build (/migration: Architect → Plan → Build → Audit;
Boss rules the open R1/R2/R3/R5/R6 decisions first). MIG-080 is self-contained. The MIG-084
Style-Setter sub-item is a quick win.

Boss: which first? Ultracode is on — Workflows for substantive work. One item at a time;
/migration for cross-subsystem builds; staged Boss tests where user-facing; full SO close-out
(commit + session log + orientation v-bump + MoCh + handover + help/manual in 15 langs) before
the next.
```
