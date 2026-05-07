# MIG-017 — Disable v2 Sight (Plan)

**Companion to**: `MIG-017-DISABLE-V2-SIGHT-ARCHITECT.md`
**Effort**: Single phase, single commit. Mini-MIG.
**Approval mode**: cascading per "Plan Approval = Build Approval" — Eisa's "Proceed" greenlights the full sequence; the only gate is the post-Build audit.

---

## §1 — Implement the SIGHT_V2_ENABLED gate (single phase)

Each step is a single file edit; together they form one commit.

### §1.1 — Create the engine flag module

**File (new)**: `src/lib/sight/engine.ts`

Content per Architect §4.2 — the comment block + `export const SIGHT_V2_ENABLED = false;`.

**Verification**: file exists, exports parse cleanly (`tsc --noEmit` will catch any error in §1.7 typecheck).

### §1.2 — Gate the dock button

**File**: `src/routes/+layout.svelte`
**Edit**: line 4358.

Before:
```svelte
{#if $appSettings.enabledFeatures?.constellationSight !== false}
```

After:
```svelte
{#if SIGHT_V2_ENABLED && $appSettings.enabledFeatures?.constellationSight !== false}
```

Add `import { SIGHT_V2_ENABLED } from '$lib/sight/engine';` to the script block (alongside the existing `ConstellationSight` import at line 64).

**Verification**: dock button no longer renders in `npm run tauri dev`.

### §1.3 — Gate the modal mount (defense-in-depth)

**File**: `src/routes/+layout.svelte`
**Edit**: line 4989 — change the inner `{#if lensActive}` to `{#if lensActive && SIGHT_V2_ENABLED}`.

This is belt-and-suspenders: the dock button gate already prevents `lensActive = true` from being set via user click; this defensive gate prevents any stray code path (current or future) from rendering the modal.

**Verification**: even if `lensActive` is forced to `true` via DevTools, the overlay does not render.

### §1.4 — Gate the "Return to Lens" button

**File**: `src/routes/+layout.svelte`
**Edit**: line 4738 — change `{#if lensReturnPending}` to `{#if lensReturnPending && SIGHT_V2_ENABLED}`.

Same defense-in-depth rationale. Unreachable cold (requires v2 to have been opened first), but a stray code path could set `lensReturnPending = true`.

**Verification**: button does not render under any state.

### §1.5 — Hide the Settings plugin entry

**File**: `src/lib/components/SettingsModal.svelte`
**Edit**: line 267.

Before:
```svelte
{ id: 'constellationSight', name: ..., desc: ..., icon: '👁️' },
```

After: wrap the line in a conditional spread so the entry is omitted when `SIGHT_V2_ENABLED` is `false`.

```svelte
...(SIGHT_V2_ENABLED ? [{ id: 'constellationSight', name: ..., desc: ..., icon: '👁️' }] : []),
```

Add `import { SIGHT_V2_ENABLED } from '$lib/sight/engine';` to the script block.

**Verification**: Settings → Plugins → Visualization renders without the "Constellation Sight" row.

### §1.6 — Prepend banner to help docs (en + ar)

**Files**:
- `docs/help.uConstellation.World/Constellation Sight/Constellation Sight.md`
- `docs/help.ar/Constellation Sight/Constellation Sight.md` *(if exists; verify before edit)*

**Edit**: prepend the banner block from Architect §5 directly after the YAML frontmatter (so it sits at the top of the rendered help page).

Arabic banner translation:
```markdown
> **🚧 جاري إعادة بناء Constellation Sight.**
>
> تم تعطيل التصور الحالي "بئر الجاذبية" (الإصدار 2). يجري تصميم
> Sight جديد (الإصدار 3) — على أساس جمالية خريطة النجوم تتيح لك
> رؤية كامل عالم معرفتك في لمحة واحدة. الصفحة المرجعية أدناه تصف
> ما كان يفعله الإصدار 2 وهي محفوظة بينما يتم بناء الإصدار 3.
>
> اقرأ [`Constellation-Sight-Concept-Paper-v1.1.md`](../../Constellation-Sight-Concept-Paper-v1.1.md)
> لفهم لماذا يوجد Sight وانظر §13–§14 لرؤية الإصدار 3.
```

