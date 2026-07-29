<script lang="ts">
	import type { FrontmatterProperty, PropertyType } from '$lib/libraries/store';
	import { saveTabContent, normalizeDateValue, buildFullContent, openTabs, isReseeding, isCascading } from '$lib/libraries/store';
	import { LIVING_LINK_BASELINE, lookupStageEmoji, splitStage, stageLabel } from '$lib/libraries/store';
	import { setRegisteredType, getRegisteredType } from '$lib/libraries/propertyTypeRegistry';
	import { t, locale } from '$lib/i18n';
	import { get } from 'svelte/store';
	import { onMount, onDestroy } from 'svelte';
	import { appSettings } from '$lib/libraries/store';
	import { culturalDateString, applyCalendarPrefs, frontmatterKey, type CalendarSystem } from '$lib/calendar/calendarMath'; // §C — Gregorian→cultural property converter
	import { invoke } from '@tauri-apps/api/core';
	// MIG-107 Slice 3 — the READ half of single ownership for properties.
	import { PROPS_SINGLE_OWNERSHIP } from '$lib/editor/ownershipFlag';
	import { getModel } from '$lib/editor/noteModel';
	import { propsVersion } from '$lib/editor/propsSignal';
	import { plan as planPropOps, apply as applyPropOps, touchedSince } from '$lib/editor/propsCommit';
	import { editPropValue, addPropTo, removePropFrom, reorderPropsIn } from '$lib/editor/noteSession';
	import { withAutoUpdatedDate } from '$lib/libraries/store';

	// Share the user's configured pill shape with BacklinksPanel /
	// OutgoingLinksPanel / CCSView so frontmatter tag pills track
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
		onLiveProps,
		onPropContextMenu,
		/* G3 — read-only display mode (second screen's default). Gates the two
		   disk-write sites (debouncedSave + onDestroy flush) so a "read-only" note
		   view can never persist a property edit, and its model stays clean so the
		   cross-window freshness sync (externalChange) always adopts. */
		readOnly = false,
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
		/* MIG-087 §E (item 2) — one-way, display-only live props-count observer
		   (mirrors §C's onLiveStats). Reports the focused tab's non-empty-key
		   property count on every edit; never writes back into editor content, so
		   it sits outside the BUG-015 / §C-2 vector. The host debounces/uses it. */
		onLiveProps?: (tabId: string, count: number) => void;
		/** MIG-077 §F-Editor — right-click a property row; NotePane builds the menu. */
		onPropContextMenu?: (prop: FrontmatterProperty, idx: number, x: number, y: number) => void;
		readOnly?: boolean;
	} = $props();

	// Sweep-2026-07-18 #1 (APP-KILLER) — snapshot the note identity at mount. The `tabId`/
	// `filePath` props update reactively when the parent swaps notes (in-place wikilink nav / tab
	// switch); during the NoteEditor {#key} teardown they already point at the INCOMING note while
	// this instance's `editableProps` still hold the OUTGOING note's frontmatter. The onDestroy
	// flush guards on these snapshots so it can never splice the outgoing props onto the incoming
	// note (the BUG-023 content-integrity class via the props channel). Mirrors NotePane's
	// `mountedFilePath` body-save guard (NotePane.svelte:297).
	const mountedTabId = tabId;
	const mountedFilePath = filePath;

	const TYPE_ICONS: Record<PropertyType, string> = {
		text: '\u2261',
		number: '#',
		date: '\uD83D\uDCC5',
		datetime: '\uD83D\uDD50',
		list: '\u2255',
		link: '\uD83D\uDD17',
		checkbox: '\u2611',
		// MIG-022 \u00A7A.1 \u2014 nested-object-list (e.g. ikhtil\u0101f). Distinct
		// glyph until the \u00A7A.3 ikhtil\u0101f widget renders the structured
		// rows directly. Tablet-with-rows icon (U+2630).
		'nested-object-list': '\u2630',
		// PJ-136 \u2014 a nested MAP (`source:` with `title`/`author`/`year` under it).
		// Rendered read-only until PJ-137 unifies the two property parsers and makes
		// nested structure editable for real. Branch/tree glyph (U+2387).
		'nested-map': '\u2387',
	};

	const TYPE_ORDER: PropertyType[] = ['text', 'number', 'date', 'datetime', 'list', 'link', 'checkbox', 'nested-object-list'];

	const TYPE_I18N_KEYS: Record<PropertyType, string> = {
		text: 'propertyEditor.typeText',
		number: 'propertyEditor.typeNumber',
		date: 'propertyEditor.typeDate',
		datetime: 'propertyEditor.typeDatetime',
		list: 'propertyEditor.typeList',
		link: 'propertyEditor.typeLink',
		checkbox: 'propertyEditor.typeCheckbox',
		// MIG-022 \u00A7A.1 \u2014 placeholder i18n key for the nested-object-list
		// type label. The \u00A7A.4 i18n cascade adds this key to en + ar +
		// 13-locale backfill alongside the other propertyEditor.* labels.
		'nested-object-list': 'propertyEditor.typeNestedObjectList',
		// PJ-136 — see TYPE_ICONS above.
		'nested-map': 'propertyEditor.typeNestedMap',
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
		// PJ-065 — the structural (parent/TOC) link properties as first-class presets. The
		// written KEY stays canonical English ('parent'/'contains') in every locale (the
		// structural reader matches those, like cid_cn/kind); the pill display is localized
		// elsewhere. Picking them coerces the type so values auto-wrap as [[wikilinks]].
		{ key: 'parent', label: 'parent', labelAr: 'parent' },
		{ key: 'contains', label: 'contains', labelAr: 'contains' },
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

	// PJ-065 — structural link properties author as 'list' types (clean chips), each item a
	// [[wikilink]] (auto-wrapped if the user omits brackets; brackets stripped for display).
	// BOTH 'parent' and 'contains' use 'list' so they render identically and avoid the
	// link-input double-wrap (the [[[triple]]] the Boss hit); 'parent' is conceptually single
	// (the reader takes the first item). Canonical English keys (the reader matches these).
	const STRUCTURAL_LIST_LINK_KEYS = new Set(['parent', 'contains']);
	const structuralKeyType = (key: string): PropertyType | null =>
		STRUCTURAL_LIST_LINK_KEYS.has(key) ? 'list' : null;

	// ─── MIG-107 Slice 3 — WHERE THIS PANEL'S TRUTH COMES FROM ──────────────────────────────────
	//
	// The `properties` PROP is a projection of `tab.content`: the file as it looked when the note was
	// opened. The model-based writers deliberately never refresh it (`saveTabContent` — "Do NOT update
	// the store during autosave"), so it is stale the moment ANY writer runs, and the two mounted
	// panels drift apart from each other and from the file (PJ-174 AK-2/AK-3).
	//
	// With the flag on, this panel reads the MODEL — the same array `compose` writes to disk — and
	// `$propsVersion` is what tells it to look again. The signal carries no payload on purpose; it
	// says "look", never "here is the value", because a payload would be the second copy all over
	// again (see propsSignal.ts).
	//
	// The `?? properties` fallback matters: a host may mount this panel before its model exists
	// (index preview, dashboard). Falling back to the projection is strictly better than rendering
	// nothing, and it is the same content the panel would have shown anyway.
	const sourceProps = $derived.by(() => {
		if (!PROPS_SINGLE_OWNERSHIP) return properties;
		void $propsVersion; // subscribe — re-read the model whenever some note's props change
		return getModel(tabId)?.props ?? properties;
	});

	let editableProps = $state<FrontmatterProperty[]>([]);
	/**
	 * MIG-107 Slice 4 — the keys this panel was showing when it last read the model.
	 *
	 * This is the ONLY thing a commit is allowed to delete from. A key that appears afterwards —
	 * a tag added from the file-tree menu, a property set in the other panel — is not in this set,
	 * so there is no path by which this panel's next save can remove it. See propsCommit.ts.
	 * Deliberately NOT `$state`: it is bookkeeping for the commit, never rendered.
	 */
	let seededKeys = new Set<string>();
	/**
	 * MIG-107 #1e — the rows exactly as this panel was seeded with them.
	 *
	 * A commit may only SET keys that DIFFER from these, so a value another writer changed on a key
	 * both panels show is never written back stale. Derived at commit time (`touchedSince`) rather
	 * than hand-marked in the edit handlers: the hand-marked version was wired at 3 of this
	 * component's 16 mutation sites and silently dropped tag edits entirely.
	 */
	let seededRows: FrontmatterProperty[] = [];
	let saveTimeout: ReturnType<typeof setTimeout> | undefined;
	let mounted = true; // §C — guards the async Hijri converter from writing after teardown
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
	/**
	 * Which option the stage list should open on: the one the note is ALREADY at.
	 *
	 * Boss-found 2026-07-29 — it always opened on the first entry (Spark), so a note at Growth
	 * offered "Spark" as the highlighted choice and one careless Enter would send it backwards.
	 * A picker for a value that already exists should show you where you ARE, not where the list
	 * happens to begin. Falls back to 0 when the current value is not one of the offered options
	 * (a custom per-note term), which is the only case where "no current entry" is the truth.
	 */
	function stageIndexOf(opts: Array<{ value: string }>, current: string): number {
		const c = (current ?? '').trim().toLowerCase();
		if (!c) return 0;
		const i = opts.findIndex((o) => o.value.toLowerCase() === c);
		return i >= 0 ? i : 0;
	}

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
		// MIG-022 §E.3.b (PJ-043, 2026-05-11): prefer i18n catalog over
		// hardcoded en/ar struct fields. See SourceReviewPanel.labelForId
		// for the same pattern.
		const i18nKey = `cece.taxonomy.${id}`;
		const translated = $t(i18nKey);
		if (translated && translated !== i18nKey) return translated;
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

	// Order selected IDs by taxonomy tree position (parent-first, depth-first
	// pre-order traversal). Returns each id with its depth so the pill can
	// be indented + prefixed with a ↳ connector. Eisa correction 2026-05-09:
	// pills must read like an outline — parent on top, children indented
	// underneath — not as a flat shuffled bag.
	function orderTaxonomyItems(items: string[], axis: 'horizontal' | 'vertical'): Array<{ id: string; depth: number }> {
		const pool = new Set(items);
		// MIG-071 audit HIGH — widen to the union element type so the childrenByParent Map / push /
		// walk type-check honestly (both node kinds carry id + parent_id, the only fields used here).
		const taxonomy: Array<HorizontalNode | VerticalNode> = axis === 'horizontal' ? horizontalTaxonomy : verticalTaxonomy;
		if (taxonomy.length === 0) {
			// Taxonomy not loaded yet — return items as-is at depth 0 so the
			// pills still render before the first picker expand.
			return items.map(id => ({ id, depth: 0 }));
		}
		const childrenByParent = new Map<string | null, typeof taxonomy>();
		for (const node of taxonomy) {
			const key = node.parent_id;
			if (!childrenByParent.has(key)) childrenByParent.set(key, []);
			childrenByParent.get(key)!.push(node);
		}
		const roots = axis === 'horizontal'
			? taxonomy.filter(n => n.parent_id === null)
			: taxonomy.filter(n => n.parent_id === 'epistemic-content');
		const out: Array<{ id: string; depth: number }> = [];
		const walk = (nodes: typeof taxonomy, depth: number) => {
			for (const node of nodes) {
				if (pool.has(node.id)) {
					out.push({ id: node.id, depth });
					pool.delete(node.id);
				}
				const kids = childrenByParent.get(node.id) ?? [];
				if (kids.length > 0) walk(kids, depth + 1);
			}
		};
		walk(roots, 0);
		// Anything left in pool wasn't found in the taxonomy — append at
		// depth 0 so we don't silently drop it.
		for (const id of pool) out.push({ id, depth: 0 });
		return out;
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
		const prop = editableProps[idx];
		const axis = isTaxonomyKey(prop.key);
		// Sort to taxonomy tree order (parent first, depth-first pre-order)
		// so the on-disk YAML reads as an outline — same order as the pills.
		// Falls back to insertion order if taxonomies haven't loaded yet
		// (shouldn't happen — user must open the picker before this fires).
		const items = (axis && taxonomiesLoaded)
			? orderTaxonomyItems([...selected], axis).map(r => r.id)
			: [...selected];
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
		// MIG-107 Slice 3 — seed from `sourceProps` (the model when the flag is on), not from the
		// stale projection. Everything below is unchanged.
		const currentSnapshot = JSON.stringify(sourceProps.map(p => ({ k: p.key, v: p.value, t: p.type })));
		const tabChanged = tabId !== prevTabId;
		const propsChanged = currentSnapshot !== prevPropsSnapshot;

		if (tabChanged || propsChanged) {
			// MIG-107 Slice 3 — do NOT re-seed over an edit the user has typed but not yet flushed.
			// The model now changes far more often than `tab.content` ever did (every writer ticks the
			// signal), so without this a keystroke in one field could be reverted by an unrelated
			// property change elsewhere. A pending debounce means `editableProps` holds at least one
			// value newer than the model; it wins until it flushes. A tab change always re-seeds —
			// that is a different note, and its pending edit is flushed by the teardown path.
			const localEditPending = saveTimeout !== undefined;
			if ((!saving && !localEditPending) || tabChanged) {
				seededKeys = new Set(sourceProps.map(p => p.key).filter(k => !!k && !!k.trim()));
				seededRows = sourceProps.map(p => ({ ...p, listItems: p.listItems ? [...p.listItems] : undefined }));
				editableProps = sourceProps.map(p => {
					// Apply registered type override if available
					const registeredType = libraryName ? getRegisteredType(libraryName, p.key) : undefined;
					// PJ-065 — structural link keys ALWAYS render as 'list' chips, overriding any
					// stale registered/inferred type (e.g. a parent once saved as a scalar 'link').
					// Seed listItems from a scalar value so an existing single-link parent still shows.
					const forced = structuralKeyType(p.key);
					let listItems = p.listItems ? [...p.listItems] : undefined;
					if (forced === 'list' && !listItems) {
						listItems = p.value ? p.value.split(',').map(s => s.trim()).filter(Boolean) : [];
					}
					return {
						...p,
						type: forced ?? registeredType ?? p.type,
						listItems
					};
				});
				// ★ PJ-174 #1e — advance the snapshot ONLY when we actually re-seeded. It used to advance
				// even on the skip path, marking a model change "seen" that never reached the rows —
				// so it never re-seeded later either (propsChanged was false forever after), and the
				// panel displayed a stale value indefinitely. Inside the branch, a skipped change is
				// retried on the next tick instead.
				prevPropsSnapshot = currentSnapshot;
			}
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

	/** The row's current stage text — the keydown handler has the row index, not the row. */
	function currentStageValue(idx: number): string {
		return editableProps[idx]?.value ?? '';
	}

	function handleStageKeydown(e: KeyboardEvent, idx: number, opts: Array<{ value: string; emoji: string }>) {
		if (e.key === 'ArrowDown') {
			e.preventDefault();
			stageUserNavigated = true;
			// Opening with a key starts from where the note IS, then moves — standard combobox behaviour.
			if (stageMenuOpen !== idx) { stageMenuOpen = idx; stageHighlight = stageIndexOf(opts, currentStageValue(idx)); return; }
			stageHighlight = Math.min(stageHighlight + 1, opts.length - 1);
		} else if (e.key === 'ArrowUp') {
			e.preventDefault();
			stageUserNavigated = true;
			if (stageMenuOpen !== idx) { stageMenuOpen = idx; stageHighlight = stageIndexOf(opts, currentStageValue(idx)); return; }
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
		mounted = false; // §C — block any in-flight Hijri-converter write from landing on a torn-down instance
		document.removeEventListener('constellation:add-property', handleAddPropertyEvent);
		document.removeEventListener('click', handleDocClick);
		if (focusRaf !== null) cancelAnimationFrame(focusRaf);
		// Flush any pending save before the component is destroyed.
		// G3 — a read-only view never writes (WA#6); still clear the timer to avoid a leak.
		if (saveTimeout) {
			clearTimeout(saveTimeout);
			// Sweep-2026-07-18 #1 (APP-KILLER) — flush ONLY when this instance still owns the note
			// it mounted for. On a note swap the live tabId/filePath already point at the INCOMING
			// note (B) while editableProps still hold the OUTGOING note's (A) frontmatter; using the
			// live identity here spliced A's props onto B's model+disk AND stomped B's tab.content
			// (both writes below). The identity gate makes this a clean skip on swap — no corruption
			// — while a genuine close/switch of the mounted note still persists its pending edit. The
			// snapshot targets defend a second time via editNoteProps' own expectPath guard.
			// 2026-07-22 inspection (APP-KILLER) — the PROPS-channel twin of hazard #6.
			// NoteEditor's body flush is gated on isReseeding/isCascading; this one was
			// not. On a watcher adopt the model is re-based from disk and the {#key}
			// block is torn down — and this teardown then wrote THIS instance's
			// pre-adopt props back over the freshly-adopted ones, durably reverting an
			// external frontmatter edit with no error, no conflict and no banner. The
			// identity gate below cannot catch it: tabId and filePath are unchanged, only
			// reloadVersion moved.
			if (isReseeding(filePath) || isCascading(filePath)) return;
			if (!readOnly && mountedTabId && mountedFilePath && tabId === mountedTabId && filePath === mountedFilePath) {
				/* Direct mutation so onflush reads fresh properties */
				const tab = get(openTabs).find(t => t.id === mountedTabId);
				if (tab) tab.content = buildFullContent(editableProps, body);
				// MIG-107 Slice 4 — the teardown flush commits through the SAME intent path. It used to
				// replay this instance's whole array, which is how an unmount reverted another writer's
				// frontmatter; now it can only touch keys this panel was actually showing.
				commitAndSave(mountedTabId, mountedFilePath).catch((e) => console.error('[PropertyEditor] Flush save failed:', e));
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
			} else if (newType === 'nested-object-list') {
				// MIG-022 §A.3 — switching TO nested-object-list seeds an
				// empty rows array. Existing flat value is preserved in
				// .value but no longer canonical (the widget reads
				// nestedObjects). User adds rows via the widget.
				if (!updated.nestedObjects) updated.nestedObjects = [];
				updated.listItems = undefined;
			} else if (newType !== 'list') {
				if (p.type === 'list' && p.listItems) {
					updated.value = p.listItems.join(', ');
				}
				updated.listItems = undefined;
				// Switching AWAY from nested-object-list — drop nested
				// rows; .value carries the compact summary as the
				// best-effort fallback.
				if (p.type === 'nested-object-list') {
					updated.nestedObjects = undefined;
				}
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
		// PJ-065 — both 'parent' and 'contains' author as a 'list' property (chips), each item
		// auto-wrapped to [[ ]]. 'list' (not 'link') is deliberate: the link-input oninput
		// double-wrapped into [[[triple]]]. setType converts the value + persists + debouncedSaves.
		const targetType = structuralKeyType(newKey);
		if (targetType && editableProps[idx].type !== targetType) {
			setType(idx, targetType);
			return;
		}
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

	// MIG-077 §F-Editor — exported so NotePane's frontmatter RC can call them via bind:this.
	export function addProperty() {
		editableProps = [...editableProps, { key: '', value: '', type: 'text' }];
	}

	export function removeProperty(idx: number) {
		editableProps = editableProps.filter((_, i) => i !== idx);
		debouncedSave();
	}

	// §C — Gregorian→cultural-date converter (ALL selected calendars). The note's primary date = `date:` if
	// present, else `created:`; if neither, fall back to the file's creation date. A "+ X" button appears per
	// non-Gregorian calendar the user has selected (primary/secondary), only while that calendar's property
	// is absent. Writes through the SAME path as a manual edit (mutate editableProps → debouncedSave).
	const CAL_LABEL_KEY: Record<string, string> = {
		hijri: 'propertyEditor.hijri', 'solar-hijri': 'propertyEditor.jalali', hebrew: 'propertyEditor.hebrew',
		indian: 'propertyEditor.saka', buddhist: 'propertyEditor.buddhist', chinese: 'propertyEditor.chinese', korean: 'propertyEditor.korean',
	};
	const selectedCulturalCals = $derived.by(() => {
		const out: { system: CalendarSystem; key: string; labelKey: string }[] = [];
		const seen = new Set<string>();
		for (const s of [$appSettings.calendarPrimarySystem, $appSettings.calendarSecondarySystem]) {
			if (!s || s === 'gregorian' || s === 'none' || seen.has(s)) continue;
			const k = frontmatterKey(s as CalendarSystem);
			if (k) { seen.add(s); out.push({ system: s as CalendarSystem, key: k, labelKey: CAL_LABEL_KEY[s] ?? 'propertyEditor.hijri' }); }
		}
		return out;
	});
	const primaryDateKey = $derived(
		editableProps.some((p) => p.key === 'date') ? 'date'
		: editableProps.some((p) => p.key === 'created') ? 'created'
		: null
	);
	const hasProp = (key: string) => editableProps.some((p) => p.key === key);
	async function addCulturalDate(system: CalendarSystem, key: string) {
		const entryTab = tabId, entryPath = filePath; // capture identity — re-verified after the awaits below
		let iso: string | undefined;
		if (primaryDateKey) {
			const dp = editableProps.find((p) => p.key === primaryDateKey);
			const m = String(dp?.value ?? '').match(/\d{4}-\d{2}-\d{2}/);
			if (m) iso = m[0];
		}
		if (!iso) {
			try {
				const meta = await invoke<{ created: number }>('get_file_metadata', { filePath });
				if (meta?.created) iso = new Date(meta.created * 1000).toLocaleDateString('en-CA'); // YYYY-MM-DD (local)
			} catch { /* metadata unavailable — skip */ }
		}
		if (!iso) return;
		try {
			await applyCalendarPrefs($appSettings.calendarCorrections ?? {}, $appSettings.calendarCalculationMode ?? 'astronomical');
			const val = await culturalDateString(system, iso);
			// teardown OR a tab switch during the await — the standalone Properties panel isn't {#key}-remounted,
			// so `mounted` alone misses a tab switch; re-verify the live tab identity before writing.
			if (!mounted || tabId !== entryTab || filePath !== entryPath) return;
			if (!val) return;
			const i = editableProps.findIndex((pp) => pp.key === key);
			if (i >= 0) editableProps = editableProps.map((pp, j) => (j === i ? { ...pp, value: val } : pp));
			else editableProps = [...editableProps, { key, value: val, type: 'text' as PropertyType }];
			debouncedSave();
		} catch { /* engine unavailable — never block */ }
	}

	function addTag(idx: number, tag: string) {
		if (!tag.trim()) return;
		editableProps = editableProps.map((p, i) => {
			if (i !== idx) return p;
			// PJ-065 — a structural link-list (parent:/contains:) stores items as [[wikilinks]]
			// so the structural reader registers them. Normalize to exactly [[name]]: strip any
			// brackets the user typed, then wrap once — a typed name OR a pasted [[X]] both land
			// as [[X]], never a double/triple wrap.
			let item = tag.trim();
			if (STRUCTURAL_LIST_LINK_KEYS.has(p.key) && item) {
				item = `[[${item.replace(/^\[+|\]+$/g, '')}]]`;
			}
			const items = [...(p.listItems ?? []), item];
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

	// MIG-022 §A.3 (D-A4.α, 2026-05-11) — ikhtilāf widget mutation
	// helpers. Each operation rebuilds the nestedObjects array and
	// recomputes the compact `value` summary used by legacy consumers
	// + search ("Hanafī: permissible | Mālikī: discouraged").
	const NESTED_FIELD_KEYS = ['school', 'position'] as const;
	function rebuildNestedSummary(nested: Array<Record<string, string>>): string {
		return nested
			.map((o) => Object.entries(o).map(([k, v]) => `${k}: ${v}`).join(' / '))
			.join(' | ');
	}
	function updateNestedField(propIdx: number, rowIdx: number, fieldKey: string, fieldVal: string) {
		editableProps = editableProps.map((p, i) => {
			if (i !== propIdx) return p;
			const nested = (p.nestedObjects ?? []).map((row, ri) =>
				ri === rowIdx ? { ...row, [fieldKey]: fieldVal } : row,
			);
			return { ...p, nestedObjects: nested, value: rebuildNestedSummary(nested) };
		});
		debouncedSave();
	}
	function addNestedRow(propIdx: number) {
		editableProps = editableProps.map((p, i) => {
			if (i !== propIdx) return p;
			const nested = [...(p.nestedObjects ?? []), { school: '', position: '' }];
			return { ...p, nestedObjects: nested, value: rebuildNestedSummary(nested) };
		});
		debouncedSave();
	}
	function removeNestedRow(propIdx: number, rowIdx: number) {
		editableProps = editableProps.map((p, i) => {
			if (i !== propIdx) return p;
			const nested = (p.nestedObjects ?? []).filter((_, ri) => ri !== rowIdx);
			return { ...p, nestedObjects: nested, value: rebuildNestedSummary(nested) };
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

	/**
	 * MIG-107 Slice 4 — THE SWAP. Put this panel's edit into the model one property at a time, then
	 * write from the model.
	 *
	 * The old path handed `editableProps` to `saveTabContent`, which replaced the model's whole
	 * array. That is the defect: an array assembled from one panel's view silently deletes whatever
	 * another writer changed in the meantime (PJ-174 AK-2/AK-3). Now the planner turns the panel's
	 * rows into per-key operations, and it may only REMOVE keys in `seededKeys` — so a key this
	 * panel never saw cannot be reached at all.
	 *
	 * `propsAlreadyInModel: true` tells `saveTabContent` not to push an array; compose reads the
	 * model, which the intents have already updated. The auto-"updated" date rule is applied here
	 * through the SAME shared helper the legacy path uses, so the two cannot drift.
	 */
	async function commitAndSave(id: string, path: string): Promise<void> {
		if (!PROPS_SINGLE_OWNERSHIP) {
			await saveTabContent(id, path, editableProps, body);
			return;
		}
		const model = getModel(id);
		if (!model) { await saveTabContent(id, path, editableProps, body); return; } // no model → legacy
		const touched = touchedSince(seededRows, editableProps);
		const ops = planPropOps(withAutoUpdatedDate(editableProps), model.props, seededKeys, touched);
		applyPropOps(ops, {
			setValue: (k, v, o) => editPropValue(id, k, v, o, path),
			add: (pr) => addPropTo(id, pr, path),
			remove: (k) => removePropFrom(id, k, path),
			order: (k, before) => reorderPropsIn(id, k, before, path),
		});
		// The panel now knows about everything the model holds — including keys another writer added
		// that this commit deliberately left alone. Without this, the NEXT commit would still treat
		// them as unseen and could never remove them even when the user does delete them.
		seededKeys = new Set(getModel(id)?.props.map((p) => p.key) ?? []);
		// Committed: the panel's rows are now the model's, so nothing of this panel's is ahead of it.
		seededRows = (getModel(id)?.props ?? []).map((p) => ({ ...p, listItems: p.listItems ? [...p.listItems] : undefined }));
		await saveTabContent(id, path, editableProps, body, false, true);
	}

	function debouncedSave() {
		// G3 — a read-only view never persists a property edit (WA#6). Returning here
		// keeps the note's model clean so the cross-window freshness sync always adopts.
		if (readOnly) return;
		// MIG-087 §E (item 2) — live props-count observer. Every real edit routes
		// through here (the init-sync $effect does NOT), so this fires only on user
		// edits — the exact analog of §C's onLiveStats(onDocChange). Report the
		// count of non-empty-key properties (matches what reconstructFrontmatter
		// saves, so the live count converges to the on-save baseline — no flicker).
		// One-way: it never writes back into editor content.
		onLiveProps?.(tabId, editableProps.filter(p => p.key.trim().length > 0).length);
		clearTimeout(saveTimeout);
		saveTimeout = setTimeout(async () => {
			// Clear the handle as the timer FIRES. It was only ever assigned, never
			// reset, so `if (saveTimeout)` in onDestroy stayed truthy forever after the
			// first property edit — arming the teardown flush on every later unmount.
			saveTimeout = undefined;
			saving = true;
			try {
				/* Update tab content in store via direct mutation (no store.update = no cascade).
				   This ensures onflush reads fresh properties when the tab is closed. */
				const tab = get(openTabs).find(t => t.id === tabId);
				if (tab) tab.content = buildFullContent(editableProps, body);
				await commitAndSave(tabId, filePath);
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
	<!-- G3 — in read-only mode the whole property body is inert (non-interactive), so it
	     matches the read-only body (CM6 editable:false) + title (readonly) at the SAME layer;
	     the debouncedSave/onDestroy write-gate above is the safety belt. display:contents keeps
	     the existing flex layout (the rows stay layout children of .property-editor). -->
	<div style="display: contents" inert={readOnly || undefined}>
	{#each editableProps as prop, idx}
		{@const iconInfo = getIcon(prop)}
		{@const isEmpty = !prop.value || (prop.type === 'list' && (!prop.listItems || prop.listItems.length === 0))}
		<!-- svelte-ignore a11y_no_static_element_interactions -->
		<div class="pe-row"
			oncontextmenu={(e) => { e.preventDefault(); onPropContextMenu?.(prop, idx, e.clientX, e.clientY); }}
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
			<!-- PJ-136 — a nested-map row is read-only as a WHOLE, key included. Leaving
			     the key editable made the label a smaller lie: renaming `source` looked
			     like it worked, and then did nothing at all, because the write path
			     refuses this type outright. A control that silently no-ops is the same
			     silent-failure class we are trying to remove. Reuses the existing
			     special-key span — same markup, same CSS, nothing duplicated. -->
			{#if iconInfo.isSpecial || prop.type === 'nested-map'}
				<span class="pe-key pe-key-special">{prop.key}</span>
			{:else}
				<div class="pe-key-wrap">
					<input class="pe-key" type="text" dir="auto" value={prop.key}
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
				{@const orderedItems = orderTaxonomyItems(items, axis)}
				<div class="pe-taxo-wrap">
					<div class="pe-taxo-row">
						<div class="pe-taxo-pills">
							{#if orderedItems.length === 0}
								<span class="pe-taxo-empty">{$t('propertyEditor.empty')}</span>
							{:else}
								{#each orderedItems as row (row.id)}
									{@const color = axis === 'horizontal' ? tierColorForId(row.id) : null}
									<div class="pe-taxo-pill-line" style:padding-inline-start={`${row.depth * 16}px`}>
										{#if row.depth > 0}
											<span class="pe-taxo-connector" aria-hidden="true">↳</span>
										{/if}
										<span class="pe-taxo-pill" style:--taxo-color={color ?? 'transparent'}>
											<span class="pe-taxo-label">{taxonomyLabel(row.id, axis)}</span>
											<button class="pe-taxo-x" onclick={() => removeTaxonomyValue(idx, row.id)} title={$t('propertyEditor.delete')}>&times;</button>
										</span>
									</div>
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
						dir="auto"
						value={prop.value}
						placeholder={$t('propertyEditor.stagePlaceholder')}
						oninput={(e) => { updateValue(idx, (e.target as HTMLInputElement).value); stageUserNavigated = false; stageMenuOpen = idx; }}
						onfocus={() => { stageMenuOpen = idx; stageHighlight = stageIndexOf(opts, prop.value); stageUserNavigated = false; }}
						onclick={(e) => { e.stopPropagation(); if (stageMenuOpen !== idx) stageHighlight = stageIndexOf(opts, prop.value); stageMenuOpen = idx; }}
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
							<span class="pe-tag" dir="auto">
								{STRUCTURAL_LIST_LINK_KEYS.has(prop.key) ? tag.replace(/^\[+|\]+$/g, '') : tag}
								<button class="pe-tag-x" onclick={() => removeTag(idx, tagIdx)}>&times;</button>
							</span>
						{/each}
					{/if}
					<input class="pe-tag-input" type="text" dir="auto"
						placeholder={isEmpty ? $t('propertyEditor.empty') : $t('propertyEditor.addPlaceholder')}
						value={tagInputs[idx] ?? ''}
						oninput={(e) => { tagInputs = { ...tagInputs, [idx]: (e.target as HTMLInputElement).value }; }}
						onkeydown={(e) => handleTagKeydown(e, idx)} />
				</div>
			{:else if prop.type === 'link'}
				{@const linkName = prop.value.replace(/^\[\[|\]\]$/g, '')}
				<div class="pe-link-wrap">
					{#if linkName && onNoteClick}
						<button class="pe-link-clickable" dir="auto" onclick={() => handleLinkClick(prop.value)}
							title={linkName}>
							<span class="pe-link-icon">\uD83D\uDD17</span>
							{linkName}
						</button>
					{:else}
						<span class="pe-link-bracket">[[</span>
						<input class="pe-val pe-link-input" type="text" dir="auto"
							size={Math.max((linkName?.length ?? 0) + 1, 5)}
							placeholder={$t('propertyEditor.empty')}
							value={linkName}
							oninput={(e) => updateValue(idx, `[[${(e.target as HTMLInputElement).value}]]`)} />
						<span class="pe-link-bracket">]]</span>
					{/if}
				</div>
			{:else if prop.type === 'nested-map'}
				<!-- ⚠ TEMPORARY — CONTAINMENT, NOT A DECISION. Boss, 2026-07-22:
				     "showing it read-only is a temporary procedure, until you research
				     for a solution and fix it for good." The end state is that nested
				     fields are EDITABLE like any other property; `source: {title,
				     author, year}` is knowledge, and locking it is a stopgap. The cure
				     is PJ-137 — retire the hand-rolled `store.parseFrontmatter` so the
				     panel and the write path share ONE YAML-document model, after which
				     editing is safe by construction instead of by refusal. Do not treat
				     this branch as settled design.

				     PJ-136 (Boss ruling 2026-07-22) — a property holding a nested block
				     (`source:` with `title` / `author` / `year` under it) SHOWS what it
				     holds, read-only. It previously fell through to the text input and
				     drew as "Empty" — a label that was both false and an invitation:
				     typing into it replaced the whole block, silently.

				     Read-only here is the visible half; `composeFrontmatter` refusing to
				     write or splice this type is the half that actually protects the data,
				     because a widget only protects it while every caller keeps it inert. -->
				<div class="pe-nested-map" dir="auto">
					{#if prop.nestedKeys && prop.nestedKeys.length > 0}
						{#each prop.nestedKeys as childKey}
							<span class="pe-nested-chip">{childKey}</span>
						{/each}
					{:else}
						<span class="pe-nested-note">{$t('propertyEditor.nestedMapEmpty')}</span>
					{/if}
					<span class="pe-nested-note" title={$t('propertyEditor.nestedMapHint')}
						>{$t('propertyEditor.nestedMapReadOnly')}</span>
				</div>
			{:else if prop.type === 'nested-object-list'}
				<!-- MIG-022 \u00A7A.3 (D-A4.\u03B1, 2026-05-11) \u2014 ikhtil\u0101f widget.
				     Renders the structured rows from prop.nestedObjects
				     as a list of {school, position} editor cards with
				     add + remove. Source-of-truth is prop.nestedObjects;
				     the compact prop.value summary is recomputed on each
				     mutation for legacy consumers (search, etc.). -->
				<div class="pe-ikhtilaf">
					{#if prop.nestedObjects && prop.nestedObjects.length > 0}
						{#each prop.nestedObjects as row, rowIdx}
							<div class="pe-ikhtilaf-row">
								<div class="pe-ikhtilaf-fields">
									<input
										class="pe-val pe-ikhtilaf-input"
										type="text"
										dir="auto"
										placeholder={$t('propertyEditor.ikhtilafSchoolPlaceholder') || 'School'}
										value={row.school ?? ''}
										aria-label={$t('propertyEditor.ikhtilafSchoolLabel') || 'School'}
										oninput={(e) => updateNestedField(idx, rowIdx, 'school', (e.target as HTMLInputElement).value)} />
									<input
										class="pe-val pe-ikhtilaf-input"
										type="text"
										dir="auto"
										placeholder={$t('propertyEditor.ikhtilafPositionPlaceholder') || 'Position'}
										value={row.position ?? ''}
										aria-label={$t('propertyEditor.ikhtilafPositionLabel') || 'Position'}
										oninput={(e) => updateNestedField(idx, rowIdx, 'position', (e.target as HTMLInputElement).value)} />
								</div>
								<button
									class="pe-ikhtilaf-remove"
									onclick={() => removeNestedRow(idx, rowIdx)}
									title={$t('propertyEditor.ikhtilafRemoveRow') || 'Remove row'}>
									<svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M18 6L6 18M6 6l12 12"/></svg>
								</button>
							</div>
						{/each}
					{/if}
					<button class="pe-ikhtilaf-add" onclick={() => addNestedRow(idx)}>
						+ {$t('propertyEditor.ikhtilafAddRow') || 'Add school'}
					</button>
				</div>
			{:else}
				<input class="pe-val" type="text" dir="auto" value={prop.value}
					placeholder={$t('propertyEditor.empty')}
					oninput={(e) => updateValue(idx, (e.target as HTMLInputElement).value)} />
			{/if}

			{#if prop.key === primaryDateKey}
				{#each selectedCulturalCals as cal (cal.key)}
					{#if !hasProp(cal.key)}
						<button class="pe-hijri-btn" onclick={() => addCulturalDate(cal.system, cal.key)} title={$t('propertyEditor.addCulturalDate') || 'Add the equivalent date'}>+ {$t(cal.labelKey)}</button>
					{/if}
				{/each}
			{/if}
			<!-- PJ-136 — no delete on a nested-map row either. `composeFrontmatter`
			     refuses to splice the block, so the button would remove the row from the
			     panel, change nothing on disk, and let the row return on reload. Offering
			     an action that cannot happen is worse than not offering it. -->
			{#if prop.type !== 'nested-map'}
				<button class="pe-del" onclick={() => removeProperty(idx)} title={$t('propertyEditor.delete')}>
					<svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M18 6L6 18M6 6l12 12"/></svg>
				</button>
			{/if}
		</div>
	{/each}

	{#if primaryDateKey === null}
		{#each selectedCulturalCals as cal (cal.key)}
			{#if !hasProp(cal.key)}
				<button class="pe-add pe-hijri-add" onclick={() => addCulturalDate(cal.system, cal.key)}>+ {$t(cal.labelKey)}</button>
			{/if}
		{/each}
	{/if}
	<button class="pe-add" bind:this={addBtnRef} onclick={addProperty}>
		+ {$t('propertyEditor.addProperty')}
	</button>
	</div>
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
	.pe-title { font-size: calc(0.78rem * var(--rs-scale, 1)); font-weight: 600; color: var(--text-muted); }
	.pe-chevron { transition: transform 0.2s; flex-shrink: 0; color: var(--text-muted); }
	.pe-chevron.collapsed { transform: rotate(-90deg); }
	:global([dir="rtl"]) .pe-chevron.collapsed { transform: rotate(90deg); }
	.pe-saving { font-size: calc(0.7rem * var(--rs-scale, 1)); color: var(--interactive-accent); }

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
		font-size: calc(0.8rem * var(--rs-scale, 1)); color: var(--text-faint);
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
		color: var(--text-faint); cursor: pointer; font-size: calc(0.8rem * var(--rs-scale, 1));
		padding: 0;
	}
	.pe-type-btn:hover { background: var(--background-modifier-border); color: var(--text-muted); }
	.pe-type-btn.pe-special { font-weight: 700; font-size: calc(0.9rem * var(--rs-scale, 1)); }

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
		font-size: calc(0.8rem * var(--rs-scale, 1)); color: var(--text-normal); font-family: inherit;
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
		font-size: calc(1.05rem * var(--rs-scale, 1)); line-height: 1;
		flex-shrink: 0;
		opacity: 0.95;
	}
	.pe-stage-input {
		flex: 1; min-width: 0; box-sizing: border-box;
		border: none; background: none; padding: 3px 4px;
		font-size: calc(0.85rem * var(--rs-scale, 1)); color: var(--text-normal);
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
		font-size: calc(0.95rem * var(--rs-scale, 1)); color: var(--text-normal); font-family: inherit;
		text-align: start;
	}
	.pe-stage-option:hover,
	.pe-stage-option.pe-stage-active { background: var(--background-modifier-hover); }
	.pe-stage-emoji { font-size: calc(1.2rem * var(--rs-scale, 1)); line-height: 1; flex-shrink: 0; }
	.pe-stage-label { flex: 1; }

	/* Key input + suggestions */
	.pe-key-wrap { position: relative; flex-shrink: 0; width: auto; min-width: 50px; max-width: 100px; }

	.pe-key {
		width: 100%; box-sizing: border-box;
		border: none; background: none; padding: 3px 4px;
		font-size: calc(0.82rem * var(--rs-scale, 1)); font-weight: 500; color: var(--text-muted);
		font-family: inherit; outline: none;
		border-radius: 3px; text-align: start;
	}
	.pe-key:focus { background: var(--background-primary); box-shadow: 0 0 0 1px hsla(var(--accent-h), var(--accent-s), var(--accent-l), 0.27); }
	.pe-key-special {
		display: inline-block; width: 100px; min-width: 70px; flex-shrink: 0;
		font-weight: 600; color: var(--text-accent);
		cursor: default; user-select: none;
		padding: 3px 4px; font-size: calc(0.82rem * var(--rs-scale, 1)); text-align: end;
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
		font-size: calc(0.78rem * var(--rs-scale, 1)); color: var(--text-normal); font-family: inherit;
		text-align: start;
	}
	.pe-suggest-item:hover, .pe-suggest-active { background: var(--background-modifier-hover); }
	.pe-suggest-key { font-weight: 500; }
	.pe-suggest-ar { color: var(--text-faint); font-size: calc(0.74rem * var(--rs-scale, 1)); }

	/* Value inputs */
	.pe-val {
		flex: 1; min-width: 0;
		border: none; background: none; padding: 3px 6px;
		font-size: calc(0.82rem * var(--rs-scale, 1)); color: var(--text-normal);
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
	.pe-checkbox-label { font-size: calc(0.78rem * var(--rs-scale, 1)); color: var(--text-muted); }

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
		font-size: calc(0.85rem * var(--rs-scale, 1)); color: var(--text-normal);
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
		/* MIG-088 Phase 1 — Style-Setter-controllable (Frontmatter → Property tags); fallbacks = today's look. */
		height: var(--pe-tag-height, var(--pill-height, 20px));
		padding: 0 8px;
		border-radius: var(--pe-tag-radius, var(--pill-radius, 10px));
		background: var(--pe-tag-bg, var(--background-modifier-border-focus)); color: var(--pe-tag-text-color, #fff);
		font-size: calc(0.75rem * var(--rs-scale, 1)); font-weight: var(--pill-weight, 700);
		line-height: 1; white-space: nowrap;
	}
	.pe-tag-x {
		border: none; background: none; color: rgba(255, 255, 255, 0.75);
		cursor: pointer; font-size: calc(0.8rem * var(--rs-scale, 1)); padding: 0 1px;
		line-height: 1;
	}
	.pe-tag-x:hover { color: #fff; }
	.pe-tag-input {
		flex: 1; min-width: 50px; border: none; background: none;
		padding: 2px 4px; font-size: calc(0.78rem * var(--rs-scale, 1)); color: var(--text-normal);
		font-family: inherit; outline: none;
	}
	.pe-tag-input::placeholder { color: var(--text-faint); font-style: italic; }

	/* Link */
	.pe-link-wrap {
		flex: 1; min-width: 0;
		display: flex; align-items: center; gap: 0;
	}
	.pe-link-bracket { color: var(--interactive-accent); font-size: calc(0.82rem * var(--rs-scale, 1)); font-weight: 600; flex-shrink: 0; }
	/* PJ-065 — size to content (via the `size` attr) so the closing ]] hugs the title
	   instead of stretching to the row's edge. Can shrink, never overflows the row. */
	.pe-link-input { flex: 0 1 auto; min-width: 4ch; max-width: 100%; color: var(--interactive-accent); }
	.pe-link-input::placeholder { color: var(--text-faint); font-style: italic; }

	.pe-link-clickable {
		border: none; background: none; padding: 2px 4px;
		font-size: calc(0.82rem * var(--rs-scale, 1)); color: var(--interactive-accent);
		cursor: pointer; font-family: inherit;
		text-decoration: none; border-radius: 3px;
		display: flex; align-items: center; gap: 4px;
		overflow: hidden; text-overflow: ellipsis; white-space: nowrap;
		max-width: 100%;
	}
	.pe-link-clickable:hover { text-decoration: underline; background: var(--background-modifier-hover); }
	.pe-link-icon { font-size: calc(0.75rem * var(--rs-scale, 1)); flex-shrink: 0; }

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

	/* §C — inline "+ Hijri" converter button (shows on the note's date row, hover-revealed) */
	.pe-hijri-btn {
		flex-shrink: 0; height: 20px; padding: 0 7px;
		display: inline-flex; align-items: center; gap: 2px;
		border: 1px solid var(--background-modifier-border); border-radius: 10px;
		background: none; color: var(--text-muted, var(--color-base-50));
		font-size: calc(11px * var(--rs-scale, 1)); line-height: 1; white-space: nowrap; cursor: pointer;
		opacity: 0; transition: opacity 0.15s;
	}
	.pe-row:hover .pe-hijri-btn { opacity: 0.85; }
	.pe-hijri-btn:hover { opacity: 1; background: var(--background-modifier-hover); color: var(--text-normal); }
	.pe-hijri-add { color: var(--text-muted, var(--color-base-50)); }

	/* Add button */
	.pe-add {
		display: block; width: 100%; margin-top: 6px;
		border: 1px dashed var(--background-modifier-border); border-radius: 4px;
		background: none; padding: 5px 8px;
		color: var(--text-faint); font-size: calc(0.78rem * var(--rs-scale, 1)); font-family: inherit;
		cursor: pointer; text-align: start;
	}
	.pe-add:hover { border-color: var(--interactive-accent); color: var(--interactive-accent); }

	/* MIG-022 §A.3 (D-A4.α) — ikhtilāf widget. List of {school,
	   position} editor cards. The two inputs sit side-by-side on
	   wide rows + stack on narrow ones (CSS-only flex wrap). */
	.pe-ikhtilaf {
		flex: 1; min-width: 0;
		display: flex; flex-direction: column; gap: 6px;
	}
	.pe-ikhtilaf-row {
		display: flex; align-items: stretch; gap: 6px;
		padding: 4px;
		background: var(--background-secondary-alt, var(--background-secondary));
		border: 1px solid var(--background-modifier-border);
		border-radius: 4px;
	}
	.pe-ikhtilaf-fields {
		flex: 1; min-width: 0;
		display: flex; flex-wrap: wrap; gap: 4px;
	}
	.pe-ikhtilaf-input {
		flex: 1; min-width: 100px;
		padding: 3px 6px;
		border: 1px solid var(--background-modifier-border);
		border-radius: 3px;
		background: var(--background-primary);
		font-size: calc(0.85rem * var(--rs-scale, 1)); font-family: inherit;
	}
	.pe-ikhtilaf-remove {
		width: 24px; height: 24px;
		display: flex; align-items: center; justify-content: center;
		border: none; background: none; border-radius: 3px;
		color: var(--color-base-40); cursor: pointer; padding: 0;
		flex-shrink: 0;
	}
	.pe-ikhtilaf-remove:hover {
		background: var(--background-modifier-error-hover);
		color: var(--text-error);
	}
	.pe-ikhtilaf-add {
		display: block; align-self: flex-start;
		margin-top: 2px;
		border: 1px dashed var(--background-modifier-border); border-radius: 4px;
		background: none; padding: 4px 10px;
		color: var(--text-faint); font-size: calc(0.78rem * var(--rs-scale, 1)); font-family: inherit;
		cursor: pointer;
	}
	.pe-ikhtilaf-add:hover {
		border-color: var(--interactive-accent);
		color: var(--interactive-accent);
	}

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
		display: flex; flex-direction: column; align-items: flex-start; gap: 3px;
	}
	.pe-taxo-pill-line {
		display: flex; align-items: center; gap: 4px;
		max-width: 100%; min-width: 0;
	}
	.pe-taxo-connector {
		color: var(--text-faint);
		font-size: calc(12px * var(--rs-scale, 1)); line-height: 1;
		flex-shrink: 0;
		user-select: none;
	}
	/* RTL: mirror the ↳ glyph so it reads right-to-left as ↲, matching the
	   indent direction. Eisa correction 2026-05-09 (§1D' Stage 3.4). */
	:global([dir="rtl"]) .pe-taxo-connector {
		transform: scaleX(-1);
	}
	.pe-taxo-empty {
		color: var(--text-faint); font-style: italic; font-size: calc(0.78rem * var(--rs-scale, 1));
	}
	/* PJ-136 — the read-only nested-map summary. Deliberately NOT input-shaped: no
	   border, no field background, no caret. The row must not look typeable, because
	   looking typeable is what caused the data loss. */
	.pe-nested-map {
		display: flex; flex-wrap: wrap; align-items: center; gap: 4px;
		min-height: var(--pill-height, 20px);
		user-select: none;
	}
	.pe-nested-chip {
		display: inline-flex; align-items: center;
		height: var(--pill-height, 20px);
		padding: 0 6px;
		border-radius: 3px;
		background: var(--background-modifier-border);
		color: var(--text-muted);
		font-size: calc(0.78rem * var(--rs-scale, 1));
	}
	.pe-nested-note {
		color: var(--text-faint); font-style: italic;
		font-size: calc(0.72rem * var(--rs-scale, 1));
	}
	.pe-taxo-pill {
		display: inline-flex; align-items: center; gap: 4px;
		box-sizing: border-box;
		height: var(--pill-height, 20px);
		padding: 0 8px;
		/* MIG-088 Phase 1 — Style-Setter-controllable (Frontmatter → Taxonomy pills); fallbacks = today's look. */
		border-radius: var(--pe-taxo-radius, var(--pill-radius, 10px));
		background: var(--pe-taxo-bg, var(--background-modifier-border-focus)); color: var(--pe-taxo-text-color, #fff);
		font-size: calc(0.75rem * var(--rs-scale, 1)); font-weight: var(--pill-weight, 700);
		line-height: 1; white-space: nowrap;
		border-inline-start: 3px solid var(--taxo-color);
	}
	.pe-taxo-label { display: inline-block; }
	.pe-taxo-x {
		border: none; background: none; color: rgba(255, 255, 255, 0.75);
		cursor: pointer; font-size: calc(0.8rem * var(--rs-scale, 1)); padding: 0 1px; line-height: 1;
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
		font-size: calc(10px * var(--rs-scale, 1));
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
