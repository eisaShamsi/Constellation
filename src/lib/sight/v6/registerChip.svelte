<!--
  MIG-025 §C.1 — Register chip component.

  Location: Sight title bar, mounted between subtitle and EXTENDED badge.
  Per Concept Paper §2.5: default state collapsed, shows current active
  register only (e.g., "Aristotelian ●"). Click → expand to show all 7
  registers. Active register has blue stroke + dot. Hover any chip → English
  secondary label tooltip per Concept Paper §2.5 + §11 invariant.

  v1-preview registers (Dignāga, Suhrawardi Ishrāqī, Mohist sān biǎo) per
  §4.2 carry a "preview" badge — they ship fully functional in v6 but with
  v4.1 polish targets for deeper internal structure.

  §C.1 partial-ships §C.8: clicking a chip writes activeRegister to
  appSettings.sight via the canonical update+saveSettings pattern (same as
  Cmd-Shift-D extended toggle at SightV6.svelte §B.10). The anchor-re-render
  based on activeRegister happens in §C.2 (register modules).

  Brand-name register labels are kept English per the §A.15 brand convention
  (same precedent as Constellation, Sight, CNS, Confidence). The cultural
  diacritics (pramāṇa, masādir, Dignāga, Suhrawardi Ishrāqī, Mohist sān biǎo)
  are Unicode and render in any modern font stack.

  Concept Paper: docs/Constellation-Sight-Concept-Paper-v4.0.md §2.5, §4.1, §4.2
  Plan:         lab/reports/MIG-025-SIGHT-V6-PLAN.md §C.1
-->
<script lang="ts">
	import { appSettings, saveSettings } from '$lib/libraries/store';
	import type { RegisterId } from './types';

	// Seven registers in canonical order: 4 production-polish first (§4.1),
	// then 3 v1-preview (§4.2). Tooltip prose distills each register's
	// English secondary label per Concept Paper §2.5 + §4.1/§4.2.
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
			id: 'dignaga',
			name: 'Dignāga',
			tooltip: 'Dignāga — Buddhist epistemological critique, two pramāṇas only (v1 preview)',
			preview: true,
		},
		{
			id: 'ishraqi',
			name: 'Suhrawardi Ishrāqī',
			tooltip: 'Suhrawardi Ishrāqī — presence-knowledge as foundation (v1 preview)',
			preview: true,
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
			ev.stopPropagation();
		}
	}

	$effect(() => {
		if (!expanded) return;
		document.addEventListener('mousedown', handleOutsideClick);
		document.addEventListener('keydown', handleKey);
		return () => {
			document.removeEventListener('mousedown', handleOutsideClick);
			document.removeEventListener('keydown', handleKey);
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
