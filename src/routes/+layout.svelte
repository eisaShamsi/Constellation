<script lang="ts">
	import NavBar from '$lib/components/NavBar.svelte';
	import { dir } from '$lib/i18n';
	import { page } from '$app/state';
	import type { Snippet } from 'svelte';

	let { children }: { children: Snippet } = $props();

	// Workspace pages get full width, other pages get centered content
	const isWorkspace = $derived(page.url.pathname === '/vaults');
</script>

<div class="app" dir={$dir}>
	<NavBar />
	{#if isWorkspace}
		<main class="workspace">
			{@render children()}
		</main>
	{:else}
		<main class="content">
			{@render children()}
		</main>
	{/if}
</div>

<style>
	:global(html) {
		margin: 0;
		padding: 0;
		overflow: hidden;
	}

	:global(body) {
		margin: 0;
		padding: 0;
		background-color: #0d1117;
		color: #e0e0e0;
		font-family: Inter, Avenir, Helvetica, Arial, sans-serif;
		font-size: 15px;
		line-height: 1.5;
		overflow: hidden;
	}

	:global(a) {
		color: #7c3aed;
		text-decoration: none;
	}

	:global(a:hover) {
		color: #9f67ff;
	}

	:global(*) {
		box-sizing: border-box;
	}

	.app {
		height: 100vh;
		display: flex;
		flex-direction: column;
		overflow: hidden;
	}

	.workspace {
		flex: 1;
		overflow: hidden;
	}

	.content {
		flex: 1;
		padding: 2rem;
		max-width: 900px;
		margin: 0 auto;
		width: 100%;
		overflow-y: auto;
	}
</style>
