<script lang="ts">
	import { t } from '$lib/i18n';

	interface LinkedNote { name: string; path: string; depth: number; stratum: number; }
	interface Note360View {
		note_path: string; note_name: string; word_count: number;
		typed_links: Record<string, LinkedNote[]>;
		untyped_links: LinkedNote[];
		total_outbound: number; total_inbound: number;
		stratum: number; maturity: string;
		contradictions: string[]; is_orphan: boolean; single_point_of_failure: boolean;
		origin_type: string; trust_depth: number;
		stage: string; last_reviewed: string | null; is_due: boolean;
		trails: string[]; lens_groups: string[];
		missing_link_types: string[]; used_link_types: string[];
	}

	let {
		data = null as Note360View | null,
		compact = false,
		onNoteClick,
		onClose,
		previousNoteName = null,
		onBack,
	}: {
		data?: Note360View | null;
		compact?: boolean;
		onNoteClick?: (path: string, name: string) => void;
		onClose?: () => void;
		previousNoteName?: string | null;
		onBack?: () => void;
	} = $props();

	// §112: Stratification Matrix replaces the spherical/angular line.
	// Vertical axis = stratum (1..8, displayed top-down 8→1). Horizontal axis
	// = link direction (7 typed + 1 untyped). Each cell holds the connected
	// notes whose stratum matches the row, drawn as small dots at the typed-
	// direction column. Empty cells are visually present (dashed stripes) so
	// gaps read as a first-class signal — see 360.3D Concept Paper §4.3.

	const TYPE_ORDER = ['supports', 'contradicts', 'causes', 'derives-from', 'generalizes', 'exemplifies', 'part-of', 'untyped'] as const;
	type LinkType = typeof TYPE_ORDER[number];

	const TYPE_COLORS: Record<LinkType, string> = {
		supports: '#4A9EFF',
		contradicts: '#FF4A4A',
		causes: '#FF8C42',
		'derives-from': '#FFD700',
		generalizes: '#A44AFF',
		exemplifies: '#4AFF88',
		'part-of': '#AAAAAA',
		untyped: '#888888',
	};

	const TYPE_LABEL_KEYS: Record<LinkType, string> = {
		supports: 'link_supports',
		contradicts: 'link_contradicts',
		causes: 'link_causes',
		'derives-from': 'link_derives_from',
		generalizes: 'link_generalizes',
		exemplifies: 'link_exemplifies',
		'part-of': 'link_part_of',
		untyped: 'untyped',
	};

	// Stratum order: highest (Worldview) at top, lowest (Datum) at bottom — so
	// the user reads "altitude" from top-down naturally.
	const STRATA = [8, 7, 6, 5, 4, 3, 2, 1] as const;

	const STRATUM_NAMES: Record<number, string> = {
		1: 'Datum',
		2: 'Information',
		3: 'Proposition',
		4: 'Concept',
		5: 'Principle',
		6: 'Theory',
		7: 'Paradigm',
		8: 'Worldview',
	};

	const MATURITY_COLORS: Record<string, string> = {
		seed: '#9ca3af', sapling: '#4ade80', evergreen: '#16a34a', canonical: '#f59e0b', wilting: '#16a34a80',
	};
	const ORIGIN_COLORS: Record<string, string> = {
		received: '#4A9EFF', discovered: '#FFB347', mixed: '#A78BFA', none: '#9ca3af',
	};
	const STAGE_ICONS: Record<string, string> = {
		fleeting: '\u{1F331}', literature: '\u{1F4D6}', permanent: '\u{1F517}', synthesis: '✨',
	};

	function truncName(name: string, max: number): string {
		return name.length > max ? name.slice(0, max - 1) + '…' : name;
	}

	function clampStratum(n: number | undefined): number {
		const v = Math.round(n ?? 0);
		if (v < 1) return 1;
		if (v > 8) return 8;
		return v;
	}

	const activeStratum = $derived(data ? clampStratum(data.stratum) : 1);

	// (stratum, type) → notes matrix.
	// Dedupe per cell by path so the same neighbour returned by outbound +
	// inbound + 2nd-order doesn't render multiple dots in the same cell.
	const matrix = $derived.by(() => {
		if (!data) return null;

		const cells: Record<number, Record<LinkType, LinkedNote[]>> = {};
		for (const s of STRATA) {
			cells[s] = { supports: [], contradicts: [], causes: [], 'derives-from': [], generalizes: [], exemplifies: [], 'part-of': [], untyped: [] };
		}

		const seen = new Set<string>();
		const place = (note: LinkedNote, type: LinkType) => {
			const stratum = clampStratum(note.stratum);
			const key = `${type}|${stratum}|${note.path || note.name}`;
			if (seen.has(key)) return;
			seen.add(key);
			cells[stratum][type].push(note);
		};

		for (const [type, notes] of Object.entries(data.typed_links)) {
			if (!(TYPE_ORDER as readonly string[]).includes(type)) continue;
			for (const n of notes) place(n, type as LinkType);
		}
		for (const n of data.untyped_links) place(n, 'untyped');

		const colTotals: Record<LinkType, number> = { supports: 0, contradicts: 0, causes: 0, 'derives-from': 0, generalizes: 0, exemplifies: 0, 'part-of': 0, untyped: 0 };
		const rowTotals: Record<number, number> = { 1: 0, 2: 0, 3: 0, 4: 0, 5: 0, 6: 0, 7: 0, 8: 0 };
		for (const s of STRATA) {
			for (const t of TYPE_ORDER) {
				const c = cells[s][t].length;
				colTotals[t] += c;
				rowTotals[s] += c;
			}
		}

		return { cells, colTotals, rowTotals };
	});

	// Compact scorecard bars: per-type counts normalized to the biggest one.
	const compactBars = $derived.by(() => {
		if (!data) return null;
		const counts: Record<LinkType, number> = { supports: 0, contradicts: 0, causes: 0, 'derives-from': 0, generalizes: 0, exemplifies: 0, 'part-of': 0, untyped: 0 };
		for (const [type, notes] of Object.entries(data.typed_links)) {
			if (!(TYPE_ORDER as readonly string[]).includes(type)) continue;
			counts[type as LinkType] = notes.length;
		}
		counts.untyped = data.untyped_links.length;
		let max = 0;
		for (const t of TYPE_ORDER) if (counts[t] > max) max = counts[t];
		return { counts, max };
	});

	// Hover shows the neighbour's name in a fixed top-right tooltip — doesn't
	// follow the mouse and doesn't pop arbitrary chrome on dense rows.
	let hoveredName = $state<string | null>(null);

	const MAX_DOTS_PER_CELL = 16;
