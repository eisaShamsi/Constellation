<script lang="ts">
	import { t } from '$lib/i18n';
	import { openNoteTab } from '$lib/libraries/store';

	interface TensionItem {
		note_name: string;
		note_path: string;
		severity: string;
		detail: string;
	}
	interface GapItem {
		tag: string;
		notes: string[];
		severity: string;
	}
	interface TensionReport {
		contradictions: TensionItem[];
		orphans: TensionItem[];
		structural_gaps: GapItem[];
		single_points: TensionItem[];
		total_linked_notes: number;
		total_notes: number;
		active: boolean;
	}

	let {
		report = null as TensionReport | null,
		loading = false,
		libraryColorMap = {} as Record<string, string>,
		onNoteClick,
	}: {
		report?: TensionReport | null;
		loading?: boolean;
		libraryColorMap?: Record<string, string>;
		onNoteClick?: (path: string, name: string) => void;
	} = $props();

	let showContradictions = $state(true);
	let showOrphans = $state(true);
	let showGaps = $state(true);
	let showSinglePoints = $state(true);

	function handleClick(path: string, name: string) {
		onNoteClick?.(path, name);
	}

	function severityDot(severity: string): string {
		return severity === 'high' ? '#ef4444' : severity === 'medium' ? '#f59e0b' : '#9ca3af';
	}
</script>

