/**
 * PJ-278 (tenth whole-app sweep) — THE POST-DURABLE-SAVE ROUTINE, AND ITS IDENTITY.
 *
 * The sweep found three durable-save paths that reindexed a note but never refreshed its semantic
 * embedding — Focus mode, the save-failure Retry, and conflict-merge resolution. None self-heals:
 * the boot backfill only embeds notes with NO vector at all, and neither index repair nor reindex
 * touches `note_embeddings`, so a body written in Focus left semantic search answering from the
 * pre-Focus text indefinitely. The fix consolidated the whole routine into `afterDurableSave`.
 *
 * The FIRST cut of that consolidation introduced a worse bug than the one it fixed, and the
 * per-build inspection caught it: it read the body from `getNoteModel(tabId)` AFTER the awaited
 * write. A tab id is a SLOT, not a note — `openNoteTab` reuses it in place, and it only flushes
 * the outgoing note when that note is dirty, so a click on note B during A's in-flight write
 * re-seeds the same id with B's model without serialising behind A. A's write then resolved and
 * embedded B's body under A's path (`INSERT OR REPLACE`), silently and permanently; with the tab
 * CLOSED the lookup was `undefined` and `?? ''` force-embedded an empty body over the real vector.
 *
 * These tests pin the property that makes both impossible: the embedded text comes from the BYTES
 * THAT WERE WRITTEN — handed to `onSaved` alongside the path — so there is no window in which the
 * answer can change, and no disposal case to paper over.
 */
import { describe, it, expect, vi, beforeEach } from 'vitest';

vi.mock('@tauri-apps/api/core', () => ({ invoke: vi.fn() }));
vi.mock('@tauri-apps/api/event', () => ({ emit: vi.fn(async () => {}), listen: vi.fn(async () => () => {}) }));

import { invoke } from '@tauri-apps/api/core';
import { afterDurableSave, appSettings } from '$lib/libraries/store';

const mockInvoke = vi.mocked(invoke);
let calls: Array<{ cmd: string; args: any }>;

const file = (body: string) => `---\ntitle: Probe\ncid_cn: C_PROBE\ntags: [a, b]\n---\n${body}`;

/** The note payload the embed command was given, if it was called at all. */
const embedCall = () => calls.find((c) => c.cmd === 'constellation_embed_notes')?.args?.notes?.[0];

beforeEach(() => {
	calls = [];
	mockInvoke.mockReset();
	mockInvoke.mockImplementation(async (cmd: string, args?: any) => {
		calls.push({ cmd, args });
		return undefined;
	});
	appSettings.update((s) => ({ ...s, enabledFeatures: { ...s.enabledFeatures, semanticSearch: true } }));
});

describe('afterDurableSave', () => {
	it('owes the note all three things: second screen, lexical index, semantic vector', async () => {
		afterDurableSave('/lib/A.md', 'A', 'Lib', file('the zarquon body'));
		await Promise.resolve();
		await Promise.resolve();
		expect(calls.some((c) => c.cmd === 'constellation_search_reindex')).toBe(true);
		expect(embedCall()).toBeDefined();
	});

	/** The cross-note bleed, as a property: what gets embedded is what was written. */
	it('embeds the body of the CONTENT IT WAS GIVEN, never a re-derived lookup', async () => {
		afterDurableSave('/lib/A.md', 'A', 'Lib', file('note A body, and only A'));
		await Promise.resolve();
		await Promise.resolve();
		const sent = embedCall();
		expect(sent.path).toBe('/lib/A.md');
		expect(sent.content).toBe('note A body, and only A');
		expect(sent.content).not.toContain('cid_cn'); // frontmatter is not part of the body
		expect(sent.content).not.toContain('---');
	});

	/** The disposal case that `?? ''` used to turn into a confident empty force-embed. */
	it('a note whose tab is gone still embeds its real body — the argument carries it', async () => {
		// No model, no tab, nothing open: afterDurableSave takes no id, so there is nothing to lose.
		afterDurableSave('/lib/Closed.md', 'Closed', 'Lib', file('body that must survive the close'));
		await Promise.resolve();
		await Promise.resolve();
		expect(embedCall().content).toBe('body that must survive the close');
	});

	it('a note with no frontmatter embeds its whole text', async () => {
		afterDurableSave('/lib/Bare.md', 'Bare', 'Lib', 'just a body');
		await Promise.resolve();
		await Promise.resolve();
		expect(embedCall().content).toBe('just a body');
	});

	it('skips the embed entirely when semantic search is off, but still reindexes', async () => {
		appSettings.update((s) => ({ ...s, enabledFeatures: { ...s.enabledFeatures, semanticSearch: false } }));
		afterDurableSave('/lib/A.md', 'A', 'Lib', file('body'));
		await Promise.resolve();
		await Promise.resolve();
		expect(calls.some((c) => c.cmd === 'constellation_search_reindex')).toBe(true);
		expect(embedCall()).toBeUndefined();
	});
});
