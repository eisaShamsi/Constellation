<script lang="ts">
	import { detectDir, renderMarkdown } from '$lib/utils';
	import MindCitationChip from './MindCitationChip.svelte';
	import MindToolCallLog from './MindToolCallLog.svelte';

	export type ToolCallEntry = {
		id: string;
		name: string;
		args: any;
		status: 'pending' | 'resolved' | 'error';
	};

	export type ChatMessage = {
		role: 'user' | 'assistant';
		text: string;
		toolCalls?: ToolCallEntry[];
		complete: boolean;
	};

	let { message }: { message: ChatMessage } = $props();

	// Split assistant text on [note:<path>] citations. Returns an
	// alternating array of plain-text chunks and citation paths.
	type Segment = { kind: 'text'; value: string } | { kind: 'citation'; path: string };

	const citationRe = /\[note:([^\]]+)\]/g;

	const segments = $derived.by<Segment[]>(() => {
		if (message.role !== 'assistant') {
			return [{ kind: 'text', value: message.text }];
		}
		const out: Segment[] = [];
		let lastIndex = 0;
		citationRe.lastIndex = 0;
		let m: RegExpExecArray | null;
		while ((m = citationRe.exec(message.text)) !== null) {
			if (m.index > lastIndex) {
				out.push({ kind: 'text', value: message.text.slice(lastIndex, m.index) });
			}
			out.push({ kind: 'citation', path: m[1] });
			lastIndex = m.index + m[0].length;
		}
		if (lastIndex < message.text.length) {
			out.push({ kind: 'text', value: message.text.slice(lastIndex) });
		}
		return out;
	});

	const dir = $derived(detectDir(message.text));
</script>

<div class="msg" class:user={message.role === 'user'} class:assistant={message.role === 'assistant'} {dir}>
	{#if message.role === 'user'}
		<div class="bubble user-bubble">{message.text}</div>
	{:else}
		<div class="bubble assistant-bubble">
			{#if message.toolCalls && message.toolCalls.length > 0}
				{#each message.toolCalls as tc (tc.id)}
					<MindToolCallLog name={tc.name} args={tc.args} status={tc.status} />
				{/each}
			{/if}
			{#each segments as seg, i (i)}
				{#if seg.kind === 'text'}
					{@html renderMarkdown(seg.value)}
				{:else}
					<MindCitationChip path={seg.path} />
				{/if}
			{/each}
			{#if !message.complete}
				<span class="cursor">▍</span>
			{/if}
		</div>
	{/if}
</div>

<style>
	.msg {
		display: flex;
		margin: 0.6rem 0;
	}
	.msg.user {
		justify-content: flex-end;
	}
	.msg.assistant {
		justify-content: flex-start;
	}
	.bubble {
		max-width: 92%;
		padding: 0.55rem 0.85rem;
		border-radius: 0.7rem;
		line-height: 1.5;
		word-wrap: break-word;
		overflow-wrap: break-word;
	}
	.user-bubble {
		background: var(--accent-color, #7c3aed);
		color: white;
		white-space: pre-wrap;
	}
	.assistant-bubble {
		background: var(--bg-soft, rgba(0, 0, 0, 0.04));
		color: inherit;
	}
	.cursor {
		display: inline-block;
		animation: blink 1s steps(2, start) infinite;
		opacity: 0.7;
	}
	@keyframes blink {
		to {
			visibility: hidden;
		}
	}
	/* Render markdown elements compactly inside the bubble */
	:global(.assistant-bubble p) {
		margin: 0.2rem 0;
	}
	:global(.assistant-bubble p:first-child) {
		margin-top: 0;
	}
	:global(.assistant-bubble p:last-child) {
		margin-bottom: 0;
	}
	:global(.assistant-bubble code) {
		background: rgba(0, 0, 0, 0.08);
		padding: 0.1em 0.3em;
		border-radius: 3px;
		font-size: 0.9em;
	}
	:global(.assistant-bubble pre) {
		background: rgba(0, 0, 0, 0.06);
		padding: 0.5rem;
		border-radius: 4px;
		overflow-x: auto;
		margin: 0.4rem 0;
	}
</style>
