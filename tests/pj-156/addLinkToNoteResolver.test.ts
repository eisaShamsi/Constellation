/**
 * MIG-105 Stage-0 C7 (PJ-156 E1, 2026-07-26) — addLinkToNote must resolve the
 * source note's library with the boundary-guarded LONGEST-root shape, not
 * first-match. The CLOSED-note branch feeds the resolved name straight into
 * `reindexNote(sourcePath, lib.name)`, so a first-match resolve on a
 * nested-library note WROTE the parent library into note_meta — the same
 * index-corrupting class as the PJ-156 Rust trio (bases/shape/tasks).
 *
 * Drives the REAL store primitive against the mocked IPC bridge
 * (the backlinksLinkMention.test.ts / reopenRecoveryClobber.test.ts pattern).
 */
import { describe, it, expect, vi, beforeEach } from 'vitest';

vi.mock('@tauri-apps/api/core', () => ({ invoke: vi.fn() }));

import { invoke } from '@tauri-apps/api/core';
import {
	addLinkToNote,
	libraryStats,
	activeTabId,
	focusedTabId,
	splitActive,
	splitDirection,
	flushDisposeClearTabs,
} from '$lib/libraries/store';
import * as S from '$lib/editor/noteSession';

const mockInvoke = vi.mocked(invoke);

let disk: Map<string, string>;
let calls: Array<{ cmd: string; args: any }>;

const note = (body: string) => `---\ntitle: T\ncid_cn: N\n---\n${body}`;

function wireInvoke() {
	mockInvoke.mockReset();
	mockInvoke.mockImplementation(async (cmd: string, args?: any) => {
		calls.push({ cmd, args });
		switch (cmd) {
			case 'read_note': {
				const c = disk.get(args.filePath);
				if (c === undefined) throw new Error('missing');
				return c;
			}
			case 'write_note': {
				disk.set(args.filePath, args.content);
				return undefined;
			}
			case 'constellation_search_reindex':
				return undefined;
			default:
				return undefined;
		}
	});
}

const reindexCalls = () => calls.filter((c) => c.cmd === 'constellation_search_reindex');

beforeEach(async () => {
	await flushDisposeClearTabs('test-reset');
	activeTabId.set(null);
	focusedTabId.set(null);
	splitActive.set(false);
	splitDirection.set('vertical');
	S.closeAll();
	disk = new Map();
	calls = [];
	wireInvoke();
});

describe('addLinkToNote — CLOSED-note library resolution (PJ-156 E1)', () => {
	it('T1: a nested-library note reindexes under the NESTED library, not the root', async () => {
		// Registry order mirrors the live universe: the root library FIRST —
		// exactly the order that made the old first-match return the root.
		libraryStats.set([
			{ name: 'Root', path: '/lib' } as any,
			{ name: 'Nested', path: '/lib/nested' } as any,
		]);
		const P = '/lib/nested/e.md';
		disk.set(P, note('body'));

		await addLinkToNote(P, 'supports', 'Target Note');

		const rc = reindexCalls().find((c) => c.args.notePath === P);
		expect(rc).toBeTruthy();
		expect(rc!.args.libraryName).toBe('Nested'); // longest-root-wins, NOT first-match 'Root'
		expect(disk.get(P)).toContain('[[Target Note]]'); // the link itself landed
	});

	it('T2: a root-library note still resolves to the root', async () => {
		libraryStats.set([
			{ name: 'Root', path: '/lib' } as any,
			{ name: 'Nested', path: '/lib/nested' } as any,
		]);
		const P = '/lib/a.md';
		disk.set(P, note('body'));

		await addLinkToNote(P, 'supports', 'Target Note');

		const rc = reindexCalls().find((c) => c.args.notePath === P);
		expect(rc).toBeTruthy();
		expect(rc!.args.libraryName).toBe('Root');
	});

	it('T3: the boundary guard — "/library2" is not inside "/lib"', async () => {
		libraryStats.set([
			{ name: 'Lib', path: '/lib' } as any,
			{ name: 'Library2', path: '/library2' } as any,
		]);
		const P = '/library2/x.md';
		disk.set(P, note('body'));

		await addLinkToNote(P, 'supports', 'Target Note');

		const rc = reindexCalls().find((c) => c.args.notePath === P);
		expect(rc).toBeTruthy();
		// The old unbounded startsWith matched '/lib' first (a false prefix).
		expect(rc!.args.libraryName).toBe('Library2');
	});
});
