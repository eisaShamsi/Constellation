<script lang="ts">
	import { t } from '$lib/i18n';

	let {
		el = $bindable<HTMLElement | null>(null),
		onaction,
		wordCount = 0,
		dir = 'ltr' as 'ltr' | 'rtl' | 'auto',
	}: {
		el?: HTMLElement | null;
		onaction?: (event: CustomEvent) => void;
		wordCount?: number;
		dir?: 'ltr' | 'rtl' | 'auto';
	} = $props();

	function dispatch(action: string, data?: Record<string, any>) {
		if (onaction) {
			onaction(new CustomEvent('action', { detail: { action, data } }));
		}
	}

	function handleHeadingSelect(e: Event) {
		const select = e.target as HTMLSelectElement;
		const val = select.value;
		if (val === 'p') dispatch('paragraph');
		else dispatch(val);
		select.value = '';
	}
</script>

<!-- svelte-ignore a11y_no_static_element_interactions -->
<div
	class="ce-toolbar"
	bind:this={el}
	dir={dir}
	onmousedown={(e) => e.preventDefault()}
>
	<!-- Undo/Redo -->
	<div class="ce-toolbar-group">
		<button class="ce-toolbar-btn" data-action="undo" title={$t('editor.undo')} onclick={() => dispatch('undo')}>
			<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M3 10h10a5 5 0 015 5v0a5 5 0 01-5 5H3"/><path d="M7 6l-4 4 4 4"/></svg>
		</button>
		<button class="ce-toolbar-btn" data-action="redo" title={$t('editor.redo')} onclick={() => dispatch('redo')}>
			<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M21 10H11a5 5 0 00-5 5v0a5 5 0 005 5h10"/><path d="M17 6l4 4-4 4"/></svg>
		</button>
	</div>

	<div class="ce-toolbar-sep"></div>

	<!-- Heading dropdown -->
	<div class="ce-toolbar-group">
		<select class="ce-toolbar-select" title={$t('editor.heading')} onchange={handleHeadingSelect}>
			<option value="" disabled selected>{$t('editor.heading')}</option>
			<option value="p">{$t('editor.paragraph')}</option>
			<option value="h1">{$t('editor.heading')} 1</option>
			<option value="h2">{$t('editor.heading')} 2</option>
			<option value="h3">{$t('editor.heading')} 3</option>
			<option value="h4">{$t('editor.heading')} 4</option>
			<option value="h5">{$t('editor.heading')} 5</option>
			<option value="h6">{$t('editor.heading')} 6</option>
		</select>
	</div>

	<div class="ce-toolbar-sep"></div>

	<!-- Inline formatting -->
	<div class="ce-toolbar-group">
		<button class="ce-toolbar-btn" data-action="bold" title="{$t('editor.bold')} (Ctrl+B)" onclick={() => dispatch('bold')}>
			<strong>B</strong>
		</button>
		<button class="ce-toolbar-btn" data-action="italic" title="{$t('editor.italic')} (Ctrl+I)" onclick={() => dispatch('italic')}>
			<em>I</em>
		</button>
		<button class="ce-toolbar-btn" data-action="underline" title="{$t('editor.underline')} (Ctrl+U)" onclick={() => dispatch('underline')}>
			<u>U</u>
		</button>
		<button class="ce-toolbar-btn" data-action="strikethrough" title="{$t('editor.strikethrough')} (Ctrl+Shift+S)" onclick={() => dispatch('strikethrough')}>
			<s>S</s>
		</button>
		<button class="ce-toolbar-btn" data-action="highlight" title="{$t('editor.highlight')} (Ctrl+Shift+H)" onclick={() => dispatch('highlight')}>
			<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M12 20h9"/><path d="M16.5 3.5l-10 10L3 21l7.5-3.5 10-10a2.828 2.828 0 00-4-4z"/></svg>
		</button>
		<button class="ce-toolbar-btn" data-action="code" title="{$t('editor.inlineCode')} (Ctrl+E)" onclick={() => dispatch('code')}>
			<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><polyline points="16 18 22 12 16 6"/><polyline points="8 6 2 12 8 18"/></svg>
		</button>
	</div>

	<div class="ce-toolbar-sep"></div>

	<!-- Alignment (future) & Lists -->
	<div class="ce-toolbar-group">
		<button class="ce-toolbar-btn" data-action="bullet-list" title="{$t('editor.bulletList')} (Ctrl+Shift+B)" onclick={() => dispatch('bullet-list')}>
			<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><line x1="8" y1="6" x2="21" y2="6"/><line x1="8" y1="12" x2="21" y2="12"/><line x1="8" y1="18" x2="21" y2="18"/><circle cx="3" cy="6" r="1" fill="currentColor"/><circle cx="3" cy="12" r="1" fill="currentColor"/><circle cx="3" cy="18" r="1" fill="currentColor"/></svg>
		</button>
		<button class="ce-toolbar-btn" data-action="ordered-list" title="{$t('editor.orderedList')} (Ctrl+Shift+O)" onclick={() => dispatch('ordered-list')}>
			<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><line x1="10" y1="6" x2="21" y2="6"/><line x1="10" y1="12" x2="21" y2="12"/><line x1="10" y1="18" x2="21" y2="18"/><text x="2" y="8" font-size="8" fill="currentColor" font-family="sans-serif">1</text><text x="2" y="14" font-size="8" fill="currentColor" font-family="sans-serif">2</text><text x="2" y="20" font-size="8" fill="currentColor" font-family="sans-serif">3</text></svg>
		</button>
		<button class="ce-toolbar-btn" data-action="task-list" title="{$t('editor.taskList')} (Ctrl+Shift+T)" onclick={() => dispatch('task-list')}>
			<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><rect x="3" y="5" width="6" height="6" rx="1"/><path d="M5 8l1.5 1.5L9 7"/><line x1="13" y1="8" x2="21" y2="8"/><rect x="3" y="14" width="6" height="6" rx="1"/><line x1="13" y1="17" x2="21" y2="17"/></svg>
		</button>
	</div>

	<div class="ce-toolbar-sep"></div>

	<!-- Indent/Outdent -->
	<div class="ce-toolbar-group">
		<button class="ce-toolbar-btn" data-action="indent" title="{$t('editor.indent')} (Tab)" onclick={() => dispatch('indent')}>
			<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><line x1="3" y1="4" x2="21" y2="4"/><line x1="13" y1="9" x2="21" y2="9"/><line x1="13" y1="14" x2="21" y2="14"/><line x1="3" y1="19" x2="21" y2="19"/><polyline points="3 8 7 11.5 3 15"/></svg>
		</button>
		<button class="ce-toolbar-btn" data-action="outdent" title="{$t('editor.outdent')} (Shift+Tab)" onclick={() => dispatch('outdent')}>
			<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><line x1="3" y1="4" x2="21" y2="4"/><line x1="13" y1="9" x2="21" y2="9"/><line x1="13" y1="14" x2="21" y2="14"/><line x1="3" y1="19" x2="21" y2="19"/><polyline points="7 8 3 11.5 7 15"/></svg>
		</button>
	</div>

	<div class="ce-toolbar-sep"></div>

	<!-- Insert elements -->
	<div class="ce-toolbar-group">
		<button class="ce-toolbar-btn" data-action="link" title="{$t('editor.link')} (Ctrl+K)" onclick={() => dispatch('link')}>
			<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M10 13a5 5 0 007.54.54l3-3a5 5 0 00-7.07-7.07l-1.72 1.71"/><path d="M14 11a5 5 0 00-7.54-.54l-3 3a5 5 0 007.07 7.07l1.71-1.71"/></svg>
		</button>
		<button class="ce-toolbar-btn" data-action="image" title="{$t('editor.image')} (Ctrl+Shift+I)" onclick={() => dispatch('image')}>
			<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><rect x="3" y="3" width="18" height="18" rx="2"/><circle cx="8.5" cy="8.5" r="1.5"/><polyline points="21 15 16 10 5 21"/></svg>
		</button>
		<button class="ce-toolbar-btn" data-action="table" title={$t('editor.table')} onclick={() => dispatch('table')}>
			<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><rect x="3" y="3" width="18" height="18" rx="2"/><line x1="3" y1="9" x2="21" y2="9"/><line x1="3" y1="15" x2="21" y2="15"/><line x1="9" y1="3" x2="9" y2="21"/><line x1="15" y1="3" x2="15" y2="21"/></svg>
		</button>
		<button class="ce-toolbar-btn" data-action="code-block" title="{$t('editor.codeBlock')} (Ctrl+Shift+C)" onclick={() => dispatch('code-block')}>
			<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><rect x="3" y="3" width="18" height="18" rx="2"/><polyline points="9 8 6 12 9 16"/><polyline points="15 8 18 12 15 16"/></svg>
		</button>
		<button class="ce-toolbar-btn" data-action="blockquote" title="{$t('editor.blockquote')} (Ctrl+Shift+Q)" onclick={() => dispatch('blockquote')}>
			<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M3 21c3-7 4-12 4-16h4c-1 5-2 10-5 16z"/><path d="M13 21c3-7 4-12 4-16h4c-1 5-2 10-5 16z"/></svg>
		</button>
		<button class="ce-toolbar-btn" data-action="callout" title={$t('editor.callout')} onclick={() => dispatch('callout')}>
			<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="12" cy="12" r="10"/><line x1="12" y1="8" x2="12" y2="12"/><line x1="12" y1="16" x2="12.01" y2="16"/></svg>
		</button>
		<button class="ce-toolbar-btn" data-action="hr" title={$t('editor.horizontalRule')} onclick={() => dispatch('hr')}>
			<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><line x1="3" y1="12" x2="21" y2="12"/></svg>
		</button>
	</div>

	<div class="ce-toolbar-sep"></div>

	<!-- Sub/Sup -->
	<div class="ce-toolbar-group">
		<button class="ce-toolbar-btn" data-action="subscript" title={$t('editor.subscript')} onclick={() => dispatch('subscript')}>
			X<sub>2</sub>
		</button>
		<button class="ce-toolbar-btn" data-action="superscript" title={$t('editor.superscript')} onclick={() => dispatch('superscript')}>
			X<sup>2</sup>
		</button>
	</div>

	<!-- Word count -->
	<div class="ce-toolbar-spacer"></div>
	<div class="ce-toolbar-info">
		{wordCount} {$t('editor.words')}
	</div>
