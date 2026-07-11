# Handover — 2026-07-11 — Knowledge Cockpit three-lens set COMPLETE

**Read `docs/Constellation Orientation & Onboarding v3.36.md` first** (highest version). Then this. Then `git pull origin main`.

---

## 1. What shipped this arc (verified + Boss-validated, protected)

The second-screen **Knowledge Cockpit** — a read-only contextual view of the open note — now has **three note-graph lenses on one chassis**, each owning one distinct axis:

| Lens | Question it answers | File |
|---|---|---|
| **Butterfly** (default) | composition — in/out + which relationship types carry the weight | `NoteButterflyGraph.svelte` |
| **Ledger** | magnitude — exact, comparable counts per type | `NoteLedgerGraph.svelte` |
| **Orrery** | time — which links are alive vs going cold, + the load-bearing-decay alarm | `NoteOrreryGraph.svelte` |

Shared chassis: `src/lib/cockpitGraphData.ts` (grouping, `deriveStats`, `relColor`, `relLabelIn`, `recencyShell`, `earnedWeight`, `RECENCY_SHELLS`) and `NoteGaugeDeck.svelte` (the four Cognitive-Engine gauges). Lens registry + `normalizeGraphStyle` in `cockpitFlag.ts`. Render switch + on-page lens toggle in `SecondScreenCockpit.svelte`.

- **Aster RETIRED**, **Heartwood CUT** (both by honest concept review — see orientation v3.36 preamble). `normalizeGraphStyle()` migrates a stored `'aster'`/`'heartwood'` → `butterfly`.
- **One palette:** `relColor(id) = var(--rel-<id>, linkTypeColor(id))` — Link Types registry is the base (pills / Sight / CCS / GraphMind + all 3 lenses agree); a Style-Setter **"Note graph → Relationship colours"** category overrides the graph alone.
- **Lens toggle on the cockpit page** (not Settings). SS never writes settings: emits `screen:set-lens` → MAIN `updateSettings` → `screen:settings-changed` broadcast re-renders SS. The dead **Peek** dial was removed (`DialMode = normal | locked`).
- **i18n ×15 incl. RTL:** whole `cockpit.*` block (60 strings) + `cockpit.orrery.*` localized; the SVG label-flip (RTL) fixed by pinning graph SVGs `direction:ltr`; type names via `relLabelIn($locale)`; note-box count reads count-then-word in RTL; **Arabic plural** `many` (11–99) → plain singular (`212 رابط`).

## 2. State of standing

- **Verified-shipped + protected:** everything in §1. `svelte-check` 0 errors / 0 warnings on all cockpit files. Binary rebuilt **2026-07-11 14:03** with the localized labels. Committed + pushed to `origin/main` (HEAD `7e43174d`).
- **At-risk / uncommitted:** none. Working tree clean, 0 unpushed.
- **Known-broken:** none in the cockpit.
- **Doc drift:** none introduced — orientation v3.36, session log (2026-07-09), User Manual §9 + the "Second Screen" help topic, and memory are all current.

## 3. Open threads for the next order of work (pick with the Boss)

1. **Auto-restore-tabs-on-relaunch** — Boss-wanted 2026-07-09. A Settings toggle, **default ON**. Today open tabs are NOT persisted across restart (only manual named workspaces are). This is a real, self-contained feature.
2. **The safety-sweep backlog** — the standing G2–G8 confirmed findings in `docs/Constellation-Safety-Audit-CHARTER.md` (the per-cycle whole-app `safety-inspection` register). Remediate Reproduce-First.
3. **Orrery polish (optional, only if the Boss raises it):** switching *directly* between two expanded wings is fiddly because the non-hovered wings shrink to slivers. A small always-clickable per-wing label handle would fix it. Not a bug — a known interaction cost of "take all the space."

## 4. Standing rules that bite in this area

- **The Art Director & Team own UX/UI design AND coding** (`feedback_art_director_team_owns_ui`). For any visual/UX build, run the multi-agent Art-Director workflow — **specialists → AD spec → N competing engineers → adversarial judges → lead-engineer merge** — don't hand-iterate visual design solo. The five Orrery workflows are the template (`.claude/workflows/scripts/orrery-*.js` under the session dir, or re-author).
- **Every build:** `/simplify`; `safety-inspection` **diff-scoped** if the change touches a write / index / lifecycle / persisted-JSON / frontmatter path (the cockpit is read-only display → exempt); `svelte-check` 0/0; **`npm run build` BEFORE `cargo build --release`** (cargo alone re-embeds a stale `build/`); **verify the binary mtime is newer than source** before any Boss test.
- **Test tutorials:** staged, one stage at a time; define the feature first, then walk click-by-click.
- **The SS is read-only always** (Display-not-Domain); it never writes settings or files.

## 5. Gotchas (will cost you a round-trip if forgotten)

- **Art-Director build workflows return the `.svelte` file HTML-escaped** (`&lt; &gt; &amp;`) inside the JSON string field. `html.unescape` it (single level) before writing to disk, then `svelte-check`.
- **Null-narrowing in keyboard handlers:** a `$state` that was just assigned non-null still types as `T | null` across the assignment — use a local `const` or an explicit guard before passing it to a non-null param.
- **i18n `$t(missingKey)` returns the KEY** (truthy), so `$t(k) || fb` never falls back. Use the `L(k, fb) = $t(k) === k ? fb : $t(k)` helper (already in the lens components).
- **The link-type registry is a per-window cache** seeded from the boot bundle in the MAIN window only — a second-screen surface that reads `linkTypeColor()` must `loadLinkTypes()` itself (SecondScreenPage already does, on mount / universe-switch / `link-types:changed`).

## 6. Where to resume

`lab/reports/NEXT-SESSION-PROMPT.md` is the ready-to-paste kickoff. Full narrative: `lab/reports/SESSION-LOG-2026-07-09.md` (the cockpit arc, 2026-07-09 → 07-11). Conversational trace: `docs/MoCh/MoCh-2026-07-11-0900.md`.
