/**
 * PJ-182 (found in-pass, WA#6) — an ikhtilāf edit was a SILENT NO-OP, at ANY indentation.
 *
 * `nestedObjects` is where a `nested-object-list` property's content actually lives;
 * `value` is only a ` | `-joined SUMMARY of it, kept for legacy display and search. Nothing
 * in the props contract compared or carried `nestedObjects`:
 *
 *   - `touchedSince` compared value / type / listItems  → a row change was only noticed
 *     because the summary happens to change with it;
 *   - `PropOp.set` carried value / type / listItems     → the rows never left the panel;
 *   - `setPropValue` did `{ ...cur, value, type }`      → it spread the STALE rows through.
 *
 * Net, observed by running the panel's own mutation (`addNestedRow`) through
 * `touchedSince → plan → apply → composeFrontmatter`: the user adds a third school, the
 * panel shows three, the summary says three — and the model and the `.md` still hold two.
 * The save reports success. The row is simply gone on reopen.
 *
 * This is the same failure shape as LL-038 rule 6: correctness depended on a field being
 * carried through four layers, and it was carried through none of them. The fix gives the
 * concept one definition (`sameNested`, beside `sameList`) and threads the rows end to end.
 */
import { describe, it, expect } from 'vitest';
import { parseFrontmatter, type FrontmatterProperty } from '$lib/libraries/store';
import { splitFrontmatter, composeFrontmatter } from '$lib/editor/yamlDoc';
import { plan, apply, touchedSince, seededKeysOf } from '$lib/editor/propsCommit';
import { sameNested } from '$lib/editor/noteModel';

const rebuildSummary = (rows: Array<Record<string, string>>) =>
	rows.map((o) => Object.entries(o).map(([k, v]) => `${k}: ${v}`).join(' / ')).join(' | ');

/** Verbatim `PropertyEditor.addNestedRow` — it updates the rows AND the summary. */
function addNestedRow(props: FrontmatterProperty[], key: string, row: Record<string, string>) {
	return props.map((p) => {
		if (p.key !== key) return p;
		const nested = [...(p.nestedObjects ?? []), row];
		return { ...p, nestedObjects: nested, value: rebuildSummary(nested) };
	});
}

/** The model, driven exactly as `noteModel.setPropValue` drives it. */
function makeModel(seed: FrontmatterProperty[]) {
	const props: FrontmatterProperty[] = seed.map((p) => ({ ...p }));
	return {
		props,
		sink: {
			setValue: (key: string, value: string, o?: { listItems?: string[]; type?: FrontmatterProperty['type']; nestedObjects?: Array<Record<string, string>> }) => {
				const i = props.findIndex((p) => p.key === key);
				if (i === -1) return false;
				props[i] = {
					...props[i],
					value,
					...(o?.type ? { type: o.type } : {}),
					...(o?.listItems ? { listItems: [...o.listItems] } : {}),
					...(o?.nestedObjects ? { nestedObjects: o.nestedObjects.map((r) => ({ ...r })) } : {}),
				};
				return true;
			},
			add: (p: FrontmatterProperty) => { props.push({ ...p }); return true; },
			remove: (k: string) => { const i = props.findIndex((p) => p.key === k); if (i === -1) return false; props.splice(i, 1); return true; },
			order: () => false,
		},
	};
}

const IKHTILAF = (indent: string) =>
	[
		'---',
		'title: Zakat on Crypto',
		'ikhtilāf:',
		`${indent}- school: Hanafī`,
		`${indent}  position: permissible`,
		`${indent}- school: Mālikī`,
		`${indent}  position: discouraged`,
		'stage: spark-seed',
		'---',
		'',
		'body',
		'',
	].join('\n');

