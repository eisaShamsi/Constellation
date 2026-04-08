<script lang="ts">
	import { t, dir } from '$lib/i18n';
	import LanguageSwitcher from './LanguageSwitcher.svelte';
	import { page } from '$app/state';
</script>

<nav class="navbar" dir={$dir}>
	<div class="nav-brand">
		<a href="/" class="brand-link">
			<svg class="brand-logo" width="24" height="24" viewBox="0 0 200 200" xmlns="http://www.w3.org/2000/svg">
				<circle cx="100" cy="100" r="30" fill="#534AB7"/>
				<circle cx="100" cy="100" r="19" fill="#3C3489"/>
				<circle cx="45" cy="42" r="24" fill="#378ADD"/>
				<circle cx="130" cy="52" r="20" fill="#7F77DD"/>
				<circle cx="162" cy="110" r="16" fill="#1D9E75"/>
				<circle cx="80" cy="158" r="13" fill="#D85A30"/>
			</svg>
			Constellation
		</a>
	</div>
	<div class="nav-links">
		<a href="/" class="nav-link" class:active={page.url.pathname === '/'}>
			{$t('nav.home')}
		</a>
		<a href="/libraries" class="nav-link" class:active={page.url.pathname === '/libraries'}>
			{$t('nav.libraries')}
		</a>
		<a href="/skills" class="nav-link" class:active={page.url.pathname === '/skills'}>
			{$t('nav.skills')}
		</a>
	</div>
	<div class="nav-actions">
		<LanguageSwitcher />
	</div>
</nav>

<style>
	.navbar {
		display: flex;
		align-items: center;
		justify-content: space-between;
		padding: 0.75rem 1.5rem;
		background: var(--background-secondary);
		border-bottom: 1px solid var(--background-modifier-border);
	}

	.brand-link {
		display: flex;
		align-items: center;
		gap: 0.5rem;
		font-size: 1.1rem;
		font-weight: 700;
		letter-spacing: 2px;
		text-transform: uppercase;
		background: linear-gradient(135deg, var(--interactive-accent), var(--color-blue));
		-webkit-background-clip: text;
		-webkit-text-fill-color: transparent;
		background-clip: text;
		text-decoration: none;
	}

	.brand-logo {
		flex-shrink: 0;
	}

	.nav-links {
		display: flex;
		gap: 0.5rem;
	}

	.nav-link {
		color: var(--text-muted);
		text-decoration: none;
		padding: 0.4em 0.8em;
		border-radius: 6px;
		font-size: 0.9rem;
		transition: all 0.2s ease;
	}

	.nav-link:hover {
		color: var(--text-normal);
		background: var(--background-modifier-hover);
	}

	.nav-link.active {
		color: var(--text-normal);
		background: var(--background-modifier-hover);
	}

	.nav-actions {
		display: flex;
		align-items: center;
		gap: 0.5rem;
	}
</style>