<div class="tension-panel">
	{#if loading}
		<div class="tp-empty">{$t('tensionPanel.analyzing') || 'Analyzing library…'}</div>
	{:else if !report}
		<!-- Distinct from `analyzing`: the load finished without a report
		     (error or no run yet). Never an eternal "Loading…". -->
		<div class="tp-empty">{$t('tensionPanel.unavailable') || 'Analysis unavailable — switch tabs and back to retry.'}</div>
	{:else if !report.active}
		<div class="tp-inactive">
			<div class="tp-inactive-icon">🩺</div>
			<div class="tp-inactive-text">{$t('tensionPanel.inactive') || 'Add more links to activate knowledge health monitoring.'}</div>
			<div class="tp-inactive-count">{report.total_linked_notes} / 50 {$t('tensionPanel.linkedNotes') || 'linked notes'}</div>
		</div>
	{:else}
		<!-- Contradictions -->
		<div class="tp-section">
			<button class="tp-header" onclick={() => showContradictions = !showContradictions}>
				<span class="tp-chevron" class:collapsed={!showContradictions}>▾</span>
				<span>{$t('tensionPanel.contradictions') || 'Contradictions'}</span>
				<span class="tp-count">{report.contradictions.length}</span>
			</button>
			{#if showContradictions}
				{#each report.contradictions as item}
					<button class="tp-item" onclick={() => handleClick(item.note_path, item.note_name)}>
						<span class="tp-dot" style="background:{severityDot(item.severity)}"></span>
						<span class="tp-name">{item.note_name}</span>
						<span class="tp-detail">{item.detail}</span>
					</button>
				{:else}
					<div class="tp-none">{$t('tensionPanel.none') || 'None found'}</div>
				{/each}
			{/if}
		</div>

		<!-- Orphans -->
		<div class="tp-section">
			<button class="tp-header" onclick={() => showOrphans = !showOrphans}>
				<span class="tp-chevron" class:collapsed={!showOrphans}>▾</span>
				<span>{$t('tensionPanel.orphans') || 'Orphan Notes'}</span>
				<span class="tp-count">{report.orphans.length}</span>
			</button>
			{#if showOrphans}
				{#each report.orphans.slice(0, 30) as item}
					<button class="tp-item" onclick={() => handleClick(item.note_path, item.note_name)}>
						<span class="tp-dot" style="background:{severityDot(item.severity)}"></span>
						<span class="tp-name">{item.note_name}</span>
						<span class="tp-detail">{item.detail}</span>
					</button>
				{:else}
					<div class="tp-none">{$t('tensionPanel.none') || 'None found'}</div>
				{/each}
				{#if report.orphans.length > 30}
					<div class="tp-more">+{report.orphans.length - 30} {$t('tensionPanel.more') || 'more'}</div>
				{/if}
			{/if}
		</div>

		<!-- Structural Gaps -->
		<div class="tp-section">
			<button class="tp-header" onclick={() => showGaps = !showGaps}>
				<span class="tp-chevron" class:collapsed={!showGaps}>▾</span>
				<span>{$t('tensionPanel.structuralGaps') || 'Structural Gaps'}</span>
				<span class="tp-count">{report.structural_gaps.length}</span>
			</button>
			{#if showGaps}
				{#each report.structural_gaps as gap}
					<div class="tp-gap">
						<span class="tp-dot" style="background:{severityDot(gap.severity)}"></span>
						<span class="tp-tag">#{gap.tag}</span>
						<span class="tp-detail">{gap.notes.join(', ')}</span>
					</div>
				{:else}
					<div class="tp-none">{$t('tensionPanel.none') || 'None found'}</div>
				{/each}
			{/if}
		</div>

		<!-- Single Points of Failure -->
		<div class="tp-section">
			<button class="tp-header" onclick={() => showSinglePoints = !showSinglePoints}>
				<span class="tp-chevron" class:collapsed={!showSinglePoints}>▾</span>
				<span>{$t('tensionPanel.singlePoints') || 'Single Points of Failure'}</span>
				<span class="tp-count">{report.single_points.length}</span>
			</button>
			{#if showSinglePoints}
				{#each report.single_points as item}
					<button class="tp-item" onclick={() => handleClick(item.note_path, item.note_name)}>
						<span class="tp-dot" style="background:{severityDot(item.severity)}"></span>
						<span class="tp-name">{item.note_name}</span>
						<span class="tp-detail">{item.detail}</span>
					</button>
				{:else}
					<div class="tp-none">{$t('tensionPanel.none') || 'None found'}</div>
				{/each}
			{/if}
		</div>
	{/if}
</div>

<style>
	/* Self-scrolling: the right-sidebar host (.rs-full-height) is a flex
	   column with overflow:hidden — children own their scroll, and this
	   panel can hold hundreds of rows. */
	.tension-panel { padding: 8px 0; flex: 1; min-height: 0; overflow-y: auto; }
	.tp-empty, .tp-none, .tp-more { font-size: 0.78rem; color: var(--text-faint); padding: 4px 12px; }
	.tp-inactive { text-align: center; padding: 24px 16px; }
	.tp-inactive-icon { font-size: 2rem; margin-bottom: 8px; }
	.tp-inactive-text { font-size: 0.82rem; color: var(--text-muted); line-height: 1.4; }
	.tp-inactive-count { font-size: 0.75rem; color: var(--text-faint); margin-top: 8px; }
	.tp-section { margin-bottom: 4px; }
	.tp-header {
		display: flex; align-items: center; gap: 6px; width: 100%;
		padding: 6px 12px; border: none; background: none; cursor: pointer;
		font-size: 0.78rem; font-weight: 600; color: var(--text-normal); font-family: inherit;
		text-align: start;
	}
	.tp-header:hover { background: var(--background-modifier-hover); }
	.tp-chevron { font-size: 0.65rem; transition: transform 0.15s; flex-shrink: 0; }
	.tp-chevron.collapsed { transform: rotate(-90deg); }
	:global([dir="rtl"]) .tp-chevron.collapsed { transform: rotate(90deg); }
	.tp-count { margin-inline-start: auto; font-size: 0.7rem; color: var(--text-faint); font-weight: 400; }
	.tp-item, .tp-gap {
		display: flex; align-items: center; gap: 6px; width: 100%;
		padding: 4px 12px 4px 24px; border: none; background: none; cursor: pointer;
		font-size: 0.78rem; color: var(--text-normal); font-family: inherit; text-align: start;
	}
	.tp-item:hover { background: var(--background-modifier-hover); }
	.tp-dot { width: 6px; height: 6px; border-radius: 50%; flex-shrink: 0; }
	.tp-name { font-weight: 500; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; max-width: 140px; }
	.tp-tag { color: var(--interactive-accent); font-weight: 500; white-space: nowrap; }
	.tp-detail { font-size: 0.72rem; color: var(--text-faint); white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }
</style>
