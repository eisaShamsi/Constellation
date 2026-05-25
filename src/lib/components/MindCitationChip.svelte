<script lang="ts">
	import { openNoteTab, libraries } from '$lib/libraries/store';
	import { get } from 'svelte/store';

	let { path }: { path: string } = $props();

	// Derive a short display label from the file path — final segment
	// without extension. Falls back to the full path if no separator.
	let label = $derived.by(() => {
		const fs = path.replace(/\\/g, '/');
		const last = fs.split('/').pop() ?? path;
		return last.replace(/\.md$/i, '');
	});

	// Resolve which library this path belongs to so openNoteTab opens
	// the tab against the correct registered library root. Color is
	// the default — openNoteTab itself reads the canonical per-library
	// colour from libraryAppearances. Best-effort match; falls back to
	// the first library if no path matches.
	function resolveLibraryName(): string {
		const libs = get(libraries);
		const fs = path.replace(/\\/g, '/');
		for (const lib of libs) {
			const root = lib.path.replace(/\\/g, '/');
			if (fs === root || fs.startsWith(root + '/')) {
				return lib.name;
			}
		}
		return libs[0]?.name ?? '';
	}

	async function handleClick() {
		const name = resolveLibraryName();
		await openNoteTab(path, name);
	}
</script>

<button class="citation-chip" onclick={handleClick} title={path}>
	<span class="icon">📎</span>
	<span class="label">{label}</span>
</button>

<style>
	.citation-chip {
		display: inline-flex;
		align-items: center;
		gap: 0.25rem;
		padding: 0.1rem 0.5rem;
		margin: 0 0.15rem;
		border-radius: 999px;
		background: var(--accent-color-soft, rgba(124, 58, 237, 0.12));
		color: var(--accent-color, #7c3aed);
		font-size: 0.85em;
		border: 1px solid var(--accent-color-soft, rgba(124, 58, 237, 0.25));
		cursor: pointer;
		font-family: inherit;
		transition: background 120ms ease, transform 80ms ease;
	}
	.citation-chip:hover {
		background: var(--accent-color-soft, rgba(124, 58, 237, 0.22));
	}
	.citation-chip:active {
		transform: scale(0.97);
	}
	.icon {
		font-size: 0.9em;
		opacity: 0.85;
	}
	.label {
		max-width: 220px;
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}
</style>
