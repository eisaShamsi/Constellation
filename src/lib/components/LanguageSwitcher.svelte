<script lang="ts">
	import { locale, SUPPORTED_LOCALES } from '$lib/i18n';

	function cycleLocale() {
		const codes = SUPPORTED_LOCALES.map(l => l.code);
		const idx = codes.indexOf($locale);
		const next = codes[(idx + 1) % codes.length];
		locale.set(next);
	}
</script>

<button class="lang-switch" onclick={cycleLocale} title="Change language">
	{$locale.toUpperCase()}
</button>

<style>
	.lang-switch {
		background: var(--interactive-normal);
		border: 1px solid var(--background-modifier-border);
		color: var(--text-normal);
		padding: 0.4em 0.8em;
		border-radius: 6px;
		cursor: pointer;
		font-size: 0.85rem;
		font-weight: 600;
		transition: all 0.2s ease;
	}
	.lang-switch:hover {
		border-color: var(--interactive-accent);
		background: var(--interactive-hover);
	}
</style>
