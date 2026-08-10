/**
 * ONE definition of "are these two property rows the same?".
 *
 * ## Why this module exists (PJ-182)
 *
 * The concept had **five** spellings: `JSON.stringify(listItems)` in `yamlDoc`,
 * `noteModel`, and `propsCommit` (which is what `sameList` was created to collapse), plus
 * `sameList` itself, plus the composer's own hand-written clause. Collapsing three of them
 * was not enough, and PJ-182 showed exactly why:
 *
 * `nestedObjects` — the ROWS of a `nested-object-list`, i.e. the property's actual content
 * — was threaded through `touchedSince`, `plan` and `setPropValue`, and stopped at the
 * fourth layer, `composeFrontmatter`, whose unchanged-check still decided from `value`
 * alone. `value` is only a ` | `-joined display summary. Proven by running: delete a row
 * from an ikhtilāf block without touching the summary, and the write is dropped — the
 * deleted row is still in the `.md`. **A chain is exactly as strong as its last link, and
 * the last link had not been moved.**
 *
 * So the predicate lives in ONE leaf module that every layer imports. Adding a sixth field
 * to `FrontmatterProperty` now means editing one function, not remembering four.
 *
 * It is a leaf on purpose: the only import is a TYPE (erased at build time), so `yamlDoc`
 * can use it without creating a runtime cycle with `store` or `noteModel`.
 */
import type { FrontmatterProperty } from '$lib/libraries/store';

/**
 * What a SET intent may carry beyond the value itself.
 *
 * ONE exported shape, because this bag is declared by `noteModel.setPropValue`,
 * `noteSession.editPropValue`, `propsCommit.IntentSink.setValue` and `propsCommit.PropOp`
 * — and PJ-182 WAS a field that had to be added to all four and was added to none. The
 * next field lands in one place.
 */
export type SetPropOpts = {
	listItems?: string[];
	type?: FrontmatterProperty['type'];
	/** The ROWS of a `nested-object-list` — its actual content, not its display summary. */
	nestedObjects?: Array<Record<string, string>>;
};

/**
 * Element-wise list equality. Absent and empty are the same thing — a property with no
 * items and a property with `[]` are indistinguishable on disk.
 */
export const sameList = (a?: string[], b?: string[]): boolean =>
	a === b || (!a?.length && !b?.length) || (!!a && !!b && a.length === b.length && a.every((v, i) => v === b[i]));

/**
 * Row-wise equality for a `nested-object-list`'s rows (MIG-022 §A.1 — ikhtilāf).
 *
 * Compared row by row and key by key rather than by `JSON.stringify`, so key ORDER within a
 * row does not create a phantom difference — and so this cannot quietly become a sixth
 * spelling of the same idea.
 */
export const sameNested = (
	a?: Array<Record<string, string>>,
	b?: Array<Record<string, string>>,
): boolean => {
	if (a === b) return true;
	if (!a?.length && !b?.length) return true;
	if (!a || !b || a.length !== b.length) return false;
	return a.every((row, i) => {
		const other = b[i];
		const ka = Object.keys(row);
		return ka.length === Object.keys(other).length && ka.every((k) => row[k] === other[k]);
	});
};

/**
 * The whole row: value, type, list items, and nested rows.
 *
 * Every "has the user actually changed this key?" decision goes through here —
 * `propsCommit.touchedSince`, `propsCommit.plan`, and `yamlDoc.composeFrontmatter`'s
 * never-rewrite-an-untouched-key check.
 */
export const samePropRow = (a: FrontmatterProperty, b: FrontmatterProperty): boolean =>
	a.value === b.value &&
	a.type === b.type &&
	sameList(a.listItems, b.listItems) &&
	sameNested(a.nestedObjects, b.nestedObjects);

/**
 * The items of a row that is SHOWN as a list, when they may still live only in the scalar `value`.
 *
 * PJ-207 §15 — this concept had the same fate `samePropRow` had: several spellings, and the ones
 * that mattered most were the ones that did not have it. A row can be displayed as chips while its
 * items are a comma scalar on disk — a type registered library-wide as `list` (the type dropdown
 * persists the choice for every note in the library, so it arrives on notes that were never opened
 * when it was made), or a `sources:`/`content_type:` value written on one line by the Bases cell
 * editor or typed by hand. The RENDERERS fell back to splitting `value`; the MUTATORS did not —
 * they read `listItems ?? []`, got an empty array, and rebuilt the property from nothing. Adding
 * one tag deleted the author the note already had; removing one taxonomy pill deleted the pills
 * next to it. No error, no refusal, nothing on screen to notice.
 *
 * Always returns a fresh array, so a caller can never alias the model's own.
 */
export const listItemsOf = (p: FrontmatterProperty): string[] =>
	p.listItems
		? [...p.listItems]
		: (p.value ? String(p.value).split(',').map((s) => s.trim()).filter(Boolean) : []);

/**
 * Copy a `nested-object-list`'s rows so a caller cannot alias the model's.
 *
 * One spelling, because `cloneProps` exists for exactly this reason: *"PropertyEditor was
 * hand-rolling this and its copy missed `nestedObjects`."* Threading the rows through the
 * intents added two more hand-spellings of the same map-of-spread; this is them.
 */
export const cloneRows = (
	rows?: Array<Record<string, string>>,
): Array<Record<string, string>> | undefined => rows?.map((r) => ({ ...r }));
