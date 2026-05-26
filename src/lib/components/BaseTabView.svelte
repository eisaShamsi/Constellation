<script lang="ts">
	// MIG-054 §I.0 — Wire BaseView for .base tabs
	//
	// Wrapper around BaseView.svelte that loads the BaseDefinition from disk
	// (via parse_base_file / parse_workspace_base) and routes row-click events
	// to openNoteTab. Mounted by +layout.svelte when a tab's path ends with
	// `.base` — without this wrapper, .base files render as raw JSON in NotePane.
	//
	// Discovered as a pre-existing gap during MIG-054 §I Boss-test: the Bases
	// MVP (commit c5b05f5c) shipped BaseView.svelte but never wired it into the
	// tab-router. §A-§G assumed it was wired; §I.0 wires it.

	import BaseView from './BaseView.svelte';
	import { parseBaseFile, parseWorkspaceBase } from '$lib/bases/store';
	import { openNoteTab } from '$lib/libraries/store';
	import { libraries } from '$lib/libraries/store';
	import { get } from 'svelte/store';
	import type { BaseDefinition } from '$lib/bases/types';

	let { tab }: { tab: { path: string; name: string; libraryName?: string; libraryColor?: string } } = $props();

	let definition = $state<BaseDefinition | null>(null);
	let loading = $state(true);
	let error = $state('');

	/**
	 * Detect whether the .base file lives in the workspace bases directory
	 * (`{universe}/.constellation/bases/`) vs a library folder. Workspace bases
	 * use a different parse path (parseWorkspaceBase) that enforces directory
	 * containment in `{universe}/.constellation/bases/`.
	 */
	function isWorkspaceBasePath(path: string): boolean {
		// Normalize separators for the check
		const normalized = path.replace(/\\/g, '/');
		return normalized.includes('/.constellation/bases/');
	}

	async function load(path: string) {
		loading = true;
		error = '';
		try {
			definition = isWorkspaceBasePath(path)
				? await parseWorkspaceBase(path)
				: await parseBaseFile(path);
		} catch (e: any) {
			error = e?.toString() ?? 'Failed to load base';
			definition = null;
		} finally {
			loading = false;
		}
	}

	// Reload whenever the tab path changes (different .base tab focused).
	$effect(() => {
		const p = tab.path;
		if (p) {
			void load(p);
		}
	});

	function handleOpenNote(path: string, libraryName: string) {
		const libs = get(libraries);
		const lib = libs.find((l) => l.name === libraryName);
		const color = lib ? '#7c3aed' : '#7c3aed';
		void openNoteTab(path, libraryName, color);
	}
</script>

{#if loading}
	<div class="base-tab-loading">
		<p>Loading base…</p>
	</div>
{:else if error}
	<div class="base-tab-error">
		<p>Failed to load base file</p>
		<pre>{error}</pre>
	</div>
{:else if definition}
	<BaseView {definition} filePath={tab.path} onOpenNote={handleOpenNote} />
{/if}

<style>
	.base-tab-loading,
	.base-tab-error {
		padding: 2rem;
		text-align: center;
		color: var(--color-text-secondary, #888);
	}

	.base-tab-error pre {
		text-align: start;
		white-space: pre-wrap;
		background: var(--color-bg-subtle, #f6f6f6);
		padding: 1rem;
		border-radius: 4px;
		margin-top: 1rem;
		font-size: 0.85rem;
	}
</style>
