// MIG-077 A3-R — the shared, CONTEXTUAL right-click menu builder.
//
// Boss steer (2026-06-14): right-click menus must be rich AND contextual —
// "it should be contextual and adapt to each type of function." One source of
// truth, but the output is a function of (object kind × surface capability):
//   - a note shows note actions; a folder shows folder actions; a library shows
//     create + expand/collapse only (no rename/delete — that lives in the
//     Library Manager);
//   - a tree node can Expand/Collapse, a flat list row cannot;
//   - the file tree won't offer "Reveal in tree" (it IS the tree).
//
// The contextual mechanism is the `actions` bag: each surface passes ONLY the
// callbacks it can fulfil, and the builder emits an item ONLY when its callback
// is provided AND it fits the target kind. Group-based separators keep the
// dividers clean no matter which items are present (no leading/trailing/double).
//
// Used by: OrgChart, the file tree (+layout getContextMenuItems), List-mode,
// Search, Sky View. Operations are reused, never reinvented (rename ->
// renamingPath; delete/reveal -> window events; copy -> clipboard; open-in-new-
// tab -> openNoteTab(newTab); suggest -> handleSuggestSourcesForNote; move ->
// the folder-picker over moveItem; addTag -> the safe frontmatter write path).

import { get } from 'svelte/store';
import { t } from '$lib/i18n';

export interface MenuItem {
	label?: string;
	icon?: string;
	action?: () => void;
	danger?: boolean;
	disabled?: boolean;
	separator?: boolean;
	/** MIG-077 §F-sub — a fly-out submenu (e.g. Copy path ▸). One level deep. */
	submenu?: MenuItem[];
}

export type NodeKind = 'note' | 'folder' | 'library';

export interface ContextTarget {
	kind: NodeKind;
	path: string;
	name: string;
	/** Notes: gate "Suggest sources" to markdown files. */
	isMarkdown?: boolean;
	/** Tree surfaces: choose the Expand vs Collapse label. */
	expanded?: boolean;
	/** Notes/folders/libraries: toggle the Bookmark vs Remove-bookmark label. */
	bookmarked?: boolean;
}

/**
 * The full vocabulary of right-click operations. A surface passes only the
 * callbacks it supports; absent callbacks => the item is omitted (contextual).
 */
export interface ContextActions {
	open?: (target: ContextTarget) => void;
	openInNewTab?: (target: ContextTarget) => void;
	rename?: (target: ContextTarget) => void;
	move?: (target: ContextTarget) => void;
	addTag?: (target: ContextTarget) => void;
	copyPath?: (target: ContextTarget) => void;
	copyName?: (target: ContextTarget) => void;
	revealInTree?: (target: ContextTarget) => void;
	suggestSources?: (target: ContextTarget) => void;
	delete?: (target: ContextTarget) => void;
	newNote?: (target: ContextTarget) => void;
	newFolder?: (target: ContextTarget) => void;
	newBase?: (target: ContextTarget) => void;
	/** MIG-103 D3 — bind a default template to this folder (deepest-wins, silent
	 *  at creation, off until set). The missing half: `folderTemplates` was read
	 *  by note creation but nothing could ever write it. */
	setFolderTemplate?: (target: ContextTarget) => void;
	toggleExpand?: (target: ContextTarget) => void;
	// MIG-077 §F — richer items (all reuse existing ops):
	bookmark?: (target: ContextTarget) => void;
	/** MIG-092 — hand-pick this note into a collection (membership only, never
	 *  writes the note). Emits an "Add to collection ▸" submenu when the picker
	 *  list is also provided. */
	addToCollection?: (target: ContextTarget, setId: string) => void;
	/** The collections offered in the "Add to collection ▸" submenu (id + already
	 *  resolved display name). Data, not a callback. */
	collectionsForPicker?: { id: string; name: string }[];
	/** "New collection…" tail item of the submenu — create + add in one step. */
	createCollectionAndAdd?: (target: ContextTarget) => void;
	/** Copy the path relative to the Library root. Pairs with `copyPath` (absolute)
	 *  → the builder emits a "Copy path ▸" submenu when BOTH are provided. */
	copyPathRelative?: (target: ContextTarget) => void;
	openInDefaultApp?: (target: ContextTarget) => void;
	showInExplorer?: (target: ContextTarget) => void;
	/** "Style…" — open the Style Setter focused on THIS surface's category
	 *  (the consumer passes the category via openStyleSetterToCategory). */
	style?: (target: ContextTarget) => void;
}

