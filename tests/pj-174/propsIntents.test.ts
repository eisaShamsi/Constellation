/**
 * MIG-107 Slice 2 — the property INTENTS and the read bridge, proven in isolation.
 *
 * Nothing in the app consumes these yet (Slice 3 does), so this slice is inert by construction.
 * These tests are what make it safe to consume: each intent is proven to touch exactly the property
 * it names, to refuse the cases the design says it must refuse, and to leave the model clean when it
 * changed nothing.
 *
 * The property that matters most is the LAST describe block: an intent cannot damage a key it was
 * not asked about. That is what makes the AK-2/AK-3 loss *unrepresentable* rather than guarded
 * against — the whole argument for this migration.
 */
import { describe, it, expect, beforeEach, vi } from 'vitest';
import { get } from 'svelte/store';

vi.mock('@tauri-apps/api/core', () => ({ invoke: vi.fn() }));

import {
	open as mOpen, closeAll, isDirty,
	editPropValue, addPropTo, removePropFrom, renamePropKeyIn, reorderPropsIn, editBody,
} from '$lib/editor/noteSession';
import { getModel } from '$lib/editor/noteModel';
import { propsVersion } from '$lib/editor/propsSignal';
import type { FrontmatterProperty } from '$lib/libraries/store';

const CONTENT = `---\ntitle: N\ncid_cn: C1\nstage: seed\n---\nbody text`;
const keys = () => (getModel('a')?.props ?? []).map((p) => p.key);
const val = (k: string) => getModel('a')?.props.find((p) => p.key === k)?.value;

beforeEach(() => {
	closeAll();
	mOpen('a', '/L/N.md', CONTENT);
});

describe('MIG-107 Slice 2 — setPropValue', () => {
	it('changes exactly the named property and nothing else', () => {
		const before = keys();
		expect(editPropValue('a', 'stage', 'sapling')).toBe(true);
		expect(val('stage')).toBe('sapling');
		expect(keys()).toEqual(before);           // no key added, removed or moved
		expect(val('title')).toBe('N');
		expect(val('cid_cn')).toBe('C1');
	});

	it('is a NO-OP for an unchanged value — the note must not become dirty for nothing', () => {
		const v = getModel('a')!.version;
		expect(editPropValue('a', 'stage', 'seed')).toBe(false);
		expect(getModel('a')!.version).toBe(v);
		expect(isDirty('a')).toBe(false);
	});

	it('refuses a key that is not there (an intent never silently creates one)', () => {
		expect(editPropValue('a', 'nope', 'x')).toBe(false);
		expect(keys()).not.toContain('nope');
	});

	it('carries listItems for a list property', () => {
		addPropTo('a', { key: 'tags', value: 'a', type: 'list', listItems: ['a'] });
		expect(editPropValue('a', 'tags', 'a, b', { listItems: ['a', 'b'] })).toBe(true);
		expect(getModel('a')!.props.find((p) => p.key === 'tags')!.listItems).toEqual(['a', 'b']);
	});

	it('honours the identity guard — a stale caller for another note is refused', () => {
		expect(editPropValue('a', 'stage', 'sapling', undefined, '/L/OTHER.md')).toBe(false);
		expect(val('stage')).toBe('seed');
	});
});

describe('MIG-107 Slice 2 — addProp', () => {
	it('adds a new property', () => {
		expect(addPropTo('a', { key: 'sources', value: 'x', type: 'text' })).toBe(true);
		expect(val('sources')).toBe('x');
	});

	/** PJ-178, closed structurally: a half-typed panel row can no longer reach the file. */
	it('REFUSES an empty key — this is what stops a blank row becoming `"": ""` on disk', () => {
		expect(addPropTo('a', { key: '', value: '', type: 'text' })).toBe(false);
		expect(addPropTo('a', { key: '   ', value: '', type: 'text' })).toBe(false);
		expect(keys()).toEqual(['title', 'cid_cn', 'stage']);
	});

	/** §5.4 ruling: a collision is the caller's to surface, never a silent last-wins. */
	it('REFUSES to overwrite an existing key', () => {
		expect(addPropTo('a', { key: 'stage', value: 'sapling', type: 'text' })).toBe(false);
		expect(val('stage')).toBe('seed'); // untouched
		expect(keys().filter((k) => k === 'stage')).toHaveLength(1);
	});
});

describe('MIG-107 Slice 2 — removeProp / renamePropKey / reorderProps', () => {
	it('removes only the named key', () => {
		expect(removePropFrom('a', 'stage')).toBe(true);
		expect(keys()).toEqual(['title', 'cid_cn']);
		expect(removePropFrom('a', 'stage')).toBe(false); // already gone → no-op, not an error
	});

	it('renames a key in place, preserving position and value', () => {
		expect(renamePropKeyIn('a', 'stage', 'phase')).toBe(true);
		expect(keys()).toEqual(['title', 'cid_cn', 'phase']);
		expect(val('phase')).toBe('seed');
	});

	it('REFUSES a rename onto an existing key — reported, never silently merged', () => {
		expect(renamePropKeyIn('a', 'stage', 'title')).toBe(false);
		expect(val('title')).toBe('N');   // the other property is untouched
		expect(val('stage')).toBe('seed');
	});

	it('refuses an empty new key and a no-op rename', () => {
		expect(renamePropKeyIn('a', 'stage', '')).toBe(false);
		expect(renamePropKeyIn('a', 'stage', 'stage')).toBe(false);
	});

	it('reorders by key, and refuses an unknown anchor rather than guessing', () => {
		expect(reorderPropsIn('a', 'stage', 'title')).toBe(true);
		expect(keys()).toEqual(['stage', 'title', 'cid_cn']);
		expect(reorderPropsIn('a', 'stage', null)).toBe(true);
		expect(keys()).toEqual(['title', 'cid_cn', 'stage']);
		expect(reorderPropsIn('a', 'stage', 'nonexistent')).toBe(false);
		expect(keys()).toEqual(['title', 'cid_cn', 'stage']);
	});
});

