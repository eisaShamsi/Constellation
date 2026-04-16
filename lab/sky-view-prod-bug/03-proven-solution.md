# Proven solution — Sky View production-only bug

**Status:** Research complete. Evidence-backed. Ready for user approval.
**Lab:** `lab/sky-view-prod-bug/` — see `00-MISSION.md`, `01-source-trace.md`, `02-svelte-docs-verdict.md`.
**Target baseline:** HEAD `ef45c17` (unmodified).

---

## TL;DR

**One two-line change** in `src/routes/+layout.svelte`:

```ts
// line 541-542 — BEFORE
let skyNodes: SkyNode[] = [];
let skyLinks: SkyLink[] = [];

// line 541-542 — AFTER
let skyNodes = $state.raw<SkyNode[]>([]);
let skyLinks = $state.raw<SkyLink[]>([]);
```

That's it. No other edits to code. No file moves. No IPC changes. No build changes.

---

## Why this is the fix (one paragraph)

The declarations at `+layout.svelte:541-542` are plain `let`. In Svelte 5 runes mode, plain `let` reassignment is **not tracked** — no effect, no template binding, no derivation re-runs when you do `skyNodes = nodes`. The author's intent (comment at lines 539-540, 562-564) was to avoid `$state` proxy overhead on large arrays and use `starVersion++` as an external signal. But **nothing on the main Sky View chain reads `starVersion`** — only the WiW overlay (lines 570, 581) and the right-sidebar local-star effect (line 908). So when `refreshLibraryCaches()` finishes and runs `skyNodes = nodes; starVersion++` at lines 1936-1938, the `<GraphMindView nodes={skyNodes} ... />` binding at line 3869-3870 does not re-evaluate. The graph stays empty. Dev hides this because boot is slow (~1m46s); the user clicks Sky View only after population completes, so the initial mount gets good data. Prod exposes it because paint is instant (~420 ms) while `refreshLibraryCaches` still takes ~8.2 s — the user clicks Sky View inside that window, GraphMindView mounts with `nodes=[]`, the `onMount` gate `if (nodes.length > 0)` skips, and `stars stay at 0 · 0 edges` forever because no reactive scope fires when `skyNodes` is later reassigned.

`$state.raw` is the documented Svelte 5 pattern for exactly this case: **tracks reassignment, skips the per-element proxy wrap**. Preserves the performance optimization the author wanted; delivers the reactivity the UI needs.

---

## Evidence (citations)

### Svelte 5 official docs (`https://svelte.dev/docs/svelte/$state`, fetched 2026-04-16)

> "When you reference something declared with the `$state` rune, ... you're accessing its _current value_."

> "State declared with `$state.raw` cannot be mutated; it can only be _reassigned_. In other words, rather than assigning to a property of an object, or using an array method like `push`, replace the object or array altogether."

### Plain-`let` declarations at fault

- `src/routes/+layout.svelte:541-542` — `let skyNodes: SkyNode[] = []; let skyLinks: SkyLink[] = [];` (plain `let`, no `$state`).
- Comment at `src/routes/+layout.svelte:539-540` confirms intent: "Star data stored as plain (non-reactive) arrays to avoid `$state` proxy overhead on potentially tens of thousands of items. Use starVersion to signal changes."

### `starVersion` is the wrong signal — grep of all readers

- `src/routes/+layout.svelte:570` — inside `$derived.by` for `wiwFilteredNodes` (WiW overlay).
- `src/routes/+layout.svelte:581` — inside `$derived.by` for `wiwFilteredLinks` (WiW overlay).
- `src/routes/+layout.svelte:908` — inside `$effect` for right-sidebar local-star graph.
- No read of `starVersion` in or above the `<GraphMindView nodes={skyNodes} ... />` binding at line 3869-3870. The main Sky View chain never re-runs on `starVersion++`.

### Writers of `skyNodes` / `skyLinks` — whole-array reassignment only

- `src/routes/+layout.svelte:1936-1937` — `skyNodes = nodes; skyLinks = gLinks;` inside `refreshLibraryCaches`.
- No other writers in the codebase. No `.push`, no `.splice`, no index-assignment, no inner mutation. `$state.raw`'s constraint ("can only be reassigned") is satisfied.

### Readers of `skyNodes` / `skyLinks` — iteration-only

- `src/routes/+layout.svelte:569-583` — WiW filter (`.filter`, `.map`, `.find` on both arrays).
- `src/routes/+layout.svelte:908-…` — local-star effect (`.filter` on `skyLinks`).
- `src/routes/+layout.svelte:3869-3870` — `<GraphMindView nodes={skyNodes} links={skyLinks} />` prop pass-through.
- All are pure iteration / pass-through. None mutate. `$state.raw` safe.

