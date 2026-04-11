<script lang="ts">
	import { t } from '$lib/i18n';
	import { invoke } from '@tauri-apps/api/core';
	import { createNote, writeNote, buildFullContent } from '$lib/libraries/store';
	import type { SkyNode } from '$lib/libraries/store';

	let {
		notes = [] as SkyNode[],
		activeTrail = null as any,
		libraryPath = '',
		libraryName = '',
		onClose,
	}: {
		notes?: SkyNode[];
		activeTrail?: any;
		libraryPath?: string;
		libraryName?: string;
		onClose?: () => void;
	} = $props();

	// ─── Canvas state ───
	interface ForgeBlock {
		id: string;
		noteName: string;
		notePath: string;
		content: string;
		collapsed: boolean;
	}

	let compositionTitle = $state('');
	let blocks = $state<ForgeBlock[]>([]);
	let transitions = $state<Record<string, string>>({}); // blockId → user text after that block
	let searchQuery = $state('');
	let strataFilter = $state(0); // 0 = all, 4 = concept+, 6 = theory+

	// ─── Filtered notes for left panel ───
	const filteredNotes = $derived.by(() => {
		let result = notes;
		if (strataFilter > 0) {
			result = result.filter(n => (n.stratum ?? 0) >= strataFilter);
		}
		if (searchQuery.trim()) {
			const q = searchQuery.toLowerCase();
			result = result.filter(n => n.name.toLowerCase().includes(q));
		}
		return result.slice(0, 100); // limit for performance
	});

	// ─── Import trail as backbone ───
	$effect(() => {
		if (activeTrail && activeTrail.notes && blocks.length === 0) {
			const trailBlocks: ForgeBlock[] = [];
			for (const note of activeTrail.notes) {
				if (note.exists) {
					trailBlocks.push({
						id: `block_${Date.now()}_${Math.random().toString(36).slice(2, 6)}`,
						noteName: note.name,
						notePath: note.path,
						content: '',
						collapsed: true,
					});
				}
			}
			blocks = trailBlocks;
			compositionTitle = activeTrail.title ? `${activeTrail.title} — Composition` : 'New Composition';
			// Load content for each block
			for (const block of blocks) {
				loadBlockContent(block);
			}
		}
	});

	async function loadBlockContent(block: ForgeBlock) {
		try {
			const content: string = await invoke('read_note', { filePath: block.notePath });
			// Strip frontmatter for display
			let body = content;
			if (body.startsWith('---')) {
				const end = body.indexOf('\n---', 3);
				if (end >= 0) body = body.substring(end + 4).trim();
			}
			block.content = body;
			blocks = [...blocks]; // trigger reactivity
		} catch {}
	}

	async function addNote(note: SkyNode) {
		// Don't add duplicates
		if (blocks.some(b => b.notePath === note.path)) return;
		const block: ForgeBlock = {
			id: `block_${Date.now()}_${Math.random().toString(36).slice(2, 6)}`,
			noteName: note.name.replace(/\.md$/, ''),
			notePath: note.path,
			content: '',
			collapsed: true,
		};
		blocks = [...blocks, block];
		loadBlockContent(block);
	}

	function removeBlock(id: string) {
		blocks = blocks.filter(b => b.id !== id);
		const { [id]: _, ...rest } = transitions;
		transitions = rest;
	}

	function moveBlock(id: string, direction: 'up' | 'down') {
		const idx = blocks.findIndex(b => b.id === id);
		if (idx < 0) return;
		const newIdx = direction === 'up' ? idx - 1 : idx + 1;
		if (newIdx < 0 || newIdx >= blocks.length) return;
		const newBlocks = [...blocks];
		[newBlocks[idx], newBlocks[newIdx]] = [newBlocks[newIdx], newBlocks[idx]];
		blocks = newBlocks;
	}

	function toggleCollapse(id: string) {
		blocks = blocks.map(b => b.id === id ? { ...b, collapsed: !b.collapsed } : b);
	}

	async function exportComposition() {
		if (!libraryPath || !compositionTitle.trim()) return;
		// Build markdown
		let md = '';
		for (const block of blocks) {
			md += `## ${block.noteName}\n\n`;
			if (block.content) {
				md += block.content + '\n\n';
			}
			const transition = transitions[block.id]?.trim();
			if (transition) {
				md += transition + '\n\n';
			}
		}

		const fileName = compositionTitle.trim().replace(/[<>:"/\\|?*]/g, '_') + '.md';
		const frontmatter = `---\nstage: synthesis\ntitle: "${compositionTitle.trim()}"\ncreated: ${new Date().toISOString().split('T')[0]}\n---\n`;
		const content = frontmatter + md;

		try {
			const newPath = await createNote(libraryPath, fileName);
			await writeNote(newPath, content);
			onClose?.();
		} catch (e) {
			console.error('Export failed:', e);
		}
	}
</script>

<div class="forge">
	<!-- Header -->
	<div class="forge-header">
		<input class="forge-title" type="text" dir="auto" placeholder={$t('expressionForge.titlePlaceholder') || 'Composition title...'} bind:value={compositionTitle} />
		<div class="forge-header-actions">
			<button class="forge-btn forge-export" onclick={exportComposition} disabled={blocks.length === 0}>
				{$t('expressionForge.export') || 'Export as Note'}
			</button>
			<button class="forge-btn forge-close" onclick={() => onClose?.()}>×</button>
		</div>
	</div>

	<div class="forge-body">
		<!-- Left panel: Note browser -->
		<div class="forge-left">
			<div class="forge-filters">
				<select class="forge-filter-select" bind:value={strataFilter}>
					<option value={0}>{$t('expressionForge.allNotes') || 'All notes'}</option>
					<option value={4}>{$t('expressionForge.conceptPlus') || 'Concept+ (4+)'}</option>
					<option value={6}>{$t('expressionForge.theoryPlus') || 'Theory+ (6+)'}</option>
				</select>
				<input class="forge-search" type="text" dir="auto" placeholder={$t('expressionForge.search') || 'Search...'} bind:value={searchQuery} />
			</div>
			<div class="forge-note-list">
				{#each filteredNotes as note}
					<button class="forge-note-item" class:added={blocks.some(b => b.notePath === note.path)} onclick={() => addNote(note)}>
						<span class="forge-note-stratum">{note.stratum ?? '·'}</span>
						<span class="forge-note-name" dir="auto">{note.name.replace(/\.md$/, '')}</span>
					</button>
				{/each}
				{#if filteredNotes.length === 0}
					<div class="forge-empty">{$t('expressionForge.noNotes') || 'No notes found'}</div>
				{/if}
			</div>
		</div>

		<!-- Divider -->
		<div class="forge-divider"></div>

		<!-- Right panel: Composition canvas -->
		<div class="forge-canvas">
			{#if blocks.length === 0}
				<div class="forge-canvas-empty">
					<div class="forge-canvas-empty-icon">✨</div>
					<div class="forge-canvas-empty-text">{$t('expressionForge.emptyCanvas') || 'Click notes from the left panel to build your composition.'}</div>
				</div>
			{:else}
				{#each blocks as block, idx (block.id)}
					<div class="forge-block">
						<div class="forge-block-header">
							<button class="forge-block-toggle" onclick={() => toggleCollapse(block.id)}>
								{block.collapsed ? '▸' : '▾'}
							</button>
							<span class="forge-block-name" dir="auto">{block.noteName}</span>
							<div class="forge-block-actions">
								<button class="forge-block-btn" disabled={idx === 0} onclick={() => moveBlock(block.id, 'up')}>↑</button>
								<button class="forge-block-btn" disabled={idx === blocks.length - 1} onclick={() => moveBlock(block.id, 'down')}>↓</button>
								<button class="forge-block-btn forge-block-remove" onclick={() => removeBlock(block.id)}>×</button>
							</div>
						</div>
						{#if !block.collapsed}
							<div class="forge-block-content" dir="auto">
								{block.content || '...'}
							</div>
						{/if}
						<!-- Transition text area -->
						<textarea class="forge-transition" dir="auto"
							placeholder={$t('expressionForge.writeTransition') || 'Write transition or annotation...'}
							value={transitions[block.id] ?? ''}
							oninput={(e) => { transitions = { ...transitions, [block.id]: (e.target as HTMLTextAreaElement).value }; }}
							rows="2"
						></textarea>
					</div>
				{/each}
			{/if}
		</div>
	</div>
</div>

<style>
	.forge { display: flex; flex-direction: column; flex: 1; overflow: hidden; background: var(--background-primary); }
	.forge-header {
		display: flex; align-items: center; gap: 8px; padding: 8px 16px;
		border-bottom: 1px solid var(--background-modifier-border); flex-shrink: 0;
	}
	.forge-title {
		flex: 1; border: none; background: none; font-size: 1.1rem; font-weight: 600;
		color: var(--text-normal); font-family: inherit; outline: none; padding: 4px 0;
	}
	.forge-title::placeholder { color: var(--text-faint); }
	.forge-header-actions { display: flex; gap: 6px; align-items: center; }
	.forge-btn {
		padding: 4px 12px; border: 1px solid var(--background-modifier-border); border-radius: 4px;
		background: none; color: var(--text-normal); cursor: pointer; font-size: 0.8rem; font-family: inherit;
	}
	.forge-btn:hover { background: var(--background-modifier-hover); }
	.forge-export { color: var(--interactive-accent); border-color: var(--interactive-accent); }
	.forge-export:hover { background: var(--interactive-accent); color: white; }
	.forge-export:disabled { opacity: 0.3; cursor: default; }
	.forge-close { font-size: 1.2rem; padding: 2px 8px; }

	.forge-body { flex: 1; display: flex; overflow: hidden; }

	/* Left panel */
	.forge-left { width: 280px; flex-shrink: 0; display: flex; flex-direction: column; border-inline-end: 1px solid var(--background-modifier-border); }
	.forge-filters { padding: 8px; display: flex; flex-direction: column; gap: 6px; }
	.forge-filter-select {
		width: 100%; padding: 4px 8px; border: 1px solid var(--background-modifier-border);
		border-radius: 4px; background: var(--background-primary); color: var(--text-normal);
		font-size: 0.78rem; font-family: inherit;
	}
	.forge-search {
		width: 100%; padding: 4px 8px; border: 1px solid var(--background-modifier-border);
		border-radius: 4px; background: var(--background-primary); color: var(--text-normal);
		font-size: 0.78rem; font-family: inherit; outline: none;
	}
	.forge-note-list { flex: 1; overflow-y: auto; padding: 4px; }
	.forge-note-item {
		display: flex; align-items: center; gap: 6px; width: 100%; padding: 4px 8px;
		border: none; background: none; border-radius: 4px; cursor: pointer;
		font-size: 0.78rem; color: var(--text-normal); font-family: inherit; text-align: start;
	}
	.forge-note-item:hover { background: var(--background-modifier-hover); }
	.forge-note-item.added { opacity: 0.4; }
	.forge-note-stratum {
		width: 18px; height: 18px; display: flex; align-items: center; justify-content: center;
		font-size: 0.65rem; font-weight: 600; color: var(--text-faint);
		background: var(--background-secondary); border-radius: 3px; flex-shrink: 0;
	}
	.forge-note-name { overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
	.forge-empty { padding: 16px; text-align: center; font-size: 0.78rem; color: var(--text-faint); }

	.forge-divider { width: 3px; background: var(--background-modifier-border); cursor: col-resize; flex-shrink: 0; }
	.forge-divider:hover { background: var(--interactive-accent); }

	/* Canvas */
	.forge-canvas { flex: 1; overflow-y: auto; padding: 16px; min-width: 0; }
	.forge-canvas-empty { text-align: center; padding: 48px 24px; }
	.forge-canvas-empty-icon { font-size: 2.5rem; margin-bottom: 12px; }
	.forge-canvas-empty-text { font-size: 0.9rem; color: var(--text-muted); line-height: 1.5; }

	.forge-block {
		margin-bottom: 12px; border: 1px solid var(--background-modifier-border);
		border-radius: 6px; overflow: hidden;
	}
	.forge-block-header {
		display: flex; align-items: center; gap: 6px; padding: 6px 10px;
		background: var(--background-secondary); cursor: pointer;
	}
	.forge-block-toggle { border: none; background: none; cursor: pointer; color: var(--text-muted); font-size: 0.8rem; padding: 0 2px; }
	.forge-block-name { flex: 1; font-weight: 600; font-size: 0.85rem; color: var(--text-normal); overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
	.forge-block-actions { display: flex; gap: 2px; }
	.forge-block-btn {
		width: 22px; height: 22px; display: flex; align-items: center; justify-content: center;
		border: none; background: none; border-radius: 3px; cursor: pointer;
		font-size: 0.75rem; color: var(--text-muted);
	}
	.forge-block-btn:hover:not(:disabled) { background: var(--background-modifier-hover); color: var(--text-normal); }
	.forge-block-btn:disabled { opacity: 0.2; cursor: default; }
	.forge-block-remove:hover { color: var(--color-red, #ef4444); }
	.forge-block-content {
		padding: 10px 12px; font-size: 0.82rem; color: var(--text-muted);
		line-height: 1.6; max-height: 200px; overflow-y: auto;
		white-space: pre-wrap; border-top: 1px solid var(--background-modifier-border);
	}
	.forge-transition {
		width: 100%; box-sizing: border-box; border: none; border-top: 1px dashed var(--background-modifier-border);
		background: color-mix(in srgb, var(--interactive-accent) 3%, transparent);
		padding: 8px 12px; font-size: 0.82rem; font-family: inherit; color: var(--text-normal);
		resize: vertical; outline: none; line-height: 1.5;
	}
	.forge-transition::placeholder { color: var(--text-faint); font-style: italic; }
</style>
