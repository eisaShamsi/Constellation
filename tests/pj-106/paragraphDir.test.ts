/**
 * PJ-106 §B4 — the per-paragraph direction override: the OFFSET-PURE recipes.
 *
 * `computeParagraphDirChanges` is deterministic string/offset logic, so these tests ARE the
 * behavior of WHAT gets written where; only the Right/Left-Ctrl gesture and the visual flip
 * are live Boss tests. The recipes lock the three §B4 contracts:
 *   1. the mark lands at the CONTENT start (markdown structure never breaks),
 *   2. structural lines (blank/fence/table/rule/fence-content) are never touched,
 *   3. the operation is idempotent and switch-safe (never two marks, never a dirty no-op).
 * Plus: bidiPlugin's detectLineDir must treat the marks as first-strong (precedence), or
 * the plugin would fight the override it is supposed to render.
 */
import { describe, it, expect } from 'vitest';
import { EditorState, EditorSelection } from '@codemirror/state';
import { computeParagraphDirChanges, contentStartOffset, RLM, LRM } from '$lib/editor/paragraphDir';
import { detectLineDir } from '$lib/editor/bidiPlugin';

/** State with the caret at `pos` (or a range), then apply the computed changes → new doc. */
function apply(doc: string, pos: number | [number, number], dir: 'rtl' | 'ltr'): string {
	const sel = Array.isArray(pos) ? EditorSelection.single(pos[0], pos[1]) : EditorSelection.single(pos);
	const state = EditorState.create({ doc, selection: sel });
	const changes = computeParagraphDirChanges(state, dir);
	return state.update({ changes }).state.doc.toString();
}

describe('PJ-106 §B4 — mark placement (markdown never breaks)', () => {
	it('plain Latin paragraph → RTL: the RLM lands at the line start', () => {
		expect(apply('hello world', 3, 'rtl')).toBe(`${RLM}hello world`);
	});

	it('plain Arabic paragraph → LTR: the LRM lands at the line start', () => {
		expect(apply('مرحبا بالعالم', 3, 'ltr')).toBe(`${LRM}مرحبا بالعالم`);
	});

	it('list item: the mark goes AFTER the bullet — the list stays a list', () => {
		expect(apply('- item one', 4, 'rtl')).toBe(`- ${RLM}item one`);
	});

	it('ordered list and task checkbox keep their markers', () => {
		expect(apply('1. first', 4, 'rtl')).toBe(`1. ${RLM}first`);
		expect(apply('- [ ] task text', 8, 'rtl')).toBe(`- [ ] ${RLM}task text`);
	});

	it('heading: the mark goes after the # marker', () => {
		expect(apply('# عنوان', 4, 'ltr')).toBe(`# ${LRM}عنوان`);
	});

	it('blockquote (nested): the mark goes after the > markers', () => {
		expect(apply('> quoted text', 4, 'rtl')).toBe(`> ${RLM}quoted text`);
		expect(apply('> > deep', 5, 'rtl')).toBe(`> > ${RLM}deep`);
	});

	it('a wikilink at line start survives intact (mark before [[, never inside)', () => {
		expect(apply('[[صفحة مهمة]] text', 5, 'rtl')).toBe(`${RLM}[[صفحة مهمة]] text`);
	});
});

describe('PJ-106 §B4 — paragraph scope', () => {
	it('every content line of the multi-line block is marked', () => {
		const doc = 'line one\nline two\n\nother para';
		expect(apply(doc, 3, 'rtl')).toBe(`${RLM}line one\n${RLM}line two\n\nother para`);
	});

	it('the blank boundary line is never marked and the neighbour block is untouched', () => {
		const doc = 'para A\n\npara B';
		const out = apply(doc, 1, 'rtl');
		expect(out).toBe(`${RLM}para A\n\npara B`);
	});

	it('a selection spanning two blocks marks both (blank line still skipped)', () => {
		const doc = 'para A\n\npara B';
		const out = apply(doc, [0, doc.length], 'rtl');
		expect(out).toBe(`${RLM}para A\n\n${RLM}para B`);
	});
});

