/**
 * Shared font catalogue — MIG-070 §C polish (Item A: real font choices).
 *
 * ONE source of truth for "what fonts can the user pick". Reused by the Style Setter's font
 * pickers (Interface / Note / Code + the file-tree & chrome fonts) AND the Settings → Font Sets
 * editor — so we never keep two font lists that drift (feedback_reuse_components).
 *
 * Strategy (the proven web pattern — the Local Font Access API, as web design tools use):
 *   1. A curated, cross-platform family list is the FLOOR — always present, identical on every
 *      machine, no permission prompt.
 *   2. `ensureSystemFonts()` enhances it with the user's ACTUALLY-installed fonts via the browser's
 *      `queryLocalFonts()` API, when the WebView allows it. If the API is absent or the permission
 *      is denied, the curated list stands (graceful degradation — the user still gets a real list).
 *
 * The detected/curated list is published once into the `systemFonts` store and reused everywhere
 * (one query per session, not one per component — Performance Rules 1 & 3).
 */
import { writable } from 'svelte/store';

/** Curated cross-platform family NAMES (Latin + common Arabic faces). The fallback floor + what we
 *  merge installed fonts into. Sorted for a stable, scannable dropdown. */
export const CURATED_FONTS: string[] = [
	'Amiri', 'Arial', 'Cairo', 'Calibri', 'Cambria', 'Cascadia Code', 'Comic Sans MS',
	'Consolas', 'Constantia', 'Corbel', 'Courier New', 'Dubai', 'Fira Code', 'Georgia',
	'Impact', 'Inter', 'JetBrains Mono', 'Lora', 'Lucida Console', 'Merriweather',
	'Noto Naskh Arabic', 'Noto Sans', 'Noto Sans Arabic', 'Noto Serif', 'Open Sans',
	'Palatino Linotype', 'Roboto', 'Sakkal Majalla', 'Segoe UI', 'Simplified Arabic',
	'Tahoma', 'Tajawal', 'Times New Roman', 'Traditional Arabic', 'Trebuchet MS', 'Verdana',
].sort((a, b) => a.localeCompare(b));

/** The live font-family list, published once. Starts at the curated floor; `ensureSystemFonts()`
 *  swaps in the installed-fonts superset when available. Components subscribe via `$systemFonts`. */
export const systemFonts = writable<string[]>(CURATED_FONTS);

let _loaded = false;
/** Enhance `systemFonts` with the machine's installed fonts (once per session). Safe to call from
 *  many components — only the first call queries. Falls back silently to the curated floor. */
export async function ensureSystemFonts(): Promise<void> {
	if (_loaded) return;
	_loaded = true;
	try {
		if (typeof window !== 'undefined' && 'queryLocalFonts' in window) {
			const fonts = await (window as unknown as { queryLocalFonts: () => Promise<Array<{ family: string }>> }).queryLocalFonts();
			const families = new Set<string>();
			for (const f of fonts) families.add(f.family);
			if (families.size > 0) systemFonts.set([...families].sort((a, b) => a.localeCompare(b)));
		}
	} catch { /* permission denied / API absent → keep the curated floor */ }
}

/** A family name → a safe CSS font-family value (quoted when it has spaces, so multi-word names
 *  resolve). Used by the Style Setter, which writes the value into a CSS variable. */
export function fontFamilyValue(family: string): string {
	return /[\s"]/.test(family) ? `"${family.replace(/"/g, '')}"` : family;
}
