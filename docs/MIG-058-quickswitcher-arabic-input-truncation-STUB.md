# MIG-058 — QuickSwitcher Arabic Input Truncation (STUB)

**Status:** Open — pending investigation.
**Opened:** 2026-05-27.
**Priority:** P2. Degrades the search UX for Arabic input but workarounds exist (paste, type-fast).

## The bug, in user terms

When typing an Arabic word in Constellation's QuickSwitcher (Ctrl+O), the input box truncates the text at normal typing speed. The full word only appears in the search box if the user **pastes it** or **types unusually fast**. Slow-typed Arabic characters get cut off.

### Canonical reproduction

- Press Ctrl+O.
- Type `الرباط` at a normal pace (one character every 200-300ms).
- Observed: the input shows `الربا` (or shorter) — last character(s) missing.
- Type the same word fast or paste it: shows the full `الرباط`.

## Suspected root cause

Svelte 5 `bind:value={query}` + `$effect` debounced async `constellationSearch()` + `$derived.by` `filtered` recomputation interacting badly with Arabic IME composition. Each keystroke updates `query`; `$effect` clears + sets a 300ms timeout; `filtered` `$derived` recomputes on every keystroke (iterates `notes` for substring match); the async `constellationSearch` may still be in-flight from the previous keystroke when the next one arrives.

Hypothesis A (frontend reactivity): when the result list re-renders (e.g., from a previous in-flight `constellationSearch` resolving), the input element's IME composition state gets disrupted; partial Arabic input is lost.

Hypothesis B (Tauri IPC queue): if an earlier `constellationSearch` is taking 20+ seconds (the slow cu1 branch issue tracked in MIG-059), subsequent IPC calls queue up; the frontend might be doing something that interferes with input responsiveness during the queue.

Hypothesis C (Svelte 5 binding race): `bind:value={query}` may not preserve partial Arabic composition state across re-renders triggered by external state changes (like `extendedResults = ...` after the search completes).

## Why this is its own MIG

Pre-existing — not caused by federation. The federation work just kept Eisa's attention on the search box long enough to notice. Fix is in `src/lib/components/QuickSwitcher.svelte` (and possibly the underlying `constellationSearch` IPC contract). Orthogonal to all the MIG-056 work.

## Proposed investigation path

1. Reproduce in isolation: open Constellation, Ctrl+O, type Arabic slowly. Confirm truncation.
2. Add a temporary console.log of `query` value on every input event vs every `$effect` fire — see if truncation happens at the input layer or downstream.
3. Test with `oninput` instead of `bind:value` to see if Svelte's two-way binding is the culprit.
4. Add `oncompositionstart` / `oncompositionend` IME handlers and gate the `$effect` debounce until composition ends.
5. Decouple the search debounce from re-rendering: store search results in a way that doesn't force the input's parent to re-render mid-keystroke.

## Verification clauses

- [ ] Type `الرباط` at 200-300ms intervals — full word lands in the input.
- [ ] Type a long English word slowly — no regression.
- [ ] Type Chinese / Japanese (IME-composed input) — no regression.
- [ ] Search results still update within ~500ms of stopping typing.
- [ ] No new IPC calls on every keystroke (only after debounce).

## Related

- Surfaced during MIG-056 §K Boss-test when Eisa repeatedly typed `الرباط` but the diag log captured `الربا` as the query string — proving the truncation happens at the input layer, not in Rust.
- File: `src/lib/components/QuickSwitcher.svelte`.
- Possibly related: the boot-time `loadAllStats` lifecycle pattern, but that one's fixed now.
