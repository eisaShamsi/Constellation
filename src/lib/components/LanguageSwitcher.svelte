<script lang="ts">
	import { locale, toggleLocale } from '$lib/i18n';
</script>

<button class="lang-switch" onclick={toggleLocale} title="Switch language / تغيير اللغة">
	{$locale === 'en' ? 'عربي' : 'EN'}
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
