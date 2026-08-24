/**
 * PJ-344 — THE REPRODUCTION.
 *
 * Reproduce-First governs this ticket: its MECHANISM was confirmed by reading the code, but its
 * TRIGGER — that an open tab's stored path can differ in form from the path the Rust cascade
 * hands back — was never demonstrated. Until it fires on demand, the reproduction is the only
 * shippable work, and writing a fix first would be guessing in the vocabulary of a plan.
 *
 * WHAT IS BEING REPRODUCED
 *
 * After a rename, the Rust walker rewrites `[[Old]]` → `[[New]]` inside every referrer on disk
 * and returns their paths. `reloadTabsFromDisk` is what pushes those rewritten bytes into any
 * tab the user has open. Its first act is:
 *
 *     const targets = filePaths.filter((fp) => tabs.some((t) => t.path === fp));
 *     if (targets.length === 0) return [];
 *
 * A RAW string comparison. Both siblings that arbitrate the same question — `openNoteTab`'s
 * dedup and `adoptExternalChangeIntoTabs` — were normPath-fixed on 2026-08-01, the latter under
 * a comment recording the real incident it caused ("made every Windows tab miss… the next
 * keystroke's save overwrote the external edit"). This one was not.
 *
 * The consequence if the forms differ: the open tab keeps its PRE-cascade body while disk holds
 * the rewrite. Cascade writes are watcher-suppressed, so nothing adopts it. The model is clean,
 * so nothing writes — until the user types one character, at which point the debounced save
 * composes the stale body and durably puts `[[Old]]` back over the walker's `[[New]]`. The link
 * now points at a title that no longer exists, the index agrees with the reverted disk, and
 * nothing is surfaced.
 *
 * WHY THIS FILE IS A REPRODUCTION AND NOT A UNIT TEST OF A HELPER
 *
 * It drives the real exported function, and it needs no Tauri and no app: the filter and its
 * early `return []` run before any `invoke`. So the skip is observable directly — an empty
 * return from a call that names a file which IS open is the defect, in one assertion.
 *
 * WHAT IT DELIBERATELY DOES NOT DO
 *
 * It does not assert the downstream revert (that needs the editor lifecycle and belongs in the
 * harness). It reproduces the SKIP, which is the root; everything after it follows from it.
 */
import { describe, it, expect, beforeEach, vi } from 'vitest';
import { get } from 'svelte/store';

// The bridge is mocked so the CONTROL case can actually complete instead of dying on an absent
// backend — the precedent is tests/mig-067. `read_note` returns the REWRITTEN body, which is what
// the cascade has just put on disk; every other command is inert.
vi.mock('@tauri-apps/api/core', () => ({
	invoke: vi.fn(async (cmd: string) => {
		if (cmd === 'read_note') return 'See [[New]] here';
		return undefined;
	}),
}));

import { invoke } from '@tauri-apps/api/core';
import { openTabs, reloadTabsFromDisk } from '$lib/libraries/store';

/** Did the cascade actually go and READ this file? That is the discriminator. */
function wasRead(path: string): boolean {
	return (invoke as unknown as { mock: { calls: unknown[][] } }).mock.calls.some(
		(c) => c[0] === 'read_note' && (c[1] as { filePath?: string })?.filePath === path,
	);
}

/** A tab shaped enough for the filter under test; the rest of the model is not consulted. */
function tabAt(path: string) {
	return {
		id: `tab-${path}`,
		path,
		name: path.split(/[\\/]/).pop() ?? path,
		content: 'See [[Old]] here',
		libraryId: 'lib',
		library_id: 'lib',
	} as unknown as ReturnType<typeof get<typeof openTabs>>[number];
}

describe('PJ-344 — reloadTabsFromDisk skips an open tab whose path form differs', () => {
	beforeEach(() => {
		openTabs.set([]);
		vi.clearAllMocks();
	});

	it('MATCHING form: the cascade reads the file back (the control)', async () => {
		// The discriminator is deliberately "was the file READ", not "was the tab adopted".
		// Adoption needs the note-model layer, which this environment does not register — and an
		// assertion that fails for the harness's reasons rather than the code's would make every
		// other test here unreadable. Reading is the first thing past the filter, so it isolates
		// exactly the question this reproduction asks.
		const p = 'E:/Universe/Lib/Referrer.md';
		openTabs.set([tabAt(p)]);
		await reloadTabsFromDisk([p]);
		expect(wasRead(p)).toBe(true);
	});

	it('SEPARATOR drift: a backslash-stored tab is silently skipped', async () => {
		openTabs.set([tabAt('E:\\Universe\\Lib\\Referrer.md')]);
		// The Rust walker returns PathBuf::to_string_lossy — OS-native — while a session-restored
		// or differently-derived tab can hold the other form.
		const out = await reloadTabsFromDisk(['E:/Universe/Lib/Referrer.md']);
		expect(out).toEqual([]); // ← the defect: an open referrer, not reloaded, no error
		// Not merely un-adopted — never even READ. The cascade does not know this tab exists.
		expect(wasRead('E:/Universe/Lib/Referrer.md')).toBe(false);
		// The harm, stated as an assertion: the tab still holds the PRE-cascade body. The next
		// keystroke's debounced save composes from this and writes `[[Old]]` back over disk.
		expect(get(openTabs)[0].content).toBe('See [[Old]] here');
	});

	it('CASE drift: a drive-letter difference is silently skipped', async () => {
		openTabs.set([tabAt('e:/Universe/Lib/Referrer.md')]);
		const out = await reloadTabsFromDisk(['E:/Universe/Lib/Referrer.md']);
		expect(out).toEqual([]);
		expect(wasRead('E:/Universe/Lib/Referrer.md')).toBe(false);
	});

	it('NFC/NFD drift: a decomposed Arabic path is silently skipped', async () => {
		// PJ-092 H1 named NFC divergence "the headline hazard" for exactly this path pair, and the
		// caller three lines above the production call site folds through NFC for that reason —
		// while this function compares raw.
		// NOTE: the first version of this test used an Arabic name whose diacritics are ALREADY
		// separate combining marks, so NFC and NFD were identical and the guard below caught my
		// own bad example rather than a defect. `آ` (alef-with-madda) genuinely decomposes.
		const composed = 'E:/Universe/Lib/آفاق.md'.normalize('NFC');
		const decomposed = composed.normalize('NFD');
		expect(decomposed).not.toBe(composed); // guard: the two forms really do differ
		openTabs.set([tabAt(decomposed)]);
		const out = await reloadTabsFromDisk([composed]);
		expect(out).toEqual([]);
		expect(wasRead(composed)).toBe(false);
	});

	it('the fix that would close it: normPath-equality recognises every drifted form', () => {
		// Stated as an executable spec so the eventual fix has a target. normPath as it exists
		// today folds separators ONLY — it does not lowercase and does not normalize Unicode — so
		// a correct fix needs all three, which is why this ticket is not the one-line change it
		// looks like.
		const fold = (p: string) => p.replace(/\\/g, '/').toLowerCase().normalize('NFC');
		const fromWalker = 'E:/Universe/Lib/Referrer.md';
		for (const stored of [
			'E:\\Universe\\Lib\\Referrer.md',
			'e:/Universe/Lib/Referrer.md',
			'E:/Universe/Lib/Referrer.md',
		]) {
			expect(fold(stored)).toBe(fold(fromWalker));
		}
	});
});
