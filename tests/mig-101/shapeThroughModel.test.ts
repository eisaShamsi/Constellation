/**
 * MIG-101 §A-fix — the regression test for the silent shape loss.
 *
 * Boss-reported 2026-07-20 as "Step 2: nothing changed". The write had in fact
 * succeeded on disk; what had failed was that the OPEN note's model knew nothing
 * about it. An open note composes its frontmatter from the byte-base captured
 * when the note was opened, so a `shape:` written straight to disk behind the
 * model was silently dropped by the next save.
 *
 * These tests pin the corrected contract: **shape reaches disk through the
 * model.** `setProps` → `compose` must emit the key, and must keep emitting it
 * across further edits, because compose is what every save writes.
 *
 * The first test is the direct reproduction: it fails against the original
 * design (disk write behind the model) and passes against the fix.
 */
import { describe, it, expect, beforeEach } from 'vitest';
import { openModel, setProps, setBody, compose, clearAllModels, getModel } from '$lib/editor/noteModel';

const PATH = '/lib/Z3fran.md';
const ID = 'tab-1';

/** A note exactly like the Boss's probe: real frontmatter, no `shape`. */
const ON_DISK = [
	'---',
	'title: "Z3fran"',
	'cid_cn: 20260707T190416Z_NOTE_943A',
	'kind: note',
	'created: 2026-07-07T19:04:16.910733+00:00',
	'---',
	'Can I write?',
	'Is it SAFE?',
	'',
].join('\n');

/** compose() returns a discriminated union; narrow it once, here, so the tests
 *  read as assertions about CONTENT rather than about the result wrapper. */
function composed(id: string, path: string): string {
	const r = compose(id, path);
	if (!r.ok) throw new Error(`compose refused: ${r.reason}`);
	return r.content;
}

function shapeOf(content: string): string | null {
	const m = /^shape:[ \t]*(.*)$/m.exec(content);
	return m ? m[1].trim() : null;
}

function withShape(id: string, value: string | null) {
	const props = getModel(id)!.props;
	const next = value
		? [...props.filter((p) => p.key.toLowerCase() !== 'shape'), { key: 'shape', value, type: 'text' as any }]
		: props.filter((p) => p.key.toLowerCase() !== 'shape');
	setProps(id, next, PATH);
}

describe('MIG-101 — shape survives the save path', () => {
	beforeEach(() => {
		clearAllModels();
		openModel(ID, PATH, ON_DISK);
	});

	it('compose emits a shape set through the model', () => {
		withShape(ID, 'scrap');
		expect(shapeOf(composed(ID, PATH))).toBe('scrap');
	});

	/**
	 * THE REGRESSION. The original design wrote `shape:` to disk while the model
	 * held the pre-write bytes; the next body edit then composed from that stale
	 * base and dropped the key with no error. Here the shape is set through the
	 * model first, so a subsequent edit must NOT lose it.
	 */
	it('a later body edit does not silently drop the shape', () => {
		withShape(ID, 'scrap');
		setBody(ID, 'Can I write?\nIs it SAFE?\nAnd now I typed more.\n', PATH);
		const content = composed(ID, PATH);
		expect(shapeOf(content)).toBe('scrap');
		expect(content).toContain('And now I typed more.');
	});

	it('changing the shape replaces the value rather than duplicating the key', () => {
		withShape(ID, 'scrap');
		withShape(ID, 'page');
		const content = composed(ID, PATH);
		expect(shapeOf(content)).toBe('page');
		expect(content.match(/^shape:/gm)?.length).toBe(1);
	});

	it('clearing the shape removes the key entirely', () => {
		withShape(ID, 'scrap');
		withShape(ID, null);
		const content = composed(ID, PATH);
		expect(shapeOf(content)).toBeNull();
		expect(content).not.toContain('shape:');
	});

	/** Shape governs presentation, never content — so the body and the other
	 *  frontmatter keys must come through a full round-trip untouched. */
	it('setting and clearing a shape leaves the rest of the note identical', () => {
		const before = composed(ID, PATH);
		withShape(ID, 'scrap');
		withShape(ID, null);
		expect(composed(ID, PATH)).toBe(before);
	});

	it('preserves the sibling frontmatter keys while shaped', () => {
		withShape(ID, 'page');
		const c = composed(ID, PATH);
		expect(c).toContain('title: "Z3fran"');
		expect(c).toContain('cid_cn: 20260707T190416Z_NOTE_943A');
		expect(c).toContain('kind: note');
		expect(c).toContain('created: 2026-07-07T19:04:16.910733+00:00');
		expect(c).toContain('Can I write?');
	});

	/**
	 * DOCUMENTS THE ORIGINAL FAULT, so the reason for the fix cannot be lost.
	 *
	 * This is what the first implementation did: Rust wrote `shape:` straight to
	 * disk while the note was open. The model was never told, so it kept composing
	 * from its open-time byte-base — and the very next save wrote that base back
	 * over the disk copy, erasing the key with no error and no warning.
	 *
	 * The assertion below is the loss itself: disk has the shape, compose does not.
	 * Nothing in the product may depend on this behaviour; it exists only to keep
	 * the mechanism legible to whoever reads this test next.
	 */
	it('documents the original fault: a disk write behind the model is invisible to compose', () => {
		const diskAfterOutOfBandWrite = ON_DISK.replace('---\nCan I write?', 'shape: scrap\n---\nCan I write?');
		expect(shapeOf(diskAfterOutOfBandWrite)).toBe('scrap'); // the value really is on disk…

		// …but the open model never heard about it, so what the next save composes
		// — and therefore what lands on disk — has no shape at all.
		expect(shapeOf(composed(ID, PATH))).toBeNull();
	});

	/** The identity guard must refuse a write aimed at a different path — a
	 *  shape action arriving from an editor whose tab was repurposed. */
	it('refuses a shape write aimed at the wrong path', () => {
		const before = composed(ID, PATH);
		const props = getModel(ID)!.props;
		setProps(ID, [...props, { key: 'shape', value: 'scrap', type: 'text' as any }], '/lib/OTHER.md');
		expect(composed(ID, PATH)).toBe(before);
	});
});
