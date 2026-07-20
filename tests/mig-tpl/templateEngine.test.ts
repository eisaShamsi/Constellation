/**
 * MIG-TPL §1 — the template engine's four de-fusings.
 *
 * These are surgical BUG FIXES, not the tokenizer redesign (that is Option B's engine pillar).
 * Each test was written RED against the shipped engine and names the exact defect it pins.
 *
 * Audit provenance: `docs/concept-papers/MIG-TPL-Templates-v2-Architect.md`, engineDefects.
 */
import { describe, it, expect } from 'vitest';
import { processTemplateAsync, extractTemplateBody } from '$lib/templates/engine';

const ctx = (over: Partial<Record<string, any>> = {}) => ({
	title: 'T', folder: 'F', library: 'L', ...over,
} as any);

describe('formatDate — repeated tokens (serial .replace had no /g)', () => {
	// DEFECT: formatDate chained `.replace('YYYY', …).replace('YY', …)…` with NO global flag, so
	// only the FIRST occurrence of each token was replaced — and the leftover second `YYYY` was
	// then eaten by the `YY` rule, yielding e.g. "2026-… 26YY". Any format string that repeats a
	// token (a header + footer date, "YYYY/MM/DD (YYYY)") corrupted silently.
	it('replaces EVERY occurrence of a repeated token', async () => {
		const r = await processTemplateAsync('{{date:YYYY YYYY}}', ctx());
		const [a, b] = r.content.split(' ');
		expect(a).toMatch(/^\d{4}$/);
		expect(b).toBe(a);            // was "26YY" before the fix
	});

	it('does not let YYYY be cannibalised by the YY rule', async () => {
		const r = await processTemplateAsync('{{date:YYYY}}|{{date:YY}}', ctx());
		const [long, short] = r.content.split('|');
		expect(long).toMatch(/^\d{4}$/);
		expect(short).toMatch(/^\d{2}$/);
		expect(long.endsWith(short)).toBe(true);
	});

	it('handles a repeated month/day token in one format', async () => {
		const r = await processTemplateAsync('{{date:DD-DD MM-MM}}', ctx());
		const [dd, mm] = r.content.split(' ');
		const [d1, d2] = dd.split('-');
		const [m1, m2] = mm.split('-');
		expect(d1).toMatch(/^\d{2}$/); expect(d2).toBe(d1);
		expect(m1).toMatch(/^\d{2}$/); expect(m2).toBe(m1);
	});
});

describe('$-substitution hazard — user text used as a replacement PATTERN', () => {
	// DEFECT: `content.replace(/\{\{title\}\}/gi, ctx.title)` passes user text as the REPLACEMENT
	// string, where `$&`, `$'`, "$`" and `$1` are special. A note titled "Cost $& benefit" injected
	// the matched text instead of itself. Fixed by using function replacers, which never interpret.
	it('keeps a literal $& in the title', async () => {
		const r = await processTemplateAsync('{{title}}', ctx({ title: 'Cost $& benefit' }));
		expect(r.content).toBe('Cost $& benefit');
	});

	it("keeps $' and $` and $1 in folder / library", async () => {
		const r = await processTemplateAsync('{{folder}}|{{library}}', ctx({ folder: "a$'b", library: 'c$`d$1' }));
		expect(r.content).toBe("a$'b|c$`d$1");
	});

	it('keeps $ sequences arriving from the clipboard', async () => {
		const r = await processTemplateAsync('{{clipboard}}', ctx(), {
			getClipboard: async () => 'paid $$100 $& more',
		});
		expect(r.content).toBe('paid $$100 $& more');
	});

	it('keeps $ sequences in a prompt answer and a suggester choice', async () => {
		const r = await processTemplateAsync('{{prompt:Q}}|{{suggester:x,y}}', ctx(), {
			promptUser: async () => 'ans $& one',
			suggestOptions: async () => 'pick $1 two',
		});
		expect(r.content).toBe('ans $& one|pick $1 two');
	});

	it('keeps $ sequences in a frontmatter value', async () => {
		const r = await processTemplateAsync('{{frontmatter.k}}', ctx({ frontmatter: { k: 'v $& w' } }));
		expect(r.content).toBe('v $& w');
	});
});