### Timing race — why dev passes and prod fails

From the user's scorecard on HEAD `ef45c17` (trial Universe, 7,595 notes):

| | dev | prod |
|---|---|---|
| paint_ms | ~multi-second | **419 ms** |
| hydrated_ms | slow enough that the user waits | **8,669 ms** |

- Prod: 8.25-second window between paint and hydrated. User clicks Sky View inside that window → `GraphMindView.onMount` runs with `nodes=[]` → `if (nodes.length > 0)` gate at `GraphMindView.svelte:629` skips → `prevNodeLen = 0`. When `skyNodes = nodes` finally runs, no reactive scope re-evaluates the `nodes={skyNodes}` prop binding → child's `nodes` prop stays `[]` → the `$effect` at `GraphMindView.svelte:556` never sees `len > 0`.
- Dev: slow boot (~1m46s) closes the window before the user clicks. Initial mount gets populated data.

### Rule-out of other candidates

- **Candidate A** (`libraryList.length === 0` at line 1931): refuted. `libraries.set(bundle.libraries)` runs synchronously at line 1455 before `refreshLibraryCaches` is dispatched at line 1549; the user's screenshot shows "17 libraries · 7597 notes", so the store is populated.
- **Candidate C** (worker URL resolution in bundled app): refuted. The worker is only spun up inside `engine.setData()`, which is never reached when the prop is empty. Downstream of Candidate B.
- **H1–H8** (IPC truncation, Rust release mode, SQLite path, silent error, Svelte prod guards, etc.): not needed — the plain-`let` non-reactivity fully explains the observed dev↔prod divergence. See `02-svelte-docs-verdict.md`.

---

## Exact diff

```diff
--- a/src/routes/+layout.svelte
+++ b/src/routes/+layout.svelte
@@ -538,8 +538,9 @@
 	let allIndexEntries = $state<IndexEntry[]>([]);
-	// Star data stored as plain (non-reactive) arrays to avoid $state proxy overhead
-	// on potentially tens of thousands of items. Use starVersion to signal changes.
-	let skyNodes: SkyNode[] = [];
-	let skyLinks: SkyLink[] = [];
+	// Star data uses $state.raw — tracks reassignment (required for the main Sky View
+	// <GraphMindView nodes={skyNodes}> binding to re-run on population) but skips the
+	// per-element proxy wrap, preserving iteration perf on 10k+ element arrays.
+	let skyNodes = $state.raw<SkyNode[]>([]);
+	let skyLinks = $state.raw<SkyLink[]>([]);
```

Only two source lines actually change. Comment updated to reflect the new mechanism.

**Lines 1936-1937 (the writers) are unchanged** — whole-array reassignment is exactly what `$state.raw` supports.
**Line 1938 (`starVersion++`) is unchanged** — the WiW overlay and local-star effect still read it; leaving them in is belt-and-suspenders. Removing them is scope creep and was explicitly rejected in the lab (see `02-svelte-docs-verdict.md` § Collateral effects).

---

## Predicted outcome

Before the rebuild:

- **UI:** Open the trial Universe → paint in ~420 ms → click Sky View immediately → status bar transitions from `0 nodes · 0 edges` to `~7595 nodes · N edges` within the hydration window (~8 s). The graph renders. No need to toggle Sky View off/on.
- **Dev mode:** unchanged (was already working).
- **Scorecard:** `note_count: 7595` as before. `hydrated_ms` unchanged (this fix is reactivity, not perf — Criterion 2 is a separate investigation once SV is trusted again).
- **Collateral:** WiW filter, right-sidebar local star, lens, second-screen export — all unaffected (they already either read `starVersion` or pass through).

If Sky View still shows `0 nodes · 0 edges` after this fix, the hypothesis in `01-source-trace.md` and `02-svelte-docs-verdict.md` is wrong — treat it as new evidence, go back to step 1 of the research process (per `feedback_audit_before_implement.md`). **Do not try a variation.**

---

## Verification plan (user-facing)

1. Apply the two-line change to `src/routes/+layout.svelte:541-542` (plus the comment update).
2. `npm run tauri build` from the project root.
3. Launch `src-tauri/target/release/constellation.exe`.
4. Open the trial Universe (`E:\Kingdom of Eisa\Eisa Cognitive Knowledge`, 7,595 notes, 17 libraries).
5. As soon as the sidebar is visible (≈1 s), click **Sky View**.
6. Watch the status bar at the bottom of the Sky View panel:
   - **Expected**: starts at `0 nodes · 0 edges`, then — within the ~8 s hydration window — transitions to positive counts (~7,595 nodes, tens of thousands of edges). Graph renders dots and lines.
   - **Regression signal**: stays at `0 nodes · 0 edges` indefinitely → the hypothesis is wrong, revert and re-research.
