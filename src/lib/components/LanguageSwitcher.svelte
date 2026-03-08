<script lang="ts">
	import { locale, toggleLocale } from '$lib/i18n';
</script>

<button class="lang-switch" onclick={toggleLocale} title="Switch language / تغيير اللغة">
	{$locale === 'en' ? 'عربي' : 'EN'}
</button>

<style>
	.lang-switch {
		background: #21262d;
		border: 1px solid #30363d;
		color: #e0e0e0;
		padding: 0.4em 0.8em;
		border-radius: 6px;
		cursor: pointer;
		font-size: 0.85rem;
		font-weight: 600;
		transition: all 0.2s ease;
	}
	.lang-switch:hover {
		border-color: #7c3aed;
		background: #30363d;
	}
</style>
