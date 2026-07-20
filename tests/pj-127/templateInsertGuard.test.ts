/**
 * PJ-125/PJ-105 — the template-insert path guard.
 *
 * The fix routes "insert template" through the active-editor registry, dispatched into the
 * CM6 view so it flows through the one write path (updateListener → editBody → model →
 * debounced save) — never the old raw `write_note` of the stale `tab.content`.
 *
 * The piece that is headlessly provable is the guard: `getActiveEditorForPath` must return
 * the registered view ONLY for the note it belongs to. This is what stops a template landing
 * in the WRONG note when the user switches tabs while a `{{prompt:…}}` dialog is open — the
 * cross-note content class (BUG-023/§C). The dispatch flow itself needs a DOM and is covered
 * by the Boss's live recipe.
 */
import { describe, it, expect, beforeEach } from 'vitest';
import type { EditorView } from '@codemirror/view';
import {
	registerActiveEditor,
	unregisterActiveEditor,
	getActiveEditor,
	getActiveEditorForPath,
} from '$lib/editor/activeEditor';

// The registry stores and compares references; it never dereferences the view. A plain
// object stands in for a real EditorView, which cannot be constructed without a DOM.
const fakeView = (tag: string) => ({ __tag: tag }) as unknown as EditorView;

beforeEach(() => {
	// Reset module state by unregistering whatever a previous test left behind.
	const v = getActiveEditor();
	if (v) unregisterActiveEditor(v);
});

describe('getActiveEditorForPath — the wrong-note dispatch guard', () => {
	it('returns the view for the path it was registered with', () => {
		const v = fakeView('a');
		registerActiveEditor(v, '/L/Alpha/Ideas.md');
		expect(getActiveEditorForPath('/L/Alpha/Ideas.md')).toBe(v);
	});

	it('returns null for any other path — the cross-note case', () => {
		const v = fakeView('a');
		registerActiveEditor(v, '/L/Alpha/Ideas.md');
		expect(getActiveEditorForPath('/L/Other/Note.md')).toBeNull();
	});

	it('follows a re-registration to a different note', () => {
		const v = fakeView('a');
		registerActiveEditor(v, '/L/Alpha/Ideas.md');
		registerActiveEditor(v, '/L/Beta/Plan.md');
		expect(getActiveEditorForPath('/L/Alpha/Ideas.md')).toBeNull();
		expect(getActiveEditorForPath('/L/Beta/Plan.md')).toBe(v);
	});

	it('returns null after the view unregisters (editor unmounted)', () => {
		const v = fakeView('a');
		registerActiveEditor(v, '/L/Alpha/Ideas.md');
		unregisterActiveEditor(v);
		expect(getActiveEditorForPath('/L/Alpha/Ideas.md')).toBeNull();
	});

	it('a DIFFERENT view registering without a path never aliases the previous note', () => {
		// Latent hazard found while writing this test: register(view) with no path used to keep
		// the previous lastPath, so the guarded getter would hand the NEW view out for a note
		// it does not belong to. No caller registers pathless today (NotePane always passes
		// filePath) — the registry now clears the path by construction.
		const v = fakeView('a');
		registerActiveEditor(v, '/L/Alpha/Ideas.md');
		const w = fakeView('b');
		registerActiveEditor(w); // different view, unknown path
		expect(getActiveEditorForPath('/L/Alpha/Ideas.md')).toBeNull();
	});

	it('a same-view re-register without a path KEEPS its known path (the focusin case)', () => {
		const v = fakeView('a');
		registerActiveEditor(v, '/L/Alpha/Ideas.md');
		registerActiveEditor(v); // focusin after mount — same view, no path arg
		expect(getActiveEditorForPath('/L/Alpha/Ideas.md')).toBe(v);
	});
});
