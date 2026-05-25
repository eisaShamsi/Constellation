<script lang="ts">
	import { Channel, invoke } from '@tauri-apps/api/core';
	import { onDestroy, tick } from 'svelte';
	import { t } from '$lib/i18n';
	import MindChatMessage from './MindChatMessage.svelte';
	import type { ChatMessage, ToolCallEntry } from './MindChatMessage.svelte';

	type StreamEvent =
		| { type: 'token'; text: string }
		| { type: 'tool_call'; id: string; name: string; args: any }
		| { type: 'done'; finish_reason: string; usage: { input_tokens: number; output_tokens: number } }
		| { type: 'error'; message: string };

	let messages = $state<ChatMessage[]>([]);
	let composer = $state('');
	let inFlight = $state(false);
	let errorBanner = $state<string | null>(null);
	let listEl: HTMLDivElement | null = $state(null);
	let activeChannel: Channel<StreamEvent> | null = null;

	async function scrollToBottom() {
		await tick();
		if (listEl) {
			listEl.scrollTop = listEl.scrollHeight;
		}
	}

	async function send() {
		const text = composer.trim();
		if (!text || inFlight) return;

		composer = '';
		errorBanner = null;

		const userMsg: ChatMessage = { role: 'user', text, complete: true };
		const assistantMsg: ChatMessage = {
			role: 'assistant',
			text: '',
			toolCalls: [],
			complete: false,
		};
		messages = [...messages, userMsg, assistantMsg];
		await scrollToBottom();

		inFlight = true;
		const ch = new Channel<StreamEvent>();
		activeChannel = ch;
		ch.onmessage = (ev: StreamEvent) => {
			// Mutate the most-recent assistant message — we know it's
			// at messages[messages.length - 1] because we just pushed it.
			const i = messages.length - 1;
			const cur = messages[i];
			if (!cur || cur.role !== 'assistant') return;

			if (ev.type === 'token') {
				const updated: ChatMessage = { ...cur, text: cur.text + ev.text };
				messages = [...messages.slice(0, i), updated];
				scrollToBottom();
			} else if (ev.type === 'tool_call') {
				const entry: ToolCallEntry = {
					id: ev.id,
					name: ev.name,
					args: ev.args,
					status: 'pending',
				};
				const updated: ChatMessage = {
					...cur,
					toolCalls: [...(cur.toolCalls ?? []), entry],
				};
				messages = [...messages.slice(0, i), updated];
			} else if (ev.type === 'done') {
				// Mark assistant message complete + flip any pending
				// tool-call entries to "resolved" (we don't get a separate
				// resolved event — the next assistant tokens after a tool
				// call imply the dispatcher returned a result).
				const updated: ChatMessage = {
					...cur,
					complete: true,
					toolCalls: (cur.toolCalls ?? []).map((tc) =>
						tc.status === 'pending' ? { ...tc, status: 'resolved' } : tc,
					),
				};
				messages = [...messages.slice(0, i), updated];
				inFlight = false;
				activeChannel = null;
			} else if (ev.type === 'error') {
				errorBanner = ev.message;
				const updated: ChatMessage = {
					...cur,
					complete: true,
				};
				messages = [...messages.slice(0, i), updated];
				inFlight = false;
				activeChannel = null;
			}
		};

		try {
			await invoke('mind_start_turn', {
				request: { user_message: text, conversation_id: 'default' },
				onEvent: ch,
			});
		} catch (e) {
			errorBanner = String(e);
			inFlight = false;
			activeChannel = null;
			const i = messages.length - 1;
			const cur = messages[i];
			if (cur && cur.role === 'assistant') {
				messages = [...messages.slice(0, i), { ...cur, complete: true }];
			}
		}
	}

	function handleComposerKey(e: KeyboardEvent) {
		// Enter sends; Shift+Enter inserts newline.
		if (e.key === 'Enter' && !e.shiftKey && !e.isComposing) {
			e.preventDefault();
			send();
		}
	}

	function clearConversation() {
		if (inFlight) return;
		messages = [];
		errorBanner = null;
	}

	onDestroy(() => {
		// Channel<T> doesn't expose an explicit close; dropping the
		// reference lets the GC collect when the backend task finishes.
		activeChannel = null;
	});
</script>

