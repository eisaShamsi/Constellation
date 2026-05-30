<script lang="ts">
	/**
	 * MIG-065 §F.2 — the standalone `.base` file rendered as a full-tab table.
	 *
	 * The "Simple" default face of the Unified Progressive Base: an Obsidian-
	 * familiar editable-name table, fed by the SQL `execute_lens` engine (NOT the
	 * retiring `query_base` live-scan). A standalone `.base` file is just lens
	 * YAML on disk — identical to an inline ` ```base ` block's body — so we hand
	 * its text straight to `executeLens`.
	 *
	 * Column semantics (which columns, header labels, cell formatting) come from
	 * the shared `$lib/lens/tableModel` so this tab and the inline
	 * `LensBlockWidget._renderTable` can never drift (CLAUDE.md "secure the
	 * winning — one source of truth").
	 *
	 * §F.2 shipped the read-only familiar table. §G adds the tiered "+ Add
	 * column" picker (Your fields / Constellation) + remove-column + the
	 * `columns:` save path (`update_base_columns`). Filter/sort + resize/reorder
	 * are §G.2; edit-in-place is §H.
	 */
	import { invoke } from '@tauri-apps/api/core';
	import { t, dir } from '$lib/i18n';
	import { detectDir } from '$lib/utils';
	import {
		executeLens,
		updateBaseColumns,
		updateBaseOrder,
		updateNoteProperty,
		convertBase,
		type LensResult,
		type LensRow,
		type LensSort,
	} from '$lib/lens/store';
	import {
		dataColumns,
		columnLabel,
		renderCellValue,
		isSortable,
		isPropColumn,
		propKey,
	} from '$lib/lens/tableModel';
	import BaseColumnPicker from '$lib/lens/BaseColumnPicker.svelte';
	import BaseSortPanel from '$lib/lens/BaseSortPanel.svelte';

	let {
		path,
		content,
	}: {
		/** Absolute path of the `.base` file. The source of truth — BaseTab
		 *  (re-)reads it from disk keyed on this, so an in-tab column edit isn't
		 *  shadowed by the parent tab's cached `content`. */
		path: string;
		/** Raw `.base` YAML, loaded by `openNoteTab` — used only as an initial
		 *  seed (avoids a flash before the disk read returns). */
		content: string;
	} = $props();

	let result = $state<LensResult | null>(null);
	let error = $state<string | null>(null);
	let legacy = $state(false); // §J — an old-MVP JSON `.base` (calm notice, not a raw error)
	let converting = $state(false); // a legacy → new-format conversion in flight
	let convertError = $state<string | null>(null);
	let loading = $state(true);
	let pickerOpen = $state(false);
	let sortPanelOpen = $state(false);
	let saving = $state(false);
	let saveError = $state<string | null>(null);

	// The `.base` YAML, owned by this component. Seeded from the `content` prop
	// to avoid a flash, then (re)loaded from disk keyed on `path` — the file is
	// the source of truth (the parent tab's cached content can go stale after an
	// in-tab column edit). Add/remove-column sets this to the YAML returned by
	// `update_base_columns`, which re-runs the query below. Rule 2-safe: this
	// effect reads `path`, writes `baseYaml`/`loadedPath`, never reads `baseYaml`.
	let baseYaml = $state(content ?? '');
	let loadedPath: string | null = null;
	$effect(() => {
		if (path === loadedPath) return; // re-read only when the file changes
		loadedPath = path;
		invoke<string>('read_note', { filePath: path })
			.then((txt) => { baseYaml = txt; })
			.catch(() => { /* keep the seeded content */ });
	});

	// Evaluate the lens whenever the YAML changes. Rule 2-safe: depends on
	// `baseYaml`; writes only result/error/loading; `lastRun` is a plain var.
	let lastRun: string | null = null;
	$effect(() => {
		const y = baseYaml ?? '';
		if (!y.trim()) return; // wait for the disk read to populate baseYaml
		if (y === lastRun) return;
		lastRun = y;
		const reqYaml = y;
		loading = true;
		error = null;
		legacy = false;
		executeLens(reqYaml)
			.then((res) => {
				if (lastRun !== reqYaml) return; // a newer evaluation superseded this one
				result = res;
				error = null;
				legacy = false;
			})
			.catch((e: unknown) => {
				if (lastRun !== reqYaml) return;
				// §J — an old-MVP JSON `.base` (BaseDefinition) parses as YAML but
				// fails the LensDefinition shape. Show a calm "older format" notice
				// instead of a raw serde error (decision #1: old JSON ignored).
				legacy = reqYaml.trimStart().startsWith('{');
				error = legacy ? null : typeof e === 'string' ? e : (e as Error)?.message ?? String(e);
				result = null;
			})
			.finally(() => {
				if (lastRun === reqYaml) loading = false;
			});
	});

	const cols = $derived(result ? dataColumns(result.columns) : []);

	// ─── §G — add / remove column ───
	// Both compute the new ordered column list and persist via
	// `update_base_columns`, which returns the re-serialized YAML; assigning it
	// to `baseYaml` re-runs the query effect. `note.name` is never offered for
	// removal (it's the implicit clickable first column).
	async function persistColumns(columns: string[]) {
		saving = true;
		saveError = null;
		try {
			baseYaml = await updateBaseColumns(path, columns);
		} catch (e: unknown) {
			saveError = typeof e === 'string' ? e : (e as Error)?.message ?? String(e);
		} finally {
			saving = false;
		}
	}
	function addColumn(dim: string) {
		if (!result) return;
		persistColumns([...result.columns, dim]);
	}
	function removeColumn(dim: string) {
		if (!result || result.columns.length <= 1) return; // a base keeps ≥1 column
		persistColumns(result.columns.filter((c) => c !== dim));
	}

	// ─── §G.2 — sorting ───
	/** Current sort direction for a column, or null if it isn't a sort key. */
	function sortDir(dim: string): 'asc' | 'desc' | null {
		return result?.order.find((o) => o.dimension === dim)?.direction ?? null;
	}
	/** §G.2 — click a header: sort by it ascending → descending → off (single
	 *  sort; replaces any existing sort). Multi-column sort comes from the panel
	 *  (§G.2b). Non-sortable columns (path / headline) don't respond. */
	function cycleSort(dim: string) {
		if (!result || !isSortable(dim)) return;
		const cur = sortDir(dim);
		let next: LensSort[];
		if (cur === null) next = [{ dimension: dim, direction: 'asc' }];
		else if (cur === 'asc') next = [{ dimension: dim, direction: 'desc' }];
		else next = []; // desc → off
		persistOrder(next);
	}
	async function persistOrder(order: LensSort[]) {
		saving = true;
		saveError = null;
		try {
			baseYaml = await updateBaseOrder(path, order);
		} catch (e: unknown) {
			saveError = typeof e === 'string' ? e : (e as Error)?.message ?? String(e);
		} finally {
			saving = false;
		}
	}

	// ─── §H — edit-in-place (frontmatter `prop.*` cells only) ───
	// Only `prop.<key>` columns are editable (they map to a note's own
	// frontmatter); registered cognitive dimensions + the Name column are
	// read-only. A commit writes the note's frontmatter via `update_note_property`
	// (which also refreshes the index server-side) and optimistically updates the
	// one cell — no full re-query (Rule 3: never re-render thousands of rows for a
	// single edit).
	let editing = $state<{ rowPath: string; dim: string } | null>(null);
	let editValue = $state('');
	let editOptions = $state<string[]>([]); // dropdown options for a list-type cell (empty → free text)

	/** Distinct non-empty values of a column across the loaded rows. Returns []
	 *  for a high-cardinality field (>max distinct, e.g. title) so it stays a
	 *  free-text input rather than a giant dropdown. */
	function distinctValuesFor(dim: string, max: number): string[] {
		if (!result) return [];
		const set = new Set<string>();
		for (const r of result.rows) {
			const v = r.dimensions[dim];
			if (v === null || v === undefined || v === '') continue;
			set.add(String(v));
			if (set.size > max) return []; // too many distinct → free text, no dropdown
		}
		return [...set].sort();
	}

	// Canonical, RANK-ordered value sets for known cognitive frontmatter fields.
	// The edit dropdown lists these in rank order (NOT alphabetical) and offers
	// the FULL set even when a value isn't present in the loaded rows — so the
	// user can switch a note to any maturity, not just the ones already in use.
	// (Only fields whose canonical order is settled are listed; stage / stratum
	// join when the Cognitive-Engine columns land — see MIG-068.)
	const KNOWN_VALUE_SETS: Record<string, string[]> = {
		maturity: ['seed', 'sapling', 'evergreen', 'canonical']
	};

	/** Option list for editing a cell as a dropdown. A known cognitive field
	 *  (maturity) → its canonical rank-ordered set, plus any stray/legacy values
	 *  present in the data, plus the current value. An unknown small-set field →
	 *  its distinct data values. A high-cardinality field → [] (free text). The
	 *  current value is always included so it shows as the selected option. */
	function editOptionsFor(dim: string, current: string): string[] {
		if (!isPropColumn(dim)) return [];
		const canonical = KNOWN_VALUE_SETS[propKey(dim).toLowerCase()];
		const distinct = distinctValuesFor(dim, 50);
		if (canonical) {
			const out = [...canonical];
			for (const v of distinct) if (!out.includes(v)) out.push(v);
			if (current && !out.includes(current)) out.push(current);
			return out;
		}
		if (distinct.length === 0) return []; // high-cardinality → free text
		return current && !distinct.includes(current) ? [...distinct, current] : distinct;
	}

	function startEdit(row: LensRow, dim: string) {
		if (!isPropColumn(dim)) return; // cognitive / Name columns are read-only
		editing = { rowPath: row.note_path, dim };
		const v = row.dimensions[dim];
		editValue = v === null || v === undefined ? '' : String(v);
		editOptions = editOptionsFor(dim, editValue);
	}
	function cancelEdit() {
		editing = null;
	}
	async function commitEdit(row: LensRow, dim: string) {
		if (!editing) return;
		const next = editValue;
		editing = null;
		const cur = row.dimensions[dim];
		const curStr = cur === null || cur === undefined ? '' : String(cur);
		if (next === curStr) return; // unchanged — nothing to write
		saving = true;
		saveError = null;
		try {
			await updateNoteProperty(row.note_path, propKey(dim), next);
			row.dimensions[dim] = next; // optimistic; index already refreshed server-side
		} catch (e: unknown) {
			saveError = typeof e === 'string' ? e : (e as Error)?.message ?? String(e);
		} finally {
			saving = false;
		}
	}
	function onEditKey(e: KeyboardEvent, row: LensRow, dim: string) {
		if (e.key === 'Enter') {
			e.preventDefault();
			commitEdit(row, dim);
		} else if (e.key === 'Escape') {
			e.preventDefault();
			cancelEdit();
		}
	}
	/** Focus + select an edit input the moment it mounts. */
	function focusSelect(node: HTMLInputElement | HTMLSelectElement) {
		node.focus();
		if (node instanceof HTMLInputElement) node.select(); // text-select only applies to inputs
	}

	// ─── column reorder (pointer-drag a data-column header) ───
	// Tauri's WebView intercepts native HTML5 drag-and-drop (it reserves it for OS
	// file-drops), so element drag never reaches the page. We reorder with raw
	// pointer events instead: press-and-move past a small threshold = a drag;
	// press-without-move = a normal sort click (handled by the header's own
	// onclick). Hit-testing reads `data-col` on the header under the cursor.
	// Reuses the §G save path (`persistColumns` → update_base_columns) so the new
	// order persists. The Name column is fixed (first, clickable) and not movable.
	let dragCol = $state<string | null>(null); // column being dragged (null until threshold crossed)
	let dropCol = $state<string | null>(null); // header currently under the cursor
	let pressCol: string | null = null; // header pressed (candidate; non-reactive)
	let pressX = 0;
	let pressDragging = false;
	let suppressNextClick = false; // set after a drag so the trailing click doesn't sort
	const DRAG_THRESHOLD = 5;

	function headerColAt(x: number, y: number): string | null {
		const th = (document.elementFromPoint(x, y) as HTMLElement | null)?.closest?.(
			'th[data-col]'
		) as HTMLElement | null;
		return th?.getAttribute('data-col') ?? null;
	}
	function onHeaderPointerDown(e: MouseEvent, dim: string) {
		if (e.button !== 0) return; // left button only
		if ((e.target as HTMLElement).closest('.th-remove')) return; // let the × button work
		suppressNextClick = false; // clear any stale flag from a prior drag
		pressCol = dim;
		pressX = e.clientX;
		pressDragging = false;
		window.addEventListener('mousemove', onHeaderPointerMove);
		window.addEventListener('mouseup', onHeaderPointerUp);
	}
	function onHeaderPointerMove(e: MouseEvent) {
		if (pressCol == null) return;
		if (!pressDragging) {
			if (Math.abs(e.clientX - pressX) < DRAG_THRESHOLD) return;
			pressDragging = true;
			dragCol = pressCol; // now visibly dragging
		}
		dropCol = headerColAt(e.clientX, e.clientY);
	}
	function onHeaderPointerUp() {
		window.removeEventListener('mousemove', onHeaderPointerMove);
		window.removeEventListener('mouseup', onHeaderPointerUp);
		const wasDragging = pressDragging;
		const src = pressCol;
		const target = dropCol;
		pressCol = null;
		pressDragging = false;
		dragCol = null;
		dropCol = null;
		if (wasDragging) {
			suppressNextClick = true; // the click that follows this mouseup must not sort
			if (src && target && src !== target) moveColumn(src, target);
		}
	}
	function moveColumn(src: string, target: string) {
		if (!result || src === target) return;
		const data = dataColumns(result.columns);
		const from = data.indexOf(src);
		const to = data.indexOf(target);
		if (from < 0 || to < 0) return;
		const next = [...data];
		const [moved] = next.splice(from, 1);
		next.splice(to, 0, moved);
		// Keep note.name (the implicit first/Name column) declared up front.
		const newCols = result.columns.includes('note.name') ? ['note.name', ...next] : next;
		persistColumns(newCols);
	}
	// Safety net (Perf Rule 4): if the component unmounts mid-drag, drop the
	// window-level pointer listeners so they can't leak.
	$effect(() => {
		return () => {
			window.removeEventListener('mousemove', onHeaderPointerMove);
			window.removeEventListener('mouseup', onHeaderPointerUp);
		};
	});

	// ─── legacy → new-format conversion ───
	// On the user's explicit choice, upgrade an old-MVP `.base` (BaseDefinition
	// JSON) to the new LensDefinition YAML in place, then render it. Until they
	// click Convert the file is left untouched.
	async function convertLegacy() {
		converting = true;
		convertError = null;
		try {
			const yaml = await convertBase(path, true);
			baseYaml = yaml; // re-render as the converted base; the query effect re-runs
			legacy = false;
		} catch (e: unknown) {
			convertError = typeof e === 'string' ? e : (e as Error)?.message ?? String(e);
		} finally {
			converting = false;
		}
	}

	// MIG-065 — row virtualization (CLAUDE.md Performance Rule 3: render only the
	// rows on screen, regardless of total). Replaces the old 500-row cap — ALL
	// matching rows are now scrollable while the DOM stays small. (`execute_lens`
	// still returns every row over IPC; the engine-side LIMIT/COUNT split is a
	// separate PJ.) Rows are single-line (uniform height), so one measured
	// row-height drives exact top/bottom spacer math.
	let scrollEl: HTMLDivElement | undefined = $state();
	let scrollTop = $state(0);
	let viewportH = $state(600);
	let rowH = $state(32);
	const OVERSCAN = 12;
	const totalRows = $derived(result ? result.rows.length : 0);
	const startIndex = $derived(Math.max(0, Math.floor(scrollTop / rowH) - OVERSCAN));
	const endIndex = $derived(Math.min(totalRows, Math.ceil((scrollTop + viewportH) / rowH) + OVERSCAN));
	const windowRows = $derived(result ? result.rows.slice(startIndex, endIndex) : []);
	const topPad = $derived(startIndex * rowH);
	const botPad = $derived(Math.max(0, (totalRows - endIndex) * rowH));

	let rafPending = false;
	function onTableScroll() {
		if (rafPending) return;
		rafPending = true;
		requestAnimationFrame(() => {
			if (scrollEl) {
				scrollTop = scrollEl.scrollTop;
				viewportH = scrollEl.clientHeight;
			}
			rafPending = false;
		});
	}

	// Keep the viewport height in sync (mount + container resize). The $effect
	// cleanup disconnects the observer (Rule 4). Reads `scrollEl`; writes
	// `viewportH` (never read here) — Rule-2-safe.
	$effect(() => {
		const el = scrollEl;
		if (!el) return;
		viewportH = el.clientHeight;
		const ro = new ResizeObserver(() => {
			viewportH = el.clientHeight;
		});
		ro.observe(el);
		return () => ro.disconnect();
	});
	// Measure the actual (theme/font-dependent) row height once rows render.
	// Reads `result`/`scrollEl`; writes `rowH` (never read here) — Rule-2-safe.
	$effect(() => {
		if (!result || result.rows.length === 0 || !scrollEl) return;
		const r = scrollEl.querySelector('.base-trow') as HTMLElement | null;
		if (r && r.offsetHeight > 0) rowH = r.offsetHeight;
	});

	/** Open a row's note. Dispatches the same `constellation:open-note` event the
	 *  Boss-validated inline table uses (handled in `+layout.svelte`), so every
	 *  base/lens surface shares one open-note path. */
	function openNote(row: LensRow) {
		window.dispatchEvent(
			new CustomEvent('constellation:open-note', {
				detail: {
					path: row.note_path,
					libraryName: row.library_name,
					libraryPath: row.library_path,
				},
			}),
		);
	}