describe('MIG-107 Slice 2 — the read bridge (propsSignal)', () => {
	it('ticks on a real property change', () => {
		const before = get(propsVersion);
		editPropValue('a', 'stage', 'sapling');
		expect(get(propsVersion)).toBeGreaterThan(before);
	});

	it('does NOT tick for a no-op — a panel must not be woken for a change that did not happen', () => {
		const before = get(propsVersion);
		editPropValue('a', 'stage', 'seed');          // same value
		addPropTo('a', { key: '', value: '', type: 'text' }); // refused
		renamePropKeyIn('a', 'stage', 'title');       // collision, refused
		expect(get(propsVersion)).toBe(before);
	});

	/** Invariant #3: this signal must stay silent while the user is writing. */
	it('does NOT tick on body typing — it is off the keystroke path', () => {
		const before = get(propsVersion);
		for (let i = 0; i < 25; i++) editBody('a', `body text ${i}`);
		expect(get(propsVersion)).toBe(before);
	});

	/**
	 * Why a single global counter is safe: a panel whose note did not change re-reads the SAME
	 * `props` array reference, so a `$derived` over it sees no change and nothing re-renders.
	 */
	it('leaves an untouched note\'s props array reference identical, so its panel does not re-render', () => {
		mOpen('b', '/L/B.md', CONTENT);
		const refBefore = getModel('b')!.props;
		editPropValue('a', 'stage', 'sapling');   // a DIFFERENT note changes
		expect(getModel('b')!.props).toBe(refBefore); // same reference — no downstream work
	});
});

describe('MIG-107 Slice 2 — THE PROPERTY THIS MIGRATION RESTS ON', () => {
	/**
	 * An intent cannot damage a key it was not asked about. This is what makes the AK-2/AK-3 loss
	 * unrepresentable: there is no operation a panel can issue that reaches another key, so there is
	 * nothing for a stale view to revert even if one existed.
	 */
	it('no intent can touch a property it was not named', () => {
		addPropTo('a', { key: 'tags', value: 'research', type: 'list', listItems: ['research'] });
		const snapshot = (k: string) => JSON.stringify(getModel('a')!.props.find((p) => p.key === k));
		const tagsBefore = snapshot('tags');
		const titleBefore = snapshot('title');

		// Everything a panel can do to OTHER properties:
		editPropValue('a', 'stage', 'sapling');
		addPropTo('a', { key: 'sources', value: 's', type: 'text' });
		removePropFrom('a', 'sources');
		renamePropKeyIn('a', 'stage', 'phase');
		reorderPropsIn('a', 'phase', 'title');

		expect(snapshot('tags')).toBe(tagsBefore);   // the other writer's work is untouched
		expect(snapshot('title')).toBe(titleBefore);
	});

	it('the model is never left holding an empty or duplicate key', () => {
		addPropTo('a', { key: '', value: 'x', type: 'text' });
		addPropTo('a', { key: 'stage', value: 'dup', type: 'text' });
		renamePropKeyIn('a', 'title', 'stage');
		const ks = keys();
		expect(ks).not.toContain('');
		expect(new Set(ks).size).toBe(ks.length); // no duplicates — matches what compose can persist
	});
});

describe('MIG-107 Slice 5 — the three converted writers keep their existing semantics', () => {
	/**
	 * The stage / shape setters matched their key CASE-INSENSITIVELY when they built whole arrays
	 * (`p.key.toLowerCase() === 'stage'`). The intents match exactly, so the conversion resolves the
	 * note's own spelling first. Without that, a note whose frontmatter says `Stage:` would have
	 * gained a SECOND `stage:` key instead of having its own updated — a silent frontmatter
	 * corruption, and precisely the kind of detail a "mechanical" conversion loses.
	 */
	it('updates a note\'s own capitalisation rather than adding a second key', () => {
		closeAll();
		mOpen('c', '/L/C.md', `---\ntitle: N\nStage: seed\n---\nbody`);
		const existing = getModel('c')!.props.find((p) => p.key.toLowerCase() === 'stage')!.key;
		expect(existing).toBe('Stage'); // the note's spelling, not ours

		expect(editPropValue('c', existing, 'sapling')).toBe(true);
		const keys = getModel('c')!.props.map((p) => p.key);
		expect(keys).toEqual(['title', 'Stage']);           // no duplicate key appeared
		expect(getModel('c')!.props.find((p) => p.key === 'Stage')!.value).toBe('sapling');
	});

	/** The template writer's anti-Evernote rule is now enforced by the primitive, not the caller. */
	it('a template can only ADD — it can never overwrite a property the note already has', () => {
		closeAll();
		mOpen('d', '/L/D.md', `---\ntitle: Mine\ncid_cn: C1\n---\nbody`);
		expect(addPropTo('d', { key: 'title', value: 'Template Title', type: 'text' })).toBe(false);
		expect(addPropTo('d', { key: 'cid_cn', value: 'OTHER', type: 'text' })).toBe(false);
		expect(getModel('d')!.props.find((p) => p.key === 'title')!.value).toBe('Mine');
		expect(getModel('d')!.cid).toBe('C1'); // identity is the NOTE's, never the template's
		expect(addPropTo('d', { key: 'status', value: 'draft', type: 'text' })).toBe(true);
	});
});
