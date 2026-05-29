/**
 * MIG-065 §F.2 — shared table-model helpers for the unified Base.
 *
 * One source of truth for how a lens/base column renders, shared by BOTH
 * table surfaces so they can never drift:
 *   - the inline ` ```base view: table ` widget — `LensBlockWidget._renderTable`
 *     in `$lib/editor/livePreview.ts` (vanilla DOM inside a CM6 decoration);
 *   - the standalone `.base` full-tab view — `$lib/lens/BaseTab.svelte`.
 *
 * Per CLAUDE.md "secure the winning — one source of truth, tested once, used
 * many times": the cell-render + column-label logic lives here. Each host
 * supplies its own i18n translate fn (the CM6 widget reads imperatively via
 * `get(t)`; the Svelte component passes the reactive `$t`) — the label
 * semantics are identical either way.
 *
 * Pure module: no DOM, no Svelte, no Tauri. Safe to import anywhere.
 */
import type { DimensionValue } from './store';

/** Frontmatter columns ride in the `dimension` field as `prop.<key>` (the
 *  MIG-065 §C+§D deviation from the Architect's separate `property:` key —
 *  same outcome, zero struct churn). Mirrors `PROP_PREFIX` in `dimensions.rs`. */
const PROP_PREFIX = 'prop.';

/** A `prop.<key>` column maps to the note's frontmatter key `<key>`. */
export function isPropColumn(dim: string): boolean {
	return dim.startsWith(PROP_PREFIX);
}

/** The frontmatter key behind a `prop.<key>` column. */
export function propKey(dim: string): string {
	return dim.slice(PROP_PREFIX.length);
}

/** Registered (non-frontmatter) dimensions → i18n key + English fallback. The
 *  English literal preserves the pre-extraction behaviour for the 13 locales
 *  that don't yet carry `lensBlock.col*` (filled in MIG-065 §L); svelte-i18n
 *  returns the key string for a miss, which `columnLabel` detects and replaces. */
const REGISTERED_LABELS: Record<string, { key: string; en: string }> = {
	'note.name': { key: 'lensBlock.colName', en: 'Name' },
	'note.headline': { key: 'lensBlock.colHeadline', en: 'Summary' },
	'note.created_at': { key: 'lensBlock.colCreated', en: 'Created' },
	'note.path': { key: 'lensBlock.colPath', en: 'Path' },
};

/**
 * The data columns to render — every declared column except `note.name`, which
 * is always rendered as the first (clickable) name column by both surfaces.
 */
export function dataColumns(columns: string[]): string[] {
	return columns.filter((c) => c !== 'note.name');
}

/**
 * Header label for a column dimension. `prop.<key>` columns show the raw
 * frontmatter key (the user's own vocabulary — never translated). Registered
 * dimensions resolve via i18n, falling back to the English literal when the
 * active locale lacks the key. Unknown dimensions show the dimension name.
 *
 * @param translate a key→string lookup (`get(t)` imperatively, or reactive `$t`).
 */
export function columnLabel(dim: string, translate: (key: string) => string): string {
	if (isPropColumn(dim)) return propKey(dim);
	const reg = REGISTERED_LABELS[dim];
	if (reg) {
		const s = translate(reg.key);
		return s && s !== reg.key ? s : reg.en;
	}
	return dim;
}

/**
 * Render one dimension value to display text:
 *   - null / undefined → '' (empty cell);
 *   - `note.created_at` number → locale date;
 *   - other number → its string form;
 *   - boolean → '✓' (true) / '' (false);
 *   - string → itself.
 */
export function renderCellValue(val: DimensionValue | undefined, dim: string): string {
	if (val === null || val === undefined) return '';
	if (typeof val === 'number') {
		if (dim === 'note.created_at') {
			try {
				return new Date(val * 1000).toLocaleDateString();
			} catch {
				return String(val);
			}
		}
		return String(val);
	}
	if (typeof val === 'boolean') return val ? '✓' : '';
	return String(val);
}
