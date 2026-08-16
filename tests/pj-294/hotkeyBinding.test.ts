/**
 * PJ-294 — THE HOTKEYS SCREEN BINDS KEYS FOR REAL.
 *
 * It has always listed every command, offered to record a key, and then thrown the keystroke away
 * ("hotkey persistence is a future feature"), so rebinding silently did nothing. Everything else
 * was already there: `customShortcuts` is persisted with the settings, and the global dispatcher
 * builds each command with its shortcut already resolved through it.
 *
 * These cover the pure decisions the screen now makes — what may be bound, what collides, and how
 * a binding is shown on each platform. The capture path itself uses `eventToShortcut`, the SAME
 * function the dispatcher matches against; the old hand-rolled version emitted modifiers in a
 * different ORDER, so even had it persisted, a three-modifier binding could never have fired.
 */
import { describe, it, expect } from 'vitest';
import {
	DEFAULT_SHORTCUTS,
	eventToShortcut,
	findShortcutConflict,
	formatShortcut,
	getResolvedShortcut,
	RESERVED_SHORTCUTS,
	parseShortcut,
	shortcutRefusal,
	normalizeShortcut,
} from '$lib/utils';
import { EDITOR_KEYMAP_RESERVED, KNOWN_SHADOWED_DEFAULTS, SHADOWED_DEFAULTS } from '$lib/editor/reservedKeys';

const press = (key: string, mods: Partial<KeyboardEvent> = {}) =>
	({ key, ctrlKey: false, shiftKey: false, altKey: false, metaKey: false, code: '', ...mods }) as KeyboardEvent;

describe('what may be bound', () => {
	it('accepts an ordinary modifier combination', () => {
		expect(shortcutRefusal('Ctrl+Shift+T')).toBeNull();
		expect(shortcutRefusal('Alt+ArrowLeft')).toBeNull();
	});

	/** A bare key would fire on every stray press outside a text field — not something a
	 *  rebinding screen should let someone do to themselves by accident. */
	it('refuses a bare key, but allows function keys', () => {
		expect(shortcutRefusal('K')).toBe('bare-key');
		expect(shortcutRefusal('F5')).toBeNull();
		expect(shortcutRefusal('F13')).toBeNull(); // F13-F24 exist on extended keyboards
		expect(shortcutRefusal('F25')).toBe('bare-key'); // outside F1-F24
		expect(shortcutRefusal('Foo')).toBe('bare-key'); // not a function key at all
	});

	/** The dispatcher documents Escape as "always closes overlays (not remappable)". Rebinding it
	 *  could strand someone inside a full-page surface — including the Settings screen they
	 *  rebound it from. */
	it('refuses Escape, with or without modifiers', () => {
		expect(shortcutRefusal('Escape')).toBe('reserved');
		expect(shortcutRefusal('Ctrl+Escape')).toBe('reserved'); // modifying it does not make it a different key
	});

	it('refuses nothing at all', () => {
		expect(shortcutRefusal('')).toBe('bare-key');
	});
});

describe('conflicts', () => {
	const ids = Object.keys(DEFAULT_SHORTCUTS);

	it('finds the command already answering to a combination', () => {
		expect(findShortcutConflict('new-tab', 'Ctrl+P', {}, ids)).toBe('command-palette');
	});

	it('does not report a command conflicting with itself', () => {
		expect(findShortcutConflict('command-palette', 'Ctrl+P', {}, ids)).toBeNull();
	});

	it('leaves a free combination free', () => {
		expect(findShortcutConflict('new-tab', 'Ctrl+Alt+F9', {}, ids)).toBeNull();
	});

	/** Compares CURRENT bindings, not defaults: once a command has been moved off a combination,
	 *  that combination is free — otherwise the screen would report a conflict with a binding
	 *  nobody has any more. */
	it('respects an existing override on both sides', () => {
		const custom = { 'command-palette': 'Ctrl+Alt+P' };
		expect(findShortcutConflict('new-tab', 'Ctrl+P', custom, ids)).toBeNull();
		expect(findShortcutConflict('new-tab', 'Ctrl+Alt+P', custom, ids)).toBe('command-palette');
	});

	it('an empty binding never collides — several commands may have none', () => {
		const custom = { 'new-note': '', 'quick-switch': '' };
		expect(findShortcutConflict('new-tab', '', custom, ids)).toBeNull();
	});
});

