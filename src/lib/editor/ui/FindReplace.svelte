<script lang="ts">
	import { t } from '$lib/i18n';

	let {
		onfind,
		onreplace,
		onreplaceall,
		onclose,
		visible = false,
	}: {
		onfind?: (query: string, direction: 'next' | 'prev') => void;
		onreplace?: (query: string, replacement: string) => void;
		onreplaceall?: (query: string, replacement: string) => void;
		onclose?: () => void;
		visible?: boolean;
	} = $props();

	let findInput = $state('');
	let replaceInput = $state('');
	let matchCount = $state(0);
	let currentMatch = $state(0);

	function handleFindNext() {
		if (onfind && findInput) onfind(findInput, 'next');
	}

	function handleFindPrev() {
		if (onfind && findInput) onfind(findInput, 'prev');
	}

	function handleReplace() {
		if (onreplace && findInput) onreplace(findInput, replaceInput);
	}

	function handleReplaceAll() {
		if (onreplaceall && findInput) onreplaceall(findInput, replaceInput);
	}

	function handleKeyDown(e: KeyboardEvent) {
		if (e.key === 'Escape') {
			if (onclose) onclose();
		} else if (e.key === 'Enter') {
			if (e.shiftKey) handleFindPrev();
			else handleFindNext();
		}
	}
</script>

{#if visible}
	<!-- svelte-ignore a11y_no_static_element_interactions -->
	<div class="ce-find-replace" onkeydown={handleKeyDown}>
		<div class="ce-find-row">
			<input
				type="text"
				class="ce-find-input"
				placeholder={$t('editor.find')}
				bind:value={findInput}
				oninput={handleFindNext}
			/>
			<span class="ce-find-count">{currentMatch}/{matchCount}</span>
			<button class="ce-find-btn" title={$t('editor.next')} onclick={handleFindPrev}>
				<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><polyline points="18 15 12 9 6 15"/></svg>
			</button>
			<button class="ce-find-btn" title={$t('editor.next')} onclick={handleFindNext}>
				<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><polyline points="6 9 12 15 18 9"/></svg>
			</button>
			<button class="ce-find-btn" onclick={() => { if (onclose) onclose(); }}>
				<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><line x1="18" y1="6" x2="6" y2="18"/><line x1="6" y1="6" x2="18" y2="18"/></svg>
			</button>
		</div>
		<div class="ce-find-row">
			<input
				type="text"
				class="ce-find-input"
				placeholder={$t('editor.replace')}
				bind:value={replaceInput}
			/>
			<button class="ce-find-btn-text" onclick={handleReplace}>{$t('editor.replace')}</button>
			<button class="ce-find-btn-text" onclick={handleReplaceAll}>{$t('editor.replaceAll')}</button>
		</div>
	</div>
{/if}

<style>
	.ce-find-replace {
		position: absolute;
		top: 0;
		right: 16px;
		z-index: 100;
		background: var(--bg-primary, #fff);
		border: 1px solid var(--border-color, #e0e0e4);
		border-radius: 0 0 8px 8px;
		box-shadow: 0 4px 12px rgba(0, 0, 0, 0.1);
		padding: 8px 12px;
		display: flex;
		flex-direction: column;
		gap: 6px;
		font-family: var(--font-interface, -apple-system, BlinkMacSystemFont, 'Segoe UI', Inter, sans-serif);
		font-size: 13px;
	}

	.ce-find-row {
		display: flex;
		align-items: center;
		gap: 4px;
	}

	.ce-find-input {
		flex: 1;
		height: 28px;
		border: 1px solid var(--border-color, #e0e0e4);
		border-radius: 4px;
		padding: 0 8px;
		font-size: 13px;
		background: var(--bg-primary, #fff);
		color: var(--text-primary, #1f2328);
		outline: none;
		min-width: 180px;
	}

	.ce-find-input:focus {
		border-color: var(--accent-color, #7c3aed);
	}

	.ce-find-count {
		color: var(--text-muted, #8b8b8b);
		font-size: 12px;
		white-space: nowrap;
		min-width: 40px;
		text-align: center;
	}

	.ce-find-btn {
		display: flex;
		align-items: center;
		justify-content: center;
		width: 24px;
		height: 24px;
		border: none;
		background: transparent;
		border-radius: 4px;
		cursor: pointer;
		color: var(--text-secondary, #5c5c66);
	}

	.ce-find-btn:hover {
		background: var(--bg-hover, #e8e8ec);
	}

	.ce-find-btn-text {
		height: 24px;
		border: 1px solid var(--border-color, #e0e0e4);
		background: transparent;
		border-radius: 4px;
		cursor: pointer;
		color: var(--text-secondary, #5c5c66);
		padding: 0 8px;
		font-size: 12px;
		white-space: nowrap;
	}

	.ce-find-btn-text:hover {
		background: var(--bg-hover, #e8e8ec);
	}
</style>
