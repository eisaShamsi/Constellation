<script lang="ts">
	import type { FrontmatterProperty, PropertyType } from '$lib/libraries/store';
	import { saveTabContent, normalizeDateValue, buildFullContent, openTabs } from '$lib/libraries/store';
	import { LIVING_LINK_BASELINE, lookupStageEmoji, splitStage, stageLabel } from '$lib/libraries/store';
	import { setRegisteredType, getRegisteredType } from '$lib/libraries/propertyTypeRegistry';
	import { t, locale } from '$lib/i18n';
	import { get } from 'svelte/store';
	import { onMount, onDestroy } from 'svelte';
	import { appSettings } from '$lib/libraries/store';

	// Share the user's configured pill shape with BacklinksPanel /
	// OutgoingLinksPanel / LinkDashboard so frontmatter tag pills track
	// the same radius / height / weight as every other pill in the app.
	const pillShape = $derived($appSettings.linkPills?.shape ?? { radius: 10, height: 20, fontWeight: 700 });
	import { formatDate } from '$lib/utils';

	// MIG-021v2 §1D' — Hierarchical pickers for `sources:` and `content_type:`
	// frontmatter fields. Single source of truth: the Rust taxonomies fetched
	// via IPC and cached. PropertyEditor stores selections as a YAML list;
	// the existing save path (`saveTabContent` → `index_note`) re-extracts
	// on save so the SQLite mirror updates. No special IPC needed here.
	import TaxonomyTreePicker from '$lib/sources/TaxonomyTreePicker.svelte';
	import { getHorizontalTaxonomy, type HorizontalNode } from '$lib/sources/horizontalTaxonomy';
	import { getVerticalTaxonomy, type VerticalNode } from '$lib/sources/verticalTaxonomy';

	let {
		properties,
		body,
		tabId,
		filePath,
		onNoteClick,
		libraryName = '',
		noteDir = 'ltr' as 'ltr' | 'rtl',
		collapsed = false,
		onToggle,
		onstagechange,
	}: {
		properties: FrontmatterProperty[];
		body: string;
		tabId: string;
		filePath: string;
		onNoteClick?: (noteName: string) => void;
		libraryName?: string;
		noteDir?: 'ltr' | 'rtl';
		collapsed?: boolean;
		onToggle?: () => void;
		onstagechange?: (stage: string) => void;
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
		{ key: 'status', label: 'status', labelAr: 'الحالة' },
		{ key: 'type', label: 'type', labelAr: 'النوع' },
		{ key: 'category', label: 'category', labelAr: 'الفئة' },
		{ key: 'related', label: 'related', labelAr: 'ذات صلة' },
		{ key: 'sources', label: 'sources', labelAr: 'المصادر' },
		{ key: 'content_type', label: 'content_type', labelAr: 'نوع المحتوى' },
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

	// MIG-014 §1C.5 — Stage combobox dropdown state. `stageMenuOpen` is the
	// property-row index whose stage dropdown is currently open, or -1.
	// Native <datalist> was replaced because Chromium/WebView2 renders the
	// option's `value` and inner-text as a two-tier line that confuses the
	// "type or pick" affordance. This is a custom dropdown matching the
	// type-icon dropdown's visual treatment.
	let stageMenuOpen = $state(-1);
	let stageHighlight = $state(0);
	// `stageUserNavigated` flips true on ArrowUp/ArrowDown and back to false
	// on typing — so Enter knows whether to commit the highlighted dropdown
	// item (user explicitly arrowed to it) or the typed input value (user
	// is creating a custom stage).
	let stageUserNavigated = $state(false);
	// MIG-014 §2C — Mode-flip combobox. The dropdown always shows 6 entries:
	// Mode A (input empty / matches a fixed lifecycle name): the 6 baseline
	// stages.  Mode B (input is a custom word or has a dash suffix): the
	// 6 paired stages (`spark-<suffix>`, `birth-<suffix>`, …) with the
	// suffix being either the part after the dash or the whole input.
	// Per Stages Concept Paper v1.2 §4: per-note scope; nothing Universe-wide.
	function buildStageOptions(inputVal: string): Array<{ value: string; emoji: string }> {
		const trimmed = inputVal.trim();
		const lcTrimmed = trimmed.toLowerCase();
		const { suffix } = splitStage(trimmed);
		const isFixed = !suffix && LIVING_LINK_BASELINE.some(b => b.name === lcTrimmed);
		if (!trimmed || isFixed) {
			return LIVING_LINK_BASELINE.map(b => ({ value: b.name, emoji: b.emoji }));
		}
		// Mode B: term is suffix when input has dash, else whole input.
		const term = (suffix || trimmed).toLowerCase();
		return LIVING_LINK_BASELINE.map(b => ({
			value: `${b.name}-${term}`,
			emoji: b.emoji,
		}));
	}

	// Key suggestion state
	let focusedKeyIdx = $state(-1);
	let suggestHighlight = $state(0);

	// Ref for focusing new property
	let addBtnRef = $state<HTMLButtonElement | null>(null);

	// Snapshot incoming props for change detection
	let prevPropsSnapshot = $state('');

	// MIG-021v2 §1D' — Lazy-loaded taxonomies + per-row tree-picker expand state.
	// The pickers stay collapsed by default (pills only); user clicks the
	// chevron to expand. `taxonomyExpanded` is the row index whose tree is
	// currently open, or -1.
	let horizontalTaxonomy = $state<HorizontalNode[]>([]);
	let verticalTaxonomy = $state<VerticalNode[]>([]);
	let taxonomiesLoaded = $state(false);
	let taxonomyExpanded = $state(-1);

	function isTaxonomyKey(key: string): 'horizontal' | 'vertical' | null {
		const k = key.toLowerCase();
		if (k === 'sources') return 'horizontal';
		if (k === 'content_type') return 'vertical';
		return null;
	}

	async function ensureTaxonomiesLoaded() {
		if (taxonomiesLoaded) return;
		try {
			[horizontalTaxonomy, verticalTaxonomy] = await Promise.all([
				getHorizontalTaxonomy(),
				getVerticalTaxonomy(),
			]);
			taxonomiesLoaded = true;
		} catch (err) {
			console.error('[PropertyEditor] taxonomy load failed:', err);
		}
	}

	function taxonomyLabel(id: string, axis: 'horizontal' | 'vertical'): string {
		const isAr = $locale === 'ar';
		if (axis === 'horizontal') {
			const node = horizontalTaxonomy.find(n => n.id === id);
			if (!node) return id;
			return isAr ? node.ar : node.en;
		}
		const node = verticalTaxonomy.find(n => n.id === id);
		if (!node) return id;
		return isAr ? node.ar : node.en;
	}

	function tierColorForId(id: string): string | null {
		const node = horizontalTaxonomy.find(n => n.id === id);
		if (!node) return null;
		const tier = node.tier > 0
			? node.tier
			: (node.parent_id ? (horizontalTaxonomy.find(n => n.id === node.parent_id)?.tier ?? 0) : 0);
		switch (tier) {
			case 1: return '#0f6e56';
			case 2: return '#534ab7';
			case 3: return '#854f0b';
			default: return null;
		}
	}

	function applyTaxonomySelection(idx: number, selected: Set<string>) {
		const items = [...selected];
		editableProps = editableProps.map((p, i) => {
			if (i !== idx) return p;
			return {
				...p,
				type: 'list' as PropertyType,
				listItems: items,
				value: items.join(', '),
			};
		});
		debouncedSave();
	}

	function removeTaxonomyValue(idx: number, id: string) {
		const prop = editableProps[idx];
		const next = (prop.listItems ?? []).filter(v => v !== id);
		applyTaxonomySelection(idx, new Set(next));
	}

	function toggleTaxonomyExpanded(idx: number) {
		if (taxonomyExpanded === idx) {
			taxonomyExpanded = -1;
		} else {
			ensureTaxonomiesLoaded();
			taxonomyExpanded = idx;
		}
	}
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
		if (stageMenuOpen >= 0 && !target.closest('.pe-stage-wrap')) {
			stageMenuOpen = -1;
		}
	}

	// MIG-014 §2B — Commit a stage selection. The custom-term flow
	// (typing a non-list value to register it Universe-wide) was dropped
	// in §2A because the custom term is now per-note (encoded as a dash
	// suffix in the value itself). §2C will add the mode-flip combobox
	// that handles the per-note suffix. For now the dropdown is fixed-only.
	function commitStage(idx: number, raw: string) {
		const v = raw.trim().toLowerCase();
		if (!v) return;
		updateValue(idx, v);
		onstagechange?.(v);
		stageMenuOpen = -1;
	}

	// Resolve which stage value to commit when the user presses Enter or
	// Tab. Three-tier precedence:
	//   1. Explicit arrow-navigation → commit the highlighted opt.
	//   2. Typed value already matches one of the dropdown options
	//      (Mode A: typed "birth", opts has "birth" → commit "birth";
	//      Mode B: typed "spark-concept" → commit "spark-concept").
	//   3. Mode-B custom-term shortcut: typed a non-matching word
	//      (e.g. "concept") → commit the dropdown's first item
	//      (`spark-<term>`), since the dropdown's contents reflect
	//      what the user typed.
	function commitFromInputOrHighlight(e: KeyboardEvent, idx: number, opts: Array<{ value: string; emoji: string }>) {
		const inputVal = (e.target as HTMLInputElement).value.trim().toLowerCase();
		if (stageUserNavigated && stageMenuOpen === idx && opts[stageHighlight]) {
			commitStage(idx, opts[stageHighlight].value);
		} else if (inputVal && opts.some(o => o.value === inputVal)) {
			commitStage(idx, inputVal);
		} else if (stageMenuOpen === idx && opts[0]) {
			commitStage(idx, opts[0].value);
		}
	}

	function handleStageKeydown(e: KeyboardEvent, idx: number, opts: Array<{ value: string; emoji: string }>) {
		if (e.key === 'ArrowDown') {
			e.preventDefault();
			stageUserNavigated = true;
			if (stageMenuOpen !== idx) { stageMenuOpen = idx; stageHighlight = 0; return; }
			stageHighlight = Math.min(stageHighlight + 1, opts.length - 1);
		} else if (e.key === 'ArrowUp') {
			e.preventDefault();
			stageUserNavigated = true;
			if (stageMenuOpen !== idx) { stageMenuOpen = idx; stageHighlight = opts.length - 1; return; }
			stageHighlight = Math.max(stageHighlight - 1, 0);
		} else if (e.key === 'Enter') {
			e.preventDefault();
			commitFromInputOrHighlight(e, idx, opts);
			(e.target as HTMLInputElement).blur();
		} else if (e.key === 'Tab') {
			// Same logic as Enter, but doesn't preventDefault — Tab still
			// moves focus to the next field.
			commitFromInputOrHighlight(e, idx, opts);
		} else if (e.key === 'Escape') {
			e.preventDefault();
			stageMenuOpen = -1;
			(e.target as HTMLInputElement).blur();
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
				/* Direct mutation so onflush reads fresh properties */
				const tab = get(openTabs).find(t => t.id === tabId);
				if (tab) tab.content = buildFullContent(editableProps, body);
				saveTabContent(tabId, filePath, editableProps, body).catch((e) => console.error('[PropertyEditor] Flush save failed:', e));
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

	function getDateScript(): string {
		const s = get(appSettings);
		const loc = get(locale);
		return noteDir === 'rtl'
			? (loc === 'he' ? 'hebrew' : 'arabic')
			: (s.primaryScript || 'latin');
	}

	function formatDateLocale(value: string): string {
		const s = get(appSettings);
		const loc = get(locale);
		const script = getDateScript();
		const fmt = (s.scriptDateFormats || {})[script] || s.dateFormat || 'DD/MM/YYYY';
		const dateLocale = noteDir === 'rtl' ? (loc === 'he' ? 'he' : loc === 'fa' ? 'fa' : loc === 'ur' ? 'ur' : 'ar') : loc;
		return formatDate(value, fmt, dateLocale, s.numeralStyle || 'arabic');
	}

	function isDateContextual(): boolean {
		const s = get(appSettings);
		const script = getDateScript();
		return (s.contextualDates || {})[script] ?? false;
	}

	function getDateDir(): 'ltr' | 'rtl' | 'auto' {
		if (!isDateContextual()) return 'ltr';
		return (noteDir as 'ltr' | 'rtl' | 'auto') || 'ltr';
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
				/* Update tab content in store via direct mutation (no store.update = no cascade).
				   This ensures onflush reads fresh properties when the tab is closed. */
				const tab = get(openTabs).find(t => t.id === tabId);
				if (tab) tab.content = buildFullContent(editableProps, body);
				await saveTabContent(tabId, filePath, editableProps, body);
			} catch (err) {
				console.error('Failed to save:', err);
			}
			saving = false;
		}, 800);
	}