describe('resolution', () => {
	it('prefers the override, falls back to the default', () => {
		expect(getResolvedShortcut('new-tab', {})).toBe('Ctrl+Shift+T');
		expect(getResolvedShortcut('new-tab', { 'new-tab': 'Ctrl+Alt+N' })).toBe('Ctrl+Alt+N');
	});

	/** "Cleared" must mean NO shortcut — not a fall-through to the default, which would make the
	 *  Clear button appear to do nothing. */
	it('an empty override means no shortcut, not the default', () => {
		expect(getResolvedShortcut('new-tab', { 'new-tab': '' })).toBe('');
	});
});

describe('capture agrees with dispatch', () => {
	/** The one that mattered: capture and dispatch must produce the SAME string for the same
	 *  keystroke, or a saved binding can never fire. */
	it('orders modifiers identically for a three-modifier combination', () => {
		const combo = eventToShortcut(press('t', { ctrlKey: true, shiftKey: true, altKey: true, code: 'KeyT' }));
		expect(combo).toBe('Ctrl+Shift+Alt+T');
		expect(normalizeShortcut(combo)).toBe(combo);
	});

	/** Cross-Platform by Design: ⌘ and Ctrl store as the SAME token, so settings synced between a
	 *  Mac and a PC need no migration. */
	it('stores Command and Control as one neutral token', () => {
		const mac = eventToShortcut(press('t', { metaKey: true, shiftKey: true, code: 'KeyT' }));
		const win = eventToShortcut(press('t', { ctrlKey: true, shiftKey: true, code: 'KeyT' }));
		expect(mac).toBe(win);
	});
});

describe('display', () => {
	it('reads as Ctrl+… off the Mac', () => {
		expect(formatShortcut('Ctrl+Shift+T', false)).toBe('Ctrl+Shift+T');
		expect(formatShortcut('Alt+ArrowLeft', false)).toBe('Alt+←');
	});

	it('reads as ⌘⇧T on the Mac', () => {
		expect(formatShortcut('Ctrl+Shift+T', true)).toBe('⌘⇧T');
		expect(formatShortcut('Alt+ArrowLeft', true)).toBe('⌥←');
	});

	it('shows nothing for no binding', () => {
		expect(formatShortcut('', true)).toBe('');
		expect(formatShortcut('', false)).toBe('');
	});
});

describe('the New Tab command', () => {
	it('has a default that collides with nothing else', () => {
		expect(DEFAULT_SHORTCUTS['new-tab']).toBe('Ctrl+Shift+T');
		const others = Object.entries(DEFAULT_SHORTCUTS).filter(([id]) => id !== 'new-tab');
		expect(others.some(([, s]) => normalizeShortcut(s) === 'Ctrl+Shift+T')).toBe(false);
	});
});

describe('reserved combinations', () => {
	/**
	 * The gate's finding: the conflict check knew the COMMAND table but not the dispatcher's own
	 * hard-coded handlers. `Ctrl+.` opens the emoji picker and returns before the command loop, so
	 * a binding to it saved, displayed as live, survived restarts — and could never once fire.
	 */
	it('refuses a combination the dispatcher answers itself', () => {
		expect(shortcutRefusal('Ctrl+.')).toBe('reserved');
		expect(RESERVED_SHORTCUTS['Ctrl+.']).toBe('emoji-icon-picker');
	});

	/** No command may ship a default that lands on a reserved combination — that would be a
	 *  binding the app advertises and then swallows. */
	it('no shipped default collides with a reserved combination', () => {
		for (const [id, combo] of Object.entries(DEFAULT_SHORTCUTS)) {
			if (KNOWN_SHADOWED_DEFAULTS.includes(combo)) continue; // recorded above, filed as PJ-295
			expect(shortcutRefusal(combo, EDITOR_KEYMAP_RESERVED), `${id} → ${combo}`).toBeNull();
		}
	});
});

