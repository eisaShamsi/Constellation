<script lang="ts">
	import type { FrontmatterProperty, PropertyType } from '$lib/vaults/store';
	import { saveTabContent } from '$lib/vaults/store';

	let {
		properties,
		body,
		tabId,
		filePath,
		ar = false
	}: {
		properties: FrontmatterProperty[];
		body: string;
		tabId: string;
		filePath: string;
		ar?: boolean;
	} = $props();

	const TYPE_ICONS: Record<PropertyType, string> = {
		text: '\u2261',
		number: '#',
		date: '\uD83D\uDCC5',
		list: '\u2255',
		link: '\uD83D\uDD17'
	};

	const TYPE_ORDER: PropertyType[] = ['text', 'number', 'date', 'list', 'link'];

	let editableProps = $state<FrontmatterProperty[]>([]);
	let saveTimeout: ReturnType<typeof setTimeout>;
	let saving = $state(false);
	let prevTabId = $state('');
	let tagInputs = $state<Record<number, string>>({});

	// Sync from props when tab changes
	$effect(() => {
		if (tabId !== prevTabId) {
			editableProps = properties.map(p => ({
				...p,
				listItems: p.listItems ? [...p.listItems] : undefined
			}));
			prevTabId = tabId;
			tagInputs = {};
		}
	});

	function cycleType(idx: number) {
		const current = editableProps[idx].type;
		const nextIdx = (TYPE_ORDER.indexOf(current) + 1) % TYPE_ORDER.length;
		const newType = TYPE_ORDER[nextIdx];
		editableProps = editableProps.map((p, i) => {
			if (i !== idx) return p;
			const updated = { ...p, type: newType };
			if (newType === 'list' && !updated.listItems) {
				updated.listItems = updated.value ? updated.value.split(',').map(s => s.trim()).filter(Boolean) : [];
				updated.value = updated.listItems.join(', ');
			} else if (newType === 'link' && !updated.value.startsWith('[[')) {
				updated.value = updated.value ? `[[${updated.value}]]` : '[[]]';
			} else if (newType !== 'list') {
				// If switching away from list, flatten listItems back to value
				if (p.type === 'list' && p.listItems) {
					updated.value = p.listItems.join(', ');
				}
				updated.listItems = undefined;
			}
			return updated;
		});
		debouncedSave();
	}

	function updateKey(idx: number, newKey: string) {
		editableProps = editableProps.map((p, i) =>
			i === idx ? { ...p, key: newKey } : p
		);
		debouncedSave();
	}

	function updateValue(idx: number, newValue: string) {
		editableProps = editableProps.map((p, i) =>
			i === idx ? { ...p, value: newValue } : p
		);
		debouncedSave();
	}

	function addProperty() {
		editableProps = [...editableProps, { key: '', value: '', type: 'text' }];
	}

	function removeProperty(idx: number) {
		editableProps = editableProps.filter((_, i) => i !== idx);
		debouncedSave();
	}

	function addTag(idx: number, tag: string) {
		if (!tag.trim()) return;
		editableProps = editableProps.map((p, i) => {
			if (i !== idx) return p;
			const items = [...(p.listItems ?? []), tag.trim()];
			return { ...p, listItems: items, value: items.join(', ') };
		});
		tagInputs = { ...tagInputs, [idx]: '' };
		debouncedSave();
	}

	function removeTag(propIdx: number, tagIdx: number) {
		editableProps = editableProps.map((p, i) => {
			if (i !== propIdx) return p;
			const items = (p.listItems ?? []).filter((_, ti) => ti !== tagIdx);
			return { ...p, listItems: items, value: items.join(', ') };
		});
		debouncedSave();
	}

	function handleTagKeydown(e: KeyboardEvent, idx: number) {
		if (e.key === 'Enter' || e.key === ',') {
			e.preventDefault();
			const val = tagInputs[idx] ?? '';
			addTag(idx, val.replace(',', ''));
		}
	}

	function debouncedSave() {
		clearTimeout(saveTimeout);
		saveTimeout = setTimeout(async () => {
			saving = true;
			try {
				await saveTabContent(tabId, filePath, editableProps, body);
			} catch (err) {
				console.error('Failed to save:', err);
			}
			saving = false;
		}, 800);
	}
</script>

