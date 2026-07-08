/**
 * MIG-076 §C — THE RUNTIME HARNESS (logic level).
 *
 * Plays each named 2026-06 failure as a full RECIPE SEQUENCE through the real
 * content-flow code the components will call (noteSession over noteModel),
 * against an in-memory fake disk, asserting **screen === disk** and **no
 * cross-note contamination** after every transition. This is the audit's
 * view-vs-disk parity gate, headless and permanent — it re-runs on every
 * future change to the editor's content layer.
 *
 * Scope honesty: this proves the content-flow LOGIC + GLUE end-to-end. It does
 * NOT mount Svelte components / CM6, so a pure-template wiring mistake at
 * integration (e.g. seeding a view from tab.content instead of bodyForView)
 * is the one residual the Boss test still closes. Single ownership removes the
 * stale alternative that made such a mistake possible, so the residual is small.
 */
import { describe, it, expect, beforeEach } from 'vitest';
import * as S from '$lib/editor/noteSession';
import * as M from '$lib/editor/noteModel';
import { parseFrontmatter, type FrontmatterProperty } from '$lib/libraries/store';

const note = (cid: string, body: string, extra = '') =>
	`---\ntitle: T\ncid_cn: ${cid}\n${extra}---\n${body}`;

/** Fake disk + writer injected into the real save path. */
let disk: Map<string, string>;
const write = (path: string, content: string) => { disk.set(path, content); };
const diskBody = (path: string) => parseFrontmatter(disk.get(path) ?? '').body;
const diskCid = (path: string) =>
	parseFrontmatter(disk.get(path) ?? '').properties.find((p) => p.key === 'cid_cn')?.value ?? null;

beforeEach(() => {
	disk = new Map();
	S.closeAll();
});

describe('Recipe A — symptom 1: Focus opens on a freshly-typed note', () => {
	it('the focus seed is the CURRENT body, never empty, and no empty-body write occurs', async () => {
		S.open('t', '/n.md', note('N', '')); // brand-new note: empty body on disk
		S.editBody('t', 'hello world'); // type in NotePane
		await S.save('t', '/n.md', write);
		expect(diskBody('/n.md')).toBe('hello world');

		// ENTER FOCUS — the seed must be the model's current body (the bug: '')
		expect(S.bodyForView('t')).toBe('hello world');

		S.editBody('t', 'hello world\nadded in focus'); // type in focus
		await S.save('t', '/n.md', write, 'focus_pane');
		expect(diskBody('/n.md')).toBe('hello world\nadded in focus');
		expect(S.bodyForView('t')).toBe(diskBody('/n.md')); // screen === disk
		// the note never lost its body to an empty write at any point:
		expect(diskBody('/n.md').length).toBeGreaterThan(0);
	});
});

describe('Recipe B — symptom 2: switch away and return', () => {
	it('returning shows the latest content, never a stale snapshot', async () => {
		S.open('a', '/a.md', note('NA', 'A1'));
		S.open('b', '/b.md', note('NB', 'B1'));
		S.editBody('a', 'A2');
		await S.save('a', '/a.md', write);

		// switch to b, then back to a (switching carries NO content op)
		// returning reads the model, not a resolve-from-snapshot:
		expect(S.bodyForView('a')).toBe('A2');
		expect(S.bodyForView('a')).toBe(diskBody('/a.md')); // screen === disk
	});
});

