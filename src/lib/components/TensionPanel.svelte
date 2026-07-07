<script lang="ts">
	import { t, tn } from '$lib/i18n';
	import { openNoteTab } from '$lib/libraries/store';
	import RelatedCandidates from './RelatedCandidates.svelte'; // MIG-086 §D — suggest + one-click typed link

	interface TensionItem {
		note_name: string;
		note_path: string;
		severity: string;
		detail: string;              // English fallback
		detail_kind?: string;        // MIG-080 §E — localization handle
		detail_args?: string[];
	}
	interface GapItem {
		tag: string;
		notes: string[];             // top-5 display names
		severity: string;
		member_names?: string[];     // MIG-080 §E — full membership (for note-scoped filtering)
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
	// MIG-095 — the note's OWN health (per-note canonical verdicts, ungated by the
	// library-wide 50-link monitor). Mirrors the Rust NoteTensionStatus.
	interface NoteStatus {
		indexed: boolean;
		ambiguous_title: boolean;
		is_orphan: boolean;
		is_fragile: boolean;
		is_contested: boolean;
		incoming_count: number;
		outgoing_count: number;
		word_count: number;
		derives_support: number;
		contested_with: string[];
	}

	let {
		report = null as TensionReport | null,
		loading = false,
		libraryColorMap = {} as Record<string, string>,
		onNoteClick,
		noteContext = null as string | null,
		noteStatus = null as NoteStatus | null,
		notePath = null as string | null,     // MIG-086 §D — the open note's path (for <RelatedCandidates>)
		libraryPath = null as string | null,  // MIG-086 §D — the open note's library path
	}: {
		report?: TensionReport | null;
		loading?: boolean;
		libraryColorMap?: Record<string, string>;
		onNoteClick?: (path: string, name: string) => void;
		notePath?: string | null;
		libraryPath?: string | null;
		/** MIG-080 §E — the open note's name when this panel is the note-scoped
		 *  right-rail Health tab (report already filtered to this note). When set,
		 *  empty sections are hidden and a positive "no tensions" state replaces the
		 *  four empty headers. Null = the library-wide framing (unchanged). */
		noteContext?: string | null;
		/** MIG-080 §E — reliability of the note-scoped verdict. `indexed:false` →
		 *  the note isn't in the analysed set (new / still indexing); `ambiguous_title`
		 *  → another note shares its title (name-keyed detection can't attribute
		 *  reliably). Either gates the positive "healthy" state into an honest one.
		 *  MIG-095 — now also carries the note's OWN verdicts (orphan/fragile/contested)
		 *  so the note tab shows this note's health regardless of the library-wide gate. */
		noteStatus?: NoteStatus | null;
	} = $props();

	// MIG-095 — the open note's own health, from the ungated per-note verdicts. Drives the
	// note tab even when the library-wide monitor is inactive (< 50 linked notes).
	const noteOwnFlags = $derived(
		noteContext && noteStatus?.indexed
			? {
				orphan: noteStatus.is_orphan,
				fragile: noteStatus.is_fragile,
				contested: noteStatus.is_contested,
				contestedWith: noteStatus.contested_with ?? [],
				incoming: noteStatus.incoming_count,
				derives: noteStatus.derives_support,
				words: noteStatus.word_count,
			}
			: null,
	);
	const noteOwnHealthy = $derived(
		!!noteOwnFlags && !noteOwnFlags.orphan && !noteOwnFlags.fragile && !noteOwnFlags.contested,
	);

	// MIG-080 §E — does this (note-scoped) report carry any tension at all?
	const noteHasTensions = $derived(
		!!report &&
			report.contradictions.length +
				report.orphans.length +
				report.structural_gaps.length +
				report.single_points.length >
				0,
	);

	// MIG-086 §D — does THIS note (note-scoped mode only) carry a CONNECTION-fixable gap?
	// Orphan / SPOF / structural-gap are healed by adding links; contradictions are not, so
	// they're excluded from the gate. Library-wide mode (noteContext null) never shows the
	// suggest block — there's no single note in hand to connect. Needs the path + lib too.
	const noteNeedsConnections = $derived(
		!!noteContext && !!notePath && !!libraryPath && !!report &&
			report.orphans.length + report.single_points.length + report.structural_gaps.length > 0,
	);
	const noteIsSPOF = $derived(!!report && report.single_points.length > 0);

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

	// MIG-080 §E — render the localized tension detail from the item's structured
	// kind + args ($t, ×15). Falls back to the English `detail` the Rust side still
	// carries (the test oracle) if the kind is unknown.
	function localizedDetail(item: TensionItem): string {
		const a: string[] = item?.detail_args ?? [];
		switch (item?.detail_kind) {
			case 'contradicts':
				return ($t('tensionPanel.detail.contradicts') || 'contradicts “{0}”').replace('{0}', a[0] ?? '');
			case 'contradicted_by':
				return ($t('tensionPanel.detail.contradictedBy') || 'contradicted by “{0}”').replace('{0}', a[0] ?? '');
			case 'orphan':
				return ($t('tensionPanel.detail.orphan') || '{0} words, no inbound links').replace('{0}', a[0] ?? '');
			case 'single_point':
				return ($t('tensionPanel.detail.singlePoint') || 'referenced by {0} notes, only {1} source')
					.replace('{0}', a[0] ?? '').replace('{1}', a[1] ?? '');
			default:
				return item?.detail ?? '';
		}
	}
</script>

<div class="tension-panel">
	{#if loading}
		<div class="tp-empty">{noteContext ? ($t('tensionPanel.analyzingNote') || 'Analyzing this note’s health…') : ($t('tensionPanel.analyzing') || 'Analyzing library…')}</div>
	{:else if !report}
		<!-- Distinct from `analyzing`: the load finished without a report
		     (error or no run yet). Never an eternal "Loading…". -->
		<div class="tp-empty">{$t('tensionPanel.unavailable') || 'Analysis unavailable — switch tabs and back to retry.'}</div>
	{:else if noteContext && noteStatus?.indexed && !report.active}
		<!-- MIG-095 — the library-wide tension monitor activates at 50 linked notes, but THIS
		     note's own health (orphan / fragile / contested) is a per-note canonical read
		     (MIG-094) and is always shown, regardless of the library's link count. -->
		{#if noteStatus.ambiguous_title}
			<div class="tp-caveat">⚠ {$t('tensionPanel.noteAmbiguous') || 'This note shares its title with another — its health can’t be reliably assessed.'}</div>
		{/if}
		{#if noteOwnHealthy}
			<div class="tp-healthy">
				<div class="tp-healthy-icon">✓</div>
				<div class="tp-healthy-text">{$t('tensionPanel.noteHealthy') || 'No tensions detected for this note — it’s well-connected.'}</div>
			</div>
		{:else}
			<div class="tp-section">
				{#if noteOwnFlags?.orphan}
					<div class="tp-item tp-self">
						<span class="tp-dot" style="background:#f59e0b"></span>
						<span class="tp-name">{$t('tensionPanel.orphans') || 'Orphan Notes'}</span>
						<span class="tp-detail">{($t('tensionPanel.detail.orphan') || '{0} words, no inbound links').replace('{0}', String(noteOwnFlags?.words ?? 0))}</span>
					</div>
				{/if}
				{#if noteOwnFlags?.fragile}
					<div class="tp-item tp-self">
						<span class="tp-dot" style="background:#ef4444"></span>
						<span class="tp-name">{$t('tensionPanel.singlePoints') || 'Single Points of Failure'}</span>
						<span class="tp-detail">{($t('tensionPanel.detail.singlePoint') || 'referenced by {0} notes, only {1} source').replace('{0}', String(noteOwnFlags?.incoming ?? 0)).replace('{1}', String(noteOwnFlags?.derives ?? 0))}</span>
					</div>
				{/if}
				{#if noteOwnFlags?.contested}
					<div class="tp-item tp-self">
						<span class="tp-dot" style="background:#ef4444"></span>
						<span class="tp-name">{$t('tensionPanel.contradictions') || 'Contradictions'}</span>
						<span class="tp-detail" dir="auto">{(noteOwnFlags?.contestedWith ?? []).join(', ')}</span>
					</div>
				{/if}
			</div>
		{/if}
		<div class="tp-inactive-count" style="margin-top:12px;">{report.total_linked_notes} / 50 {$t('tensionPanel.linkedNotes') || 'linked notes'}</div>
	{:else if !report.active}
		<div class="tp-inactive">
			<div class="tp-inactive-icon">🩺</div>
			<div class="tp-inactive-text">{$t('tensionPanel.inactive') || 'Add more links to activate knowledge health monitoring.'}</div>
			<div class="tp-inactive-count">{report.total_linked_notes} / 50 {$t('tensionPanel.linkedNotes') || 'linked notes'}</div>
		</div>
	{:else if noteContext && noteStatus && !noteStatus.indexed}
		<!-- MIG-080 §E (#9) — not in the analysed set (newly created / still indexing).
		     Don't claim "healthy" — say so honestly. -->
		<div class="tp-healthy">
			<div class="tp-healthy-icon" style="color: var(--text-faint);">…</div>
			<div class="tp-healthy-text">{$t('tensionPanel.noteNotAnalyzed') || 'This note isn’t analyzed yet — it may be newly created or still indexing.'}</div>
		</div>
	{:else if noteContext && !noteHasTensions && !noteStatus}
		<!-- §E-fix #1 — the report slice is synchronous but noteStatus is an async
		     lookup; until it resolves, show pending rather than a (possibly false)
		     clean verdict against the previous note's stale status. -->
		<div class="tp-empty">{$t('tensionPanel.analyzingNote') || 'Analyzing this note’s health…'}</div>
	{:else if noteContext && !noteHasTensions && noteStatus?.ambiguous_title}
		<!-- MIG-080 §E (#10) — shares its title with another note; the name-keyed
		     detection can't reliably attribute tensions, so "clean" isn't trustworthy. -->
		<div class="tp-healthy">
			<div class="tp-healthy-icon" style="color: #f59e0b;">⚠</div>
			<div class="tp-healthy-text">{$t('tensionPanel.noteAmbiguous') || 'This note shares its title with another — its health can’t be reliably assessed.'}</div>
		</div>
	{:else if noteContext && !noteHasTensions}
		<!-- MIG-080 §E — note-scoped & clean: a positive state instead of four
		     empty section headers (which read as "broken" for a healthy note). -->
		<div class="tp-healthy">
			<div class="tp-healthy-icon">✓</div>
			<div class="tp-healthy-text">{$t('tensionPanel.noteHealthy') || 'No tensions detected for this note — it’s well-connected.'}</div>
		</div>
	{:else}
		{#if noteContext && noteStatus?.ambiguous_title}
			<!-- §E-fix #4 — a note WITH tensions that ALSO shares its title: caveat the
			     rows (they may belong to the same-titled sibling) instead of showing them bare. -->
			<div class="tp-caveat">⚠ {$t('tensionPanel.noteAmbiguous') || 'This note shares its title with another — its health can’t be reliably assessed.'}</div>
		{/if}
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
						<span class="tp-name" dir="auto">{item.note_name}</span>
						<span class="tp-detail">{localizedDetail(item)}</span>
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
						<span class="tp-name" dir="auto">{item.note_name}</span>
						<span class="tp-detail">{localizedDetail(item)}</span>
					</button>
				{:else}
					<div class="tp-none">{$t('tensionPanel.none') || 'None found'}</div>
				{/each}
				{#if report.orphans.length > 30}
					<div class="tp-more">+{$tn('plurals.more', report.orphans.length - 30)}</div>
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
						<span class="tp-detail" dir="auto">{gap.notes.join(', ')}</span>
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
						<span class="tp-name" dir="auto">{item.note_name}</span>
						<span class="tp-detail">{localizedDetail(item)}</span>
					</button>
				{:else}
					<div class="tp-none">{$t('tensionPanel.none') || 'None found'}</div>
				{/each}
			{/if}
		</div>

		<!-- MIG-086 §D — surface #4: turn the diagnosis into an action. When THIS note is
		     flagged orphan / SPOF / in a structural gap, offer the same suggest + one-click
		     typed-link block beneath the tension rows. Direction INBOUND (suggestion → this
		     note) — the new link gives the note an incoming connection, clearing the flag on
		     the next analysis pass. SPOF pre-sets derives-from + a "shore it up" heading. -->
		{#if noteNeedsConnections}
			<div class="tp-suggest">
				<RelatedCandidates
					{notePath}
					noteName={noteContext}
					{libraryPath}
					direction="inbound"
					defaultType={noteIsSPOF ? 'derives-from' : 'associative'}
					heading={noteIsSPOF ? ($t('reviewer.suggestLabelFragile') || 'Shore it up — connect to:') : null}
				/>
			</div>
		{/if}
	{/if}
</div>

<style>
	/* Self-scrolling: the right-sidebar host (.rs-full-height) is a flex
	   column with overflow:hidden — children own their scroll, and this
	   panel can hold hundreds of rows. */
	.tension-panel { padding: 8px 0; flex: 1; min-height: 0; overflow-y: auto; }
	.tp-empty, .tp-none, .tp-more { font-size: calc(0.78rem * var(--rs-scale, 1)); color: var(--text-faint); padding: 4px 12px; }
	.tp-inactive { text-align: center; padding: 24px 16px; }
	.tp-inactive-icon { font-size: calc(2rem * var(--rs-scale, 1)); margin-bottom: 8px; }
	.tp-inactive-text { font-size: calc(0.82rem * var(--rs-scale, 1)); color: var(--text-muted); line-height: 1.4; }
	.tp-inactive-count { font-size: calc(0.75rem * var(--rs-scale, 1)); color: var(--text-faint); margin-top: 8px; }
	/* MIG-080 §E — note-scoped clean state */
	.tp-healthy { text-align: center; padding: 28px 16px; }
	.tp-healthy-icon { font-size: calc(1.8rem * var(--rs-scale, 1)); color: #10b981; margin-bottom: 8px; line-height: 1; }
	.tp-healthy-text { font-size: calc(0.82rem * var(--rs-scale, 1)); color: var(--text-muted); line-height: 1.4; }
	/* MIG-080 §E-fix #4 — reliability caveat above the tension rows of an ambiguous-titled note */
	.tp-caveat { font-size: calc(0.74rem * var(--rs-scale, 1)); color: #f59e0b; padding: 6px 12px; margin-bottom: 4px; line-height: 1.3; }
	/* MIG-086 §D — the inline suggest block beneath the tension rows. Side inset matches the
	   rows (12px); RelatedCandidates owns its own internal scroll for long candidate lists. */
	.tp-suggest { padding: 0 12px 8px; }
	.tp-section { margin-bottom: 4px; }
	.tp-header {
		display: flex; align-items: center; gap: 6px; width: 100%;
		padding: 6px 12px; border: none; background: none; cursor: pointer;
		font-size: calc(0.78rem * var(--rs-scale, 1)); font-weight: 600; color: var(--text-normal); font-family: inherit;
		text-align: start;
	}
	.tp-header:hover { background: var(--background-modifier-hover); }
	.tp-chevron { font-size: calc(0.65rem * var(--rs-scale, 1)); transition: transform 0.15s; flex-shrink: 0; }
	.tp-chevron.collapsed { transform: rotate(-90deg); }
	:global([dir="rtl"]) .tp-chevron.collapsed { transform: rotate(90deg); }
	.tp-count { margin-inline-start: auto; font-size: calc(0.7rem * var(--rs-scale, 1)); color: var(--text-faint); font-weight: 400; }
	.tp-item, .tp-gap {
		display: flex; align-items: center; gap: 6px; width: 100%;
		padding: 4px 12px; padding-inline-start: 24px; border: none; background: none; cursor: pointer;
		font-size: calc(0.78rem * var(--rs-scale, 1)); color: var(--text-normal); font-family: inherit; text-align: start;
	}
	.tp-item:hover { background: var(--background-modifier-hover); }
	.tp-dot { width: 6px; height: 6px; border-radius: 50%; flex-shrink: 0; }
	.tp-name { font-weight: 500; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; max-width: 140px; }
	.tp-tag { color: var(--interactive-accent); font-weight: 500; white-space: nowrap; }
	.tp-detail { font-size: calc(0.72rem * var(--rs-scale, 1)); color: var(--text-faint); white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }
</style>
