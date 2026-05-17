<!--
  MIG-025 §C.1 — Tradition chip component (renamed from Register chip).
  MIG-026 Phase 0 — K1 full rename: "register" → "tradition" throughout.

  Location: Sight title bar, mounted between subtitle and EXTENDED badge.
  Per Concept Paper §2.5: default state collapsed, shows current active
  tradition only (e.g., "Aristotelian ●"). Click → expand to show all 5
  baseline traditions (MIG-026 Phases γ–θ add 19 more to bring the
  curated set to 24). Active tradition has blue stroke + dot. Hover any
  chip → English secondary label tooltip per Concept Paper §2.5 +
  §11 invariant.

  v1-preview tradition (Mohist sān biǎo) per §4.2 carries a "preview"
  badge — it ships fully functional in v6 but with v4.1 polish targets
  for deeper internal structure.

  §C.4-religious-rule (Eisa 2026-05-16): Suhrawardi Ishrāqī tradition
  EXCLUDED entirely per the new top-principal religious-lineage rule
  (orientation v2.09). The Ishrāqī tradition was overwhelmingly absorbed
  into Twelver Shīʿī ḥikma (Mulla Sadra, Sabzavari, modern Qom curriculum)
  and is fundamentally religious-mystical rather than philosophical-
  epistemological. The tradition set shrunk from 6 to 5. Concept Paper
  §4.2.2 and Plan §D.2 both carry EXCLUDED notes; the TraditionId type
  no longer admits 'ishraqi'; a settings migration in store.ts
  applyParsedSettings rewrites any persisted 'ishraqi' value back to
  'aristotelian'.

  §C.1-fix-1 (Eisa 2026-05-16): Dignāga tradition EXCLUDED entirely
  per Eisa's direction. Same exclusion mechanism as Ishrāqī above.

  §C.1-fix-1 (Eisa 2026-05-16): Esc-while-chip-expanded bug fix.
  Previously chip Esc handler was on document (bubble phase) but
  +layout.svelte registers the global Esc-closes-Sight handler on
  document in capture phase — Layout's handler fired first and closed
  Sight. Fix: chip handler now registers on window (which sits OUTSIDE
  document in the capture chain) in capture phase. stopPropagation +
  preventDefault kill the event before it reaches the Layout handler.

  §C.1 partial-ships §C.8: clicking a chip writes activeTradition to
  appSettings.sight via the canonical update+saveSettings pattern (same
  as Cmd-Shift-D extended toggle at SightV6.svelte §B.10). The anchor-
  re-render based on activeTradition happens in §C.2 (tradition modules).

  Brand-name tradition labels are kept English per the §A.15 brand
  convention (same precedent as Constellation, Sight, CNS, Confidence).
  The cultural diacritics (pramāṇa, masādir, Mohist sān biǎo) are
  Unicode and render in any modern font stack.

  Concept Paper: docs/Constellation-Sight-Concept-Paper-v4.0.md §2.5, §4.1, §4.2
  Plan:         lab/reports/MIG-025-SIGHT-V6-PLAN.md §C.1 +
                 lab/reports/MIG-026-SIGHT-REGISTER-EXPANSION-PLAN.md §2