</script>

<div class="property-editor" style="--pill-radius:{pillShape.radius}px;--pill-height:{pillShape.height}px;--pill-weight:{pillShape.fontWeight}">
	<!-- svelte-ignore a11y_click_events_have_key_events a11y_no_static_element_interactions -->
	<div class="pe-header" class:pe-clickable={!!onToggle} onclick={() => onToggle?.()}>
		{#if onToggle}
			<svg class="pe-chevron" class:collapsed={collapsed} width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M6 9l6 6 6-6"/></svg>
		{/if}
		<span class="pe-title">{$t('propertyEditor.title')}</span>
		{#if saving}
			<span class="pe-saving">{$t('propertyEditor.saving')}</span>
		{/if}
	</div>

	{#if !collapsed}
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
			{#if isTaxonomyKey(prop.key)}
				<!-- MIG-021v2 §1D' — Hierarchical taxonomy picker for `sources:`
				     and `content_type:` frontmatter fields. Pills always visible;
				     chevron toggles inline TaxonomyTreePicker (height-capped).
				     Saves through the standard YAML write path — no special IPC. -->
				{@const axis = isTaxonomyKey(prop.key)!}
				{@const items = prop.listItems ?? (prop.value ? prop.value.split(',').map(s => s.trim()).filter(Boolean) : [])}
				{@const selectedSet = new Set(items)}
				<div class="pe-taxo-wrap">
					<div class="pe-taxo-row">
						<div class="pe-taxo-pills">
							{#if items.length === 0}
								<span class="pe-taxo-empty">{$t('propertyEditor.empty')}</span>
							{:else}
								{#each items as id (id)}
									{@const color = axis === 'horizontal' ? tierColorForId(id) : null}
									<span class="pe-taxo-pill" style:--taxo-color={color ?? 'transparent'}>
										<span class="pe-taxo-label">{taxonomyLabel(id, axis)}</span>
										<button class="pe-taxo-x" onclick={() => removeTaxonomyValue(idx, id)} title={$t('propertyEditor.delete')}>&times;</button>
									</span>
								{/each}
							{/if}
						</div>
						<button
							class="pe-taxo-edit"
							class:expanded={taxonomyExpanded === idx}
							onclick={() => toggleTaxonomyExpanded(idx)}
							title={$t('taxonomyTreePicker.expandAll') || 'Open picker'}
						>▸</button>
					</div>
					{#if taxonomyExpanded === idx}
						<div class="pe-taxo-tree">
							{#if !taxonomiesLoaded}
								<div class="pe-taxo-loading">…</div>
							{:else if axis === 'horizontal'}
								<TaxonomyTreePicker
									taxonomy={horizontalTaxonomy}
									axis="horizontal"
									selected={selectedSet}
									onChange={(s) => applyTaxonomySelection(idx, s)}
									tierColors={true}
								/>
							{:else}
								<TaxonomyTreePicker
									taxonomy={verticalTaxonomy}
									axis="vertical"
									selected={selectedSet}
									onChange={(s) => applyTaxonomySelection(idx, s)}
									tierColors={false}
								/>
							{/if}
						</div>
					{/if}
				</div>
			{:else if prop.key.toLowerCase() === 'stage'}
				<!-- MIG-014 §2C — mode-flip combobox per Stages Concept Paper v1.2.
				     The dropdown is always 6 entries:
				       Mode A (input empty / matches a fixed lifecycle name):
				         the 6 Living Link baselines.
				       Mode B (custom word in input or dash suffix present):
				         the 6 paired stages with the user's suffix —
				         "spark-<suffix>", "birth-<suffix>", …
				     Per-note scope — nothing Universe-wide. The custom term
				     is encoded into the on-disk value as the dash suffix. -->
				{@const opts = buildStageOptions(prop.value)}
				<div class="pe-stage-wrap">
					<span class="pe-stage-current-emoji" aria-hidden="true">{lookupStageEmoji(prop.value)}</span>
					<input
						class="pe-val pe-stage-input"
						type="text"
						value={prop.value}
						placeholder={$t('propertyEditor.stagePlaceholder')}
						oninput={(e) => { updateValue(idx, (e.target as HTMLInputElement).value); stageUserNavigated = false; stageMenuOpen = idx; }}
						onfocus={() => { stageMenuOpen = idx; stageHighlight = 0; stageUserNavigated = false; }}
						onclick={(e) => { e.stopPropagation(); stageMenuOpen = idx; }}
						onkeydown={(e) => handleStageKeydown(e, idx, opts)}
					/>
					{#if stageMenuOpen === idx}
						<div class="pe-stage-dropdown">
							{#each opts as opt, optIdx}
								<button class="pe-stage-option" class:pe-stage-active={optIdx === stageHighlight}
									onmousedown={(e) => e.preventDefault()}
									onclick={(e) => { e.stopPropagation(); commitStage(idx, opt.value); }}>
									<span class="pe-stage-emoji">{opt.emoji}</span>
									<span class="pe-stage-label">{stageLabel(opt.value, $t)}</span>
								</button>
							{/each}
						</div>
					{/if}
				</div>
			{:else if prop.type === 'checkbox'}
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
					<input class="pe-date-hidden" type="date" value={prop.value}
						oninput={(e) => updateValue(idx, (e.target as HTMLInputElement).value)} />
					<span class="pe-date-display" dir={getDateDir()} onclick={(e) => {
						const input = (e.currentTarget as HTMLElement).previousElementSibling as HTMLInputElement;
						input?.showPicker?.();
					}}>{prop.value ? formatDateLocale(prop.value) : $t('propertyEditor.empty')}</span>
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
	{/if}
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
		display: flex; align-items: center; gap: 4px;
		margin-bottom: 8px;
	}
	.pe-header.pe-clickable { cursor: pointer; border-radius: 4px; padding: 2px 4px; margin: -2px -4px 8px; }
	.pe-header.pe-clickable:hover { background: var(--background-modifier-hover); }
	.pe-title { font-size: 0.78rem; font-weight: 600; color: var(--text-muted); }
	.pe-chevron { transition: transform 0.2s; flex-shrink: 0; color: var(--text-muted); }
	.pe-chevron.collapsed { transform: rotate(-90deg); }
	:global([dir="rtl"]) .pe-chevron.collapsed { transform: rotate(90deg); }
	.pe-saving { font-size: 0.7rem; color: var(--interactive-accent); }

	.pe-row {
		display: flex; align-items: center; gap: 5px;
		padding: 6px 0;
		border-bottom: 1px solid var(--background-secondary-alt);
		transition: opacity 0.15s, border-color 0.1s;
		min-width: 0;
	}
	.pe-row:last-of-type { border-bottom: none; }
	.pe-row.pe-dragging { opacity: 0.35; }
	.pe-row.pe-drop-above { border-top: 2px solid var(--interactive-accent); }
	.pe-row.pe-drop-below { border-bottom: 2px solid var(--interactive-accent); }

	/* Drag handle */
	.pe-drag-handle {
		flex-shrink: 0; width: 10px;
		font-size: 0.8rem; color: var(--text-faint);
		cursor: grab; opacity: 0; transition: opacity 0.15s;
		user-select: none; text-align: center;
	}
	.pe-row:hover .pe-drag-handle { opacity: 0.7; }
	.pe-drag-handle:active { cursor: grabbing; }

	/* Type button + dropdown */
	.pe-type-dropdown-wrap { position: relative; flex-shrink: 0; }

	.pe-type-btn {
		width: 20px; height: 20px;
		display: flex; align-items: center; justify-content: center;
		border: none; background: none; border-radius: 3px;
		color: var(--text-faint); cursor: pointer; font-size: 0.8rem;
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

	/* MIG-014 §1C.5 — stage combobox (custom dropdown, no native <datalist>) */
	.pe-stage-wrap {
		position: relative; flex: 1; min-width: 0;
		display: flex; align-items: center; gap: 6px;
	}
	.pe-stage-current-emoji {
		font-size: 1.05rem; line-height: 1;
		flex-shrink: 0;
		opacity: 0.95;
	}
	.pe-stage-input {
		flex: 1; min-width: 0; box-sizing: border-box;
		border: none; background: none; padding: 3px 4px;
		font-size: 0.85rem; color: var(--text-normal);
		font-family: inherit; outline: none;
		border-radius: 3px; text-align: start;
	}
	.pe-stage-input:focus { background: var(--background-primary); box-shadow: 0 0 0 1px hsla(var(--accent-h), var(--accent-s), var(--accent-l), 0.27); }
	.pe-stage-dropdown {
		position: absolute; top: 100%; left: 0; right: 0; z-index: 100;
		margin-top: 2px;
		background: var(--background-primary);
		border: 1px solid var(--background-modifier-border);
		border-radius: 8px; padding: 4px;
		box-shadow: var(--shadow-s);
		max-height: 280px; overflow-y: auto;
	}
	.pe-stage-option {
		display: flex; align-items: center; gap: 10px;
		width: 100%; border: none; background: none;
		padding: 6px 10px;
		border-radius: 6px; cursor: pointer;
		font-size: 0.95rem; color: var(--text-normal); font-family: inherit;
		text-align: start;
	}
	.pe-stage-option:hover,
	.pe-stage-option.pe-stage-active { background: var(--background-modifier-hover); }
	.pe-stage-emoji { font-size: 1.2rem; line-height: 1; flex-shrink: 0; }
	.pe-stage-label { flex: 1; }

	/* Key input + suggestions */
	.pe-key-wrap { position: relative; flex-shrink: 0; width: auto; min-width: 50px; max-width: 100px; }

	.pe-key {
		width: 100%; box-sizing: border-box;
		border: none; background: none; padding: 3px 4px;
		font-size: 0.82rem; font-weight: 500; color: var(--text-muted);
		font-family: inherit; outline: none;
		border-radius: 3px; text-align: start;
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
		position: relative;
	}
	.pe-date-hidden {
		position: absolute; opacity: 0; width: 0; height: 0; overflow: hidden; pointer-events: none;
	}
	.pe-date-display {
		font-size: 0.85rem; color: var(--text-normal);
		cursor: pointer; padding: 2px 4px; border-radius: 4px;
		white-space: nowrap;
	}
	.pe-date-display:hover {
		background: var(--background-modifier-hover);
	}

	/* Tags/List */
	.pe-tags {
		flex: 1; min-width: 0;
		display: flex; flex-wrap: wrap; align-items: center; gap: 4px;
	}
	.pe-tag {
		display: inline-flex; align-items: center; gap: 4px;
		box-sizing: border-box;
		height: var(--pill-height, 20px);
		padding: 0 8px;
		border-radius: var(--pill-radius, 10px);
		background: var(--background-modifier-border-focus); color: #fff;
		font-size: 0.75rem; font-weight: var(--pill-weight, 700);
		line-height: 1; white-space: nowrap;
	}
	.pe-tag-x {
		border: none; background: none; color: rgba(255, 255, 255, 0.75);
		cursor: pointer; font-size: 0.8rem; padding: 0 1px;
		line-height: 1;
	}
	.pe-tag-x:hover { color: #fff; }
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

	/* MIG-021v2 §1D' — taxonomy picker (sources / content_type) */
	.pe-taxo-wrap {
		flex: 1; min-width: 0;
		display: flex; flex-direction: column; gap: 4px;
	}
	.pe-taxo-row {
		display: flex; align-items: center; gap: 6px;
		min-width: 0;
	}
	.pe-taxo-pills {
		flex: 1; min-width: 0;
		display: flex; flex-wrap: wrap; align-items: center; gap: 4px;
	}
	.pe-taxo-empty {
		color: var(--text-faint); font-style: italic; font-size: 0.78rem;
	}
	.pe-taxo-pill {
		display: inline-flex; align-items: center; gap: 4px;
		box-sizing: border-box;
		height: var(--pill-height, 20px);
		padding: 0 8px;
		border-radius: var(--pill-radius, 10px);
		background: var(--background-modifier-border-focus); color: #fff;
		font-size: 0.75rem; font-weight: var(--pill-weight, 700);
		line-height: 1; white-space: nowrap;
		border-inline-start: 3px solid var(--taxo-color);
	}
	.pe-taxo-label { display: inline-block; }
	.pe-taxo-x {
		border: none; background: none; color: rgba(255, 255, 255, 0.75);
		cursor: pointer; font-size: 0.8rem; padding: 0 1px; line-height: 1;
	}
	.pe-taxo-x:hover { color: #fff; }
	.pe-taxo-edit {
		flex-shrink: 0;
		width: 18px; height: 18px;
		display: flex; align-items: center; justify-content: center;
		border: 1px solid var(--background-modifier-border);
		background: transparent;
		color: var(--text-muted);
		border-radius: 4px;
		cursor: pointer;
		font-size: 10px;
		padding: 0;
		transition: transform 0.12s;
	}
	.pe-taxo-edit:hover { background: var(--background-modifier-hover); }
	.pe-taxo-edit.expanded { transform: rotate(90deg); }
	.pe-taxo-tree {
		max-height: 320px;
		border: 1px solid var(--background-modifier-border);
		border-radius: 6px;
		overflow: hidden;
		background: var(--background-primary);
		display: flex; flex-direction: column;
	}
	.pe-taxo-loading {
		padding: 16px; text-align: center; color: var(--text-faint);
	}
</style>
