<script lang="ts">
	import { t } from '$lib/i18n';
	import HelpTip from './HelpTip.svelte';

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

	// §119: explanations surfaced via the HelpTip `?` affordance. First-time
	// readers of the matrix don't know what each stratum, type, or dimension
	// means; these tooltips make every element self-teaching without
	// pushing the explanations into the visual itself.
	const HELP_STRATUM: Record<number, string> = {
		8: 'Worldview — the deepest layer of your thinking. Foundational beliefs and paradigm-spanning principles. The lens through which all other thinking is filtered. A note here is something you would defend at the level of identity.',
		7: 'Paradigm — an established framework of thinking. A coherent system of principles that organizes how you approach a domain (your model of physics, your theory of leadership, your practice of Islam). One step below Worldview.',
		6: 'Theory — a structured explanation that connects multiple principles into a model. A theory both predicts and explains; a principle on its own does not.',
		5: 'Principle — a general law or rule abstracted from concepts. "Power corrupts." "Compounding rewards patience." Principles are reusable across domains.',
		4: 'Concept — an abstract idea or category. Concepts name a pattern across many specific instances. A note at L4 has been refined enough to stand on its own as a defined idea.',
		3: 'Proposition — a specific claim or assertion. Stronger than information (it has a stance), weaker than a principle (it is not yet generalized).',
		2: 'Information — processed data with meaning. Information is what data becomes after you have made sense of it, but before you have drawn a claim from it.',
		1: 'Datum — a single fact, observation, or quote. The atomic unit of knowledge. A date, a number, a passage transcribed verbatim.',
	};
	const HELP_TYPE: Record<LinkType, string> = {
		supports:       'Supports — this note backs up another note’s claim. Annotated as [[target|supports]]. Use when the target’s argument is strengthened by what this note contains.',
		contradicts:    'Contradicts — this note disputes or challenges another note’s claim. [[target|contradicts]]. Tracking contradictions is intellectual honesty: every Supports without a Contradicts is one-sided thinking.',
		causes:         'Causes — this note describes a causal relationship. A leads to B. [[target|causes]]. Use when there is a directional cause-and-effect, not just correlation.',
		'derives-from': 'Derives From — this note’s reasoning is based on another note. The target is the source. [[target|derives-from]]. The trust depth (e.g. d11) in the dimension strip counts how deep this chain runs to reach a root.',
		generalizes:    'Generalizes — this note draws a broader pattern from specific examples. [[target|generalizes]]. Use when this note abstracts upward from the target.',
		exemplifies:    'Exemplifies — this note is an instance or example of a broader idea in the target. [[target|exemplifies]]. The opposite direction of Generalizes.',
		'part-of':      'Part Of — this note is a component of a larger idea, system, or hierarchy in the target. [[target|part-of]]. Structural composition rather than logical relationship.',
		untyped:        'Untyped — plain wikilinks without a relationship type, written as [[target]]. The link exists but you have not yet committed to what it means. Untyped is a starting state; mature thinking moves links toward typed forms over time.',
	};
	const HELP_DIM = {
		stratum:  'Stratum — the note’s intellectual altitude on the 8-level hierarchy from L1 Datum to L8 Worldview. Computed from word count, inbound links, source depth, and the typed-link directions you have used. The matrix highlights this row in purple.',
		maturity: 'Maturity — the note’s lifecycle state: Seed → Sapling → Evergreen → Canonical → Wilting. Tracks whether the note is raw, growing, established, locked, or decaying.',
		origin:   'Origin — where the note came from: Received (read from a source), Discovered (your own thinking), or Mixed. The depth number (e.g. d11) is how far the source chain runs through derives-from links to reach a root.',
		stage:    'Stage — how formally the idea is committed: Fleeting (quick capture), Literature (notes from a source), Permanent (refined and standalone), Synthesis (cross-source consolidation).',
		review:   'Review — when this note was last intentionally reviewed. Shows the date or "Due" if the next review is past. Tracked by the Cognitive Engine’s Review Pulse so old notes do not decay silently.',
		trails:   'Trails — the number of sequenced narratives this note participates in. Trails are paths through your library that you create in the Trails feature.',
		lenses:   'Lenses — the number of Multi-Lens groups this note belongs to. Lenses are thematic groupings (e.g. all notes tagged with a project or color).',
	};
	const HELP_GRAND = 'Grand total — the count of every (deduped) connection to this note across all strata and link types. Equals the sum of the 8 column totals at the top, and equals the sum of the 8 row Σ totals on the right. If those three numbers diverge, the matrix derivation has a bug.';
	const HELP_HUD = {
		orphan:     'Orphan — this note has zero inbound links. Nothing in your library references it. Orphans are not always wrong, but they signal possible unintentional isolation.',
		fragile:    'Fragile — many notes link TO this one, but it has few derives-from links of its own. The note is load-bearing on a thin foundation. If you ever change or remove it, many dependents shake.',
		blindSpots: 'Blind spots — typed link directions you have not used for this note. With 7 typed directions available, having 5 blind spots means only 2 of the seven directions of reasoning have been declared for this note.',
		tensions:   'Tensions — active disagreements with this note (Contradicts links pointing at it). Tensions are not problems to fix; they are knowledge to develop. A note with multiple tensions is a note where your thinking is alive.',
	};
	const HELP_AXIS_STRATUM = 'Vertical axis — stratum. The 8 rows from L1 Datum at the bottom to L8 Worldview at the top represent intellectual altitude. The active note’s row is highlighted in purple. Dots above your row are connections at higher altitudes (more abstract); dots below are at lower altitudes (more concrete).';
	const HELP_AXIS_TYPE = 'Horizontal axis — link type. The 8 columns are the 7 typed link directions plus Untyped. Each connected note becomes a coloured dot in the cell where its stratum meets the typed direction it shares with the active note. Diagonal stripes mean the cell is empty — a blind spot at that (stratum, type) intersection.';

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
		const counts: Record<LinkType, number> = { supports: 0, contradicts: 0, causes: 0, 'derives-from': 0, generalizes: 0, exemplifies: 0, 'part-of': 0, untyped: 0 };
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

	// §113: per-type display labels. 'untyped' is hardcoded because
	// `$t('inspector360.untyped')` returns the literal key when missing
	// from the locale (truthy ⇒ the OR fallback never fires) — same
	// regression §104 closed before the §112 rewrite re-introduced it.
	// For typed directions we still read $t inside the derived so locale
	// switches are picked up. (Loop variable is `lt` instead of `t`
	// because the imported i18n store is `t` and shadowing it breaks the
	// $-auto-subscription.)
	const typeLabels: Record<LinkType, string> = $derived.by(() => {
		const m: Record<LinkType, string> = {
			supports: 'Supports', contradicts: 'Contradicts', causes: 'Causes',
			'derives-from': 'Derives From', generalizes: 'Generalizes',
			exemplifies: 'Exemplifies', 'part-of': 'Part Of', untyped: 'Untyped',
		};
		for (const lt of TYPE_ORDER) {
			if (lt === 'untyped') continue;
			const key = `inspector360.${TYPE_LABEL_KEYS[lt]}`;
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
					<span class="i360-strip-label">Stratum <HelpTip tooltip={HELP_DIM.stratum} position="bottom" /></span>
					<span class="i360-strip-value accent">L{activeStratum} {STRATUM_NAMES[activeStratum]}</span>
				</div>
				<div class="i360-strip-cell">
					<span class="i360-strip-label">Maturity <HelpTip tooltip={HELP_DIM.maturity} position="bottom" /></span>
					<span class="i360-strip-value"><span class="i360-dot" style="background: {MATURITY_COLORS[data.maturity] ?? '#999'}"></span>{data.maturity}</span>
				</div>
				<div class="i360-strip-cell">
					<span class="i360-strip-label">Origin <HelpTip tooltip={HELP_DIM.origin} position="bottom" /></span>
					<span class="i360-strip-value"><span class="i360-dot" style="background: {ORIGIN_COLORS[data.origin_type] ?? '#999'}"></span>{data.origin_type} {'·'} {$t('inspector360.depth') || 'd'}{data.trust_depth}</span>
				</div>
				<div class="i360-strip-cell">
					<span class="i360-strip-label">Stage <HelpTip tooltip={HELP_DIM.stage} position="bottom" /></span>
					<span class="i360-strip-value">{STAGE_ICONS[data.stage] || ''} {data.stage || 'none'}</span>
				</div>
				<div class="i360-strip-cell">
					<span class="i360-strip-label">Review <HelpTip tooltip={HELP_DIM.review} position="bottom" /></span>
					<span class="i360-strip-value">
						{#if data.is_due}<span class="i360-warn">Due</span>{:else if data.last_reviewed}{data.last_reviewed.slice(0, 10)}{:else}{'—'}{/if}
					</span>
				</div>
				{#if data.trails.length > 0}
					<div class="i360-strip-cell">
						<span class="i360-strip-label">Trails <HelpTip tooltip={HELP_DIM.trails} position="bottom" /></span>
						<span class="i360-strip-value">{'\u{1F6E4}️'} {data.trails.length}</span>
					</div>
				{/if}
				{#if data.lens_groups.length > 0}
					<div class="i360-strip-cell">
						<span class="i360-strip-label">Lenses <HelpTip tooltip={HELP_DIM.lenses} position="bottom" /></span>
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
							<span class="i360-corner-stratum">{'▲'} Stratum <HelpTip tooltip={HELP_AXIS_STRATUM} position="bottom" /></span>
							<span class="i360-corner-type">Type {'→'} <HelpTip tooltip={HELP_AXIS_TYPE} position="bottom" /></span>
						</div>
						{#each TYPE_ORDER as type}
							<div class="i360-col-header" style="--col-color: {TYPE_COLORS[type]}">
								<div class="i360-col-name">
									{typeLabels[type].toUpperCase()}
									<HelpTip tooltip={HELP_TYPE[type]} position="bottom" />
								</div>
								<div class="i360-col-count">{matrix.colTotals[type]}</div>
							</div>
						{/each}
						<div class="i360-rowtot-header">
							<span class="i360-grand-symbol">{'Σ'} <HelpTip tooltip={HELP_GRAND} position="bottom" /></span>
							<span class="i360-grand-value">{matrix.grandTotal}</span>
						</div>

						<!-- Data rows -->
						{#each STRATA as stratum}
							{@const isActive = activeStratum === stratum}
							{@const isEmptyRow = matrix.rowTotals[stratum] === 0}
							<div class="i360-row-header" class:active={isActive} class:empty-row={isEmptyRow && !isActive}>
								<span class="i360-row-num">L{stratum}</span>
								<span class="i360-row-name">{STRATUM_NAMES[stratum]}</span>
								<HelpTip tooltip={HELP_STRATUM[stratum]} position="top" />
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
					<span class="i360-hud-item">{'⬆'} {data.total_outbound} {$t('inspector360.outbound') || 'outbound'}</span>
					<span class="i360-hud-item">{'⬇'} {data.total_inbound} {$t('inspector360.inbound') || 'inbound'}</span>
					<span class="i360-hud-item">{'\u{1F4DD}'} {data.word_count.toLocaleString()} {$t('inspector360.words') || 'words'}</span>
				</div>
				<div class="i360-hud-right">
					{#if data.is_orphan}<span class="i360-hud-item i360-hud-warn">{'⚠'} {$t('inspector360.orphan') || 'Orphan'} <HelpTip tooltip={HELP_HUD.orphan} position="top" /></span>{/if}
					{#if data.single_point_of_failure}<span class="i360-hud-item i360-hud-warn">{'⚠'} {$t('inspector360.fragile') || 'Fragile'} <HelpTip tooltip={HELP_HUD.fragile} position="top" /></span>{/if}
					{#if data.missing_link_types.length > 0}<span class="i360-hud-item i360-hud-warn">{'⚠'} {data.missing_link_types.length} {$t('inspector360.blindSpots') || 'blind spots'} <HelpTip tooltip={HELP_HUD.blindSpots} position="top" /></span>{/if}
					{#if data.contradictions.length > 0}<span class="i360-hud-item i360-hud-warn">{'⚡'} {data.contradictions.length} {$t('inspector360.tensions') || 'tensions'} <HelpTip tooltip={HELP_HUD.tensions} position="top" /></span>{/if}
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
		font-size: 1.4rem;
		cursor: pointer;
		text-align: start;
	}
	.i360-back-bar:hover {
		background: var(--background-modifier-hover);
		color: var(--text-normal);
	}
	.i360-back-arrow { font-size: 1.6rem; line-height: 1; flex-shrink: 0; }
	.i360-back-name { flex: 1; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
	.i360-empty { text-align: center; padding: 32px 16px; }
	.i360-empty-icon { font-size: 3.5rem; margin-bottom: 12px; }
	.i360-empty-text { font-size: 1.5rem; color: var(--text-muted); }

	.i360-card { display: flex; flex-direction: column; gap: 14px; }
	.i360-card-name {
		font-size: 1.85rem; font-weight: 700; color: var(--text-normal);
		overflow: hidden; text-overflow: ellipsis; white-space: nowrap;
	}
	.i360-card-meta { display: flex; flex-wrap: wrap; gap: 8px; align-items: center; }
	.i360-stratum-pill {
		padding: 4px 12px; border-radius: 6px;
		background: color-mix(in srgb, var(--text-accent) 15%, transparent);
		color: var(--text-accent);
		font-size: 1.4rem; font-weight: 600;
	}
	.i360-pill {
		padding: 4px 12px; border-radius: 6px;
		font-size: 1.4rem; font-weight: 600;
	}
	.i360-pill-soft {
		padding: 4px 12px; border-radius: 6px;
		background: var(--background-secondary);
		color: var(--text-muted);
		font-size: 1.4rem;
	}
	.i360-card-counts {
		display: flex; gap: 16px;
		font-size: 1.45rem; color: var(--text-muted);
	}
	.i360-bars { display: flex; flex-direction: column; gap: 6px; }
	.i360-bar-row {
		display: grid;
		grid-template-columns: 130px 1fr 60px;
		align-items: center;
		gap: 10px;
		font-size: 1.4rem;
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
		font-size: 1.4rem;
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
	.i360-empty-icon-lg { font-size: 6rem; margin-bottom: 20px; opacity: 0.5; }
	.i360-empty-text-lg { font-size: 1.6rem; color: var(--text-muted); }

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
	.i360-header-icon { font-size: 40px; }
	.i360-header-label {
		font-size: 24px; color: var(--text-accent); font-weight: 700;
		letter-spacing: 3px; text-transform: uppercase;
	}
	.i360-header-name {
		font-size: 32px; font-weight: 700; color: var(--text-normal);
		overflow: hidden; text-overflow: ellipsis; white-space: nowrap;
	}
	.i360-header-right { display: flex; align-items: center; gap: 14px; flex-shrink: 0; }
	.i360-close {
		width: 48px; height: 48px; border-radius: 50%;
		border: 1px solid var(--background-modifier-border);
		background: var(--background-secondary);
		color: var(--text-muted); font-size: 26px; cursor: pointer;
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
		color: var(--text-normal); font-size: 17px;
		cursor: pointer; flex-shrink: 0;
		max-width: 360px;
	}
	.i360-back-full:hover {
		background: var(--background-modifier-hover);
	}
	.i360-back-full .i360-back-arrow { font-size: 22px; line-height: 1; flex-shrink: 0; }
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
		font-size: 16px; color: var(--text-faint);
		text-transform: uppercase; letter-spacing: 1.5px;
	}
	.i360-strip-value {
		font-size: 22px; color: var(--text-normal);
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
		font-size: 16px;
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
		font-size: 16px; color: var(--text-faint); font-weight: 600;
	}
	.i360-grand-value {
		font-size: 22px; color: var(--text-accent); font-weight: 700;
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
	.i360-col-name {
		font-size: 14px; font-weight: 700; letter-spacing: 1px;
		color: color-mix(in srgb, var(--col-color, currentColor) 55%, var(--text-normal));
		text-align: center;
		text-transform: uppercase;
		line-height: 1.2;
	}
	.i360-col-count {
		font-size: 20px; font-weight: 700;
		color: color-mix(in srgb, var(--col-color, currentColor) 55%, var(--text-normal));
		font-variant-numeric: tabular-nums;
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
		color: var(--text-accent); font-weight: 700; font-size: 20px;
		width: 38px; flex-shrink: 0;
	}
	.i360-row-name {
		color: var(--text-normal); font-size: 18px;
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
		font-size: 14px; color: var(--text-muted);
		font-weight: 600; padding: 0 3px;
	}
	.i360-overflow-btn {
		font-size: 14px; font-weight: 600;
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
		font-size: 16px; line-height: 1;
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
		font-size: 13px;
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
		color: var(--text-muted); font-size: 22px;
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
		color: var(--text-normal); font-size: 17px;
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
		font-size: 21px; color: var(--text-muted);
		display: flex; align-items: center; gap: 8px;
	}
	.i360-hud-warn { color: var(--text-error, #ef4444); }
</style>
