/**
 * PJ-114 §3b — theme.css integrity for the app-drawn link tooltip.
 *
 * WHY THIS EXISTS (a real regression, 2026-07-18): editing the comment above `.link-tip` left a
 * stray `*​/` and a block of bare text OUTSIDE any comment. CSS does not error on that — the
 * stray text is absorbed INTO THE NEXT SELECTOR, so the rule silently became
 * `stray text… *​/ .link-tip`, which matches nothing. The tooltip element was still created and
 * still had its text, but carried ZERO styling: no `position: fixed`, no background, no
 * `visibility` control. On screen it simply stopped working, and the row's native tooltip showed
 * through instead.
 *
 * Nothing caught it. `svelte-check` does not read CSS files, vitest had no CSS assertion, and —
 * the trap worth remembering — grepping the BUILT BUNDLE for the new variable name still
 * SUCCEEDED, because the text was present in the output even though it no longer parsed as a
 * rule. Presence in the bundle is not proof that CSS parses.
 *
 * These tests assert the rule exists as a rule, with the declarations the tooltip depends on
 * for its very visibility, and that the Style-Setter variables are actually wired.
 */
import { describe, it, expect } from 'vitest';
import { readFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import postcss from 'postcss';

const THEME = fileURLToPath(new URL('../../src/lib/theme.css', import.meta.url));

function rules(): Map<string, Map<string, string>> {
	const root = postcss.parse(readFileSync(THEME, 'utf8'), { from: THEME });
	const out = new Map<string, Map<string, string>>();
	root.walkRules((r) => {
		const decls = out.get(r.selector) ?? new Map<string, string>();
		// Block body, not a concise arrow: postcss's walker treats a returned value as a
		// stop-signal (`void | false`), and returning the Map is a type error.
		r.walkDecls((d) => { decls.set(d.prop, d.value); });
		out.set(r.selector, decls);
	});
	return out;
}

describe('theme.css — the .link-tip rule survives', () => {
	it('parses, and .link-tip exists as an exact selector', () => {
		const all = rules();
		expect(
			all.has('.link-tip'),
			`.link-tip missing. Selectors containing "link-tip": ${JSON.stringify(
				[...all.keys()].filter((s) => s.includes('link-tip')),
			)}`,
		).toBe(true);
	});

	it('no selector has swallowed stray text or a comment close', () => {
		// The exact shape of the 2026-07-18 regression: a selector carrying `*​/` or newlines of
		// prose. Guards the whole file, not just this rule.
		for (const selector of rules().keys()) {
			expect(selector, `selector looks like absorbed comment text: ${selector}`).not.toContain('*/');
			expect(selector, `selector looks like absorbed comment text: ${selector}`).not.toContain('/*');
		}
	});

	it('carries the declarations the tooltip depends on to be visible at all', () => {
		const d = rules().get('.link-tip')!;
		// Without position:fixed the JS-set left/top do nothing and the box lands in body flow.
		expect(d.get('position')).toBe('fixed');
		// linkTip.ts places the box while hidden, then reveals it. Losing this shows it at 0,0.
		expect(d.get('visibility')).toBe('hidden');
		// The box must never eat the hover that keeps it open.
		expect(d.get('pointer-events')).toBe('none');
		expect(d.has('z-index')).toBe(true);
	});

	it('wires every Style-Setter control, each with a fallback so it is inert until edited', () => {
		const d = rules().get('.link-tip')!;
		const wired: [string, string][] = [
			['max-width', '--link-tip-max-width'],
			['padding', '--link-tip-pad-y'],
			['background', '--link-tip-bg'],
			['color', '--link-tip-text'],
			['border', '--link-tip-border'],
			['border-radius', '--link-tip-radius'],
			['font-size', '--link-tip-font-size'],
			['line-height', '--link-tip-line-height'],
		];
		for (const [prop, cssVar] of wired) {
			const v = d.get(prop);
			expect(v, `${prop} declaration missing`).toBeTruthy();
			expect(v, `${prop} does not read ${cssVar}`).toContain(cssVar);
			// `var(--x)` with no comma would render nothing when unset — every one needs a fallback.
			expect(v!.includes(',') || v!.includes('var(--'), `${prop} has no fallback`).toBe(true);
		}
	});

	it('reuses the shared tooltip shadow rather than minting its own', () => {
		// One dial should move every tooltip's elevation together (Global -> Shadows).
		const shadow = rules().get('.link-tip')!.get('box-shadow');
		expect(shadow).toContain('--tooltip-shadow');
		expect(shadow).not.toContain('--link-tip-shadow');
	});
});
