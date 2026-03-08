<script lang="ts">
	import { t, dir } from '$lib/i18n';
	import LanguageSwitcher from './LanguageSwitcher.svelte';
	import { page } from '$app/state';
</script>

<nav class="navbar" dir={$dir}>
	<div class="nav-brand">
		<a href="/" class="brand-link">Constellation</a>
	</div>
	<div class="nav-links">
		<a href="/" class="nav-link" class:active={page.url.pathname === '/'}>
			{$t('nav.home')}
		</a>
		<a href="/vaults" class="nav-link" class:active={page.url.pathname === '/vaults'}>
			{$t('nav.vaults')}
		</a>
		<a href="/skills" class="nav-link" class:active={page.url.pathname === '/skills'}>
			{$t('nav.skills')}
		</a>
		<a href="/settings" class="nav-link" class:active={page.url.pathname === '/settings'}>
			{$t('nav.settings')}
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
		background: #f6f8fa;
		border-bottom: 1px solid #d0d7de;
	}

	.brand-link {
		font-size: 1.1rem;
		font-weight: 700;
		background: linear-gradient(135deg, #7c3aed, #3b82f6);
		-webkit-background-clip: text;
		-webkit-text-fill-color: transparent;
		background-clip: text;
		text-decoration: none;
	}

	.nav-links {
		display: flex;
		gap: 0.5rem;
	}

	.nav-link {
		color: #57606a;
		text-decoration: none;
		padding: 0.4em 0.8em;
		border-radius: 6px;
		font-size: 0.9rem;
		transition: all 0.2s ease;
	}

	.nav-link:hover {
		color: #24292f;
		background: #eaeef2;
	}

	.nav-link.active {
		color: #24292f;
		background: #eaeef2;
	}

	.nav-actions {
		display: flex;
		align-items: center;
		gap: 0.5rem;
	}
</style>
