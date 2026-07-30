/**
 * MIG-108 inspection — two APP-KILLERs of the shape PJ-187 already cured for collections,
 * on sibling stores that never received the fix (a Whole-Ecosystem gap the migration's own
 * pre-commit inspection surfaced).
 *
 * The class: **a failed READ presenting as "you have none", which the next WRITE makes true
 * on disk.** Both stores are atomically replaced whole, neither has a `.prev` rotation, and
 * both read paths mapped an error to empty — so one ordinary click destroyed everything.
 *
 *   · `workspaces.json` — the user's NAMED snapshots. MIG-100's own comment calls it "the
 *     precious file"; session.json got `.prev` rotation for exactly this reason and this
 *     never did. One **Save workspace** replaced every earlier snapshot with one entry.
 *   · `property-types.json` — every property-type assignment across every library. One type
 *     assignment replaced the whole registry with a single entry.
 *
 * Why they matter to MIG-108 specifically: its JSON rewrite phase reads and rewrites BOTH of
 * these files. Shipping a migration that rewrites a store which can be silently wiped by an
 * ordinary click is not something to discover afterwards.
 *
 * The cure is the collections one: `loaded` means a read SUCCEEDED — nothing else — and a
 * write is refused until it does.
 */
import { describe, it, expect, vi, beforeEach } from 'vitest';

vi.mock('@tauri-apps/api/core', () => ({ invoke: vi.fn() }));

import { invoke } from '@tauri-apps/api/core';
import { loadWorkspaces, saveWorkspace, workspaces, workspacesError, openTabs } from '$lib/libraries/store';
import {
	loadPropertyTypes,
	seedFromBundle,
	setRegisteredType,
	propertyTypesUnavailable,
} from '$lib/libraries/propertyTypeRegistry';

const mockInvoke = vi.mocked(invoke);
const called = () => mockInvoke.mock.calls.map((c) => c[0] as string);

beforeEach(() => {
	mockInvoke.mockReset();
	mockInvoke.mockResolvedValue(undefined);
	workspaces.set([]);
	workspacesError.set(null);
	openTabs.set([] as never);
});

describe('MIG-108 — a failed read must not let the next write erase the file', () => {
	it('workspaces: a read failure disables saving instead of clobbering the snapshots', async () => {
		mockInvoke.mockImplementation(async (cmd: string) => {
			if (cmd === 'read_universe_workspaces') throw new Error('EBUSY: locked by another process');
			return undefined;
		});
		await loadWorkspaces();

		mockInvoke.mockClear();
		mockInvoke.mockResolvedValue(undefined);
		saveWorkspace('My Layout');

		expect(called()).not.toContain('save_universe_workspaces');
		let err: string | null = null;
		workspacesError.subscribe((v) => (err = v))();
		expect(err).toBeTruthy(); // and it is VISIBLE, not a console line release builds drop
	});

	it('workspaces: after a SUCCESSFUL read, saving works — including from an empty universe', async () => {
		mockInvoke.mockImplementation(async (cmd: string) => (cmd === 'read_universe_workspaces' ? [] : undefined));
		await loadWorkspaces();

		mockInvoke.mockClear();
		mockInvoke.mockResolvedValue(undefined);
		saveWorkspace('First Layout');

		expect(called()).toContain('save_universe_workspaces');
	});

	it('property types: a read failure makes the registry READ-ONLY for the session', async () => {
		mockInvoke.mockImplementation(async (cmd: string) => {
			if (cmd === 'read_universe_property_types') throw new Error('EBUSY');
			return undefined;
		});
		await loadPropertyTypes();
		expect(propertyTypesUnavailable()).toBe(true);

		mockInvoke.mockClear();
		setRegisteredType('MyLib', 'status', 'text');
		await new Promise((r) => setTimeout(r, 600)); // past the 500 ms persist debounce

		expect(called()).not.toContain('save_universe_property_types');
	});

	it('property types: an AMBIGUOUS empty bundle does not unlock writing on its own', async () => {
		// boot_bundle maps a read FAILURE to {} — indistinguishable from genuinely empty — so
		// the bundle may not latch; the explicit read decides.
		seedFromBundle({});
		mockInvoke.mockClear();
		setRegisteredType('MyLib', 'status', 'text');
		await new Promise((r) => setTimeout(r, 600));
		expect(called()).not.toContain('save_universe_property_types');

		// A successful explicit read then unlocks it.
		mockInvoke.mockImplementation(async (cmd: string) =>
			cmd === 'read_universe_property_types' ? {} : undefined,
		);
		await loadPropertyTypes();
		expect(propertyTypesUnavailable()).toBe(false);

		mockInvoke.mockClear();
		mockInvoke.mockResolvedValue(undefined);
		setRegisteredType('MyLib', 'status', 'text');
		await new Promise((r) => setTimeout(r, 600));
		expect(called()).toContain('save_universe_property_types');
	});

	it('property types: a NON-empty bundle proves a successful read and unlocks writing', async () => {
		seedFromBundle({ MyLib: { status: 'text' } });
		expect(propertyTypesUnavailable()).toBe(false);

		mockInvoke.mockClear();
		mockInvoke.mockResolvedValue(undefined);
		setRegisteredType('MyLib', 'stage', 'text');
		await new Promise((r) => setTimeout(r, 600));
		expect(called()).toContain('save_universe_property_types');
	});
});