describe('PJ-106 §B4 — structural lines are never touched', () => {
	it('fence lines and fence CONTENT are skipped (no invisible chars into code)', () => {
		const doc = '```\nconst x = 1;\n```';
		expect(apply(doc, 8, 'rtl')).toBe(doc); // caret on the code line — nothing changes
	});

	it('table rows are skipped', () => {
		const doc = '| a | b |\n| --- | --- |\n| c | d |';
		expect(apply(doc, 2, 'rtl')).toBe(doc);
	});

	it('horizontal rules / setext underlines are skipped', () => {
		expect(apply('---', 1, 'rtl')).toBe('---');
		expect(apply('===', 1, 'rtl')).toBe('===');
	});

	it('code AFTER a closed fence is markable again (parity tracking)', () => {
		const doc = '```\ncode\n```\nprose after';
		const out = apply(doc, doc.length - 2, 'rtl');
		expect(out).toBe(`\`\`\`\ncode\n\`\`\`\n${RLM}prose after`);
	});
});

describe('PJ-106 §B4 — idempotence and switching', () => {
	it('setting the same direction twice is a no-op (zero changes — never dirties)', () => {
		const once = apply('hello', 2, 'rtl');
		const state = EditorState.create({ doc: once, selection: EditorSelection.single(2) });
		expect(computeParagraphDirChanges(state, 'rtl')).toEqual([]);
	});

	it('switching direction REPLACES the mark — never two marks', () => {
		const rtl = apply('hello', 2, 'rtl');
		const state = EditorState.create({ doc: rtl, selection: EditorSelection.single(2) });
		const back = state.update({ changes: computeParagraphDirChanges(state, 'ltr') }).state.doc.toString();
		expect(back).toBe(`${LRM}hello`);
		expect(back.includes(RLM)).toBe(false);
	});

	it('an imported mark-run (even a weird double) is normalized to ONE desired mark', () => {
		const doc = `${LRM}${RLM}text`;
		expect(apply(doc, 3, 'rtl')).toBe(`${RLM}text`);
	});
});

describe('PJ-106 §B4 — contentStartOffset (the placement rule itself)', () => {
	it('computes the content start across the block-marker shapes', () => {
		expect(contentStartOffset('plain')).toBe(0);
		expect(contentStartOffset('  indented')).toBe(2);
		expect(contentStartOffset('- item')).toBe(2);
		expect(contentStartOffset('1. item')).toBe(3);
		expect(contentStartOffset('- [ ] task')).toBe(6);
		expect(contentStartOffset('# head')).toBe(2);
		expect(contentStartOffset('> q')).toBe(2);
	});
});

