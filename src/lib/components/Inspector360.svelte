<script lang="ts">
	import { t } from '$lib/i18n';

	interface LinkedNote { name: string; path: string; depth: number; }
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
	}: {
		data?: Note360View | null;
		compact?: boolean;
		onNoteClick?: (path: string, name: string) => void;
	} = $props();

	// Link type → angle sector (in degrees, 0 = top/north)
	const SECTOR_MAP: Record<string, { angle: number; color: string; label: string }> = {
		supports:      { angle: 0,   color: '#4A9EFF', label: 'Supports' },
		contradicts:   { angle: 180, color: '#FF4A4A', label: 'Contradicts' },
		causes:        { angle: 90,  color: '#FF8C42', label: 'Causes' },
		'derives-from':{ angle: 270, color: '#FFD700', label: 'Derives From' },
		generalizes:   { angle: 45,  color: '#A44AFF', label: 'Generalizes' },
		exemplifies:   { angle: 315, color: '#4AFF88', label: 'Exemplifies' },
		'part-of':     { angle: 135, color: '#AAAAAA', label: 'Part Of' },
	};

	const MATURITY_COLORS: Record<string, string> = {
		seed: '#9ca3af', sapling: '#4ade80', evergreen: '#16a34a', canonical: '#f59e0b', wilting: '#16a34a80',
	};
	const ORIGIN_COLORS: Record<string, string> = {
		received: '#4A9EFF', discovered: '#FFB347', mixed: '#A78BFA', none: '#9ca3af',
	};

	const size = $derived(compact ? 280 : 500);
	const cx = $derived(size / 2);
	const cy = $derived(size / 2);
	const ringRadii = $derived([size * 0.2, size * 0.32, size * 0.44]);

	function polarToXY(angle: number, radius: number): { x: number; y: number } {
		const rad = (angle - 90) * Math.PI / 180;
		return { x: cx + radius * Math.cos(rad), y: cy + radius * Math.sin(rad) };
	}
</script>