describe('prompt / suggester answers must be INERT — no re-scan', () => {
	// DEFECT: after substituting an answer the loop re-ran `regex.exec(content)` over the WHOLE
	// content INCLUDING the freshly inserted answer. An answer containing `{{prompt:…}}` was
	// re-prompted — unbounded in the pathological case, and at minimum a second unexpected dialog.
	// The user's answer is DATA; it is never re-interpreted as template syntax.
	it('does not re-prompt when the answer itself contains a {{prompt:}} token', async () => {
		let calls = 0;
		const r = await processTemplateAsync('{{prompt:First}}', ctx(), {
			promptUser: async () => { calls++; return calls === 1 ? '{{prompt:GOTCHA}}' : 'SECOND'; },
		});
		expect(calls).toBe(1);                         // was 2 before the fix
		expect(r.content).toBe('{{prompt:GOTCHA}}');   // the answer survives verbatim
	});

	it('does not re-run the suggester when a choice contains a {{suggester:}} token', async () => {
		let calls = 0;
		const r = await processTemplateAsync('{{suggester:a,b}}', ctx(), {
			suggestOptions: async () => { calls++; return calls === 1 ? '{{suggester:x,y}}' : 'SECOND'; },
		});
		expect(calls).toBe(1);
		expect(r.content).toBe('{{suggester:x,y}}');
	});

	it('still resolves several DISTINCT prompts in order', async () => {
		const asked: string[] = [];
		const r = await processTemplateAsync('{{prompt:One}} {{prompt:Two}} {{prompt:Three}}', ctx(), {
			promptUser: async (q) => { asked.push(q); return q.toLowerCase(); },
		});
		expect(asked).toEqual(['One', 'Two', 'Three']);
		expect(r.content).toBe('one two three');
	});
});

describe('extractTemplateBody — the frontmatter close must be LINE-ANCHORED', () => {
	// DEFECT: `content.indexOf('---', 3)` matched a `---` ANYWHERE, including inside a frontmatter
	// VALUE (an em-dash-heavy title, a URL with `---`). The frontmatter was cut early and its
	// remainder leaked into the inserted body.
	it('does not cut on a --- inside a frontmatter value', () => {
		const tpl = ['---', 'title: A --- B', 'kind: TMPL', '---', 'BODY'].join('\n');
		expect(extractTemplateBody(tpl)).toBe('BODY');
	});

	it('does not cut on a --- inside a URL in frontmatter', () => {
		const tpl = ['---', 'source: https://x.test/a---b', '---', 'BODY'].join('\n');
		expect(extractTemplateBody(tpl)).toBe('BODY');
	});

	it('strips a normal frontmatter block', () => {
		expect(extractTemplateBody('---\ntitle: T\n---\nBODY')).toBe('BODY');
	});

	it('leaves a body with no frontmatter untouched', () => {
		expect(extractTemplateBody('no frontmatter here')).toBe('no frontmatter here');
		expect(extractTemplateBody('--- not frontmatter, just a rule')).toBe('--- not frontmatter, just a rule');
	});

	it('keeps a --- horizontal rule that appears in the BODY', () => {
		const tpl = ['---', 'title: T', '---', 'above', '', '---', '', 'below'].join('\n');
		expect(extractTemplateBody(tpl)).toBe(['above', '', '---', '', 'below'].join('\n'));
	});

	it('tolerates CRLF line endings', () => {
		expect(extractTemplateBody('---\r\ntitle: T\r\n---\r\nBODY')).toBe('BODY');
	});
});

describe('regressions — the ordinary cases still work', () => {
	it('expands date, time, title, folder, library', async () => {
		const r = await processTemplateAsync(
			'{{date}}|{{title}}|{{folder}}|{{library}}', ctx({ title: 'N', folder: 'Fold', library: 'Lib' }));
		const [d, t, f, l] = r.content.split('|');
		expect(d).toMatch(/^\d{4}-\d{2}-\d{2}$/);
		expect([t, f, l]).toEqual(['N', 'Fold', 'Lib']);
	});

	it('reports the cursor offset and removes the marker', async () => {
		const r = await processTemplateAsync('AB{{cursor}}CD', ctx());
		expect(r.content).toBe('ABCD');
		expect(r.cursorOffset).toBe(2);
	});

	it('leaves an unknown variable verbatim rather than silently deleting it', async () => {
		const r = await processTemplateAsync('{{nope}}', ctx());
		expect(r.content).toBe('{{nope}}');
	});
});
