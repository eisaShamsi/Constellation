/**
 * PJ-294 — the key combinations THE EDITOR owns, derived from the keymaps actually installed.
 *
 * ## Why this module exists
 *
 * The Hotkeys screen must refuse to hand out a combination the editor answers, because the global
 * dispatcher is registered capture-phase on `document` and calls `preventDefault` before
 * CodeMirror sees the event — CM6's `runHandlers` then stops on an already-defaulted event, so the
 * editor's binding never runs. Give away `Ctrl+Z` and undo dies in every note, silently.
 *
 * The first version of this guard was a hand-written list in `utils.ts`, and the safety gate found
 * a missing source on three consecutive rounds — first Constellation's own RTL motion keymaps,
 * then CodeMirror's stock ones. The list carried a comment claiming it was "derived"; it was not.
 * It named eight combinations where the installed keymaps bind about thirty-five.
 *
 * So this asks the keymaps themselves. The sources are every keymap NotePane installs — read off
 * its extension list rather than recalled: `defaultKeymap`, `historyKeymap`, `searchKeymap`,
 * `closeBracketsKeymap`, `autocompletion()`'s `completionKeymap`, and the project's own bindings
 * below. A binding added to any of them by a future package upgrade arrives here without anyone
 * remembering to copy it across.
 *
 * An earlier version of this comment claimed "the three NotePane installs" and named three of the
 * six; `completionKeymap` (Ctrl+Space) was the gap the gate found, and the fix was to go and READ
 * the extension list instead of adding one more source per round.
 *
 * It lives under `src/lib/editor/` rather than in `utils.ts` deliberately: `utils` is imported
 * almost everywhere, and pulling CodeMirror into it would put editor packages in every bundle that
 * merely wanted `detectDir` (Rule 6). Only the Settings screen needs this, and it already lives
 * beside the editor.
 */
import { defaultKeymap, historyKeymap } from '@codemirror/commands';
import { searchKeymap } from '@codemirror/search';
import { closeBracketsKeymap, completionKeymap } from '@codemirror/autocomplete';
import { DEFAULT_SHORTCUTS } from '$lib/utils';

/**
 * EVERY combination one binding registers — not merely its `key`.
 *
 * The first cut of this module read `b.key` alone, and the gate found the gap: CodeMirror also
 * registers `"Shift-" + key` for any binding carrying a `shift:` handler (`@codemirror/view`,
 * `add(scope, "Shift-" + name, b.shift, …)`), and a binding may name its combination under
 * `mac` / `win` / `linux` instead of `key`. So nine live combinations — `Ctrl+Shift+End`,
 * `Ctrl+Shift+ArrowUp`, `Ctrl+Shift+G` among them — were reported free, and giving one away kills
 * select-to-end-of-document in every note as surely as giving away Ctrl+Z kills undo.
 *
 * A derivation that reads one field of a structure with four is a hand-list wearing a loop.
 */
function bindingCombos(b: { key?: string; mac?: string; win?: string; linux?: string; shift?: unknown }): string[] {
	const named = [b.key, b.mac, b.win, b.linux].filter((k): k is string => typeof k === 'string');
	return named.flatMap((k) => (b.shift ? [k, `Shift-${k}`] : [k]));
}

/** CodeMirror notation ("Shift-Mod-l") → the canonical form the app stores ("Ctrl+Shift+L"). */
function toCanonical(cm: string): string {
	const parts = cm.split('-');
	const key = parts.pop() ?? '';
	const mods = new Set(parts.map((p) => (p === 'Mod' || p === 'Cmd' || p === 'Meta' ? 'Ctrl' : p)));
	// ★ An UPPERCASE letter in a keymap name means the user must hold Shift to produce it.
	// CodeMirror suppresses Shift when looking up character keys, so `Alt-A` is reached by
	// physically pressing Shift+Alt+A — and that is what `eventToShortcut` records. Without this
	// the reservation was exactly INVERTED: it refused `Alt+A`, which the editor never answers,
	// and handed out `Shift+Alt+A`, which it does, killing block-comment toggling in every note.
	if (/^[A-Z]$/.test(key)) mods.add('Shift');
	const ordered = ['Ctrl', 'Shift', 'Alt'].filter((m) => mods.has(m));
	const shown = /^[a-z]$/.test(key) ? key.toUpperCase() : key;
	return [...ordered, shown].join('+');
}