describe.each([
	['indented', '  '],
	['zero-indent', ''],
])('PJ-182 — an ikhtilāf row edit reaches the file (%s)', (_label, indent) => {
	const NOTE = IKHTILAF(indent);

	it('projects both authored rows', () => {
		const k = parseFrontmatter(NOTE).properties.find((p) => p.key === 'ikhtilāf')!;
		expect(k.type).toBe('nested-object-list');
		expect(k.nestedObjects).toEqual([
			{ school: 'Hanafī', position: 'permissible' },
			{ school: 'Mālikī', position: 'discouraged' },
		]);
	});

	it('adding a school lands in the model AND in the composed file', () => {
		const { properties, body } = parseFrontmatter(NOTE);
		const { yaml, hadFence } = splitFrontmatter(NOTE);

		const seeded = properties.map((p) => ({ ...p }));
		const local = addNestedRow(properties, 'ikhtilāf', { school: 'Shāfiʿī', position: 'obligatory' });

		const touched = touchedSince(seeded, local);
		expect([...touched]).toEqual(['ikhtilāf']);

		const ops = plan(local, properties, seededKeysOf(seeded), touched);
		const setOp = ops.find((o) => o.op === 'set' && o.key === 'ikhtilāf');
		expect(setOp).toBeDefined();
		// The rows must ride along — carrying only the summary is what lost the edit.
		expect((setOp as { nestedObjects?: unknown[] }).nestedObjects).toHaveLength(3);

		const m = makeModel(properties);
		apply(ops, m.sink);
		expect(m.props.find((p) => p.key === 'ikhtilāf')!.nestedObjects).toHaveLength(3);

		const out = composeFrontmatter(yaml, hadFence, properties, m.props, body);
		expect(out).toContain('school: Shāfiʿī');
		expect(out).toContain('position: obligatory');
		// The two authored rows are still there.
		expect(out).toContain('school: Hanafī');
		expect(out).toContain('school: Mālikī');
		// And it survives a re-read.
		expect(parseFrontmatter(out).properties.find((p) => p.key === 'ikhtilāf')!.nestedObjects).toEqual([
			{ school: 'Hanafī', position: 'permissible' },
			{ school: 'Mālikī', position: 'discouraged' },
			{ school: 'Shāfiʿī', position: 'obligatory' },
		]);
	});

	/**
	 * The summary is derived, so two different row-sets CAN share one summary. The contract
	 * must not depend on the summary noticing — at ANY layer.
	 *
	 * The second half of this test was added by the `/simplify` altitude pass, which found
	 * that `touchedSince`, `plan` and `setPropValue` had all been taught to carry the rows
	 * and `composeFrontmatter` — the layer that actually writes the file — still decided
	 * from `value`. Proven before the fix: the deleted row was still in the output. A chain
	 * is exactly as strong as its last link.
	 */
	it('an edit that changes ONLY the rows is detected AND reaches the file', () => {
		const { properties, body } = parseFrontmatter(NOTE);
		const { yaml, hadFence } = splitFrontmatter(NOTE);
		const seeded = properties.map((p) => ({ ...p }));
		const local = properties.map((p) =>
			p.key === 'ikhtilāf'
				? { ...p, nestedObjects: [{ school: 'Hanafī', position: 'permissible' }] } // value left stale on purpose
				: p,
		);
		expect([...touchedSince(seeded, local)]).toEqual(['ikhtilāf']);

		const out = composeFrontmatter(yaml, hadFence, properties, local, body);
		expect(out).toContain('school: Hanafī');
		expect(out).not.toContain('school: Mālikī'); // the deleted row actually left the file
	});
});

describe('sameNested', () => {
	it('treats absent and empty as equal, and compares row-wise', () => {
		expect(sameNested(undefined, undefined)).toBe(true);
		expect(sameNested(undefined, [])).toBe(true);
		expect(sameNested([{ a: '1' }], [{ a: '1' }])).toBe(true);
		expect(sameNested([{ a: '1' }], [{ a: '2' }])).toBe(false);
		expect(sameNested([{ a: '1' }], [{ a: '1', b: '2' }])).toBe(false);
		expect(sameNested([{ a: '1' }], [{ a: '1' }, { a: '1' }])).toBe(false);
	});
});