<div class="i360" class:compact>
	{#if !data}
		<div class="i360-empty">
			<div class="i360-empty-icon">🔮</div>
			<div class="i360-empty-text">{$t('inspector360.noData') || 'Open a note to see its 360° view'}</div>
		</div>
	{:else}
		<svg viewBox="0 0 {size} {size}" class="i360-svg">
			<!-- Concentric rings -->
			{#each ringRadii as r, i}
				<circle cx={cx} cy={cy} r={r} fill="none" stroke="rgba(0,0,0,0.06)" stroke-width="1" />
			{/each}

			<!-- Sector lines (dashed for missing types, solid for used) -->
			{#each Object.entries(SECTOR_MAP) as [type, info]}
				{@const isUsed = data.used_link_types.includes(type)}
				{@const endPos = polarToXY(info.angle, ringRadii[2] + 15)}
				<line x1={cx} y1={cy} x2={endPos.x} y2={endPos.y}
					stroke={isUsed ? info.color : 'rgba(0,0,0,0.08)'}
					stroke-width={isUsed ? 1.5 : 0.5}
					stroke-dasharray={isUsed ? 'none' : '4,4'} />
				<!-- Sector label -->
				{@const labelPos = polarToXY(info.angle, ringRadii[2] + (compact ? 22 : 30))}
				<text x={labelPos.x} y={labelPos.y} text-anchor="middle" dominant-baseline="central"
					font-size={compact ? '7' : '9'} fill={isUsed ? info.color : 'rgba(0,0,0,0.2)'}
					font-weight={isUsed ? '600' : '400'}>
					{$t(`inspector360.link_${type.replace('-', '_')}`) || info.label}
				</text>
			{/each}

			<!-- Connected nodes -->
			{#each Object.entries(data.typed_links) as [type, notes]}
				{@const info = SECTOR_MAP[type]}
				{#if info}
					{#each notes as note, i}
						{@const spread = notes.length > 1 ? (i - (notes.length - 1) / 2) * 12 : 0}
						{@const ring = note.depth <= 1 ? 0 : note.depth <= 2 ? 1 : 2}
						{@const pos = polarToXY(info.angle + spread, ringRadii[ring])}
						<circle cx={pos.x} cy={pos.y} r={compact ? 4 : 6}
							fill={info.color} opacity="0.8" cursor="pointer"
							onclick={() => onNoteClick?.(note.path, note.name)}>
							<title>{note.name} ({type}, depth {note.depth})</title>
						</circle>
					{/each}
				{/if}
			{/each}

			<!-- Untyped links (scattered around outer ring) -->
			{#each data.untyped_links.slice(0, 20) as note, i}
				{@const angle = (i / Math.max(data.untyped_links.length, 1)) * 360}
				{@const ring = note.depth <= 1 ? 0 : note.depth <= 2 ? 1 : 2}
				{@const pos = polarToXY(angle, ringRadii[ring])}
				<circle cx={pos.x} cy={pos.y} r={compact ? 3 : 4}
					fill="#888" opacity="0.4" cursor="pointer"
					onclick={() => onNoteClick?.(note.path, note.name)}>
					<title>{note.name} (untyped, depth {note.depth})</title>
				</circle>
			{/each}

			<!-- Center: note name + stratum -->
			<circle cx={cx} cy={cy} r={compact ? 24 : 36} fill="white" stroke="rgba(0,0,0,0.1)" stroke-width="2" />
			<text x={cx} y={cy - (compact ? 4 : 6)} text-anchor="middle" dominant-baseline="central"
				font-size={compact ? '8' : '11'} font-weight="700" fill="#333">
				{data.note_name.length > (compact ? 12 : 20) ? data.note_name.slice(0, compact ? 10 : 18) + '…' : data.note_name}
			</text>
			<text x={cx} y={cy + (compact ? 8 : 12)} text-anchor="middle" dominant-baseline="central"
				font-size={compact ? '7' : '9'} fill="#999">
				L{data.stratum}
			</text>

			<!-- Badges around the sphere -->
			<!-- Maturity -->
			<circle cx={size - 20} cy={20} r={compact ? 6 : 8}
				fill={MATURITY_COLORS[data.maturity] ?? '#999'} opacity="0.8" />
			<text x={size - 20} y={compact ? 36 : 38} text-anchor="middle"
				font-size={compact ? '6' : '7'} fill="#999">{data.maturity}</text>

			<!-- Origin -->
			<circle cx={20} cy={20} r={compact ? 6 : 8}
				fill={ORIGIN_COLORS[data.origin_type] ?? '#999'} opacity="0.8" />
			<text x={20} y={compact ? 36 : 38} text-anchor="middle"
				font-size={compact ? '6' : '7'} fill="#999">{data.origin_type}</text>

			<!-- Stage -->
			{#if data.stage}
				<text x={20} y={size - 12} text-anchor="start"
					font-size={compact ? '7' : '8'} fill="#666">
					{data.stage === 'fleeting' ? '🌱' : data.stage === 'literature' ? '📖' : data.stage === 'permanent' ? '🔗' : '✨'}
				</text>
			{/if}

			<!-- Review due -->
			{#if data.is_due}
				<circle cx={size - 20} cy={size - 20} r={compact ? 5 : 7} fill="#ef4444" opacity="0.7" />
				<text x={size - 20} y={size - (compact ? 8 : 6)} text-anchor="middle"
					font-size="6" fill="#ef4444">due</text>
			{/if}

			<!-- Gap indicator -->
			{#if data.missing_link_types.length > 0}
				<text x={cx} y={size - 8} text-anchor="middle"
					font-size={compact ? '6' : '8'} fill="rgba(0,0,0,0.25)">
					{data.missing_link_types.length} {$t('inspector360.gaps') || 'gaps'}
				</text>
			{/if}
		</svg>

		<!-- Stats bar -->
		{#if !compact}
		<div class="i360-stats">
			<span>⬆{data.total_outbound} ⬇{data.total_inbound}</span>
			<span>📝{data.word_count}</span>
			{#if data.trails.length > 0}<span>🛤️{data.trails.length}</span>{/if}
			{#if data.lens_groups.length > 0}<span>🏷️{data.lens_groups.length}</span>{/if}
			{#if data.is_orphan}<span class="i360-warn">⚠️ orphan</span>{/if}
			{#if data.single_point_of_failure}<span class="i360-warn">⚠️ fragile</span>{/if}
		</div>
		{/if}
	{/if}
</div>

<style>
	.i360 { display: flex; flex-direction: column; align-items: center; padding: 8px; }
	.i360.compact { padding: 4px; }
	.i360-svg { width: 100%; max-width: 500px; height: auto; }
	.i360.compact .i360-svg { max-width: 280px; }
	.i360-empty { text-align: center; padding: 24px; }
	.i360-empty-icon { font-size: 2rem; margin-bottom: 8px; }
	.i360-empty-text { font-size: 0.82rem; color: var(--text-muted); }
	.i360-stats {
		display: flex; gap: 10px; padding: 8px 0; font-size: 0.75rem; color: var(--text-muted);
		flex-wrap: wrap; justify-content: center;
	}
	.i360-warn { color: #ef4444; font-weight: 600; }
</style>
