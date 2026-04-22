<script lang="ts">
	import { onMount } from 'svelte';
	import { t, dir } from '$lib/i18n';

	type CursorContext = 'normal' | 'heading' | 'table' | 'checkbox' | 'link' | 'codeblock' | 'list' | 'blockquote';

	let {
		x,
		y,
		onClose,
		hasSelection = false,
		cursorContext = 'normal' as CursorContext,
		currentHeadingLevel = null as number | null,
		onFormat,
		onInsert,
		onHeading,
		onList,
		onClipboard,
		onTableAction,
		onLinkAction,
	}: {
		x: number;
		y: number;
		onClose: () => void;
		hasSelection: boolean;
		cursorContext: CursorContext;
		currentHeadingLevel: number | null;
		onFormat: (type: string) => void;
		onInsert: (type: string) => void;
		onHeading: (level: number) => void;
		onList: (type: string) => void;
		onClipboard: (action: string) => void;
		onTableAction?: (action: string) => void;
		onLinkAction?: (action: string) => void;
	} = $props();

	let menuEl: HTMLDivElement;
	let showHeadingSub = $state(false);

	onMount(() => {
		function handleClickOutside(e: MouseEvent) {
			if (menuEl && !menuEl.contains(e.target as Node)) onClose();
		}
		function handleEscape(e: KeyboardEvent) {
			if (e.key === 'Escape') onClose();
		}
		let openTimer: ReturnType<typeof setTimeout> | null = setTimeout(() => {
			openTimer = null;
			document.addEventListener('click', handleClickOutside);
			document.addEventListener('contextmenu', handleClickOutside);
			document.addEventListener('keydown', handleEscape);
		}, 10);
		return () => {
			if (openTimer !== null) clearTimeout(openTimer);
			document.removeEventListener('click', handleClickOutside);
			document.removeEventListener('contextmenu', handleClickOutside);
			document.removeEventListener('keydown', handleEscape);
		};
	});

	const adjustedX = $derived(Math.min(x, window.innerWidth - 240));
	const adjustedY = $derived(Math.min(y, window.innerHeight - 420));

	function act(fn: () => void) {
		fn();
		onClose();
	}
</script>

