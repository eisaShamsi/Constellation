<script lang="ts">
	import { onMount } from 'svelte';
	import { t, locale } from '$lib/i18n';
	import { hasProvider } from '$lib/ai/store';
	import { vaultStats, loadVaults, loadAllStats, totalStars } from '$lib/vaults/store';

	onMount(async () => {
		await loadVaults();
		await loadAllStats();
	});

	const colors = ['#7c3aed', '#3b82f6', '#06b6d4', '#10b981', '#f59e0b', '#ef4444'];
	const ar = $derived($locale === 'ar');
</script>

<div class="sky-view">
	<div class="hero">
		<h1>{$t('app.name')}</h1>
		<p class="tagline">{$t('app.tagline')}</p>
	</div>

	<!-- Sky Map — visual overview -->
	{#if $vaultStats.length > 0}
		<div class="sky-map">
			{#each $vaultStats as universe, i}
				<a href="/vaults" class="universe-bubble" style="--color: {colors[i % colors.length]}; --size: {Math.max(60, Math.min(130, 60 + universe.star_count * 3))}px">
					<div class="bubble-glow"></div>
					<span class="bubble-count">{universe.star_count}</span>
					<span class="bubble-name">{universe.name}</span>
				</a>
			{/each}
		</div>

		<div class="sky-stats">
			<div class="stat">
				<span class="stat-value">{$vaultStats.length}</span>
				<span class="stat-label">{ar ? 'عوالم' : 'Universes'}</span>
			</div>
			<div class="stat">
				<span class="stat-value">{$totalStars}</span>
				<span class="stat-label">{ar ? 'نجمة' : 'Stars'}</span>
			</div>
			<div class="stat">
				<span class="stat-value">{$vaultStats.reduce((s, v) => s + v.folder_count, 0)}</span>
				<span class="stat-label">{ar ? 'مجموعة' : 'Clusters'}</span>
			</div>
		</div>
	{:else}
		<div class="empty-sky">
			<div class="empty-orbs">
				<div class="ghost-orb" style="--color: #7c3aed"></div>
				<div class="ghost-orb" style="--color: #3b82f6"></div>
				<div class="ghost-orb" style="--color: #06b6d4"></div>
			</div>
			<p class="empty-text">{ar ? 'أضف عوالمك لرؤية سمائك' : 'Add your universes to see your sky'}</p>
		</div>
	{/if}

	<!-- Actions -->
	<div class="actions">
		<a href="/vaults" class="btn primary">
			{ar ? 'استكشف العوالم' : 'Explore Universes'} →
		</a>
		{#if $hasProvider}
			<a href="/skills" class="btn secondary">{$t('nav.skills')}</a>
		{:else}
			<a href="/settings" class="btn secondary">{ar ? 'أضف مزود ذكاء اصطناعي' : 'Connect AI'} →</a>
		{/if}
	</div>
</div>

<style>
	.sky-view { text-align: center; padding-top: 4vh; }

	h1 {
		font-size: 3rem;
		font-weight: 700;
		background: linear-gradient(135deg, #7c3aed, #3b82f6, #06b6d4);
		-webkit-background-clip: text;
		-webkit-text-fill-color: transparent;
		background-clip: text;
		margin: 0;
	}

	.tagline {
		font-size: 1.1rem;
		color: #8b949e;
		margin: 0.25rem 0 2.5rem;
		font-style: italic;
	}

	/* Sky Map */
	.sky-map {
		display: flex;
		justify-content: center;
		align-items: center;
		gap: 2rem;
		flex-wrap: wrap;
		padding: 2rem 0;
		min-height: 180px;
	}

	.universe-bubble {
		width: var(--size);
		height: var(--size);
		border-radius: 50%;
		display: flex;
		flex-direction: column;
		align-items: center;
		justify-content: center;
		position: relative;
		text-decoration: none;
		color: white;
		transition: transform 0.3s ease;
	}
	.universe-bubble:hover { transform: scale(1.1); }

	.bubble-glow {
		position: absolute;
		inset: 0;
		border-radius: 50%;
		background: radial-gradient(circle, var(--color)44, var(--color)11);
		border: 2px solid var(--color)66;
		transition: box-shadow 0.3s;
	}
	.universe-bubble:hover .bubble-glow {
		box-shadow: 0 0 30px var(--color)44;
	}

	.bubble-count {
		position: relative;
		font-size: 1.3rem;
		font-weight: 700;
		z-index: 1;
	}
	.bubble-name {
		position: relative;
		font-size: 0.7rem;
		color: #c9d1d9;
		z-index: 1;
		max-width: calc(var(--size) - 16px);
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}

	/* Stats */
	.sky-stats {
		display: flex;
		justify-content: center;
		gap: 3rem;
		margin: 2rem 0;
	}
	.stat { display: flex; flex-direction: column; align-items: center; }
	.stat-value { font-size: 1.8rem; font-weight: 700; color: #e0e0e0; }
	.stat-label { font-size: 0.8rem; color: #484f58; text-transform: uppercase; letter-spacing: 0.05em; }

	/* Empty State */
	.empty-sky { padding: 3rem 0; }
	.empty-orbs {
		display: flex;
		justify-content: center;
		gap: 1.5rem;
		margin-bottom: 1.5rem;
	}
	.ghost-orb {
		width: 60px;
		height: 60px;
		border-radius: 50%;
		background: radial-gradient(circle, var(--color)22, transparent);
		border: 1px dashed var(--color)33;
		animation: pulse 3s ease-in-out infinite;
	}
	.ghost-orb:nth-child(2) { animation-delay: 1s; width: 80px; height: 80px; }
	.ghost-orb:nth-child(3) { animation-delay: 2s; }
	@keyframes pulse { 0%, 100% { opacity: 0.4; } 50% { opacity: 0.8; } }
	.empty-text { color: #484f58; font-size: 0.95rem; }

	/* Actions */
	.actions { margin-top: 2rem; }
	.btn {
		display: inline-block;
		padding: 0.75em 1.5em;
		border-radius: 8px;
		font-weight: 600;
		font-size: 1rem;
		text-decoration: none;
		transition: all 0.2s;
	}
	.btn.primary { background: #7c3aed; color: white; }
	.btn.primary:hover { background: #6d28d9; }
	.btn.secondary {
		background: #21262d;
		border: 1px solid #30363d;
		color: #e0e0e0;
		margin-inline-start: 0.75rem;
	}
	.btn.secondary:hover { border-color: #7c3aed; background: #30363d; }
</style>
