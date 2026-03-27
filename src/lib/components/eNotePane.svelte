<script lang="ts">
	/**
	 * eNotePane — Phase 0: The Skeleton
	 * Gray desk + white paper + title. NO editor.
	 * Spec: docs/eNotePane-spec.md, Section 10 (Phase 0)
	 */
	import { t } from '$lib/i18n';
	import { appSettings } from '$lib/libraries/store';

	let {
		title = '',
		dir = 'ltr' as 'ltr' | 'rtl',
		ontitlechange,
	}: {
		title?: string;
		dir?: 'ltr' | 'rtl';
		ontitlechange?: (newTitle: string) => void;
	} = $props();

	let titleValue = $state(title);
	let titleEl: HTMLInputElement | undefined;

	/* ─── Title ─── */
	function generateAutoTitle(): string {
		const now = new Date();
		const dd = String(now.getDate()).padStart(2, '0');
		const mm = String(now.getMonth() + 1).padStart(2, '0');
		const yyyy = now.getFullYear();
		const hh = String(now.getHours()).padStart(2, '0');
		const min = String(now.getMinutes()).padStart(2, '0');
		return `CoNote${dd}${mm}${yyyy}.${hh}:${min}`;
	}

	function handleTitleBlur() {
		const trimmed = titleValue.trim();
		if (!trimmed) {
			titleValue = generateAutoTitle();
		}
		if (titleValue !== title) {
			ontitlechange?.(titleValue);
		}
	}

	function handleTitleKeydown(e: KeyboardEvent) {
		if (e.key === 'Enter') {
			e.preventDefault();
			/* Phase 1 will focus the editor here */
		}
	}

	const titleAlignment = $derived($appSettings.titleAlignment ?? 'center');
</script>

<div class="e-desk" dir={dir}>
	<div class="e-paper">
		<input
			class="e-title"
			class:e-title-center={titleAlignment === 'center'}
			bind:this={titleEl}
			bind:value={titleValue}
			dir="auto"
			placeholder={$t('eNotePane.titlePlaceholder')}
			spellcheck="false"
			onblur={handleTitleBlur}
			onkeydown={handleTitleKeydown}
		/>
	</div>
</div>

<style>
	/* ─── The Desk: gray surface behind the paper (spec 3.1) ─── */
	.e-desk {
		flex: 1;
		display: flex;
		flex-direction: column;
		align-items: center;
		background: #e8e8ec;
		padding-inline: 24px;
		overflow-y: auto;
		overflow-x: hidden;
		min-width: 0;
		min-height: 0;
	}

	/* ─── The Paper: white writing surface (spec 3.1) ─── */
	.e-paper {
		width: 100%;
		max-width: 1200px;
		flex: 1;
		display: flex;
		flex-direction: column;
		background: #ffffff;
		padding: 48px;
		min-width: 0;
		overflow-y: auto;
		overflow-x: hidden;
	}

	/* ─── Title: note identity (spec 0.3) ─── */
	.e-title {
		display: block;
		width: 100%;
		border: none;
		outline: none;
		background: transparent;
		font-size: 28px;
		font-weight: 700;
		font-family: inherit;
		color: var(--text-normal, #1a1a1a);
		padding: 0;
		margin-block: 0 24px;
		margin-inline: 0;
		text-align: start;
	}
	.e-title.e-title-center {
		text-align: center;
	}
	.e-title::placeholder {
		color: var(--text-faint, #ccc);
		font-weight: 400;
	}
</style>