<div class="mind-chat-pane">
	<header class="pane-header">
		<div class="title">{$t('mind.chat.paneTitle') || 'Constellation Mind'}</div>
		<button class="clear-btn" onclick={clearConversation} disabled={inFlight || messages.length === 0} title={$t('mind.chat.clearTooltip') || 'Clear conversation'}>
			🗑
		</button>
	</header>

	<div class="messages" bind:this={listEl}>
		{#if messages.length === 0}
			<div class="empty-hint">
				<div class="hint-icon">💬</div>
				<p>{$t('mind.chat.emptyHint') || 'Ask me anything about your notes. I cite every claim to its source.'}</p>
			</div>
		{:else}
			{#each messages as msg, i (i)}
				<MindChatMessage message={msg} />
			{/each}
		{/if}
	</div>

	{#if errorBanner}
		<div class="error-banner">
			<span>{errorBanner}</span>
			<button onclick={() => (errorBanner = null)}>✕</button>
		</div>
	{/if}

	<div class="composer">
		<textarea
			bind:value={composer}
			onkeydown={handleComposerKey}
			placeholder={$t('mind.chat.composerPlaceholder') || 'Ask about your notes…'}
			rows="2"
			disabled={inFlight}
		></textarea>
		<button class="send-btn" onclick={send} disabled={inFlight || composer.trim().length === 0} title={$t('mind.chat.sendTooltip') || 'Send'}>
			{#if inFlight}
				⟳
			{:else}
				➤
			{/if}
		</button>
	</div>
</div>

<style>
	.mind-chat-pane {
		display: flex;
		flex-direction: column;
		height: 100%;
		min-height: 0;
		font-size: 0.95em;
	}
	.pane-header {
		display: flex;
		align-items: center;
		justify-content: space-between;
		padding: 0.5rem 0.75rem;
		border-bottom: 1px solid var(--border-color, rgba(0, 0, 0, 0.08));
	}
	.title {
		font-weight: 600;
		font-size: 0.95em;
	}
	.clear-btn {
		background: transparent;
		border: 1px solid var(--border-color, rgba(0, 0, 0, 0.08));
		border-radius: 4px;
		padding: 0.2rem 0.5rem;
		cursor: pointer;
		font-family: inherit;
	}
	.clear-btn:disabled {
		opacity: 0.4;
		cursor: not-allowed;
	}
	.messages {
		flex: 1;
		min-height: 0;
		overflow-y: auto;
		padding: 0.5rem 0.75rem;
	}
	.empty-hint {
		text-align: center;
		padding: 2rem 1rem;
		opacity: 0.7;
	}
	.hint-icon {
		font-size: 2rem;
		margin-bottom: 0.5rem;
	}
	.error-banner {
		display: flex;
		align-items: center;
		justify-content: space-between;
		padding: 0.4rem 0.75rem;
		background: rgba(192, 57, 43, 0.12);
		color: var(--err-color, #c0392b);
		font-size: 0.85em;
		border-top: 1px solid rgba(192, 57, 43, 0.3);
		border-bottom: 1px solid rgba(192, 57, 43, 0.3);
	}
	.error-banner button {
		background: transparent;
		border: none;
		color: inherit;
		cursor: pointer;
		font-size: 1em;
	}
	.composer {
		display: flex;
		gap: 0.4rem;
		padding: 0.5rem 0.75rem;
		border-top: 1px solid var(--border-color, rgba(0, 0, 0, 0.08));
	}
	.composer textarea {
		flex: 1;
		resize: none;
		padding: 0.4rem 0.5rem;
		border: 1px solid var(--border-color, rgba(0, 0, 0, 0.1));
		border-radius: 4px;
		background: var(--bg, var(--background, white));
		color: inherit;
		font-family: inherit;
		font-size: inherit;
		line-height: 1.4;
	}
	.composer textarea:focus {
		outline: none;
		border-color: var(--accent-color, #7c3aed);
	}
	.send-btn {
		background: var(--accent-color, #7c3aed);
		color: white;
		border: none;
		border-radius: 4px;
		padding: 0 0.8rem;
		cursor: pointer;
		font-size: 1.1em;
		font-family: inherit;
	}
	.send-btn:disabled {
		opacity: 0.4;
		cursor: not-allowed;
	}
</style>
