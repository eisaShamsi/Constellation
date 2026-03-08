<script lang="ts">
	import NavBar from '$lib/components/NavBar.svelte';
	import { dir } from '$lib/i18n';
	import type { Snippet } from 'svelte';

	let { children }: { children: Snippet } = $props();
</script>

<div class="app" dir={$dir}>
	<NavBar />
	<main class="content">
		{@render children()}
	</main>
</div>

<style>
	:global(html) {
		margin: 0;
		padding: 0;
	}

	:global(body) {
		margin: 0;
		padding: 0;
		background-color: #0d1117;
		color: #e0e0e0;
		font-family: Inter, Avenir, Helvetica, Arial, sans-serif;
		font-size: 16px;
		line-height: 1.5;
	}

	:global(a) {
		color: #7c3aed;
		text-decoration: none;
	}

	:global(a:hover) {
		color: #9f67ff;
	}

	.app {
		min-height: 100vh;
		display: flex;
		flex-direction: column;
	}

	.content {
		flex: 1;
		padding: 2rem;
		max-width: 900px;
		margin: 0 auto;
		width: 100%;
		box-sizing: border-box;
	}
</style>
