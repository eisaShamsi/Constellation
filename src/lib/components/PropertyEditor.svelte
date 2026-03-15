<script lang="ts">
	import type { FrontmatterProperty, PropertyType } from '$lib/libraries/store';
	import { saveTabContent, normalizeDateValue } from '$lib/libraries/store';
	import { setRegisteredType, getRegisteredType } from '$lib/libraries/propertyTypeRegistry';
	import { t, locale } from '$lib/i18n';
	import { onMount, onDestroy } from 'svelte';

	let {
		properties,
		body,
		tabId,
		filePath,
		onNoteClick,
		libraryName = '',
	}: {
		properties: FrontmatterProperty[];
		body: string;
		tabId: string;
		filePath: string;
		onNoteClick?: (noteName: string) => void;
		libraryName?: string;
	} = $props();

	const TYPE_ICONS: Record<PropertyType, string> = {
		text: '\u2261',
		number: '#',
		date: '\uD83D\uDCC5',
		datetime: '\uD83D\uDD50',
		list: '\u2255',
		link: '\uD83D\uDD17',
		checkbox: '\u2611'
	};

	const TYPE_ORDER: PropertyType[] = ['text', 'number', 'date', 'datetime', 'list', 'link', 'checkbox'];

	const TYPE_I18N_KEYS: Record<PropertyType, string> = {
		text: 'propertyEditor.typeText',
		number: 'propertyEditor.typeNumber',
		date: 'propertyEditor.typeDate',
		datetime: 'propertyEditor.typeDatetime',
		list: 'propertyEditor.typeList',
		link: 'propertyEditor.typeLink',
		checkbox: 'propertyEditor.typeCheckbox',
	};

	// Special well-known property keys with distinct icons (English + Arabic)
	const SPECIAL_KEYS: Record<string, { icon: string; color: string }> = {
		tags: { icon: '#', color: 'var(--interactive-accent)' },
		aliases: { icon: '\u2194', color: 'var(--text-accent)' },
		cssclasses: { icon: '{ }', color: 'var(--color-orange)' },
		cssclass: { icon: '{ }', color: 'var(--color-orange)' },
		'الوسم': { icon: '#', color: 'var(--interactive-accent)' },
		'وسوم': { icon: '#', color: 'var(--interactive-accent)' },
		'أسماء بديلة': { icon: '\u2194', color: 'var(--text-accent)' },
	};

	// Property key suggestions (bilingual)
	const KEY_SUGGESTIONS = [
		{ key: 'tags', label: 'tags', labelAr: 'الوسم' },
		{ key: 'aliases', label: 'aliases', labelAr: 'أسماء بديلة' },
		{ key: 'cssclasses', label: 'cssclasses', labelAr: 'cssclasses' },
		{ key: 'publish', label: 'publish', labelAr: 'منشور' },
		{ key: 'permalink', label: 'permalink', labelAr: 'رابط ثابت' },
		{ key: 'description', label: 'description', labelAr: 'الوصف' },
		{ key: 'image', label: 'image', labelAr: 'الصورة' },
		{ key: 'cover', label: 'cover', labelAr: 'الغلاف' },
		{ key: 'date', label: 'date', labelAr: 'تاريخ' },
		{ key: 'created', label: 'created', labelAr: 'أنشئ' },
		{ key: 'updated', label: 'updated', labelAr: 'حُدث' },
		{ key: 'author', label: 'author', labelAr: 'المؤلف' },
		{ key: 'source', label: 'source', labelAr: 'المصدر' },
		{ key: 'status', label: 'status', labelAr: 'الحالة' },
		{ key: 'type', label: 'type', labelAr: 'النوع' },
		{ key: 'category', label: 'category', labelAr: 'الفئة' },
		{ key: 'related', label: 'related', labelAr: 'ذات صلة' },
	];

	let editableProps = $state<FrontmatterProperty[]>([]);
	let saveTimeout: ReturnType<typeof setTimeout>;
	let focusRaf: number | null = null;
	let saving = $state(false);
	let prevTabId = $state('');
	let tagInputs = $state<Record<number, string>>({});

	// Drag-to-reorder state
	let dragIdx = $state(-1);
	let dropIdx = $state(-1);

	// Type dropdown state
	let openTypeMenu = $state(-1);

	// Key suggestion state
	let focusedKeyIdx = $state(-1);
	let suggestHighlight = $state(0);

	// Ref for focusing new property
	let addBtnRef = $state<HTMLButtonElement | null>(null);

	// Snapshot incoming props for change detection
	let prevPropsSnapshot = $state('');

	// Sync from props when tab changes OR when properties change externally
	$effect(() => {
		const currentSnapshot = JSON.stringify(properties.map(p => ({ k: p.key, v: p.value, t: p.type })));
		const tabChanged = tabId !== prevTabId;
		const propsChanged = currentSnapshot !== prevPropsSnapshot;

		if (tabChanged || propsChanged) {
			if (!saving || tabChanged) {
				editableProps = properties.map(p => {
					// Apply registered type override if available
					const registeredType = libraryName ? getRegisteredType(libraryName, p.key) : undefined;
					return {
						...p,
						type: registeredType ?? p.type,
						listItems: p.listItems ? [...p.listItems] : undefined
					};
				});
			}
			prevPropsSnapshot = currentSnapshot;
			if (tabChanged) {
				prevTabId = tabId;
				tagInputs = {};
			}
		}
	});

	// Close dropdowns on outside click
	function handleDocClick(e: MouseEvent) {
		const target = e.target as HTMLElement;
		if (openTypeMenu >= 0 && !target.closest('.pe-type-dropdown-wrap')) {
			openTypeMenu = -1;
		}
		if (focusedKeyIdx >= 0 && !target.closest('.pe-key-wrap')) {
			focusedKeyIdx = -1;
		}
	}

	// Listen for global add-property event (Ctrl+;)
	function handleAddPropertyEvent() {
		addProperty();
		if (focusRaf !== null) cancelAnimationFrame(focusRaf);
		focusRaf = requestAnimationFrame(() => {
			focusRaf = null;
			const rows = document.querySelectorAll('.pe-key');
			const last = rows[rows.length - 1] as HTMLInputElement | undefined;
			last?.focus();
		});
	}

	onMount(() => {
		document.addEventListener('constellation:add-property', handleAddPropertyEvent);
		document.addEventListener('click', handleDocClick);
	});
	onDestroy(() => {
		document.removeEventListener('constellation:add-property', handleAddPropertyEvent);
		document.removeEventListener('click', handleDocClick);
		if (focusRaf !== null) cancelAnimationFrame(focusRaf);
		// Flush any pending save before the component is destroyed
		if (saveTimeout) {
			clearTimeout(saveTimeout);
			if (tabId && filePath) {
				saveTabContent(tabId, filePath, editableProps, body).catch(() => {});
			}
		}
	});

	function getIcon(prop: FrontmatterProperty): { icon: string; color?: string; isSpecial: boolean } {
		const special = SPECIAL_KEYS[prop.key.toLowerCase()] || SPECIAL_KEYS[prop.key];
		if (special) return { icon: special.icon, color: special.color, isSpecial: true };
		return { icon: TYPE_ICONS[prop.type], isSpecial: false };
	}

	function setType(idx: number, newType: PropertyType) {
		openTypeMenu = -1;
		const prop = editableProps[idx];
		editableProps = editableProps.map((p, i) => {
			if (i !== idx) return p;
			const updated = { ...p, type: newType };
			if (newType === 'list' && !updated.listItems) {
				updated.listItems = updated.value ? updated.value.split(',').map(s => s.trim()).filter(Boolean) : [];
				updated.value = updated.listItems.join(', ');
			} else if (newType === 'link' && !updated.value.startsWith('[[')) {
				updated.value = updated.value ? `[[${updated.value}]]` : '';
			} else if (newType === 'checkbox') {
				const lv = updated.value.toLowerCase();
				updated.value = (lv === 'true' || lv === '1' || lv === 'yes') ? 'true' : 'false';
				updated.listItems = undefined;
			} else if (newType === 'date' || newType === 'datetime') {
				if (updated.value) updated.value = normalizeDateValue(updated.value);
				updated.listItems = undefined;
			} else if (newType !== 'list') {
				if (p.type === 'list' && p.listItems) {
					updated.value = p.listItems.join(', ');
				}
				updated.listItems = undefined;
			}
			return updated;
		});
		// Persist type choice library-wide
		if (libraryName && prop.key) {
			setRegisteredType(libraryName, prop.key, newType);
		}
		debouncedSave();
	}

	function updateKey(idx: number, newKey: string) {
		editableProps = editableProps.map((p, i) =>
			i === idx ? { ...p, key: newKey } : p
		);
		debouncedSave();
	}

	function selectKeySuggestion(idx: number, suggestion: typeof KEY_SUGGESTIONS[0]) {
		const isAr = $locale === 'ar' || $locale === 'fa' || $locale === 'ur' || $locale === 'he';
		const newKey = isAr ? suggestion.labelAr : suggestion.key;
		updateKey(idx, newKey);
		focusedKeyIdx = -1;
	}

	function updateValue(idx: number, newValue: string) {
		editableProps = editableProps.map((p, i) =>
			i === idx ? { ...p, value: newValue } : p
		);
		debouncedSave();
	}

	function toggleCheckbox(idx: number) {
		const current = editableProps[idx].value === 'true';
		updateValue(idx, current ? 'false' : 'true');
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

	function handleKeyInputKeydown(e: KeyboardEvent, idx: number) {
		const filtered = getFilteredSuggestions(idx);
		if (e.key === 'ArrowDown') {
			e.preventDefault();
			suggestHighlight = Math.min(suggestHighlight + 1, filtered.length - 1);
		} else if (e.key === 'ArrowUp') {
			e.preventDefault();
			suggestHighlight = Math.max(suggestHighlight - 1, 0);
		} else if (e.key === 'Enter' && focusedKeyIdx === idx && filtered.length > 0) {
			e.preventDefault();
			selectKeySuggestion(idx, filtered[suggestHighlight]);
		} else if (e.key === 'Escape') {
			focusedKeyIdx = -1;
		}
	}

	function getFilteredSuggestions(idx: number): typeof KEY_SUGGESTIONS {
		const currentKey = editableProps[idx]?.key?.toLowerCase() ?? '';
		const usedKeys = new Set(editableProps.map((p, i) => i !== idx ? p.key.toLowerCase() : ''));
		return KEY_SUGGESTIONS.filter(s =>
			!usedKeys.has(s.key) && !usedKeys.has(s.labelAr) &&
			(currentKey === '' || s.key.includes(currentKey) || s.label.includes(currentKey) || s.labelAr.includes(currentKey))
		);
	}

	function formatDateLocale(value: string): string {
		if (!value) return '';
		try {
			const d = new Date(value + 'T00:00:00');
			if (isNaN(d.getTime())) return '';
			return d.toLocaleDateString($locale === 'ar' ? 'ar-SA' : $locale === 'fa' ? 'fa-IR' : $locale === 'he' ? 'he-IL' : $locale === 'ja' ? 'ja-JP' : $locale === 'ko' ? 'ko-KR' : $locale === 'zh' ? 'zh-CN' : $locale === 'hi' ? 'hi-IN' : $locale === 'ur' ? 'ur-PK' : $locale === 'ru' ? 'ru-RU' : $locale === 'de' ? 'de-DE' : $locale === 'fr' ? 'fr-FR' : $locale === 'es' ? 'es-ES' : $locale === 'pt' ? 'pt-BR' : $locale === 'tr' ? 'tr-TR' : 'en-US', { year: 'numeric', month: 'long', day: 'numeric' });
		} catch { return ''; }
	}

	function handleLinkClick(value: string) {
		if (!onNoteClick) return;
		const noteName = value.replace(/^\[\[|\]\]$/g, '');
		if (noteName) onNoteClick(noteName);
	}

	// ─── Drag-to-reorder ───
	function onDragStart(e: DragEvent, idx: number) {
		dragIdx = idx;
		if (e.dataTransfer) {
			e.dataTransfer.effectAllowed = 'move';
			e.dataTransfer.setData('text/plain', String(idx));
		}
	}

	function onDragOver(e: DragEvent, idx: number) {
		e.preventDefault();
		if (e.dataTransfer) e.dataTransfer.dropEffect = 'move';
		dropIdx = idx;
	}

	function onDragEnd() {
		dragIdx = -1;
		dropIdx = -1;
	}

	function onDrop(e: DragEvent, targetIdx: number) {
		e.preventDefault();
		if (dragIdx < 0 || dragIdx === targetIdx) { onDragEnd(); return; }
		const reordered = [...editableProps];
		const [moved] = reordered.splice(dragIdx, 1);
		reordered.splice(targetIdx, 0, moved);
		editableProps = reordered;
		onDragEnd();
		debouncedSave();
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
		<span class="pe-title">{$t('propertyEditor.title')}</span>
		{#if saving}
			<span class="pe-saving">{$t('propertyEditor.saving')}</span>
		{/if}
	</div>

	{#each editableProps as prop, idx}
		{@const iconInfo = getIcon(prop)}
		{@const isEmpty = !prop.value || (prop.type === 'list' && (!prop.listItems || prop.listItems.length === 0))}
		<div class="pe-row"
			class:pe-dragging={dragIdx === idx}
			class:pe-drop-above={dropIdx === idx && dragIdx !== idx && dragIdx > idx}
			class:pe-drop-below={dropIdx === idx && dragIdx !== idx && dragIdx < idx}
			draggable="true"
			ondragstart={(e) => onDragStart(e, idx)}
			ondragover={(e) => onDragOver(e, idx)}
			ondragleave={() => { if (dropIdx === idx) dropIdx = -1; }}
			ondragend={onDragEnd}
			ondrop={(e) => onDrop(e, idx)}
		>
			<!-- Drag handle -->
			<span class="pe-drag-handle" title={$t('propertyEditor.reorder')}>&#x2807;</span>

			<!-- Type icon with dropdown -->
			<div class="pe-type-dropdown-wrap">
				<button class="pe-type-btn" class:pe-special={iconInfo.isSpecial}
					title={$t('propertyEditor.selectType')}
					style={iconInfo.color ? `color: ${iconInfo.color}` : ''}
					onclick={(e) => { e.stopPropagation(); openTypeMenu = openTypeMenu === idx ? -1 : idx; }}>
					{iconInfo.icon}
				</button>
				{#if openTypeMenu === idx}
					<div class="pe-type-dropdown">
						{#each TYPE_ORDER as typeOpt}
							<button class="pe-type-option" class:pe-type-active={prop.type === typeOpt}
								onclick={(e) => { e.stopPropagation(); setType(idx, typeOpt); }}>
								<span class="pe-type-option-icon">{TYPE_ICONS[typeOpt]}</span>
								<span class="pe-type-option-label">{$t(TYPE_I18N_KEYS[typeOpt])}</span>
							</button>
						{/each}
					</div>
				{/if}
			</div>

			<!-- Key input with suggestions -->
			{#if iconInfo.isSpecial}
				<span class="pe-key pe-key-special">{prop.key}</span>
			{:else}
				<div class="pe-key-wrap">
					<input class="pe-key" type="text" value={prop.key}
						placeholder={$t('propertyEditor.keyPlaceholder')}
						oninput={(e) => { updateKey(idx, (e.target as HTMLInputElement).value); suggestHighlight = 0; }}
						onfocus={() => { focusedKeyIdx = idx; suggestHighlight = 0; }}
						onkeydown={(e) => handleKeyInputKeydown(e, idx)} />
					{#if focusedKeyIdx === idx}
						{@const filtered = getFilteredSuggestions(idx)}
						{#if filtered.length > 0}
							<div class="pe-suggest-dropdown">
								{#each filtered as sug, si}
									<button class="pe-suggest-item" class:pe-suggest-active={suggestHighlight === si}
										onmousedown={(e) => { e.preventDefault(); selectKeySuggestion(idx, sug); }}>
										<span class="pe-suggest-key">{sug.key}</span>
										{#if sug.labelAr !== sug.key}
											<span class="pe-suggest-ar">{sug.labelAr}</span>
										{/if}
									</button>
								{/each}
							</div>
						{/if}
					{/if}
				</div>
			{/if}

			<!-- Value input by type -->
			{#if prop.type === 'checkbox'}
				<label class="pe-checkbox-wrap">
					<input type="checkbox" class="pe-checkbox"
						checked={prop.value === 'true'}
						onchange={() => toggleCheckbox(idx)} />
					<span class="pe-checkbox-label">{prop.value === 'true' ? $t('propertyEditor.true') : $t('propertyEditor.false')}</span>
				</label>
			{:else if prop.type === 'datetime'}
				<input class="pe-val" type="datetime-local" value={prop.value}
					oninput={(e) => updateValue(idx, (e.target as HTMLInputElement).value)} />
			{:else if prop.type === 'date'}
				<div class="pe-date-wrap">
					<input class="pe-val pe-date-input" type="date" value={prop.value}
						oninput={(e) => updateValue(idx, (e.target as HTMLInputElement).value)} />
					{#if prop.value}
						{@const formatted = formatDateLocale(prop.value)}
						{#if formatted}
							<span class="pe-date-display">{formatted}</span>
						{/if}
					{/if}
				</div>
			{:else if prop.type === 'number'}
				<input class="pe-val" type="number" value={prop.value}
					placeholder={$t('propertyEditor.empty')}
					oninput={(e) => updateValue(idx, (e.target as HTMLInputElement).value)} />
			{:else if prop.type === 'list'}
				<div class="pe-tags">
					{#if prop.listItems && prop.listItems.length > 0}
						{#each prop.listItems as tag, tagIdx}
							<span class="pe-tag">
								{tag}
								<button class="pe-tag-x" onclick={() => removeTag(idx, tagIdx)}>&times;</button>
							</span>
						{/each}
					{/if}
					<input class="pe-tag-input" type="text"
						placeholder={isEmpty ? $t('propertyEditor.empty') : $t('propertyEditor.addPlaceholder')}
						value={tagInputs[idx] ?? ''}
						oninput={(e) => { tagInputs = { ...tagInputs, [idx]: (e.target as HTMLInputElement).value }; }}
						onkeydown={(e) => handleTagKeydown(e, idx)} />
				</div>
			{:else if prop.type === 'link'}
				{@const linkName = prop.value.replace(/^\[\[|\]\]$/g, '')}
				<div class="pe-link-wrap">
					{#if linkName && onNoteClick}
						<button class="pe-link-clickable" onclick={() => handleLinkClick(prop.value)}
							title={linkName}>
							<span class="pe-link-icon">\uD83D\uDD17</span>
							{linkName}
						</button>
					{:else}
						<span class="pe-link-bracket">[[</span>
						<input class="pe-val pe-link-input" type="text"
							placeholder={$t('propertyEditor.empty')}
							value={linkName}
							oninput={(e) => updateValue(idx, `[[${(e.target as HTMLInputElement).value}]]`)} />
						<span class="pe-link-bracket">]]</span>
					{/if}
				</div>
			{:else}
				<input class="pe-val" type="text" value={prop.value}
					placeholder={$t('propertyEditor.empty')}
					oninput={(e) => updateValue(idx, (e.target as HTMLInputElement).value)} />
			{/if}

			<button class="pe-del" onclick={() => removeProperty(idx)} title={$t('propertyEditor.delete')}>
				<svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M18 6L6 18M6 6l12 12"/></svg>
			</button>
		</div>
	{/each}

	<button class="pe-add" bind:this={addBtnRef} onclick={addProperty}>
		+ {$t('propertyEditor.addProperty')}
	</button>
</div>

<style>
	.property-editor {
		background: var(--background-primary-alt);
		border: 1px solid var(--background-modifier-border-focus);
		border-radius: 6px;
		padding: 10px 14px;
		margin-bottom: 4px;
	}

	.pe-header {
		display: flex; align-items: center; justify-content: space-between;
		margin-bottom: 8px;
	}
	.pe-title { font-size: 0.78rem; font-weight: 600; color: var(--text-muted); }
	.pe-saving { font-size: 0.7rem; color: var(--interactive-accent); }

	.pe-row {
		display: flex; align-items: center; gap: 6px;
		padding: 6px 0;
		border-bottom: 1px solid var(--background-secondary-alt);
		transition: opacity 0.15s, border-color 0.1s;
	}
	.pe-row:last-of-type { border-bottom: none; }
	.pe-row.pe-dragging { opacity: 0.35; }
	.pe-row.pe-drop-above { border-top: 2px solid var(--interactive-accent); }
	.pe-row.pe-drop-below { border-bottom: 2px solid var(--interactive-accent); }

	/* Drag handle */
	.pe-drag-handle {
		flex-shrink: 0; width: 14px;
		font-size: 0.9rem; color: var(--text-faint);
		cursor: grab; opacity: 0; transition: opacity 0.15s;
		user-select: none; text-align: center;
	}
	.pe-row:hover .pe-drag-handle { opacity: 0.7; }
	.pe-drag-handle:active { cursor: grabbing; }

	/* Type button + dropdown */
	.pe-type-dropdown-wrap { position: relative; flex-shrink: 0; }

	.pe-type-btn {
		width: 24px; height: 24px;
		display: flex; align-items: center; justify-content: center;
		border: none; background: none; border-radius: 3px;
		color: var(--text-faint); cursor: pointer; font-size: 0.85rem;
		padding: 0;
	}
	.pe-type-btn:hover { background: var(--background-modifier-border); color: var(--text-muted); }
	.pe-type-btn.pe-special { font-weight: 700; font-size: 0.9rem; }

	.pe-type-dropdown {
		position: absolute; top: 100%; left: 0; z-index: 100;
		background: var(--background-primary);
		border: 1px solid var(--background-modifier-border);
		border-radius: 6px; padding: 4px;
		box-shadow: var(--shadow-s);
		min-width: 140px;
	}
	:global([dir="rtl"]) .pe-type-dropdown { left: auto; right: 0; }

	.pe-type-option {
		display: flex; align-items: center; gap: 8px;
		width: 100%; border: none; background: none; padding: 5px 8px;
		border-radius: 4px; cursor: pointer;
		font-size: 0.8rem; color: var(--text-normal); font-family: inherit;
		text-align: start;
	}
	.pe-type-option:hover { background: var(--background-modifier-hover); }
	.pe-type-option.pe-type-active { background: var(--background-modifier-border-focus); font-weight: 600; }
	.pe-type-option-icon { width: 18px; text-align: center; flex-shrink: 0; }
	.pe-type-option-label { flex: 1; }

	/* Key input + suggestions */
	.pe-key-wrap { position: relative; flex-shrink: 0; width: 100px; min-width: 70px; }

	.pe-key {
		width: 100%; box-sizing: border-box;
		border: none; background: none; padding: 3px 4px;
		font-size: 0.82rem; font-weight: 500; color: var(--text-muted);
		font-family: inherit; outline: none;
		border-radius: 3px; text-align: end;
	}
	.pe-key:focus { background: var(--background-primary); box-shadow: 0 0 0 1px hsla(var(--accent-h), var(--accent-s), var(--accent-l), 0.27); }
	.pe-key-special {
		display: inline-block; width: 100px; min-width: 70px; flex-shrink: 0;
		font-weight: 600; color: var(--text-accent);
		cursor: default; user-select: none;
		padding: 3px 4px; font-size: 0.82rem; text-align: end;
	}

	.pe-suggest-dropdown {
		position: absolute; top: 100%; left: 0; right: 0; z-index: 100;
		background: var(--background-primary);
		border: 1px solid var(--background-modifier-border);
		border-radius: 6px; padding: 4px;
		box-shadow: var(--shadow-s);
		max-height: 200px; overflow-y: auto;
		min-width: 160px;
	}
	.pe-suggest-item {
		display: flex; align-items: center; justify-content: space-between; gap: 6px;
		width: 100%; border: none; background: none; padding: 4px 8px;
		border-radius: 4px; cursor: pointer;
		font-size: 0.78rem; color: var(--text-normal); font-family: inherit;
		text-align: start;
	}
	.pe-suggest-item:hover, .pe-suggest-active { background: var(--background-modifier-hover); }
	.pe-suggest-key { font-weight: 500; }
	.pe-suggest-ar { color: var(--text-faint); font-size: 0.74rem; }

	/* Value inputs */
	.pe-val {
		flex: 1; min-width: 0;
		border: none; background: none; padding: 3px 6px;
		font-size: 0.82rem; color: var(--text-normal);
		font-family: inherit; outline: none;
		border-radius: 3px;
	}
	.pe-val:focus { background: var(--background-primary); box-shadow: 0 0 0 1px hsla(var(--accent-h), var(--accent-s), var(--accent-l), 0.27); }
	.pe-val::placeholder { color: var(--text-faint); font-style: italic; }

	/* Checkbox */
	.pe-checkbox-wrap {
		flex: 1; display: flex; align-items: center; gap: 6px;
		cursor: pointer; min-width: 0;
	}
	.pe-checkbox {
		width: 16px; height: 16px; cursor: pointer;
		accent-color: var(--interactive-accent);
	}
	.pe-checkbox-label { font-size: 0.78rem; color: var(--text-muted); }

	/* Date display */
	.pe-date-wrap {
		flex: 1; min-width: 0;
		display: flex; align-items: center; gap: 6px;
	}
	.pe-date-input { flex: 0 1 auto; max-width: 150px; }
	.pe-date-display {
		font-size: 0.76rem; color: var(--text-faint);
		white-space: nowrap; overflow: hidden; text-overflow: ellipsis;
	}

	/* Tags/List */
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
	.pe-tag-input::placeholder { color: var(--text-faint); font-style: italic; }

	/* Link */
	.pe-link-wrap {
		flex: 1; min-width: 0;
		display: flex; align-items: center; gap: 0;
	}
	.pe-link-bracket { color: var(--interactive-accent); font-size: 0.82rem; font-weight: 600; flex-shrink: 0; }
	.pe-link-input { flex: 1; color: var(--interactive-accent); }
	.pe-link-input::placeholder { color: var(--text-faint); font-style: italic; }

	.pe-link-clickable {
		border: none; background: none; padding: 2px 4px;
		font-size: 0.82rem; color: var(--interactive-accent);
		cursor: pointer; font-family: inherit;
		text-decoration: none; border-radius: 3px;
		display: flex; align-items: center; gap: 4px;
		overflow: hidden; text-overflow: ellipsis; white-space: nowrap;
		max-width: 100%;
	}
	.pe-link-clickable:hover { text-decoration: underline; background: var(--background-modifier-hover); }
	.pe-link-icon { font-size: 0.75rem; flex-shrink: 0; }

	/* Delete button */
	.pe-del {
		width: 20px; height: 20px; flex-shrink: 0;
		display: flex; align-items: center; justify-content: center;
		border: none; background: none; border-radius: 3px;
		color: var(--color-base-40); cursor: pointer; padding: 0;
		opacity: 0; transition: opacity 0.15s;
	}
	.pe-row:hover .pe-del { opacity: 1; }
	.pe-del:hover { background: var(--background-modifier-error-hover); color: var(--text-error); }

	/* Add button */
	.pe-add {
		display: block; width: 100%; margin-top: 6px;
		border: 1px dashed var(--background-modifier-border); border-radius: 4px;
		background: none; padding: 5px 8px;
		color: var(--text-faint); font-size: 0.78rem; font-family: inherit;
		cursor: pointer; text-align: start;
	}
	.pe-add:hover { border-color: var(--interactive-accent); color: var(--interactive-accent); }
</style>