describe('Recipe C — the landmine: tab switch WHILE in Focus', () => {
	it('the in-focus note saves its OWN content; the other file is never cross-written', async () => {
		S.open('a', '/a.md', note('NA', 'A body'));
		S.open('b', '/b.md', note('NB', 'B body'));
		await S.save('b', '/b.md', write); // B persisted as itself

		// Focus on A; user switches to B; A's view flushes A on teardown:
		S.editBody('a', 'A body edited');
		const rA = await S.save('a', '/a.md', write);
		expect(rA.ok).toBe(true);
		expect(diskBody('/a.md')).toBe('A body edited');
		expect(diskCid('/a.md')).toBe('NA');
		expect(disk.get('/a.md')).not.toContain('B body');

		// A stale callback trying to write A's content to B's path is REFUSED:
		const bad = await S.save('a', '/b.md', write);
		expect(bad.ok).toBe(false);
		if (!bad.ok) expect(bad.reason).toBe('path_mismatch');
		// B on disk still holds ONLY B's content + identity:
		expect(diskBody('/b.md')).toBe('B body');
		expect(diskCid('/b.md')).toBe('NB');
	});
});

describe('Recipe D — rename with a link (the BUG-023 shape)', () => {
	it('renamed note keeps its identity; the linking note keeps its own', async () => {
		S.open('a', '/a.md', note('NA', 'see [[B]]'));
		S.open('b', '/b.md', note('NB', 'B content'));

		// rename B (title-rename or file-rename): identity moves, content intact
		S.repath('b', '/B-renamed.md');
		const rb = await S.save('b', '/B-renamed.md', write);
		expect(rb.ok).toBe(true);
		expect(diskCid('/B-renamed.md')).toBe('NB');
		expect(diskBody('/B-renamed.md')).toBe('B content');

		// the cascade rewrites A's link — A saves its OWN content only:
		S.editBody('a', 'see [[B-renamed]]');
		await S.save('a', '/a.md', write);
		expect(diskCid('/a.md')).toBe('NA');
		expect(diskBody('/a.md')).toBe('see [[B-renamed]]');

		// neither file acquired the other's identity (the BUG-023 wound):
		expect(disk.get('/a.md')).not.toContain('NB');
		expect(disk.get('/B-renamed.md')).not.toContain('NA');
	});
});

describe('Recipe E — second screen / external change (freshness)', () => {
	it('a clean note adopts external edits; a dirty note rejects them (local wins)', () => {
		S.open('t', '/n.md', note('N', 'v1'));
		expect(S.externalChange('t', note('N', 'v2 from second screen'))).toBe(true);
		expect(S.bodyForView('t')).toBe('v2 from second screen');

		S.editBody('t', 'local unsaved edit'); // now dirty
		expect(S.externalChange('t', note('N', 'v3'))).toBe(false);
		expect(S.bodyForView('t')).toBe('local unsaved edit');
	});
});

describe('Recipe F — restart / workspace restore', () => {
	it('every note reopens with its OWN content from disk', async () => {
		S.open('a', '/a.md', note('NA', 'A'));
		S.open('b', '/b.md', note('NB', 'B'));
		S.editBody('a', 'A edited');
		S.editBody('b', 'B edited');
		await S.save('a', '/a.md', write);
		await S.save('b', '/b.md', write);

		S.closeAll(); // restart

		S.open('a', '/a.md', disk.get('/a.md')!);
		S.open('b', '/b.md', disk.get('/b.md')!);
		expect(S.bodyForView('a')).toBe('A edited');
		expect(S.bodyForView('b')).toBe('B edited');
		expect(S.bodyForView('a')).not.toBe(S.bodyForView('b'));
	});
});

describe('Recipe G — new note created while another is open (the 2026-06-12 Boss-test leak)', () => {
	it('a stale prop-save from the PREVIOUS note cannot poison the reused tab’s model', async () => {
		// 1. Note A open on a tab; user edited its properties.
		S.open('t', '/A.md', note('NA', 'A body'));
		// 2. User creates note B — the SAME tab is reused (openNoteTab reuse →
		//    open() now drives the model synchronously to B's identity).
		S.open('t', '/B.md', note('NB', ''));
		// 3. The torn-down PropertyEditor for A fires its last save, addressing
		//    A's path — this is the poison. The write-side identity guard must
		//    REJECT it because the model now holds B.
		S.editProps('t', [{ key: 'cid_cn', value: 'NA', type: 'text' } as FrontmatterProperty], '/A.md');
		// 4. User types B's body and it saves.
		S.editBody('t', 'B body typed', '/B.md');
		await S.save('t', '/B.md', write);
		// B's file carries ONLY B's identity — never A's (the leak that put
		// §C test's cid into §C Eisa No. 2.md).
		expect(diskCid('/B.md')).toBe('NB');
		expect(diskBody('/B.md')).toBe('B body typed');
		expect(disk.get('/B.md')).not.toContain('NA');
	});

	it('a stale BODY flush from the previous note is rejected too', () => {
		S.open('t', '/A.md', note('NA', 'A body'));
		S.open('t', '/B.md', note('NB', 'B body'));
		S.editBody('t', 'A late flush', '/A.md'); // stale path → rejected
		expect(S.bodyForView('t')).toBe('B body'); // model still B's
	});
});