<!-- svelte-ignore a11y_no_static_element_interactions -->
<div class="ecm" bind:this={menuEl} style="left: {adjustedX}px; top: {adjustedY}px;" dir={$dir}>

	<!-- Format section (only when text selected) -->
	{#if hasSelection}
		<button class="ecm-item" onclick={() => act(() => onFormat('bold'))}>
			<span class="ecm-label">{$t('contextMenu.bold')}</span>
			<span class="ecm-shortcut">Ctrl+B</span>
		</button>
		<button class="ecm-item" onclick={() => act(() => onFormat('italic'))}>
			<span class="ecm-label">{$t('contextMenu.italic')}</span>
			<span class="ecm-shortcut">Ctrl+I</span>
		</button>
		<button class="ecm-item" onclick={() => act(() => onFormat('strikethrough'))}>
			<span class="ecm-label">{$t('contextMenu.strikethrough')}</span>
			<span class="ecm-shortcut">Ctrl+Shift+S</span>
		</button>
		<button class="ecm-item" onclick={() => act(() => onFormat('underline'))}>
			<span class="ecm-label">{$t('contextMenu.underline')}</span>
			<span class="ecm-shortcut">Ctrl+U</span>
		</button>
		<button class="ecm-item" onclick={() => act(() => onFormat('highlight'))}>
			<span class="ecm-label">{$t('contextMenu.highlight')}</span>
			<span class="ecm-shortcut">Ctrl+Shift+H</span>
		</button>
		<button class="ecm-item" onclick={() => act(() => onFormat('code'))}>
			<span class="ecm-label">{$t('contextMenu.inlineCode')}</span>
			<span class="ecm-shortcut">Ctrl+`</span>
		</button>
		<button class="ecm-item" onclick={() => act(() => onFormat('subscript'))}>
			<span class="ecm-label">{$t('contextMenu.subscript') || 'Subscript'}</span>
		</button>
		<button class="ecm-item" onclick={() => act(() => onFormat('superscript'))}>
			<span class="ecm-label">{$t('contextMenu.superscript') || 'Superscript'}</span>
		</button>
		<button class="ecm-item" onclick={() => act(() => onFormat('clear'))}>
			<span class="ecm-label">{$t('contextMenu.clearFormatting')}</span>
		</button>
		<div class="ecm-sep"></div>
	{/if}

	<!-- Context-specific: Table -->
	{#if cursorContext === 'table' && onTableAction}
		<button class="ecm-item" onclick={() => act(() => onTableAction!('addRow'))}>
			<span class="ecm-label">{$t('contextMenu.addRow')}</span>
		</button>
		<button class="ecm-item" onclick={() => act(() => onTableAction!('addColumn'))}>
			<span class="ecm-label">{$t('contextMenu.addColumn')}</span>
		</button>
		<button class="ecm-item" onclick={() => act(() => onTableAction!('deleteRow'))}>
			<span class="ecm-label">{$t('contextMenu.deleteRow')}</span>
		</button>
		<button class="ecm-item" onclick={() => act(() => onTableAction!('deleteColumn'))}>
			<span class="ecm-label">{$t('contextMenu.deleteColumn')}</span>
		</button>
		<button class="ecm-item" onclick={() => act(() => onTableAction!('sortAsc'))}>
			<span class="ecm-label">{$t('contextMenu.sortAscending')}</span>
		</button>
		<button class="ecm-item" onclick={() => act(() => onTableAction!('sortDesc'))}>
			<span class="ecm-label">{$t('contextMenu.sortDescending')}</span>
		</button>
		<div class="ecm-sep"></div>
	{/if}

	<!-- Context-specific: Checkbox -->
	{#if cursorContext === 'checkbox'}
		<button class="ecm-item" onclick={() => act(() => onFormat('toggleCheckbox'))}>
			<span class="ecm-label">{$t('contextMenu.toggleCheckbox')}</span>
			<span class="ecm-shortcut">Ctrl+Enter</span>
		</button>
		<div class="ecm-sep"></div>
	{/if}

	<!-- Context-specific: Link -->
	{#if cursorContext === 'link' && onLinkAction}
		<button class="ecm-item" onclick={() => act(() => onLinkAction!('edit'))}>
			<span class="ecm-label">{$t('contextMenu.editLink')}</span>
		</button>
		<button class="ecm-item" onclick={() => act(() => onLinkAction!('open'))}>
			<span class="ecm-label">{$t('contextMenu.openLink')}</span>
		</button>
		<button class="ecm-item" onclick={() => act(() => onLinkAction!('remove'))}>
			<span class="ecm-label">{$t('contextMenu.removeLink')}</span>
		</button>
		<div class="ecm-sep"></div>
	{/if}

	<!-- Heading submenu -->
	<div class="ecm-submenu-wrap"
		onmouseenter={() => showHeadingSub = true}
		onmouseleave={() => showHeadingSub = false}
	>
		<button class="ecm-item ecm-has-sub">
			<span class="ecm-label">{$t('contextMenu.heading')}</span>
			<span class="ecm-arrow">{$dir === 'rtl' ? '◂' : '▸'}</span>
		</button>
		{#if showHeadingSub}
			<div class="ecm-sub" class:rtl={$dir === 'rtl'}>
				{#each [1,2,3,4,5,6] as level}
					<button
						class="ecm-item"
						class:ecm-active={currentHeadingLevel === level}
						onclick={() => act(() => onHeading(level))}
					>
						<span class="ecm-label" style="font-size: {1.2 - level * 0.08}em; font-weight: 700;">H{level}</span>
					</button>
				{/each}
				<div class="ecm-sep"></div>
				<button class="ecm-item" onclick={() => act(() => onHeading(0))}>
					<span class="ecm-label">{$t('contextMenu.paragraph')}</span>
				</button>
			</div>
		{/if}
	</div>

	<!-- Insert section -->
	<button class="ecm-item" onclick={() => act(() => onInsert('link'))}>
		<span class="ecm-label">{$t('contextMenu.link')}</span>
		<span class="ecm-shortcut">Ctrl+K</span>
	</button>
	<button class="ecm-item" onclick={() => act(() => onInsert('image'))}>
		<span class="ecm-label">{$t('contextMenu.image')}</span>
	</button>
	<button class="ecm-item" onclick={() => act(() => onInsert('table'))}>
		<span class="ecm-label">{$t('contextMenu.table')}</span>
	</button>
	<button class="ecm-item" onclick={() => act(() => onInsert('horizontalRule'))}>
		<span class="ecm-label">{$t('contextMenu.horizontalRule')}</span>
	</button>
	<button class="ecm-item" onclick={() => act(() => onInsert('codeBlock'))}>
		<span class="ecm-label">{$t('contextMenu.codeBlock')}</span>
	</button>
	<button class="ecm-item" onclick={() => act(() => onInsert('callout'))}>
		<span class="ecm-label">{$t('contextMenu.callout')}</span>
	</button>
	<button class="ecm-item" onclick={() => act(() => onInsert('mathBlock'))}>
		<span class="ecm-label">{$t('contextMenu.mathBlock')}</span>
	</button>

	<div class="ecm-sep"></div>

	<!-- Lists -->
	<button class="ecm-item" onclick={() => act(() => onList('bullet'))}>
		<span class="ecm-label">{$t('contextMenu.bulletList')}</span>
	</button>
	<button class="ecm-item" onclick={() => act(() => onList('numbered'))}>
		<span class="ecm-label">{$t('contextMenu.numberedList')}</span>
	</button>
	<button class="ecm-item" onclick={() => act(() => onList('task'))}>
		<span class="ecm-label">{$t('contextMenu.taskList')}</span>
	</button>

	<div class="ecm-sep"></div>

	<!-- Block -->
	<button class="ecm-item" onclick={() => act(() => onInsert('blockquote'))}>
		<span class="ecm-label">{$t('contextMenu.blockquote')}</span>
	</button>
	<button class="ecm-item" onclick={() => act(() => onFormat('toggleComment'))}>
		<span class="ecm-label">{$t('contextMenu.toggleComment')}</span>
		<span class="ecm-shortcut">Ctrl+/</span>
	</button>

	<div class="ecm-sep"></div>

	<!-- Clipboard -->
	<button class="ecm-item" onclick={() => act(() => onClipboard('cut'))}>
		<span class="ecm-label">{$t('contextMenu.cut')}</span>
		<span class="ecm-shortcut">Ctrl+X</span>
	</button>
	<button class="ecm-item" onclick={() => act(() => onClipboard('copy'))}>
		<span class="ecm-label">{$t('contextMenu.copy')}</span>
		<span class="ecm-shortcut">Ctrl+C</span>
	</button>
	<button class="ecm-item" onclick={() => act(() => onClipboard('paste'))}>
		<span class="ecm-label">{$t('contextMenu.paste')}</span>
		<span class="ecm-shortcut">Ctrl+V</span>
	</button>
	<button class="ecm-item" onclick={() => act(() => onClipboard('pastePlain'))}>
		<span class="ecm-label">{$t('contextMenu.pasteAsPlainText')}</span>
		<span class="ecm-shortcut">Ctrl+Shift+V</span>
	</button>
</div>

<style>
	.ecm {
		position: fixed;
		z-index: 1100;
		min-width: 220px;
		max-height: 80vh;
		overflow-y: auto;
		background: var(--background-primary);
		border: 1px solid var(--background-modifier-border);
		border-radius: 8px;
		box-shadow: 0 6px 24px rgba(0, 0, 0, 0.2);
		padding: 4px;
		display: flex;
		flex-direction: column;
		font-family: var(--font-interface-theme);
	}
	.ecm-item {
		display: flex;
		align-items: center;
		gap: 8px;
		width: 100%;
		padding: 6px 12px;
		border: none;
		background: none;
		font-size: 0.82rem;
		font-family: inherit;
		color: var(--text-normal);
		cursor: pointer;
		border-radius: 4px;
		text-align: start;
		white-space: nowrap;
	}
	.ecm-item:hover {
		background: var(--background-modifier-hover);
	}
	.ecm-item.ecm-active {
		color: var(--interactive-accent);
		font-weight: 600;
	}
	.ecm-label {
		flex: 1;
	}
	.ecm-shortcut {
		font-size: 0.72rem;
		color: var(--text-muted);
		opacity: 0.7;
		font-family: var(--font-monospace-theme);
	}
	.ecm-arrow {
		font-size: 0.65rem;
		color: var(--text-muted);
	}
	.ecm-sep {
		height: 1px;
		background: var(--background-modifier-border);
		margin: 3px 8px;
	}
	.ecm-submenu-wrap {
		position: relative;
	}
	.ecm-has-sub {
		display: flex;
		align-items: center;
	}
	.ecm-sub {
		position: absolute;
		top: -4px;
		left: 100%;
		min-width: 100px;
		background: var(--background-primary);
		border: 1px solid var(--background-modifier-border);
		border-radius: 6px;
		box-shadow: 0 4px 16px rgba(0, 0, 0, 0.18);
		padding: 4px;
		display: flex;
		flex-direction: column;
		z-index: 1110;
	}
	.ecm-sub.rtl {
		left: auto;
		right: 100%;
	}
</style>