/** Flatten groups, inserting a separator only between non-empty groups. */
function joinGroups(groups: MenuItem[][]): MenuItem[] {
	const out: MenuItem[] = [];
	for (const g of groups) {
		if (g.length === 0) continue;
		if (out.length > 0) out.push({ separator: true });
		out.push(...g);
	}
	return out;
}

/** Copy path as a flat item, or a "Copy path ▸" fly-out (from Library folder /
 *  from system root) when BOTH the relative + absolute callbacks are wired. */
function copyPathItem($t: (k: string) => string, target: ContextTarget, a: ContextActions): MenuItem | null {
	if (a.copyPath && a.copyPathRelative) {
		return {
			label: $t('contextMenu.copyPath'), icon: '📋',
			submenu: [
				{ label: $t('contextMenu.fromLibraryFolder'), icon: '📁', action: () => a.copyPathRelative!(target) },
				{ label: $t('contextMenu.fromSystemRoot'), icon: '🖥️', action: () => a.copyPath!(target) },
			],
		};
	}
	if (a.copyPath) return { label: $t('contextMenu.copyPath'), icon: '📋', action: () => a.copyPath!(target) };
	return null;
}

/** Bookmark vs Remove-bookmark, label chosen from target.bookmarked. */
function bookmarkItem($t: (k: string) => string, target: ContextTarget, a: ContextActions): MenuItem | null {
	if (!a.bookmark) return null;
	return { label: target.bookmarked ? $t('contextMenu.removeBookmark') : $t('contextMenu.bookmark'), icon: '🔖', action: () => a.bookmark!(target) };
}