describe('Recipe H — title-rename re-land through the quiesce path (the §D BUG-023 sequence)', () => {
	it('renaming B by title cascades A’s link, and a stale pre-rename flush is REFUSED — both identities intact', async () => {
		// A links B; both open; both persisted as themselves.
		S.open('a', '/a.md', note('NA', 'see [[B]]'));
		S.open('b', '/b.md', note('NB', 'B content'));
		await S.save('a', '/a.md', write);
		await S.save('b', '/b.md', write);

		// 1) TITLE RENAME of B → /B-renamed.md. The live flow (store.renameItem under
		//    §C) rewrites B's frontmatter on disk, fs-renames the old path away, and
		//    RE-SEEDS the model at the new path (openNoteModel). Modeled here:
		const bRenamed = note('NB', 'B content');
		disk.delete('/b.md');             // fs::rename moved the file away
		disk.set('/B-renamed.md', bRenamed);
		S.open('b', '/B-renamed.md', disk.get('/B-renamed.md')!); // = renameItem's re-seed

		// 2) THE POISON VECTOR — a torn-down editor / late callback for B's OLD path
		//    fires a flush DURING the cascade window (the exact in-flight write behind
		//    BUG-023). Under §C the compose REFUSES it because the model now holds the
		//    new path. Without the guard this would RESURRECT a ghost at /b.md (or, with
		//    A's content captured, write a cross-identity file).
		const stale = await S.save('b', '/b.md', write);
		expect(stale.ok).toBe(false);
		if (!stale.ok) expect(stale.reason).toBe('path_mismatch');
		expect(disk.has('/b.md')).toBe(false); // refused → no resurrected ghost at the old path

		// 3) The cascade rewrites A's [[B]] → [[B-renamed]] on disk and reloadTabsFromDisk
		//    re-seeds A's model from the rewritten content.
		const aRewritten = note('NA', 'see [[B-renamed]]');
		disk.set('/a.md', aRewritten);
		S.open('a', '/a.md', disk.get('/a.md')!); // = reloadTabsFromDisk's re-seed

		// 4) Post-cascade saves compose each note's OWN content for its OWN path.
		const ra = await S.save('a', '/a.md', write);
		const rb = await S.save('b', '/B-renamed.md', write);
		expect(ra.ok && rb.ok).toBe(true);

		// view === disk, and NEITHER file acquired the other's identity (the BUG-023 wound):
		expect(diskCid('/B-renamed.md')).toBe('NB');
		expect(diskBody('/B-renamed.md')).toBe('B content');
		expect(diskCid('/a.md')).toBe('NA');
		expect(diskBody('/a.md')).toBe('see [[B-renamed]]');
		expect(disk.get('/a.md')).not.toContain('NB');
		expect(disk.get('/B-renamed.md')).not.toContain('NA');
		expect(S.bodyForView('a')).toBe(diskBody('/a.md'));
		expect(S.bodyForView('b')).toBe(diskBody('/B-renamed.md'));
	});

	it('the freeze invariant: an edit addressing the pre-rename epoch cannot reach the repurposed model', () => {
		S.open('b', '/b.md', note('NB', 'B content'));
		S.repath('b', '/B-renamed.md');                                 // rename moved identity
		S.editBody('b', 'edit from a stale pre-rename view', '/b.md');  // stale path → ignored
		expect(S.bodyForView('b')).toBe('B content');                  // model untouched by the stale write
	});
});