describe('the binding is canonical; the display is only a rendering', () => {
	/**
	 * The root cause behind three separate gate findings: commands carried the DISPLAY string and
	 * the dispatcher matched on it, so every cosmetic substitution was a binding that could never
	 * fire. `Ctrl+ArrowUp` saved, displayed as live, survived restarts and did nothing — and the
	 * macOS branch would have killed EVERY shortcut in the app the moment that build existed.
	 *
	 * The invariant: whatever the dispatcher compares must round-trip from a real keystroke.
	 * Display formatting is applied at the render site and nowhere else.
	 */
	const roundTrips = (combo: string) => normalizeShortcut(combo) === combo;

	it('every shipped default round-trips through the dispatcher comparison', () => {
		for (const [id, combo] of Object.entries(DEFAULT_SHORTCUTS)) {
			expect(roundTrips(combo), `${id} → ${combo}`).toBe(true);
		}
	});

	/** The arrow keys are the specific trap: formatShortcut rewrites all four, normalizeShortcut
	 *  reverses only two — so a DISPLAY string is not a safe dispatch key on any platform. */
	it('a display string is NOT a dispatch key — the arrows prove it', () => {
		expect(formatShortcut('Ctrl+ArrowUp', false)).toBe('Ctrl+↑');
		expect(normalizeShortcut('Ctrl+↑')).not.toBe('Ctrl+ArrowUp'); // no inverse: would never match
		expect(roundTrips('Ctrl+ArrowUp')).toBe(true); // …but the canonical form always does
	});

	/** And on the Mac the display form shares nothing with the canonical one. */
	it('the Mac display form could never be matched against a keystroke', () => {
		expect(formatShortcut('Ctrl+Shift+T', true)).toBe('⌘⇧T');
		const fromKeystroke = eventToShortcut(press('t', { metaKey: true, shiftKey: true, code: 'KeyT' }));
		expect(fromKeystroke).toBe('Ctrl+Shift+T');
		expect(normalizeShortcut('⌘⇧T')).not.toBe(fromKeystroke);
	});
});

describe('conflicts against commands that are not registered right now', () => {
	/**
	 * The caller passes the commands REGISTERED at that moment, and some are conditional:
	 * `second-screen` (Ctrl+Shift+2) only exists once a second display is detected. On an ordinary
	 * single-monitor machine its combination read as free — so it could be given away, and the day
	 * a monitor was attached the two collided and first-match-wins left the second screen's own
	 * shortcut dead, with both rows still showing it.
	 *
	 * The earlier tests hid this by passing the COMPLETE id set; these pass what the screen really
	 * has.
	 */
	const registeredOnOneMonitor = Object.keys(DEFAULT_SHORTCUTS).filter((id) => id !== 'second-screen');

	it('still finds a command missing from the caller list', () => {
		expect(findShortcutConflict('new-note', 'Ctrl+Shift+2', {}, registeredOnOneMonitor)).toBe('second-screen');
	});

	it('and finds one that exists only as a user override', () => {
		const custom = { 'some-plugin-command': 'Ctrl+Alt+9' };
		expect(findShortcutConflict('new-note', 'Ctrl+Alt+9', custom, [])).toBe('some-plugin-command');
	});

	it('an empty caller list still protects every shipped default', () => {
		expect(findShortcutConflict('new-note', 'Ctrl+P', {}, [])).toBe('command-palette');
	});
});

