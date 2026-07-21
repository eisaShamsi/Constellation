/**
 * MIG-103 D2 — the template door must apply to ITS OWN note, and only when blank.
 *
 * **APP-KILLER, caught by the safety inspection 2026-07-21, in code written the
 * same day.** The door dispatched `detail.path`, and the handler threw it away:
 * `applyTemplateToCurrentNote` targeted `get(focusedTab)` and replaced the WHOLE
 * document (`from: 0, to: doc.length`).
 *
 * The split-view failure: pane A holds a long note and has focus; pane B is blank
 * and shows the door. The door fires on MOUSEDOWN and the picker mounts as a
 * full-screen overlay before mouseup — so the pane's click-to-focus never runs and
 * focus stays on A. Picking a template then wiped A's entire body and merged the
 * template's properties into A. B — the note the user actually pointed at — was
 * untouched. No error; the user was looking at B.
 *
 * The original guards checked focus *consistency* (`still.id === tab.id`), not
 * that the target was the door's note — so they passed while doing the wrong
 * thing. Two layers now: resolve the target by the door's own path, and refuse
 * to replace a document that is not blank.
 *
 * These are structural assertions over the source, the same approach used for the
 * MIG-101 read-only guard, because the logic lives inside a Svelte component.
 */
import { describe, it, expect } from 'vitest';
import { readFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';

const LAYOUT = readFileSync(
	fileURLToPath(new URL('../../src/routes/+layout.svelte', import.meta.url)),
	'utf-8',
);
const NOTEPANE = readFileSync(
	fileURLToPath(new URL('../../src/lib/components/NotePane.svelte', import.meta.url)),
	'utf-8',
);

/** Brace-match a `function NAME(` body out of the source. */
function functionBody(source: string, name: string): string {
	const m = new RegExp(`function ${name}\\s*\\(`).exec(source);
	expect(m, `function ${name} not found`).not.toBeNull();
	const open = source.indexOf('{', m!.index);
	let depth = 0;
	for (let i = open; i < source.length; i++) {
		if (source[i] === '{') depth++;
		else if (source[i] === '}' && --depth === 0) return source.slice(open, i + 1);
	}
	throw new Error(`unbalanced braces reading ${name}`);
}

describe('MIG-103 D2 — the door applies to its own note', () => {
	it('the door still dispatches the note path it belongs to', () => {
		expect(NOTEPANE).toContain('constellation:apply-template-here');
		const idx = NOTEPANE.indexOf('constellation:apply-template-here');
		expect(
			NOTEPANE.slice(idx, idx + 200).includes('path: filePath'),
			'the door must carry its own note path in the event detail',
		).toBe(true);
	});

	/** THE REGRESSION — the handler must READ that path, not discard it. */
	it('handleApplyTemplateHere reads detail.path', () => {
		const body = functionBody(LAYOUT, 'handleApplyTemplateHere');
		expect(
			/detail/.test(body) && /\.path/.test(body),
			'handleApplyTemplateHere must read the dispatched detail.path — discarding it ' +
				'is what let a blank pane wipe the other, focused note in split view.',
		).toBe(true);
		expect(
			/applyTemplateTargetPath\s*=/.test(body),
			'the resolved target path must be recorded for the apply step',
		).toBe(true);
	});

	/** Layer 1 — the apply must target the recorded path, never merely the focus. */
	it('applyTemplateToCurrentNote resolves its target by path, not by focus', () => {
		const body = functionBody(LAYOUT, 'applyTemplateToCurrentNote');
		expect(
			/const targetPath = applyTemplateTargetPath/.test(body),
			'the apply must start from the door-supplied target path',
		).toBe(true);
		expect(
			/openTabs\)?\s*\)?\.find\(\(?t\)?\s*=>\s*t\.path === targetPath\)/.test(body.replace(/\s+/g, ' ')) ||
				body.includes('t.path === targetPath'),
			'the target tab must be found by PATH',
		).toBe(true);
		expect(
			body.includes('getActiveEditorForPath(targetPath)'),
			'the editor view must be resolved from the target path, not the focused tab',
		).toBe(true);
	});

	/** Layer 2 — a whole-document replacement may only run on a blank document. */
	it('refuses to replace a target that is no longer blank', () => {
		const body = functionBody(LAYOUT, 'applyTemplateToCurrentNote');
		expect(
			/stillBlank/.test(body),
			'a full-document replacement must be gated on the target still being blank — ' +
				'otherwise a note that gained content between opening the picker and choosing ' +
				'a template would be silently overwritten',
		).toBe(true);
		const guard = body.indexOf('stillBlank');
		const replace = body.indexOf('view.dispatch');
		expect(guard).toBeGreaterThan(-1);
		expect(replace).toBeGreaterThan(-1);
		expect(guard, 'the blank guard must precede the replacement').toBeLessThan(replace);
	});

	/** The frontmatter merge must never clobber the note's own identity. */
	it('never lets a template overwrite the note identity keys', () => {
		const body = functionBody(LAYOUT, 'applyTemplateToCurrentNote');
		for (const key of ['title', 'cid_cn', 'created']) {
			expect(body.includes(`'${key}'`), `${key} must be in the identity-exclusion set`).toBe(true);
		}
	});
});
