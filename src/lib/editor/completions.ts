/**
 * Shared autocompletion functions for NotePane and CodeMirrorEditor.
 * Factory pattern: each function takes data as params, returns a CompletionSource.
 */
import { type CompletionContext, type Completion, startCompletion } from '@codemirror/autocomplete';
import { EditorView } from '@codemirror/view';
import { generateTable } from './tableUtils';
import { getLinkTypes, isLinkTypeValue } from '$lib/libraries/linkTypeRegistry';
import { taskDateCompletions, TASK_RE } from './taskDates';

export const SLASH_COMMANDS: { label: string; detail: string; apply: string }[] = [
	{ label: '/heading1', detail: 'H1', apply: '# ' },
	{ label: '/heading2', detail: 'H2', apply: '## ' },
	{ label: '/heading3', detail: 'H3', apply: '### ' },
	{ label: '/bullet', detail: 'Bullet list', apply: '- ' },
	{ label: '/numbered', detail: 'Numbered list', apply: '1. ' },
	{ label: '/task', detail: 'Task', apply: '- [ ] ' },
	{ label: '/code', detail: 'Code block', apply: '```\n\n```' },
	{ label: '/quote', detail: 'Blockquote', apply: '> ' },
	{ label: '/divider', detail: 'Horizontal rule', apply: '---\n' },
	{ label: '/table', detail: 'Table (or /table 3x4)', apply: '' },
	{ label: '/callout', detail: 'Callout', apply: '> [!note] Title\n> Content\n' },
	{ label: '/math', detail: 'Math block', apply: '$$\n\n$$' },
	{ label: '/mermaid', detail: 'Mermaid diagram', apply: '```mermaid\ngraph TD\n  A --> B\n```\n' },
	{ label: '/template', detail: 'Insert template', apply: '' },
];

type NoteInfo = { name: string; path: string; libraryName?: string };

/** Consume a trailing `]]` (or stray `]`) left by closeBrackets, returning the
 *  end offset the replacement should cover. */
function consumeTrailingBrackets(v: EditorView, to: number): number {
	const after = v.state.doc.sliceString(to, Math.min(to + 2, v.state.doc.length));
	if (after === ']]') return to + 2;
	if (after[0] === ']') return to + 1;
	return to;
}

/** A completion that replaces from the opening `[[` (at `fromBracket`) with
 *  `insert`, placing the cursor at `fromBracket + caret`. */
function linkReplaceOption(
	label: string, detail: string | undefined, ctype: string,
	fromBracket: number, insert: string, caret: number, boost?: number, retrigger?: boolean,
): Completion {
	return {
		label, detail, type: ctype, boost,
		apply: (v: EditorView, _c: Completion, _f: number, to: number) => {
			const end = consumeTrailingBrackets(v, to);
			v.dispatch({ changes: { from: fromBracket, to: end, insert }, selection: { anchor: fromBracket + caret } });
			if (retrigger) setTimeout(() => startCompletion(v), 0);
		},
	};
}

/**
 * Wikilink autocomplete (MIG-067 §E — type-first, canonical Type→Note order).
 *
 * Two phases off one trigger (`[[` … with no `|`):
 *  1. `[[partial`  → the link TYPES (to start a typed link, boosted to the top)
 *     PLUS the matching NOTES (for a plain untyped link). Picking a type writes
 *     `[[type::]]` and re-opens the menu on the target; picking a note writes
 *     `[[Note]]`.
 *  2. `[[type::partial` → the TARGET notes for that type. Picking one writes
 *     `[[type::Note]]`; typing a name that doesn't exist is fine (it resolves /
 *     creates on click).
 *
 * Perf: the note list is lowercased ONCE per source array (not per keystroke),
 * and the trigger excludes the post-`|` region so it never churns while the
 * legacy `[[note|type]]` menu is open.
 */
