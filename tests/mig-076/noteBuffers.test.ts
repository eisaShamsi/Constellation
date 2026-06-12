/**
 * MIG-076 §CB-1 — noteBuffers unit tests.
 *
 * The buffer layer is the document model of the Buffer Pattern (ARCHITECT
 * doc §7): non-reactive, snapshot-semantics, identity-travels-with-content.
 * These tests pin the contract the §CB-2/3/4 steps will build on.
 */
import { describe, it, expect, beforeEach, vi, afterEach } from 'vitest';
import {
	setBuffer, getBuffer, deleteBuffer, clearAllBuffers,
	updateBufferPath, bufferCount, toText, parityProbe,
	ensureBuffer, setBufferBody, setBufferProps, composeBuffer,
} from '$lib/editor/noteBuffers';
import { buildFullContent } from '$lib/editor/frontmatter';
import type { FrontmatterProperty } from '$lib/libraries/store';

const P = (key: string, value: string): FrontmatterProperty =>
	({ key, value, type: 'text' }) as FrontmatterProperty;

beforeEach(() => clearAllBuffers());
afterEach(() => vi.restoreAllMocks());

describe('setBuffer / getBuffer round-trip', () => {
	it('stores body as a Text rope that round-trips exactly', () => {
		for (const body of ['', 'one line', 'a\nb\nc', 'trailing\n', '\nleading', 'win\n\n\ngaps']) {
			setBuffer('t1', '/u/n.md', [], body);
			expect(getBuffer('t1')!.body.toString()).toBe(body);
		}
	});

	it('accepts a pre-built Text without conversion', () => {
		const text = toText('shared rope');
		setBuffer('t1', '/u/n.md', [], text);
		expect(getBuffer('t1')!.body).toBe(text);
	});

	it('extracts cid_cn as the buffer identity', () => {
		setBuffer('t1', '/u/n.md', [P('title', 'X'), P('cid_cn', 'NOTE_AB12')], 'b');
		expect(getBuffer('t1')!.cid).toBe('NOTE_AB12');
		setBuffer('t2', '/u/m.md', [P('title', 'Y')], 'b');
		expect(getBuffer('t2')!.cid).toBeNull();
	});

	it('snapshot-clones props — later mutation of the source never reaches the buffer', () => {
		const live: FrontmatterProperty[] = [
			{ key: 'tags', value: 'a, b', type: 'list', listItems: ['a', 'b'] } as FrontmatterProperty,
		];
		setBuffer('t1', '/u/n.md', live, 'b');
		live[0].value = 'MUTATED';
		live[0].listItems!.push('MUTATED');
		const b = getBuffer('t1')!;
		expect(b.props[0].value).toBe('a, b');
		expect(b.props[0].listItems).toEqual(['a', 'b']);
	});

	it('replaces content on re-set but preserves a captured paneState', () => {
		setBuffer('t1', '/u/n.md', [], 'v1');
		const fakeState = { marker: 'pane-state' } as unknown as import('@codemirror/state').EditorState;
		getBuffer('t1')!.paneState = fakeState;
		setBuffer('t1', '/u/n.md', [], 'v2');
		expect(getBuffer('t1')!.body.toString()).toBe('v2');
		expect(getBuffer('t1')!.paneState).toBe(fakeState);
	});

	it('ignores empty tabIds', () => {
		setBuffer('', '/u/n.md', [], 'b');
		expect(bufferCount()).toBe(0);
	});
});

describe('identity updates and lifecycle', () => {
	it('updateBufferPath moves identity without touching content', () => {
		setBuffer('t1', '/u/old.md', [P('cid_cn', 'NOTE_1')], 'keep');
		updateBufferPath('t1', '/u/new.md');
		const b = getBuffer('t1')!;
		expect(b.path).toBe('/u/new.md');
		expect(b.cid).toBe('NOTE_1');
		expect(b.body.toString()).toBe('keep');
	});

	it('deleteBuffer and clearAllBuffers dispose correctly', () => {
		setBuffer('t1', '/a.md', [], '1');
		setBuffer('t2', '/b.md', [], '2');
		deleteBuffer('t1');
		expect(getBuffer('t1')).toBeUndefined();
		expect(bufferCount()).toBe(1);
		clearAllBuffers();
		expect(bufferCount()).toBe(0);
	});
});

