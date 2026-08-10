/**
 * PJ-207 §15 whole-app sweep — APP-KILLER: **a property edit on a note whose YAML cannot be
 * parsed is silently discarded and reported as saved.**
 *
 * The mechanism, verified in source before this test was written:
 *
 *   1. `composeFrontmatter`'s H1 branch (`yamlDoc.ts`) returns the ORIGINAL frontmatter bytes
 *      whenever `parseDocument(rawYaml).errors.length` — correct for *preserving* the file, but it
 *      discards every pending property intent along with it.
 *   2. The model holds props projected by the LENIENT hand-rolled parser, so the edit is accepted
 *      (`setPropValue` finds the key and mutates) and the panel keeps displaying it all session.
 *   3. `FmDoc.hasErrors` is computed and consumed NOWHERE — grep-confirmed — so no banner, no
 *      console error, no read-only state ever tells the user.
 *
 * Net effect: the value is in the UI, never on disk, and only reveals itself as a silent revert
 * after a reopen or restart. The triggers are ordinary — a TAB-indented list item (routine in
 * hand-authored and Obsidian-imported notes), a duplicate key, a value starting with `@`.
 *
 * The fix is NOT to make compose write malformed YAML — that would risk corrupting the file. It is
 * to REFUSE the intent at the model, so the panel's existing PJ-187 "could not be saved" banner
 * fires and the user knows.
 */
import { describe, it, expect, beforeEach } from 'vitest';
import { openModel, getModel, setPropValue, addProp, removeProp, compose } from '$lib/editor/noteModel';

/** compose() returns a typed result; these tests only care about the composed bytes. */
function composedOf(id: string, path: string): string {
	const r = compose(id, path);
	return r.ok ? r.content : '';
}
import { parseFrontmatterDoc } from '$lib/editor/yamlDoc';

/** A tab-indented list item — yaml rejects tabs as indentation. Common in imported vaults. */
const TAB_INDENTED = ['---', 'title: Imported Note', 'tags:', '\t- history', '---', '', 'Body text.', ''].join('\n');

/** A duplicate top-level key — "Map keys must be unique". */
const DUPLICATE_KEY = ['---', 'title: One', 'title: Two', '---', '', 'Body.', ''].join('\n');

describe('PJ-207 §15 — malformed frontmatter must REFUSE property edits, not swallow them', () => {
	beforeEach(() => {
		// Each test opens its own id; no shared state to reset.
	});

	it('yaml really does reject these shapes (the premise, not an assumption)', () => {
		expect(parseFrontmatterDoc(TAB_INDENTED).hasErrors).toBe(true);
		expect(parseFrontmatterDoc(DUPLICATE_KEY).hasErrors).toBe(true);
		// And a WELL-FORMED note must not be caught by the same guard.
		const ok = ['---', 'title: Fine', 'tags:', '  - history', '---', '', 'Body.', ''].join('\n');
		expect(parseFrontmatterDoc(ok).hasErrors).toBe(false);
	});

	it('SETTING a value on an unparseable note is refused, not silently dropped', () => {
		openModel('t1', '/lib/Imported.md', TAB_INDENTED);
		const took = setPropValue('t1', 'title', 'Edited Title');
		expect(took).toBe(false); // refused → the panel's banner fires
		// And the composed content must NOT claim the edit landed.
		expect(composedOf('t1', '/lib/Imported.md')).not.toContain('Edited Title');
	});

	it('ADDING a property to an unparseable note is refused', () => {
		openModel('t2', '/lib/Imported.md', TAB_INDENTED);
		const took = addProp('t2', { key: 'status', value: 'draft', type: 'text' } as never);
		expect(took).toBe(false);
		expect(composedOf('t2', '/lib/Imported.md')).not.toContain('status: draft');
	});

	it('REMOVING a property from an unparseable note is refused', () => {
		openModel('t3', '/lib/Dup.md', DUPLICATE_KEY);
		expect(removeProp('t3', 'title')).toBe(false);
	});

	it('the note itself is never damaged — frontmatter AND body survive verbatim', () => {
		openModel('t4', '/lib/Imported.md', TAB_INDENTED);
		setPropValue('t4', 'title', 'Edited Title');
		const out = composedOf('t4', '/lib/Imported.md');
		expect(out).toContain('\t- history'); // the tab that broke the parse, untouched
		expect(out).toContain('title: Imported Note'); // original value intact
		expect(out).toContain('Body text.');
	});

	it('a WELL-FORMED note is completely unaffected — edits still land', () => {
		const good = ['---', 'title: Fine', 'tags:', '  - history', '---', '', 'Body.', ''].join('\n');
		openModel('t5', '/lib/Good.md', good);
		expect(setPropValue('t5', 'title', 'Edited')).toBe(true);
		expect(composedOf('t5', '/lib/Good.md')).toContain('title: Edited');
		expect(getModel('t5')?.props.find((p) => p.key === 'title')?.value).toBe('Edited');
	});
});