export function createWikilinkCompletion(getNotes: () => NoteInfo[]) {
	let cachedSrc: NoteInfo[] | null = null;
	let cachedLower: { n: NoteInfo; lower: string }[] = [];
	const notesLower = () => {
		const notes = getNotes();
		if (notes !== cachedSrc) {
			cachedSrc = notes;
			cachedLower = notes.map(n => ({ n, lower: n.name.toLowerCase() }));
		}
		return cachedLower;
	};

	return function wikilinkCompletion(context: CompletionContext) {
		const before = context.matchBefore(/\[\[[^\]|]*$/);
		if (!before) return null;
		const inner = before.text.slice(2);
		const ci = inner.indexOf('::');

		// Phase 2 — [[type::target : suggest the target note for a known type.
		if (ci >= 0) {
			const typeId = inner.slice(0, ci).trim().toLowerCase();
			if (!isLinkTypeValue(typeId)) return null; // a real "::" in a name, not a type
			const query = inner.slice(ci + 2).toLowerCase();
			const options: Completion[] = [];
			for (const { n, lower } of notesLower()) {
				if (lower.includes(query)) {
					const insert = `[[${typeId}::${n.name}]]`;
					options.push(linkReplaceOption(n.name, n.libraryName ? ` — ${n.libraryName}` : undefined, 'text', before.from, insert, insert.length));
					if (options.length >= 16) break;
				}
			}
			if (options.length === 0) return null;
			return { from: before.from + 2 + ci + 2, options, filter: false };
		}

		// Phase 1 — [[partial : link types first (start a typed link), then notes.
		const query = inner.toLowerCase();
		const options: Completion[] = [];
		for (const t of getLinkTypes()) {
			if (t.id.startsWith(query)) {
				const head = `[[${t.id}::`;
				options.push(linkReplaceOption(
					t.id, t.desc ? `${t.desc} — typed link` : 'typed link', 'keyword',
					before.from, head + ']]', head.length, /* boost */ 2, /* retrigger */ true,
				));
			}
		}
		for (const { n, lower } of notesLower()) {
			if (lower.includes(query)) {
				const insert = `[[${n.name}]]`;
				options.push(linkReplaceOption(n.name, n.libraryName ? ` — ${n.libraryName}` : undefined, 'text', before.from, insert, insert.length));
				if (options.length >= 16) break;
			}
		}
		return { from: before.from, options, filter: false };
	};
}

/** Tag autocomplete: type # → search tags */
export function createTagCompletion(getTags: () => string[]) {
	return function tagCompletion(context: CompletionContext) {
		const before = context.matchBefore(/#[\w\u0600-\u06FF/\-]*$/);
		if (!before) return null;
		const query = before.text.slice(1).toLowerCase();
		const options: Completion[] = [];
		for (const t of getTags()) {
			if (t.toLowerCase().includes(query)) {
				options.push({
					label: '#' + t,
					type: 'keyword',
					apply: (v: EditorView, _c: Completion, from: number, to: number) => {
						v.dispatch({ changes: { from: before.from, to, insert: '#' + t }, selection: { anchor: before.from + t.length + 1 } });
					}
				});
				if (options.length >= 20) break;
			}
		}
		return { from: before.from, options, filter: false };
	};
}

/** Typed link autocomplete: [[note| → suggests semantic link types.
 *  MIG-067 §D — the list comes from the active Link-Type Registry (the 8 seeds +
 *  any user-defined types, with their `desc` as the hint), read fresh per
 *  invocation so a vocabulary change shows up without reloading the editor. */
export function createTypedLinkCompletion() {
	return function typedLinkCompletion(context: CompletionContext) {
		// Match [[note_name| with optional partial type already typed
		const before = context.matchBefore(/\[\[[^\]\|]+\|[^\]]*$/);
		if (!before) return null;
		const pipeIdx = before.text.lastIndexOf('|');
		const typed = before.text.slice(pipeIdx + 1).toLowerCase();
		const from = before.from + pipeIdx + 1;
		const options = getLinkTypes()
			.filter(t => t.id.startsWith(typed))
			.map(t => ({
				label: t.id,
				detail: t.desc ?? undefined,
				type: 'keyword',
				apply: (v: EditorView, _c: Completion, _f: number, to: number) => {
					// Consume existing ]] if present, then re-add after type
					let end = to;
					const after = v.state.doc.sliceString(to, Math.min(to + 2, v.state.doc.length));
					if (after === ']]') end = to + 2;
					else if (after[0] === ']') end = to + 1;
					// Rebuild the link in canonical predicate-first form:
					//   [[Target|display|<type>   ->   [[type::Target|display]]
					const segs = before.text.slice(2).split('|');
					const target = segs[0];
					const display = segs.slice(1, -1).join('|');
					const insert = display ? `[[${t.id}::${target}|${display}]]` : `[[${t.id}::${target}]]`;
					v.dispatch({
						changes: { from: before.from, to: end, insert },
						selection: { anchor: before.from + insert.length }
					});
				}
			}));
		if (options.length === 0) return null;
		return { from, options, filter: false };
	};
}

/** Slash command autocomplete: type / → command palette */
export function createSlashCompletion() {
	return function slashCompletion(context: CompletionContext) {
		const line = context.state.doc.lineAt(context.pos);
		if (!line.text.trimStart().startsWith('/')) return null;
		const before = context.matchBefore(/\/\w*$/);
		if (!before) return null;
		return {
			from: before.from,
			options: SLASH_COMMANDS.map(c => ({
				...c,
				apply: (v: EditorView, _comp: Completion, from: number, to: number) => {
					if (c.label === '/template') {
						v.dispatch({ changes: { from: line.from, to } });
						window.dispatchEvent(new CustomEvent('constellation:open-template-picker'));
						return;
					}
					if (c.label === '/table') {
						const typed = line.text.trim();
						const dimMatch = typed.match(/\/table\s+(\d+)\s*[x×X]\s*(\d+)/);
						const tableStr = dimMatch
							? generateTable(Math.max(1, Math.min(parseInt(dimMatch[1]), 20)), Math.max(2, Math.min(parseInt(dimMatch[2]), 50)))
							: generateTable(2, 2);
						v.dispatch({ changes: { from: line.from, to, insert: tableStr + '\n' } });
						return;
					}
					v.dispatch({ changes: { from: line.from, to, insert: c.apply } });
				}
			})),
			filter: true
		};
	};
}

/**
 * MIG-080 §C.2 (Boss 2026-06-21, research-backed) — natural-language task due-date
 * autosuggest. On a TASK line, an `@today`/`@tomorrow`/… trigger (or a bare keyword
 * as a fallback) offers a `📅 YYYY-MM-DD` suggestion the user ACCEPTS (the Obsidian
 * nldates `@` + Tasks task-line-gate pattern; never a silent rewrite). Gated by
 * `isEnabled()` (Settings → Tasks → "Natural-language dates"). The pure resolution +
 * matching lives in taskDates.ts (unit-tested); this is the thin CM6 wrapper.
 */
export function createTaskDateCompletion(isEnabled: () => boolean) {
	return function taskDateCompletion(context: CompletionContext) {
		if (!isEnabled()) return null;
		const line = context.state.doc.lineAt(context.pos);
		if (!TASK_RE.test(line.text)) return null;
		const before = line.text.slice(0, context.pos - line.from);
		const res = taskDateCompletions(before, new Date());
		if (!res) return null;
		const options: Completion[] = res.options.map((o) => ({
			label: o.label, detail: o.detail, apply: o.label, type: 'keyword',
		}));
		// filter:false — we pre-filter by the typed @-partial; the labels are dates,
		// so CM6's own text-filter (against the date string) would hide everything.
		return { from: line.from + res.from, options, filter: false };
	};
}
