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
 * The two damage tests are written with `it.fails`: they are EXPECTED to fail against the code as it
 * stands, which keeps the suite green (so it can go on gating every slice) while making the moment
 * the fix lands unmissable — vitest reports an expected-failure that PASSES as a failure. MIG-107
 * Slice 4 flips them back to plain `it`. They are the harness single ownership must turn green.
 */
import { describe, it, expect, beforeEach, afterEach, vi } from 'vitest';

vi.mock('@tauri-apps/api/core', () => ({ invoke: vi.fn() }));
import { invoke } from '@tauri-apps/api/core';
import { get } from 'svelte/store';

import { openTabs, saveTabContent, type OpenTab, type FrontmatterProperty, type PropertyType } from '$lib/libraries/store';
import { open as mOpen, closeAll } from '$lib/editor/noteSession';
import { getModel } from '$lib/editor/noteModel';

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
	// `it.fails` — this reproduction is EXPECTED to fail until MIG-107 Slice 4 lands. It is written
	// as an expected-failure rather than deleted or skipped so that (a) the suite stays green and can
	// keep gating every slice, and (b) the moment single ownership makes it pass, vitest reports THIS
	// test as failing ("expected to fail but passed") — a loud, unmissable signal. Slice 4 flips it
	// back to a plain `it`.
	it.fails('a tag added by the tree/menu writer is wiped by the next panel save', async () => {
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
		expect(stale.some((p) => p.key === 'tags')).toBe(false); // the panel cannot know about the tag
		await saveTabContent('a', '/L/N.md', stale, 'body text');

		// THE DAMAGE: the whole-array replace dropped `tags` from the model, and the composed write
		// carries no `tags:` line. The user's tag is gone from the .md with no error of any kind.
		expect(modelKeys()).toContain('tags');
		expect(writes.at(-1)).toContain('tags');
	});
});

describe('PJ-174 AK-3 — two panels bound to one note revert each other', () => {
	// `it.fails` until MIG-107 Slice 4 — see the note on the AK-2 reproduction above.
	it.fails('the second panel to save reinstates its stale copy over the first panel\'s edit', async () => {
		// Both instances mount for the same tabId and are seeded from the same content.
		const inNote = propsFromTabContent();
		const sidebar = propsFromTabContent();

		// (1) The in-note Properties block sets stage → sapling. Model + disk agree.
		await saveTabContent('a', '/L/N.md',
			inNote.map((p) => (p.key === 'stage' ? P('stage', 'sapling') : p)), 'body text');
		expect(getModel('a')!.props.find((p) => p.key === 'stage')!.value).toBe('sapling');

		// (2) The sidebar instance never re-seeded (no store notification), so it still holds
		// `stage: seed`. The user adds a tag THERE.
		await saveTabContent('a', '/L/N.md',
			[...sidebar, L('tags', ['x'])], 'body text');

		// THE DAMAGE: the stage edit is reverted in the model AND on disk, while the in-note panel
		// still displays `sapling` — the UI actively hides the loss until the note is reopened.
		expect(getModel('a')!.props.find((p) => p.key === 'stage')!.value).toBe('sapling');
		expect(writes.at(-1)).toContain('sapling');
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
