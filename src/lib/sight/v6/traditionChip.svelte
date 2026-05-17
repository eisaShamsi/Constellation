<!--
  MIG-026 Phase β — A3+A6 chip UI rebuild.

  Inherits from MIG-025 §C.1 chip (collapsed/expanded toggle) +
  §C.1-fix-1 Esc bug fix + MIG-026 Phase 0 rename.

  NEW in Phase β: family-categorized dropdown (A3) + 4 favorites
  pinned inline (A6). Replaces the previous inline-row-of-N pattern
  which doesn't scale to 24 traditions.

  Layout structure:
  - Inline row (always visible): 4 favorite chips + "All ▾" dropdown
    trigger. Active tradition has blue stroke + dot. Hover shows
    English secondary label tooltip.
  - Dropdown panel (toggled by All trigger): family-collapsible
    accordion. Each family section shows ≥1 tradition with name +
    scope-strip + pin/unpin star toggle. Click any tradition to
    switch active + close dropdown. Pin toggle adds/removes from
    favoriteTraditions (max 4 inline; rest live in dropdown).
  - Esc closes dropdown (still uses window+capture pattern from
    §C.1-fix-1 to beat Layout's global Esc-closes-Sight handler).
  - Click outside dropdown closes it (mousedown listener on document).

  Favorites persistence: appSettings.sight.favoriteTraditions
  (string[]). Default ['aristotelian', 'pramana', 'masadir', 'polanyi'].
  Order = display order. First 4 shown inline; rest visible in
  dropdown with "★" (pinned) state.

  Scope strips: hardcoded in TRADITIONS_META for Phase β. Phase ι.2
  (J3 + J5 disclosure layer) will replace with manifest reads —
  manifests at docs/traditions/<id>.md provide canonical scope text.

  Brand-name tradition labels stay English per §A.15 brand convention.
  Cultural diacritics (pramāṇa, masādir, Mohist sān biǎo, etc.)
  render via Unicode in any modern font stack.

  Concept Paper: docs/Constellation-Sight-Concept-Paper-v4.0.md §2.5, §4.1, §4.2
  Plan:          lab/reports/MIG-026-SIGHT-REGISTER-EXPANSION-PLAN.md §4 (Phase β)
  Architect:     §3.A choice A3+A6
-->
<script lang="ts">
	import { appSettings, saveSettings } from '$lib/libraries/store';
	import type { TraditionId } from './types';
	import { FAMILIES, type FamilyId } from './traditions';

	// Per-tradition metadata (name, tooltip, scope, preview flag).
	// Hardcoded here for Phase β; Phase ι.2 replaces scope reads with
	// manifest fetches from docs/traditions/<id>.md.
	type TraditionMeta = {
		name: string;          // chip label (kept English per §A.15 brand convention)
		tooltip: string;       // hover tooltip per Concept Paper §2.5
		scope: string;         // scope strip per Plan §4 + J5
		preview: boolean;      // v1-preview badge per §4.2
	};

	const TRADITIONS_META: Record<TraditionId, TraditionMeta> = {
		aristotelian: {
			name: 'Aristotelian',
			tooltip: 'Aristotelian — Western-classical, knowledge as maturity gradient',
			scope: 'For any content; the default Sight grammar (stratum × time).',
			preview: false,
		},
		pramana: {
			name: 'pramāṇa',
			tooltip: 'pramāṇa — Nyāya fourfold valid means of knowing',
			scope: 'For epistemological analysis of cognitive acts: perception, inference, analogy, testimony.',
			preview: false,
		},
		masadir: {
			name: 'masādir',
			tooltip: 'masādir — Sunni uṣūl al-fiqh, sources as kinds of proof',
			scope: 'For Sunni Islamic legal-scholarly content. Not designed for secular or non-Islamic content.',
			preview: false,
		},
		polanyi: {
			name: 'Polanyi',
			tooltip: 'Polanyi — modern pluralism, tacit as the proximal pole',
			scope: 'For knowledge with a tacit-vs-explicit dimension; what you know vs. what you can articulate.',
			preview: false,
		},
		'mohist-san-biao': {
			name: 'Mohist sān biǎo',
			tooltip: 'Mohist sān biǎo — three standards as tests of doctrines (v1 preview)',
			scope: 'For doctrines tested by historical precedent / observational evidence / social benefit.',
			preview: true,
		},
		peirce: {
			name: 'Peirce',
			tooltip: 'Peirce — American pragmatist, 3 phaneroscopic categories (Firstness / Secondness / Thirdness)',
			scope: 'For phenomenological classification: quality/feeling (Firstness), reaction/relation (Secondness), mediation/law (Thirdness).',
			preview: false,
		},
		habermas: {
			name: 'Habermas',
			tooltip: 'Habermas — Frankfurt School critical theory, 3 knowledge-constitutive interests',
			scope: 'For knowledge classified by orientation: prediction/control (technical), mutual understanding (practical), reflection/liberation (emancipatory).',
			preview: false,
		},
		dewey: {
			name: 'Dewey',
			tooltip: 'Dewey — pragmatist 5-stage pattern of inquiry (cyclic flow)',
			scope: 'For knowledge as the resolution of an indeterminate situation through inquiry: indeterminate → problem → hypothesis → reasoning → testing (and back).',
			preview: false,
		},
		husserl: {
			name: 'Husserl',
			tooltip: 'Husserl — phenomenological regional ontologies (4 concentric zones)',
			scope: 'For knowledge classified by ontological region: formal ontology (center), material nature, animal nature, spirit / Geist (outer).',
			preview: false,
		},
		longino: {
			name: 'Longino',
			tooltip: 'Longino — Critical Contextual Empiricism, 4 norms of objective inquiry',
			scope: 'For social conditions of objectivity: venues (public forums), uptake (response to criticism), public standards (shared criteria), tempered equality (credentialed disagreement).',
			preview: false,
		},
		'ibn-rushd-burhan': {
			name: 'Ibn Rushd burhān',
			tooltip: 'Ibn Rushd (Averroes) — 4 concentric demonstrative arts ranked by epistemic force',
			scope: 'For Islamic Aristotelian commentary tradition: burhān (apodictic demonstration, innermost), jadal (dialectic), khaṭāba (rhetoric), shiʿr (poetics, outermost).',
			preview: false,
		},
		'shatibi-maqasid': {
			name: 'Shāṭibī maqāṣid',
			tooltip: 'al-Shāṭibī — maqāṣid al-sharīʿa: 5 universal essentials × 3 tiers of necessity (15-cell grid)',
			scope: 'For Sunni Islamic legal-purpose analysis: each note at the intersection of an essential (dīn / nafs / ʿaql / nasl / māl) and a tier (ḍarūriyyāt / ḥājiyyāt / taḥsīniyyāt).',
			preview: false,
		},
		'ibn-khaldun-umran': {
			name: 'Ibn Khaldūn ʿumrān',
			tooltip: 'Ibn Khaldūn — ʿilm al-ʿumrān: bedouin↔urban cyclical civilizational dynamic',
			scope: 'For Islamic philosophical historiography: ḥaḍarī (sedentary/urban) above, badawī (nomadic/rural) below; cyclic arrows convey the bidirectional generational cycle of ʿaṣabiyya rise + decay.',
			preview: false,
		},
		pardes: {
			name: 'PaRDeS',
			tooltip: 'PaRDeS — 4 levels of Torah interpretation, literal to mystical (Hebrew acronym)',
			scope: 'For Jewish hermeneutical analysis: peshat (literal, innermost) → remez (allusion) → derash (interpretive) → sod (mystical, outermost).',
			preview: false,
		},
		'maimonidean-prophecy': {
			name: 'Maimonidean prophecy',
			tooltip: 'Maimonides — 11 degrees of prophecy in Guide of the Perplexed II:45 (spiral ladder)',
			scope: 'For Jewish philosophical theology: 11 ascending degrees of prophetic experience, from ruaḥ ha-qodesh (holy spirit, innermost) to angel-vision-while-awake (Moses-level, outermost).',
			preview: false,
		},
	};

	let dropdownOpen = $state(false);
	let rootEl = $state<HTMLDivElement | null>(null);
	// Track which family sections are expanded in the dropdown.
	// Default: all families expanded so user sees the full menu on first open.
	let expandedFamilies = $state<Set<FamilyId>>(new Set(Object.keys(FAMILIES) as FamilyId[]));

	const activeId = $derived<TraditionId>(
		($appSettings.sight?.activeTradition as TraditionId | undefined) ?? 'aristotelian'
	);

	// Favorite ids from settings (default: 4 production traditions).
	const favoriteIds = $derived<TraditionId[]>(
		($appSettings.sight?.favoriteTraditions as TraditionId[] | undefined) ??
			['aristotelian', 'pramana', 'masadir', 'polanyi']
	);

	// Inline chips = first 4 favorites that have metadata. (TraditionId
	// guarantees the meta exists for shipping traditions; we filter
	// defensively in case settings.json contains a value that's been
	// excluded since last save — same pattern as the dignaga/ishraqi
	// migration safeguards in store.ts.)
	const inlineChips = $derived<TraditionId[]>(
		favoriteIds.filter((id) => id in TRADITIONS_META).slice(0, 4)
	);

	// Families that have ≥1 tradition listed in FAMILIES. Mohist's
	// 'chinese-pragmatist' family appears even before Phase γ ships
	// the Mohist module (clicking the chip writes activeTradition;
	// dome fall-back to Aristotelian positions until Phase γ lands
	// the module). Empty families (Phase ε/ζ/η/θ pending) are hidden.
	type FamilySection = { id: FamilyId; label: string; traditions: TraditionId[] };
	const familiesWithTraditions = $derived<FamilySection[]>(
		(Object.entries(FAMILIES) as [FamilyId, { label: string; traditions: TraditionId[] }][])
			.filter(([, fam]) => fam.traditions.length > 0)
			.map(([id, fam]) => ({ id, label: fam.label, traditions: fam.traditions }))
	);

	function activeMeta(id: TraditionId): TraditionMeta {
		return TRADITIONS_META[id] ?? TRADITIONS_META.aristotelian;
	}

	function isFavorite(id: TraditionId): boolean {
		return favoriteIds.includes(id);
	}

	function handleChipClick(id: TraditionId) {
		// Canonical write pattern (matches §B.10 extended toggle).
		appSettings.update((s) => ({
			...s,
			sight: { ...s.sight, activeTradition: id },
		}));
		saveSettings();
		// Close dropdown after switch (chip stays inline).
		dropdownOpen = false;
	}

	function handleAllClick() {
		dropdownOpen = !dropdownOpen;
	}

	function handleFamilyToggle(familyId: FamilyId) {
		const next = new Set(expandedFamilies);
		if (next.has(familyId)) {
			next.delete(familyId);
		} else {
			next.add(familyId);
		}
		expandedFamilies = next;
	}

	function handlePinToggle(id: TraditionId) {
		const current = [...favoriteIds];
		const idx = current.indexOf(id);
		if (idx >= 0) {
			// Unpin: remove from list.
			current.splice(idx, 1);
		} else {
			// Pin: append to end of list. Inline shows first 4; if
			// favorites > 4, the new one is in the dropdown until user
			// unpins one of the inline 4.
			current.push(id);
		}
		appSettings.update((s) => ({
			...s,
			sight: { ...s.sight, favoriteTraditions: current },
		}));
		saveSettings();
	}

	// Click-outside closes dropdown.
	function handleOutsideClick(ev: MouseEvent) {
		if (!dropdownOpen || !rootEl) return;
		if (!rootEl.contains(ev.target as Node)) {
			dropdownOpen = false;
		}
	}

	// Esc closes dropdown. Same window+capture pattern as §C.1-fix-1
	// to beat Layout's global Esc-closes-Sight handler.
	function handleKey(ev: KeyboardEvent) {
		if (ev.key === 'Escape' && dropdownOpen) {
			dropdownOpen = false;
			ev.stopPropagation();
			ev.preventDefault();
		}
	}

	$effect(() => {
		if (!dropdownOpen) return;
		document.addEventListener('mousedown', handleOutsideClick);
		window.addEventListener('keydown', handleKey, true);
		return () => {
			document.removeEventListener('mousedown', handleOutsideClick);
			window.removeEventListener('keydown', handleKey, true);
		};
	});
</script>

<div bind:this={rootEl} class="tradition-chip-root">
	<!-- Inline row: 4 favorite chips + All trigger -->
	<div class="tradition-chip-inline-row">
		{#each inlineChips as id (id)}
			{@const meta = activeMeta(id)}
			<button
				class="tradition-chip"
				class:is-active={id === activeId}
				class:is-preview={meta.preview}
				type="button"
				title={meta.tooltip}
				onclick={() => handleChipClick(id)}
			>
				<span class="chip-name">{meta.name}</span>
				{#if id === activeId}
					<span class="chip-dot" aria-hidden="true"></span>
				{/if}
				{#if meta.preview}
					<span class="chip-preview-badge" title="v1 preview — deeper internal structure is a v4.1 polish target">preview</span>
				{/if}
			</button>
		{/each}
		<button
			class="tradition-chip-all-trigger"
			class:is-open={dropdownOpen}
			type="button"
			onclick={handleAllClick}
			title="Show all scholarly traditions"
			aria-haspopup="true"
			aria-expanded={dropdownOpen}
		>
			All <span class="trigger-chevron" class:is-open={dropdownOpen}>▾</span>
		</button>
	</div>

	<!-- Dropdown panel (family-categorized accordion) -->
	{#if dropdownOpen}
		<div class="tradition-chip-dropdown" role="dialog" aria-label="Scholarly traditions">
			{#each familiesWithTraditions as fam (fam.id)}
				{@const isExpanded = expandedFamilies.has(fam.id)}
				<div class="tradition-chip-family-section">
					<button
						class="tradition-chip-family-header"
						class:is-expanded={isExpanded}
						type="button"
						onclick={() => handleFamilyToggle(fam.id)}
						aria-expanded={isExpanded}
					>
						<span class="family-chevron" class:is-expanded={isExpanded}>▶</span>
						<span class="family-label">{fam.label}</span>
						<span class="family-count">({fam.traditions.length})</span>
					</button>
					{#if isExpanded}
						<ul class="tradition-chip-family-list">
							{#each fam.traditions as id (id)}
								{@const meta = activeMeta(id)}
								{@const fav = isFavorite(id)}
								<li class="tradition-row" class:is-active={id === activeId}>
									<button
										class="tradition-row-name-btn"
										type="button"
										onclick={() => handleChipClick(id)}
										title={meta.tooltip}
									>
										<span class="chip-name">{meta.name}</span>
										{#if id === activeId}
											<span class="chip-dot" aria-hidden="true"></span>
										{/if}
										{#if meta.preview}
											<span class="chip-preview-badge">preview</span>
										{/if}
										<span class="tradition-row-scope">{meta.scope}</span>
									</button>
									<button
										class="tradition-row-pin-btn"
										class:is-pinned={fav}
										type="button"
										onclick={() => handlePinToggle(id)}
										title={fav ? 'Unpin from favorites' : 'Pin to favorites (inline row)'}
										aria-label={fav ? 'Unpin from favorites' : 'Pin to favorites'}
									>
										{fav ? '★' : '☆'}
									</button>
								</li>
							{/each}
						</ul>
					{/if}
				</div>
			{/each}
		</div>
	{/if}
</div>

<style>
	/* MIG-026 Phase β — A3+A6 chip UI styles. */

	.tradition-chip-root {
		display: inline-flex;
		align-items: center;
		position: relative;
	}

	.tradition-chip-inline-row {
		display: inline-flex;
		align-items: center;
		gap: 4px;
		padding: 2px;
		/* MIG-027 — theme-aware chip-row container */
		background: var(--background-secondary, rgba(58, 67, 90, 0.35));
		border: 1px solid var(--background-modifier-border, rgba(59, 89, 152, 0.55));
		border-radius: 5px;
	}

	/* Individual favorite chip (same compact style as MIG-025 §C.1).
	   MIG-027 §-fix-1: inactive-chip text uses --text-normal (not
	   --text-muted) so chips read at full contrast on light themes too.
	   The active vs inactive distinction is carried by the border + bg
	   tint + dot (not by text dimness), so full-contrast text is safe
	   in both themes. */
	.tradition-chip {
		display: inline-flex;
		align-items: center;
		gap: 5px;
		padding: 3px 9px;
		font-size: 11px;
		font-family: inherit;
		color: var(--text-normal, #c8cdd9);
		background: transparent;
		border: 1px solid transparent;
		border-radius: 3px;
		cursor: pointer;
		transition: background 0.12s ease, color 0.12s ease, border-color 0.12s ease;
		white-space: nowrap;
	}
	.tradition-chip:hover {
		color: var(--text-normal, #e8ebf2);
		background: var(--background-modifier-hover, rgba(74, 90, 130, 0.35));
	}
	.tradition-chip.is-active {
		color: var(--text-normal, #e8ebf2);
		border-color: var(--text-accent, #3b5998);
		background: hsla(var(--accent-h, 220), var(--accent-s, 50%), 50%, 0.18);
	}
	.tradition-chip.is-active:hover {
		background: hsla(var(--accent-h, 220), var(--accent-s, 50%), 50%, 0.28);
	}

	/* "All ▾" dropdown trigger.
	   MIG-027 §-fix-1: text bumped to --text-normal so the trigger is
	   readable on light themes (where --text-muted goes too faint). */
	.tradition-chip-all-trigger {
		display: inline-flex;
		align-items: center;
		gap: 3px;
		padding: 3px 9px;
		font-size: 11px;
		font-family: inherit;
		color: var(--text-normal, #a0a8ba);
		background: transparent;
		border: 1px dashed var(--background-modifier-border, rgba(160, 168, 186, 0.40));
		border-radius: 3px;
		cursor: pointer;
		transition: background 0.12s ease, color 0.12s ease, border-color 0.12s ease;
		white-space: nowrap;
	}
	.tradition-chip-all-trigger:hover {
		color: var(--text-normal, #e8ebf2);
		background: var(--background-modifier-hover, rgba(74, 90, 130, 0.35));
		border-color: var(--text-muted, rgba(160, 168, 186, 0.65));
	}
	.tradition-chip-all-trigger.is-open {
		color: var(--text-normal, #e8ebf2);
		background: hsla(var(--accent-h, 220), var(--accent-s, 50%), 50%, 0.22);
		border-color: var(--text-accent, #3b5998);
		border-style: solid;
	}
	.trigger-chevron {
		font-size: 9px;
		line-height: 1;
		transition: transform 0.18s ease;
	}
	.trigger-chevron.is-open {
		transform: rotate(180deg);
	}

	.chip-name {
		font-weight: 500;
		letter-spacing: 0.2px;
	}

	.chip-dot {
		display: inline-block;
		width: 6px;
		height: 6px;
		border-radius: 50%;
		/* MIG-027 — active-tradition indicator uses theme accent */
		background: var(--text-accent, #3b5998);
		box-shadow: 0 0 4px hsla(var(--accent-h, 220), var(--accent-s, 50%), 50%, 0.6);
	}

	.chip-preview-badge {
		display: inline-flex;
		align-items: center;
		padding: 0 4px;
		font-size: 8px;
		font-weight: 600;
		letter-spacing: 0.5px;
		color: #c9a155;
		background: rgba(201, 161, 85, 0.08);
		border: 1px solid rgba(201, 161, 85, 0.42);
		border-radius: 2px;
		text-transform: uppercase;
		font-variant: small-caps;
	}

	/* Dropdown panel. */
	.tradition-chip-dropdown {
		position: absolute;
		top: calc(100% + 6px);
		left: 0;
		min-width: 340px;
		max-width: 460px;
		max-height: 70vh;
		overflow-y: auto;
		padding: 6px 0;
		/* MIG-027 — theme-aware dropdown surface */
		background: var(--background-secondary, rgba(13, 19, 34, 0.98));
		border: 1px solid var(--background-modifier-border, rgba(59, 89, 152, 0.55));
		border-radius: 5px;
		box-shadow: var(--shadow-l, 0 6px 24px rgba(0, 0, 0, 0.55));
		z-index: 50;
	}

	.tradition-chip-family-section {
		display: flex;
		flex-direction: column;
	}

	.tradition-chip-family-header {
		display: inline-flex;
		align-items: center;
		gap: 8px;
		padding: 8px 14px 6px 14px;
		font-size: 10px;
		font-weight: 600;
		letter-spacing: 0.8px;
		text-transform: uppercase;
		font-variant: small-caps;
		color: var(--text-muted, #7b8499);
		background: transparent;
		border: none;
		cursor: pointer;
		text-align: left;
	}
	.tradition-chip-family-header:hover {
		color: var(--text-normal, #c8cdd9);
	}
	.family-chevron {
		display: inline-block;
		font-size: 9px;
		transition: transform 0.15s ease;
		color: var(--text-faint, #5a6275);
	}
	.family-chevron.is-expanded {
		transform: rotate(90deg);
		color: var(--text-muted, #7b8499);
	}
	.family-label {
		font-weight: 600;
	}
	.family-count {
		color: var(--text-faint, #5a6275);
		font-weight: 400;
		font-size: 9px;
	}

	.tradition-chip-family-list {
		list-style: none;
		margin: 0;
		padding: 0 0 4px 0;
	}

	.tradition-row {
		display: flex;
		align-items: stretch;
		gap: 4px;
		padding: 0 4px 0 22px;
	}
	.tradition-row.is-active {
		background: hsla(var(--accent-h, 220), var(--accent-s, 50%), 50%, 0.12);
	}
	.tradition-row:hover {
		background: var(--background-modifier-hover, rgba(74, 90, 130, 0.18));
	}
	.tradition-row.is-active:hover {
		background: hsla(var(--accent-h, 220), var(--accent-s, 50%), 50%, 0.20);
	}

	.tradition-row-name-btn {
		flex: 1 1 auto;
		display: flex;
		flex-direction: column;
		align-items: flex-start;
		gap: 2px;
		padding: 8px 10px;
		font-family: inherit;
		color: var(--text-normal, #e8ebf2);
		background: transparent;
		border: none;
		border-radius: 3px;
		cursor: pointer;
		text-align: left;
	}
	.tradition-row-name-btn > .chip-name {
		font-size: 12px;
		font-weight: 500;
	}
	.tradition-row-scope {
		font-size: 10px;
		font-style: italic;
		color: var(--text-muted, #7b8499);
		line-height: 1.35;
	}

	.tradition-row-pin-btn {
		flex: 0 0 auto;
		display: inline-flex;
		align-items: center;
		justify-content: center;
		width: 28px;
		font-size: 14px;
		/* Pin star color stays semantic-gold (matches EXTENDED badge
		   gold convention); not theme-derived. */
		color: var(--text-faint, #5a6275);
		background: transparent;
		border: none;
		border-radius: 3px;
		cursor: pointer;
		transition: color 0.12s ease;
	}
	.tradition-row-pin-btn:hover {
		color: #c9a155;
	}
	.tradition-row-pin-btn.is-pinned {
		color: #c9a155;
	}
	.tradition-row-pin-btn.is-pinned:hover {
		color: #d4b072;
	}
</style>
