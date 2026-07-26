<script lang="ts">
	import { t, tn, isRTL as isRTLStore } from '$lib/i18n';
	import { invoke } from '@tauri-apps/api/core';
	import { createNote, writeNote, openNoteTab, reindexNote } from '$lib/libraries/store';

	let {
		libraryPath = '',
		libraryName = '',
		libraryColor = '#7c3aed',
		onClose,
	}: {
		libraryPath?: string;
		libraryName?: string;
		libraryColor?: string;
		onClose?: () => void;
	} = $props();

	interface CanvasItem {
		id: string;
		type: string;
		x: number;
		y: number;
		width?: number;
		height?: number;
		content: string;
		quadrant: string | null;
		color?: string;
	}

	// ─── Canvas state ───
	let canvasPath = $state('');
	let canvasTitle = $state('');
	let items = $state<CanvasItem[]>([]);
	// Center the view on the quadrant grid (quadrants span -500,-500 to 500,500)
	// Initial offset calculated after mount to center in viewport
	let viewX = $state(0);
	let viewY = $state(0);
	let viewScale = $state(0.7); // zoom out slightly to show all quadrants
	let viewInitialized = false;
	let isPanning = $state(false);
	let panStartX = 0;
	let panStartY = 0;
	let panViewX = 0;
	let panViewY = 0;
	let draggingItem = $state<string | null>(null);
	let dragOffsetX = 0;
	let dragOffsetY = 0;
	let editingItem = $state<string | null>(null);
	let viewportEl: HTMLDivElement | undefined;
	let showCanvasPicker = $state(true);
	let availableCanvases = $state<any[]>([]);
	let newCanvasName = $state('');
	let saveTimer: ReturnType<typeof setTimeout> | null = null;

	// ─── Cynefin quadrants (positioned at fixed world coordinates) ───
	const QUADRANTS = [
		{ id: 'complex', x: -500, y: -500, color: 'rgba(124, 58, 237, 0.06)' },
		{ id: 'complicated', x: 0, y: -500, color: 'rgba(59, 130, 246, 0.06)' },
		{ id: 'chaotic', x: -500, y: 0, color: 'rgba(239, 68, 68, 0.06)' },
		{ id: 'clear', x: 0, y: 0, color: 'rgba(34, 197, 94, 0.06)' },
	];

	// ─── Load canvases ───
	async function loadCanvases() {
		if (!libraryPath) return;
		try {
			availableCanvases = await invoke('list_canvases', { libraryPath });
		} catch { availableCanvases = []; }
	}
	loadCanvases();

	// Center view on quadrant grid when viewport becomes visible
	$effect(() => {
		if (!showCanvasPicker && viewportEl && !viewInitialized) {
			viewInitialized = true;
			const w = viewportEl.clientWidth;
			const h = viewportEl.clientHeight;
			// Quadrant center is at (0, 0) in world space — offset view to center it
			viewX = w / 2;
			viewY = h / 2;
		}
	});

	async function openCanvas(path: string) {
		try {
			const data: any = await invoke('read_canvas', { canvasPath: path });
			canvasPath = path;
			canvasTitle = data.title || '';
			items = data.items || [];
			showCanvasPicker = false;
		} catch {}
	}

	async function createNewCanvas() {
		if (!newCanvasName.trim() || !libraryPath) return;
		try {
			const path: string = await invoke('create_canvas', { libraryPath, name: newCanvasName.trim() });
			canvasPath = path;
			canvasTitle = newCanvasName.trim();
			items = [];
			showCanvasPicker = false;
			newCanvasName = '';
		} catch {}
	}

	// ─── Settings ───
	let showSettings = $state(false);
	let quadrantColors = $state({
		complex: 'rgba(124, 58, 237, 0.06)',
		complicated: 'rgba(59, 130, 246, 0.06)',
		chaotic: 'rgba(239, 68, 68, 0.06)',
		clear: 'rgba(34, 197, 94, 0.06)',
	});
	const COLOR_PRESETS = [
		{ id: 'default', complex: 'rgba(124,58,237,0.06)', complicated: 'rgba(59,130,246,0.06)', chaotic: 'rgba(239,68,68,0.06)', clear: 'rgba(34,197,94,0.06)' },
		{ id: 'muted', complex: 'rgba(124,58,237,0.03)', complicated: 'rgba(59,130,246,0.03)', chaotic: 'rgba(239,68,68,0.03)', clear: 'rgba(34,197,94,0.03)' },
		{ id: 'bold', complex: 'rgba(124,58,237,0.12)', complicated: 'rgba(59,130,246,0.12)', chaotic: 'rgba(239,68,68,0.12)', clear: 'rgba(34,197,94,0.12)' },
		{ id: 'warm', complex: 'rgba(217,119,6,0.06)', complicated: 'rgba(245,158,11,0.06)', chaotic: 'rgba(239,68,68,0.06)', clear: 'rgba(16,185,129,0.06)' },
		{ id: 'invisible', complex: 'transparent', complicated: 'transparent', chaotic: 'transparent', clear: 'transparent' },
	];

	function fitToScreen() {
		if (!viewportEl) return;
		const w = viewportEl.clientWidth;
		const h = viewportEl.clientHeight;
		// Bounding box: quadrants span -500,-500 to 500,500 + any items outside
		let minX = -500, minY = -500, maxX = 500, maxY = 500;
		for (const item of items) {
			minX = Math.min(minX, item.x - 20);
			minY = Math.min(minY, item.y - 20);
			maxX = Math.max(maxX, item.x + 240);
			maxY = Math.max(maxY, item.y + 100);
		}
		const contentW = maxX - minX;
		const contentH = maxY - minY;
		const scale = Math.min(w / contentW, h / contentH) * 0.9; // 90% to add padding
		viewScale = Math.max(0.1, Math.min(5, scale));
		viewX = (w - contentW * viewScale) / 2 - minX * viewScale;
		viewY = (h - contentH * viewScale) / 2 - minY * viewScale;
	}

	function debouncedSave() {
		if (saveTimer) clearTimeout(saveTimer);
		saveTimer = setTimeout(async () => {
			if (!canvasPath) return;
			try {
				await invoke('write_canvas', { canvasPath, data: { title: canvasTitle, items } });
			} catch {}
		}, 1000);
	}

	// ─── Pan & Zoom ───
	function onWheel(e: WheelEvent) {
		e.preventDefault();
		const factor = e.deltaY > 0 ? 0.92 : 1.08;
		const rect = (e.currentTarget as HTMLElement).getBoundingClientRect();
		const mx = e.clientX - rect.left;
		const my = e.clientY - rect.top;
		const wx = (mx - viewX) / viewScale;
		const wy = (my - viewY) / viewScale;
		viewScale = Math.max(0.1, Math.min(5, viewScale * factor));
		viewX = mx - wx * viewScale;
		viewY = my - wy * viewScale;
	}

	function onPointerDown(e: PointerEvent) {
		if (e.button === 1 || (e.button === 0 && e.shiftKey)) {
			isPanning = true;
			panStartX = e.clientX;
			panStartY = e.clientY;
			panViewX = viewX;
			panViewY = viewY;
			(e.currentTarget as HTMLElement).setPointerCapture(e.pointerId);
			e.preventDefault();
		}
	}

	function onPointerMove(e: PointerEvent) {
		if (isPanning) {
			viewX = panViewX + (e.clientX - panStartX);
			viewY = panViewY + (e.clientY - panStartY);
		}
		if (draggingItem) {
			const item = items.find(i => i.id === draggingItem);
			if (item) {
				const rect = (e.currentTarget as HTMLElement).getBoundingClientRect();
				item.x = (e.clientX - rect.left - viewX) / viewScale - dragOffsetX;
				item.y = (e.clientY - rect.top - viewY) / viewScale - dragOffsetY;
				// Detect quadrant
				item.quadrant = detectQuadrant(item.x, item.y);
				items = [...items];
			}
		}
	}

	function onPointerUp(e: PointerEvent) {
		if (isPanning) { isPanning = false; }
		if (draggingItem) { draggingItem = null; debouncedSave(); }
	}

	function detectQuadrant(x: number, y: number): string | null {
		for (const q of QUADRANTS) {
			if (x >= q.x && x < q.x + 500 && y >= q.y && y < q.y + 500) return q.id;
		}
		return null;
	}

	// ─── Item operations ───
	function addItem(e: MouseEvent) {
		if (e.detail !== 2) return; // double-click only
		const rect = (e.currentTarget as HTMLElement).getBoundingClientRect();
		const x = (e.clientX - rect.left - viewX) / viewScale;
		const y = (e.clientY - rect.top - viewY) / viewScale;
		const newItem: CanvasItem = {
			id: `item_${Date.now()}_${Math.random().toString(36).slice(2, 6)}`,
			type: 'text',
			x, y,
			content: '',
			quadrant: detectQuadrant(x, y),
		};
		items = [...items, newItem];
		editingItem = newItem.id;
		debouncedSave();
	}

	function startDragItem(e: PointerEvent, item: CanvasItem) {
		if (editingItem === item.id) return;
		e.stopPropagation();
		draggingItem = item.id;
		const rect = (e.currentTarget as HTMLElement).closest('.smc-viewport')!.getBoundingClientRect();
		dragOffsetX = (e.clientX - rect.left - viewX) / viewScale - item.x;
		dragOffsetY = (e.clientY - rect.top - viewY) / viewScale - item.y;
	}

	function deleteItem(id: string) {
		items = items.filter(i => i.id !== id);
		debouncedSave();
	}

	// ─── Promote dialog state ───
	let promoteDialogItem = $state<CanvasItem | null>(null);
	let promoteNoteName = $state('');
	let promoteSelectedLib = $state({ name: libraryName, path: libraryPath, color: libraryColor });
	let promoteLibraries = $state<{ name: string; path: string; color: string }[]>([]);

	async function startPromote(item: CanvasItem) {
		promoteDialogItem = item;
		promoteNoteName = item.content.trim().slice(0, 100).replace(/[<>:"/\\|?*\n]/g, '_');
		promoteSelectedLib = { name: libraryName, path: libraryPath, color: libraryColor };
		// Load all libraries
		try {
			const libs: any[] = await invoke('resolve_universe_libraries');
			promoteLibraries = libs.map((l: any) => ({ name: l.name, path: l.path, color: '#7c3aed' }));
		} catch {
			promoteLibraries = [{ name: libraryName, path: libraryPath, color: libraryColor }];
		}
	}

	async function confirmPromote() {
		const item = promoteDialogItem;
		if (!item || !promoteNoteName.trim()) return;
		const targetPath = promoteSelectedLib.path;
		const targetName = promoteSelectedLib.name;
		const targetColor = promoteSelectedLib.color;
		const fileName = promoteNoteName.trim() + '.md';
		// MIG-014 §2F — canvas-promoted note is a defined-concept-in-progress → growth
		// (was `permanent` from the dropped Zettelkasten model).
		const frontmatter = `---\nstage: growth\ncanvas_origin: "${canvasTitle}"\n${item.quadrant ? `canvas_quadrant: ${item.quadrant}\n` : ''}---\n`;
		try {
			const newPath = await createNote(targetPath, fileName);
			await writeNote(newPath, frontmatter + item.content, 'canvas_export');
			// Whole-Ecosystem (PJ-140): createNote indexed the empty stub; reindex the promoted body
			// so it is searchable/backlinked at once, not only after a boot reindex (index-divergence class).
			reindexNote(newPath, targetName).catch((e) => console.error('[canvas_export] reindex failed:', e));
			const noteName = promoteNoteName.trim();
			item.content = `[[${noteName}]]`;
			item.type = 'link';
			items = [...items];
			debouncedSave();
			promoteDialogItem = null;
			await openNoteTab(newPath, targetName, targetColor);
		} catch (e) {
			console.error('Promote failed:', e);
		}
	}

	function updateItemContent(id: string, content: string) {
		const item = items.find(i => i.id === id);
		if (item) { item.content = content; items = [...items]; debouncedSave(); }
	}
</script>

<div class="smc" dir={$isRTLStore ? 'rtl' : 'ltr'}>
	{#if showCanvasPicker}
		<!-- Canvas picker -->
		<div class="smc-picker">
			<div class="smc-picker-header">
				<span class="smc-picker-title">{$t('senseMakingCanvas.title') || 'Sense-Making Canvas'}</span>
				<button class="smc-close" onclick={() => onClose?.()}>×</button>
			</div>
			<div class="smc-picker-body">
				{#if availableCanvases.length > 0}
					<div class="smc-picker-label">{$t('senseMakingCanvas.openExisting') || 'Open existing canvas:'}</div>
					{#each availableCanvases as c}
						<button class="smc-picker-item" onclick={() => openCanvas(c.path)}>🎨 {c.name}</button>
					{/each}
				{/if}
				<div class="smc-picker-label" style="margin-top:12px;">{$t('senseMakingCanvas.createNew') || 'Create new canvas:'}</div>
				<div class="smc-picker-create">
					<input class="smc-picker-input" type="text" dir="auto" placeholder={$t('senseMakingCanvas.canvasName') || 'Canvas name...'} bind:value={newCanvasName} onkeydown={(e) => e.key === 'Enter' && createNewCanvas()} />
					<button class="smc-picker-btn" onclick={createNewCanvas}>{$t('settings.knowledge.create') || 'Create'}</button>
				</div>
			</div>
		</div>
	{:else}
		<!-- Canvas workspace -->
		<div class="smc-header">
			<span class="smc-header-title">🎨 {canvasTitle}</span>
			<span class="smc-header-count">{$tn('plurals.items', items.length)}</span>
			<span class="smc-header-hint">{$t('senseMakingCanvas.hint') || 'Double-click to add • Shift+drag to pan • Scroll to zoom'}</span>
			<button class="smc-header-btn" title="Fit to screen" onclick={fitToScreen}>
				<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M8 3H5a2 2 0 0 0-2 2v3M21 8V5a2 2 0 0 0-2-2h-3M3 16v3a2 2 0 0 0 2 2h3M16 21h3a2 2 0 0 0 2-2v-3"/></svg>
			</button>
			<button class="smc-header-btn" title="Settings" onclick={() => showSettings = !showSettings}>
				<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="12" cy="12" r="3"/><path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1-2.83 2.83l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-4 0v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83-2.83l.06-.06A1.65 1.65 0 0 0 4.68 15a1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1 0-4h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 2.83-2.83l.06.06A1.65 1.65 0 0 0 9 4.68a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 4 0v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 2.83l-.06.06A1.65 1.65 0 0 0 19.4 9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 0 4h-.09a1.65 1.65 0 0 0-1.51 1z"/></svg>
			</button>
			<button class="smc-header-btn" onclick={() => { showCanvasPicker = true; }}>{$t('senseMakingCanvas.switchCanvas') || 'Switch'}</button>
			<button class="smc-header-btn smc-close" onclick={() => onClose?.()}>×</button>
		</div>
		{#if showSettings}
			<div class="smc-settings">
				<div class="smc-settings-title">{$t('senseMakingCanvas.colorPresets') || 'Zone Color Presets'}</div>
				<div class="smc-settings-presets">
					{#each COLOR_PRESETS as preset}
						<button class="smc-preset-btn" onclick={() => {
							quadrantColors = { complex: preset.complex, complicated: preset.complicated, chaotic: preset.chaotic, clear: preset.clear };
						}}>
							<div class="smc-preset-dots">
								<span style="background:{preset.complex}; border:1px solid rgba(0,0,0,0.1);"></span>
								<span style="background:{preset.complicated}; border:1px solid rgba(0,0,0,0.1);"></span>
								<span style="background:{preset.chaotic}; border:1px solid rgba(0,0,0,0.1);"></span>
								<span style="background:{preset.clear}; border:1px solid rgba(0,0,0,0.1);"></span>
							</div>
							<span class="smc-preset-name">{$t(`senseMakingCanvas.preset_${preset.id}`) || preset.id}</span>
						</button>
					{/each}
				</div>
			</div>
		{/if}
		<!-- svelte-ignore a11y_no_static_element_interactions -->
		<div class="smc-viewport" bind:this={viewportEl}
			onwheel={onWheel}
			onpointerdown={onPointerDown}
			onpointermove={onPointerMove}
			onpointerup={onPointerUp}
			ondblclick={addItem}
		>
			<div class="smc-world" style="transform: translate({viewX}px, {viewY}px) scale({viewScale})">
				<!-- Cynefin quadrants -->
				{#each QUADRANTS as q}
					<div class="smc-quadrant" style="left:{q.x}px; top:{q.y}px; width:500px; height:500px; background:{quadrantColors[q.id as keyof typeof quadrantColors] ?? q.color};">
						<span class="smc-quadrant-label">{$t(`senseMakingCanvas.q_${q.id}`) || q.id}</span>
					</div>
				{/each}

				<!-- Items -->
				{#each items as item (item.id)}
					<!-- svelte-ignore a11y_no_static_element_interactions -->
					<div class="smc-item" class:editing={editingItem === item.id} class:dragging={draggingItem === item.id}
						style="left:{item.x}px; top:{item.y}px;"
						onpointerdown={(e) => startDragItem(e, item)}
					>
						<div class="smc-item-header" style="background:{item.quadrant === 'complex' ? 'rgba(124,58,237,0.15)' : item.quadrant === 'complicated' ? 'rgba(59,130,246,0.15)' : item.quadrant === 'chaotic' ? 'rgba(239,68,68,0.15)' : item.quadrant === 'clear' ? 'rgba(34,197,94,0.15)' : 'transparent'}">
							{#if item.quadrant}
								<span class="smc-item-quad">{$t(`senseMakingCanvas.q_${item.quadrant}`) || item.quadrant}</span>
							{/if}
							<div class="smc-item-actions">
								<button class="smc-item-btn" title={$t('senseMakingCanvas.promote') || 'Promote to note'} onclick={() => startPromote(item)}>🔗</button>
								<button class="smc-item-btn" onclick={() => { editingItem = editingItem === item.id ? null : item.id; }}>✏️</button>
								<button class="smc-item-btn smc-item-del" onclick={() => deleteItem(item.id)}>×</button>
							</div>
						</div>
						{#if editingItem === item.id}
							<textarea class="smc-item-edit" dir="auto" value={item.content}
								oninput={(e) => updateItemContent(item.id, (e.target as HTMLTextAreaElement).value)}
								onkeydown={(e) => { if (e.key === 'Escape') editingItem = null; }}
								rows="4"
							></textarea>
						{:else}
							<!-- svelte-ignore a11y_no_static_element_interactions -->
							<div class="smc-item-content" dir="auto" ondblclick={() => editingItem = item.id}>
								{item.content || '(empty — double-click to edit)'}
							</div>
						{/if}
					</div>
				{/each}
			</div>
		</div>
	{/if}

	<!-- Promote dialog -->
	{#if promoteDialogItem}
		<!-- svelte-ignore a11y_no_static_element_interactions -->
		<div class="smc-promote-overlay" onclick={() => promoteDialogItem = null}>
			<!-- svelte-ignore a11y_no_static_element_interactions -->
			<div class="smc-promote-dialog" onclick={(e) => e.stopPropagation()}>
				<div class="smc-promote-title">Promote to Note</div>
				<div class="smc-promote-preview" dir="auto">"{promoteDialogItem.content.slice(0, 80)}{promoteDialogItem.content.length > 80 ? '...' : ''}"</div>
				<label class="smc-promote-label">
					<span>Note name</span>
					<input class="smc-promote-input" type="text" dir="auto" bind:value={promoteNoteName}
						onkeydown={(e) => e.key === 'Enter' && confirmPromote()} />
				</label>
				<div class="smc-promote-label">
					<span>Select library</span>
					<div class="smc-promote-libs">
						{#each promoteLibraries as lib}
							<button class="smc-promote-lib" class:selected={promoteSelectedLib.path === lib.path}
								onclick={() => promoteSelectedLib = lib}>
								<span class="smc-promote-lib-dot" style="background:{lib.color}"></span>
								<span dir="auto">{lib.name}</span>
							</button>
						{/each}
					</div>
				</div>
				<div class="smc-promote-actions">
					<button class="smc-promote-btn primary" onclick={confirmPromote}>Create Note</button>
					<button class="smc-promote-btn" onclick={() => promoteDialogItem = null}>Cancel</button>
				</div>
			</div>
		</div>
	{/if}
</div>

<style>
	.smc { display: flex; flex-direction: column; flex: 1; overflow: hidden; background: #f8f8fc; }
	.smc-header {
		display: flex; align-items: center; gap: 8px; padding: 6px 16px;
		border-bottom: 1px solid var(--background-modifier-border); background: var(--background-primary);
		flex-shrink: 0; font-size: 0.82rem;
	}
	.smc-header-title { font-weight: 600; color: var(--text-normal); }
	.smc-header-count { color: var(--text-faint); }
	.smc-header-hint { color: var(--text-faint); font-size: 0.72rem; margin-inline-start: auto; }
	.smc-header-btn {
		padding: 3px 10px; border: 1px solid var(--background-modifier-border); border-radius: 4px;
		background: none; cursor: pointer; font-size: 0.78rem; color: var(--text-muted); font-family: inherit;
	}
	.smc-header-btn:hover { background: var(--background-modifier-hover); }
	.smc-close { font-size: 1.1rem; padding: 2px 8px; }

	/* Settings panel */
	.smc-settings {
		padding: 8px 16px; background: var(--background-secondary);
		border-bottom: 1px solid var(--background-modifier-border);
		display: flex; align-items: center; gap: 12px; flex-shrink: 0;
	}
	.smc-settings-title { font-size: 0.75rem; font-weight: 600; color: var(--text-muted); }
	.smc-settings-presets { display: flex; gap: 6px; }
	.smc-preset-btn {
		display: flex; align-items: center; gap: 4px; padding: 4px 8px;
		border: 1px solid var(--background-modifier-border); border-radius: 4px;
		background: var(--background-primary); cursor: pointer; font-size: 0.7rem;
		font-family: inherit; color: var(--text-normal);
	}
	.smc-preset-btn:hover { border-color: var(--interactive-accent); }
	.smc-preset-dots { display: flex; gap: 2px; }
	.smc-preset-dots span { width: 10px; height: 10px; border-radius: 2px; display: block; }
	.smc-preset-name { font-size: 0.68rem; }

	/* Promote dialog */
	.smc-promote-overlay {
		position: fixed; inset: 0; z-index: 200;
		background: rgba(0,0,0,0.3); display: flex; align-items: center; justify-content: center;
	}
	.smc-promote-dialog {
		background: var(--background-primary); border-radius: 12px;
		box-shadow: 0 12px 40px rgba(0,0,0,0.2); padding: 20px 24px;
		width: 400px; max-width: 90vw;
	}
	.smc-promote-title { font-weight: 700; font-size: 1rem; margin-bottom: 8px; color: var(--text-normal); }
	.smc-promote-preview {
		font-size: 0.78rem; color: var(--text-muted); font-style: italic;
		margin-bottom: 16px; padding: 8px; background: var(--background-secondary);
		border-radius: 6px; line-height: 1.4;
	}
	.smc-promote-label {
		display: flex; flex-direction: column; gap: 4px; margin-bottom: 12px;
		font-size: 0.78rem; color: var(--text-muted); font-weight: 500;
	}
	.smc-promote-input {
		padding: 6px 10px; border: 1px solid var(--background-modifier-border);
		border-radius: 6px; font-size: 0.85rem; font-family: inherit;
		outline: none; color: var(--text-normal); background: var(--background-primary);
	}
	.smc-promote-input:focus { border-color: var(--interactive-accent); }
	.smc-promote-actions { display: flex; gap: 8px; justify-content: flex-end; margin-top: 4px; }
	.smc-promote-btn {
		padding: 6px 16px; border: 1px solid var(--background-modifier-border); border-radius: 6px;
		background: none; cursor: pointer; font-size: 0.82rem; font-family: inherit;
		color: var(--text-normal);
	}
	.smc-promote-btn:hover { background: var(--background-modifier-hover); }
	.smc-promote-btn.primary {
		background: var(--interactive-accent); color: white; border-color: var(--interactive-accent);
	}
	.smc-promote-btn.primary:hover { opacity: 0.9; }
	.smc-promote-libs { display: flex; flex-direction: column; gap: 4px; }
	.smc-promote-lib {
		display: flex; align-items: center; gap: 8px; padding: 8px 12px;
		border: 1px solid var(--background-modifier-border); border-radius: 6px;
		background: var(--background-primary); cursor: pointer; font-size: 0.85rem;
		font-family: inherit; color: var(--text-normal); text-align: start;
	}
	.smc-promote-lib:hover { background: var(--background-modifier-hover); }
	.smc-promote-lib.selected {
		border-color: var(--interactive-accent);
		background: color-mix(in srgb, var(--interactive-accent) 8%, var(--background-primary));
	}
	.smc-promote-lib-dot { width: 10px; height: 10px; border-radius: 50%; flex-shrink: 0; }

	.smc-viewport { flex: 1; overflow: hidden; position: relative; cursor: crosshair; }
	.smc-world { position: absolute; top: 0; left: 0; transform-origin: 0 0; }

	/* Cynefin quadrants */
	.smc-quadrant {
		position: absolute; border: 1px dashed rgba(0,0,0,0.08); border-radius: 8px;
	}
	.smc-quadrant-label {
		position: absolute; top: 8px; inset-inline-start: 12px;
		font-size: 14px; font-weight: 600; color: rgba(0,0,0,0.12);
		pointer-events: none; user-select: none;
	}

	/* Items — Post-it sticky note style */
	.smc-item {
		position: absolute; width: 200px; min-height: 80px;
		background: #fff9c4; /* Post-it yellow */
		border: none; border-radius: 2px;
		box-shadow: 2px 3px 8px rgba(0,0,0,0.15), 0 1px 2px rgba(0,0,0,0.1);
		cursor: grab;
		transition: box-shadow 0.15s, transform 0.1s;
		transform: rotate(-1deg); /* slight tilt like a real sticky note */
	}
	.smc-item:nth-child(even) { transform: rotate(0.5deg); }
	.smc-item:nth-child(3n) { transform: rotate(-0.8deg); }
	.smc-item:hover {
		box-shadow: 4px 6px 20px rgba(0,0,0,0.2);
		transform: rotate(0deg) scale(1.02);
		z-index: 10;
	}
	.smc-item.dragging {
		opacity: 0.85; cursor: grabbing;
		box-shadow: 8px 10px 30px rgba(0,0,0,0.25);
		transform: rotate(0deg) scale(1.05);
		z-index: 100;
	}
	.smc-item.editing { cursor: auto; transform: rotate(0deg); }
	.smc-item-header {
		display: flex; align-items: center; gap: 4px; padding: 4px 8px;
		border-bottom: 1px dashed rgba(0,0,0,0.08); font-size: 0.68rem;
	}
	.smc-item-quad {
		color: var(--text-faint); font-weight: 500; text-transform: capitalize;
		background: var(--background-secondary); border-radius: 3px; padding: 0 4px;
	}
	.smc-item-actions { display: flex; gap: 2px; margin-inline-start: auto; }
	.smc-item-btn {
		width: 20px; height: 20px; display: flex; align-items: center; justify-content: center;
		border: none; background: none; border-radius: 3px; cursor: pointer; font-size: 0.7rem;
		color: var(--text-muted); opacity: 0;
	}
	.smc-item:hover .smc-item-btn { opacity: 1; }
	.smc-item-btn:hover { background: var(--background-modifier-hover); }
	.smc-item-del:hover { color: #ef4444; }
	.smc-item-content {
		padding: 10px 12px; font-size: 0.82rem; color: #333;
		white-space: pre-wrap; line-height: 1.5; cursor: text; min-height: 40px;
		font-family: 'Segoe UI', system-ui, sans-serif;
	}
	.smc-item-edit {
		width: 100%; box-sizing: border-box; border: none; padding: 10px 12px;
		font-size: 0.82rem; font-family: 'Segoe UI', system-ui, sans-serif;
		resize: vertical; outline: none;
		line-height: 1.5; color: #333; background: rgba(255,255,255,0.3);
		border-radius: 0 0 2px 2px;
	}

	/* Picker */
	.smc-picker {
		max-width: 400px; margin: 80px auto; background: var(--background-primary);
		border: 1px solid var(--background-modifier-border); border-radius: 12px;
		box-shadow: 0 8px 32px rgba(0,0,0,0.12); overflow: hidden;
	}
	.smc-picker-header {
		display: flex; align-items: center; padding: 12px 16px;
		border-bottom: 1px solid var(--background-modifier-border);
	}
	.smc-picker-title { flex: 1; font-weight: 600; font-size: 1rem; }
	.smc-picker-body { padding: 16px; }
	.smc-picker-label { font-size: 0.78rem; font-weight: 600; color: var(--text-muted); margin-bottom: 6px; }
	.smc-picker-item {
		display: block; width: 100%; padding: 8px 12px; border: 1px solid var(--background-modifier-border);
		border-radius: 6px; background: none; cursor: pointer; font-size: 0.85rem;
		color: var(--text-normal); font-family: inherit; text-align: start; margin-bottom: 4px;
	}
	.smc-picker-item:hover { background: var(--background-modifier-hover); }
	.smc-picker-create { display: flex; gap: 6px; }
	.smc-picker-input {
		flex: 1; padding: 6px 10px; border: 1px solid var(--background-modifier-border);
		border-radius: 6px; font-size: 0.85rem; font-family: inherit; outline: none;
	}
	.smc-picker-btn {
		padding: 6px 16px; border: none; border-radius: 6px;
		background: var(--interactive-accent); color: white; cursor: pointer;
		font-size: 0.85rem; font-family: inherit;
	}
	.smc-picker-btn:hover { opacity: 0.9; }
</style>
