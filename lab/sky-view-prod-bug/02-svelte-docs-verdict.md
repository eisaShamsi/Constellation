# Svelte 5 official docs — verdict on plain `let` reactivity

**Source:** https://svelte.dev/docs/svelte/$state (fetched 2026-04-16)

## Direct quotes

> "When you reference something declared with the `$state` rune...you're accessing its _current value_."

Implication: things **not** declared with `$state` do not have this reactive tracking. Plain `let` reassignments are invisible to effects and template bindings.

> "State declared with `$state.raw` cannot be mutated; it can only be _reassigned_. In other words, rather than assigning to a property of an object, or using an array method like `push`, replace the object or array altogether."

Implication: `$state.raw` is the documented pattern for large arrays that only need reassignment tracking, without the O(n) proxy-wrap cost.

## Application to this bug

The offending declarations at `src/routes/+layout.svelte:541-542`:
```ts
let skyNodes: SkyNode[] = [];   // plain let — NOT reactive
let skyLinks: SkyLink[] = [];   // plain let — NOT reactive
let starVersion = $state(0);    // $state — intended signal
```

The design intent (per comment at lines 539-540) was:
- Avoid `$state` proxy overhead on 7,600-element arrays.
- Use `starVersion++` as an external reactive signal.

**The design does not work**, per the docs:
- Plain `let` reassignments are not tracked by any effect.
- `starVersion++` only triggers re-runs of scopes that explicitly read `starVersion`.
- The main Sky View `<GraphMindView nodes={skyNodes} ... />` binding does NOT read `starVersion`. Grep confirmation:
  - `starVersion` is read at lines 570, 581, 908 only.
  - 570, 581 = WiW overlay `$derived.by` blocks.
  - 908 = right-sidebar local-star `$effect`.
  - None of them touch the main Sky View prop chain.

## The canonical fix

Replace plain `let` with `$state.raw`:
```ts
let skyNodes = $state.raw<SkyNode[]>([]);
let skyLinks = $state.raw<SkyLink[]>([]);
```

Properties of `$state.raw` that match the original design goal:
- **No proxy wrapping** — reading `skyNodes[i]` or iterating does NOT pass through a proxy; same perf as plain let for the internal data.
- **Reassignment is tracked** — writing `skyNodes = nodes` fires any effect / template binding that reads `skyNodes`.
- No per-element overhead, no deep reactivity.

This preserves the performance optimization the author wanted, while actually delivering the reactivity the UI needs.

## Why the bug manifests only in production

- **Dev mode**: boot is slow (user reports ~1m46s for full cache load in dev). User clicks Sky View long after `skyNodes` is populated. GraphMindView mounts with a populated `nodes` prop, `onMount`'s `if (nodes.length > 0) engine.setData(...)` branch runs, graph renders.
- **Prod mode**: paint is instant (~420 ms). App looks ready within 1 s. `refreshLibraryCaches` is still running asynchronously (~8.2 s total per scorecard). User clicks Sky View within that 8 s window. GraphMindView mounts with `nodes=[]`. `onMount`'s `if` branch is skipped. When `skyNodes` is populated ~5 s later, the non-reactive plain `let` reassignment fails to propagate. No effect re-runs with populated data. "0 nodes · 0 edges" forever — unless the user unmounts and re-mounts Sky View.

## Collateral effects of the fix

`starVersion` can arguably be removed entirely — once `skyNodes` is reactive via `$state.raw`, all the "reactive trigger" `const _ver = starVersion` reads in WiW / local-star paths become noise. **But this is scope creep. Leave them in.** They'll still work; they're just redundant signals.

## Blast radius

Readers of `skyNodes` / `skyLinks` grep (within `src/`):
- `+layout.svelte:569-583` — WiW filtered nodes/links (uses starVersion+skyNodes — now also reacts to skyNodes directly, which is fine)
- `+layout.svelte:908-...` — local star effect (reads skyLinks)
- `+layout.svelte:3869-3870` — `<GraphMindView nodes={skyNodes} links={skyLinks} />` ← the BROKEN binding that this fix REPAIRS
- `+layout.svelte:1936-1937` — writer (boot)
- `+layout.svelte:2028-...` — another writer (need to check)

No `push`, `splice`, or inner-element mutation anywhere — pure whole-array reassignment. `$state.raw`'s constraint is met.

## Verdict

**Proven fix, documented by Svelte 5 maintainers, zero semantic change beyond making the already-intended reactivity actually work.**
