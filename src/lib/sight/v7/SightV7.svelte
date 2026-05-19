<!--
  MIG-036 P1 (2026-05-19) — Sight v7 placeholder mount.

  This is the parallel-mount entry point for the Form-Aligns-To-
  Purpose redesign. Gated behind SIGHT_V7_ENABLED (currently false
  in engine.ts); production users see v6 until ship-gate flips the
  flag.

  This file is intentionally minimal for P1. Subsequent phases
  fill in:
    P2 — density.ts + stack.ts pure-function rendering primitives
    P3 — anchor-v7.ts universe-view dispatcher (per-cell density)
    P4–P5 — per-shape redesign for the 9 renderers
    P6 — Aristotelian rewrite + Time Dome new module
    P7 — Cell view (drill-in interaction)
    P8 — Mini-dome adaptation
    P9 — Dropdown reorganization (Time group + 24 traditions)
    P10 — User Manual chapter rewrite
    P11 — Flag flip + ship gate

  Architect doc: lab/reports/MIG-036-SIGHT-V7-ARCHITECT.md
-->
<script lang="ts">
	import { t } from '$lib/i18n';

	let {
		onOpenNote: _onOpenNote = () => {},
	}: {
		onOpenNote?: (path: string, libraryName: string) => void;
	} = $props();
</script>

<div class="sight-v7-root">
	<div class="sight-v7-header">
		<span class="sight-v7-title">{$t('sight.v6.title')}</span>
		<span class="sight-v7-subtitle">v7.0 — Form-Aligns-To-Purpose (P1 placeholder)</span>
	</div>
	<div class="sight-v7-placeholder">
		<p>
			Sight v7 — Form-Aligns-To-Purpose redesign is under construction.
			This is the P1 scaffolding mount; subsequent phases fill in the
			Hybrid X+Y rendering (density at universe view, stack at cell
			drill-in), Time Dome as a separate entry, and Aristotelian as a
			pure-radial maturity tower.
		</p>
		<p>
			See <code>lab/reports/MIG-036-SIGHT-V7-ARCHITECT.md</code> for the
			full design.
		</p>
	</div>
</div>

<style>
	.sight-v7-root {
		flex: 1 1 auto;
		display: flex;
		flex-direction: column;
		background: var(--background-primary, #0c1322);
		color: var(--text-normal, #cdd5e0);
		font-family: var(--interface-font, 'Inter', system-ui, sans-serif);
	}
	.sight-v7-header {
		flex: 0 0 auto;
		display: flex;
		align-items: baseline;
		gap: 14px;
		padding: 12px 18px;
		border-bottom: 1px solid var(--background-modifier-border, #1a1f2e);
	}
	.sight-v7-title {
		font-size: 16px;
		font-weight: 600;
		color: var(--text-normal, #cdd5e0);
	}
	.sight-v7-subtitle {
		font-size: 11px;
		font-style: italic;
		color: var(--text-faint, #5a6275);
	}
	.sight-v7-placeholder {
		flex: 1 1 auto;
		padding: 24px 40px;
		max-width: 720px;
		line-height: 1.6;
		font-size: 14px;
		color: var(--text-muted, #a0aabe);
	}
	.sight-v7-placeholder code {
		font-family: var(--mono-font, 'Fira Code', monospace);
		font-size: 13px;
		background: var(--background-modifier-hover, #13192b);
		padding: 1px 6px;
		border-radius: 3px;
	}
</style>
