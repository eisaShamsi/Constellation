/**
 * G4 Phase 2 (unified) — the LIVE save authority (noteModel) round-trips rich
 * frontmatter safely. Critically, these exercise the exact scenarios the
 * adversarial review flagged as APP-KILLERS: the model uses the SAME
 * `parseFrontmatter` projection the PropertyEditor uses for BOTH base and
 * current, so editing ONE field leaves every other key byte-perfect — even a
 * block scalar / nested map / escaped-quote value that parseFrontmatter itself
 * projects lossily. Consistency (not losslessness) is what makes the diff safe.
 *
 * The running-app Editor-Surface Gate (Boss test) is the Reproduce-First runtime
 * verification per CLAUDE.md; this proves the model-level contract first.
 */
import { describe, it, expect, beforeEach } from 'vitest';
import { openModel, setProps, setBody, compose, adoptDisk, getModel, clearAllModels } from '$lib/editor/noteModel';

beforeEach(() => clearAllModels());

const editTitle = (id: string, value: string) => {
	const m = getModel(id)!;
	setProps(id, m.props.map((p) => (p.key === 'title' ? { ...p, value } : p)), m.path);
};

describe('G4 Phase 2 — editing one field preserves the rest (the review app-killers)', () => {
	it('Finding 1: editing the title preserves a nested map + block scalar (no drop/corruption)', () => {
		const src = [
			'---', 'title: Old', 'source:', '  author: Ibn Khaldun', '  year: 1377',
			'description: |', '  line one', '  line two', 'cid_cn: NOTE_1', '---', 'body', '',
		].join('\n');
		openModel('t1', '/n.md', src);
		editTitle('t1', 'New');
		const r = compose('t1', '/n.md');
		expect(r.ok).toBe(true);
		if (!r.ok) return;
		expect(r.content).toContain('title: New');
		expect(r.content).toContain('author: Ibn Khaldun'); // nested map child preserved
		expect(r.content).toContain('year: 1377');
		expect(r.content).toContain('description: |\n  line one\n  line two'); // block scalar preserved
		expect(r.content).not.toContain('description: "|"'); // NOT the corruption signature
		expect(r.content).toContain('cid_cn: NOTE_1');
	});

	it('Finding 2: editing the title preserves an escaped/quoted value byte-perfect', () => {
		const src = ['---', 'title: Old', 'note: "She said \\"hi\\""', 'cid_cn: NOTE_1', '---', 'body', ''].join('\n');
		openModel('t1', '/n.md', src);
		editTitle('t1', 'New');
		const r = compose('t1', '/n.md');
		expect(r.ok).toBe(true);
		if (r.ok) expect(r.content).toContain('note: "She said \\"hi\\""'); // exact original bytes
	});

	it('Finding 2 (compounding): repeated open/edit/save cycles never grow backslashes', () => {
		let content = ['---', 'title: T', 'note: "a \\"b\\" c"', 'cid_cn: NOTE_1', '---', 'body', ''].join('\n');
		for (let i = 0; i < 5; i++) {
			clearAllModels();
			openModel('t1', '/n.md', content);
			editTitle('t1', `T${i}`); // edit a DIFFERENT field each cycle
			const r = compose('t1', '/n.md');
			content = r.ok ? r.content : content;
		}
		expect(content).not.toMatch(/\\\\/); // no doubled backslash ever
		expect(content).toContain('note: "a \\"b\\" c"'); // value byte-perfect throughout
	});

	it('Finding 3: editing the title leaves an unedited tags list in place (no reorder churn)', () => {
		const src = ['---', 'title: Old', 'tags: [alpha, beta]', 'cid_cn: NOTE_1', '---', 'body', ''].join('\n');
		openModel('t1', '/n.md', src);
		editTitle('t1', 'New');
		const r = compose('t1', '/n.md');
		expect(r.ok).toBe(true);
		if (r.ok) {
			expect(r.content).toContain('tags: [alpha, beta]'); // untouched, in place
			expect(r.content.indexOf('tags:')).toBeLessThan(r.content.indexOf('cid_cn:')); // order kept
		}
	});
});

describe('G4 Phase 2 — round-trip + freshness invariants intact', () => {
	it('a no-edit open→compose is byte-perfect', () => {
		const src = ['---', 'title: T', 'cid_cn: NOTE_1', 'kind: note', '---', 'body text', ''].join('\n');
		openModel('t1', '/n.md', src);
		const r = compose('t1', '/n.md');
		expect(r.ok && r.content === src).toBe(true);
	});

	it('a body edit persists while frontmatter stays byte-perfect', () => {
		const src = ['---', 'title: T', 'source:', '  a: 1', 'cid_cn: NOTE_1', '---', 'old body', ''].join('\n');
		openModel('t1', '/n.md', src);
		setBody('t1', 'new body', '/n.md');
		const r = compose('t1', '/n.md');
		expect(r.ok).toBe(true);
		if (r.ok) {
			expect(r.content).toContain('new body');
			expect(r.content).toContain('source:\n  a: 1'); // nested map untouched by a body edit
		}
	});

	it('review #1: a `...`-terminated note is NOT duplicated on save', () => {
		const src = ['---', 'title: Foo', '...', 'body line 1', 'body line 2', ''].join('\n');
		openModel('t1', '/n.md', src);
		setBody('t1', 'body line 1\nbody line 2 edited\n', '/n.md');
		const r = compose('t1', '/n.md');
		expect(r.ok).toBe(true);
		if (!r.ok) return;
		// no frontmatter duplication — the `...` note is treated as fence-less by BOTH
		// parsers (parity), so compose returns the body it was given, unduplicated.
		expect((r.content.match(/^---$/gm) || []).length).toBeLessThanOrEqual(1);
		expect(r.content).not.toContain('---\n---'); // the duplication signature
	});

	it('review #2: a CRLF note is byte-perfect on a no-edit save (fences match EOL)', () => {
		const src = '---\r\ntitle: Foo\r\ntags:\r\n  - a\r\n  - b\r\n---\r\nbody line\r\n';
		openModel('t1', '/n.md', src);
		const r = compose('t1', '/n.md');
		expect(r.ok).toBe(true);
		if (r.ok) expect(r.content).toBe(src); // no LF/CRLF churn
	});

	it('review #2: editing the title on a CRLF note keeps CRLF fences + untouched keys', () => {
		const src = '---\r\ntitle: Old\r\ncid_cn: NOTE_1\r\n---\r\nbody\r\n';
		openModel('t1', '/n.md', src);
		editTitle('t1', 'New');
		const r = compose('t1', '/n.md');
		expect(r.ok).toBe(true);
		if (r.ok) {
			expect(r.content).toContain('title: New');
			expect(r.content).toContain('---\r\n'); // CRLF fence preserved
			expect(r.content).not.toContain('---\ntitle'); // no LF fence
		}
	});

	it('adoptDisk of a rich external edit round-trips; a clean model ignores its own echo', () => {
		openModel('t1', '/n.md', '---\ntitle: T\ncid_cn: NOTE_1\n---\nold\n');
		const rich = ['---', 'title: T', 'desc: |', '  m1', '  m2', 'cid_cn: NOTE_1', '---', 'new', ''].join('\n');
		expect(adoptDisk('t1', rich)).toBe(true);
		const r = compose('t1', '/n.md');
		expect(r.ok && r.content === rich).toBe(true); // adopted disk composes back byte-perfect
		if (r.ok) expect(adoptDisk('t1', r.content)).toBe(false); // its own echo is not re-adopted
	});
});
