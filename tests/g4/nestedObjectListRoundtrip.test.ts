/**
 * G4 — the `nested-object-list` serializer, restored.
 *
 * **APP-KILLER, confirmed by the safety inspection 2026-07-20.** `serializeLine`
 * in `yamlDoc.ts` had no `nested-object-list` branch, so it fell through to
 * `value = prop.value` — the flat ` | `-joined SUMMARY string. Editing one row of
 * a structured `ikhtilāf` block therefore spliced the whole block-seq out of the
 * CST and wrote a single scalar in its place:
 *
 *   ikhtilāf: 'Hanafī: permissible | Mālikī: discouraged'
 *
 * On reopen the parser's nested branch requires an EMPTY value (`!value`), so it
 * never fires — `nestedObjects` comes back undefined and the widget renders ZERO
 * rows. **Every structured row is gone from the `.md` source of truth, with no
 * error and no journal anomaly.** The legacy `reconstructFrontmatter`
 * (store.ts) serialized this type correctly; the G4 swap silently dropped it.
 *
 * These tests are written to fail against the pre-fix serializer.
 */
import { describe, it, expect } from 'vitest';
import { composeFrontmatter, parseFrontmatterDoc } from '$lib/editor/yamlDoc';
import { parseFrontmatter } from '$lib/libraries/store';
import type { FrontmatterProperty } from '$lib/libraries/store';

const NOTE = [
	'---',
	'title: Zakat rulings',
	'ikhtilāf:',
	'  - school: Hanafī',
	'    position: permissible',
	'  - school: Mālikī',
	'    position: discouraged',
	'tags:',
	'  - fiqh',
	'---',
	'Body text.',
	'',
].join('\n');

function propsOf(content: string): FrontmatterProperty[] {
	return parseFrontmatter(content).properties;
}

describe('G4 — nested-object-list survives compose', () => {
	it('the fixture parses into structured rows to begin with', () => {
		const ik = propsOf(NOTE).find((p) => p.key === 'ikhtilāf');
		expect(ik, 'fixture must parse as a nested-object-list').toBeDefined();
		expect(ik!.type).toBe('nested-object-list');
		expect(ik!.nestedObjects).toHaveLength(2);
	});

	/** THE REGRESSION: edit one field, and every row must survive on disk. */
	it('editing one row does not flatten the block to a scalar', () => {
		const fm = parseFrontmatterDoc(NOTE);
		const oldProps = propsOf(NOTE);

		const newProps = oldProps.map((p) => {
			if (p.key !== 'ikhtilāf') return p;
			const rows = (p.nestedObjects ?? []).map((r) => ({ ...r }));
			rows[1] = { ...rows[1], position: 'prohibited' };
			return {
				...p,
				nestedObjects: rows,
				// PropertyEditor rebuilds the flat summary alongside the rows; it is
				// this changed `value` that makes the diff comparator treat the key as
				// edited, which is what dragged it into the scalar path.
				value: rows.map((r) => Object.values(r).join(': ')).join(' | '),
			};
		});

		const out = composeFrontmatter(fm.rawYaml, fm.hadFence, oldProps, newProps, 'Body text.\n');

		expect(out, 'the block must not collapse onto the key line').not.toMatch(/^ikhtilāf: /m);
		expect(out).toContain('- school: Hanafī');
		expect(out).toContain('- school: Mālikī');
		expect(out).toContain('position: prohibited');
		expect(out).toContain('position: permissible');
	});

	/** The real damage was only visible on REOPEN — assert the round-trip. */
	it('round-trips: the edited note re-parses into the same number of rows', () => {
		const fm = parseFrontmatterDoc(NOTE);
		const oldProps = propsOf(NOTE);
		const newProps = oldProps.map((p) => {
			if (p.key !== 'ikhtilāf') return p;
			const rows = (p.nestedObjects ?? []).map((r) => ({ ...r }));
			rows[1] = { ...rows[1], position: 'prohibited' };
			return { ...p, nestedObjects: rows, value: 'changed-summary' };
		});

		const out = composeFrontmatter(fm.rawYaml, fm.hadFence, oldProps, newProps, 'Body text.\n');
		const reparsed = propsOf(out).find((p) => p.key === 'ikhtilāf');

		expect(reparsed, 'the key must still be present after a round-trip').toBeDefined();
		expect(reparsed!.type).toBe('nested-object-list');
		expect(reparsed!.nestedObjects).toHaveLength(2);
		expect(reparsed!.nestedObjects![0].school).toBe('Hanafī');
		expect(reparsed!.nestedObjects![1].position).toBe('prohibited');
	});

	/** Untouched siblings must stay byte-perfect — the whole point of G4. */
	it('leaves the other keys and the body untouched', () => {
		const fm = parseFrontmatterDoc(NOTE);
		const oldProps = propsOf(NOTE);
		const newProps = oldProps.map((p) =>
			p.key === 'ikhtilāf'
				? { ...p, nestedObjects: [{ school: 'Shāfiʿī', position: 'obligatory' }], value: 'x' }
				: p,
		);
		const out = composeFrontmatter(fm.rawYaml, fm.hadFence, oldProps, newProps, 'Body text.\n');
		expect(out).toContain('title: Zakat rulings');
		expect(out).toMatch(/tags:\r?\n\s+- fiqh/);
		expect(out).toContain('Body text.');
	});

	/** An emptied list must not leave a dangling key with a stale scalar. */
	it('handles an emptied row set without writing a stale summary', () => {
		const fm = parseFrontmatterDoc(NOTE);
		const oldProps = propsOf(NOTE);
		const newProps = oldProps.map((p) =>
			p.key === 'ikhtilāf' ? { ...p, nestedObjects: [], value: '' } : p,
		);
		const out = composeFrontmatter(fm.rawYaml, fm.hadFence, oldProps, newProps, 'Body text.\n');
		expect(out).not.toContain('Hanafī');
		expect(out).not.toContain('Mālikī');
	});
});
