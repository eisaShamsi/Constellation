/**
 * PJ-130 Batch 1 (APP-KILLER) — a display-only window never writes to disk.
 *
 * Confirmed by the whole-app safety inspection (2026-07-20) and re-verified live
 * against HEAD: the second screen mounts core components read-only, but PJ-108
 * makes it PRESERVE the shared crash-recovery net, and `resolveNoteContent` still
 * returns `recoveredFromNet: true` on that path — so `openNoteTab` called
 * `markModelRecoveredFromNet` (→ `m.version++`) and the second-screen model was
 * born DIRTY. Any departure from it (closeTab, tab switch, history nav) then
 * routed through `flushOutgoing` and durably wrote that stale snapshot over the
 * note; because the MAIN window's model for the same note was clean, the watcher
 * ADOPTED the revert instead of raising a conflict. Silent loss of newer content,
 * on screen and on disk. "Additional screens are displays, not domains."
 *
 * `flushOutgoing` and the born-dirty sites are module-private, so this pins the
 * two guards STRUCTURALLY (the same approach used for the MIG-101 read-only
 * guard). Both are written to fail if either guard is removed.
 */
import { describe, it, expect } from 'vitest';
import { readFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';

const SRC = fileURLToPath(new URL('../../src/lib/libraries/store.ts', import.meta.url));
const source = readFileSync(SRC, 'utf-8');

/** Body of a `function NAME(` (or `async function NAME(`) by brace-matching. */
function functionBody(name: string): string {
	const m = new RegExp(`function ${name}\\s*\\(`).exec(source);
	expect(m, `function ${name} not found in store.ts`).not.toBeNull();
	const open = source.indexOf('{', m!.index);
	let depth = 0;
	for (let i = open; i < source.length; i++) {
		if (source[i] === '{') depth++;
		else if (source[i] === '}' && --depth === 0) return source.slice(open, i + 1);
	}
	throw new Error(`unbalanced braces reading ${name}`);
}

describe('PJ-130 — display-only windows never write', () => {
	/** THE PRIMARY GUARD — the choke point every departure-flush routes through. */
	it('flushOutgoing returns early when displayOnlyWindow', () => {
		const body = functionBody('flushOutgoing');
		const guard = /if\s*\(\s*displayOnlyWindow\s*\)\s*return/;
		expect(
			guard.test(body),
			'flushOutgoing must return before any write when displayOnlyWindow — it is the ' +
				'single choke point for closeTab / tab-switch / history-nav flushes.',
		).toBe(true);

		// The guard must be BEFORE the write call, or it protects nothing.
		const gi = body.search(guard);
		const write = body.indexOf('flushOutgoingModel(');
		expect(write).toBeGreaterThan(-1);
		expect(gi, 'the displayOnly guard must precede flushOutgoingModel').toBeLessThan(write);
	});

	/** THE BELT — the SS model must not be born dirty in the first place. Every
	 *  markModelRecoveredFromNet call must be gated on `!displayOnlyWindow`. */
	it('no markModelRecoveredFromNet call fires in a display-only window', () => {
		const calls = source.match(/markModelRecoveredFromNet\s*\(/g) ?? [];
		// One of the matches is the import binding line; count the CALL sites (there
		// are three: openNoteTab reuse, openNoteTab new-tab, loadTabHistoryEntry).
		const guarded = source.match(
			/if\s*\(\s*resolved\.recoveredFromNet\s*&&\s*!displayOnlyWindow\s*\)\s*markModelRecoveredFromNet/g,
		) ?? [];
		expect(guarded.length, 'every recoveredFromNet mark must be `&& !displayOnlyWindow`').toBe(3);
		// And there must be NO unguarded `if (resolved.recoveredFromNet) markModel…`.
		const unguarded = source.match(
			/if\s*\(\s*resolved\.recoveredFromNet\s*\)\s*markModelRecoveredFromNet/g,
		) ?? [];
		expect(unguarded.length, 'an unguarded recoveredFromNet mark survives — SS can be born dirty').toBe(0);
		expect(calls.length).toBeGreaterThanOrEqual(3); // sanity: the sites still exist
	});
});
