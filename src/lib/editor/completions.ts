/**
 * Shared autocompletion functions for NotePane and CodeMirrorEditor.
 * Factory pattern: each function takes data as params, returns a CompletionSource.
 */
import { type CompletionContext, type Completion } from '@codemirror/autocomplete';
import { EditorView } from '@codemirror/view';
import { generateTable } from './tableUtils';

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

/** Wikilink autocomplete: type [[ → search notes. Handles trailing ]] from closeBrackets. */
export function createWikilinkCompletion(getNotes: () => NoteInfo[]) {
	return function wikilinkCompletion(context: CompletionContext) {
		const before = context.matchBefore(/\[\[[^\]]*$/);
		if (!before) return null;
		const query = before.text.slice(2).toLowerCase();
		const options: Completion[] = [];
		for (const n of getNotes()) {
			if (n.name.toLowerCase().includes(query)) {
				options.push({
					label: n.name,
					detail: n.libraryName ? ` — ${n.libraryName}` : undefined,
					type: 'text',
					apply: (v: EditorView, _c: Completion, from: number, to: number) => {
						/* Consume any trailing ]] left by closeBrackets */
						let end = to;
						const after = v.state.doc.sliceString(to, Math.min(to + 2, v.state.doc.length));
						if (after === ']]') end = to + 2;
						else if (after[0] === ']') end = to + 1;
						const insert = `[[${n.name}]]`;
						v.dispatch({ changes: { from: before.from, to: end, insert }, selection: { anchor: before.from + insert.length } });
					}
				});
				if (options.length >= 20) break;
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

/** Typed link autocomplete: [[note| → suggests semantic link types (CE Phase 1) */
export function createTypedLinkCompletion() {
	const LINK_TYPES = [
		{ label: 'supports',     detail: 'Evidence for a claim'      },
		{ label: 'contradicts',  detail: 'Tension / opposition'       },
		{ label: 'causes',       detail: 'Causal relationship'        },
		{ label: 'exemplifies',  detail: 'Instance-of'                },
		{ label: 'generalizes',  detail: 'Abstraction'                },
		{ label: 'derives-from', detail: 'Provenance / source'        },
		{ label: 'part-of',      detail: 'Compositional hierarchy'    },
		{ label: 'supersedes',   detail: 'Replaces an earlier stance' },
	];
	return function typedLinkCompletion(context: CompletionContext) {
		// Match [[note_name| with optional partial type already typed
		const before = context.matchBefore(/\[\[[^\]\|]+\|[^\]]*$/);
		if (!before) return null;
		const pipeIdx = before.text.lastIndexOf('|');
		const typed = before.text.slice(pipeIdx + 1).toLowerCase();
		const from = before.from + pipeIdx + 1;
		const options = LINK_TYPES
			.filter(t => t.label.startsWith(typed))
			.map(t => ({
				label: t.label,
				detail: t.detail,
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
					const insert = display ? `[[${t.label}::${target}|${display}]]` : `[[${t.label}::${target}]]`;
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