describe('parityProbe (DEV drift detector)', () => {
	it('is silent when buffer and legacy pieces agree', () => {
		const err = vi.spyOn(console, 'error').mockImplementation(() => {});
		const props = [P('title', 'X')];
		setBuffer('t1', '/u/n.md', props, 'same body');
		parityProbe('t1', { props, body: 'same body' }, 'test');
		expect(err).not.toHaveBeenCalled();
	});

	it('screams on body drift', () => {
		const err = vi.spyOn(console, 'error').mockImplementation(() => {});
		setBuffer('t1', '/u/n.md', [], 'buffer body');
		parityProbe('t1', { props: [], body: 'legacy body' }, 'test');
		expect(err).toHaveBeenCalledOnce();
		expect(String(err.mock.calls[0][0])).toContain('PARITY MISMATCH');
	});

	it('screams on props drift', () => {
		const err = vi.spyOn(console, 'error').mockImplementation(() => {});
		setBuffer('t1', '/u/n.md', [P('stage', 'spark')], 'b');
		parityProbe('t1', { props: [P('stage', 'birth')], body: 'b' }, 'test');
		expect(err).toHaveBeenCalledOnce();
	});

	it('flags a missing buffer as a missed creation site', () => {
		const err = vi.spyOn(console, 'error').mockImplementation(() => {});
		parityProbe('ghost', { props: [], body: '' }, 'test');
		expect(String(err.mock.calls[0][0])).toContain('missed creation site');
	});
});

describe('§CB-2 — partial setters', () => {
	it('setBufferBody updates the body half only', () => {
		setBuffer('t1', '/u/n.md', [P('cid_cn', 'NOTE_1'), P('stage', 'spark')], 'old');
		setBufferBody('t1', 'new body');
		const b = getBuffer('t1')!;
		expect(b.body.toString()).toBe('new body');
		expect(b.props.map(p => p.value)).toEqual(['NOTE_1', 'spark']);
		expect(b.cid).toBe('NOTE_1');
	});

	it('setBufferProps updates the props half only and re-extracts cid', () => {
		setBuffer('t1', '/u/n.md', [P('cid_cn', 'NOTE_1')], 'keep');
		setBufferProps('t1', [P('cid_cn', 'NOTE_2'), P('stage', 'birth')]);
		const b = getBuffer('t1')!;
		expect(b.body.toString()).toBe('keep');
		expect(b.cid).toBe('NOTE_2');
	});

	it('both refuse (dev-error, no create) when the buffer is absent', () => {
		const err = vi.spyOn(console, 'error').mockImplementation(() => {});
		setBufferBody('ghost', 'x');
		setBufferProps('ghost', []);
		expect(getBuffer('ghost')).toBeUndefined();
		expect(err).toHaveBeenCalledTimes(2);
	});
});

describe('§CB-2 — ensureBuffer', () => {
	it('creates from content when absent', () => {
		ensureBuffer('t1', '/u/n.md', '---\ncid_cn: NOTE_9\n---\nhello');
		const b = getBuffer('t1')!;
		expect(b.cid).toBe('NOTE_9');
		expect(b.body.toString()).toBe('hello');
	});

	it('is a no-op when the buffer already matches the path', () => {
		setBuffer('t1', '/u/n.md', [], 'live edits');
		ensureBuffer('t1', '/u/n.md', 'stale seed');
		expect(getBuffer('t1')!.body.toString()).toBe('live edits');
	});

	it('re-seeds when the host moved its slot to a different path', () => {
		setBuffer('t1', '/u/old.md', [], 'old note');
		ensureBuffer('t1', '/u/new.md', 'new note content');
		const b = getBuffer('t1')!;
		expect(b.path).toBe('/u/new.md');
		expect(b.body.toString()).toBe('new note content');
	});
});

describe('§CB-2 — composeBuffer (the single content source)', () => {
	it('composes props + body from the one buffer', () => {
		const props = [P('cid_cn', 'NOTE_5')];
		setBuffer('t1', '/u/n.md', props, 'the body');
		const r = composeBuffer('t1', '/u/n.md', 'test');
		expect(r.ok).toBe(true);
		if (r.ok) {
			expect(r.content).toBe(buildFullContent(props, 'the body'));
			expect(r.body).toBe('the body');
			expect(r.path).toBe('/u/n.md');
			expect(r.cid).toBe('NOTE_5');
		}
	});

	it('REFUSES on path mismatch (the Frankenstein guard)', () => {
		const err = vi.spyOn(console, 'error').mockImplementation(() => {});
		setBuffer('t1', '/u/actual.md', [], 'body');
		const r = composeBuffer('t1', '/u/other.md', 'test');
		expect(r.ok).toBe(false);
		if (!r.ok) {
			expect(r.reason).toBe('path_mismatch');
			expect(r.bufferPath).toBe('/u/actual.md');
		}
		expect(String(err.mock.calls[0][0])).toContain('REFUSED');
	});

	it('REFUSES when no buffer exists', () => {
		vi.spyOn(console, 'error').mockImplementation(() => {});
		const r = composeBuffer('ghost', '/u/n.md', 'test');
		expect(r.ok).toBe(false);
		if (!r.ok) expect(r.reason).toBe('no_buffer');
	});
});
