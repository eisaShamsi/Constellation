/**
 * PJ-174 AK-2/3 — PROPS HAVE NO SINGLE OWNER IN THE UI LAYER.
 *
 * The inspection reported these as two APP-KILLERs (`PropertyEditor.svelte:851` and `:852`). They
 * are ONE defect with two faces, and these tests reproduce the mechanism at the store layer, where
 * the damage actually happens — no Svelte mounting required.
 *
 * The three facts that compose the bug:
 *
 *   1. `setProps` (noteModel.ts:214) REPLACES the model's whole props array — `m.props =
 *      cloneProps(props)` — with no staleness check of any kind.
 *   2. `saveTabContent` deliberately does NOT update `tab.content` and never notifies `openTabs`
 *      ("Do NOT update the store during autosave"). PropertyEditor's `properties` prop derives from
 *      `tab.content`, so after ANY model-based write the panel's `editableProps` is stale — and its
 *      own sync `$effect` compares the same unchanged JSON and skips the re-seed.
 *   3. Two PropertyEditor instances are mounted for the SAME tabId by default (NotePane's embedded
 *      block + the right-sidebar tab), each holding its own independent snapshot.
 *
 * ⇒ Whoever saves second replays a full array assembled from a stale projection, and silently
 * erases whatever the other writer had already persisted. `composeFrontmatter` then sees no diff
 * for the erased key against the OPEN-TIME base, re-emits the base's bytes, and the write is clean:
 * no error, no dirty flag, no conflict, nothing downstream notices.
 *
 * FIXED by MIG-107 Slice 4. Both damage tests were `it.fails` (expected-failures) through Slices 0-3
 * — which kept the suite green so it could go on gating each slice, while making the fix's arrival
 * unmissable. They are now plain `it` and GREEN, driving the SAME user sequences through the panel's
 * real commit path (`panelCommit` below).
 *
 * What changed is not the serializer and not the save: the panel now commits ONE OPERATION PER KEY
 * and may only remove keys it was actually showing, so a key another writer added is unreachable.
 */
import { describe, it, expect, beforeEach, afterEach, vi } from 'vitest';

vi.mock('@tauri-apps/api/core', () => ({ invoke: vi.fn() }));
import { invoke } from '@tauri-apps/api/core';
import { get } from 'svelte/store';

import { openTabs, saveTabContent, type OpenTab, type FrontmatterProperty, type PropertyType } from '$lib/libraries/store';
import { open as mOpen, closeAll } from '$lib/editor/noteSession';
import { getModel } from '$lib/editor/noteModel';
import { plan as planPropOps, apply as applyPropOps } from '$lib/editor/propsCommit';
import { editPropValue, addPropTo, removePropFrom, reorderPropsIn } from '$lib/editor/noteSession';

const mockInvoke = vi.mocked(invoke);

/** The REAL shape — not a local look-alike, so the harness cannot drift from production types. */
type Prop = FrontmatterProperty;
const P = (key: string, value: string, type: PropertyType = 'text'): Prop => ({ key, value, type });
/** A `list` prop carries its items in `listItems`; `value` is the display string. */
const L = (key: string, items: string[]): Prop =>
	({ key, value: items.join(', '), type: 'list', listItems: items });

const CONTENT = `---\ntitle: N\ncid_cn: C1\nstage: seed\n---\nbody text`;

const tab = (id: string, path: string, content: string): OpenTab => ({
	id, path, content,
	libraryName: 'L', libraryPath: '/L', name: 'N.md',
	libraryColor: '#000', history: [path], historyIndex: 0,
});

/** Every `write_note` payload, so we can assert what actually reached disk. */
let writes: string[] = [];

beforeEach(() => {
	closeAll();
	openTabs.set([]);
	writes = [];
	mockInvoke.mockReset();
	mockInvoke.mockImplementation(async (cmd: string, args?: unknown) => {
		if (cmd === 'write_note') writes.push(String((args as { content?: unknown })?.content ?? ''));
		return undefined;
	});
	openTabs.set([tab('a', '/L/N.md', CONTENT)]);
	mOpen('a', '/L/N.md', CONTENT);
});
afterEach(() => { openTabs.set([]); closeAll(); });

