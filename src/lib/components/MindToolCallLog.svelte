<script lang="ts">
	import { t } from '$lib/i18n';

	let {
		name,
		args,
		status = 'pending',
	}: {
		name: string;
		args: any;
		status?: 'pending' | 'resolved' | 'error';
	} = $props();

	let expanded = $state(false);

	const statusGlyph = $derived.by(() => {
		switch (status) {
			case 'pending':
				return '⏳';
			case 'resolved':
				return '✓';
			case 'error':
				return '⚠';
			default:
				return '·';
		}
	});

	const argsPreview = $derived.by(() => {
		try {
			const s = JSON.stringify(args ?? {});
			return s.length > 60 ? s.slice(0, 57) + '…' : s;
		} catch {
			return '{…}';
		}
	});

	const argsFull = $derived.by(() => {
		try {
			return JSON.stringify(args ?? {}, null, 2);
		} catch {
			return String(args);
		}
	});
</script>

<div class="tool-call-log" class:expanded class:err={status === 'error'}>
	<button class="header" onclick={() => (expanded = !expanded)}>
		<span class="glyph">{statusGlyph}</span>
		<span class="name">{$t('mind.chat.toolCallLabel') || 'Tool'}: <code>{name}</code></span>
		{#if !expanded}
			<span class="preview">{argsPreview}</span>
		{/if}
		<span class="chevron">{expanded ? '▾' : '▸'}</span>
	</button>
	{#if expanded}
		<pre class="args">{argsFull}</pre>
	{/if}
</div>

<style>
	.tool-call-log {
		margin: 0.4rem 0;
		border: 1px solid var(--border-color, rgba(0, 0, 0, 0.1));
		border-radius: 6px;
		background: var(--bg-soft, rgba(0, 0, 0, 0.02));
		font-size: 0.85em;
	}
	.tool-call-log.err {
		border-color: var(--err-color, #c0392b);
		background: rgba(192, 57, 43, 0.06);
	}
	.header {
		display: flex;
		align-items: center;
		gap: 0.5rem;
		padding: 0.35rem 0.6rem;
		width: 100%;
		background: transparent;
		border: none;
		cursor: pointer;
		font-family: inherit;
		color: inherit;
		text-align: left;
	}
	.header:hover {
		background: var(--bg-hover, rgba(0, 0, 0, 0.04));
	}
	.glyph {
		flex-shrink: 0;
	}
	.name {
		flex-shrink: 0;
	}
	code {
		font-size: 0.95em;
		color: var(--accent-color, #7c3aed);
	}
	.preview {
		flex: 1;
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
		opacity: 0.75;
		font-family: ui-monospace, monospace;
	}
	.chevron {
		flex-shrink: 0;
		opacity: 0.6;
	}
	.args {
		margin: 0;
		padding: 0.5rem 0.75rem;
		border-top: 1px solid var(--border-color, rgba(0, 0, 0, 0.08));
		font-family: ui-monospace, monospace;
		font-size: 0.9em;
		white-space: pre-wrap;
		word-break: break-word;
		max-height: 240px;
		overflow: auto;
	}
</style>