describe('every command can actually be bound', () => {
	/**
	 * The Hotkeys screen offers a Record button on EVERY row, but eleven commands were built with
	 * no `shortcut` field at all — and the dispatcher matches solely on that field. So recording a
	 * key for "The Cataloger" saved it to settings, closed the chip with no error, and produced a
	 * binding that could never fire; the row then read "Not set" beside an orphan Reset button,
	 * which looks like "the app missed my keypress". Worse, the dead entry then POISONED that
	 * combination — the conflict check unions the saved overrides, so binding it to a command that
	 * would have worked was refused by name.
	 *
	 * A source assertion, because the registry is built inside a Svelte component: the failure is
	 * an omission at the point a command is DECLARED, which is exactly what a new command is most
	 * likely to repeat.
	 */
	it('no command is declared without a resolvable shortcut', async () => {
		const fs = await import('node:fs/promises');
		const src = await fs.readFile('src/routes/+layout.svelte', 'utf-8');
		// Collect every declared command id, then require a resolver call for it. Checking the ID
		// rather than the surrounding text keeps this independent of the order the properties are
		// written in — the first cut matched up to the first `name:` and so reported every command
		// that declares its shortcut afterwards, which is most of them.
		const ids = [...src.matchAll(/\{ id: '([a-z0-9-]+)', (?:shortcut|name):/g)].map((m) => m[1]);
		expect(ids.length).toBeGreaterThan(30); // the registry was found at all
		const unbindable = [...new Set(ids)].filter((id) => !src.includes(`scRaw('${id}')`));
		expect(unbindable, `declared with no resolvable binding: ${unbindable.join(', ')}`).toEqual([]);
	});
});

describe('the separator is also a key', () => {
	/**
	 * `+` is both the delimiter and something a user can press — the numpad one, and unshifted `+`
	 * on German and Nordic layouts. `'+'.split('+')` gives two empty strings, so a naive parse
	 * counted two "parts", concluded the combination was modified, and let a completely bare `+`
	 * through the refusal that exists to stop exactly that. The same split rendered it as an EMPTY
	 * row on the Mac display: a live binding shown as no binding at all.
	 */
	it('parses a bare separator as the key, with no modifiers', () => {
		expect(parseShortcut('+')).toEqual({ mods: [], key: '+' });
		expect(parseShortcut('Ctrl++')).toEqual({ mods: ['Ctrl'], key: '+' });
		expect(parseShortcut('Ctrl+Shift+Alt+T')).toEqual({ mods: ['Ctrl', 'Shift', 'Alt'], key: 'T' });
		expect(parseShortcut('T')).toEqual({ mods: [], key: 'T' });
	});

	it('refuses a bare + exactly as it refuses any other bare key', () => {
		expect(shortcutRefusal('+')).toBe('bare-key');
		expect(shortcutRefusal('Ctrl++')).toBeNull(); // modified, so fine
	});

	it('shows a + binding on both platforms instead of swallowing it', () => {
		expect(formatShortcut('Ctrl++', false)).toBe('Ctrl++');
		expect(formatShortcut('Ctrl++', true)).toBe('⌘+');
	});
});

describe("the editor's own keys are not the app's to give away", () => {
	/**
	 * The same hazard as `Ctrl+.`, one layer down. The global dispatcher is capture-phase and calls
	 * `preventDefault`; CodeMirror's `runHandlers` stops on an already-defaulted event. So a command
	 * bound to Ctrl+Z WINS, and the editor's undo never runs: the user presses undo in a note,
	 * nothing happens, no error appears, and the edit they wanted back is what the debounced save
	 * writes to disk.
	 */
	it('refuses the keys NotePane installs', () => {
		for (const combo of ['Ctrl+Z', 'Ctrl+Y', 'Ctrl+Shift+Z', 'Ctrl+X', 'Ctrl+C', 'Ctrl+V', 'Ctrl+A', 'Ctrl+F']) {
			expect(shortcutRefusal(combo), combo).toBe('reserved');
		}
	});

	/**
	 * These six ship as defaults AND sit on editor combinations — a pre-existing collision, filed
	 * as PJ-295. The earlier version of this test asserted they were "deliberately app-owned" and
	 * therefore fine, which CERTIFIED the broken state: each of their commands has an empty action,
	 * so the dispatcher consumes the key and neither the command nor the editor's binding runs.
	 * Named here so the strict rule below can stay strict about every other default.
	 */
	it('the shadowed defaults are computed and surfaced, not quietly excused', () => {
		// Every entry is, by construction, a shipped default sitting on an editor key.
		for (const { id, combo } of SHADOWED_DEFAULTS) {
			expect(combo in EDITOR_KEYMAP_RESERVED, `${id} → ${combo}`).toBe(true);
			expect(DEFAULT_SHORTCUTS[id]).toBe(combo);
		}
		expect(SHADOWED_DEFAULTS.length).toBeGreaterThan(0); // PJ-295 is real, not theoretical
	});

	/** Search is Ctrl+Shift+F precisely so it does not take find-in-note's key. */
	it('search keeps its own combination, distinct from find-in-note', () => {
		expect(DEFAULT_SHORTCUTS['search']).toBe('Ctrl+Shift+F');
		expect(shortcutRefusal('Ctrl+Shift+F')).toBeNull();
	});
});

describe('the reserved table is DERIVED from the editor, not remembered', () => {
	/**
	 * Eight gate rounds on this feature, and the last three were all the same answer: the checker
	 * did not know about a source of bindings. Reserved combinations, then conditional commands,
	 * then CodeMirror's stock keys, then Constellation's OWN editor keymaps (PJ-106's RTL motion
	 * and selection keys). Hand-listing was never going to converge — there is always another
	 * keymap.
	 *
	 * So the list stops being a memory and becomes a derivation: this walks every keymap declared
	 * under `src/lib/editor/` and fails if a MODIFIED binding is missing from RESERVED_SHORTCUTS.
	 * Add a keymap entry tomorrow and this goes red until the Hotkeys screen is taught to refuse
	 * it — which is the only version of this guard that stays true without anyone remembering.
	 *
	 * Bare editor keys (Enter, Tab, ArrowLeft…) are exempt: the dispatcher early-returns for
	 * unmodified keys inside an editable target, so it cannot steal them.
	 */
	it('reserves every modified key the editor binds', async () => {
		const fs = await import('node:fs/promises');
		const path = await import('node:path');
		const dir = 'src/lib/editor';
		const files = (await fs.readdir(dir)).filter((f) => f.endsWith('.ts'));
		const declared = new Set<string>();
		for (const f of files) {
			const src = await fs.readFile(path.join(dir, f), 'utf-8');
			for (const m of src.matchAll(/\bkey:\s*'([^']+)'/g)) declared.add(m[1]);
		}
		expect(declared.size).toBeGreaterThan(3); // the keymaps were found at all

		/** CodeMirror notation ("Shift-Mod-l") → the canonical form the app stores. */
		const toCanonical = (cm: string): string => {
			const parts = cm.split('-');
			const key = parts.pop()!;
			const mods = new Set(parts.map((p) => (p === 'Mod' || p === 'Cmd' ? 'Ctrl' : p)));
			const ordered = ['Ctrl', 'Shift', 'Alt'].filter((m) => mods.has(m));
			const k = /^[a-z]$/.test(key) ? key.toUpperCase() : key;
			return [...ordered, k].join('+');
		};

		const unreserved = [...declared]
			.map(toCanonical)
			.filter((combo) => combo.includes('+')) // modified only
			.filter((combo) => !(combo in RESERVED_SHORTCUTS));
		expect(unreserved, `editor keys the Hotkeys screen would hand out: ${unreserved.join(', ')}`).toEqual([]);
	});
});

describe('the editor reservation is genuinely derived', () => {
	/**
	 * The claim that broke on round nine. The table said it was "derived so a keymap added tomorrow
	 * cannot quietly become a combination the Hotkeys screen hands out" — but the derivation only
	 * walked `src/lib/editor/`, so it was structurally blind to the three CodeMirror keymaps
	 * NotePane installs and passed green while naming eight combinations out of ~35.
	 */
	it('covers far more than any hand-written list did', () => {
		expect(Object.keys(EDITOR_KEYMAP_RESERVED).length).toBeGreaterThan(25);
	});

	it('includes the keys a hand-list forgot', () => {
		for (const combo of ['Ctrl+Backspace', 'Ctrl+Delete', 'Ctrl+[', 'Ctrl+]', 'Alt+ArrowUp', 'Ctrl+Enter', 'Ctrl+Shift+K']) {
			expect(combo in EDITOR_KEYMAP_RESERVED, combo).toBe(true);
		}
	});

	it('and the screen refuses them', () => {
		expect(shortcutRefusal('Ctrl+Backspace', EDITOR_KEYMAP_RESERVED)).toBe('reserved');
		expect(shortcutRefusal('Ctrl+Alt+F9', EDITOR_KEYMAP_RESERVED)).toBeNull(); // still free
	});
});

describe('the derivation reads the whole binding, not one field of it', () => {
	/**
	 * Round ten. The derivation read `b.key` alone — but CodeMirror also registers `Shift-<key>`
	 * for any binding carrying a `shift:` handler, and a binding may name its combination under
	 * `mac`/`win`/`linux` instead. Nine live combinations were therefore reported FREE, and giving
	 * one away kills select-to-end-of-document in every note as surely as giving away Ctrl+Z kills
	 * undo. A derivation that reads one field of a four-field structure is a hand-list wearing a
	 * loop.
	 */
	it('includes the Shift variants of shift-capable bindings', () => {
		for (const combo of ['Ctrl+Shift+ArrowUp', 'Ctrl+Shift+ArrowDown', 'Ctrl+Shift+End', 'Ctrl+Shift+Home', 'Ctrl+Shift+ArrowLeft', 'Ctrl+Shift+ArrowRight']) {
			expect(combo in EDITOR_KEYMAP_RESERVED, combo).toBe(true);
		}
	});

	it('and the screen refuses them', () => {
		expect(shortcutRefusal('Ctrl+Shift+End', EDITOR_KEYMAP_RESERVED)).toBe('reserved');
		expect(shortcutRefusal('Ctrl+Shift+ArrowUp', EDITOR_KEYMAP_RESERVED)).toBe('reserved');
	});

	/** The project's own keys, including the shift variants their bindings declare. */
	it('covers Constellation own editor bindings too', () => {
		for (const combo of ['Ctrl+ArrowUp', 'Ctrl+ArrowDown', 'Ctrl+L', 'Alt+L', 'Ctrl+Shift+L', 'Ctrl+Shift+S']) {
			expect(combo in EDITOR_KEYMAP_RESERVED, combo).toBe(true);
		}
	});
});

describe('every keymap NotePane installs is a source', () => {
	/**
	 * Round eleven. `autocompletion()` installs `completionKeymap` (Ctrl+Space → start completion),
	 * and it was not in the source list — so Ctrl+Space read as free, and giving it away kills the
	 * manual completion trigger in every note. The fix was not to add one more source: it was to go
	 * and READ NotePane's extension list, which installs SIX keymaps, and cover all of them.
	 */
	it('reserves the autocomplete and bracket keymaps', () => {
		expect('Ctrl+Space' in EDITOR_KEYMAP_RESERVED).toBe(true);
		expect(shortcutRefusal('Ctrl+Space', EDITOR_KEYMAP_RESERVED)).toBe('reserved');
	});

	/** Shift+←/→ is ordinary text selection — the RTL arrow keymap's shift variants. */
	it('reserves shift-selection with the arrows', () => {
		expect('Shift+ArrowLeft' in EDITOR_KEYMAP_RESERVED).toBe(true);
		expect('Shift+ArrowRight' in EDITOR_KEYMAP_RESERVED).toBe(true);
	});

	/**
	 * The scan that keeps the project half honest. It reads key literals straight out of
	 * `src/lib/editor/` — including files this module does not import — so a project keymap added
	 * there fails this until it is declared in PROJECT_BINDINGS.
	 */
	it('every project editor key literal is reserved', async () => {
		const fs = await import('node:fs/promises');
		const path = await import('node:path');
		const dir = 'src/lib/editor';
		const declared = new Set<string>();
		for (const f of (await fs.readdir(dir)).filter((n) => n.endsWith('.ts'))) {
			const src = await fs.readFile(path.join(dir, f), 'utf-8');
			for (const m of src.matchAll(/\bkey:\s*'([^']+)'/g)) declared.add(m[1]);
		}
		const toCanonical = (cm: string) => {
			const parts = cm.split('-');
			const key = parts.pop()!;
			const mods = new Set(parts.map((p) => (p === 'Mod' || p === 'Cmd' ? 'Ctrl' : p)));
			const ordered = ['Ctrl', 'Shift', 'Alt'].filter((m) => mods.has(m));
			return [...ordered, /^[a-z]$/.test(key) ? key.toUpperCase() : key].join('+');
		};
		const missing = [...declared]
			.map(toCanonical)
			.filter((c) => c.includes('+'))
			.filter((c) => !(c in EDITOR_KEYMAP_RESERVED));
		expect(missing, `project editor keys not reserved: ${missing.join(', ')}`).toEqual([]);
	});
});

describe('the reservation matches the KEYSTROKE, not the keymap label', () => {
	/**
	 * Round twelve, and the sharpest finding of the set: the reservation was INVERTED for two
	 * combinations. CodeMirror suppresses Shift when looking up character keys, so its `Alt-A`
	 * binding (toggleBlockComment) is reached by physically pressing Shift+Alt+A — which is what
	 * `eventToShortcut` records. The table reserved `Alt+A`, which the editor never answers, and
	 * left `Shift+Alt+A` free, which it does.
	 */
	it('reserves the combination the user actually presses', () => {
		expect('Shift+Alt+A' in EDITOR_KEYMAP_RESERVED).toBe(true);
		expect(shortcutRefusal('Shift+Alt+A', EDITOR_KEYMAP_RESERVED)).toBe('reserved');
	});

	/**
	 * `F3` is find-next. It was dropped by a filter whose stated reason was that the dispatcher
	 * exempts bare keys inside the editor — but that exemption covers single characters and a named
	 * list, not function keys, and `shortcutRefusal` deliberately allows F1–F24.
	 */
	it('reserves bare function keys the editor binds', () => {
		expect('F3' in EDITOR_KEYMAP_RESERVED).toBe(true);
		expect('Shift+F3' in EDITOR_KEYMAP_RESERVED).toBe(true); // both halves of one binding
		expect(shortcutRefusal('F3', EDITOR_KEYMAP_RESERVED)).toBe('reserved');
	});

	/** Function keys the editor does NOT bind stay available — this is not a blanket ban. */
	it('leaves unbound function keys free', () => {
		expect(shortcutRefusal('F9', EDITOR_KEYMAP_RESERVED)).toBeNull();
	});
});
