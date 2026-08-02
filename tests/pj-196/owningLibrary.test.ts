/**
 * "Which library owns this path" — the one resolver.
 *
 * Boss-found 2026-08-02: deleting a note left it sitting in the file tree, unclickable. The
 * file and every index row were correctly destroyed; the tree was simply never told, because
 * the delete handler resolved the library by FIRST match and then refreshed only `if (lib)`.
 * No error surfaced, because nothing had failed.
 *
 * MIG-108 is what made it reachable: every library now lives INSIDE the universe root, so the
 * root library's path is a prefix of all of them and first-match can return the root for a
 * note that belongs to a nested library.
 *
 * Six call sites answered this question three different ways. These cases pin the shared one.
 */
import { describe, it, expect } from 'vitest';
import { owningLibrary } from '$lib/libraries/store';

const L = (path: string, name = path) => ({ path, name });

// The Boss's real shape after MIG-108: the root library first in the list, nested under it.
const ROOT = L('E:\\Constellation Universes\\Eisa Cognitive Knowledge', 'universe_notes');
const PHYSICS = L('E:\\Constellation Universes\\Eisa Cognitive Knowledge\\Physics', 'Physics');
const ARABIC = L('E:\\Constellation Universes\\Eisa Cognitive Knowledge\\علوم عربية', 'علوم عربية');
const LIBS = [ROOT, PHYSICS, ARABIC];

describe('owningLibrary — longest root wins', () => {
	it('returns the NESTED library, not the root that also prefixes it', () => {
		const note = 'E:\\Constellation Universes\\Eisa Cognitive Knowledge\\Physics\\Inertia.md';
		expect(owningLibrary(LIBS, note)).toBe(PHYSICS);
		// first-match would have returned ROOT here — the reported bug
		expect(owningLibrary(LIBS, note)).not.toBe(ROOT);
	});

	it('returns the root for a note that genuinely sits at the root', () => {
		expect(owningLibrary(LIBS, 'E:\\Constellation Universes\\Eisa Cognitive Knowledge\\Loose.md')).toBe(ROOT);
	});

	it('works for a non-Latin library name', () => {
		const note = 'E:\\Constellation Universes\\Eisa Cognitive Knowledge\\علوم عربية\\ابن رشد.md';
		expect(owningLibrary(LIBS, note)).toBe(ARABIC);
	});
});

describe('owningLibrary — the separator boundary', () => {
	it('does NOT match a sibling whose name merely starts the same', () => {
		const libs = [L('E:\\U\\Research'), L('E:\\U\\Research Notes')];
		const note = 'E:\\U\\Research Notes\\Paper.md';
		expect(owningLibrary(libs, note)?.path).toBe('E:\\U\\Research Notes');
	});

	it('a note under a same-prefixed sibling never resolves to the shorter library', () => {
		const libs = [L('E:\\U\\Research')];
		expect(owningLibrary(libs, 'E:\\U\\Research Notes\\Paper.md')).toBeNull();
	});
});

describe('owningLibrary — path shape tolerance', () => {
	it('matches across separator styles', () => {
		const libs = [L('E:\\U\\Lib')];
		expect(owningLibrary(libs, 'E:/U/Lib/note.md')?.path).toBe('E:\\U\\Lib');
	});

	it('matches case-insensitively (Windows)', () => {
		const libs = [L('E:\\U\\Lib')];
		expect(owningLibrary(libs, 'e:\\u\\lib\\Note.md')?.path).toBe('E:\\U\\Lib');
	});

	it('matches the library root itself', () => {
		expect(owningLibrary(LIBS, PHYSICS.path)).toBe(PHYSICS);
	});

	it('tolerates a trailing separator on the library path', () => {
		const libs = [L('E:\\U\\Lib\\')];
		expect(owningLibrary(libs, 'E:\\U\\Lib\\note.md')?.path).toBe('E:\\U\\Lib\\');
	});
});

describe('owningLibrary — no owner is an answer, not a crash', () => {
	it('returns null for a path outside every library', () => {
		expect(owningLibrary(LIBS, 'D:\\Elsewhere\\note.md')).toBeNull();
	});

	it('returns null for an empty library list', () => {
		expect(owningLibrary([], 'E:\\U\\Lib\\note.md')).toBeNull();
	});

	it('ignores a library with an empty path rather than matching everything', () => {
		const libs = [L(''), L('E:\\U\\Lib')];
		expect(owningLibrary(libs, 'E:\\U\\Lib\\note.md')?.path).toBe('E:\\U\\Lib');
		expect(owningLibrary(libs, 'D:\\Nowhere\\note.md')).toBeNull();
	});
});

/**
 * RED-proof, kept in the suite rather than performed once by hand: the two legacy
 * implementations this replaced, run against the same cases. If someone "simplifies"
 * `owningLibrary` back into either shape, the assertions above stop being satisfiable —
 * these pin WHY the shared resolver has to be what it is.
 */
describe('the shapes this replaced — proof the cases discriminate', () => {
	const firstMatch = <T extends { path: string }>(libs: readonly T[], p: string): T | null =>
		libs.find((l) => p.startsWith(l.path)) ?? null;
	const longestButUnbounded = <T extends { path: string }>(libs: readonly T[], p: string): T | null => {
		let best: T | null = null;
		for (const l of libs) if (p === l.path || p.startsWith(l.path)) {
			if (!best || l.path.length > best.path.length) best = l;
		}
		return best;
	};

	it('FIRST-MATCH returns the root library — the reported delete bug', () => {
		const note = 'E:\\Constellation Universes\\Eisa Cognitive Knowledge\\Physics\\Inertia.md';
		expect(firstMatch(LIBS, note)).toBe(ROOT);          // wrong tree refreshed
		expect(owningLibrary(LIBS, note)).toBe(PHYSICS);    // ours
	});

	it('UNBOUNDED longest-match crosses a sibling name boundary', () => {
		const libs = [L('E:\\U\\Research')];
		const note = 'E:\\U\\Research Notes\\Paper.md';
		expect(longestButUnbounded(libs, note)?.path).toBe('E:\\U\\Research'); // wrong owner
		expect(owningLibrary(libs, note)).toBeNull();                          // ours
	});
});
