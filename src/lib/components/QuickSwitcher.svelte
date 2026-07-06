<script lang="ts">
	import { onMount } from 'svelte';
	import { t } from '$lib/i18n';
	import { invoke } from '@tauri-apps/api/core';
	import { constellationSearch, parseSearchQuery } from '$lib/libraries/store';

	let {
		notes = [] as { name: string; path: string; libraryName: string }[],
		onSelect,
		onClose,
	}: {
		notes: { name: string; path: string; libraryName: string }[];
		onSelect: (path: string, libraryName: string) => void;
		onClose: () => void;
	} = $props();

	let query = $state('');
	let selectedIndex = $state(0);
	let inputEl: HTMLInputElement;
	let searchTimer: ReturnType<typeof setTimeout>;

	// MIG-058 fix — `filtered` is now $state, not $derived.
	//
	// PROBLEM (pre-MIG-058): `filtered` was a `$derived.by(() => ...)`
	// that read `query`, `notes`, and `extendedResults`. Every keystroke
	// triggered a synchronous rebuild: walk `notes` (1101+ rows), lower-
	// case + substring-match each, then merge with `extendedResults`.
	// That work happened on the keystroke's tick, on the main thread —
	// before the next paint. The keyed `{#each filtered ... (note.path)}`
	// then re-rendered the list on every keystroke. Under WebView2 main-
	// thread pressure, this caused Arabic keystrokes to drop at slow
	// typing speeds (300-400ms/char) — Eisa's MIG-058 truncation report.
	//
	// FIX: `filtered` is now a plain `$state` array updated by the
	// debounced effect below. Typing no longer triggers a synchronous
	// filter/re-key cycle; the list updates only once per 300ms debounce
	// window. This eliminates the per-keystroke main-thread pressure
	// that the QuickSwitcher Arabic-input research traced as the
	// most-likely cause.
	//
	// References:
	// - Svelte 5 `bind_value` source guarantees the focused input is
	//   never rewritten by reactive updates, so the prior hypothesis
	//   ("value-rewrite breaks IME composition") was wrong — confirmed
	//   by the bind_value source: github.com/sveltejs/svelte/blob/main/
	//   packages/svelte/src/internal/client/dom/elements/bindings/input.js
	// - Arabic 101 (Windows) is a direct keyboard layout, NOT an IME,
	//   so composition events never fire for Arabic — the "gate on
	//   compositionstart/end" pattern from React #34485 / Vue v-model
	//   doesn't apply here (kbdlayout.info/KBDA1).
	// - The actual cause is synchronous main-thread pressure from
	//   `$derived` + keyed list re-renders on every keystroke, per
	//   CodeMirror discuss #9741 and Tauri discussion #3136.
	let filtered = $state<{ name: string; path: string; libraryName: string }[]>([]);

	// MIG-058 — `composing` flag for CJK / Indic / any IME-composed
	// input. Arabic 101 doesn't trigger composition events, but this
	// is cheap insurance for users typing Chinese / Japanese / Korean
	// / Hindi / Vietnamese / etc. While composing, we skip the debounced
	// effect entirely so the in-progress composition isn't disturbed
	// by any state churn. The Vue v-model pattern, applied to Svelte 5.
	let composing = $state(false);

	// ── QS-speed reproduce-first instrumentation (2026-07-06, TEMPORARY) ──
	// The Boss reports 2+ minute result retrieval with heavy thrashing.
	// Devtools is OFF in release builds, so the trace renders IN the panel:
	// per-run local-filter ms, the Rust await wall ms, and the Rust-side
	// phase trace (db-lock wait / per-branch federated ms) fetched after
	// each search. `runSeq` counts searches issued since the modal opened
	// (a pile-up indicator: several in-flight 2-minute searches serialize
	// on the DB lock → the thrash signature).
	let diag = $state('');
	let runSeq = 0;

	// MIG-058 — initial filtered state when the modal opens with empty
	// query: show the top 30 of the cached notes (existing behavior).
	$effect(() => {
		if (!query.trim()) {
			filtered = notes.slice(0, 30);
		}
	});

	// Combined search effect: substring-filter the cached `notes` AND
	// fetch federated results from Rust, both in the same debounced
	// timer. Typing no longer rebuilds `filtered` synchronously.
	//
	// MIG-058 — also resets selectedIndex INSIDE this effect rather
	// than via a separate $effect on `filtered`. The old separate
	// effect fired EVERY time `filtered` updated, including from the
	// async resolve, which contributed to mid-typing reactive churn.
	//
	// MIG-058/MIG-059 Option E — stale-result-discard guard. When the
	// async `constellationSearch` is slow (10+s on cold federated
	// FTS5), the user may type more characters while the previous
	// search is still in-flight. When the stale result resolves, its
	// reactive cascade (filtered = ..., selectedIndex = 0) would fire
	// even though the result is for the OLD query — causing visible
	// flicker and reactive churn that contributes to the perceived
	// Arabic input truncation. The guard `if (q !== query) return`
	// drops stale results before they touch state. Same pattern as
	// Solr `newSearcher` swap-in skip when the searcher is obsolete.
	$effect(() => {
		const q = query;
		clearTimeout(searchTimer);
		if (composing) return;       // MIG-058 — skip during IME composition
		if (!q.trim()) return;
		searchTimer = setTimeout(async () => {
			// QS-speed instrumentation — time each phase of this run.
			const myRun = ++runSeq;
			const t0 = performance.now();
			// 1) Local substring filter against the cached notes array.
			const qLower = q.toLowerCase();
			const local = notes
				.filter(n => n.name.toLowerCase().includes(qLower) || n.path.toLowerCase().includes(qLower))
				.slice(0, 20);
			const localMs = performance.now() - t0;
			diag = `run #${myRun} · local ${localMs.toFixed(1)}ms (${local.length} hits) · rust search running…`;
			let next = local;
			let rustMs = 0;
			let rustTrace = '';
			// 2) For queries ≥ 3 chars, augment with federated Rust results.
			if (q.trim().length >= 3) {
				try {
					const req = parseSearchQuery(q);
					req.limit = 15;
					const tR = performance.now();
					const results = await constellationSearch(req);
					rustMs = performance.now() - tR;
					try {
						const tr = await invoke<[string, number][]>('get_last_search_trace');
						rustTrace = tr.map(([k, v]) => `${k} ${v >= 1000 ? (v / 1000).toFixed(1) + 's' : v.toFixed(0) + 'ms'}`).join(' · ');
					} catch { /* trace unavailable */ }
					// Option E — discard if user has moved on since search start.
					if (q !== query) {
						diag = `run #${myRun} STALE (typed on) · local ${localMs.toFixed(1)}ms · rust ${(rustMs / 1000).toFixed(1)}s [${rustTrace}]`;
						return;
					}
					const seen = new Set(local.map(n => n.path));
					const merged = [...local];
					for (const r of results) {
						if (!seen.has(r.path)) {
							merged.push({ name: r.name, path: r.path, libraryName: r.library_name });
							seen.add(r.path);
						}
					}
					next = merged;
				} catch {
					// federation/search error — fall through with local-only results
				}
			}
			// Option E — final guard before reactive write. Even the
			// local-filter-only path can be stale if the user pivoted
			// to a different query while the setTimeout was queued.
			if (q !== query) return;
			filtered = next.slice(0, 30);
			selectedIndex = 0;
			// QS-speed instrumentation — the run's full breakdown, on-screen.
			diag = rustMs > 0
				? `run #${myRun} · local ${localMs.toFixed(1)}ms (${local.length}) · rust ${rustMs >= 1000 ? (rustMs / 1000).toFixed(1) + 's' : rustMs.toFixed(0) + 'ms'} [${rustTrace}]`
				: `run #${myRun} · local ${localMs.toFixed(1)}ms (${local.length}) · no rust search (<3 chars)`;
		}, 300);
	});

	onMount(() => {
		inputEl?.focus();
	});

	function handleKeydown(e: KeyboardEvent) {
		if (e.key === 'Escape') {
			onClose();
		} else if (e.key === 'ArrowDown') {
			e.preventDefault();
			selectedIndex = Math.min(selectedIndex + 1, filtered.length - 1);
		} else if (e.key === 'ArrowUp') {
			e.preventDefault();
			selectedIndex = Math.max(selectedIndex - 1, 0);
		} else if (e.key === 'Enter') {
			e.preventDefault();
			if (filtered[selectedIndex]) {
				onSelect(filtered[selectedIndex].path, filtered[selectedIndex].libraryName);
				onClose();
			}
		}
	}