describe('PJ-106 §B4 — the review-earned guards (adversarial findings, all fixed)', () => {
	it('[HIGH] a document-leading YAML frontmatter block is never marked (the merge-view full-file case)', () => {
		const doc = '---\ntitle: X\ntags: [مهم]\n---\nمرحبا بالعالم';
		// Caret in the body paragraph — contiguous with the closing fence, so the naive block
		// would climb into the YAML. Only the body line may gain a mark:
		const out = apply(doc, doc.length - 2, 'rtl');
		expect(out).toBe(`---\ntitle: X\ntags: [مهم]\n---\n${RLM}مرحبا بالعالم`);
		// Caret directly ON a YAML line — nothing changes at all:
		expect(apply(doc, 6, 'rtl')).toBe(`---\ntitle: X\ntags: [مهم]\n---\n${RLM}مرحبا بالعالم`.replace(RLM, ''));
	});

	it('[HIGH] a content-leading #tag line is skipped (a mark before # kills the tag everywhere)', () => {
		expect(apply('#todo buy milk', 3, 'rtl')).toBe('#todo buy milk');
		expect(apply('- [ ] #مهم مهمة', 9, 'rtl')).toBe('- [ ] #مهم مهمة');
		// …but a REAL heading (hash + space) still gets its mark after the marker:
		expect(apply('# عنوان', 3, 'ltr')).toBe(`# ${LRM}عنوان`);
	});

	it('[MED] a literal ~~~ inside a ``` fence does NOT close it (opener-matched parity)', () => {
		const doc = '```\ncode line\n~~~\nmore code\n```\noutro';
		const out = apply(doc, doc.length - 2, 'rtl');
		// 'more code' stays untouched (still inside the ``` fence); only 'outro' is marked.
		expect(out).toBe(`\`\`\`\ncode line\n~~~\nmore code\n\`\`\`\n${RLM}outro`);
	});

	it('[MED] a fence nested under a quote marker still fences its content', () => {
		const doc = '> ```\n> const x = 1;\n> ```';
		expect(apply(doc, 10, 'rtl')).toBe(doc);
	});

	it('[MED] indented-code-shaped lines are never marked', () => {
		const doc = 'نص\n    const x = 1;';
		// The indented line is inside the caret's block but keeps its bytes:
		const out = apply(doc, 1, 'rtl');
		expect(out).toBe(`${RLM}نص\n    const x = 1;`);
	});

	it('[LOW] link-reference / footnote definitions are skipped', () => {
		expect(apply('[quran]: https://quran.com', 3, 'rtl')).toBe('[quran]: https://quran.com');
	});

	it('[LOW] digit-only lines ARE content — a forced block never renders half-flipped', () => {
		const doc = '123 456\nعربي';
		expect(apply(doc, 1, 'rtl')).toBe(`${RLM}123 456\n${RLM}عربي`);
	});

	it('[AUDIT-FAIL] a callout HEADER is never marked (a mark before [! severs the callout)', () => {
		const doc = '> [!note] ملاحظة\n> نص داخل الصندوق';
		const out = apply(doc, 3, 'rtl');
		// The header keeps its bytes; the CONTENT line still gets its mark after `> `.
		expect(out).toBe(`> [!note] ملاحظة\n> ${RLM}نص داخل الصندوق`);
	});
});

describe('PJ-106 §B4 — bidiPlugin detectLineDir honors the marks (no fighting)', () => {
	it('[MED] a checked task-line reads past the [x] — the x is never the first strong char', () => {
		expect(detectLineDir('- [x] مهمة عربية')).toBe('rtl');
		expect(detectLineDir(`- [x] ${LRM}مهمة`)).toBe('ltr'); // and the mark still wins after it
	});

	it('RLM + Latin content → rtl (the override wins over first-strong Latin)', () => {
		expect(detectLineDir(`${RLM}hello world`)).toBe('rtl');
	});

	it('LRM + Arabic content → ltr (the override wins over first-strong Arabic)', () => {
		expect(detectLineDir(`${LRM}مرحبا`)).toBe('ltr');
	});

	it('marks after a list marker still win (the syntax strip preserves them)', () => {
		expect(detectLineDir(`- ${RLM}item`)).toBe('rtl');
		expect(detectLineDir(`# ${LRM}عنوان`)).toBe('ltr');
	});

	it('unmarked lines keep their first-strong behavior (no regression)', () => {
		expect(detectLineDir('hello')).toBe('ltr');
		expect(detectLineDir('مرحبا')).toBe('rtl');
		expect(detectLineDir('')).toBeNull();
	});

	it('a callout HEADER takes its direction from the TITLE, not the hidden [!type] keyword', () => {
		expect(detectLineDir('> [!note] ملاحظة')).toBe('rtl'); // the Boss-reported split box
		expect(detectLineDir('> [!note] English title')).toBe('ltr');
		expect(detectLineDir('> [!فكرة] عنوان عربي')).toBe('rtl'); // custom Arabic trigger
		expect(detectLineDir('> [!note]')).toBeNull(); // no title → neutral, inherits
	});
});
