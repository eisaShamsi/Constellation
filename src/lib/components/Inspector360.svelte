<script lang="ts">
	import { t, tn } from '$lib/i18n';
	import { lookupStageEmoji, libraryStats } from '$lib/libraries/store';
	import { cognitiveLinkTypes } from '$lib/libraries/linkTypeRegistry';
	import HelpTip from './HelpTip.svelte';
	import RelatedCandidates from './RelatedCandidates.svelte'; // MIG-086 §D — suggest + one-click typed link

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

	// MIG-086 §D — the library path for <RelatedCandidates>, derived from the CURRENTLY
	// DISPLAYED note (`data.note_path`) rather than plumbed as a prop. The host's
	// sidebarTab-derived libraryPath updates ~200 ms before `data` (the 360 fetch is
	// debounced), so a plumbed prop would briefly pair the NEW library with the OLD
	// note_path → a suggest query for the wrong note in the wrong library. Deriving from
	// `data` keeps notePath + libraryPath in lockstep with what's on screen.
	const relLibraryPath = $derived.by(() => {
		const p = data?.note_path;
		if (!p) return null;
		return $libraryStats.find((l) => p.startsWith(l.path))?.path ?? null;
	});

	// §112: Stratification Matrix replaces the spherical/angular line.
	// Vertical axis = stratum (1..8, displayed top-down 8→1). Horizontal axis
	// = link direction (7 typed + 1 untyped). Each cell holds the connected
	// notes whose stratum matches the row, drawn as small dots at the typed-
	// direction column. Empty cells are visually present (dashed stripes) so
	// gaps read as a first-class signal — see 360.3D Concept Paper §4.3.

	// MIG-067 §D — the typed-act columns come from the active Link-Type Registry
	// (the 8 seeds + any custom types, canonical order) + the 'untyped' bucket for
	// null/associative links. Was a hardcoded 7 (missing supersedes); 360.3D now
	// shows supersedes and any user-defined type. Read at setup — the registry is
	// boot-seeded before this panel can open.
	// PJ-065 — the 360 matrix is the cognitive grammar; the structural (parent/TOC)
	// lane is excluded so it never becomes a matrix column or a "missing" cognitive type.
	const REG_TYPES = cognitiveLinkTypes();
	type LinkType = string;
	const TYPE_ORDER: LinkType[] = [...REG_TYPES.map((t) => t.id), 'untyped'];

	const TYPE_COLORS: Record<string, string> = {
		...Object.fromEntries(REG_TYPES.map((t): [string, string] => [t.id, t.color])),
		untyped: '#888888',
	};

	// i18n key per type: built-ins keep their `link_<id>` keys; 'untyped' its own.
	// Custom types have no key → typeLabels falls back to the registry label.
	const TYPE_LABEL_KEYS: Record<string, string> = {
		...Object.fromEntries(REG_TYPES.map((t): [string, string] => [t.id, `link_${t.id.replace(/-/g, '_')}`])),
		untyped: 'untyped',
	};

	// Stratum order: highest (Worldview) at top, lowest (Datum) at bottom — so
	// the user reads "altitude" from top-down naturally.
	const STRATA = [8, 7, 6, 5, 4, 3, 2, 1] as const;

	// §120: stratum names live in i18n now (`inspector360.stratum_name_N`).
	// This map stays as the English fallback used when the i18n lookup
	// returns the literal key (i.e. the key is absent in both the active
	// locale and en.json). With the §120 fallback chain in place, this
	// fallback only fires if en.json itself is missing the key.
	const STRATUM_FALLBACK: Record<number, string> = {
		1: 'Datum', 2: 'Information', 3: 'Proposition', 4: 'Concept',
		5: 'Principle', 6: 'Theory', 7: 'Paradigm', 8: 'Worldview',
	};

	// §120: translation helper. $t returns the literal key on miss; this
	// helper returns the fallback when that happens, otherwise the
	// translated value.
	function tr(value: string, key: string, fallback: string): string {
		return value && value !== key ? value : fallback;
	}

	// MIG-088 §2a — shared Maturity colours (Style Setter → Cognitive colours); fallback = today's value.
	const MATURITY_COLORS: Record<string, string> = {
		seed: 'var(--maturity-seed, #9ca3af)', sapling: 'var(--maturity-sapling, #4ade80)', evergreen: 'var(--maturity-evergreen, #16a34a)', canonical: 'var(--maturity-canonical, #f59e0b)', wilting: 'var(--maturity-wilting, #16a34a80)',
	};
	const ORIGIN_COLORS: Record<string, string> = {
		received: 'var(--origin-received, #4A9EFF)', discovered: 'var(--origin-discovered, #FFB347)', mixed: 'var(--origin-mixed, #A78BFA)', none: 'var(--origin-none, #9ca3af)',
	};
	// MIG-014 §1D — stage icons resolved through lookupStageEmoji
	// (Living Link 6-stage baseline + per-Universe customs + legacy
	// Zettelkasten fallback). Old hardcoded STAGE_ICONS removed.

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
			cells[s] = Object.fromEntries(TYPE_ORDER.map((t): [string, LinkedNote[]] => [t, []]));
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

		const colTotals: Record<string, number> = Object.fromEntries(TYPE_ORDER.map((t): [string, number] => [t, 0]));
		const rowTotals: Record<number, number> = { 1: 0, 2: 0, 3: 0, 4: 0, 5: 0, 6: 0, 7: 0, 8: 0 };
		let grandTotal = 0;
		for (const s of STRATA) {
			for (const t of TYPE_ORDER) {
				const c = cells[s][t].length;
				colTotals[t] += c;
				rowTotals[s] += c;
				grandTotal += c;
			}
		}

		return { cells, colTotals, rowTotals, grandTotal };
	});

	// Compact scorecard bars: per-type **share of total** so the visual signal
	// reads as proportion regardless of magnitude. With one count dominating
	// (e.g. 6107 untyped vs 101 supports) max-normalisation collapsed every
	// typed bar to <2 % width and made them invisible. Percent-of-total keeps
	// the same widths but the readable signal moves to the percent text.
	const compactBars = $derived.by(() => {
		if (!data) return null;
		const counts: Record<string, number> = Object.fromEntries(TYPE_ORDER.map((t): [string, number] => [t, 0]));
		for (const [type, notes] of Object.entries(data.typed_links)) {
			if (!(TYPE_ORDER as readonly string[]).includes(type)) continue;
			counts[type as LinkType] = notes.length;
		}
		counts.untyped = data.untyped_links.length;
		let total = 0;
		for (const t of TYPE_ORDER) total += counts[t];
		return { counts, total };
	});

	// Hover state for the floating tooltip that follows the dot.
	// We store the dot's screen-space rect so the tooltip can sit above
	// it (position: fixed) regardless of the matrix's overflow / scroll.
	let hoveredDot = $state<{ name: string; cx: number; top: number } | null>(null);

	function showDotHover(e: MouseEvent, name: string) {
		const target = e.currentTarget as HTMLElement | null;
		if (!target) return;
		const rect = target.getBoundingClientRect();
		hoveredDot = {
			name,
			cx: rect.left + rect.width / 2,
			top: rect.top,
		};
	}
	function hideDotHover() {
		hoveredDot = null;
	}

	// §114: cell-expand state for `+N` overflow. §115 reworked the expanded
	// view from "more dots" to "list of note titles" with internal scroll, so
	// §116 the original Untyped exclusion no longer applies — expanded
	// Untyped renders the same scrollable title list. State auto-clears
	// whenever the active note changes (forward via title-click or back via
	// the back-bar) — Boss S1.3.5 retest finding: persisting the expanded
	// state across navigation is illogical.
	let expandedCells = $state<Set<string>>(new Set());

	$effect(() => {
		// Read note_path so the effect re-runs on navigation; reset state.
		void data?.note_path;
		expandedCells = new Set();
	});

	function toggleCellExpand(stratum: number, type: LinkType) {
		const key = `${stratum}-${type}`;
		const next = new Set(expandedCells);
		if (next.has(key)) next.delete(key); else next.add(key);
		expandedCells = next;
	}

	function isCellExpanded(stratum: number, type: LinkType): boolean {
		return expandedCells.has(`${stratum}-${type}`);
	}

	// §121: per-type display labels. The §113 hardcode for 'untyped' was
	// originally needed because `$t('inspector360.untyped')` returned the
	// literal key (truthy ⇒ OR fallback never fired). With the §120 i18n
	// fallback chain (active locale → en.json → key), `inspector360.untyped`
	// resolves correctly in every locale that has the key, falling back to
	// English when missing. The loop now treats untyped uniformly; the
	// hardcoded English values stay as the final defensive fallback.
	// (Loop variable is `lt` instead of `t` because the imported i18n store
	// is `t` and shadowing it breaks $-auto-subscription.)
	const typeLabels: Record<string, string> = $derived.by(() => {
		// Fallback labels from the registry (covers custom types); 'untyped' fixed.
		const m: Record<string, string> = { untyped: 'Untyped' };
		for (const rt of REG_TYPES) m[rt.id] = rt.label;
		for (const lt of TYPE_ORDER) {
			const key = lt === 'untyped' ? 'inspector360.untyped' : `inspector360.${TYPE_LABEL_KEYS[lt]}`;
			const tr = $t(key);
			if (tr && tr !== key) m[lt] = tr;
		}
		return m;
	});

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
					<span class="i360-stratum-pill">L{activeStratum} {tr($t(`inspector360.stratum_name_${activeStratum}`), `inspector360.stratum_name_${activeStratum}`, STRATUM_FALLBACK[activeStratum])}</span>
					<span class="i360-pill" style="background: color-mix(in srgb, {MATURITY_COLORS[data.maturity] ?? '#999'} 18%, transparent); color: {MATURITY_COLORS[data.maturity] ?? '#999'}">{data.maturity}</span>
					{#if data.stage}<span class="i360-pill-soft">{lookupStageEmoji(data.stage)} {data.stage}</span>{/if}
				</div>
				<div class="i360-card-counts">
					<span>{'⬆'} {data.total_outbound}</span>
					<span>{'⬇'} {data.total_inbound}</span>
					<span>{'\u{1F4DD}'} {data.word_count.toLocaleString()}</span>
				</div>
				<div class="i360-bars">
					{#each TYPE_ORDER as type}
						{@const count = compactBars.counts[type]}
						{@const pct = compactBars.total > 0 ? (count / compactBars.total) * 100 : 0}
						<div class="i360-bar-row" class:gap-row={count === 0}>
							<span class="i360-bar-label">{typeLabels[type]}</span>
							<div class="i360-bar-track">
								<div class="i360-bar-fill" style="width: {pct}%; background: {TYPE_COLORS[type]}"></div>
							</div>
							<span class="i360-bar-count">{count === 0 ? '—' : `${pct.toFixed(1)}%`}</span>
						</div>
					{/each}
				</div>
				<div class="i360-card-flags">
					{#if data.is_orphan}<span class="i360-warn">{'⚠'} {$t('inspector360.orphan') || 'Orphan'}</span>{/if}
					{#if data.single_point_of_failure}<span class="i360-warn">{'⚠'} {$t('inspector360.fragile') || 'Fragile'}</span>{/if}
					{#if data.missing_link_types.length > 0}<span class="i360-warn">{'⚠'} {$tn('plurals.gaps', data.missing_link_types.length)}</span>{/if}
					{#if data.is_due}<span class="i360-warn">{'\u{1F4CB}'} {$t('inspector360.dueForReview') || 'Review due'}</span>{/if}
				</div>
				<!-- MIG-086 §D — surface #3 (compact right-rail scorecard): same suggest +
				     one-click typed-link block as the full-window matrix. Direction INBOUND
				     (suggestion → this note); SPOF/blind-spot pre-set derives-from + "shore it up". -->
				{#if data.is_orphan || data.single_point_of_failure || data.missing_link_types.length > 0}
					<div class="i360-suggest">
						<RelatedCandidates
							notePath={data.note_path}
							noteName={data.note_name}
							libraryPath={relLibraryPath}
							direction="inbound"
							defaultType={data.single_point_of_failure ? 'derives-from' : 'associative'}
							heading={data.single_point_of_failure ? ($t('reviewer.suggestLabelFragile') || 'Shore it up — connect to:') : null}
						/>
					</div>
				{/if}
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
					<span class="i360-strip-label">{tr($t('inspector360.dim_stratum'), 'inspector360.dim_stratum', 'Stratum')} <HelpTip tooltip={tr($t('inspector360.help_dim_stratum'), 'inspector360.help_dim_stratum', '')} position="bottom" /></span>
					<span class="i360-strip-value accent">L{activeStratum} {tr($t(`inspector360.stratum_name_${activeStratum}`), `inspector360.stratum_name_${activeStratum}`, STRATUM_FALLBACK[activeStratum])}</span>
				</div>
				<div class="i360-strip-cell">
					<span class="i360-strip-label">{tr($t('inspector360.dim_maturity'), 'inspector360.dim_maturity', 'Maturity')} <HelpTip tooltip={tr($t('inspector360.help_dim_maturity'), 'inspector360.help_dim_maturity', '')} position="bottom" /></span>
					<span class="i360-strip-value"><span class="i360-dot" style="background: {MATURITY_COLORS[data.maturity] ?? '#999'}"></span>{tr($t(`inspector360.maturity_${data.maturity}`), `inspector360.maturity_${data.maturity}`, data.maturity)}</span>
				</div>
				<div class="i360-strip-cell">
					<span class="i360-strip-label">{tr($t('inspector360.dim_origin'), 'inspector360.dim_origin', 'Origin')} <HelpTip tooltip={tr($t('inspector360.help_dim_origin'), 'inspector360.help_dim_origin', '')} position="bottom" /></span>
					<span class="i360-strip-value"><span class="i360-dot" style="background: {ORIGIN_COLORS[data.origin_type] ?? '#999'}"></span>{tr($t(`inspector360.origin_${data.origin_type}`), `inspector360.origin_${data.origin_type}`, data.origin_type)} {'·'} {$t('inspector360.depth') || 'd'}{data.trust_depth}</span>
				</div>
				<div class="i360-strip-cell">
					<span class="i360-strip-label">{tr($t('inspector360.dim_stage'), 'inspector360.dim_stage', 'Stage')} <HelpTip tooltip={tr($t('inspector360.help_dim_stage'), 'inspector360.help_dim_stage', '')} position="bottom" /></span>
					<span class="i360-strip-value">{lookupStageEmoji(data.stage)} {data.stage ? tr($t(`inspector360.stage_${data.stage}`), `inspector360.stage_${data.stage}`, data.stage) : tr($t('inspector360.stage_none'), 'inspector360.stage_none', 'none')}</span>
				</div>
				<div class="i360-strip-cell">
					<span class="i360-strip-label">{tr($t('inspector360.dim_review'), 'inspector360.dim_review', 'Review')} <HelpTip tooltip={tr($t('inspector360.help_dim_review'), 'inspector360.help_dim_review', '')} position="bottom" /></span>
					<span class="i360-strip-value">
						{#if data.is_due}<span class="i360-warn">{tr($t('inspector360.review_due'), 'inspector360.review_due', 'Due')}</span>{:else if data.last_reviewed}{data.last_reviewed.slice(0, 10)}{:else}{tr($t('inspector360.review_none'), 'inspector360.review_none', '—')}{/if}
					</span>
				</div>
				{#if data.trails.length > 0}
					<div class="i360-strip-cell">
						<span class="i360-strip-label">{tr($t('inspector360.dim_trails'), 'inspector360.dim_trails', 'Trails')} <HelpTip tooltip={tr($t('inspector360.help_dim_trails'), 'inspector360.help_dim_trails', '')} position="bottom" /></span>
						<span class="i360-strip-value">{'\u{1F6E4}️'} {data.trails.length}</span>
					</div>
				{/if}
				{#if data.lens_groups.length > 0}
					<div class="i360-strip-cell">
						<span class="i360-strip-label">{tr($t('inspector360.dim_lenses'), 'inspector360.dim_lenses', 'Lenses')} <HelpTip tooltip={tr($t('inspector360.help_dim_lenses'), 'inspector360.help_dim_lenses', '')} position="bottom" /></span>
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
							<span class="i360-corner-stratum">{'▲'} {tr($t('inspector360.axis_stratum_label'), 'inspector360.axis_stratum_label', 'Stratum')} <HelpTip tooltip={tr($t('inspector360.help_axis_stratum'), 'inspector360.help_axis_stratum', '')} position="bottom" /></span>
							<span class="i360-corner-type">{tr($t('inspector360.axis_type_label'), 'inspector360.axis_type_label', 'Type')} {'→'} <HelpTip tooltip={tr($t('inspector360.help_axis_type'), 'inspector360.help_axis_type', '')} position="bottom" /></span>
						</div>
						{#each TYPE_ORDER as type}
							{@const typeHelpKey = `inspector360.help_type_${TYPE_LABEL_KEYS[type].replace(/^link_/, '')}`}
							{@const isBlindSpot = type !== 'untyped' && matrix.colTotals[type] === 0}
							{@const isTensionsCol = type === 'contradicts' && data.contradictions.length > 0 && !isBlindSpot}
							{@const isFragileCol = type === 'derives-from' && data.single_point_of_failure && !isBlindSpot}
							<div class="i360-col-header"
								class:blind-spot={isBlindSpot}
								class:tensions-flag={isTensionsCol}
								class:fragile-flag={isFragileCol}
								style="--col-color: {TYPE_COLORS[type]}">
								{#if isBlindSpot}<div class="i360-col-warn warn-blind" title="Blind spot — typed direction not used">{'⚠'}</div>{/if}
								{#if isTensionsCol}<div class="i360-col-warn warn-tensions" title="Tensions — active contradicts pointing here">{'⚡'}</div>{/if}
								{#if isFragileCol}<div class="i360-col-warn warn-fragile" title="Fragile — load-bearing on thin foundation">{'⚠'}</div>{/if}
								<div class="i360-col-name">
									{typeLabels[type].toUpperCase()}
									<HelpTip tooltip={tr($t(typeHelpKey), typeHelpKey, '')} position="bottom" />
								</div>
								<div class="i360-col-count">{matrix.colTotals[type]}</div>
							</div>
						{/each}
						<div class="i360-rowtot-header">
							<span class="i360-grand-symbol">{'Σ'} <HelpTip tooltip={tr($t('inspector360.help_grand_total'), 'inspector360.help_grand_total', '')} position="bottom" /></span>
							<span class="i360-grand-value">{matrix.grandTotal}</span>
						</div>

						<!-- Data rows -->
						{#each STRATA as stratum}
							{@const isActive = activeStratum === stratum}
							{@const isEmptyRow = matrix.rowTotals[stratum] === 0}
							<div class="i360-row-header" class:active={isActive} class:empty-row={isEmptyRow && !isActive}>
								<span class="i360-row-num">L{stratum}</span>
								<span class="i360-row-name">{tr($t(`inspector360.stratum_name_${stratum}`), `inspector360.stratum_name_${stratum}`, STRATUM_FALLBACK[stratum])}</span>
								<HelpTip tooltip={tr($t(`inspector360.help_stratum_${stratum}`), `inspector360.help_stratum_${stratum}`, '')} position="top" />
							</div>
							{#each TYPE_ORDER as type}
								{@const cellNotes = matrix.cells[stratum][type]}
								{@const cellEmpty = cellNotes.length === 0}
								{@const expanded = isCellExpanded(stratum, type)}
								<div class="i360-cell"
									class:active-row={isActive}
									class:empty-cell={cellEmpty}
									class:expanded
									style="--col-color: {TYPE_COLORS[type]}">
									{#if expanded}
										<!-- §115/§116: expanded cell renders as a vertical list of
										     note titles. §116 removed the Untyped exclusion — the
										     scrollable list handles 800+ items without ballooning. -->
										<button class="i360-list-collapse"
											onclick={() => toggleCellExpand(stratum, type)}
											title="Collapse">×</button>
										<div class="i360-list-scroll">
											{#each cellNotes as note}
												<button class="i360-list-item"
													onclick={() => onNoteClick?.(note.path, note.name)}
													title={note.name}>
													<span class="i360-list-bullet" style="background: {TYPE_COLORS[type]}"></span>
													<span class="i360-list-name" dir="auto">{note.name}</span>
												</button>
											{/each}
										</div>
									{:else}
										{#each cellNotes.slice(0, MAX_DOTS_PER_CELL) as note}
											<button class="i360-dot-btn"
												style="--dot-color: {TYPE_COLORS[type]}"
												aria-label={note.name}
												onmouseenter={(e) => showDotHover(e, note.name)}
												onmouseleave={hideDotHover}
												onclick={() => onNoteClick?.(note.path, note.name)}>
											</button>
										{/each}
										{#if cellNotes.length > MAX_DOTS_PER_CELL}
											<button class="i360-overflow-btn"
												onclick={() => toggleCellExpand(stratum, type)}
												title={`Show all ${cellNotes.length}`}>+{cellNotes.length - MAX_DOTS_PER_CELL}</button>
										{/if}
									{/if}
								</div>
							{/each}
							<div class="i360-rowtot" class:active={isActive}>{matrix.rowTotals[stratum]}</div>
						{/each}
					</div>

					<!-- MIG-086 §D — surface #3: turn the diagnosis into an action. When THIS note
					     is an orphan, a single point of failure, or has blind-spot link types, offer
					     the same suggest + one-click typed-link block beneath the matrix (inside the
					     scrollable matrix-wrap so it scrolls with content, not clipped by the absolute
					     HUD). Direction INBOUND (suggestion → this note): the link lives in the
					     candidate's frontmatter pointing here, so an orphan gains an incoming link and
					     the flag clears on the next open. SPOF/blind-spot pre-set derives-from. -->
					{#if data.is_orphan || data.single_point_of_failure || data.missing_link_types.length > 0}
						<div class="i360-suggest">
							<RelatedCandidates
								notePath={data.note_path}
								noteName={data.note_name}
								libraryPath={relLibraryPath}
								direction="inbound"
								defaultType={data.single_point_of_failure ? 'derives-from' : 'associative'}
								heading={data.single_point_of_failure ? ($t('reviewer.suggestLabelFragile') || 'Shore it up — connect to:') : null}
							/>
						</div>
					{/if}

				</div>
			</div>

			<!-- Floating hover tooltip — sits directly above the hovered dot. -->
			{#if hoveredDot}
				<div class="i360-dot-tooltip"
					style="left: {hoveredDot.cx}px; top: {hoveredDot.top}px;"
					dir="auto">
					{hoveredDot.name}
				</div>
			{/if}

			<!-- Bottom HUD -->
			<div class="i360-hud">
				<div class="i360-hud-left">
					<span class="i360-hud-item">{'⬆'} {$tn('plurals.outbound', data.total_outbound)}</span>
					<span class="i360-hud-item">{'⬇'} {$tn('plurals.inbound', data.total_inbound)}</span>
					<span class="i360-hud-item">{'\u{1F4DD}'} {$tn('plurals.words', data.word_count)}</span>
				</div>
				<div class="i360-hud-right">
					{#if data.is_orphan}<span class="i360-hud-item i360-hud-warn-orphan">{'⚠'} {$t('inspector360.orphan') || 'Orphan'} <HelpTip tooltip={tr($t('inspector360.help_hud_orphan'), 'inspector360.help_hud_orphan', '')} position="top" /></span>{/if}
					{#if data.single_point_of_failure}<span class="i360-hud-item i360-hud-warn-fragile">{'⚠'} {$t('inspector360.fragile') || 'Fragile'} <HelpTip tooltip={tr($t('inspector360.help_hud_fragile'), 'inspector360.help_hud_fragile', '')} position="top" /></span>{/if}
					{#if data.missing_link_types.length > 0}<span class="i360-hud-item i360-hud-warn-blind">{'⚠'} {$tn('plurals.blindSpots', data.missing_link_types.length)} <HelpTip tooltip={tr($t('inspector360.help_hud_blind_spots'), 'inspector360.help_hud_blind_spots', '')} position="top" /></span>{/if}
					{#if data.contradictions.length > 0}<span class="i360-hud-item i360-hud-warn-tensions">{'⚡'} {$tn('plurals.tensions', data.contradictions.length)} <HelpTip tooltip={tr($t('inspector360.help_hud_tensions'), 'inspector360.help_hud_tensions', '')} position="top" /></span>{/if}
				</div>
			</div>
		{/if}
	</div>
{/if}

<style>
	/* §113: theme-aware colours via Constellation's CSS variable system.
	 * The matrix lived in a hardcoded dark surface (#060612 / #0a0a1c / etc.)
	 * and looked wrong against a light interface. Now: backgrounds, text and
	 * accents come from the theme; only the link-type dot colours stay literal
	 * because they're semantic. Sizes doubled per Boss directive on S1.1 + S1.2.
	 */

	/* ===== COMPACT SIDEBAR — scorecard ===== */
	.i360.compact { display: flex; flex-direction: column; padding: 12px; gap: 14px; }
	.i360-back-bar {
		display: flex; align-items: center; gap: 8px;
		padding: 8px 12px;
		background: var(--background-secondary);
		border: 1px solid var(--background-modifier-border);
		border-radius: 8px;
		color: var(--text-muted);
		font-size: calc(1.4rem * var(--rs-scale, 1));
		cursor: pointer;
		text-align: start;
	}
	.i360-back-bar:hover {
		background: var(--background-modifier-hover);
		color: var(--text-normal);
	}
	.i360-back-arrow { font-size: calc(1.6rem * var(--rs-scale, 1)); line-height: 1; flex-shrink: 0; }
	.i360-back-name { flex: 1; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
	.i360-empty { text-align: center; padding: 32px 16px; }
	.i360-empty-icon { font-size: calc(3.5rem * var(--rs-scale, 1)); margin-bottom: 12px; }
	.i360-empty-text { font-size: calc(1.5rem * var(--rs-scale, 1)); color: var(--text-muted); }

	.i360-card { display: flex; flex-direction: column; gap: 14px; }
	.i360-card-name {
		font-size: calc(1.85rem * var(--rs-scale, 1)); font-weight: 700; color: var(--text-normal);
		overflow: hidden; text-overflow: ellipsis; white-space: nowrap;
	}
	.i360-card-meta { display: flex; flex-wrap: wrap; gap: 8px; align-items: center; }
	.i360-stratum-pill {
		padding: 4px 12px; border-radius: 6px;
		background: color-mix(in srgb, var(--text-accent) 15%, transparent);
		color: var(--text-accent);
		font-size: calc(1.4rem * var(--rs-scale, 1)); font-weight: 600;
	}
	.i360-pill {
		padding: 4px 12px; border-radius: 6px;
		font-size: calc(1.4rem * var(--rs-scale, 1)); font-weight: 600;
	}
	.i360-pill-soft {
		padding: 4px 12px; border-radius: 6px;
		background: var(--background-secondary);
		color: var(--text-muted);
		font-size: calc(1.4rem * var(--rs-scale, 1));
	}
	.i360-card-counts {
		display: flex; gap: 16px;
		font-size: calc(1.45rem * var(--rs-scale, 1)); color: var(--text-muted);
	}
	.i360-bars { display: flex; flex-direction: column; gap: 6px; }
	.i360-bar-row {
		display: grid;
		/* MIG-080 §0b — label column shrinkable (minmax floor 0) so the fixed
		   count column can't overflow a narrow sidebar and clip against the edge;
		   keeps the .i360.compact 12px side inset intact (count stays 60px so
		   the bars still align across rows). */
		grid-template-columns: minmax(0, 130px) 1fr 60px;
		align-items: center;
		gap: 10px;
		font-size: calc(1.4rem * var(--rs-scale, 1));
	}
	.i360-bar-row.gap-row { opacity: 0.5; }
	.i360-bar-label {
		color: var(--text-muted);
		overflow: hidden; text-overflow: ellipsis; white-space: nowrap;
	}
	.i360-bar-track {
		height: 14px; border-radius: 7px;
		background: var(--background-secondary);
		overflow: hidden;
	}
	.i360-bar-fill {
		height: 100%; border-radius: 7px;
		min-width: 2px;
		transition: width 0.3s ease;
	}
	.i360-bar-count {
		text-align: end;
		color: var(--text-muted);
		font-weight: 600;
		font-variant-numeric: tabular-nums;
	}
	.i360-card-flags {
		display: flex; flex-wrap: wrap; gap: 8px;
		font-size: calc(1.4rem * var(--rs-scale, 1));
	}
	.i360-warn { color: var(--text-error, #ef4444); font-weight: 600; }

	/* ===== FULL-WINDOW — Stratification Matrix ===== */
	.i360-full {
		position: relative;
		width: 100%; height: 100%;
		background: var(--background-primary);
		color: var(--text-normal);
		display: flex; flex-direction: column;
		overflow: hidden;
	}
	.i360-empty-full {
		flex: 1; display: flex; flex-direction: column;
		align-items: center; justify-content: center;
		background: var(--background-primary);
	}
	.i360-empty-icon-lg { font-size: calc(6rem * var(--rs-scale, 1)); margin-bottom: 20px; opacity: 0.5; }
	.i360-empty-text-lg { font-size: calc(1.6rem * var(--rs-scale, 1)); color: var(--text-muted); }

	/* Header — §114: scaled down ~25 % from §113's 2× so the full matrix fits. */
	.i360-header {
		display: flex; align-items: center; justify-content: space-between;
		padding: 16px 32px;
		background: var(--background-primary-alt);
		border-bottom: 1px solid var(--background-modifier-border);
		z-index: 20; position: relative;
		gap: 20px;
		flex-shrink: 0;
	}
	.i360-header-left { display: flex; align-items: center; gap: 18px; flex: 1; min-width: 0; }
	.i360-header-icon { font-size: calc(40px * var(--rs-scale, 1)); }
	.i360-header-label {
		font-size: calc(24px * var(--rs-scale, 1)); color: var(--text-accent); font-weight: 700;
		letter-spacing: 3px; text-transform: uppercase;
	}
	.i360-header-name {
		font-size: calc(32px * var(--rs-scale, 1)); font-weight: 700; color: var(--text-normal);
		overflow: hidden; text-overflow: ellipsis; white-space: nowrap;
	}
	.i360-header-right { display: flex; align-items: center; gap: 14px; flex-shrink: 0; }
	.i360-close {
		width: 48px; height: 48px; border-radius: 50%;
		border: 1px solid var(--background-modifier-border);
		background: var(--background-secondary);
		color: var(--text-muted); font-size: calc(26px * var(--rs-scale, 1)); cursor: pointer;
		display: flex; align-items: center; justify-content: center;
		flex-shrink: 0;
	}
	.i360-close:hover {
		background: var(--background-modifier-hover);
		color: var(--text-normal);
	}
	.i360-back-full {
		display: flex; align-items: center; gap: 10px;
		padding: 8px 16px; border-radius: 10px;
		background: var(--background-secondary);
		border: 1px solid var(--background-modifier-border);
		color: var(--text-normal); font-size: calc(17px * var(--rs-scale, 1));
		cursor: pointer; flex-shrink: 0;
		max-width: 360px;
	}
	.i360-back-full:hover {
		background: var(--background-modifier-hover);
	}
	.i360-back-full .i360-back-arrow { font-size: calc(22px * var(--rs-scale, 1)); line-height: 1; flex-shrink: 0; }
	.i360-back-full .i360-back-name { overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }

	/* Dimensions strip */
	.i360-strip {
		display: flex; gap: 36px; padding: 12px 32px 16px;
		flex-wrap: wrap; flex-shrink: 0;
	}
	.i360-strip-cell {
		display: flex; flex-direction: column; gap: 4px;
	}
	.i360-strip-label {
		font-size: calc(16px * var(--rs-scale, 1)); color: var(--text-faint);
		text-transform: uppercase; letter-spacing: 1.5px;
	}
	.i360-strip-value {
		font-size: calc(22px * var(--rs-scale, 1)); color: var(--text-normal);
		display: inline-flex; align-items: center; gap: 8px;
	}
	.i360-strip-value.accent { color: var(--text-accent); }
	.i360-dot {
		width: 14px; height: 14px; border-radius: 50%;
		display: inline-block; flex-shrink: 0;
	}

	/* Canvas (matrix container) — §114: row min reduced from 110px → 78px so
	 * all 8 stratum rows fit in a typical 1080p viewport. The previous
	 * `flex: 1` + `overflow: hidden` combination was clipping the bottom
	 * rows when 8 × 110 + header exceeded canvas height. */
	.i360-canvas {
		flex: 1; position: relative;
		padding: 8px 32px 88px;
		overflow: auto;
		display: flex; align-items: stretch; justify-content: center;
	}
	.i360-matrix-wrap {
		position: relative;
		flex: 1;
		max-width: 1600px;
		display: flex; flex-direction: column;
		min-height: 0;
	}
	/* MIG-086 §D — the inline suggest block (beneath the full-window matrix and inside the
	   compact scorecard). A top rule separates it from the diagnosis it acts on; the
	   <RelatedCandidates> component owns its own internal scroll (max-height: 60vh). */
	.i360-suggest {
		margin-top: 16px;
		padding-top: 12px;
		border-top: 1px solid var(--background-modifier-border);
		flex-shrink: 0;
	}
	.i360-matrix {
		display: grid;
		grid-template-columns: 220px repeat(8, minmax(96px, 1fr)) 76px;
		grid-auto-rows: minmax(78px, 1fr);
		gap: 1px;
		background: var(--background-modifier-border);
		border: 1px solid var(--background-modifier-border);
		border-radius: 12px;
		overflow: hidden;
		flex: 1;
	}

	.i360-corner {
		background: var(--background-secondary);
		display: flex; align-items: center; justify-content: center;
		gap: 6px;
		font-size: calc(16px * var(--rs-scale, 1));
		color: var(--text-muted);
		padding: 6px 10px;
		flex-direction: column;
	}
	.i360-corner-stratum { color: var(--text-accent); font-weight: 600; }
	.i360-corner-type { color: var(--color-blue); font-weight: 600; }

	/* §115: top-right corner shows the matrix grand total (Σ + count). */
	.i360-rowtot-header {
		background: var(--background-secondary);
		display: flex; flex-direction: column; align-items: center; justify-content: center;
		gap: 2px;
		padding: 6px 8px;
	}
	.i360-grand-symbol {
		font-size: calc(16px * var(--rs-scale, 1)); color: var(--text-faint); font-weight: 600;
	}
	.i360-grand-value {
		font-size: calc(22px * var(--rs-scale, 1)); color: var(--text-accent); font-weight: 700;
		font-variant-numeric: tabular-nums;
		line-height: 1;
	}

	/* §115/§117: column header — softened background tint
	 * (22→10→5%) and text colour mixed with --text-normal for contrast
	 * against the tinted gradient. The coloured bottom border still
	 * carries the type-coding signal. */
	.i360-col-header {
		display: flex; flex-direction: column; align-items: center; justify-content: center;
		padding: 10px 4px;
		gap: 4px;
		background:
			linear-gradient(180deg,
				color-mix(in srgb, var(--col-color, currentColor) 5%, transparent),
				var(--background-primary-alt) 90%);
		border-bottom: 3px solid var(--col-color, currentColor);
	}
	/* §122: blind-spot column header — typed direction with zero connections
	 * for this note. Warning tint (theme-aware via --text-error) + red
	 * bottom border + red count colour replace the type-colour scheme so
	 * the gap is undeniable at a glance. Untyped excluded (it's the absence
	 * of typing, not a typed direction). */
	.i360-col-header.blind-spot {
		background:
			linear-gradient(180deg,
				color-mix(in srgb, var(--text-error, #ef4444) 14%, transparent),
				var(--background-primary-alt) 90%);
		border-bottom-color: var(--text-error, #ef4444);
	}
	/* §124/§125: column-header warning treatment.
	 *
	 * §124 originally added a 3-px coloured top border per warning, but the
	 * matrix's `border-radius: 12px` + `overflow: hidden` clips the top
	 * border on the leftmost / rightmost column headers and makes a 3 px
	 * stripe hard to see even on middle columns. §125 keeps the top border
	 * as a secondary cue and adds an inline icon above the column name as
	 * the primary signal — same icon as the corresponding HUD chip so
	 * the visual line from chip to column is direct.
	 *
	 *  - Blind spot:   ⚠ red    (also full red column treatment from §122)
	 *  - Fragile:      ⚠ yellow (Derives From column)
	 *  - Tensions:     ⚡ brown  (Contradicts column)
	 *
	 * Brown isn't in the theme palette, so it's hardcoded with a
	 * .theme-dark cascade override for visibility on both themes. */
	.i360-col-header.tensions-flag {
		border-top: 3px solid #8b4513;
	}
	:global(.theme-dark) .i360-col-header.tensions-flag {
		border-top-color: #c89875;
	}
	.i360-col-header.fragile-flag {
		border-top: 3px solid var(--color-yellow, #e0ac00);
	}
	.i360-col-warn {
		font-size: calc(18px * var(--rs-scale, 1));
		line-height: 1;
		font-weight: 700;
		text-align: center;
	}
	.i360-col-warn.warn-blind { color: var(--text-error, #ef4444); }
	.i360-col-warn.warn-fragile { color: var(--color-yellow, #e0ac00); }
	.i360-col-warn.warn-tensions { color: #8b4513; }
	:global(.theme-dark) .i360-col-warn.warn-tensions { color: #c89875; }
	.i360-col-name {
		font-size: calc(14px * var(--rs-scale, 1)); font-weight: 700; letter-spacing: 1px;
		color: color-mix(in srgb, var(--col-color, currentColor) 55%, var(--text-normal));
		text-align: center;
		text-transform: uppercase;
		line-height: 1.2;
	}
	.i360-col-header.blind-spot .i360-col-name {
		color: var(--text-error, #ef4444);
	}
	.i360-col-count {
		font-size: calc(20px * var(--rs-scale, 1)); font-weight: 700;
		color: color-mix(in srgb, var(--col-color, currentColor) 55%, var(--text-normal));
		font-variant-numeric: tabular-nums;
	}
	.i360-col-header.blind-spot .i360-col-count {
		color: var(--text-error, #ef4444);
	}

	.i360-row-header {
		background: var(--background-primary-alt);
		display: flex; align-items: center; gap: 10px;
		padding: 8px 14px;
		border-right: 3px solid color-mix(in srgb, var(--text-accent) 25%, transparent);
	}
	.i360-row-header.active {
		background:
			linear-gradient(90deg,
				color-mix(in srgb, var(--text-accent) 28%, var(--background-primary-alt)),
				color-mix(in srgb, var(--text-accent) 6%, var(--background-primary-alt)));
		border-right-color: var(--text-accent);
	}
	.i360-row-header.empty-row { opacity: 0.45; }
	.i360-row-num {
		color: var(--text-accent); font-weight: 700; font-size: calc(20px * var(--rs-scale, 1));
		width: 38px; flex-shrink: 0;
	}
	.i360-row-name {
		color: var(--text-normal); font-size: calc(18px * var(--rs-scale, 1));
		flex-shrink: 0;
	}

	.i360-cell {
		background: var(--background-primary-alt);
		display: flex; flex-wrap: wrap; align-items: center; justify-content: center;
		padding: 8px 6px;
		gap: 5px;
		position: relative;
	}
	.i360-cell.active-row {
		background: color-mix(in srgb, var(--text-accent) 8%, var(--background-primary-alt));
	}
	.i360-cell.empty-cell {
		background:
			repeating-linear-gradient(45deg,
				var(--background-primary-alt) 0,
				var(--background-primary-alt) 6px,
				color-mix(in srgb, var(--text-muted) 8%, var(--background-primary-alt)) 6px,
				color-mix(in srgb, var(--text-muted) 8%, var(--background-primary-alt)) 12px);
	}
	.i360-cell.active-row.empty-cell {
		background:
			linear-gradient(
				color-mix(in srgb, var(--text-accent) 8%, transparent),
				color-mix(in srgb, var(--text-accent) 8%, transparent)),
			repeating-linear-gradient(45deg,
				var(--background-primary-alt) 0,
				var(--background-primary-alt) 6px,
				color-mix(in srgb, var(--text-muted) 12%, var(--background-primary-alt)) 6px,
				color-mix(in srgb, var(--text-muted) 12%, var(--background-primary-alt)) 12px);
	}
	.i360-dot-btn {
		width: 13px; height: 13px; border-radius: 50%;
		background: var(--dot-color, #888);
		border: none;
		padding: 0;
		cursor: pointer;
		transition: transform 0.15s ease, box-shadow 0.15s ease, opacity 0.15s ease;
		opacity: 0.85;
	}
	.i360-dot-btn:hover {
		transform: scale(1.7);
		box-shadow: 0 0 12px var(--dot-color, #888);
		opacity: 1;
		z-index: 5;
	}
	.i360-overflow {
		font-size: calc(14px * var(--rs-scale, 1)); color: var(--text-muted);
		font-weight: 600; padding: 0 3px;
	}
	.i360-overflow-btn {
		font-size: calc(14px * var(--rs-scale, 1)); font-weight: 600;
		color: var(--text-muted);
		background: var(--background-secondary);
		border: 1px solid var(--background-modifier-border);
		border-radius: 6px;
		padding: 2px 8px;
		cursor: pointer;
		font-variant-numeric: tabular-nums;
		transition: background 0.15s ease, color 0.15s ease, border-color 0.15s ease;
	}
	.i360-overflow-btn:hover {
		background: var(--background-modifier-hover);
		color: var(--text-normal);
		border-color: var(--col-color, var(--background-modifier-border-focus));
	}

	/* §115: expanded typed cell renders a vertical list of titles. Cell layout
	 * switches from `flex-wrap center` (dots) to `flex-column stretch` (list).
	 * `×` collapse button is absolutely-positioned at the top-right and
	 * always visible regardless of how long the list grows. The scroll
	 * container caps height so very large cells (e.g. 49 supports) don't
	 * balloon the row past the canvas. */
	.i360-cell.expanded {
		align-items: stretch;
		justify-content: flex-start;
		flex-wrap: nowrap;
		flex-direction: column;
		padding: 6px 4px 4px;
		gap: 0;
	}
	.i360-list-collapse {
		position: absolute;
		top: 4px;
		inset-inline-end: 4px;
		width: 22px; height: 22px;
		background: var(--background-secondary);
		border: 1px solid var(--background-modifier-border);
		border-radius: 5px;
		color: var(--text-muted);
		cursor: pointer;
		font-size: calc(16px * var(--rs-scale, 1)); line-height: 1;
		display: flex; align-items: center; justify-content: center;
		z-index: 3;
		flex-shrink: 0;
		padding: 0;
	}
	.i360-list-collapse:hover {
		background: var(--background-modifier-hover);
		color: var(--text-normal);
		border-color: var(--col-color, var(--background-modifier-border-focus));
	}
	.i360-list-scroll {
		display: flex;
		flex-direction: column;
		gap: 1px;
		overflow-y: auto;
		max-height: 240px;
		padding-top: 26px;
		padding-inline-end: 2px;
		width: 100%;
	}
	.i360-list-item {
		display: flex;
		align-items: center;
		gap: 7px;
		background: transparent;
		border: none;
		border-radius: 4px;
		padding: 3px 6px;
		text-align: start;
		cursor: pointer;
		font-size: calc(13px * var(--rs-scale, 1));
		color: var(--text-normal);
		transition: background 0.15s ease;
		width: 100%;
		min-width: 0;
	}
	.i360-list-item:hover {
		background: color-mix(in srgb, var(--col-color, var(--text-accent)) 12%, transparent);
	}
	.i360-list-bullet {
		width: 8px; height: 8px;
		border-radius: 50%;
		flex-shrink: 0;
	}
	.i360-list-name {
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
		flex: 1;
		min-width: 0;
		line-height: 1.4;
	}

	.i360-rowtot {
		background: var(--background-primary-alt);
		display: flex; align-items: center; justify-content: center;
		color: var(--text-muted); font-size: calc(22px * var(--rs-scale, 1));
		font-variant-numeric: tabular-nums;
	}
	.i360-rowtot.active {
		color: var(--text-accent); font-weight: 700;
		background: color-mix(in srgb, var(--text-accent) 8%, var(--background-primary-alt));
	}

	/* §113: floating tooltip — sits directly above the hovered dot via
	 * `position: fixed` so it escapes `overflow: hidden` on the matrix and
	 * doesn't depend on cell layout. JS sets left/top to the dot's centre-x
	 * and top-y; the transform shifts it above-and-centred. */
	.i360-dot-tooltip {
		position: fixed;
		transform: translate(-50%, calc(-100% - 10px));
		padding: 6px 12px;
		background: color-mix(in srgb, var(--background-secondary) 92%, var(--text-normal));
		border: 1px solid var(--background-modifier-border-focus);
		border-radius: 7px;
		color: var(--text-normal); font-size: calc(17px * var(--rs-scale, 1));
		font-weight: 500;
		pointer-events: none;
		z-index: 9999;
		max-width: 420px;
		overflow: hidden; text-overflow: ellipsis; white-space: nowrap;
		box-shadow: 0 6px 18px rgba(0, 0, 0, 0.35);
	}

	/* Bottom HUD */
	.i360-hud {
		position: absolute; bottom: 0; left: 0; right: 0;
		padding: 16px 36px;
		display: flex; justify-content: space-between;
		z-index: 20;
		background: var(--background-primary-alt);
		border-top: 1px solid var(--background-modifier-border);
		flex-wrap: wrap; gap: 10px;
	}
	.i360-hud-left, .i360-hud-right {
		display: flex; gap: 26px;
		flex-wrap: wrap;
	}
	.i360-hud-item {
		font-size: calc(21px * var(--rs-scale, 1)); color: var(--text-muted);
		display: flex; align-items: center; gap: 8px;
	}
	.i360-hud-warn { color: var(--text-error, #ef4444); }
	/* §124: per-warning HUD chip colors so each warning is visually
	 * distinguishable. Red was overloaded across all four; now each
	 * warning carries its own semantic colour:
	 *  - Orphan (no inbound): orange — isolation
	 *  - Fragile (load-bearing on thin foundation): yellow — caution
	 *  - Blind spots (typed directions unused): red — serious gap
	 *  - Tensions (active contradicts): brown — clash / disagreement
	 * Each chip's HelpTip ? icon inherits the chip's colour via CSS
	 * cascade since the ? button uses currentColor for its border.
	 * Brown isn't in the theme palette; we hardcode + override per theme. */
	.i360-hud-warn-orphan   { color: var(--color-orange, #ea580c); }
	.i360-hud-warn-fragile  { color: var(--color-yellow, #e0ac00); }
	.i360-hud-warn-blind    { color: var(--text-error, #ef4444); }
	.i360-hud-warn-tensions { color: #8b4513; }
	:global(.theme-dark) .i360-hud-warn-tensions { color: #c89875; }
</style>
