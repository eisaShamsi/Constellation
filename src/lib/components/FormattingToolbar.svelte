<script lang="ts">
	import { t } from '$lib/i18n';

	let {
		x = 0,
		y = 0,
		onBold,
		onItalic,
		onStrikethrough,
		onHighlight,
		onCode,
		onLink,
		onHeading,
		onConvertToTable,
		showConvertToTable = false,
	}: {
		x: number;
		y: number;
		onBold: () => void;
		onItalic: () => void;
		onStrikethrough: () => void;
		onHighlight: () => void;
		onCode: () => void;
		onLink: () => void;
		onHeading: (level: number) => void;
		onConvertToTable?: () => void;
		showConvertToTable?: boolean;
	} = $props();

	let showHeadingMenu = $state(false);

	function handleHeading(level: number) {
		showHeadingMenu = false;
		onHeading(level);
	}

	function prevent(e: MouseEvent) { e.preventDefault(); }
</script>

<!-- svelte-ignore a11y_no_static_element_interactions -->
<div
	class="formatting-toolbar"
	style="left: {x}px; top: {y}px;"
	onmousedown={prevent}
>
	<button class="tb-btn" title={$t('toolbar.bold')} onmousedown={prevent} onclick={onBold}>
		<strong>B</strong>
	</button>
	<button class="tb-btn" title={$t('toolbar.italic')} onmousedown={prevent} onclick={onItalic}>
		<em>I</em>
	</button>
	<button class="tb-btn" title={$t('toolbar.strikethrough')} onmousedown={prevent} onclick={onStrikethrough}>
		<s>S</s>
	</button>
	<button class="tb-btn" title={$t('toolbar.highlight')} onmousedown={prevent} onclick={onHighlight}>
		<span class="highlight-icon">H</span>
	</button>
	<button class="tb-btn mono" title={$t('toolbar.code')} onmousedown={prevent} onclick={onCode}>
		&lt;/&gt;
	</button>
	<button class="tb-btn" title={$t('toolbar.link')} onmousedown={prevent} onclick={onLink}>
		<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round">
			<path d="M10 13a5 5 0 0 0 7.54.54l3-3a5 5 0 0 0-7.07-7.07l-1.72 1.71" />
			<path d="M14 11a5 5 0 0 0-7.54-.54l-3 3a5 5 0 0 0 7.07 7.07l1.71-1.71" />
		</svg>
	</button>
	<div class="tb-separator"></div>
	<div class="heading-wrapper">
		<button
			class="tb-btn"
			title={$t('toolbar.heading')}
			onmousedown={prevent}
			onclick={() => showHeadingMenu = !showHeadingMenu}
		>
			H<span class="caret">▾</span>
		</button>
		{#if showHeadingMenu}
			<div class="heading-dropdown" onmousedown={prevent}>
				{#each [1,2,3,4,5,6] as level}
					<button class="heading-option" onclick={() => handleHeading(level)}>
						<span class="heading-label" style="font-size: {1.3 - level * 0.1}em; font-weight: 700;">H{level}</span>
					</button>
				{/each}
			</div>
		{/if}
	</div>
	{#if showConvertToTable && onConvertToTable}
		<div class="tb-separator"></div>
		<button class="tb-btn" title={$t('toolbar.convertToTable')} onmousedown={prevent} onclick={onConvertToTable}>
			<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
				<rect x="3" y="3" width="18" height="18" rx="2" ry="2"/>
				<line x1="3" y1="9" x2="21" y2="9"/>
				<line x1="3" y1="15" x2="21" y2="15"/>
				<line x1="9" y1="3" x2="9" y2="21"/>
				<line x1="15" y1="3" x2="15" y2="21"/>
			</svg>
		</button>
	{/if}
</div>

<style>
	.formatting-toolbar {
		position: absolute;
		z-index: 100;
		display: flex;
		align-items: center;
		gap: 1px;
		background: var(--background-primary);
		border: 1px solid var(--background-modifier-border);
		border-radius: 8px;
		box-shadow: var(--popover-shadow, 0 4px 16px rgba(0,0,0,0.18));
		padding: 3px 4px;
		transform: translateX(-50%);
		pointer-events: auto;
	}

	.tb-btn {
		display: flex;
		align-items: center;
		justify-content: center;
		width: 28px;
		height: 28px;
		border: none;
		border-radius: 5px;
		background: transparent;
		color: var(--text-normal);
		cursor: pointer;
		font-size: 0.82rem;
		font-weight: 600;
		padding: 0;
	}
	.tb-btn:hover {
		background: var(--background-modifier-hover);
	}
	.tb-btn.mono {
		font-family: var(--font-monospace-theme);
		font-size: 0.72rem;
		font-weight: 700;
	}

	.highlight-icon {
		background: color-mix(in srgb, var(--color-yellow) 40%, transparent);
		border-radius: 2px;
		padding: 0 3px;
	}

	.tb-separator {
		width: 1px;
		height: 18px;
		background: var(--background-modifier-border);
		margin: 0 2px;
	}

	.heading-wrapper {
		position: relative;
	}
	.caret {
		font-size: 0.6em;
		margin-left: 1px;
		opacity: 0.6;
	}

	.heading-dropdown {
		position: absolute;
		top: 100%;
		left: 50%;
		transform: translateX(-50%);
		margin-top: 4px;
		background: var(--background-primary);
		border: 1px solid var(--background-modifier-border);
		border-radius: 6px;
		box-shadow: var(--popover-shadow, 0 4px 12px rgba(0,0,0,0.15));
		padding: 4px;
		display: flex;
		flex-direction: column;
		gap: 1px;
		z-index: 110;
	}

	.heading-option {
		display: flex;
		align-items: center;
		padding: 4px 12px;
		border: none;
		border-radius: 4px;
		background: transparent;
		color: var(--text-normal);
		cursor: pointer;
		white-space: nowrap;
	}
	.heading-option:hover {
		background: var(--background-modifier-hover);
	}
</style>
