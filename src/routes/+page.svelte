<script lang="ts">
	import { onMount } from 'svelte';
	import { t } from '$lib/i18n';
	import { hasProvider } from '$lib/ai/store';
	import { vaults, loadVaults } from '$lib/vaults/store';

	onMount(() => { loadVaults(); });
</script>

<div class="hero">
	<h1>{$t('app.name')}</h1>
	<p class="tagline">{$t('app.tagline')}</p>
	<p class="description">{$t('app.description')}</p>

	<div class="actions">
		<a href="/vaults" class="btn primary">{$t('nav.vaults')} ({$vaults.length}) →</a>
		{#if $hasProvider}
			<a href="/skills" class="btn secondary">{$t('nav.skills')}</a>
		{:else}
			<a href="/settings" class="btn secondary">{$t('settings.ai')} →</a>
		{/if}
	</div>

	<div class="features">
		<div class="feature">
			<span class="feature-icon">✦</span>
			<span>{$t('skill.summarize')}</span>
		</div>
		<div class="feature">
			<span class="feature-icon">?</span>
			<span>{$t('skill.qa')}</span>
		</div>
		<div class="feature">
			<span class="feature-icon">✎</span>
			<span>{$t('skill.write')}</span>
		</div>
		<div class="feature">
			<span class="feature-icon">⟡</span>
			<span>{$t('skill.autolink')}</span>
		</div>
	</div>
</div>

<style>
	.hero {
		text-align: center;
		padding-top: 8vh;
	}

	h1 {
		font-size: 3.5rem;
		font-weight: 700;
		background: linear-gradient(135deg, #7c3aed, #3b82f6, #06b6d4);
		-webkit-background-clip: text;
		-webkit-text-fill-color: transparent;
		background-clip: text;
		margin-bottom: 0.25rem;
	}

	.tagline {
		font-size: 1.25rem;
		color: #8b949e;
		margin-top: 0;
		margin-bottom: 2rem;
		font-style: italic;
	}

	.description {
		color: #8b949e;
		line-height: 1.6;
		max-width: 500px;
		margin: 0 auto 2rem;
	}

	.actions { margin-bottom: 3rem; }

	.btn {
		display: inline-block;
		padding: 0.75em 1.5em;
		border-radius: 8px;
		font-weight: 600;
		font-size: 1rem;
		text-decoration: none;
		transition: all 0.2s;
	}
	.btn.primary {
		background: #7c3aed;
		color: white;
	}
	.btn.primary:hover { background: #6d28d9; }
	.btn.secondary {
		background: #21262d;
		border: 1px solid #30363d;
		color: #e0e0e0;
		margin-inline-start: 0.75rem;
	}
	.btn.secondary:hover { border-color: #7c3aed; background: #30363d; }

	.features {
		display: flex;
		justify-content: center;
		gap: 1.5rem;
		flex-wrap: wrap;
	}

	.feature {
		display: flex;
		align-items: center;
		gap: 0.5rem;
		background: #161b22;
		border: 1px solid #21262d;
		padding: 0.6em 1em;
		border-radius: 8px;
		font-size: 0.9rem;
		color: #c9d1d9;
	}

	.feature-icon { font-size: 1.1rem; }
</style>