describe('Global invariant — disk never holds a foreign identity', () => {
	it('after a mixed session, every file on disk carries only its own cid', async () => {
		S.open('a', '/a.md', note('NA', 'alpha'));
		S.open('b', '/b.md', note('NB', 'beta'));
		S.open('c', '/c.md', note('NC', 'gamma'));
		S.editBody('a', 'alpha-2');
		S.editBody('c', 'gamma-2');
		await S.save('a', '/a.md', write);
		await S.save('b', '/b.md', write);
		await S.save('c', '/c.md', write);

		expect(diskCid('/a.md')).toBe('NA');
		expect(diskCid('/b.md')).toBe('NB');
		expect(diskCid('/c.md')).toBe('NC');
		expect(diskBody('/a.md')).toBe('alpha-2');
		expect(diskBody('/b.md')).toBe('beta');
		expect(diskBody('/c.md')).toBe('gamma-2');
	});
});

/**
 * Save-Durability (2026-07-08) — the reproduction harness for the "mark-clean-
 * before-durable-write" app-killer. `noteSession.save()` may only mark the model
 * clean AFTER a durable write; a failed write must keep it DIRTY, RETAIN the net,
 * and SURFACE the error (never a silent loss). Drives the REAL save primitive with
 * an injected failing/succeeding writer + spies.
 */
