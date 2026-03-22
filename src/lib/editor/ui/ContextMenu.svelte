<script lang="ts">
	import { onMount, onDestroy } from 'svelte';
	import { t } from '$lib/i18n';

	let {
		x = 0,
		y = 0,
		onaction,
		onclose,
		dir = 'ltr' as 'ltr' | 'rtl' | 'auto',
	}: {
		x?: number;
		y?: number;
		onaction?: (event: CustomEvent) => void;
		onclose?: () => void;
		dir?: 'ltr' | 'rtl' | 'auto';
	} = $props();

	let menuEl: HTMLDivElement;

	interface MenuItem {
		label: string;
		action: string;
		shortcut?: string;
		data?: Record<string, any>;
		separator?: boolean;
	}

	const items: MenuItem[] = [
		{ label: 'editor.cut', action: 'cut', shortcut: 'Ctrl+X' },
		{ label: 'editor.copy', action: 'copy', shortcut: 'Ctrl+C' },
		{ label: 'editor.paste', action: 'paste', shortcut: 'Ctrl+V' },
		{ label: 'editor.selectAll', action: 'select-all', shortcut: 'Ctrl+A' },
		{ label: '', action: '', separator: true },
		{ label: 'editor.bold', action: 'bold', shortcut: 'Ctrl+B' },
		{ label: 'editor.italic', action: 'italic', shortcut: 'Ctrl+I' },
		{ label: 'editor.underline', action: 'underline', shortcut: 'Ctrl+U' },
		{ label: 'editor.strikethrough', action: 'strikethrough', shortcut: 'Ctrl+Shift+S' },
		{ label: 'editor.highlight', action: 'highlight', shortcut: 'Ctrl+Shift+H' },
		{ label: 'editor.inlineCode', action: 'code', shortcut: 'Ctrl+E' },
		{ label: '', action: '', separator: true },
		{ label: 'editor.paragraph', action: 'paragraph' },
		{ label: 'editor.heading', action: 'h1', data: { label: '1' } },
		{ label: 'editor.heading', action: 'h2', data: { label: '2' } },
		{ label: 'editor.heading', action: 'h3', data: { label: '3' } },
		{ label: '', action: '', separator: true },
		{ label: 'editor.bulletList', action: 'bullet-list' },
		{ label: 'editor.orderedList', action: 'ordered-list' },
		{ label: 'editor.taskList', action: 'task-list' },
		{ label: 'editor.blockquote', action: 'blockquote' },
		{ label: 'editor.codeBlock', action: 'code-block' },
		{ label: 'editor.callout', action: 'callout' },
		{ label: 'editor.horizontalRule', action: 'hr' },
		{ label: '', action: '', separator: true },
		{ label: 'editor.link', action: 'link', shortcut: 'Ctrl+K' },
		{ label: 'editor.image', action: 'image' },
	];

	function dispatch(item: MenuItem) {
		if (onaction) {
			onaction(new CustomEvent('action', { detail: { action: item.action, data: item.data } }));
		}
	}

	function handleClickOutside(e: MouseEvent) {
		if (menuEl && !menuEl.contains(e.target as Node)) {
			if (onclose) onclose();
		}
	}

	function handleKeyDown(e: KeyboardEvent) {
		if (e.key === 'Escape') {
			if (onclose) onclose();
		}
	}

	onMount(() => {
		// Position adjustment to stay within viewport
		requestAnimationFrame(() => {
			if (!menuEl) return;
			const rect = menuEl.getBoundingClientRect();
			if (rect.right > window.innerWidth) {
				menuEl.style.left = (x - rect.width) + 'px';
			}
			if (rect.bottom > window.innerHeight) {
				menuEl.style.top = (y - rect.height) + 'px';
			}
		});

		document.addEventListener('mousedown', handleClickOutside);
		document.addEventListener('keydown', handleKeyDown);
	});

	onDestroy(() => {
		document.removeEventListener('mousedown', handleClickOutside);
		document.removeEventListener('keydown', handleKeyDown);
	});
</script>

<div
	class="ce-context-menu"
	bind:this={menuEl}
	style="left: {x}px; top: {y}px;"
	role="menu"
>
	{#each items as item}
		{#if item.separator}
			<div class="ce-context-sep"></div>
		{:else}
			<button
				class="ce-context-item"
				role="menuitem"
				onclick={() => dispatch(item)}
			>
				<span class="ce-context-label">
					{$t(item.label)}{item.data?.label ? ` ${item.data.label}` : ''}
				</span>
				{#if item.shortcut}
					<span class="ce-context-shortcut">{item.shortcut}</span>
				{/if}
			</button>
		{/if}
	{/each}
</div>

<style>
	.ce-context-menu {
		position: fixed;
		z-index: 10000;
		background: var(--bg-primary, #fff);
		border: 1px solid var(--border-color, #e0e0e4);
		border-radius: 8px;
		box-shadow: 0 8px 24px rgba(0, 0, 0, 0.15);
		padding: 4px 0;
		min-width: 200px;
		max-height: 80vh;
		overflow-y: auto;
		animation: ce-menu-fade 0.12s ease-out;
		font-family: var(--font-interface, -apple-system, BlinkMacSystemFont, 'Segoe UI', Inter, sans-serif);
		font-size: 13px;
	}

	@keyframes ce-menu-fade {
		from { opacity: 0; transform: scale(0.97); }
		to { opacity: 1; transform: scale(1); }
	}

	.ce-context-item {
		display: flex;
		align-items: center;
		justify-content: space-between;
		width: 100%;
		padding: 6px 12px;
		border: none;
		background: transparent;
		cursor: pointer;
		color: var(--text-primary, #1f2328);
		text-align: start;
		gap: 16px;
	}

	.ce-context-item:hover {
		background: var(--accent-bg, #ede9fe);
		color: var(--accent-color, #7c3aed);
	}

	.ce-context-label {
		white-space: nowrap;
	}

	.ce-context-shortcut {
		color: var(--text-muted, #8b8b8b);
		font-size: 11px;
		white-space: nowrap;
	}

	.ce-context-sep {
		height: 1px;
		background: var(--border-color, #e0e0e4);
		margin: 4px 8px;
	}
</style>