-->
<script lang="ts">
	import { appSettings, saveSettings } from '$lib/libraries/store';
	import type { TraditionId } from './types';

	// Five baseline traditions in canonical order: 4 production-polish first
	// (§4.1), then 1 v1-preview (§4.2). Tooltip prose distills each
	// tradition's English secondary label per Concept Paper §2.5 + §4.1/§4.2.
	// §C.1-fix-1: Dignāga tradition excluded entirely (was 7 → 6).
	// §C.4-religious-rule: Suhrawardi Ishrāqī also excluded (was 6 → 5)
	//   per the new top-principal religious-lineage rule. The 'dignaga'
	//   and 'ishraqi' literals are both removed from TraditionId.
	// MIG-026 Phases γ–θ extend this list to 24 traditions.
	type TraditionDef = {
		id: TraditionId;
		name: string;          // chip label (kept English per §A.15 brand convention)
		tooltip: string;       // hover tooltip per §2.5
		preview: boolean;      // v1-preview badge per §4.2
	};

	const TRADITIONS: TraditionDef[] = [
		{
			id: 'aristotelian',
			name: 'Aristotelian',
			tooltip: 'Aristotelian — Western-classical, knowledge as maturity gradient',
			preview: false,
		},
		{
			id: 'pramana',
			name: 'pramāṇa',
			tooltip: 'pramāṇa — Nyāya fourfold valid means of knowing',
			preview: false,
		},
		{
			id: 'masadir',
			name: 'masādir',
			tooltip: 'masādir — Sunni uṣūl al-fiqh, sources as kinds of proof',
			preview: false,
		},
		{
			id: 'polanyi',
			name: 'Polanyi',
			tooltip: 'Polanyi — modern pluralism, tacit as the proximal pole',
			preview: false,
		},
		{
			id: 'mohist-san-biao',
			name: 'Mohist sān biǎo',
			tooltip: 'Mohist sān biǎo — three standards as tests of doctrines (v1 preview)',
			preview: true,
		},
	];

	let expanded = $state(false);
	let rootEl = $state<HTMLDivElement | null>(null);

	const activeId = $derived<TraditionId>(
		($appSettings.sight?.activeTradition as TraditionId | undefined) ?? 'aristotelian'
	);
	const activeDef = $derived(TRADITIONS.find((t) => t.id === activeId) ?? TRADITIONS[0]);

	function handleCollapsedClick() {
		expanded = !expanded;
	}

	function handleSelect(id: TraditionId) {
		// Canonical write pattern (matches SightV6 §B.10 extended toggle):
		// immutable update on appSettings.sight, then saveSettings() to persist.
		appSettings.update((s) => ({
			...s,
			sight: { ...s.sight, activeTradition: id },
		}));
		saveSettings();
		expanded = false;
	}

	// Collapse on outside click + Escape so the chip behaves like a popover.
	function handleOutsideClick(ev: MouseEvent) {
		if (!expanded || !rootEl) return;
		if (!rootEl.contains(ev.target as Node)) {
			expanded = false;
		}
	}
	function handleKey(ev: KeyboardEvent) {
		if (ev.key === 'Escape' && expanded) {
			expanded = false;
			// §C.1-fix-1: kill the event before Layout's capture-phase
			// global handler sees it and closes Sight. stopPropagation +
			// preventDefault are belt-and-braces; the real fix is
			// registering on `window` (capture) below so we run BEFORE
			// Layout's `document` (capture) handler.
			ev.stopPropagation();
			ev.preventDefault();
		}
	}

	$effect(() => {
		if (!expanded) return;
		document.addEventListener('mousedown', handleOutsideClick);
		// §C.1-fix-1 (Eisa cycle-1 Stage 6 step 4-5 FAIL: "When I pressed
		// Esc Sight was closed"): chip handler must beat +layout.svelte's
		// global Esc-closes-Sight handler. Layout's handler is on
		// `document` in capture phase; ours goes on `window` in capture
		// phase. Capture order is window → document → ... so a window
		// capture handler fires BEFORE any document capture handler. The
		// stopPropagation inside handleKey then prevents the event from
		// continuing the capture journey down to document. Result:
		// Esc-while-chip-expanded collapses the chip and Sight stays open.
		// Esc-while-chip-collapsed is unaffected (this effect only mounts
		// while `expanded` is true).
		window.addEventListener('keydown', handleKey, true);
		return () => {
			document.removeEventListener('mousedown', handleOutsideClick);
			// Third arg must match addEventListener's capture flag (true)
			// or removeEventListener silently no-ops.
			window.removeEventListener('keydown', handleKey, true);
		};
	});
