/**
 * App icon overrides — decision 3 of the Emoji & Icon Library plug-in.
 *
 * Every customizable shell icon has a stable `slot` id. The user can
 * replace the default SVG with any emoji or library icon via
 * `iconOverrides[slot]`. Stored in AppSettings.iconOverrides as a
 * Record<slot, iconRef> where iconRef is either:
 *   - a raw emoji character ("❤️")
 *   - a namespaced icon id ("lucide:heart", "phosphor:heart", ...)
 *
 * The shell reads its icon via `resolveIcon(slot)` which returns the
 * user's override if set, else null (component falls back to its
 * hard-coded default SVG).
 */

import { get } from 'svelte/store';
import { appSettings } from '$lib/libraries/store';
import { loadAllIcons, wrapForInsertion, type Icon } from '$lib/editor/iconSets';

/** Every overridable shell slot. Grouped for display in Settings. */
export const ICON_SLOTS = [
	// Dock
	{ group: 'Dock', slot: 'dock.search', label: 'Search' },
	{ group: 'Dock', slot: 'dock.skyview', label: 'Sky View' },
	{ group: 'Dock', slot: 'dock.orgchart', label: 'Organization chart' },
	{ group: 'Dock', slot: 'dock.constellationMap', label: 'Constellation Map' },
	{ group: 'Dock', slot: 'dock.constellationSight', label: 'Constellation Sight' },
	{ group: 'Dock', slot: 'dock.knowledgeHealth', label: 'Knowledge Health' },
	{ group: 'Dock', slot: 'dock.aiSkills', label: 'AI Skills' },
	{ group: 'Dock', slot: 'dock.dailyNote', label: 'Daily note' },
	{ group: 'Dock', slot: 'dock.index', label: 'Index' },
	{ group: 'Dock', slot: 'dock.globalTasks', label: 'Global Tasks' },
	{ group: 'Dock', slot: 'dock.search.lens', label: 'Lens' },
	{ group: 'Dock', slot: 'dock.settings', label: 'Settings' },
	{ group: 'Dock', slot: 'dock.secondScreen', label: 'Second Screen' },
	{ group: 'Dock', slot: 'dock.importer', label: 'Importer' },

	// Sidebar action toolbar
	{ group: 'Sidebar Toolbar', slot: 'sidebar.newNote', label: 'New note' },
	{ group: 'Sidebar Toolbar', slot: 'sidebar.newTable', label: 'New table' },
	{ group: 'Sidebar Toolbar', slot: 'sidebar.newFolder', label: 'New folder' },
	{ group: 'Sidebar Toolbar', slot: 'sidebar.newCapture', label: 'New capture' },
	{ group: 'Sidebar Toolbar', slot: 'sidebar.newLibrary', label: 'New library' },
	{ group: 'Sidebar Toolbar', slot: 'sidebar.collapseAll', label: 'Collapse all' },
	{ group: 'Sidebar Toolbar', slot: 'sidebar.expandAll', label: 'Expand all' },
	{ group: 'Sidebar Toolbar', slot: 'sidebar.sort', label: 'Sort' },

	// Layout bar (pane toggles)
	{ group: 'Layout Bar', slot: 'layout.leftSidebar', label: 'Left sidebar toggle' },
	{ group: 'Layout Bar', slot: 'layout.split', label: 'Split view' },
	{ group: 'Layout Bar', slot: 'layout.rightSidebar', label: 'Right sidebar toggle' },

	// Right sidebar inspector tabs
	{ group: 'Inspector Tabs', slot: 'inspector.properties', label: 'Properties' },
	{ group: 'Inspector Tabs', slot: 'inspector.backlinks', label: 'Backlinks' },
	{ group: 'Inspector Tabs', slot: 'inspector.tags', label: 'Tags' },
	{ group: 'Inspector Tabs', slot: 'inspector.star', label: 'Sky view (local)' },
	{ group: 'Inspector Tabs', slot: 'inspector.tasks', label: 'Tasks' },
	{ group: 'Inspector Tabs', slot: 'inspector.calendar', label: 'Calendar' },
	{ group: 'Inspector Tabs', slot: 'inspector.health', label: 'Knowledge Health' },
	{ group: 'Inspector Tabs', slot: 'inspector.provenance', label: 'Provenance' },
	{ group: 'Inspector Tabs', slot: 'inspector.review', label: 'Review' },

	// File tree
	{ group: 'File Tree', slot: 'tree.folder', label: 'Folder' },
	{ group: 'File Tree', slot: 'tree.folderOpen', label: 'Folder (open)' },
	{ group: 'File Tree', slot: 'tree.file', label: 'Note' },
	{ group: 'File Tree', slot: 'tree.baseFile', label: 'Base file' },
	{ group: 'File Tree', slot: 'tree.library', label: 'Library' },
	{ group: 'File Tree', slot: 'tree.universe', label: 'Universe' },
	{ group: 'File Tree', slot: 'tree.childUniverse', label: 'Child universe' },

	// Editor toolbar
	{ group: 'Editor Toolbar', slot: 'editor.bold', label: 'Bold' },
	{ group: 'Editor Toolbar', slot: 'editor.italic', label: 'Italic' },
	{ group: 'Editor Toolbar', slot: 'editor.underline', label: 'Underline' },
	{ group: 'Editor Toolbar', slot: 'editor.strikethrough', label: 'Strikethrough' },
	{ group: 'Editor Toolbar', slot: 'editor.highlight', label: 'Highlight' },
	{ group: 'Editor Toolbar', slot: 'editor.code', label: 'Inline code' },
	{ group: 'Editor Toolbar', slot: 'editor.heading', label: 'Heading' },
	{ group: 'Editor Toolbar', slot: 'editor.list', label: 'Bullet list' },
	{ group: 'Editor Toolbar', slot: 'editor.numberedList', label: 'Numbered list' },
	{ group: 'Editor Toolbar', slot: 'editor.checkbox', label: 'Checkbox' },
	{ group: 'Editor Toolbar', slot: 'editor.quote', label: 'Blockquote' },
	{ group: 'Editor Toolbar', slot: 'editor.link', label: 'Link' },
	{ group: 'Editor Toolbar', slot: 'editor.image', label: 'Image' },
	{ group: 'Editor Toolbar', slot: 'editor.table', label: 'Table' },

	// Callout types
	{ group: 'Callouts', slot: 'callout.note', label: 'Note' },
	{ group: 'Callouts', slot: 'callout.info', label: 'Info' },
	{ group: 'Callouts', slot: 'callout.tip', label: 'Tip' },
	{ group: 'Callouts', slot: 'callout.success', label: 'Success' },
	{ group: 'Callouts', slot: 'callout.warning', label: 'Warning' },
	{ group: 'Callouts', slot: 'callout.danger', label: 'Danger' },
	{ group: 'Callouts', slot: 'callout.question', label: 'Question' },
	{ group: 'Callouts', slot: 'callout.quote', label: 'Quote' },
	{ group: 'Callouts', slot: 'callout.example', label: 'Example' },
	{ group: 'Callouts', slot: 'callout.abstract', label: 'Abstract' },
] as const;

