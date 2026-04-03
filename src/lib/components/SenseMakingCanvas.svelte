<script lang="ts">
	import { t } from '$lib/i18n';
	import { invoke } from '@tauri-apps/api/core';
	import { createNote, writeNote, openNoteTab } from '$lib/libraries/store';

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
		{ id: 'complex', label: 'Complex', labelAr: 'معقد', x: -500, y: -500, color: 'rgba(124, 58, 237, 0.06)' },
		{ id: 'complicated', label: 'Complicated', labelAr: 'مُعَقَّد', x: 0, y: -500, color: 'rgba(59, 130, 246, 0.06)' },
		{ id: 'chaotic', label: 'Chaotic', labelAr: 'فوضوي', x: -500, y: 0, color: 'rgba(239, 68, 68, 0.06)' },
		{ id: 'clear', label: 'Clear', labelAr: 'واضح', x: 0, y: 0, color: 'rgba(34, 197, 94, 0.06)' },
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

	async function promoteItem(item: CanvasItem) {
		if (!libraryPath || !item.content.trim()) return;
		const fileName = item.content.trim().slice(0, 40).replace(/[<>:"/\\|?*\n]/g, '_') + '.md';
		const frontmatter = `---\nstage: permanent\ncanvas_origin: "${canvasTitle}"\n${item.quadrant ? `canvas_quadrant: ${item.quadrant}\n` : ''}---\n`;
		try {
			const newPath = await createNote(libraryPath, fileName);
			await writeNote(newPath, frontmatter + item.content);
			// Replace item content with wikilink
			const noteName = fileName.replace(/\.md$/, '');
			item.content = `[[${noteName}]]`;
			item.type = 'link';
			items = [...items];
			debouncedSave();
			await openNoteTab(newPath, libraryName, libraryColor);
		} catch {}
	}

	function updateItemContent(id: string, content: string) {
		const item = items.find(i => i.id === id);
		if (item) { item.content = content; items = [...items]; debouncedSave(); }
	}
</script>

<div class="smc">
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
			<span class="smc-header-count">{items.length} {$t('senseMakingCanvas.items') || 'items'}</span>
			<span class="smc-header-hint">{$t('senseMakingCanvas.hint') || 'Double-click to add • Shift+drag to pan • Scroll to zoom'}</span>
			<button class="smc-header-btn" onclick={() => { showCanvasPicker = true; }}>{$t('senseMakingCanvas.switchCanvas') || 'Switch'}</button>
			<button class="smc-header-btn smc-close" onclick={() => onClose?.()}>×</button>
		</div>
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
					<div class="smc-quadrant" style="left:{q.x}px; top:{q.y}px; width:500px; height:500px; background:{q.color};">
						<span class="smc-quadrant-label">{q.label}</span>
					</div>
				{/each}

				<!-- Items -->
				{#each items as item (item.id)}
					<!-- svelte-ignore a11y_no_static_element_interactions -->
					<div class="smc-item" class:editing={editingItem === item.id} class:dragging={draggingItem === item.id}
						style="left:{item.x}px; top:{item.y}px;"
						onpointerdown={(e) => startDragItem(e, item)}
					>
						<div class="smc-item-header">
							{#if item.quadrant}
								<span class="smc-item-quad">{item.quadrant}</span>
							{/if}
							<div class="smc-item-actions">
								<button class="smc-item-btn" title={$t('senseMakingCanvas.promote') || 'Promote to note'} onclick={() => promoteItem(item)}>🔗</button>
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

	.smc-viewport { flex: 1; overflow: hidden; position: relative; cursor: crosshair; }
	.smc-world { position: absolute; top: 0; left: 0; transform-origin: 0 0; }

	/* Cynefin quadrants */
	.smc-quadrant {
		position: absolute; border: 1px dashed rgba(0,0,0,0.08); border-radius: 8px;
	}
	.smc-quadrant-label {
		position: absolute; top: 8px; left: 12px;
		font-size: 14px; font-weight: 600; color: rgba(0,0,0,0.12);
		pointer-events: none; user-select: none;
	}

	/* Items */
	.smc-item {
		position: absolute; width: 220px; min-height: 60px;
		background: white; border: 1px solid #ddd; border-radius: 8px;
		box-shadow: 0 2px 8px rgba(0,0,0,0.08); cursor: grab;
		transition: box-shadow 0.15s;
	}
	.smc-item:hover { box-shadow: 0 4px 16px rgba(0,0,0,0.12); }
	.smc-item.dragging { opacity: 0.7; cursor: grabbing; box-shadow: 0 8px 24px rgba(0,0,0,0.18); }
	.smc-item.editing { cursor: auto; }
	.smc-item-header {
		display: flex; align-items: center; gap: 4px; padding: 4px 8px;
		border-bottom: 1px solid #eee; font-size: 0.68rem;
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
		padding: 8px 10px; font-size: 0.82rem; color: var(--text-normal);
		white-space: pre-wrap; line-height: 1.4; cursor: text; min-height: 30px;
	}
	.smc-item-edit {
		width: 100%; box-sizing: border-box; border: none; padding: 8px 10px;
		font-size: 0.82rem; font-family: inherit; resize: vertical; outline: none;
		line-height: 1.4; color: var(--text-normal); background: #fafafa;
		border-radius: 0 0 8px 8px;
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