/** What the panel would be seeded from — the store projection, NOT the model. */
function propsFromTabContent(): Prop[] {
	const c = get(openTabs)[0].content;
	const fm = c.split('---')[1] ?? '';
	return fm.trim().split('\n').filter(Boolean).map((line) => {
		const i = line.indexOf(':');
		return P(line.slice(0, i).trim(), line.slice(i + 1).trim());
	});
}
const modelKeys = () => (getModel('a')?.props ?? []).map((p) => p.key);

/**
 * MIG-107 Slice 4 — what the PANEL now does on commit, reproduced exactly: plan per-key operations
 * from its own rows against the LIVE model, apply them, then write from the model. `seededKeys` is
 * what that panel was showing when it last read — the only keys its commit is allowed to remove.
 */
async function panelCommit(rows: Prop[], seededKeys: Set<string>) {
	const model = getModel('a')!;
	applyPropOps(planPropOps(rows, model.props, seededKeys), {
		setValue: (k, v, o) => editPropValue('a', k, v, o, '/L/N.md'),
		add: (pr) => addPropTo('a', pr, '/L/N.md'),
		remove: (k) => removePropFrom('a', k, '/L/N.md'),
		order: (k, before) => reorderPropsIn('a', k, before, '/L/N.md'),
	});
	await saveTabContent('a', '/L/N.md', rows, 'body text', false, true);
}

describe('PJ-174 AK-2/3 — the ROOT CAUSE: the panel reads a projection the writers never update', () => {
	it('saveTabContent changes the model but leaves tab.content stale, with no notification', async () => {
		const before = get(openTabs)[0].content;

		await saveTabContent('a', '/L/N.md', [P('title', 'N'), P('cid_cn', 'C1'), P('stage', 'sapling')], 'body text');

		expect(modelKeys()).toContain('stage');
		expect(getModel('a')!.props.find((p) => p.key === 'stage')!.value).toBe('sapling');

		// THE ROOT CAUSE: the projection every PropertyEditor is seeded from did not move, and no
		// subscriber was notified — so any panel still holding `stage: seed` believes it is current.
		expect(get(openTabs)[0].content).toBe(before);
		expect(propsFromTabContent().find((p) => p.key === 'stage')!.value).toBe('seed');
	});
});

describe('PJ-174 AK-2 — a stale replay erases frontmatter another writer already persisted', () => {
	// FLIPPED TO GREEN BY SLICE 4. Until then this was an `it.fails`: the panel handed over a whole
	// array and the tag was erased. It now commits per-key, and the recipe below is the SAME user
	// sequence driven through the panel's real path.
	it('a tag added by the tree/menu writer SURVIVES the next panel save', async () => {
		// Writer A — "Add tag" from the file-tree context menu. Goes through the model and lands
		// on disk (this is `addTagToNote`'s OPEN branch, reduced to its store-level effect).
		await saveTabContent('a', '/L/N.md',
			[P('title', 'N'), P('cid_cn', 'C1'), P('stage', 'seed'), L('tags', ['research'])],
			'body text');
		expect(modelKeys()).toContain('tags');
		expect(writes.at(-1)).toContain('tags');

		// Writer B — the user now edits an UNRELATED property in the Properties panel. The panel was
		// seeded from tab.content, which writer A never updated, so its array has no `tags` at all.
		const stale = propsFromTabContent().map((p) => (p.key === 'stage' ? P('stage', 'sapling') : p));
		expect(stale.some((p) => p.key === 'tags')).toBe(false); // the panel STILL cannot see the tag…
		// …and that no longer matters: `tags` is not among the keys this panel was showing, so its
		// commit has no operation that can reach it.
		await panelCommit(stale, new Set(['title', 'cid_cn', 'stage']));

		// THE DAMAGE: the whole-array replace dropped `tags` from the model, and the composed write
		// carries no `tags:` line. The user's tag is gone from the .md with no error of any kind.
		expect(modelKeys()).toContain('tags');
		expect(writes.at(-1)).toContain('tags');
	});
});

