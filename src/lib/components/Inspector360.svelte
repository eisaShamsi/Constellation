<script lang="ts">
	import { t } from '$lib/i18n';
	import { detectDir } from '$lib/utils';

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

	// Visualization mode: user preference
	let vizMode = $state<'atmospheric' | 'neural' | 'cosmic'>('atmospheric');

	// Link type → sector config
	const SECTOR_MAP: Record<string, { angle: number; color: string; labelKey: string }> = {
		supports:      { angle: 0,   color: '#4A9EFF', labelKey: 'link_supports' },
		contradicts:   { angle: 180, color: '#FF4A4A', labelKey: 'link_contradicts' },
		causes:        { angle: 90,  color: '#FF8C42', labelKey: 'link_causes' },
		'derives-from':{ angle: 270, color: '#FFD700', labelKey: 'link_derives_from' },
		generalizes:   { angle: 45,  color: '#A44AFF', labelKey: 'link_generalizes' },
		exemplifies:   { angle: 315, color: '#4AFF88', labelKey: 'link_exemplifies' },
		'part-of':     { angle: 135, color: '#AAAAAA', labelKey: 'link_part_of' },
	};

	const MATURITY_COLORS: Record<string, string> = {
		seed: '#9ca3af', sapling: '#4ade80', evergreen: '#16a34a', canonical: '#f59e0b', wilting: '#16a34a80',
	};
	const ORIGIN_COLORS: Record<string, string> = {
		received: '#4A9EFF', discovered: '#FFB347', mixed: '#A78BFA', none: '#9ca3af',
	};
	const STAGE_ICONS: Record<string, string> = {
		fleeting: '\u{1F331}', literature: '\u{1F4D6}', permanent: '\u{1F517}', synthesis: '\u2728',
	};

	// --- Common helpers ---
	function polarToXY(cx: number, cy: number, angle: number, radius: number): { x: number; y: number } {
		const rad = (angle - 90) * Math.PI / 180;
		return { x: cx + radius * Math.cos(rad), y: cy + radius * Math.sin(rad) };
	}

	function truncName(name: string, max: number): string {
		return name.length > max ? name.slice(0, max - 1) + '\u2026' : name;
	}

	// Ring-per-group layout (§102). Each typed-link group occupies its own
	// concentric ring around the core. Groups are sorted by note count
	// ascending: smaller groups on inner rings (closer to the core),
	// larger groups on outer rings (more circumference for many nodes).
	// Untyped links are treated as one additional group on whichever ring
	// their count places them. Within a ring, nodes are spread evenly
	// around the full 360°. §104: dedupe by path within each group — the
	// IPC returns the same note from outbound + inbound + second-order
	// sources, which would otherwise render the same dot multiple times.
	const ringsLayout = $derived.by(() => {
		if (!data) return [] as Array<{ type: string; color: string; labelKey: string; radius: number; notes: LinkedNote[] }>;
		type Group = { type: string; color: string; labelKey: string; radius: number; notes: LinkedNote[] };
		const groups: Group[] = [];

		const dedupeByPath = (notes: LinkedNote[]): LinkedNote[] => {
			const seen = new Set<string>();
			const unique: LinkedNote[] = [];
			for (const note of notes) {
				const key = note.path || note.name;
				if (seen.has(key)) continue;
				seen.add(key);
				unique.push(note);
			}
			return unique;
		};

		for (const [type, noteList] of Object.entries(data.typed_links)) {
			const info = SECTOR_MAP[type];
			if (!info) continue;
			const unique = dedupeByPath(noteList);
			if (unique.length === 0) continue;
			groups.push({ type, color: info.color, labelKey: info.labelKey, radius: 0, notes: unique });
		}
		if (data.untyped_links.length > 0) {
			const unique = dedupeByPath(data.untyped_links).slice(0, 30);
			if (unique.length > 0) {
				groups.push({ type: 'untyped', color: '#888', labelKey: 'untyped', radius: 0, notes: unique });
			}
		}

		// Smaller groups inner, larger groups outer.
		groups.sort((a, b) => a.notes.length - b.notes.length);

		const minRadius = 110;
		const maxRadius = 380;
		const n = groups.length;
		for (let g = 0; g < n; g++) {
			groups[g].radius = n === 1
				? (minRadius + maxRadius) / 2
				: minRadius + (maxRadius - minRadius) * (g / (n - 1));
		}
		return groups;
	});

	// §104: layout mode is hybrid. When every typed-link group fits cleanly
	// in its sector (largest typed group ≤ SECTOR_THRESHOLD), use the
	// original compass-position sector design with smaller nodes. Above the
	// threshold, fall back to the ring-per-group layout (§102) so dense
	// groups don't pile into pile-ups inside a 50° wedge.
	const SECTOR_THRESHOLD = 8;
	const layoutMode = $derived.by((): 'sector' | 'rings' => {
		let maxTypedCount = 0;
		for (const ring of ringsLayout) {
			if (ring.type !== 'untyped' && ring.notes.length > maxTypedCount) {
				maxTypedCount = ring.notes.length;
			}
		}
		return maxTypedCount <= SECTOR_THRESHOLD ? 'sector' : 'rings';
	});

	const allNodes = $derived.by(() => {
		const nodes: Array<{ name: string; path: string; x: number; y: number; color: string; type: string; depth: number; r: number }> = [];
		const cx = 600; const cy = 400;
		// Minimised node radii (§103). Names are revealed on hover, not always-on.
		const radiusFor = (depth: number) => depth <= 1 ? 6 : depth <= 2 ? 4 : 3;

		if (layoutMode === 'sector') {
			// Sector layout: typed groups at SECTOR_MAP compass angles, depth
			// determines which of the three ring radii a node lands on, spread
			// within a 50° wedge per sector. Untyped nodes scatter around the
			// full circle on depth-based rings.
			const sectorRings = [160, 270, 380];
			const SECTOR_WIDTH = 50;
			for (const ring of ringsLayout) {
				const n = ring.notes.length;
				if (ring.type === 'untyped') {
					for (let i = 0; i < n; i++) {
						const note = ring.notes[i];
						const angle = (i / Math.max(n, 1)) * 360;
						const ringIndex = note.depth <= 1 ? 0 : note.depth <= 2 ? 1 : 2;
						const pos = polarToXY(cx, cy, angle, sectorRings[ringIndex]);
						nodes.push({ ...note, x: pos.x, y: pos.y, color: ring.color, type: ring.type, r: radiusFor(note.depth) });
					}
				} else {
					const info = SECTOR_MAP[ring.type];
					if (!info) continue;
					for (let i = 0; i < n; i++) {
						const note = ring.notes[i];
						const offset = n > 1 ? (i / (n - 1) - 0.5) * SECTOR_WIDTH : 0;
						const ringIndex = note.depth <= 1 ? 0 : note.depth <= 2 ? 1 : 2;
						const pos = polarToXY(cx, cy, info.angle + offset, sectorRings[ringIndex]);
						nodes.push({ ...note, x: pos.x, y: pos.y, color: info.color, type: ring.type, r: radiusFor(note.depth) });
					}
				}
			}
		} else {
			// Ring-per-group layout. Reserve a 30° clear gap at the top of each
			// ring for the type label.
			const reservedTop = 30;
			for (const ring of ringsLayout) {
				const n = ring.notes.length;
				for (let i = 0; i < n; i++) {
					const note = ring.notes[i];
					let angle: number;
					if (n === 1) {
						angle = 180;
					} else {
						angle = (reservedTop / 2) + i * ((360 - reservedTop) / (n - 1));
					}
					const pos = polarToXY(cx, cy, angle, ring.radius);
					nodes.push({ ...note, x: pos.x, y: pos.y, color: ring.color, type: ring.type, r: radiusFor(note.depth) });
				}
			}
		}
		return nodes;
	});

	// Hover state
	let hoveredNode = $state<string | null>(null);