export type IconSlot = typeof ICON_SLOTS[number]['slot'];

let _iconMap: Map<string, Icon> | null = null;
let _loading: Promise<void> | null = null;

async function ensureIconsLoaded() {
	if (_iconMap) return;
	if (!_loading) {
		_loading = loadAllIcons().then(all => {
			_iconMap = new Map(all.map(i => [i.id, i]));
		});
	}
	await _loading;
}

/**
 * Resolve an icon reference (emoji char or "set:name" id) to a renderable
 * string. Emoji returned as-is; icon refs resolved via the cache. Returns
 * null if the ref is unknown (caller should fall back to default SVG).
 */
export async function resolveOverride(slot: string): Promise<string | null> {
	const settings = get(appSettings);
	const overrides = settings.iconOverrides ?? {};
	const ref = overrides[slot];
	if (!ref) return null;
	// Emoji (single Unicode code point or ZWJ sequence) — no colon
	if (!ref.includes(':')) return ref;
	await ensureIconsLoaded();
	const icon = _iconMap?.get(ref);
	if (!icon) return null;
	return wrapForInsertion(icon);
}

/** Synchronous peek at the override without waiting for icons to load.
 *  Returns the raw ref string ("lucide:heart" or an emoji) or null. */
export function peekOverride(slot: string): string | null {
	const settings = get(appSettings);
	return settings.iconOverrides?.[slot] ?? null;
}

export function setOverride(slot: string, ref: string | null) {
	const settings = get(appSettings);
	const next = { ...(settings.iconOverrides ?? {}) };
	if (ref) next[slot] = ref; else delete next[slot];
	// Dynamically import to keep this module free of circular deps
	import('$lib/libraries/store').then(({ updateSettings }) => {
		updateSettings({ iconOverrides: next });
	});
}

export function clearAllOverrides() {
	import('$lib/libraries/store').then(({ updateSettings }) => {
		updateSettings({ iconOverrides: {} });
	});
}
