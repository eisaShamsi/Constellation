/**
 * Recipe Q (PJ-089, 2026-07-13) — THE INDEX-PANEL PREVIEW TWO-WRITABLE-MODEL CLOBBER.
 *
 * The Index panel's split-pane preview (`+layout.svelte` handleIndexNoteClick →
 * `<NoteEditor tab={indexNoteTab}>`) mounts a WRITABLE editor on a standalone tab
 * whose id is a fresh `index_preview_${Date.now()}` — distinct from any main tab's id.
 * Because the single-ownership `models` Map is keyed by ID (noteModel.ts), an
 * already-open note gets a SECOND, independent NoteModel: one for the main tab, one
 * for the preview. Both are writable and both target the same path on disk.
 *
 * Worse, the preview tab is a standalone `$state`, never in `openTabs`, so the
 * watcher/second-screen reconciliation (`adoptExternalChangeIntoTabs`, which filters
 * to open-tab paths) NEVER adopts a main-window save into the preview's model. The two
 * models drift with nothing to reconcile them → whoever saves last wins → the other
 * view's edit is silently gone, with NO `.conflict` sidecar (the sidecar path also runs
 * only over `openTabs`).
 *
 * This recipe reproduces the MECHANISM on demand at the content-flow layer, exactly as
 * Recipe O did for PJ-070. The RED half proves two writable models on one path clobber
 * each other; the RED-2 half proves the preview model is invisible to the freshness
 * adopt. The GREEN half proves the fix's invariant: the Boss-chosen approach mounts the
 * preview READ-ONLY, so it issues NO writes → disk always equals what the sole writable
 * owner (the main tab) wrote → the clobber is structurally impossible.
 *
 * Scope honesty: this proves the ownership LOGIC. The FIX itself is a component-wiring
 * change (`readOnly={true}` on the preview mount + an "Open to edit" that promotes the
 * peek to a real single-owner tab via openNoteTab's path-dedup) — its proof is the
 * running-app Boss test. vitest is necessary here, not sufficient, for the editor-
 * lifecycle content-integrity class (CLAUDE.md Reproduce-First).
 */
import { describe, it, expect, beforeEach } from 'vitest';
import * as S from '$lib/editor/noteSession';
import * as M from '$lib/editor/noteModel';
import { parseFrontmatter } from '$lib/libraries/store';

const note = (cid: string, body: string) => `---\ntitle: T\ncid_cn: ${cid}\n---\n${body}`;

let disk: Map<string, string>;
const write = (path: string, content: string) => { disk.set(path, content); };
const diskBody = (path: string) => parseFrontmatter(disk.get(path) ?? '').body;

beforeEach(() => {
	disk = new Map();
	S.closeAll();
});

describe('Recipe Q — Index-panel preview two-writable-model clobber (PJ-089)', () => {
	it('RED (today): a preview edit composed from a stale seed CLOBBERS a newer main-tab save', async () => {
		const PATH = '/n.md';
		// The note is open in the MAIN window (a real store tab).
		S.open('main', PATH, note('N', 'line one'));
		S.editBody('main', 'line one\nmain edit A');
		await S.save('main', PATH, write);
		expect(diskBody(PATH)).toBe('line one\nmain edit A');

		// The user opens the Index panel and clicks the SAME note. handleIndexNoteClick
		// reads it FRESH from disk (read_note) into a standalone tab with a unique id →
		// a SECOND, independent writable model. At this instant both models agree.
		S.open('index_preview_1', PATH, disk.get(PATH)!);
		expect(S.bodyForView('index_preview_1')).toBe('line one\nmain edit A');

		// Back in the main tab the user keeps working; the main model saves again.
		S.editBody('main', 'line one\nmain edit A\nmain edit B');
		await S.save('main', PATH, write);
		expect(diskBody(PATH)).toBe('line one\nmain edit A\nmain edit B');

		// The preview model was NEVER told about main-edit-B (it is not in openTabs, so the
		// watcher-adopt skips it). Its seed is stale. The user tweaks the preview and it saves:
		S.editBody('index_preview_1', 'line one\nmain edit A\npreview tweak');
		const r = await S.save('index_preview_1', PATH, write);
		expect(r.ok).toBe(true);

		// THE WOUND — main-edit-B is silently gone from disk; the stale preview save won:
		expect(diskBody(PATH)).toBe('line one\nmain edit A\npreview tweak');
		expect(disk.get(PATH)).not.toContain('main edit B'); // last-writer-wins, no conflict surfaced

		// Symmetry: the two models are genuinely independent — touching the preview never
		// changed the main model (I5 independence), which is exactly why nothing reconciled.
		expect(S.bodyForView('main')).toBe('line one\nmain edit A\nmain edit B'); // main view still shows B
		expect(S.bodyForView('main')).not.toBe(diskBody(PATH));                    // …but disk no longer has B
	});

	it('RED-2 (today): the preview model is invisible to the freshness adopt — a main save cannot reach it', async () => {
		const PATH = '/n.md';
		S.open('main', PATH, note('N', 'v1'));
		// Preview opens the same note (fresh from disk) — second model, same path.
		S.open('index_preview_1', PATH, note('N', 'v1'));

		// Main saves a new version.
		S.editBody('main', 'v2 from main');
		await S.save('main', PATH, write);

		// THE GAP: nothing in the live app calls externalChange for the preview id (it is a
		// standalone $state outside openTabs, so adoptExternalChangeIntoTabs never sees it), so
		// the preview keeps its stale seed while disk has moved on — a silent divergence:
		expect(S.bodyForView('index_preview_1')).toBe('v1'); // still the pre-save seed
		expect(diskBody(PATH)).toBe('v2 from main');         // disk moved on — the preview never learned
		// The preview's model is merely ELIGIBLE to adopt (clean + genuinely-different disk); it is
		// simply never wired to. This is the reconciliation hole the writable preview left open —
		// which the read-only fix closes structurally (a peek issues no writes to diverge in the
		// first place). The externalChange call here just proves eligibility (and re-syncs it):
		expect(M.isDirty('index_preview_1')).toBe(false);
		expect(S.externalChange('index_preview_1', disk.get(PATH)!)).toBe(true);
	});

	it('GREEN (fix): a READ-ONLY preview issues no writes → disk always equals the main tab; clobber is impossible', async () => {
		const PATH = '/n.md';
		// The main tab is the SOLE writable owner.
		S.open('main', PATH, note('N', 'line one'));
		// The preview opens the same note. In the fixed app it mounts `readOnly`, so
		// onDocChange never calls editBody and handleSave/handleFlush early-return — the
		// preview id issues NO save. We model exactly that: the preview only ever READS.
		S.open('index_preview_1', PATH, note('N', 'line one'));
		expect(S.bodyForView('index_preview_1')).toBe('line one'); // the peek shows the note

		// Main edits and saves repeatedly across the preview's whole lifetime.
		S.editBody('main', 'line one\nmain edit A');
		await S.save('main', PATH, write);
		S.editBody('main', 'line one\nmain edit A\nmain edit B');
		await S.save('main', PATH, write);

		// The read-only preview never wrote, so disk is EXACTLY what main wrote — no clobber:
		expect(diskBody(PATH)).toBe('line one\nmain edit A\nmain edit B');
		expect(disk.get(PATH)).toContain('main edit B'); // the edit the RED case erased survives
		// And the preview never dirtied its own model (a read-only view mutates nothing):
		expect(M.isDirty('index_preview_1')).toBe(false);
	});
});