</script>

{#if compact}
	<!-- ===== COMPACT SIDEBAR MODE ===== -->
	<div class="i360 compact">
		{#if previousNoteName && onBack}
			<button class="i360-back-bar" onclick={onBack} title={`Back to ${previousNoteName}`}>
				<span class="i360-back-arrow">{'←'}</span>
				<span class="i360-back-name" dir="auto">{truncName(previousNoteName, 22)}</span>
			</button>
		{/if}
		{#if !data}
			<div class="i360-empty">
				<div class="i360-empty-icon">{'\u{1F52E}'}</div>
				<div class="i360-empty-text">{$t('inspector360.noData') || 'Open a note to see its 360\u00B0 view'}</div>
			</div>
		{:else}
			<svg viewBox="0 0 280 280" class="i360-svg">
				{#each [56, 89.6, 123.2] as r}
					<circle cx={140} cy={140} {r} fill="none" stroke="rgba(167,139,250,0.15)" stroke-width="0.5" />
				{/each}
				{#each Object.entries(SECTOR_MAP) as [type, info]}
					{@const isUsed = data.used_link_types.includes(type)}
					{@const endPos = polarToXY(140, 140, info.angle, 138)}
					<line x1={140} y1={140} x2={endPos.x} y2={endPos.y}
						stroke={isUsed ? info.color : 'rgba(255,255,255,0.06)'}
						stroke-width={isUsed ? 1 : 0.3}
						stroke-dasharray={isUsed ? 'none' : '3,3'}
						opacity={isUsed ? 0.3 : 0.15} />
				{/each}
				{#each Object.entries(data.typed_links) as [type, notes]}
					{@const info = SECTOR_MAP[type]}
					{#if info}
						{#each notes as note, i}
							{@const spread = notes.length > 1 ? (i - (notes.length - 1) / 2) * 8 : 0}
							{@const ring = note.depth <= 1 ? 0 : note.depth <= 2 ? 1 : 2}
							{@const pos = polarToXY(140, 140, info.angle + spread, [56, 89.6, 123.2][ring])}
							<circle cx={pos.x} cy={pos.y} r="4" fill={info.color} opacity="0.7" cursor="pointer"
								onclick={() => onNoteClick?.(note.path, note.name)}>
								<title>{note.name} ({type})</title>
							</circle>
						{/each}
					{/if}
				{/each}
				<circle cx={140} cy={140} r="22" fill="#1a1a3a" stroke="#a78bfa" stroke-width="1.5" />
				<text x={140} y={137} text-anchor="middle" dominant-baseline="central" font-size="7" font-weight="700" fill="#fff">
					{truncName(data.note_name, 12)}
				</text>
				<text x={140} y={148} text-anchor="middle" font-size="6" fill="#a78bfa">L{data.stratum}</text>
			</svg>
			<div class="i360-stats-compact">
				<span>{'\u2B06'}{data.total_outbound} {'\u2B07'}{data.total_inbound}</span>
				{#if data.missing_link_types.length > 0}<span class="i360-warn">{'\u26A0'} {data.missing_link_types.length} {$t('inspector360.gaps') || 'gaps'}</span>{/if}
			</div>
		{/if}
	</div>
{:else}
	<!-- ===== FULL-WINDOW MODE ===== -->
	<div class="i360-full">
		{#if !data}
			<div class="i360-empty-full">
				<div class="i360-empty-icon-lg">{'\u{1F52E}'}</div>
				<div class="i360-empty-text-lg">{$t('inspector360.noData') || 'Open a note to see its 360\u00B0 view'}</div>
			</div>
		{:else}
			<!-- Header bar -->
			<div class="i360-header">
				<div class="i360-header-left">
					{#if previousNoteName && onBack}
						<button class="i360-back-full" onclick={onBack} title={`Return to ${previousNoteName}`}>
							<span class="i360-back-arrow">{'\u2190'}</span>
							<span class="i360-back-name" dir="auto">{truncName(previousNoteName, 24)}</span>
						</button>
					{/if}
					<span class="i360-header-icon">{'\u{1F9E0}'}</span>
					<span class="i360-header-label">{$t('inspector360.title') || '360.3D'}</span>
					<span class="i360-header-name" dir="auto">{data.note_name}</span>
				</div>
				<div class="i360-header-right">
					<!-- Visualization mode dropdown -->
					<select class="i360-mode-select" bind:value={vizMode}>
						<option value="atmospheric">{$t('inspector360.mode_atmospheric') || 'Atmospheric Rings'}</option>
						<option value="neural">{$t('inspector360.mode_neural') || 'Neural Web'}</option>
						<option value="cosmic">{$t('inspector360.mode_cosmic') || 'Cosmic Sphere'}</option>
					</select>
					{#if onClose}
						<button class="i360-close" onclick={onClose} title="Close">{'\u00D7'}</button>
					{/if}
				</div>
			</div>

			<!-- Main visualization area -->
			<div class="i360-canvas">
				{#if vizMode === 'atmospheric'}
					<!-- ===== MODE 1: ATMOSPHERIC RINGS ===== -->
					<svg class="i360-viz" viewBox="0 0 1200 800" preserveAspectRatio="xMidYMid meet">
						<defs>
							<filter id="glow-b"><feGaussianBlur stdDeviation="4" result="blur"/><feMerge><feMergeNode in="blur"/><feMergeNode in="SourceGraphic"/></feMerge></filter>
							<filter id="glow-r"><feGaussianBlur stdDeviation="4" result="blur"/><feMerge><feMergeNode in="blur"/><feMergeNode in="SourceGraphic"/></feMerge></filter>
							<filter id="glow-g"><feGaussianBlur stdDeviation="4" result="blur"/><feMerge><feMergeNode in="blur"/><feMergeNode in="SourceGraphic"/></feMerge></filter>
							<filter id="glow-p"><feGaussianBlur stdDeviation="3" result="blur"/><feMerge><feMergeNode in="blur"/><feMergeNode in="SourceGraphic"/></feMerge></filter>
							<filter id="glow-c"><feGaussianBlur stdDeviation="8" result="blur"/><feMerge><feMergeNode in="blur"/><feMergeNode in="SourceGraphic"/></feMerge></filter>
							<radialGradient id="atmo-center">
								<stop offset="0%" stop-color="#a78bfa" stop-opacity="0.3"/>
								<stop offset="50%" stop-color="#7c3aed" stop-opacity="0.1"/>
								<stop offset="100%" stop-color="#7c3aed" stop-opacity="0"/>
							</radialGradient>
						</defs>

						<!-- Center ambient glow -->
						<circle cx="600" cy="400" r="140" fill="url(#atmo-center)" filter="url(#glow-c)"/>

						<!-- Atmospheric concentric rings with 3D perspective tilt -->
						<ellipse cx="600" cy="400" rx="110" ry="100" fill="none" stroke="rgba(167,139,250,0.15)" stroke-width="1">
							<animateTransform attributeName="transform" type="rotate" from="0 600 400" to="360 600 400" dur="120s" repeatCount="indefinite"/>
						</ellipse>
						<ellipse cx="600" cy="400" rx="200" ry="170" fill="none" stroke="rgba(167,139,250,0.08)" stroke-width="0.8">
							<animateTransform attributeName="transform" type="rotate" from="360 600 400" to="0 600 400" dur="90s" repeatCount="indefinite"/>
						</ellipse>
						<ellipse cx="600" cy="400" rx="300" ry="250" fill="none" stroke="rgba(167,139,250,0.04)" stroke-width="0.5" stroke-dasharray="3,5">
							<animateTransform attributeName="transform" type="rotate" from="0 600 400" to="360 600 400" dur="150s" repeatCount="indefinite"/>
						</ellipse>

						<!-- Group labels — ring labels in rings mode; sector rim labels in sector mode. -->
						{#if layoutMode === 'rings'}
							{#each ringsLayout as ring}
								{@const labelText = ring.type === 'untyped'
									? 'Untyped'
									: ($t(`inspector360.${ring.labelKey}`) || ring.type)}
								<text x="600" y={400 - ring.radius - 10} text-anchor="middle" font-size="13" font-weight="600"
									fill={ring.color} opacity="0.78" letter-spacing="1">
									{labelText.toUpperCase()} ({ring.notes.length})
								</text>
							{/each}
						{:else}
							{#each ringsLayout as ring}
								{#if ring.type !== 'untyped'}
									{@const sectorInfo = SECTOR_MAP[ring.type]}
									{#if sectorInfo}
										{@const labelPos = polarToXY(600, 400, sectorInfo.angle, 415)}
										{@const labelText = $t(`inspector360.${ring.labelKey}`) || ring.type}
										<text x={labelPos.x} y={labelPos.y} text-anchor="middle" dominant-baseline="central"
											font-size="13" font-weight="600" letter-spacing="1"
											fill={ring.color} opacity="0.78">
											{labelText.toUpperCase()} ({ring.notes.length})
										</text>
									{/if}
								{/if}
							{/each}
						{/if}

						<!-- Connection lines (synaptic) -->
						{#each allNodes as node}
							<line x1="600" y1="400" x2={node.x} y2={node.y}
								stroke={node.color} stroke-width={node.depth <= 1 ? 1.5 : node.depth <= 2 ? 0.8 : 0.4}
								opacity={node.depth <= 1 ? 0.3 : node.depth <= 2 ? 0.15 : 0.08} />
						{/each}

						<!-- Gap zone indicators -->
						{#each data.missing_link_types as gapType}
							{@const gapInfo = SECTOR_MAP[gapType]}
							{#if gapInfo}
								{@const gapPos = polarToXY(600, 400, gapInfo.angle, 220)}
								<circle cx={gapPos.x} cy={gapPos.y} r="40" fill="none" stroke="rgba(255,255,255,0.04)" stroke-dasharray="4,6" stroke-width="0.5"/>
								<text x={gapPos.x} y={gapPos.y + 3} text-anchor="middle" font-size="8" fill="rgba(255,255,255,0.1)">
									{$t(`inspector360.${gapInfo.labelKey}`) || gapType}
								</text>
							{/if}
						{/each}

						<!-- Orbital nodes -->
						{#each allNodes as node}
							<!-- svelte-ignore a11y_no_static_element_interactions -->
							<g class="i360-node" style="cursor:pointer"
								onmouseenter={() => hoveredNode = node.path}
								onmouseleave={() => hoveredNode = null}
								onclick={() => onNoteClick?.(node.path, node.name)}>
								<!-- Invisible hit-area expands the click target so smaller
								     visible nodes stay easy to mouse-over. -->
								<circle cx={node.x} cy={node.y} r={node.r + 6} fill="transparent" pointer-events="all"/>
								{#if node.depth <= 2}
									<circle cx={node.x} cy={node.y} r={node.r + 4} fill={node.color} opacity="0.15" filter="url(#glow-b)" pointer-events="none"/>
								{/if}
								<circle cx={node.x} cy={node.y} r={node.r} fill={node.color}
									opacity={node.depth <= 1 ? 0.8 : node.depth <= 2 ? 0.6 : 0.3}
									pointer-events="none">
									{#if node.depth <= 1}
										<animate attributeName="r" values="{node.r};{node.r + 1};{node.r}" dur="{3 + node.r * 0.2}s" repeatCount="indefinite"/>
									{/if}
								</circle>
								{#if hoveredNode === node.path}
									<text x={node.x} y={node.y - node.r - 8} text-anchor="middle" font-size="13" font-weight="600"
										fill="rgba(255,255,255,0.95)" pointer-events="none"
										style="paint-order: stroke; stroke: rgba(0,0,0,0.85); stroke-width: 3px; stroke-linejoin: round;">
										{truncName(node.name, 32)}
									</text>
								{/if}
							</g>
						{/each}

						<!-- Center orb -->
						<circle cx="600" cy="400" r="50" fill="#1a1a3a" stroke="#a78bfa" stroke-width="2" filter="url(#glow-c)"/>
						<circle cx="600" cy="400" r="40" fill="#12122a"/>
						<text x="600" y="393" text-anchor="middle" font-size="12" font-weight="700" fill="#fff">{truncName(data.note_name, 14)}</text>
						<text x="600" y="410" text-anchor="middle" font-size="9" fill="#a78bfa">L{data.stratum} {'\u00B7'} {data.maturity}</text>
					</svg>

				{:else if vizMode === 'neural'}
					<!-- ===== MODE 2: NEURAL WEB ===== -->
					<svg class="i360-viz" viewBox="0 0 1200 800" preserveAspectRatio="xMidYMid meet">
						<defs>
							<filter id="n-glow"><feGaussianBlur stdDeviation="4" result="blur"/><feMerge><feMergeNode in="blur"/><feMergeNode in="SourceGraphic"/></feMerge></filter>
							<filter id="n-center"><feGaussianBlur stdDeviation="8" result="blur"/><feMerge><feMergeNode in="blur"/><feMergeNode in="SourceGraphic"/></feMerge></filter>
							<radialGradient id="neural-grad">
								<stop offset="0%" stop-color="#a78bfa" stop-opacity="0.4"/>
								<stop offset="50%" stop-color="#7c3aed" stop-opacity="0.15"/>
								<stop offset="100%" stop-color="#7c3aed" stop-opacity="0"/>
							</radialGradient>
						</defs>

						<!-- Center glow -->
						<circle cx="600" cy="400" r="120" fill="url(#neural-grad)" filter="url(#n-center)"/>

						<!-- Neural connections (synaptic lines — organic) -->
						{#each allNodes as node}
							<line x1="600" y1="400" x2={node.x} y2={node.y}
								stroke={node.color}
								stroke-width={node.depth <= 1 ? 1.5 : node.depth <= 2 ? 1 : 0.5}
								opacity={node.depth <= 1 ? 0.3 : node.depth <= 2 ? 0.2 : 0.1} />
						{/each}

						<!-- Ring outlines + group labels — only meaningful in rings mode. -->
						{#if layoutMode === 'rings'}
							{#each ringsLayout as ring}
								<circle cx="600" cy="400" r={ring.radius} fill="none"
									stroke={ring.color} stroke-width="0.6" opacity="0.18" stroke-dasharray="2,4"/>
							{/each}
							{#each ringsLayout as ring}
								{@const labelText = ring.type === 'untyped'
									? 'Untyped'
									: ($t(`inspector360.${ring.labelKey}`) || ring.type)}
								<text x="600" y={400 - ring.radius - 10} text-anchor="middle" font-size="13" font-weight="600"
									fill={ring.color} opacity="0.78" letter-spacing="1">
									{labelText.toUpperCase()} ({ring.notes.length})
								</text>
							{/each}
						{:else}
							{#each ringsLayout as ring}
								{#if ring.type !== 'untyped'}
									{@const sectorInfo = SECTOR_MAP[ring.type]}
									{#if sectorInfo}
										{@const labelPos = polarToXY(600, 400, sectorInfo.angle, 415)}
										{@const labelText = $t(`inspector360.${ring.labelKey}`) || ring.type}
										<text x={labelPos.x} y={labelPos.y} text-anchor="middle" dominant-baseline="central"
											font-size="13" font-weight="600" letter-spacing="1"
											fill={ring.color} opacity="0.78">
											{labelText.toUpperCase()} ({ring.notes.length})
										</text>
									{/if}
								{/if}
							{/each}
						{/if}

						<!-- (Former second-order branching block removed in §102 — it
						     used the old sector-based positioning; under ring-per-group
						     the lines would be disconnected from actual node positions.) -->

						<!-- Gap zones (dashed circles in empty directions) -->
						{#each data.missing_link_types as gapType}
							{@const gapInfo = SECTOR_MAP[gapType]}
							{#if gapInfo}
								{@const gapPos = polarToXY(600, 400, gapInfo.angle, 220)}
								<circle cx={gapPos.x} cy={gapPos.y} r="40" fill="none" stroke="rgba(255,255,255,0.04)" stroke-dasharray="4,6" stroke-width="0.5"/>
								<text x={gapPos.x} y={gapPos.y + 3} text-anchor="middle" font-size="7" fill="rgba(255,255,255,0.08)">
									{$t(`inspector360.${gapInfo.labelKey}`) || gapType}
								</text>
							{/if}
						{/each}


						<!-- Neural nodes (bright, pulsing) -->
						{#each allNodes as node}
							<!-- svelte-ignore a11y_no_static_element_interactions -->
							<g class="i360-node" style="cursor:pointer"
								onmouseenter={() => hoveredNode = node.path}
								onmouseleave={() => hoveredNode = null}
								onclick={() => onNoteClick?.(node.path, node.name)}>
								<circle cx={node.x} cy={node.y} r={node.r + 6} fill="transparent" pointer-events="all"/>
								<circle cx={node.x} cy={node.y} r={node.r} fill={node.color}
									opacity={node.depth <= 1 ? 0.8 : node.depth <= 2 ? 0.7 : 0.3}
									filter={node.depth <= 2 ? "url(#n-glow)" : undefined}
									pointer-events="none">
									{#if node.depth <= 1}
										<animate attributeName="r" values="{node.r};{node.r + 0.6};{node.r}" dur="{2.5 + node.r * 0.3}s" repeatCount="indefinite"/>
									{/if}
								</circle>
								{#if hoveredNode === node.path}
									<text x={node.x} y={node.y - node.r - 8} text-anchor="middle" font-size="13" font-weight="600"
										fill="rgba(255,255,255,0.95)" pointer-events="none"
										style="paint-order: stroke; stroke: rgba(0,0,0,0.85); stroke-width: 3px; stroke-linejoin: round;">
										{truncName(node.name, 32)}
									</text>
								{/if}
							</g>
						{/each}

						<!-- Center node -->
						<circle cx="600" cy="400" r="28" fill="#1a1a3a" stroke="#a78bfa" stroke-width="2" filter="url(#n-center)"/>
						<circle cx="600" cy="400" r="22" fill="#12122a"/>
						<text x="600" y="395" text-anchor="middle" font-size="10" font-weight="700" fill="#fff">{truncName(data.note_name, 12)}</text>
						<text x="600" y="410" text-anchor="middle" font-size="7" fill="#a78bfa">L{data.stratum} {'\u00B7'} {data.maturity}</text>
					</svg>

				{:else if vizMode === 'cosmic'}
					<!-- ===== MODE 3: COSMIC SPHERE ===== -->
					<svg class="i360-viz" viewBox="0 0 1200 800" preserveAspectRatio="xMidYMid meet">
						<defs>
							<filter id="c-glow"><feGaussianBlur stdDeviation="4" result="blur"/><feMerge><feMergeNode in="blur"/><feMergeNode in="SourceGraphic"/></feMerge></filter>
							<filter id="c-center"><feGaussianBlur stdDeviation="6" result="blur"/><feMerge><feMergeNode in="blur"/><feMergeNode in="SourceGraphic"/></feMerge></filter>
							<radialGradient id="cosmic-grad">
								<stop offset="0%" stop-color="rgba(167,139,250,0.4)"/>
								<stop offset="40%" stop-color="rgba(167,139,250,0.1)"/>
								<stop offset="100%" stop-color="rgba(167,139,250,0)"/>
							</radialGradient>
						</defs>

						<!-- Stars background particles -->
						{#each Array(30) as _, i}
							{@const sx = 80 + (i * 37 + i * i * 13) % 1040}
							{@const sy = 40 + (i * 53 + i * i * 7) % 720}
							<circle cx={sx} cy={sy} r={0.5 + (i % 3) * 0.3} fill="rgba(255,255,255,{0.1 + (i % 5) * 0.06})" />
						{/each}

						<!-- Orbital rings — per-group in rings mode, three fixed in sector mode. -->
						{#if layoutMode === 'rings'}
							{#each ringsLayout as ring}
								<circle cx="600" cy="400" r={ring.radius} fill="none"
									stroke={ring.color} stroke-width="0.9" opacity="0.32"/>
							{/each}
							{#each ringsLayout as ring}
								{@const labelText = ring.type === 'untyped'
									? 'Untyped'
									: ($t(`inspector360.${ring.labelKey}`) || ring.type)}
								<text x="600" y={400 - ring.radius - 10} text-anchor="middle" font-size="13" font-weight="600"
									fill={ring.color} opacity="0.8" letter-spacing="1">
									{labelText.toUpperCase()} ({ring.notes.length})
								</text>
							{/each}
						{:else}
							<circle cx="600" cy="400" r="160" fill="none" stroke="rgba(255,255,255,0.08)" stroke-width="1"/>
							<circle cx="600" cy="400" r="270" fill="none" stroke="rgba(255,255,255,0.05)" stroke-width="0.8"/>
							<circle cx="600" cy="400" r="380" fill="none" stroke="rgba(255,255,255,0.03)" stroke-width="0.5"/>
						{/if}

						<!-- Sector lines — solid for active, dashed for gaps -->
						{#each Object.entries(SECTOR_MAP) as [type, info]}
							{@const isUsed = data.used_link_types.includes(type)}
							{@const endPos = polarToXY(600, 400, info.angle, 400)}
							<line x1="600" y1="400" x2={endPos.x} y2={endPos.y}
								stroke={isUsed ? info.color : 'rgba(255,255,255,0.03)'}
								stroke-width={isUsed ? 1 : 0.5}
								stroke-dasharray={isUsed ? 'none' : '4,6'}
								opacity={isUsed ? 0.35 : 0.15} />
							<!-- Sector label at edge -->
							{@const labelPos = polarToXY(600, 400, info.angle, 415)}
							<text x={labelPos.x} y={labelPos.y} text-anchor="middle" dominant-baseline="central"
								font-size="10" font-weight="600" letter-spacing="1"
								fill={isUsed ? info.color : 'rgba(255,255,255,0.1)'}
								opacity={isUsed ? 0.4 : 0.15}>
								{($t(`inspector360.${info.labelKey}`) || type).toUpperCase()}
							</text>
						{/each}

						<!-- Gap warning labels -->
						{#each data.missing_link_types as gapType}
							{@const gapInfo = SECTOR_MAP[gapType]}
							{#if gapInfo}
								{@const gapPos = polarToXY(600, 400, gapInfo.angle, 415)}
								<text x={gapPos.x + 4} y={gapPos.y + 12} text-anchor="middle" font-size="7" fill="rgba(255,255,255,0.1)">
									{'\u26A0'}
								</text>
							{/if}
						{/each}

						<!-- Orb nodes -->
						{#each allNodes as node}
							<!-- svelte-ignore a11y_no_static_element_interactions -->
							<g class="i360-node" style="cursor:pointer"
								onmouseenter={() => hoveredNode = node.path}
								onmouseleave={() => hoveredNode = null}
								onclick={() => onNoteClick?.(node.path, node.name)}>
								<circle cx={node.x} cy={node.y} r={node.r + 6} fill="transparent" pointer-events="all"/>
								<circle cx={node.x} cy={node.y} r={node.r}
									fill={node.color} opacity={node.depth <= 1 ? 0.85 : node.depth <= 2 ? 0.6 : 0.3}
									filter={node.depth <= 1 ? "url(#c-glow)" : undefined}
									pointer-events="none" />
								{#if hoveredNode === node.path}
									<text x={node.x} y={node.y - node.r - 8} text-anchor="middle"
										font-size="13" font-weight="600" fill="rgba(255,255,255,0.95)" pointer-events="none"
										style="paint-order: stroke; stroke: rgba(0,0,0,0.85); stroke-width: 3px; stroke-linejoin: round;">
										{truncName(node.name, 32)}
									</text>
								{/if}
							</g>
						{/each}

						<!-- Center core with pulse -->
						<circle cx="600" cy="400" r="60" fill="url(#cosmic-grad)" filter="url(#c-center)">
							<animate attributeName="r" values="60;63;60" dur="3s" repeatCount="indefinite"/>
						</circle>
						<circle cx="600" cy="400" r="36" fill="radial-gradient(circle, #1a1a3a, #0d0d2b)"/>
						<circle cx="600" cy="400" r="36" fill="#0d0d2b" stroke="rgba(167,139,250,0.6)" stroke-width="2"/>
						<text x="600" y="395" text-anchor="middle" font-size="11" font-weight="700" fill="#e0e0e0">{truncName(data.note_name, 14)}</text>
						<text x="600" y="411" text-anchor="middle" font-size="8" fill="#a78bfa">L{data.stratum} {'\u00B7'} {data.maturity}</text>
					</svg>
				{/if}
			</div>

			<!-- Side panels (floating glass cards) -->
			<div class="i360-panel i360-panel-tr">
				<div class="i360-panel-title">{$t('inspector360.dimensions') || 'Dimensions'}</div>
				<div class="i360-panel-item">
					<span class="i360-dot" style="background:{MATURITY_COLORS[data.maturity] ?? '#999'}"></span>
					{data.maturity}
				</div>
				<div class="i360-panel-item">
					<span class="i360-dot" style="background:{ORIGIN_COLORS[data.origin_type] ?? '#999'}"></span>
					{data.origin_type} {'\u00B7'} {$t('inspector360.depth') || 'Depth'} {data.trust_depth}
				</div>
				<div class="i360-panel-item">
					<span class="i360-dot" style="background:#a78bfa"></span>
					{data.stage || 'none'}
				</div>
				{#if data.is_due}
					<div class="i360-panel-item i360-panel-warn">
						{'\u{1F4CB}'} {$t('inspector360.dueForReview') || 'Due for review'}
					</div>
				{/if}
			</div>

			<div class="i360-panel i360-panel-bl">
				<div class="i360-panel-title">{$t('inspector360.context') || 'Context'}</div>
				{#if data.trails.length > 0}
					<div class="i360-panel-item">{'\u{1F6E4}\uFE0F'} {data.trails.join(', ')}</div>
				{/if}
				{#if data.lens_groups.length > 0}
					<div class="i360-panel-item">{'\u{1F3F7}\uFE0F'} {data.lens_groups.join(', ')}</div>
				{/if}
				{#if data.missing_link_types.length > 0}
					<div class="i360-panel-item i360-panel-warn">
						{'\u26A0'} {data.missing_link_types.length} {$t('inspector360.gaps') || 'gaps'} ({data.missing_link_types.join(', ')})
					</div>
				{/if}
				{#if data.contradictions.length > 0}
					<div class="i360-panel-item i360-panel-warn">
						{'\u26A1'} {data.contradictions.length} {$t('inspector360.tensions') || 'tensions'}
					</div>
				{/if}
			</div>

			<!-- Bottom HUD -->
			<div class="i360-hud">
				<div class="i360-hud-left">
					<span class="i360-hud-item">{'\u2B06'} {data.total_outbound} {$t('inspector360.outbound') || 'outbound'}</span>
					<span class="i360-hud-item">{'\u2B07'} {data.total_inbound} {$t('inspector360.inbound') || 'inbound'}</span>
					<span class="i360-hud-item">{'\u{1F4DD}'} {data.word_count.toLocaleString()} {$t('inspector360.words') || 'words'}</span>
				</div>
				<div class="i360-hud-right">
					{#if data.is_orphan}<span class="i360-hud-item i360-hud-warn">{'\u26A0'} {$t('inspector360.orphan') || 'Orphan'}</span>{/if}
					{#if data.single_point_of_failure}<span class="i360-hud-item i360-hud-warn">{'\u26A0'} {$t('inspector360.fragile') || 'Fragile'}</span>{/if}
					{#if data.missing_link_types.length > 0}<span class="i360-hud-item i360-hud-warn">{'\u26A0'} {data.missing_link_types.length} {$t('inspector360.blindSpots') || 'blind spots'}</span>{/if}
				</div>
			</div>
		{/if}
	</div>
{/if}

<style>
	/* ===== COMPACT SIDEBAR ===== */
	.i360.compact { display: flex; flex-direction: column; align-items: center; padding: 4px; }
	.i360-svg { width: 100%; max-width: 280px; height: auto; }
	.i360-back-bar {
		align-self: stretch;
		display: flex; align-items: center; gap: 6px;
		padding: 4px 8px; margin-bottom: 4px;
		background: rgba(127,127,127,0.06);
		border: 1px solid rgba(127,127,127,0.18);
		border-radius: 6px;
		color: var(--text-muted, #888);
		font-size: 0.78rem;
		cursor: pointer;
		text-align: start;
	}
	.i360-back-bar:hover {
		background: rgba(127,127,127,0.14);
		color: var(--text, inherit);
	}
	.i360-back-arrow { font-size: 0.9rem; line-height: 1; flex-shrink: 0; }
	.i360-back-name { flex: 1; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
	.i360-empty { text-align: center; padding: 24px; }
	.i360-empty-icon { font-size: 2rem; margin-bottom: 8px; }
	.i360-empty-text { font-size: 0.82rem; color: var(--text-muted, #999); }
	.i360-stats-compact { display: flex; gap: 10px; padding: 4px; font-size: 0.7rem; color: rgba(255,255,255,0.4); }
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

	/* Header — full-window. Sized 2x for the deliberate-study surface. */
	.i360-header {
		display: flex; align-items: center; justify-content: space-between;
		padding: 18px 32px;
		background: linear-gradient(180deg, rgba(6,6,18,0.95), transparent);
		z-index: 20; position: relative;
		gap: 16px;
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
	.i360-mode-select {
		background: rgba(255,255,255,0.06);
		border: 1px solid rgba(255,255,255,0.12);
		border-radius: 10px; padding: 8px 16px;
		color: #ccc; font-size: 16px;
		cursor: pointer; outline: none;
	}
	.i360-mode-select:hover { background: rgba(255,255,255,0.1); }
	.i360-mode-select option { background: #1a1a3a; color: #ccc; }
	.i360-close {
		width: 48px; height: 48px; border-radius: 50%;
		border: 1px solid rgba(255,255,255,0.1);
		background: rgba(255,255,255,0.03);
		color: #888; font-size: 28px; cursor: pointer;
		display: flex; align-items: center; justify-content: center;
		flex-shrink: 0;
	}
	.i360-close:hover { background: rgba(255,255,255,0.08); color: #fff; }
	/* Full-window back button — "Return to {previous note}". */
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
	.i360-back-full .i360-back-name {
		overflow: hidden; text-overflow: ellipsis; white-space: nowrap;
	}

	/* Canvas area — fills the available viewport (2.1.5: take advantage of available space). */
	.i360-canvas {
		flex: 1; position: relative;
		background: radial-gradient(ellipse at 50% 45%, #0e0e28 0%, #060612 70%);
		display: flex; align-items: center; justify-content: center;
	}
	.i360-viz {
		width: 100%; height: 100%;
	}

	/* Node hover effect */
	.i360-node:hover circle { opacity: 1 !important; }

	/* Floating panels — sized 2x. */
	.i360-panel {
		position: absolute; z-index: 15;
		padding: 18px 22px; border-radius: 14px;
		background: rgba(12,12,30,0.8);
		border: 1px solid rgba(255,255,255,0.06);
		backdrop-filter: blur(10px);
		font-size: 18px; line-height: 1.7;
		max-width: 380px;
	}
	.i360-panel-tr { top: 100px; right: 32px; }
	.i360-panel-bl { bottom: 96px; left: 32px; }
	.i360-panel-title {
		font-weight: 700; font-size: 18px; color: #a78bfa;
		margin-bottom: 10px; letter-spacing: 0.5px;
	}
	.i360-panel-item {
		display: flex; align-items: center; gap: 10px;
		color: rgba(255,255,255,0.6);
	}
	.i360-panel-warn { color: #ef4444; }
	.i360-dot {
		width: 14px; height: 14px; border-radius: 50%;
		display: inline-block; flex-shrink: 0;
	}

	/* Bottom HUD — sized 2x. */
	.i360-hud {
		position: absolute; bottom: 0; left: 0; right: 0;
		padding: 18px 36px;
		display: flex; justify-content: space-between;
		z-index: 20;
		background: linear-gradient(0deg, rgba(6,6,18,0.95), transparent);
	}
	.i360-hud-left, .i360-hud-right { display: flex; gap: 28px; }
	.i360-hud-item {
		font-size: 18px; color: rgba(255,255,255,0.55);
		display: flex; align-items: center; gap: 8px;
	}
	.i360-hud-warn { color: #ef4444; }
</style>
