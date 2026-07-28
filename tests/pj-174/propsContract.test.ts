/**
 * MIG-107 Slice 0 — THE SUBSTRATE CONTRACT the props fix must satisfy.
 *
 * Slice 0's job was to answer the one design question flagged `[UNVERIFIED]` in the
 * Architect+Plan §5.4 — *do intents key on the property KEY, or on a stable row id?* — by
 * measuring the code rather than reasoning about it. These tests are that measurement, kept
 * permanently so the answer cannot be re-derived wrongly later.
 *
 * ── THE ANSWER: KEY, not row id ──────────────────────────────────────────────────────────────
 *
 * `composeFrontmatter` is entirely key-addressed: it builds `oldByKey` / `newByKey` as `Map`s
 * keyed by `p.key` (yamlDoc.ts:338-343). A `Map` cannot hold two entries for one key, so **the
 * persisted representation structurally cannot carry duplicate keys** — a second row for the same
 * key is silently dropped at save time, last-one-wins.
 *
 * So the shared authority (the model) should hold exactly what can be persisted: well-formed,
 * key-unique properties. Intents key on the property key, which is the only identity the file
 * format actually has.
 *
 * ── WHAT THAT MEANS FOR THE PANEL ────────────────────────────────────────────────────────────
 *
 * `addProperty()` (PropertyEditor.svelte:605) appends `{ key: '', value: '', type: 'text' }`, and
 * nothing filters empty keys before compose. Two blank rows can therefore coexist, which is what
 * made key-addressing look impossible. The resolution is that a half-typed row is an **editing
 * state of one panel**, not a state of the shared authority: it stays local to the panel until it
 * has a non-empty key, and only then becomes an `addProp` intent.
 *
 * That also removes a real defect this slice uncovered — see `an_empty_key_row_is_written_to_the_file`
 * below. It is filed as PJ-178 because it is reachable TODAY, independently of this migration.
 */
import { describe, it, expect } from 'vitest';
import { composeFrontmatter } from '$lib/editor/yamlDoc';
import type { FrontmatterProperty, PropertyType } from '$lib/libraries/store';

const RAW = `title: N\ncid_cn: C1\nstage: seed\n`;
const P = (key: string, value: string, type: PropertyType = 'text'): FrontmatterProperty => ({ key, value, type });
const OLD = [P('title', 'N'), P('cid_cn', 'C1'), P('stage', 'seed')];
const compose = (next: FrontmatterProperty[]) => composeFrontmatter(RAW, true, OLD, next, 'body');

describe('MIG-107 Slice 0 — the persisted representation is KEY-ADDRESSED', () => {
	/** The measurement that settles §5.4. */
	it('cannot hold duplicate keys — the second row silently wins and the first is discarded', () => {
		const out = compose([P('title', 'N'), P('cid_cn', 'C1'), P('stage', 'sapling'), P('stage', 'wilting')]);
		expect(out).toContain('stage: wilting');
		expect(out).not.toContain('stage: sapling');
		// One key, one line — there is no arrangement of the array that yields two `stage:` lines.
		expect(out.match(/^stage:/gm)?.length).toBe(1);
	});

	it('addresses by key, not by position — reordering the array alone changes nothing on disk', () => {
		const inOrder = compose([P('title', 'N'), P('cid_cn', 'C1'), P('stage', 'sapling')]);
		const shuffled = compose([P('stage', 'sapling'), P('cid_cn', 'C1'), P('title', 'N')]);
		expect(shuffled).toBe(inOrder);
	});

	it('never rewrites a key the user did not edit (the byte-perfect in-place contract)', () => {
		const out = compose([P('title', 'N'), P('cid_cn', 'C1'), P('stage', 'sapling')]);
		expect(out).toContain('title: N');
		expect(out).toContain('cid_cn: C1');
		expect(out).toContain('stage: sapling');
	});

	it('a REMOVED key is spliced out, and only that key', () => {
		const out = compose([P('title', 'N'), P('cid_cn', 'C1')]);
		expect(out).not.toContain('stage:');
		expect(out).toContain('title: N');
		expect(out).toContain('cid_cn: C1');
	});

	it('a RENAMED key is a remove + an add, which is why rename needs its own intent', () => {
		const out = compose([P('title', 'N'), P('cid_cn', 'C1'), P('phase', 'seed')]);
		expect(out).not.toContain('stage:');
		expect(out).toContain('phase: seed');
	});
});

describe('MIG-107 Slice 0 — PJ-178: a blank property row is written into the user\'s file', () => {
	/**
	 * Reachable TODAY, with no migration involved: click "+" in the Properties panel, then edit any
	 * other property. `addProperty` appends a `key: ''` row, `debouncedSave` flushes the whole array,
	 * and nothing filters empty keys before compose — so the note's frontmatter gains a `"": ""`
	 * line. Filed as PJ-178; the single-ownership design fixes it structurally by keeping a
	 * half-typed row local to the panel until it has a key.
	 */
	it('an_empty_key_row_is_written_to_the_file', () => {
		const out = compose([P('title', 'N'), P('cid_cn', 'C1'), P('stage', 'sapling'), P('', '')]);
		// Documents CURRENT behaviour. When the fix lands this becomes `not.toContain`.
		expect(out).toContain('"": ""');
	});
});