**Verification**: opening either help doc shows the banner above the v2 documentation.

### §1.7 — Type-check + smoke test

Run `npm run check` (or `npx svelte-check`) — must produce no new errors.

Visual smoke test: `npm run tauri dev`, verify:
- No dock button between OrgChart and Inspector360 dock buttons.
- Settings → Plugins → Visualization: no "Constellation Sight" row.
- Sky View, OrgChart, Map, Index, SearchHub, Inspector360 still open from their own dock buttons.
- Help library: opening "Constellation Sight" page shows the 🚧 banner.

### §1.8 — Bump orientation v1.55 → v1.56 inline (SO #6)

New file `docs/Constellation Orientation & Onboarding v1.56.md` — copies v1.55 with:
- Version + date bump.
- v1.56 preamble: MIG-017 closes, PJ-039 done, v2 Sight unreachable in production, v3 (PJ-038) unblocked.
- §8 Migrations table: MIG-017 row flips from "Next-up" to ✅ Closed with this commit's hash.

### §1.9 — Bump Pending Jobs v1.4 → v1.5

New file `docs/Constellation Pending Jobs v1.5.md` — copies v1.4 with:
- v1.5 preamble: PJ-039 closed.
- PJ-039 entry status: Open (next-up) → Done; commit hash recorded.
- §9 Done table: add PJ-039 row.
- Top of queue rotates: PJ-038 (Sight v3 + own Concept Paper) → PJ-005 → PJ-002.

### §1.10 — Three-agent audit (parallel)

After Build completes and before commit:

- **Invariants agent**: verify all 12 invariants of Architect §6 hold. Full grep for any missed v2 entry point (`lensActive\s*=\s*true`, `toggleLens\s*\(`, `ConstellationSight2`).
- **Drift agent**: verify no consumer of `SIGHT_V2_ENABLED` is missed.
- **Migration-path agent**: verify the five scenarios of Architect §8 still produce the expected end-state.

Audit doc: `lab/reports/MIG-017-DISABLE-V2-SIGHT-AUDIT.md`. P0 / P1 fixes block commit; P2 / P3 logged as PJs.

### §1.11 — Session log + commit

- Append to `lab/reports/SESSION-LOG-2026-05-07.md` with the close-out summary.
- Single commit: all 6+ files (engine.ts, +layout.svelte, SettingsModal.svelte, two help docs, orientation v1.56, Pending Jobs v1.5, audit report, session log).
- Commit message format per project convention.

---

## Verification checklist (final)

| # | Check | Pass criterion |
|---|---|---|
| 1 | `npm run check` | No new errors. Pre-existing PJ-012 LinkLifecycle.fresh error allowed. |
| 2 | `npm run tauri dev` boots cleanly | App launches; first paint normal. |
| 3 | No Sight dock button | Visual confirmation. |
| 4 | No Settings plugin entry | Settings → Plugins → Visualization renders without the row. |
| 5 | No regression in other views | Sky View, OrgChart, Map, Index, SearchHub, Inspector360 all open from their dock buttons. |
| 6 | Help doc banner | Top of "Constellation Sight" help page shows the 🚧 banner. |
| 7 | v2 still on disk | `ConstellationSight2.svelte` exists; `lens.rs` exists; `constellation_sight_*` IPCs registered in `lib.rs`. |
| 8 | Three-agent audit | 0 P0, 0 P1. |
| 9 | M11 zero-diff | `git diff src-tauri/src/lexicon/` is empty. |

---

**End of Plan.** Build cascades from §1.1 through §1.11 in one commit. Audit gates the commit; if any P0/P1 surface, fix and re-audit before committing.