describe('Save-Durability — clean only trails a durable write', () => {
	it('GREEN — a failed write keeps the model DIRTY, retains the net, surfaces once, writes nothing', async () => {
		S.open('t', '/n.md', note('N', 'old body'));
		S.editBody('t', 'freshly typed');
		const calls = { net: [] as unknown[], clearIf: [] as unknown[], err: [] as unknown[], ok: [] as unknown[] };
		const out = await S.save('t', '/n.md', {
			write: () => { throw new Error('EBUSY: file locked'); },
			setNet: (p, c) => calls.net.push([p, c]),
			clearNetIf: (p, c) => calls.clearIf.push([p, c]),
			onError: (i) => calls.err.push(i),
			onSuccess: (i) => calls.ok.push(i),
		});
		expect(out.ok).toBe(false);
		expect((out as { reason?: string }).reason).toBe('write_failed');
		expect(S.isDirty('t')).toBe(true);       // NOT falsely clean — the whole fix
		expect(calls.net.length).toBe(1);        // net set BEFORE the write
		expect(calls.clearIf.length).toBe(0);    // net RETAINED on failure
		expect(calls.err.length).toBe(1);        // surfaced exactly once
		expect(calls.ok.length).toBe(0);
		expect(disk.has('/n.md')).toBe(false);   // nothing reached disk
	});

	it('SUCCESS — a durable write marks clean, compare-and-clears the net with the written content', async () => {
		S.open('t', '/n.md', note('N', 'old'));
		S.editBody('t', 'new content');
		const calls = { net: [] as unknown[], clearIf: [] as string[][], ok: [] as unknown[] };
		const out = await S.save('t', '/n.md', {
			write: (p, c) => { disk.set(p, c); },
			setNet: (p, c) => calls.net.push([p, c]),
			clearNetIf: (p, c) => calls.clearIf.push([p, c]),
			onSuccess: (i) => calls.ok.push(i),
		});
		expect(out.ok).toBe(true);
		expect(S.isDirty('t')).toBe(false);
		expect(calls.net.length).toBe(1);
		expect(calls.clearIf.length).toBe(1);                    // cleared on success
		expect(calls.clearIf[0][1]).toBe(disk.get('/n.md'));     // with exactly what we wrote
		expect(calls.ok.length).toBe(1);
		expect(diskBody('/n.md')).toBe('new content');
	});

	it('RED baseline — the inlined order (markSaved BEFORE a failing write) yields a FALSELY-CLEAN model', () => {
		// Replicates the shipping bug shape (NoteEditor.handleSave:233) to prove why the
		// fix matters: mark clean, THEN write; a failed write leaves isDirty === false.
		M.openModel('t', '/n.md', note('N', 'old'));
		M.setBody('t', 'edited', '/n.md');
		const r = M.compose('t', '/n.md');
		expect(r.ok).toBe(true);
		if (r.ok) M.markSaved('t', r.version);   // WRONG order: clean before durable
		let threw = false;
		try { throw new Error('EBUSY'); } catch { threw = true; } // the write "fails"
		expect(threw).toBe(true);
		expect(M.isDirty('t')).toBe(false);      // ← FALSELY CLEAN — the edit is now unrecoverable
	});

	it('type-during-await keeps the model dirty for the NEWER edit (version semantics)', async () => {
		S.open('t', '/n.md', note('N', ''));
		S.editBody('t', 'v1');
		let release!: () => void;
		const gate = new Promise<void>((res) => { release = res; });
		const p = S.save('t', '/n.md', { write: async (path, c) => { await gate; disk.set(path, c); } });
		S.editBody('t', 'v2');   // user types MORE during the in-flight write
		release();
		await p;
		expect(diskBody('/n.md')).toBe('v1');    // the composed (v1) content persisted
		expect(S.isDirty('t')).toBe(true);       // v2 is still unsaved → correctly dirty
	});

	it('compare-and-clear — a completed save does NOT wipe a NEWER net set during its write', async () => {
		S.open('t', '/n.md', note('N', 'old'));
		S.editBody('t', 'first');
		const nets = new Map<string, string>();
		let release!: () => void;
		const gate = new Promise<void>((res) => { release = res; });
		const out = S.save('t', '/n.md', {
			write: async (path, c) => { await gate; disk.set(path, c); },
			setNet: (path, c) => nets.set(path, c),
			clearNetIf: (path, c) => { if (nets.get(path) === c) nets.delete(path); }, // real compare-and-clear
		});
		nets.set('/n.md', 'NEWER-NET');   // a newer edit replaces the net mid-write (handleFlush)
		release();
		await out;
		expect(nets.get('/n.md')).toBe('NEWER-NET'); // the newer net survived the older write's clear
	});
});

/**
 * APP-KILLER #2 (2026-07-08) — NoteModel-Ownership silent nav-loss. A navigation is
 * a DEPARTURE, and a departure must FLUSH the outgoing dirty model to disk BEFORE its
 * id-slot is re-seeded with the next note (openNoteModel). The reproduction proves
 * (I) flush-before-replace persists the outgoing edit, (J) a FAILED flush ABORTS the
 * nav (never a silent discard), and (K) a late-resolving save cannot poison a model
 * that was swapped in under its id (markSaved's path-lineage guard). Drives the REAL
 * noteSession.flushIfDirty + noteModel.markSaved with the in-memory fake disk.
 */