</div>

<style>
	.ce-toolbar {
		display: flex;
		align-items: center;
		gap: 2px;
		padding: 4px 8px;
		background: var(--bg-secondary, #f6f6f9);
		border-bottom: 1px solid var(--border-color, #e0e0e4);
		flex-shrink: 0;
		overflow-x: auto;
		user-select: none;
		font-family: var(--font-interface, -apple-system, BlinkMacSystemFont, 'Segoe UI', Inter, sans-serif);
		font-size: 13px;
	}

	.ce-toolbar[dir="rtl"] {
		flex-direction: row-reverse;
	}

	.ce-toolbar-group {
		display: flex;
		align-items: center;
		gap: 1px;
	}

	.ce-toolbar-btn {
		display: flex;
		align-items: center;
		justify-content: center;
		width: 28px;
		height: 28px;
		border: none;
		background: transparent;
		border-radius: 4px;
		cursor: pointer;
		color: var(--text-secondary, #5c5c66);
		transition: background-color 0.1s, color 0.1s;
		padding: 0;
		font-size: 14px;
	}

	.ce-toolbar-btn:hover {
		background: var(--bg-hover, #e8e8ec);
		color: var(--text-primary, #1f2328);
	}

	:global(.ce-toolbar-btn.active) {
		background: var(--accent-bg, #ede9fe);
		color: var(--accent-color, #7c3aed);
	}

	.ce-toolbar-sep {
		width: 1px;
		height: 20px;
		background: var(--border-color, #e0e0e4);
		margin: 0 4px;
		flex-shrink: 0;
	}

	.ce-toolbar-select {
		height: 28px;
		border: 1px solid var(--border-color, #e0e0e4);
		border-radius: 4px;
		background: transparent;
		color: var(--text-secondary, #5c5c66);
		font-size: 12px;
		padding: 0 6px;
		cursor: pointer;
		min-width: 80px;
	}

	.ce-toolbar-spacer {
		flex: 1;
	}

	.ce-toolbar-info {
		color: var(--text-muted, #8b8b8b);
		font-size: 12px;
		white-space: nowrap;
		padding: 0 8px;
	}
</style>
