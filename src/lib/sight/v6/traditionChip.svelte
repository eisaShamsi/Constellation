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
	import { t } from '$lib/i18n';
	import type { TraditionId } from './types';
	import { FAMILIES, type FamilyId } from './traditions';
	import type { UserTraditionModule } from './traditions/userDefinedLoader';

	// MIG-026 Phase ι.2 — disclosure-layer callbacks.
	//
	// `openManifest(id)` fires when the user clicks the ⓘ button in the
	// dropdown next to a tradition row. The parent (SightV6.svelte)
	// resolves the id to the bundled manifest markdown and renders it in
	// a modal overlay.
	//
	// `onDropdownClose()` fires whenever the dropdown closes (Esc,
	// click-outside, tradition switch). The parent uses this to cascade-
	// close the manifest modal so it never floats over a closed dropdown.
	//
	// MIG-026 Phase κ.1 — userTraditions prop carries any declarative
	// JSON traditions loaded from <Universe>/.constellation/traditions/
	// by SightV6's mount handler. The chip surfaces these in a
	// "User-defined" section at the bottom of the dropdown alongside
	// curated families. Empty array (default) hides the section so users
	// without any user traditions see no extra UI noise.
	let {
		openManifest = (_id: string) => {},
		onDropdownClose = () => {},
		userTraditions = [] as UserTraditionModule[],
	}: {
		openManifest?: (id: string) => void;
		onDropdownClose?: () => void;
		userTraditions?: UserTraditionModule[];
	} = $props();

	// Per-tradition metadata (name, tooltip, scope, preview flag).
	//
	// MIG-026 §λ-fix-2 (2026-05-18): the day-one Standing Order "when
	// user switches language, EVERYTHING translates" applies. Name,
	// tooltip, and scope now resolve via $t() at render time from
	// `sight.v6.tradition.list.<id>.{name|tooltip|scope}` keys. The
	// META map below carries the `preview` flag only; the literal
	// strings live in the i18n locale files where they get
	// proper-native-equivalent translations (per the same SO:
	// مصادر, not مَسَادِر).
	type TraditionMeta = {
		name: string;          // chip label, resolved via $t at render time
		tooltip: string;       // hover tooltip, resolved via $t
		scope: string;         // scope strip, resolved via $t
		preview: boolean;      // v1-preview badge per §4.2 (data-only flag)
	};

	const CURATED_TRADITION_IDS: TraditionId[] = [
		'aristotelian', 'pramana', 'masadir', 'polanyi', 'mohist-san-biao',
		'peirce', 'habermas', 'dewey', 'husserl', 'longino',
		'ibn-rushd-burhan', 'shatibi-maqasid', 'ibn-khaldun-umran',
		'pardes', 'maimonidean-prophecy', 'talmudic-middot',
		'mencian-sprouts', 'wang-yangming', 'korean-songnihak',
		'mignolo-pluriversal', 'dussel-transmodernity', 'maldonado-torres',
		'akan-wiredu', 'ibuanyidanda',
		// MIG-037 P1 (2026-05-19) — Time Dome added under v6.3 pivot.
		'time-dome',
	];

	function curatedMeta(id: TraditionId): TraditionMeta {
		return {
			name: $t(`sight.v6.tradition.list.${id}.name`),
			tooltip: $t(`sight.v6.tradition.list.${id}.tooltip`),
			scope: $t(`sight.v6.tradition.list.${id}.scope`),
			preview: false,
		};
	}

	let dropdownOpen = $state(false);
	let rootEl = $state<HTMLDivElement | null>(null);
	// Track which family sections are expanded in the dropdown.
	// Default: all families expanded so user sees the full menu on first open.
	// MIG-026 Phase κ.1 — synthetic 'user-defined' family id added to
	// the expanded-set so user traditions show on first open too.
	let expandedFamilies = $state<Set<string>>(
		new Set<string>([...(Object.keys(FAMILIES) as string[]), 'user-defined']),
	);

	const activeId = $derived<string>(
		($appSettings.sight?.activeTradition as string | undefined) ?? 'aristotelian',
	);

	// Favorite ids from settings (default: 4 production traditions).
	const favoriteIds = $derived<string[]>(
		($appSettings.sight?.favoriteTraditions as string[] | undefined) ??
			['aristotelian', 'pramana', 'masadir', 'polanyi']
	);

	// Per-tradition lookup for user-defined entries (built from the prop).
	// Maps a user id (e.g. 'user-example-three-acts') to its TraditionMeta-
	// shaped record. Synthesized from UserTraditionModule fields so the
	// dropdown rendering code can treat curated + user entries uniformly.
	const userTraditionMeta = $derived.by<Record<string, TraditionMeta>>(() => {
		const m: Record<string, TraditionMeta> = {};
		for (const t of userTraditions) {
			m[t.id] = {
				name: t.name,
				tooltip: t.tooltip || t.name,
				scope: t.scope || '',
				preview: false,
			};
		}
		return m;
	});

	// Inline chips = first 4 favorites that have metadata. Now checks
	// CURATED_TRADITION_IDS (curated) and the userTraditionMeta map so
	// a user-defined tradition pinned via the dropdown's star surfaces
	// inline correctly.
	const inlineChips = $derived<string[]>(
		favoriteIds.filter((id) => CURATED_TRADITION_IDS.includes(id as TraditionId) || id in userTraditionMeta).slice(0, 4)
	);

	// Families that have ≥1 tradition listed in FAMILIES. Mohist's
	// 'chinese-pragmatist' family appears even before Phase γ ships
	// the Mohist module (clicking the chip writes activeTradition;
	// dome fall-back to Aristotelian positions until Phase γ lands
	// the module). Empty families (Phase ε/ζ/η/θ pending) are hidden.
	//
	// MIG-026 Phase κ.1: synthetic 'user-defined' section appended
	// at the bottom when userTraditions.length > 0. All user-defined
	// modules go there regardless of their JSON's `family` field
	// (the field is reserved for κ.2 where it can mix into curated
	// families with permission-style consent).
	type FamilySection = { id: string; label: string; traditions: string[] };
	// MIG-026 §λ-fix-2 — family.label resolves via $t() at render
	// time from sight.v6.tradition.family.<id> keys. FAMILIES.label
	// in traditions/index.ts is still the EN source-of-truth fallback
	// (used when a locale's family key is missing — i18n fallback
	// chain handles this gracefully).
	const familiesWithTraditions = $derived.by<FamilySection[]>(() => {
		const curated = (Object.entries(FAMILIES) as [FamilyId, { label: string; traditions: TraditionId[] }][])
			.filter(([, fam]) => fam.traditions.length > 0)
			.map(([id, _fam]) => ({
				id: id as string,
				label: $t(`sight.v6.tradition.family.${id}`),
				traditions: _fam.traditions as string[],
			}));
		if (userTraditions.length > 0) {
			curated.push({
				id: 'user-defined',
				label: $t('sight.v6.tradition.chip.userDefinedFamily'),
				traditions: userTraditions.map((ut) => ut.id),
			});
		}
		return curated;
	});

	function activeMeta(id: string): TraditionMeta {
		if (CURATED_TRADITION_IDS.includes(id as TraditionId)) {
			return curatedMeta(id as TraditionId);
		}
		const user = userTraditionMeta[id];
		if (user) return user;
		return curatedMeta('aristotelian');
	}

	function isFavorite(id: string): boolean {
		return favoriteIds.includes(id);
	}

	function handleChipClick(id: string) {
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

	function handleFamilyToggle(familyId: string) {
		const next = new Set(expandedFamilies);
		if (next.has(familyId)) {
			next.delete(familyId);
		} else {
			next.add(familyId);
		}
		expandedFamilies = next;
	}

	function handlePinToggle(id: string) {
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

	// MIG-026 Phase ι.2 — cascade-close the parent's manifest modal
	// whenever the dropdown closes. Prevents the modal from floating
	// over a closed dropdown after an Esc / click-outside / tradition
	// switch. The effect fires on every dropdownOpen transition; the
	// parent's closeManifestModal is idempotent (no-op when already
	// closed) so the on-mount fire is harmless.
	$effect(() => {
		if (!dropdownOpen) onDropdownClose();
	});

	// MIG-026 Phase ι.2 — ⓘ button click handler. Calls the parent's
	// openManifest callback; explicitly does NOT close the dropdown so
	// the user can dismiss the modal and return to the dropdown to
	// pick another tradition's manifest.
	function handleManifestClick(ev: MouseEvent, id: string) {
		ev.stopPropagation();
		openManifest(id);
	}
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
					<span class="chip-preview-badge" title={$t('sight.v6.tradition.chip.previewBadgeTooltip')}>{$t('sight.v6.tradition.chip.previewBadge')}</span>
				{/if}
			</button>
		{/each}
		<button
			class="tradition-chip-all-trigger"
			class:is-open={dropdownOpen}
			type="button"
			onclick={handleAllClick}
			title={$t('sight.v6.tradition.chip.allTriggerTooltip')}
			aria-haspopup="true"
			aria-expanded={dropdownOpen}
		>
			{$t('sight.v6.tradition.chip.allTrigger')} <span class="trigger-chevron" class:is-open={dropdownOpen}>▾</span>
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
											<span class="chip-preview-badge">{$t('sight.v6.tradition.chip.previewBadge')}</span>
										{/if}
										<span class="tradition-row-scope">{meta.scope}</span>
									</button>
									<button
										class="tradition-row-info-btn"
										type="button"
										onclick={(ev) => handleManifestClick(ev, id)}
										title={$t('sight.v6.tradition.chip.manifestButtonTooltip')}
										aria-label={$t('sight.v6.tradition.chip.manifestButtonAriaLabel')}
									>
										ⓘ
									</button>
									<button
										class="tradition-row-pin-btn"
										class:is-pinned={fav}
										type="button"
										onclick={() => handlePinToggle(id)}
										title={fav ? $t('sight.v6.tradition.chip.unpinTooltip') : $t('sight.v6.tradition.chip.pinTooltip')}
										aria-label={fav ? $t('sight.v6.tradition.chip.unpinAriaLabel') : $t('sight.v6.tradition.chip.pinAriaLabel')}
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

	/* MIG-026 Phase ι.2 — ⓘ disclosure button. Sits between the
	   row-name button and the pin star. Theme-aware text color via
	   --text-faint / --text-normal CSS vars (set by the active theme
	   in +layout.svelte) so the button reads correctly on both light
	   and dark themes. Click opens the bundled manifest in a modal
	   inside SightV6 (parent handles state + render). */
	.tradition-row-info-btn {
		flex: 0 0 auto;
		display: inline-flex;
		align-items: center;
		justify-content: center;
		width: 28px;
		font-size: 14px;
		line-height: 1;
		color: var(--text-faint, #5a6275);
		background: transparent;
		border: none;
		border-radius: 3px;
		cursor: pointer;
		transition: color 0.12s ease, background 0.12s ease;
	}
	.tradition-row-info-btn:hover {
		color: var(--text-normal, #c8cdd9);
		background: var(--background-modifier-hover, rgba(74, 90, 130, 0.18));
	}
</style>
