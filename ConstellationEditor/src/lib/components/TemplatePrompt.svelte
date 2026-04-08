<script lang="ts">
	import { onMount } from 'svelte';
	import { t } from '$lib/i18n';

	let {
		question = '',
		defaultValue = '',
		onSubmit,
		onCancel,
	}: {
		question: string;
		defaultValue?: string;
		onSubmit: (value: string) => void;
		onCancel: () => void;
	} = $props();

	let value = $state(defaultValue || '');
	let inputEl: HTMLInputElement;

	onMount(() => {
		inputEl?.focus();
		if (defaultValue) inputEl?.select();
	});

	function handleKeydown(e: KeyboardEvent) {
		if (e.key === 'Escape') {
			onCancel();
		} else if (e.key === 'Enter') {
			e.preventDefault();
			onSubmit(value);
		}
	}
</script>

<!-- svelte-ignore a11y_no_static_element_interactions -->
<div class="tpl-prompt-overlay" onclick={onCancel} onkeydown={handleKeydown}>
	<!-- svelte-ignore a11y_click_events_have_key_events -->
	<!-- svelte-ignore a11y_no_static_element_interactions -->
	<div class="tpl-prompt-panel" onclick={(e) => e.stopPropagation()}>
		<div class="tpl-prompt-header">{$t('templates.promptTitle')}</div>
		<div class="tpl-prompt-question">{question}</div>
		<input
			bind:this={inputEl}
			type="text"
			class="tpl-prompt-input"
			bind:value={value}
			onkeydown={handleKeydown}
			placeholder={defaultValue || ''}
		/>
		<div class="tpl-prompt-actions">
			<button class="tpl-prompt-btn tpl-prompt-cancel" onclick={onCancel}>
				{$t('templates.promptCancel')}
			</button>
			<button class="tpl-prompt-btn tpl-prompt-submit" onclick={() => onSubmit(value)}>
				{$t('templates.promptSubmit')}
			</button>
		</div>
	</div>
</div>

<style>
	.tpl-prompt-overlay {
		position: fixed; inset: 0; z-index: 1100;
		background: var(--background-modifier-cover);
		display: flex; justify-content: center; padding-top: 20vh;
	}
	.tpl-prompt-panel {
		background: var(--background-primary); border-radius: 8px;
		box-shadow: var(--shadow-l);
		width: 420px; max-height: 280px;
		display: flex; flex-direction: column;
		padding: 16px;
	}
	.tpl-prompt-header {
		font-size: 0.75rem; font-weight: 600;
		color: var(--text-faint); text-transform: uppercase;
		letter-spacing: 0.5px; margin-bottom: 8px;
	}
	.tpl-prompt-question {
		font-size: 0.9rem; color: var(--text-normal);
		margin-bottom: 12px; line-height: 1.4;
	}
	.tpl-prompt-input {
		border: 1px solid var(--background-modifier-border);
		border-radius: 6px; padding: 8px 12px;
		font-size: 0.9rem; font-family: inherit;
		color: var(--text-normal); background: var(--background-primary);
		outline: none; width: 100%; box-sizing: border-box;
	}
	.tpl-prompt-input:focus { border-color: var(--interactive-accent); }
	.tpl-prompt-actions {
		display: flex; justify-content: flex-end; gap: 8px; margin-top: 14px;
	}
	.tpl-prompt-btn {
		padding: 6px 16px; border-radius: 6px; font-size: 0.82rem;
		font-family: inherit; cursor: pointer; border: none;
	}
	.tpl-prompt-cancel {
		background: var(--background-modifier-hover); color: var(--text-muted);
	}
	.tpl-prompt-cancel:hover { background: var(--background-modifier-border); }
	.tpl-prompt-submit {
		background: var(--interactive-accent); color: var(--text-on-accent);
	}
	.tpl-prompt-submit:hover { filter: brightness(1.1); }
</style>