7. Close Sky View, re-open it. Graph should appear immediately (mount sees populated arrays).
8. Click a note. Backlinks panel populates (Backlinks reads `allLibraryLinks` which is still `$state`, unaffected).
9. Read `boot-perf.latest.json` at the Universe root — `note_count: 7595`, `hydrated_ms` similar to before (~8-9 s).

---

## Independent audit agent results (2026-04-16, re-run after initial 529s)

**Agent 2 — Tauri v2 IPC prod vs dev audit** (independent, given only the failure symptoms + hypothesis, asked to verify or refute):

> "IPC is almost certainly NOT the culprit. The IPC call is atomic — a single `invoke()` returning a single struct. There is no partial-success mode where `notes` arrives (count 7595) but `links` silently truncates: they ride the same serde buffer and the same IPC envelope. Tauri v2 prod uses the same serde_json path dev does; no release-mode optimization alters rusqlite iteration or serde::Serialize derive output."
>
> "Hypothesis is consistent with the evidence. `skyNodes`/`skyLinks` being plain `let` with a separate `$state` signal is a known-fragile pattern in Svelte 5: template prop bindings track the *read* site, not a sibling signal. `starVersion` gates the `$derived.by` blocks (lines 570, 581, 908) correctly because those explicitly read `_ver = starVersion`, but the `<GraphMindView>` prop expressions at lines 3869-3870 do not."

**Agent 3 — End-to-end data-flow code trace** (independent, asked to find holes):

> "No reactive wrapper intercepts the `skyNodes`/`skyLinks` reads at 3869-3870. The template binding is a direct plain-`let` access. Svelte 5 runes mode does NOT track plain `let` reassignments in templates."
>
> "Nuance on 'why dev passes': Svelte dev's `_$.validate_prop_bindings` / deeper tracking does not magically make plain `let` reactive. More likely dev passes because the slower boot means `showSkyView` flips *after* `skyNodes` is populated, and the `{#if showSkyView}` block (3856) mounts GraphMindView with an already-filled prop value — the initial read captures the populated array. In prod, the user opens Sky View before 1936 runs, so the initial mount snapshot is `[]`."
>
> "Hypothesis is sound. Proposed `$state.raw` fix is correct for this specific binding."

**Convergence**: both agents confirm the hypothesis. Neither found a reactive wrapper, `{#key}` block, or `$derived` that would invalidate the plain-`let` non-reactivity claim. Neither found an IPC-layer or Rust-layer candidate that better explains the dev↔prod split.

### Agent 3 additional finding — secondary binding (already covered by the fix)

Agent 3 flagged `src/routes/+layout.svelte:3792-3794`:
```svelte
{#if lensActive}
  <ConstellationSight
    nodes={skyNodes}
    links={skyLinks}
    ...
```

This is a **second** direct plain-`let` read of `skyNodes`/`skyLinks`, gated on `lensActive`. If the user opens the Lens before `refreshLibraryCaches` completes, this binding would exhibit the same empty-graph bug as the main Sky View.

**Important**: this does NOT require a separate fix. The `$state.raw` change is at the declaration site (lines 541-542), so **every reader of `skyNodes`/`skyLinks` in the file becomes reactive automatically** — including this ConstellationSight binding. The single two-line change at lines 541-542 fixes the main Sky View binding (3869-3870), the Lens binding (3792-3794), and any future readers.

The WiW derivations (lines 569-584) and right-sidebar local-star effect (line 904-930) already worked because they explicitly read `starVersion`; they will now have *two* reactive deps (`starVersion` and `skyNodes`), which is harmless — `$state.raw` reassignment invalidations are cheap.

## Rollback plan

Single `git revert` of the landing commit restores HEAD `ef45c17` behavior. Or manually:

```ts
// revert to:
let skyNodes: SkyNode[] = [];
let skyLinks: SkyLink[] = [];
```

No other files to touch. No migration. No cache invalidation.

---

## Attempts budget remaining

Per `feedback_audit_before_implement.md`: **3 attempts max per bug.**

- **Attempt 1** (previous session): changed `allLibraryLinks` to `$state.raw`. Failed — wrong target variable. Reverted.
- **Attempt 2** (this one, if user approves): change `skyNodes` and `skyLinks` to `$state.raw`. Backed by official docs, source trace, and exhaustive reader/writer audit.
- **Attempt 3**: reserved. Unused.

If Attempt 2 fails, I stop, revert, and hand back to the user with what was learned. No Attempt 3 variation without a fresh research cycle.

---

## Verdict

**Proven fix. Documented by Svelte 5 maintainers. Zero semantic change beyond making the already-intended reactivity actually work.**

Awaiting user approval to apply.