describe('Recipe I — nav-away-while-dirty flushes the outgoing model', () => {
	it('RED baseline — replacing the model WITHOUT a flush loses the outgoing edit (the bug)', () => {
		S.open('t', '/a.md', note('NA', 'A body'));
		S.editBody('t', 'A body + just typed'); // dirty, inside the 1.5 s debounce window
		// The CURRENT nav: openNoteModel re-seeds the id with B and NO flush runs first.
		S.open('t', '/b.md', note('NB', 'B body'));
		expect(S.bodyForView('t')).toBe('B body');
		expect(disk.has('/a.md')).toBe(false); // A's just-typed text never reached disk — THE APP-KILLER
	});

	it('GREEN — flushIfDirty persists the outgoing edit BEFORE the replace, then B seeds cleanly', async () => {
		S.open('t', '/a.md', note('NA', 'A body'));
		S.editBody('t', 'A body + just typed');
		expect(S.isDirty('t')).toBe(true);

		const r = await S.flushIfDirty('t', write); // the nav flush — sources A's path FROM THE MODEL
		expect(r.ok).toBe(true);
		expect(diskBody('/a.md')).toBe('A body + just typed'); // durable before any replace
		expect(S.isDirty('t')).toBe(false);

		S.open('t', '/b.md', note('NB', 'B body')); // now the replace is safe
		expect(S.bodyForView('t')).toBe('B body');
		expect(diskBody('/a.md')).toBe('A body + just typed'); // A intact on disk
		expect(diskCid('/a.md')).toBe('NA');
	});

	it('a clean outgoing model is a NO-OP flush (no spurious write)', async () => {
		S.open('t', '/a.md', note('NA', 'A body')); // never edited → clean
		const r = await S.flushIfDirty('t', write);
		expect(r.ok).toBe(true);
		expect(disk.has('/a.md')).toBe(false); // nothing written for a clean note
	});
});

describe('Recipe J — a failed nav-flush ABORTS (never a silent discard)', () => {
	it('a throwing writer keeps the model DIRTY, retains the net, signals !ok — the caller must not replace', async () => {
		S.open('t', '/a.md', note('NA', 'A body'));
		S.editBody('t', 'edit that must survive');
		const net = new Map<string, string>();
		const r = await S.flushIfDirty('t', {
			write: () => { throw new Error('EBUSY: file locked'); },
			setNet: (p, c) => net.set(p, c),
			clearNetIf: (p, c) => { if (net.get(p) === c) net.delete(p); },
		});
		expect(r.ok).toBe(false); // abort signal — the store keeps the user on A
		expect(S.isDirty('t')).toBe(true); // NOT falsely clean
		expect(net.get('/a.md')).toContain('edit that must survive'); // recovery net retained
		expect(disk.has('/a.md')).toBe(false); // nothing durable
	});
});

describe('Recipe K — a late save cannot poison a swapped-in model (markSaved path-guard)', () => {
	it('markSaved for the OLD path no-ops after the id was re-seeded to a new note', () => {
		M.openModel('t', '/a.md', note('NA', 'A'));
		M.setBody('t', 'A edited', '/a.md');
		const rA = M.compose('t', '/a.md');
		expect(rA.ok).toBe(true);
		const verA = rA.ok ? rA.version : -1;

		// openNoteTab reuse re-seeds the id to B while A's autosave is still in flight:
		M.openModel('t', '/b.md', note('NB', 'B')); // version/savedVersion reset to 0
		expect(M.isDirty('t')).toBe(false);

		// A's in-flight save resolves LATE and marks its version clean — addressed to A's path.
		// The guard must REFUSE it because the model now holds /b.md (without the guard it would
		// set B.savedVersion = verA and silently hide B's first edits from autosave).
		M.markSaved('t', verA, '/a.md');

		M.setBody('t', 'B first edit', '/b.md'); // B.version → 1
		expect(M.isDirty('t')).toBe(true); // guarded: dirty (poison would make this false)
	});
});

describe('Recipe M — close-tab-while-dirty flushes before the model is disposed', () => {
	it('the closing tab flushes its edit to disk BEFORE close() deletes the model — nothing lost', async () => {
		S.open('t', '/a.md', note('NA', 'A body'));
		S.editBody('t', 'typed then closed'); // dirty, inside the debounce window
		// closeTab is a DEPARTURE that disposes the model: flush first (best-effort, net-backed),
		// THEN close. The teardown flush can't save it once the model is gone — so the store must.
		const r = await S.flushIfDirty('t', write);
		expect(r.ok).toBe(true);
		expect(diskBody('/a.md')).toBe('typed then closed');
		S.close('t'); // model disposed AFTER the durable flush
		expect(diskBody('/a.md')).toBe('typed then closed'); // edit safe on disk, model gone
	});
});
