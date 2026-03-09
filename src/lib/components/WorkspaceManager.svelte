<script lang="ts">
	import { workspaces, saveWorkspace, restoreWorkspace, deleteWorkspace, type Workspace } from '$lib/vaults/store';

	let {
		onClose,
		ar = false,
	}: {
		onClose: () => void;
		ar?: boolean;
	} = $props();

	let newName = $state('');
	let saving = $state(false);

	function handleSave() {
		const name = newName.trim();
		if (!name) return;
		saving = true;
		saveWorkspace(name);
		newName = '';
		saving = false;
	}

	async function handleRestore(ws: Workspace) {
		await restoreWorkspace(ws);
		onClose();
	}

	function handleDelete(id: string) {
		deleteWorkspace(id);
	}

	function formatDate(ts: number) {
		return new Date(ts).toLocaleDateString(undefined, {
			month: 'short', day: 'numeric', hour: '2-digit', minute: '2-digit'
		});
	}
</script>

<!-- svelte-ignore a11y_no_static_element_interactions -->
<div class="ws-overlay" onclick={onClose}>
	<!-- svelte-ignore a11y_click_events_have_key_events -->
	<!-- svelte-ignore a11y_no_static_element_interactions -->
	<div class="ws-panel" onclick={(e) => e.stopPropagation()}>
		<div class="ws-header">
			<span class="ws-title">{ar ? 'مساحات العمل' : 'Workspaces'}</span>
			<button class="ws-close" onclick={onClose}>×</button>
		</div>

		<div class="ws-save">
			<input
				type="text"
				bind:value={newName}
				placeholder={ar ? 'اسم مساحة العمل...' : 'Workspace name...'}
				onkeydown={(e) => { if (e.key === 'Enter') handleSave(); }}
			/>
			<button class="ws-save-btn" onclick={handleSave} disabled={!newName.trim() || saving}>
				{ar ? 'حفظ' : 'Save Current'}
			</button>
		</div>

		<div class="ws-list">
			{#if $workspaces.length === 0}
				<div class="ws-empty">{ar ? 'لا توجد مساحات عمل محفوظة' : 'No saved workspaces'}</div>
			{:else}
				{#each $workspaces as ws (ws.id)}
					<div class="ws-item">
						<div class="ws-item-info">
							<span class="ws-item-name">{ws.name}</span>
							<span class="ws-item-meta">{ws.tabs.length} {ar ? 'تبويبات' : 'tabs'} · {formatDate(ws.timestamp)}</span>
						</div>
						<div class="ws-item-actions">
							<button class="ws-restore" onclick={() => handleRestore(ws)} title={ar ? 'استعادة' : 'Restore'}>
								<svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M21 12a9 9 0 1 1-6.219-8.56"/><polyline points="22 2 22 8 16 8"/></svg>
							</button>
							<button class="ws-delete" onclick={() => handleDelete(ws.id)} title={ar ? 'حذف' : 'Delete'}>×</button>
						</div>
					</div>
				{/each}
			{/if}
		</div>
	</div>
</div>

<style>
	.ws-overlay {
		position: fixed; inset: 0; z-index: 1000;
		background: var(--background-modifier-cover); display: flex; align-items: flex-start; justify-content: center;
		padding-top: 15vh;
	}
	.ws-panel {
		background: var(--bg); border: 1px solid var(--border); border-radius: 8px;
		width: 400px; max-width: 90vw; box-shadow: var(--shadow-l);
		overflow: hidden;
	}
	.ws-header {
		display: flex; align-items: center; justify-content: space-between;
		padding: 12px 16px; border-bottom: 1px solid var(--border);
	}
	.ws-title { font-weight: 600; font-size: 0.95rem; }
	.ws-close {
		width: 24px; height: 24px; display: flex; align-items: center; justify-content: center;
		border: none; background: none; border-radius: 3px; color: var(--text-muted); cursor: pointer;
		font-size: 1.2rem;
	}
	.ws-close:hover { background: var(--bg-hover); color: var(--text); }

	.ws-save {
		display: flex; gap: 6px; padding: 10px 16px; border-bottom: 1px solid var(--border-light);
	}
	.ws-save input {
		flex: 1; padding: 6px 8px; border: 1px solid var(--border); border-radius: 4px;
		font-size: 0.82rem; background: var(--bg); color: var(--text); font-family: inherit;
		outline: none;
	}
	.ws-save input:focus { border-color: var(--accent); }
	.ws-save-btn {
		padding: 6px 12px; border: none; background: var(--accent); color: var(--text-on-accent);
		border-radius: 4px; font-size: 0.8rem; cursor: pointer; white-space: nowrap;
		font-family: inherit;
	}
	.ws-save-btn:hover { background: var(--accent-hover); }
	.ws-save-btn:disabled { opacity: 0.5; cursor: default; }

	.ws-list { max-height: 300px; overflow-y: auto; }
	.ws-empty { padding: 24px; text-align: center; color: var(--text-faint); font-size: 0.85rem; }

	.ws-item {
		display: flex; align-items: center; justify-content: space-between;
		padding: 8px 16px; border-bottom: 1px solid var(--border-light);
	}
	.ws-item:hover { background: var(--bg-hover); }
	.ws-item-info { display: flex; flex-direction: column; gap: 2px; min-width: 0; flex: 1; }
	.ws-item-name { font-size: 0.85rem; font-weight: 500; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
	.ws-item-meta { font-size: 0.72rem; color: var(--text-muted); }
	.ws-item-actions { display: flex; gap: 4px; flex-shrink: 0; }
	.ws-restore {
		width: 24px; height: 24px; display: flex; align-items: center; justify-content: center;
		border: none; background: none; border-radius: 3px; color: var(--accent); cursor: pointer;
	}
	.ws-restore:hover { background: var(--accent-bg); }
	.ws-delete {
		width: 24px; height: 24px; display: flex; align-items: center; justify-content: center;
		border: none; background: none; border-radius: 3px; color: var(--text-muted); cursor: pointer;
		font-size: 1rem;
	}
	.ws-delete:hover { background: var(--bg-hover); color: var(--danger); }
</style>