describe('PJ-174 AK-3 — two panels bound to one note revert each other', () => {
	// FLIPPED TO GREEN BY SLICE 4 — see the note on the AK-2 reproduction above.
	it('the second panel to save can no longer erase what it never saw', async () => {
		// Both instances mount for the same tabId and are seeded from the same content.
		const inNote = propsFromTabContent();
		const sidebar = propsFromTabContent();

		// (1) The in-note Properties block sets stage → sapling. Model + disk agree.
		await panelCommit(inNote.map((p) => (p.key === 'stage' ? P('stage', 'sapling') : p)),
			new Set(['title', 'cid_cn', 'stage']));
		expect(getModel('a')!.props.find((p) => p.key === 'stage')!.value).toBe('sapling');

		// (2) The sidebar instance never re-seeded (no store notification), so it still holds
		// `stage: seed`. The user adds a tag THERE.
		// The stale sidebar instance — the worst case, and the one Slice 3's re-seed does NOT cover
		// (it is holding an un-flushed edit). The user adds a tag there.
		await panelCommit([...sidebar, L('tags', ['x'])], new Set(['title', 'cid_cn', 'stage']));

		// The stale panel DOES write its own `stage: seed` — that is an edit it is entitled to make on
		// a key it was showing, and the user sees the result. What it can no longer do is reach a key
		// it never saw. The tag it just added is present, and nothing was silently erased.
		expect(modelKeys()).toContain('tags');
		expect(writes.at(-1)).toContain('tags');
	});
});

describe('MIG-107 Slice 1 — the DESIGN, proven at the substrate before it is built', () => {
	/**
	 * The same two-writer sequence that loses the tag above, with ONE change: writer B builds its
	 * array from the MODEL instead of from the stale projection. Nothing else differs — same
	 * `saveTabContent`, same compose, same base.
	 *
	 * It passes. That is the whole argument for single ownership, demonstrated on the real code
	 * paths rather than asserted in a design document: `composeFrontmatter`'s diff was never wrong,
	 * it was being fed an array that had been assembled from the file as it looked at open time.
	 */
	it('reading from the MODEL makes the very same sequence lossless', async () => {
		// Writer A — "Add tag" from the tree menu.
		await saveTabContent('a', '/L/N.md',
			[P('title', 'N'), P('cid_cn', 'C1'), P('stage', 'seed'), L('tags', ['research'])],
			'body text');

		// Writer B — the panel edits an unrelated property, but sources its array from the model
		// (what a single-owner panel does) rather than from tab.content.
		const fromModel = (getModel('a')!.props ?? []).map((p) =>
			p.key === 'stage' ? P('stage', 'sapling') : p);
		expect(fromModel.some((p) => p.key === 'tags')).toBe(true); // it can SEE the other writer's work
		await saveTabContent('a', '/L/N.md', fromModel, 'body text');

		// Both survive — the tag AND the stage edit, in the model and in what reached disk.
		expect(modelKeys()).toContain('tags');
		expect(getModel('a')!.props.find((p) => p.key === 'stage')!.value).toBe('sapling');
		expect(writes.at(-1)).toContain('tags');
		expect(writes.at(-1)).toContain('stage: sapling');
	});

	/**
	 * The mechanism behind it, pinned: `setProps` deliberately does NOT advance `m.base` (unlike
	 * replaceContent/adoptDisk, which re-base). So compose always diffs against the OPEN-TIME bytes
	 * — correct with one source of truth, and the reason two sources silently cancel each other.
	 */
	it('setProps does not advance the write-base — which is correct, and why one source is required', async () => {
		const baseBefore = getModel('a')?.base?.rawYaml;
		// Assert the base EXISTS, so the comparison below cannot pass vacuously on two undefineds.
		expect(typeof baseBefore).toBe('string');
		await saveTabContent('a', '/L/N.md',
			[P('title', 'N'), P('cid_cn', 'C1'), P('stage', 'sapling')], 'body text');
		expect(getModel('a')?.base?.rawYaml).toBe(baseBefore);
	});
});
