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
	toggleExpand?: (target: ContextTarget) => void;
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

export function buildContextMenu(target: ContextTarget, a: ContextActions): MenuItem[] {
	const $t = get(t);

	if (target.kind === 'note') {
		const openGroup: MenuItem[] = [];
		if (a.open) openGroup.push({ label: $t('contextMenu.open'), icon: '📂', action: () => a.open!(target) });
		if (a.openInNewTab) openGroup.push({ label: $t('contextMenu.openInNewTab'), icon: '📑', action: () => a.openInNewTab!(target) });

		const editGroup: MenuItem[] = [];
		if (a.rename) editGroup.push({ label: $t('actions.rename'), icon: '✏️', action: () => a.rename!(target) });
		if (a.move) editGroup.push({ label: $t('contextMenu.move'), icon: '📦', action: () => a.move!(target) });
		if (a.addTag) editGroup.push({ label: $t('contextMenu.addTag'), icon: '🏷️', action: () => a.addTag!(target) });

		const utilGroup: MenuItem[] = [];
		if (a.copyPath) utilGroup.push({ label: $t('contextMenu.copyPath'), icon: '📋', action: () => a.copyPath!(target) });
		if (a.copyName) utilGroup.push({ label: $t('contextMenu.copyName'), icon: '📋', action: () => a.copyName!(target) });
		if (a.revealInTree) utilGroup.push({ label: $t('contextMenu.revealInTree'), icon: '🎯', action: () => a.revealInTree!(target) });
		if (a.suggestSources && target.isMarkdown) utilGroup.push({ label: $t('sources.contextMenu.suggest'), icon: '✨', action: () => a.suggestSources!(target) });

		const dangerGroup: MenuItem[] = [];
		if (a.delete) dangerGroup.push({ label: $t('actions.delete'), icon: '🗑️', action: () => a.delete!(target), danger: true });

		return joinGroups([openGroup, editGroup, utilGroup, dangerGroup]);
	}

	// folder or library
	const createGroup: MenuItem[] = [];
	if (a.newNote) createGroup.push({ label: $t('actions.newNote'), icon: '📄', action: () => a.newNote!(target) });
	if (a.newFolder) createGroup.push({ label: $t('actions.newFolder'), icon: '📁', action: () => a.newFolder!(target) });
	if (a.newBase) createGroup.push({ label: $t('actions.newBase'), icon: '▦', action: () => a.newBase!(target) });

	const expandGroup: MenuItem[] = [];
	if (a.toggleExpand) expandGroup.push({ label: target.expanded ? $t('contextMenu.collapse') : $t('contextMenu.expand'), icon: target.expanded ? '⊟' : '⊞', action: () => a.toggleExpand!(target) });

	// rename/move are passed only for folders (callers omit them for libraries,
	// matching the existing isLibraryRoot behaviour — library management lives in
	// the Library Manager).
	const editGroup: MenuItem[] = [];
	if (a.rename) editGroup.push({ label: $t('actions.rename'), icon: '✏️', action: () => a.rename!(target) });
	if (a.move) editGroup.push({ label: $t('contextMenu.move'), icon: '📦', action: () => a.move!(target) });

	const dangerGroup: MenuItem[] = [];
	if (a.delete) dangerGroup.push({ label: $t('actions.delete'), icon: '🗑️', action: () => a.delete!(target), danger: true });

	return joinGroups([createGroup, expandGroup, editGroup, dangerGroup]);
}