</script>

<div class="base-tab" dir={$dir} data-base-path={path}>
	{#if loading && !result}
		<div class="base-state">{$t('lensBlock.loading') || 'Loading…'}</div>
	{:else if legacy}
		<div class="base-state base-legacy">
			<p dir="auto">
				{$t('lensBlock.legacyBaseConvert') ||
					'This base is in an older format. Your file is untouched — convert it to a Constellation Base to view and edit it.'}
			</p>
			<button class="convert-btn" disabled={converting} onclick={convertLegacy}>
				{converting
					? $t('lensBlock.converting') || 'Converting…'
					: $t('lensBlock.convertBase') || 'Convert to Constellation Base'}
			</button>
			{#if convertError}
				<p class="base-save-error" dir="auto">{convertError}</p>
			{/if}
		</div>
	{:else if error}
		<div class="base-state base-error">
			<span class="base-error-label">{$t('lensBlock.errorLabel') || 'Base error'}</span>
			<span class="base-error-msg" dir="auto">{error}</span>
		</div>
	{:else if result}
		<div class="base-header">
			<h2 class="base-name" dir={detectDir(result.lens_name)}>{result.lens_name}</h2>
			<span class="base-count">{result.total_count}</span>
			<div class="base-actions">
				<div class="sort-wrap">
					<button
						class="action-btn"
						class:active={result.order.length > 0}
						disabled={saving}
						aria-haspopup="dialog"
						aria-expanded={sortPanelOpen}
						onclick={() => (sortPanelOpen = !sortPanelOpen)}
					>
						↕ {$t('lensBlock.sort') || 'Sort'}{result.order.length > 0 ? ` (${result.order.length})` : ''}
					</button>
					{#if sortPanelOpen}
						<BaseSortPanel
							order={result.order}
							columns={result.columns}
							onChange={persistOrder}
							onClose={() => (sortPanelOpen = false)}
						/>
					{/if}
				</div>
				<div class="add-col-wrap">
					<button
						class="action-btn"
						disabled={saving}
						aria-haspopup="dialog"
						aria-expanded={pickerOpen}
						onclick={() => (pickerOpen = !pickerOpen)}
					>
						+ {$t('lensBlock.addColumn') || 'Add column'}
					</button>
					{#if pickerOpen}
						<BaseColumnPicker
							currentColumns={result.columns}
							onAdd={addColumn}
							onClose={() => (pickerOpen = false)}
						/>
					{/if}
				</div>
			</div>
		</div>

		{#if result.rows.length === 0}
			<div class="base-state">{$t('lensBlock.empty') || 'No notes match this base.'}</div>
		{:else}
			<div class="base-table-scroll" bind:this={scrollEl} onscroll={onTableScroll}>
				<table class="base-table">
					<thead>
						<tr>
							<th class="th-name">
								<button
									class="th-sort can-sort"
									title={$t('lensBlock.sortBy') || 'Sort by this column'}
									onclick={() => cycleSort('note.name')}
								>
									<span class="th-label">{$t('lensBlock.colName') || 'Name'}</span>
									{#if sortDir('note.name')}
										<span class="th-arrow">{sortDir('note.name') === 'asc' ? '↑' : '↓'}</span>
									{/if}
								</button>
							</th>
							{#each cols as c (c)}
								<th
									class="th-data"
									class:drag-over={dropCol === c}
									class:dragging={dragCol === c}
									data-col={c}
									onmousedown={(e) => onHeaderPointerDown(e, c)}
								>
									<span class="th-inner">
										<span
											class="th-sort"
											class:can-sort={isSortable(c)}
											role="button"
											tabindex="0"
											title={isSortable(c) ? ($t('lensBlock.sortBy') || 'Sort by this column') : ''}
											onclick={() => {
												if (suppressNextClick) {
													suppressNextClick = false;
													return;
												}
												cycleSort(c);
											}}
											onkeydown={(e) => {
												if (e.key === 'Enter' || e.key === ' ') {
													e.preventDefault();
													cycleSort(c);
												}
											}}
										>
											<span class="th-label" dir="auto">{columnLabel(c, $t)}</span>
											{#if sortDir(c)}
												<span class="th-arrow">{sortDir(c) === 'asc' ? '↑' : '↓'}</span>
											{/if}
										</span>
										<button
											class="th-remove"
											title={$t('lensBlock.removeColumn') || 'Remove column'}
											aria-label={$t('lensBlock.removeColumn') || 'Remove column'}
											onclick={() => removeColumn(c)}
										>×</button>
									</span>
								</th>
							{/each}
						</tr>
					</thead>
					<tbody>
						{#if topPad > 0}
								<tr aria-hidden="true" class="v-spacer"><td colspan={cols.length + 1} style="height:{topPad}px"></td></tr>
							{/if}
							{#each windowRows as row (row.note_path)}
							<tr class="base-trow">
								<td class="cell-name" dir={detectDir(row.name)}>
									<button
										type="button"
										class="row-name"
										title={row.note_path}
										dir={detectDir(row.name)}
										onclick={() => openNote(row)}
									>
										{row.name}
									</button>
								</td>
								{#each cols as c (c)}
									{@const text = renderCellValue(row.dimensions[c], c)}
									{@const editable = isPropColumn(c)}
									<td
										class:editable-cell={editable}
										dir={text ? detectDir(text) : undefined}
										title={editable ? ($t('lensBlock.editCell') || 'Double-click to edit') : undefined}
										ondblclick={editable ? () => startEdit(row, c) : undefined}
									>
										{#if editing?.rowPath === row.note_path && editing?.dim === c}
											{#if editOptions.length}
												<select
													class="cell-edit cell-edit-select"
													dir="auto"
													bind:value={editValue}
													onchange={() => commitEdit(row, c)}
													onblur={() => commitEdit(row, c)}
													onkeydown={(e) => {
														if (e.key === 'Escape') {
															e.preventDefault();
															cancelEdit();
														}
													}}
													use:focusSelect
												>
													{#each editOptions as opt (opt)}
														<option value={opt}>{opt}</option>
													{/each}
												</select>
											{:else}
												<input
													class="cell-edit"
													dir="auto"
													bind:value={editValue}
													onblur={() => commitEdit(row, c)}
													onkeydown={(e) => onEditKey(e, row, c)}
													use:focusSelect
												/>
											{/if}
										{:else}
											{text}
										{/if}
									</td>
								{/each}
							</tr>
						{/each}
					{#if botPad > 0}
								<tr aria-hidden="true" class="v-spacer"><td colspan={cols.length + 1} style="height:{botPad}px"></td></tr>
							{/if}
						</tbody>
				</table>
			</div>
		{/if}

		<div class="base-footer">
			{#if saveError}
				<span class="base-save-error" dir="auto">{saveError}</span>
			{/if}
			<span class="base-time">{result.query_time_ms}ms</span>
		</div>
	{/if}
</div>

<style>
	.base-tab {
		height: 100%;
		display: flex;
		flex-direction: column;
		overflow: hidden;
		padding: 18px 24px;
		box-sizing: border-box;
	}

	.base-header {
		display: flex;
		align-items: center;
		gap: 10px;
		margin-bottom: 12px;
		flex-shrink: 0;
	}
	.base-name {
		margin: 0;
		font-size: 1.25rem;
		font-weight: 650;
		color: var(--text-normal);
	}
	.base-count {
		display: inline-flex;
		align-items: center;
		justify-content: center;
		min-width: 1.4em;
		height: 1.4em;
		padding: 0 0.5em;
		border-radius: 999px;
		background: var(--interactive-accent);
		color: #fff;
		font-size: 0.78rem;
		font-weight: 600;
		font-variant-numeric: tabular-nums;
	}

	/* §G/§G.2 — header action buttons (Sort · + Add column) + popover anchors */
	.base-actions {
		display: flex;
		align-items: center;
		gap: 8px;
		margin-inline-start: auto; /* push the action group to the trailing edge */
	}
	.sort-wrap,
	.add-col-wrap {
		position: relative;
	}
	.action-btn {
		background: none;
		border: 1px solid var(--background-modifier-border);
		border-radius: 6px;
		padding: 4px 11px;
		font: inherit;
		font-size: 0.82rem;
		color: var(--text-muted);
		cursor: pointer;
		white-space: nowrap;
	}
	.action-btn:hover:not(:disabled) {
		color: var(--interactive-accent);
		border-color: var(--interactive-accent);
	}
	.action-btn:disabled {
		opacity: 0.5;
		cursor: default;
	}
	.action-btn.active {
		color: var(--interactive-accent);
		border-color: var(--interactive-accent);
	}

	.base-table-scroll {
		flex: 1 1 auto;
		min-height: 0;
		overflow: auto;
	}
	.base-table {
		border-collapse: collapse;
		width: max-content;
		min-width: 100%;
		font-size: 0.9rem;
	}
	.base-table thead {
		position: sticky;
		top: 0;
		z-index: 1;
	}
	.base-table th {
		text-align: start;
		padding: 7px 12px;
		background: var(--background-secondary);
		border-bottom: 2px solid var(--background-modifier-border);
		color: var(--text-muted);
		font-weight: 600;
		font-size: 0.8rem;
		white-space: nowrap;
	}
	/* §G — column header with a hover remove (×) */
	.th-inner {
		display: inline-flex;
		align-items: center;
		gap: 6px;
	}
	/* §G.2 — click-to-sort header button + direction arrow */
	.th-sort {
		display: inline-flex;
		align-items: center;
		gap: 4px;
		background: none;
		border: none;
		padding: 0;
		margin: 0;
		font: inherit;
		color: inherit;
		cursor: default;
		text-align: start;
		white-space: nowrap;
	}
	.th-sort.can-sort {
		cursor: pointer;
	}
	.th-sort.can-sort:hover {
		color: var(--text-normal);
	}
	.th-arrow {
		color: var(--interactive-accent);
		font-size: 0.9em;
		font-weight: 700;
	}
	/* §K — pointer-drag a data-column header to reorder */
	.base-table th.th-data {
		cursor: grab;
	}
	.base-table th.th-data:active {
		cursor: grabbing;
	}
	.base-table th.dragging {
		opacity: 0.45;
		cursor: grabbing;
	}
	.base-table th.drag-over {
		background: var(--background-modifier-hover);
		box-shadow: inset 0 -2px 0 var(--interactive-accent);
	}
	.th-remove {
		flex-shrink: 0;
		background: none;
		border: none;
		padding: 0 2px;
		margin: 0;
		font: inherit;
		font-size: 1rem;
		line-height: 1;
		color: var(--text-faint);
		cursor: pointer;
		opacity: 0;
		transition: opacity 0.12s;
	}
	.base-table th:hover .th-remove {
		opacity: 1;
	}
	.th-remove:hover {
		color: var(--text-error, #e53e3e);
	}
	.base-table td {
		padding: 6px 12px;
		border-bottom: 1px solid var(--background-modifier-border);
		vertical-align: top;
		max-width: 360px;
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
		color: var(--text-normal);
	}
	.base-trow:hover td {
		background: var(--background-modifier-hover);
	}
	/* §H — editable frontmatter cells (prop.*); cognitive / Name cells stay static */
	.editable-cell {
		cursor: text;
	}
	.editable-cell:hover {
		box-shadow: inset 0 0 0 1px var(--interactive-accent);
	}
	.cell-edit {
		width: 100%;
		box-sizing: border-box;
		background: var(--background-primary);
		border: 1.5px solid var(--interactive-accent);
		border-radius: 4px;
		padding: 1px 5px;
		font: inherit;
		color: var(--text-normal);
		outline: none;
	}
	.cell-edit-select {
		cursor: pointer;
		padding-inline-end: 2px;
	}

	.cell-name {
		font-weight: 500;
	}
	.row-name {
		background: none;
		border: none;
		padding: 0;
		margin: 0;
		font: inherit;
		font-weight: 500;
		color: var(--text-normal);
		cursor: pointer;
		text-align: start;
		max-width: 100%;
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}
	.row-name:hover {
		color: var(--interactive-accent);
		text-decoration: underline;
		text-underline-offset: 2px;
	}

	.base-footer {
		flex-shrink: 0;
		padding-top: 8px;
		color: var(--text-faint);
		font-size: 0.75rem;
		display: flex;
		justify-content: space-between;
		align-items: baseline;
		gap: 12px;
	}
	/* §K — virtualization spacer rows: zero padding/border so the height is exact */
	.v-spacer td {
		padding: 0;
		border: 0;
	}
	.base-save-error {
		color: var(--text-error, #e53e3e);
	}

	.base-state {
		color: var(--text-muted);
		padding: 12px 0;
		font-size: 0.9rem;
	}
	.base-legacy {
		max-width: 48ch;
		line-height: 1.5;
	}
	.convert-btn {
		margin-top: 10px;
		background: var(--interactive-accent);
		color: #fff;
		border: none;
		border-radius: 6px;
		padding: 7px 14px;
		font: inherit;
		font-size: 0.86rem;
		font-weight: 600;
		cursor: pointer;
	}
	.convert-btn:hover:not(:disabled) {
		filter: brightness(1.08);
	}
	.convert-btn:disabled {
		opacity: 0.6;
		cursor: default;
	}
	.base-error {
		display: flex;
		flex-direction: column;
		gap: 4px;
	}
	.base-error-label {
		color: var(--text-error, #e53e3e);
		font-weight: 600;
	}
	.base-error-msg {
		color: var(--text-muted);
		font-family: var(--font-monospace, monospace);
		font-size: 0.82rem;
		white-space: pre-wrap;
	}
</style>
