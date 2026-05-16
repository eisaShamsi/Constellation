<!--
  MIG-025 §C.1 — Register chip component.

  Location: Sight title bar, mounted between subtitle and EXTENDED badge.
  Per Concept Paper §2.5: default state collapsed, shows current active
  register only (e.g., "Aristotelian ●"). Click → expand to show all 5
  registers. Active register has blue stroke + dot. Hover any chip → English
  secondary label tooltip per Concept Paper §2.5 + §11 invariant.

  v1-preview register (Mohist sān biǎo) per §4.2 carries a "preview" badge —
  it ships fully functional in v6 but with v4.1 polish targets for deeper
  internal structure.

  §C.4-religious-rule (Eisa 2026-05-16): Suhrawardi Ishrāqī register
  EXCLUDED entirely per the new top-principal religious-lineage rule (see
  orientation v2.09). The Ishrāqī tradition is overwhelmingly absorbed into
  Twelver Shīʿī ḥikma (Mulla Sadra, Sabzavari, modern Qom curriculum) and
  is also fundamentally religious-mystical rather than philosophical-
  epistemological. The register set shrinks from 6 to 5. Concept Paper
  §4.2.2 (Ishrāqī geometry spec) and Plan §D.2 (Ishrāqī build step) both
  carry EXCLUDED notes; the RegisterId type no longer admits 'ishraqi';
  a settings migration in store.ts applyParsedSettings rewrites any
  persisted 'ishraqi' value back to 'aristotelian'.

  §C.1-fix-1 (Eisa 2026-05-16): Dignāga register EXCLUDED entirely per
  Eisa's direction "don't include the 'Dignāga' at all in any of Constellation
  functions". Same exclusion mechanism as Ishrāqī above.

  §C.1-fix-1 (Eisa 2026-05-16): Esc-while-chip-expanded bug fixed.
  Previously chip Esc handler was on document (bubble phase) but
  +layout.svelte:2335 registers the global Esc-closes-Sight handler on
  document in capture phase — Layout's handler fired first and closed Sight.
  Fix: chip handler now registers on window (which sits OUTSIDE document
  in the capture chain) in capture phase. stopPropagation + preventDefault
  kill the event before it reaches the Layout handler.

  §C.1 partial-ships §C.8: clicking a chip writes activeRegister to
  appSettings.sight via the canonical update+saveSettings pattern (same as
  Cmd-Shift-D extended toggle at SightV6.svelte §B.10). The anchor-re-render
  based on activeRegister happens in §C.2 (register modules).

  Brand-name register labels are kept English per the §A.15 brand convention
  (same precedent as Constellation, Sight, CNS, Confidence). The cultural
  diacritics (pramāṇa, masādir, Mohist sān biǎo) are Unicode and render in
  any modern font stack.

  Concept Paper: docs/Constellation-Sight-Concept-Paper-v4.0.md §2.5, §4.1, §4.2
  Plan:         lab/reports/MIG-025-SIGHT-V6-PLAN.md §C.1
-->
<script lang="ts">
	import { appSettings, saveSettings } from '$lib/libraries/store';
	import type { RegisterId } from './types';

	// Five registers in canonical order: 4 production-polish first (§4.1),
	// then 1 v1-preview (§4.2). Tooltip prose distills each register's
	// English secondary label per Concept Paper §2.5 + §4.1/§4.2.
	// §C.1-fix-1: Dignāga register excluded entirely (was 7 → 6).
	// §C.4-religious-rule: Suhrawardi Ishrāqī also excluded (was 6 → 5)
	//   per the new top-principal religious-lineage rule. The 'dignaga'
	//   and 'ishraqi' literals are both removed from RegisterId.
	type RegisterDef = {
		id: RegisterId;
		name: string;          // chip label (kept English per §A.15 brand convention)
		tooltip: string;       // hover tooltip per §2.5
		preview: boolean;      // v1-preview badge per §4.2
	};

	const REGISTERS: RegisterDef[] = [
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

	const activeId = $derived<RegisterId>(
		($appSettings.sight?.activeRegister as RegisterId | undefined) ?? 'aristotelian'
	);
	const activeDef = $derived(REGISTERS.find((r) => r.id === activeId) ?? REGISTERS[0]);

	function handleCollapsedClick() {
		expanded = !expanded;
	}

	function handleSelect(id: RegisterId) {
		// Canonical write pattern (matches SightV6 §B.10 extended toggle):
		// immutable update on appSettings.sight, then saveSettings() to persist.
		appSettings.update((s) => ({
			...s,
			sight: { ...s.sight, activeRegister: id },
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
			// global handler (+layout.svelte:2335) sees it and closes
			// Sight. stopPropagation + preventDefault are belt-and-braces;
			// the real fix is registering on `window` (capture) below so
			// we run BEFORE Layout's `document` (capture) handler.
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

<div bind:this={rootEl} class="register-chip-root">
	{#if !expanded}
		<button
			class="register-chip-collapsed"
			type="button"
			onclick={handleCollapsedClick}
			title="Active epistemic register. Click to switch."
			aria-haspopup="listbox"
			aria-expanded="false"
		>
			<span class="chip-name">{activeDef.name}</span>
			<span class="chip-dot" aria-hidden="true"></span>
		</button>
	{:else}
		<div class="register-chip-expanded" role="listbox" aria-label="Epistemic register">
			{#each REGISTERS as reg (reg.id)}
				<button
					class="register-chip"
					class:is-active={reg.id === activeId}
					class:is-preview={reg.preview}
					type="button"
					role="option"
					aria-selected={reg.id === activeId}
					title={reg.tooltip}
					onclick={() => handleSelect(reg.id)}
				>
					<span class="chip-name">{reg.name}</span>
					{#if reg.id === activeId}
						<span class="chip-dot" aria-hidden="true"></span>
					{/if}
					{#if reg.preview}
						<span class="chip-preview-badge" title="v1 preview — deeper internal structure is a v4.1 polish target">preview</span>
					{/if}
				</button>
			{/each}
		</div>
	{/if}
</div>

<style>
	/* §C.1 — Register chip. Sits in the Sight title bar between subtitle
	   and EXTENDED badge. Collapsed default shows only the active register;
	   click expands to a horizontal row of all 7 registers.

	   Color palette derived from the existing header palette:
	   - chip background:  rgba(58, 67, 90, 0.35)  (same as filter-count)
	   - active border:    #3b5998                 (blue, same as Reset View)
	   - preview accent:   #c9a155                 (muted gold, distinct
	                                                from the #fbbf24 used by
	                                                EXTENDED so the eye can
	                                                distinguish them). */

	.register-chip-root {
		display: inline-flex;
		align-items: center;
		position: relative;
	}

	.register-chip-collapsed {
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
	.register-chip-collapsed:hover {
		background: rgba(74, 90, 130, 0.55);
	}

	.register-chip-expanded {
		display: inline-flex;
		align-items: center;
		gap: 4px;
		padding: 2px;
		background: rgba(58, 67, 90, 0.35);
		border: 1px solid rgba(59, 89, 152, 0.55);
		border-radius: 5px;
	}

	.register-chip {
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
	.register-chip:hover {
		color: #e8ebf2;
		background: rgba(74, 90, 130, 0.35);
	}
	.register-chip.is-active {
		color: #e8ebf2;
		border-color: #3b5998;
		background: rgba(59, 89, 152, 0.18);
	}
	.register-chip.is-active:hover {
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