/** A function key — bindable per `shortcutRefusal`, so it must be reservable too. */
const isFunctionKey = (k: string) => /^F([1-9]|1[0-9]|2[0-4])$/.test(k);

/**
 * Constellation's OWN editor bindings (PJ-106 §B1/§B2/§B3), declared here in the same shape the
 * packages use so they flow through `bindingCombos` identically — including the `shift:` variants
 * `Mod-ArrowUp`/`Mod-ArrowDown` carry, which is how `Ctrl+Shift+ArrowUp` enters the table.
 *
 * Written out rather than imported because the keymaps are built inside functions that return
 * `Prec.high(keymap.of([…]))`, so the arrays are not reachable from outside. The `tests/pj-294`
 * source scan is what keeps this honest: it reads the key literals straight out of
 * `src/lib/editor/` and fails if any of them is missing here.
 */
const PROJECT_BINDINGS = [
	{ key: 'Mod-ArrowUp', shift: true },
	{ key: 'Mod-ArrowDown', shift: true },
	{ key: 'Mod-l' },
	{ key: 'Alt-l' },
	{ key: 'Shift-Mod-l' },
	{ key: 'Mod-Shift-s' },
	// PJ-106 §B4 — the RTL logical arrows (`rtlMotion.ts`). Their KEYS are bare and so exempt, but
	// their `shift:` variants are Shift+←/Shift+→: ordinary text selection, which a command must
	// never be allowed to take.
	{ key: 'ArrowLeft', shift: true },
	{ key: 'ArrowRight', shift: true },
];

/**
 * Every MODIFIED combination the installed editor keymaps bind, canonicalised.
 *
 * Unmodified keys (Enter, Tab, ArrowLeft…) are left out on purpose: the dispatcher early-returns
 * for a bare key inside an editable target, so it cannot steal them in the first place.
 */
export const EDITOR_KEYMAP_RESERVED: Record<string, string> = Object.fromEntries(
	[...defaultKeymap, ...historyKeymap, ...searchKeymap, ...closeBracketsKeymap, ...completionKeymap, ...PROJECT_BINDINGS]
		.flatMap(bindingCombos)
		// Unmodified keys are dropped BECAUSE the dispatcher early-returns for a bare key inside an
		// editable target — except that its exemption list is `e.key.length === 1` plus a named set
		// (Home/End/Arrow*/PageUp/PageDown/Backspace/Delete/Enter/Tab/Space), and a FUNCTION key is
		// in neither, while `shortcutRefusal` deliberately makes F1–F24 bindable. So bare `F3`
		// (find-next) was droppable, bindable, and would have killed find-in-note — with the tell
		// sitting in plain sight: `Shift+F3`, the shift half of the SAME binding, was reserved.
		.filter((k) => k.includes('-') || isFunctionKey(k))
		.map((k) => [toCanonical(k), 'editor-keymap'] as const),
);

/**
 * Shipped defaults that ALREADY sit on an editor combination — a pre-existing collision this
 * module makes visible rather than creates.
 *
 * COMPUTED, not listed. The first version of this named six combinations from memory and was
 * wrong twice over — it included `Ctrl+B`, which the editor does not bind at all, and missed
 * `Alt+ArrowLeft` (nav-back), which it does. Intersecting the two tables is the only version that
 * cannot be wrong, and it updates itself when either side changes.
 *
 * Most of these are palette commands whose `action` is literally `() => {}`: the dispatcher still
 * matches, still calls `preventDefault`, and then does nothing — so the key is consumed and
 * neither the command nor the editor's binding happens. Filed as **PJ-295**; resolving it is a
 * decision about whether these should be real commands or not commands at all, which is not this
 * feature's to make.
 *
 * Surfaced — not silently excluded — so the test forbidding a default on a reserved combination
 * can stay strict about every OTHER default.
 */
export const SHADOWED_DEFAULTS: Array<{ id: string; combo: string }> = Object.entries(DEFAULT_SHORTCUTS)
	.filter(([, combo]) => combo in EDITOR_KEYMAP_RESERVED)
	.map(([id, combo]) => ({ id, combo }));

/** Just the combinations, for the tests that only need to skip them. */
export const KNOWN_SHADOWED_DEFAULTS = SHADOWED_DEFAULTS.map((s) => s.combo);