<div class="property-editor">
	<div class="pe-header">
		<span class="pe-title">{ar ? '\u0627\u0644\u062E\u0635\u0627\u0626\u0635' : 'Properties'}</span>
		{#if saving}
			<span class="pe-saving">{ar ? '\u062C\u0627\u0631\u064D \u0627\u0644\u062D\u0641\u0638...' : 'Saving...'}</span>
		{/if}
	</div>

	{#each editableProps as prop, idx}
		<div class="pe-row">
			<button class="pe-type-btn" title={prop.type} onclick={() => cycleType(idx)}>
				{TYPE_ICONS[prop.type]}
			</button>
			<input class="pe-key" type="text" value={prop.key}
				placeholder={ar ? '\u0645\u0641\u062A\u0627\u062D' : 'Key'}
				oninput={(e) => updateKey(idx, (e.target as HTMLInputElement).value)} />

			{#if prop.type === 'date'}
				<input class="pe-val" type="date" value={prop.value}
					oninput={(e) => updateValue(idx, (e.target as HTMLInputElement).value)} />
			{:else if prop.type === 'number'}
				<input class="pe-val" type="number" value={prop.value}
					oninput={(e) => updateValue(idx, (e.target as HTMLInputElement).value)} />
			{:else if prop.type === 'list'}
				<div class="pe-tags">
					{#each prop.listItems ?? [] as tag, tagIdx}
						<span class="pe-tag">
							{tag}
							<button class="pe-tag-x" onclick={() => removeTag(idx, tagIdx)}>\u00d7</button>
						</span>
					{/each}
					<input class="pe-tag-input" type="text"
						placeholder={ar ? '\u0623\u0636\u0641...' : 'Add...'}
						value={tagInputs[idx] ?? ''}
						oninput={(e) => { tagInputs = { ...tagInputs, [idx]: (e.target as HTMLInputElement).value }; }}
						onkeydown={(e) => handleTagKeydown(e, idx)} />
				</div>
			{:else if prop.type === 'link'}
				<div class="pe-link-wrap">
					<span class="pe-link-bracket">[[</span>
					<input class="pe-val pe-link-input" type="text"
						value={prop.value.replace(/^\[\[|\]\]$/g, '')}
						oninput={(e) => updateValue(idx, `[[${(e.target as HTMLInputElement).value}]]`)} />
					<span class="pe-link-bracket">]]</span>
				</div>
			{:else}
				<input class="pe-val" type="text" value={prop.value}
					placeholder={ar ? '\u0642\u064A\u0645\u0629' : 'Value'}
					oninput={(e) => updateValue(idx, (e.target as HTMLInputElement).value)} />
			{/if}

			<button class="pe-del" onclick={() => removeProperty(idx)} title={ar ? '\u062D\u0630\u0641' : 'Delete'}>
				<svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M18 6L6 18M6 6l12 12"/></svg>
			</button>
		</div>
	{/each}

	<button class="pe-add" onclick={addProperty}>
		+ {ar ? '\u0625\u0636\u0627\u0641\u0629 \u062E\u0627\u0635\u064A\u0629' : 'Add property'}
	</button>
</div>

<style>
	.property-editor {
		background: var(--background-primary-alt); border: 1px solid var(--background-modifier-border-focus); border-radius: 6px;
		padding: 10px 14px; margin-bottom: 4px;
	}

	.pe-header {
		display: flex; align-items: center; justify-content: space-between;
		margin-bottom: 8px;
	}
	.pe-title { font-size: 0.78rem; font-weight: 600; color: var(--text-muted); }
	.pe-saving { font-size: 0.7rem; color: var(--interactive-accent); }

	.pe-row {
		display: flex; align-items: center; gap: 6px;
		padding: 4px 0; border-bottom: 1px solid var(--background-secondary-alt);
	}
	.pe-row:last-of-type { border-bottom: none; }

	.pe-type-btn {
		width: 24px; height: 24px; flex-shrink: 0;
		display: flex; align-items: center; justify-content: center;
		border: none; background: none; border-radius: 3px;
		color: var(--text-faint); cursor: pointer; font-size: 0.85rem;
		padding: 0;
	}
	.pe-type-btn:hover { background: var(--background-modifier-border); color: var(--text-muted); }

	.pe-key {
		width: 90px; min-width: 60px; flex-shrink: 0;
		border: none; background: none; padding: 3px 4px;
		font-size: 0.82rem; font-weight: 500; color: var(--text-muted);
		font-family: inherit; outline: none;
		border-radius: 3px; text-align: end;
	}
	.pe-key:focus { background: var(--background-primary); box-shadow: 0 0 0 1px hsla(var(--accent-h), var(--accent-s), var(--accent-l), 0.27); }

	.pe-val {
		flex: 1; min-width: 0;
		border: none; background: none; padding: 3px 6px;
		font-size: 0.82rem; color: var(--text-normal);
		font-family: inherit; outline: none;
		border-radius: 3px;
	}
	.pe-val:focus { background: var(--background-primary); box-shadow: 0 0 0 1px hsla(var(--accent-h), var(--accent-s), var(--accent-l), 0.27); }

	.pe-tags {
		flex: 1; min-width: 0;
		display: flex; flex-wrap: wrap; align-items: center; gap: 4px;
	}
	.pe-tag {
		display: inline-flex; align-items: center; gap: 2px;
		background: var(--background-modifier-border-focus); color: var(--text-muted);
		padding: 1px 6px; border-radius: 10px;
		font-size: 0.75rem; white-space: nowrap;
	}
	.pe-tag-x {
		border: none; background: none; color: var(--text-faint);
		cursor: pointer; font-size: 0.75rem; padding: 0 1px;
		line-height: 1;
	}
	.pe-tag-x:hover { color: var(--text-error); }
	.pe-tag-input {
		flex: 1; min-width: 50px; border: none; background: none;
		padding: 2px 4px; font-size: 0.78rem; color: var(--text-normal);
		font-family: inherit; outline: none;
	}

	.pe-link-wrap {
		flex: 1; min-width: 0;
		display: flex; align-items: center; gap: 0;
	}
	.pe-link-bracket { color: var(--interactive-accent); font-size: 0.82rem; font-weight: 600; flex-shrink: 0; }
	.pe-link-input { flex: 1; color: var(--interactive-accent); }

	.pe-del {
		width: 20px; height: 20px; flex-shrink: 0;
		display: flex; align-items: center; justify-content: center;
		border: none; background: none; border-radius: 3px;
		color: var(--color-base-40); cursor: pointer; padding: 0;
		opacity: 0; transition: opacity 0.15s;
	}
	.pe-row:hover .pe-del { opacity: 1; }
	.pe-del:hover { background: var(--background-modifier-error-hover); color: var(--text-error); }

	.pe-add {
		display: block; width: 100%; margin-top: 6px;
		border: 1px dashed var(--background-modifier-border); border-radius: 4px;
		background: none; padding: 4px 8px;
		color: var(--text-faint); font-size: 0.78rem; font-family: inherit;
		cursor: pointer; text-align: start;
	}
	.pe-add:hover { border-color: var(--interactive-accent); color: var(--interactive-accent); }
</style>