</script>

<div bind:this={rootEl} class="tradition-chip-root">
	{#if !expanded}
		<button
			class="tradition-chip-collapsed"
			type="button"
			onclick={handleCollapsedClick}
			title="Active scholarly tradition. Click to switch."
			aria-haspopup="listbox"
			aria-expanded="false"
		>
			<span class="chip-name">{activeDef.name}</span>
			<span class="chip-dot" aria-hidden="true"></span>
		</button>
	{:else}
		<div class="tradition-chip-expanded" role="listbox" aria-label="Scholarly tradition">
			{#each TRADITIONS as trad (trad.id)}
				<button
					class="tradition-chip"
					class:is-active={trad.id === activeId}
					class:is-preview={trad.preview}
					type="button"
					role="option"
					aria-selected={trad.id === activeId}
					title={trad.tooltip}
					onclick={() => handleSelect(trad.id)}
				>
					<span class="chip-name">{trad.name}</span>
					{#if trad.id === activeId}
						<span class="chip-dot" aria-hidden="true"></span>
					{/if}
					{#if trad.preview}
						<span class="chip-preview-badge" title="v1 preview — deeper internal structure is a v4.1 polish target">preview</span>
					{/if}
				</button>
			{/each}
		</div>
	{/if}
</div>

<style>
	/* §C.1 — Tradition chip. Sits in the Sight title bar between subtitle
	   and EXTENDED badge. Collapsed default shows only the active tradition;
	   click expands to a horizontal row of baseline traditions (5 today; 24
	   after MIG-026 Phases γ–θ ship; Phase β replaces inline-row with a
	   family-categorized A3+A6 hybrid).

	   Color palette derived from the existing header palette:
	   - chip background:  rgba(58, 67, 90, 0.35)  (same as filter-count)
	   - active border:    #3b5998                 (blue, same as Reset View)
	   - preview accent:   #c9a155                 (muted gold, distinct
	                                                from the #fbbf24 used by
	                                                EXTENDED so the eye can
	                                                distinguish them). */

	.tradition-chip-root {
		display: inline-flex;
		align-items: center;
		position: relative;
	}

	.tradition-chip-collapsed {
		display: inline-flex;
		align-items: center;
		gap: 6px;
		padding: 3px 10px;
		font-size: 11px;
		font-family: inherit;
		color: #e8ebf2;
		background: rgba(58, 67, 90, 0.35);
		border: 1px solid #3b5998;
		border-radius: 4px;
		cursor: pointer;
		transition: background 0.12s ease;
	}
	.tradition-chip-collapsed:hover {
		background: rgba(74, 90, 130, 0.55);
	}

	.tradition-chip-expanded {
		display: inline-flex;
		align-items: center;
		gap: 4px;
		padding: 2px;
		background: rgba(58, 67, 90, 0.35);
		border: 1px solid rgba(59, 89, 152, 0.55);
		border-radius: 5px;
	}

	.tradition-chip {
		display: inline-flex;
		align-items: center;
		gap: 5px;
		padding: 3px 9px;
		font-size: 11px;
		font-family: inherit;
		color: #c8cdd9;
		background: transparent;
		border: 1px solid transparent;
		border-radius: 3px;
		cursor: pointer;
		transition: background 0.12s ease, color 0.12s ease, border-color 0.12s ease;
		white-space: nowrap;
	}
	.tradition-chip:hover {
		color: #e8ebf2;
		background: rgba(74, 90, 130, 0.35);
	}
	.tradition-chip.is-active {
		color: #e8ebf2;
		border-color: #3b5998;
		background: rgba(59, 89, 152, 0.18);
	}
	.tradition-chip.is-active:hover {
		background: rgba(59, 89, 152, 0.28);
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
		background: #3b5998;
		box-shadow: 0 0 4px rgba(59, 89, 152, 0.6);
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
</style>
