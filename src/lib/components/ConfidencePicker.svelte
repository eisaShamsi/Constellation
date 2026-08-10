<script lang="ts">
	import { t, dir, isRTL } from '$lib/i18n';
	import { setLinkConfidence, archiveLink, type LinkConfidence } from '$lib/libraries/store';

	// MIG-077 A4 — the shared confidence/archive popover.
	//
	// Extracted from the byte-identical inline copies that lived in BacklinksPanel
	// and OutgoingLinksPanel. ONE source of truth (Editor-Parity / reuse rule): the
	// component owns the popover markup, its CSS (self-contained — own size via
	// --rs-scale), the link IPC (setLinkConfidence / archiveLink) and dismissal. A
	// host opens it by setting `menu` on a backlink / outgoing-link row right-click;
	// after a successful write the component fires the host callback so the host can
	// refresh its local state, then closes. No host duplicates this UI again.
	let {
		menu,
		onConfidenceChange = undefined,
		onArchive = undefined,
		onClose
	}: {
		menu: { x: number; y: number; sourcePath: string; targetName: string; current: LinkConfidence } | null;
		onConfidenceChange?: (sourcePath: string, targetName: string, confidence: LinkConfidence) => void;
		onArchive?: (sourcePath: string, targetName: string) => void;
		onClose: () => void;
	} = $props();

	const CONFIDENCE_LEVELS: LinkConfidence[] = ['hypothesis', 'evidence', 'established', 'contested'];

	// MIG-077 RTL: in an RTL UI anchor the popover's right edge at the cursor.
	const confRight = $derived(menu ? window.innerWidth - menu.x : 0);

	// Dismiss on outside click / right-click / Escape — the same mechanism as the
	// shared <ContextMenu> (no backdrop div: consistent + a11y-clean). Listeners are
	// registered only while open, after a tick so the opening click can't close it.
	let menuEl: HTMLDivElement | undefined = $state();
	$effect(() => {
		if (!menu) return;
		function onDocPointer(e: MouseEvent) {
			if (menuEl && !menuEl.contains(e.target as Node)) onClose();
		}
		function onKey(e: KeyboardEvent) { if (e.key === 'Escape') onClose(); }
		const timer = setTimeout(() => {
			document.addEventListener('click', onDocPointer);
			document.addEventListener('contextmenu', onDocPointer);
			document.addEventListener('keydown', onKey);
		}, 10);
		return () => {
			clearTimeout(timer);
			document.removeEventListener('click', onDocPointer);
			document.removeEventListener('contextmenu', onDocPointer);
			document.removeEventListener('keydown', onKey);
		};
	});

	/**
	 * PJ-207 §15 — the popover closed FIRST and then swallowed the failure, so a link whose
	 * confidence (or archive) never reached the database looked exactly like one that had: the
	 * menu vanished, the badge kept its old value, and nothing said why. Both actions now close
	 * only on success; on failure the menu stays open with the reason, which is the one moment
	 * the user is still looking at the thing they just tried to change.
	 */
	let writeError = $state('');

	async function applyConf(level: LinkConfidence) {
		if (!menu) return;
		const { sourcePath, targetName } = menu;
		try {
			await setLinkConfidence(sourcePath, targetName, level);
			onClose();
			onConfidenceChange?.(sourcePath, targetName, level);
		} catch (e) {
			writeError = String((e as { message?: string })?.message ?? e);
			console.error('[ConfidencePicker] setLinkConfidence failed:', e);
		}
	}
	async function applyArchive() {
		if (!menu) return;
		const { sourcePath, targetName } = menu;
		try {
			await archiveLink(sourcePath, targetName);
			onClose();
			onArchive?.(sourcePath, targetName);
		} catch (e) {
			writeError = String((e as { message?: string })?.message ?? e);
			console.error('[ConfidencePicker] archiveLink failed:', e);
		}
	}
</script>

{#if menu}
	<div class="conf-menu" bind:this={menuEl} dir={$dir} style="{$isRTL ? `right:${confRight}px` : `left:${menu.x}px`};top:{menu.y}px">
		<div class="conf-menu-header">{$t('linkConfidence.setConfidence') || 'Set confidence'}</div>
		{#if writeError}
			<div class="conf-menu-error" role="alert">{$t('linkConfidence.notSaved')}</div>
		{/if}
		{#each CONFIDENCE_LEVELS as level}
			<button class="conf-menu-item" class:active={level === menu.current} onclick={() => applyConf(level)}>
				<span class="conf-dot conf-dot-{level}"></span>
				{$t(`linkConfidence.${level}`) || level}
			</button>
		{/each}
		<div class="conf-menu-sep"></div>
		<button class="conf-menu-item conf-menu-archive" onclick={applyArchive}>
			<svg width="10" height="10" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M21 8v13H3V8M1 3h22v5H1zM10 12h4"/></svg>
			{$t('linkConfidence.archive') || 'Archive link'}
		</button>
	</div>
{/if}

<style>
	/* Confidence popover — self-contained visual grammar (was duplicated in
	   BacklinksPanel + OutgoingLinksPanel before MIG-077 A4). */
	.conf-menu {
		position: fixed; z-index: 100;
		background: var(--bg-secondary, #fff);
		border: 1px solid var(--border); border-radius: 6px;
		box-shadow: var(--popover-shadow, 0 8px 20px rgba(0,0,0,0.18));
		padding: 4px; min-width: 160px;
		font-size: calc(0.78rem * var(--rs-scale, 1));
	}
	.conf-menu-header {
		padding: 6px 8px 4px; color: var(--text-muted); font-size: calc(0.68rem * var(--rs-scale, 1));
		text-transform: uppercase; letter-spacing: 0.04em; font-weight: 600;
	}
	.conf-menu-error {
		padding: 4px 8px 6px;
		color: var(--danger, #ff6b6b);
		font-size: calc(0.7rem * var(--rs-scale, 1));
		line-height: 1.35;
		max-width: 22ch;
	}
	.conf-menu-item {
		display: flex; align-items: center; gap: 8px;
		width: 100%; padding: 6px 8px; border: none; background: none;
		cursor: pointer; border-radius: 4px; text-align: start;
		color: var(--text-normal); font-family: inherit; font-size: calc(0.78rem * var(--rs-scale, 1));
	}
	.conf-menu-item:hover { background: var(--background-modifier-hover); }
	.conf-menu-item.active { font-weight: 600; color: var(--interactive-accent); }
	.conf-dot {
		width: 8px; height: 8px; border-radius: 50%; flex-shrink: 0;
		border: 1px solid var(--border);
	}
	/* MIG-088 §2b — shared Confidence colours (Style Setter → Cognitive colours); fallback = today's value. */
	.conf-dot-hypothesis { background: var(--confidence-hypothesis, color-mix(in srgb, var(--interactive-accent, #7c3aed) 14%, transparent)); }
	.conf-dot-evidence   { background: var(--confidence-evidence, color-mix(in srgb, var(--interactive-accent, #7c3aed) 40%, transparent)); }
	.conf-dot-established{ background: var(--confidence-established, var(--interactive-accent, #7c3aed)); border-color: var(--confidence-established, var(--interactive-accent, #7c3aed)); }
	.conf-dot-contested  { background: var(--confidence-contested, #d97706); border-color: var(--confidence-contested, #d97706); }
	.conf-menu-sep { height: 1px; margin: 4px 4px; background: var(--border-light, var(--border)); }
	.conf-menu-archive { color: var(--text-muted); }
	.conf-menu-archive:hover { color: #d97706; }
	.conf-menu-archive svg { flex-shrink: 0; }
</style>
