/**
 * MIG-101 §A — the read-only write guard, as a structural invariant.
 *
 * The safety inspection (2026-07-20) confirmed an APP-KILLER in this migration's
 * own code: `applyShape` was the ONLY write path in NoteEditor.svelte without
 * `if (readOnly) return`. Its four siblings all carried it. Read-only hosts —
 * the Index preview and the display-only second screen — mount NoteEditor with
 * `readOnly={true}`, and the Index preview keeps a SECOND model for a path that
 * may also be open in a real tab. A shape click there would compose that stale
 * preview body over the live note; and because the real tab's model is CLEAN,
 * the watcher would ADOPT the reverted content rather than raise a conflict
 * sidecar. A silent revert, on screen and on disk.
 *
 * A unit test cannot easily drive a Svelte component's internals, so this pins
 * the rule STRUCTURALLY instead: **any function that reaches the durable save
 * must first refuse when read-only.** That is the invariant "Additional screens
 * are displays, not domains" reduces to in this file.
 */
import { describe, it, expect } from 'vitest';
import { readFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';

const SRC = fileURLToPath(new URL('../../src/lib/components/NoteEditor.svelte', import.meta.url));
const source = readFileSync(SRC, 'utf-8');

/** Slice out one `function NAME(...) { ... }` body by brace-matching. */
function functionBody(name: string): string {
	const start = source.indexOf(`function ${name}(`);
	expect(start, `function ${name} not found in NoteEditor.svelte`).toBeGreaterThan(-1);
	const open = source.indexOf('{', start);
	let depth = 0;
	for (let i = open; i < source.length; i++) {
		if (source[i] === '{') depth++;
		else if (source[i] === '}' && --depth === 0) return source.slice(open, i + 1);
	}
	throw new Error(`unbalanced braces reading ${name}`);
}

/** Every function that performs a durable save. Add to this list, never remove. */
const WRITE_PATHS = ['applyShape', 'handleSave', 'handleFlush', 'handlePromote'];

describe('NoteEditor — read-only hosts never write', () => {
	it.each(WRITE_PATHS)('%s refuses when readOnly', (name) => {
		const body = functionBody(name);
		expect(
			/if\s*\(\s*readOnly\s*\)\s*return/.test(body),
			`${name} performs a write but has no \`if (readOnly) return\` guard. ` +
				`A read-only host (Index preview, second screen) could durably write a stale ` +
				`compose over the note on disk — silently, because a clean receiving model adopts ` +
				`rather than conflicts.`,
		).toBe(true);
	});

	/**
	 * The guard must come BEFORE any mutation. A guard placed after `editProps`
	 * would still corrupt the in-memory model even if the disk write were skipped.
	 */
	it('applyShape refuses before it touches the model', () => {
		const body = functionBody('applyShape');
		const guard = body.search(/if\s*\(\s*readOnly\s*\)\s*return/);
		const firstMutation = body.search(/editProps\(|saveNoteSession\(|invoke\(/);
		expect(guard).toBeGreaterThan(-1);
		expect(firstMutation).toBeGreaterThan(-1);
		expect(guard, 'the readOnly guard must precede every mutation').toBeLessThan(firstMutation);
	});

	/**
	 * Second layer: a display must not even be OFFERED the action. This catches the
	 * menu being re-exposed unconditionally in a later edit.
	 */
	it('the shape menu items are hidden when readOnly', () => {
		const pane = readFileSync(
			fileURLToPath(new URL('../../src/lib/components/NotePane.svelte', import.meta.url)),
			'utf-8',
		);
		const blockStart = pane.indexOf("handleMoreAction('shapeScrap')");
		const blockEnd = pane.indexOf("handleMoreAction('shapeRevert')");
		expect(blockStart).toBeGreaterThan(-1);
		expect(blockEnd).toBeGreaterThan(blockStart);
		// The nearest preceding conditional must be the readOnly gate.
		const before = pane.slice(0, blockStart);
		const lastIf = before.lastIndexOf('{#if');
		expect(
			pane.slice(lastIf, lastIf + 40).includes('!readOnly'),
			'the shape menu items must sit inside {#if !readOnly}',
		).toBe(true);
	});
});