</script>

{#if compact}
	<!-- ===== COMPACT SIDEBAR — scorecard ===== -->
	<div class="i360 compact">
		{#if previousNoteName && onBack}
			<button class="i360-back-bar" onclick={onBack} title={`Back to ${previousNoteName}`}>
				<span class="i360-back-arrow">{'←'}</span>
				<span class="i360-back-name" dir="auto">{truncName(previousNoteName, 22)}</span>
			</button>
		{/if}
		{#if !data || !compactBars}
			<div class="i360-empty">
				<div class="i360-empty-icon">{'\u{1F52E}'}</div>
				<div class="i360-empty-text">{$t('inspector360.noData') || 'Open a note to see its 360° view'}</div>
			</div>
		{:else}
			<div class="i360-card">
				<div class="i360-card-name" dir="auto">{truncName(data.note_name, 28)}</div>
				<div class="i360-card-meta">
					<span class="i360-stratum-pill">L{activeStratum} {STRATUM_NAMES[activeStratum]}</span>
					<span class="i360-pill" style="background: color-mix(in srgb, {MATURITY_COLORS[data.maturity] ?? '#999'} 18%, transparent); color: {MATURITY_COLORS[data.maturity] ?? '#999'}">{data.maturity}</span>
					{#if data.stage}<span class="i360-pill-soft">{STAGE_ICONS[data.stage] || ''} {data.stage}</span>{/if}
				</div>
				<div class="i360-card-counts">
					<span>{'⬆'} {data.total_outbound}</span>
					<span>{'⬇'} {data.total_inbound}</span>
					<span>{'\u{1F4DD}'} {data.word_count.toLocaleString()}</span>
				</div>
				<div class="i360-bars">
					{#each TYPE_ORDER as type}
						{@const count = compactBars.counts[type]}
						{@const pct = compactBars.max > 0 ? (count / compactBars.max) * 100 : 0}
						<div class="i360-bar-row" class:gap-row={count === 0}>
							<span class="i360-bar-label">{$t(`inspector360.${TYPE_LABEL_KEYS[type]}`) || type}</span>
							<div class="i360-bar-track">
								<div class="i360-bar-fill" style="width: {pct}%; background: {TYPE_COLORS[type]}"></div>
							</div>
							<span class="i360-bar-count">{count || '—'}</span>
						</div>
					{/each}
				</div>
				<div class="i360-card-flags">
					{#if data.is_orphan}<span class="i360-warn">{'⚠'} {$t('inspector360.orphan') || 'Orphan'}</span>{/if}
					{#if data.single_point_of_failure}<span class="i360-warn">{'⚠'} {$t('inspector360.fragile') || 'Fragile'}</span>{/if}
					{#if data.missing_link_types.length > 0}<span class="i360-warn">{'⚠'} {data.missing_link_types.length} {$t('inspector360.gaps') || 'gaps'}</span>{/if}
					{#if data.is_due}<span class="i360-warn">{'\u{1F4CB}'} {$t('inspector360.dueForReview') || 'Review due'}</span>{/if}
				</div>
			</div>
		{/if}
	</div>
{:else}
	<!-- ===== FULL-WINDOW — Stratification Matrix ===== -->
	<div class="i360-full">
		{#if !data || !matrix}
			<div class="i360-empty-full">
				<div class="i360-empty-icon-lg">{'\u{1F52E}'}</div>
				<div class="i360-empty-text-lg">{$t('inspector360.noData') || 'Open a note to see its 360° view'}</div>
			</div>
		{:else}
			<!-- Header -->
			<div class="i360-header">
				<div class="i360-header-left">
					{#if previousNoteName && onBack}
						<button class="i360-back-full" onclick={onBack} title={`Return to ${previousNoteName}`}>
							<span class="i360-back-arrow">{'←'}</span>
							<span class="i360-back-name" dir="auto">{truncName(previousNoteName, 24)}</span>
						</button>
					{/if}
					<span class="i360-header-icon">{'\u{1F9E0}'}</span>
					<span class="i360-header-label">{$t('inspector360.title') || '360.3D'}</span>
					<span class="i360-header-name" dir="auto">{data.note_name}</span>
				</div>
				<div class="i360-header-right">
					{#if onClose}
						<button class="i360-close" onclick={onClose} title="Close">{'×'}</button>
					{/if}
				</div>
			</div>

			<!-- Non-spatial dimension strip -->
			<div class="i360-strip">
				<div class="i360-strip-cell">
					<span class="i360-strip-label">Stratum</span>
					<span class="i360-strip-value" style="color: #a78bfa">L{activeStratum} {STRATUM_NAMES[activeStratum]}</span>
				</div>
				<div class="i360-strip-cell">
					<span class="i360-strip-label">Maturity</span>
					<span class="i360-strip-value"><span class="i360-dot" style="background: {MATURITY_COLORS[data.maturity] ?? '#999'}"></span>{data.maturity}</span>
				</div>
				<div class="i360-strip-cell">
					<span class="i360-strip-label">Origin</span>
					<span class="i360-strip-value"><span class="i360-dot" style="background: {ORIGIN_COLORS[data.origin_type] ?? '#999'}"></span>{data.origin_type} {'·'} {$t('inspector360.depth') || 'd'}{data.trust_depth}</span>
				</div>
				<div class="i360-strip-cell">
					<span class="i360-strip-label">Stage</span>
					<span class="i360-strip-value">{STAGE_ICONS[data.stage] || ''} {data.stage || 'none'}</span>
				</div>
				<div class="i360-strip-cell">
					<span class="i360-strip-label">Review</span>
					<span class="i360-strip-value">
						{#if data.is_due}<span class="i360-warn">Due</span>{:else if data.last_reviewed}{data.last_reviewed.slice(0, 10)}{:else}{'—'}{/if}
					</span>
				</div>
				{#if data.trails.length > 0}
					<div class="i360-strip-cell">
						<span class="i360-strip-label">Trails</span>
						<span class="i360-strip-value">{'\u{1F6E4}️'} {data.trails.length}</span>
					</div>
				{/if}
				{#if data.lens_groups.length > 0}
					<div class="i360-strip-cell">
						<span class="i360-strip-label">Lenses</span>
						<span class="i360-strip-value">{'\u{1F3F7}️'} {data.lens_groups.length}</span>
					</div>
				{/if}
			</div>

			<!-- Matrix canvas -->
			<div class="i360-canvas">
				<div class="i360-matrix-wrap">
					<div class="i360-matrix">
						<!-- Header row: corner + 8 column headers + row-total header -->
						<div class="i360-corner">
							<span class="i360-corner-stratum">{'▲'} Stratum</span>
							<span class="i360-corner-type">Type {'→'}</span>
						</div>
						{#each TYPE_ORDER as type}
							<div class="i360-col-header" style="--col-color: {TYPE_COLORS[type]}">
								<div class="i360-col-name">{(($t(`inspector360.${TYPE_LABEL_KEYS[type]}`) || type)).toUpperCase()}</div>
								<div class="i360-col-count">{matrix.colTotals[type]}</div>
							</div>
						{/each}
						<div class="i360-rowtot-header">{'Σ'}</div>

						<!-- Data rows -->
						{#each STRATA as stratum}
							{@const isActive = activeStratum === stratum}
							{@const isEmptyRow = matrix.rowTotals[stratum] === 0}
							<div class="i360-row-header" class:active={isActive} class:empty-row={isEmptyRow && !isActive}>
								<span class="i360-row-num">L{stratum}</span>
								<span class="i360-row-name">{STRATUM_NAMES[stratum]}</span>
								{#if isActive}
									<span class="i360-active-chip" dir="auto" title={data.note_name}>{truncName(data.note_name, 16)}</span>
								{/if}
							</div>
							{#each TYPE_ORDER as type}
								{@const cellNotes = matrix.cells[stratum][type]}
								{@const cellEmpty = cellNotes.length === 0}
								<div class="i360-cell"
									class:active-row={isActive}
									class:empty-cell={cellEmpty}
									style="--col-color: {TYPE_COLORS[type]}">
									{#each cellNotes.slice(0, MAX_DOTS_PER_CELL) as note}
										<button class="i360-dot-btn"
											style="--dot-color: {TYPE_COLORS[type]}"
											aria-label={note.name}
											onmouseenter={() => hoveredName = note.name}
											onmouseleave={() => hoveredName = null}
											onclick={() => onNoteClick?.(note.path, note.name)}>
										</button>
									{/each}
									{#if cellNotes.length > MAX_DOTS_PER_CELL}
										<span class="i360-overflow" title={`${cellNotes.length - MAX_DOTS_PER_CELL} more`}>+{cellNotes.length - MAX_DOTS_PER_CELL}</span>
									{/if}
								</div>
							{/each}
							<div class="i360-rowtot" class:active={isActive}>{matrix.rowTotals[stratum]}</div>
						{/each}
					</div>

					<!-- Floating hover label (fixed top-right of canvas) -->
					{#if hoveredName}
						<div class="i360-hover-label" dir="auto">{hoveredName}</div>
					{/if}
				</div>
			</div>

			<!-- Bottom HUD -->
			<div class="i360-hud">
				<div class="i360-hud-left">
					<span class="i360-hud-item">{'⬆'} {data.total_outbound} {$t('inspector360.outbound') || 'outbound'}</span>
					<span class="i360-hud-item">{'⬇'} {data.total_inbound} {$t('inspector360.inbound') || 'inbound'}</span>
					<span class="i360-hud-item">{'\u{1F4DD}'} {data.word_count.toLocaleString()} {$t('inspector360.words') || 'words'}</span>
				</div>
				<div class="i360-hud-right">
					{#if data.is_orphan}<span class="i360-hud-item i360-hud-warn">{'⚠'} {$t('inspector360.orphan') || 'Orphan'}</span>{/if}
					{#if data.single_point_of_failure}<span class="i360-hud-item i360-hud-warn">{'⚠'} {$t('inspector360.fragile') || 'Fragile'}</span>{/if}
					{#if data.missing_link_types.length > 0}<span class="i360-hud-item i360-hud-warn">{'⚠'} {data.missing_link_types.length} {$t('inspector360.blindSpots') || 'blind spots'}</span>{/if}
					{#if data.contradictions.length > 0}<span class="i360-hud-item i360-hud-warn">{'⚡'} {data.contradictions.length} {$t('inspector360.tensions') || 'tensions'}</span>{/if}
				</div>
			</div>
		{/if}
	</div>
{/if}

<style>
	/* ===== COMPACT SIDEBAR ===== */
	.i360.compact { display: flex; flex-direction: column; padding: 8px; gap: 8px; }
	.i360-back-bar {
		display: flex; align-items: center; gap: 6px;
		padding: 4px 8px;
		background: rgba(127,127,127,0.06);
		border: 1px solid rgba(127,127,127,0.18);
		border-radius: 6px;
		color: var(--text-muted, #888);
		font-size: 0.78rem;
		cursor: pointer;
		text-align: start;
	}
	.i360-back-bar:hover { background: rgba(127,127,127,0.14); color: var(--text, inherit); }
	.i360-back-arrow { font-size: 0.9rem; line-height: 1; flex-shrink: 0; }
	.i360-back-name { flex: 1; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
	.i360-empty { text-align: center; padding: 24px; }
	.i360-empty-icon { font-size: 2rem; margin-bottom: 8px; }
	.i360-empty-text { font-size: 0.82rem; color: var(--text-muted, #999); }

	.i360-card { display: flex; flex-direction: column; gap: 8px; }
	.i360-card-name {
		font-size: 0.95rem; font-weight: 700; color: var(--text, #ddd);
		overflow: hidden; text-overflow: ellipsis; white-space: nowrap;
	}
	.i360-card-meta { display: flex; flex-wrap: wrap; gap: 6px; align-items: center; }
	.i360-stratum-pill {
		padding: 2px 8px; border-radius: 4px;
		background: rgba(167,139,250,0.15); color: #a78bfa;
		font-size: 0.72rem; font-weight: 600;
	}
	.i360-pill {
		padding: 2px 8px; border-radius: 4px;
		font-size: 0.72rem; font-weight: 600;
	}
	.i360-pill-soft {
		padding: 2px 8px; border-radius: 4px;
		background: rgba(127,127,127,0.12);
		color: var(--text-muted, #aaa);
		font-size: 0.72rem;
	}
	.i360-card-counts {
		display: flex; gap: 10px;
		font-size: 0.74rem; color: var(--text-muted, #aaa);
	}
	.i360-bars { display: flex; flex-direction: column; gap: 3px; }
	.i360-bar-row {
		display: grid;
		grid-template-columns: 90px 1fr 28px;
		align-items: center;
		gap: 6px;
		font-size: 0.72rem;
	}
	.i360-bar-row.gap-row { opacity: 0.5; }
	.i360-bar-label {
		color: var(--text-muted, #aaa);
		overflow: hidden; text-overflow: ellipsis; white-space: nowrap;
	}
	.i360-bar-track {
		height: 8px; border-radius: 4px;
		background: rgba(255,255,255,0.06);
		overflow: hidden;
	}
	.i360-bar-fill {
		height: 100%; border-radius: 4px;
		transition: width 0.3s ease;
	}
	.i360-bar-count {
		text-align: end;
		color: var(--text-muted, #aaa);
		font-weight: 600;
		font-variant-numeric: tabular-nums;
	}
	.i360-card-flags {
		display: flex; flex-wrap: wrap; gap: 6px;
		font-size: 0.72rem;
	}
	.i360-warn { color: #ef4444; font-weight: 600; }

	/* ===== FULL-WINDOW ===== */
	.i360-full {
		position: relative;
		width: 100%; height: 100%;
		background: #060612;
		color: #e0e0e0;
		display: flex; flex-direction: column;
		overflow: hidden;
	}
	.i360-empty-full {
		flex: 1; display: flex; flex-direction: column;
		align-items: center; justify-content: center;
		background: radial-gradient(ellipse at 50% 45%, #0e0e28, #060612);
	}
	.i360-empty-icon-lg { font-size: 4rem; margin-bottom: 16px; opacity: 0.5; }
	.i360-empty-text-lg { font-size: 1.1rem; color: rgba(255,255,255,0.3); }

	/* Header */
	.i360-header {
		display: flex; align-items: center; justify-content: space-between;
		padding: 18px 32px;
		background: linear-gradient(180deg, rgba(6,6,18,0.95), transparent);
		z-index: 20; position: relative;
		gap: 16px;
		flex-shrink: 0;
	}
	.i360-header-left { display: flex; align-items: center; gap: 16px; flex: 1; min-width: 0; }
	.i360-header-icon { font-size: 28px; }
	.i360-header-label {
		font-size: 16px; color: #7c3aed; font-weight: 700;
		letter-spacing: 2px; text-transform: uppercase;
	}
	.i360-header-name {
		font-size: 26px; font-weight: 700; color: #f0f0f0;
		overflow: hidden; text-overflow: ellipsis; white-space: nowrap;
	}
	.i360-header-right { display: flex; align-items: center; gap: 14px; flex-shrink: 0; }
	.i360-close {
		width: 48px; height: 48px; border-radius: 50%;
		border: 1px solid rgba(255,255,255,0.1);
		background: rgba(255,255,255,0.03);
		color: #888; font-size: 28px; cursor: pointer;
		display: flex; align-items: center; justify-content: center;
		flex-shrink: 0;
	}
	.i360-close:hover { background: rgba(255,255,255,0.08); color: #fff; }
	.i360-back-full {
		display: flex; align-items: center; gap: 8px;
		padding: 8px 16px; border-radius: 10px;
		background: rgba(255,255,255,0.06);
		border: 1px solid rgba(255,255,255,0.14);
		color: #ddd; font-size: 16px;
		cursor: pointer; flex-shrink: 0;
		max-width: 320px;
	}
	.i360-back-full:hover { background: rgba(255,255,255,0.12); color: #fff; }
	.i360-back-full .i360-back-arrow { font-size: 20px; line-height: 1; flex-shrink: 0; }
	.i360-back-full .i360-back-name { overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }

	/* Dimensions strip */
	.i360-strip {
		display: flex; gap: 32px; padding: 0 32px 14px;
		flex-wrap: wrap; flex-shrink: 0;
	}
	.i360-strip-cell {
		display: flex; flex-direction: column; gap: 4px;
	}
	.i360-strip-label {
		font-size: 11px; color: rgba(255,255,255,0.4);
		text-transform: uppercase; letter-spacing: 1px;
	}
	.i360-strip-value {
		font-size: 16px; color: #ddd;
		display: inline-flex; align-items: center; gap: 6px;
	}
	.i360-dot {
		width: 10px; height: 10px; border-radius: 50%;
		display: inline-block; flex-shrink: 0;
	}

	/* Canvas (matrix container) */
	.i360-canvas {
		flex: 1; position: relative;
		padding: 8px 32px 92px;
		overflow: auto;
		display: flex; align-items: stretch; justify-content: center;
	}
	.i360-matrix-wrap {
		position: relative;
		flex: 1;
		max-width: 1400px;
		display: flex; flex-direction: column;
	}
	.i360-matrix {
		display: grid;
		grid-template-columns: 200px repeat(8, minmax(80px, 1fr)) 64px;
		grid-auto-rows: minmax(72px, 1fr);
		gap: 1px;
		background: rgba(255,255,255,0.06);
		border-radius: 12px;
		overflow: hidden;
		flex: 1;
	}

	.i360-corner,
	.i360-rowtot-header {
		background: #060614;
		display: flex; align-items: center; justify-content: center;
		gap: 8px;
		font-size: 12px;
		color: rgba(255,255,255,0.45);
		padding: 4px 8px;
	}
	.i360-corner { flex-direction: column; }
	.i360-corner-stratum { color: #a78bfa; font-weight: 600; }
	.i360-corner-type { color: #4A9EFF; font-weight: 600; }

	.i360-col-header {
		display: flex; flex-direction: column; align-items: center; justify-content: center;
		padding: 10px 4px;
		gap: 4px;
		background:
			linear-gradient(180deg,
				color-mix(in srgb, var(--col-color, #fff) 18%, transparent),
				#0a0a1c 90%);
		border-bottom: 2px solid var(--col-color, #fff);
	}
	.i360-col-name {
		font-size: 10px; font-weight: 700; letter-spacing: 1px;
		color: var(--col-color, #fff);
		text-align: center;
		text-transform: uppercase;
		line-height: 1.2;
	}
	.i360-col-count {
		font-size: 14px; font-weight: 700;
		color: var(--col-color, #fff);
		font-variant-numeric: tabular-nums;
	}

	.i360-row-header {
		background: #0a0a1c;
		display: flex; align-items: center; gap: 8px;
		padding: 6px 14px;
		border-right: 2px solid rgba(167,139,250,0.18);
	}
	.i360-row-header.active {
		background: linear-gradient(90deg, rgba(167,139,250,0.22), rgba(167,139,250,0.04));
		border-right-color: #a78bfa;
	}
	.i360-row-header.empty-row { opacity: 0.45; }
	.i360-row-num {
		color: #a78bfa; font-weight: 700; font-size: 13px;
		width: 28px; flex-shrink: 0;
	}
	.i360-row-name {
		color: rgba(255,255,255,0.7); font-size: 13px;
		flex-shrink: 0;
	}
	.i360-active-chip {
		margin-inline-start: auto;
		padding: 2px 8px; border-radius: 4px;
		background: #a78bfa; color: #060612;
		font-size: 11px; font-weight: 700;
		overflow: hidden; text-overflow: ellipsis; white-space: nowrap;
		max-width: 110px;
	}

	.i360-cell {
		background: #0a0a1c;
		display: flex; flex-wrap: wrap; align-items: center; justify-content: center;
		padding: 6px 4px;
		gap: 4px;
		position: relative;
	}
	.i360-cell.active-row {
		background: rgba(167,139,250,0.06);
	}
	.i360-cell.empty-cell {
		background: repeating-linear-gradient(45deg, #0a0a1c, #0a0a1c 5px, rgba(255,255,255,0.025) 5px, rgba(255,255,255,0.025) 10px);
	}
	.i360-cell.active-row.empty-cell {
		background:
			linear-gradient(rgba(167,139,250,0.06), rgba(167,139,250,0.06)),
			repeating-linear-gradient(45deg, #0a0a1c, #0a0a1c 5px, rgba(255,255,255,0.04) 5px, rgba(255,255,255,0.04) 10px);
	}
	.i360-dot-btn {
		width: 11px; height: 11px; border-radius: 50%;
		background: var(--dot-color, #888);
		border: none;
		padding: 0;
		cursor: pointer;
		transition: transform 0.15s ease, box-shadow 0.15s ease, opacity 0.15s ease;
		opacity: 0.85;
	}
	.i360-dot-btn:hover {
		transform: scale(1.6);
		box-shadow: 0 0 10px var(--dot-color, #888);
		opacity: 1;
		z-index: 5;
	}
	.i360-overflow {
		font-size: 10px; color: rgba(255,255,255,0.55);
		font-weight: 600; padding: 0 2px;
	}

	.i360-rowtot {
		background: #0a0a1c;
		display: flex; align-items: center; justify-content: center;
		color: rgba(255,255,255,0.4); font-size: 14px;
		font-variant-numeric: tabular-nums;
	}
	.i360-rowtot.active {
		color: #a78bfa; font-weight: 700;
		background: rgba(167,139,250,0.06);
	}

	.i360-hover-label {
		position: absolute;
		top: 12px; right: 12px;
		padding: 6px 12px;
		background: rgba(0,0,0,0.85);
		border: 1px solid rgba(255,255,255,0.15);
		border-radius: 6px;
		color: #fff; font-size: 13px;
		pointer-events: none;
		z-index: 30;
		max-width: 320px;
		overflow: hidden; text-overflow: ellipsis; white-space: nowrap;
	}

	/* Bottom HUD */
	.i360-hud {
		position: absolute; bottom: 0; left: 0; right: 0;
		padding: 16px 36px;
		display: flex; justify-content: space-between;
		z-index: 20;
		background: linear-gradient(0deg, rgba(6,6,18,0.95), transparent);
		flex-wrap: wrap; gap: 8px;
	}
	.i360-hud-left, .i360-hud-right {
		display: flex; gap: 24px;
		flex-wrap: wrap;
	}
	.i360-hud-item {
		font-size: 16px; color: rgba(255,255,255,0.55);
		display: flex; align-items: center; gap: 6px;
	}
	.i360-hud-warn { color: #ef4444; }
</style>