export function buildContextMenu(target: ContextTarget, a: ContextActions): MenuItem[] {
	const $t = get(t);
	// MIG-092 — English fallback for keys that land in all 15 locales at §10.
	const tl = (k: string, fb: string): string => { const v = $t(k); return v === k ? fb : v; };

	if (target.kind === 'note') {
		// Group order follows Obsidian's Note menu (MIG-077 §F): open · organize ·
		// copy/reveal · system · diagnostic · rename/delete. Each item is emitted
		// only when its callback is wired (contextual).
		const openGroup: MenuItem[] = [];
		if (a.open) openGroup.push({ label: $t('contextMenu.open'), icon: '📂', action: () => a.open!(target) });
		if (a.openInNewTab) openGroup.push({ label: $t('contextMenu.openInNewTab'), icon: '📑', action: () => a.openInNewTab!(target) });

		const orgGroup: MenuItem[] = [];
		if (a.move) orgGroup.push({ label: $t('contextMenu.move'), icon: '📦', action: () => a.move!(target) });
		const bm = bookmarkItem($t, target, a);
		if (bm) orgGroup.push(bm);
		// MIG-092 — "Add to collection ▸" submenu; auto-surfaces in every consumer
		// that wires the callback + picker list (the bookmark precedent).
		if (a.addToCollection && a.collectionsForPicker) {
			const picks: MenuItem[] = a.collectionsForPicker.map(c => ({
				label: c.name, icon: '🗂️', action: () => a.addToCollection!(target, c.id),
			}));
			if (a.createCollectionAndAdd) {
				picks.push({ separator: true });
				picks.push({ label: tl('contextMenu.newCollection', 'New collection…'), icon: '＋', action: () => a.createCollectionAndAdd!(target) });
			}
			orgGroup.push({ label: tl('contextMenu.addToCollection', 'Add to collection'), icon: '🗂️', submenu: picks });
		}
		if (a.addTag) orgGroup.push({ label: $t('contextMenu.addTag'), icon: '🏷️', action: () => a.addTag!(target) });

		const pathGroup: MenuItem[] = [];
		const cp = copyPathItem($t, target, a);
		if (cp) pathGroup.push(cp);
		if (a.copyName) pathGroup.push({ label: $t('contextMenu.copyName'), icon: '🏷', action: () => a.copyName!(target) });
		if (a.revealInTree) pathGroup.push({ label: $t('contextMenu.revealInTree'), icon: '🎯', action: () => a.revealInTree!(target) });

		const sysGroup: MenuItem[] = [];
		if (a.openInDefaultApp) sysGroup.push({ label: $t('contextMenu.openDefaultApp'), icon: '↗', action: () => a.openInDefaultApp!(target) });
		if (a.showInExplorer) sysGroup.push({ label: $t('contextMenu.showInExplorer'), icon: '🗂️', action: () => a.showInExplorer!(target) });

		const diagGroup: MenuItem[] = [];
		if (a.suggestSources && target.isMarkdown) diagGroup.push({ label: $t('sources.contextMenu.suggest'), icon: '✨', action: () => a.suggestSources!(target) });

		const styleGroup: MenuItem[] = [];
		if (a.style) styleGroup.push({ label: $t('contextMenu.style'), icon: '🎨', action: () => a.style!(target) });

		const finalGroup: MenuItem[] = [];
		if (a.rename) finalGroup.push({ label: $t('actions.rename'), icon: '✏️', action: () => a.rename!(target) });
		if (a.delete) finalGroup.push({ label: $t('actions.delete'), icon: '🗑️', action: () => a.delete!(target), danger: true });

		return joinGroups([openGroup, orgGroup, pathGroup, sysGroup, diagGroup, styleGroup, finalGroup]);
	}

	// folder or library — Obsidian order: create · expand · organize · copy/system · rename/delete.
	const createGroup: MenuItem[] = [];
	if (a.newNote) createGroup.push({ label: $t('actions.newNote'), icon: '📄', action: () => a.newNote!(target) });
	if (a.newFolder) createGroup.push({ label: $t('actions.newFolder'), icon: '📁', action: () => a.newFolder!(target) });
	if (a.newBase) createGroup.push({ label: $t('actions.newBase'), icon: '▦', action: () => a.newBase!(target) });

	const expandGroup: MenuItem[] = [];
	if (a.toggleExpand) expandGroup.push({ label: target.expanded ? $t('contextMenu.collapse') : $t('contextMenu.expand'), icon: target.expanded ? '⊟' : '⊞', action: () => a.toggleExpand!(target) });

	// rename/move are passed only for folders (callers omit them for libraries,
	// matching the existing isLibraryRoot behaviour — library management lives in
	// the Library Manager).
	const orgGroup: MenuItem[] = [];
	if (a.setFolderTemplate) orgGroup.push({ label: $t('templates.setFolderTemplate'), icon: '🗂️', action: () => a.setFolderTemplate!(target) });
	if (a.move) orgGroup.push({ label: $t('contextMenu.move'), icon: '📦', action: () => a.move!(target) });
	const bmf = bookmarkItem($t, target, a);
	if (bmf) orgGroup.push(bmf);

	const pathGroup: MenuItem[] = [];
	const cpf = copyPathItem($t, target, a);
	if (cpf) pathGroup.push(cpf);
	if (a.showInExplorer) pathGroup.push({ label: $t('contextMenu.showInExplorer'), icon: '🗂️', action: () => a.showInExplorer!(target) });

	const styleGroup: MenuItem[] = [];
	if (a.style) styleGroup.push({ label: $t('contextMenu.style'), icon: '🎨', action: () => a.style!(target) });

	const finalGroup: MenuItem[] = [];
	if (a.rename) finalGroup.push({ label: $t('actions.rename'), icon: '✏️', action: () => a.rename!(target) });
	if (a.delete) finalGroup.push({ label: $t('actions.delete'), icon: '🗑️', action: () => a.delete!(target), danger: true });

	return joinGroups([createGroup, expandGroup, orgGroup, pathGroup, styleGroup, finalGroup]);
}