</script>

<!-- svelte-ignore a11y_no_static_element_interactions -->
<div class="qs-overlay" onclick={onClose} onkeydown={handleKeydown}>
	<!-- svelte-ignore a11y_click_events_have_key_events -->
	<!-- svelte-ignore a11y_no_static_element_interactions -->
	<div class="qs-panel" onclick={(e) => e.stopPropagation()}>
		<input
			bind:this={inputEl}
			type="text"
			class="qs-input"
			placeholder={$t('quickSwitcher.placeholder')}
			bind:value={query}
			onkeydown={handleKeydown}
			oncompositionstart={() => composing = true}
			oncompositionend={() => composing = false}
		/>
		<div class="qs-list">
			{#each filtered as note, i (note.path)}
				<button
					class="qs-item"
					class:selected={i === selectedIndex}
					onclick={() => { onSelect(note.path, note.libraryName); onClose(); }}
					onmouseenter={() => selectedIndex = i}
				>
					<span class="qs-name">{note.name}</span>
					<span class="qs-path">{note.libraryName}</span>
				</button>
			{/each}
			{#if filtered.length === 0 && query}
				<div class="qs-empty">{$t('quickSwitcher.noResults')}</div>
			{/if}
		</div>
		{#if diag}
			<!-- QS-speed reproduce-first instrumentation (TEMPORARY) -->
			<div class="qs-diag" dir="ltr">{diag}</div>
		{/if}
	</div>
</div>

<style>
	.qs-overlay {
		position: fixed; inset: 0; z-index: 1000;
		background: var(--background-modifier-cover);
		display: flex; justify-content: center; padding-top: 15vh;
	}
	.qs-panel {
		background: var(--background-primary); border-radius: 8px;
		box-shadow: var(--shadow-l);
		width: 500px; max-height: 400px;
		display: flex; flex-direction: column;
		overflow: hidden;
	}
	.qs-input {
		border: none; padding: 12px 16px;
		font-size: 0.95rem; font-family: inherit;
		color: var(--text-normal); outline: none;
		background: var(--background-primary);
		border-bottom: 1px solid var(--background-modifier-border-focus);
	}
	.qs-input::placeholder { color: var(--color-base-40); }
	.qs-list { flex: 1; overflow-y: auto; padding: 4px; }
	.qs-item {
		display: flex; align-items: center; justify-content: space-between;
		width: 100%; padding: 6px 12px;
		background: none; border: none; border-radius: 4px;
		cursor: pointer; font-family: inherit; text-align: start;
		color: var(--text-normal); font-size: 0.85rem;
	}
	.qs-item.selected { background: var(--interactive-accent); color: var(--text-on-accent); }
	.qs-name { font-weight: 500; }
	.qs-path { font-size: 0.72rem; color: var(--text-faint); }
	.qs-item.selected .qs-path { color: rgba(255,255,255,0.7); }
	.qs-empty { padding: 16px; text-align: center; color: var(--text-faint); font-size: 0.85rem; }
	/* QS-speed instrumentation (TEMPORARY) */
	.qs-diag {
		padding: 4px 12px; font-size: 0.68rem; font-family: var(--font-monospace, monospace);
		color: var(--text-faint); border-top: 1px solid var(--background-modifier-border);
		white-space: normal; word-break: break-all;
	}
</style>
