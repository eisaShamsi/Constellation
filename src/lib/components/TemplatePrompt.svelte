<script lang="ts">
	import { onMount } from 'svelte';
	import { t } from '$lib/i18n';
	import { detectDir } from '$lib/utils';

	let {
		question = '',
		defaultValue = '',
		onSubmit,
		onCancel,
		// MIG-103 §1 — an OPTIONAL second decision rendered above the input, used by
		// "Save as template (snippet)" to let the user pick the snippet's extent:
		// their selection, or the whole note. Purely additive — every existing
		// mount site omits it and is unchanged, and `onSubmit`'s second argument is
		// simply ignored by callers that don't ask a choice.
		choiceLabel = '',
		choices = [] as { value: string; label: string }[],
		choiceDefault = '',
		// MIG-103 — the destination row for "New note from template": the proposed
		// folder is SHOWN (never chosen silently) with a Change… door to the folder
		// tree. onChangeDestination receives the current input value so an edited
		// title survives the round-trip through the picker. Optional — every other
		// mount site omits both and is unchanged.
		destinationLabel = '',
		onChangeDestination = undefined,
	}: {
		question: string;
		defaultValue?: string;
		onSubmit: (value: string, choice?: string) => void;
		onCancel: () => void;
		choiceLabel?: string;
		choices?: { value: string; label: string }[];
		choiceDefault?: string;
		destinationLabel?: string;
		onChangeDestination?: (currentValue: string) => void;
	} = $props();

	let value = $state(defaultValue || '');
	let choice = $state(choiceDefault || choices[0]?.value || '');
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
			onSubmit(value, choice || undefined);
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
		{#if choices.length > 1}
			<div class="tpl-prompt-choice">
				{#if choiceLabel}<div class="tpl-prompt-choice-label">{choiceLabel}</div>{/if}
				<div class="tpl-prompt-choice-row">
					{#each choices as c (c.value)}
						<button type="button" class="tpl-prompt-choice-btn" class:selected={choice === c.value}
							onclick={() => choice = c.value}>{c.label}</button>
					{/each}
				</div>
			</div>
		{/if}
		<input
			bind:this={inputEl}
			type="text"
			class="tpl-prompt-input"
			dir="auto"
			bind:value={value}
			onkeydown={handleKeydown}
			placeholder={defaultValue || ''}
		/>
		{#if destinationLabel}
			<div class="tpl-prompt-dest">
				<svg class="tpl-prompt-dest-ic" width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M3 7a2 2 0 0 1 2-2h4l2 2h8a2 2 0 0 1 2 2v8a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2z"/></svg>
				<span class="tpl-prompt-dest-where">{$t('templates.newNoteDestination')}</span>
				<span class="tpl-prompt-dest-label" dir={detectDir(destinationLabel)}>{destinationLabel}</span>
				{#if onChangeDestination}
					<button type="button" class="tpl-prompt-dest-change" onclick={() => onChangeDestination?.(value)}>{$t('templates.newNoteDestinationChange')}</button>
				{/if}
			</div>
		{/if}
		<div class="tpl-prompt-actions">
			<button class="tpl-prompt-btn tpl-prompt-cancel" onclick={onCancel}>
				{$t('templates.promptCancel')}
			</button>
			<button class="tpl-prompt-btn tpl-prompt-submit" onclick={() => onSubmit(value, choice || undefined)}>
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
	.tpl-prompt-choice { padding: 0 16px 4px; }
	.tpl-prompt-choice-label { font-size: 0.72rem; color: var(--text-faint); margin-bottom: 4px; }
	.tpl-prompt-choice-row { display: flex; gap: 6px; flex-wrap: wrap; }
	.tpl-prompt-choice-btn {
		padding: 4px 10px; border-radius: 6px; cursor: pointer; font-size: 0.78rem;
		font-family: inherit; text-align: start;
		border: 1px solid var(--background-modifier-border, #ccc);
		background: var(--background-primary, #fff); color: var(--text-normal);
	}
	.tpl-prompt-choice-btn.selected {
		background: var(--interactive-accent); color: var(--text-on-accent);
		border-color: var(--interactive-accent);
	}
	.tpl-prompt-dest {
		display: flex; align-items: center; gap: 6px; min-width: 0;
		margin-top: 10px; font-size: 0.78rem; color: var(--text-muted);
	}
	.tpl-prompt-dest-ic { flex-shrink: 0; opacity: 0.7; }
	.tpl-prompt-dest-where { flex-shrink: 0; }
	.tpl-prompt-dest-label {
		overflow: hidden; text-overflow: ellipsis; white-space: nowrap;
		color: var(--text-normal);
	}
	.tpl-prompt-dest-change {
		flex-shrink: 0; margin-inline-start: auto;
		border: none; background: none; cursor: pointer;
		color: var(--interactive-accent); font-size: 0.78rem; font-family: inherit;
		padding: 2px 4px; border-radius: 4px;
	}
	.tpl-prompt-dest-change:hover { background: var(--background-modifier-hover); }
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
