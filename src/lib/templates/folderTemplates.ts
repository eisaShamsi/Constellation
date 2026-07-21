/**
 * MIG-103 D3 — a folder's default template: resolution, deepest-wins.
 *
 * **Boss-ruled contract (2026-07-21):** off by default · opt-in per context · one
 * template per context · **deepest wins** · applied silently · only onto a
 * still-empty note (guaranteed here — it runs at creation) · with an
 * excluded-folders carve-out.
 *
 * Extracted from `createNoteWithTemplate` in `+layout.svelte`, where the matching
 * was **substring-based** and therefore wrong:
 *
 * ```js
 * if (noteFolder.includes(normFolder) || noteFolder.endsWith(normFolder))
 * ```
 *
 * A folder configured as `Books` also matched `/Cookbooks/`, `/MyBooks/` and
 * `/x/Notebooks/y/` — silently applying a template to notes that were never meant
 * to get one. Depth was measured from the *configured* path's segment count
 * rather than from how much of the note's path actually matched, so "deepest
 * wins" was unreliable too.
 *
 * Here the match is a true **path-prefix** match on normalised segments, and the
 * winner is the **longest** matching prefix.
 */

/** Normalise a path for comparison: forward slashes, no trailing slash, lower-case. */
export function normalizeFolder(p: string): string {
	return p.replace(/\\/g, '/').replace(/\/+$/, '').toLowerCase();
}

/**
 * Does `folder` contain `noteFolder` — as a path, not as a substring?
 * True when they are the same folder, or `folder` is an ancestor directory.
 */
export function isAncestorOrSame(folder: string, noteFolder: string): boolean {
	const f = normalizeFolder(folder);
	const n = normalizeFolder(noteFolder);
	if (!f) return false;
	return n === f || n.startsWith(f + '/');
}

/**
 * The template configured for the deepest folder containing `noteFolder`.
 *
 * @param noteFolder  the folder the new note is being created in
 * @param map         `appSettings.folderTemplates` — folder path → template name
 * @param excluded    folders that must NEVER receive a template (the templates
 *                    directory itself: creating a template must not fire one)
 * @returns the configured template name, or `null` when nothing applies
 */
export function resolveFolderTemplate(
	noteFolder: string,
	map: Record<string, string> | undefined | null,
	excluded: string[] = [],
): string | null {
	if (!noteFolder || !map) return null;

	// Carve-out first — an excluded folder never receives a template, however the
	// map is configured. Otherwise a note created inside Templates/ would itself
	// be templated, which is circular.
	for (const ex of excluded) {
		if (ex && isAncestorOrSame(ex, noteFolder)) return null;
	}

	let best: string | null = null;
	let bestLen = -1;
	for (const [folder, template] of Object.entries(map)) {
		if (!template) continue; // an empty value means "cleared"
		if (!isAncestorOrSame(folder, noteFolder)) continue;
		const len = normalizeFolder(folder).length;
		if (len > bestLen) {
			bestLen = len; // longest matching PREFIX wins — the deepest context
			best = template;
		}
	}
	return best;
}

/** A template name → its file name inside the templates directory. */
export function templateFileName(name: string): string {
	return name.endsWith('.md') ? name : `${name}.md`;
}
