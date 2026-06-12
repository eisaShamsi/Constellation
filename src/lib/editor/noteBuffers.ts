/**
 * noteBuffers — MIG-076 §CB: the document-model layer (the Buffer Pattern).
 *
 * Every mature editor separates the BUFFER (owns content, one per open file)
 * from the VIEW (a disposable viewport owning nothing): Emacs/Vim buffers vs
 * windows, VS Code's TextFileEditorModel vs stateless tabs, CM6's own
 * state-per-buffer guidance (discuss.codemirror.net t/2946), Obsidian's
 * TextFileView.data keyed to an identity-stable TFile. Saves read the model,
 * never the view — so view teardown carries no content, and the teardown
 * hand-off that produced BUG-012/015/F2/BUG-023 and both §C regressions
 * cannot exist. Full mapping: lab/reports/MIG-076-WRITE-INTEGRITY-ARCHITECT.md §7.
 *
 * THIS MODULE IS DELIBERATELY NON-REACTIVE. Content lives in a plain
 * module-level Map, outside Svelte's store graph: a Map.set announces
 * nothing, so writing it from ANY lifecycle moment — including pane
 * teardown — is inert by construction (the §C-2 lesson: any store
 * announcement inside the `{#key}` teardown re-enters the render that
 * the store itself drives). `openTabs` keeps METADATA ONLY.
 *
 * It is also a dependency LEAF: only @codemirror/state (the Text rope) and
 * type-only imports. parseFrontmatter/buildFullContent stay at the call
 * sites until §CB-2 moves composition here behind an extraction.
 *
 * §CB-1 (this step): buffers are a write-through MIRROR of every legacy
 * tab.content writer — no reader consumes them yet. §CB-2 makes saves
 * compose from here; §CB-3 makes panes mount from here and deletes the
 * teardown flush; §CB-4 retires tab.content.
 */
import { Text } from '@codemirror/state';
import type { EditorState } from '@codemirror/state';
import type { FrontmatterProperty } from '$lib/libraries/store';

export interface NoteBuffer {
	tabId: string;
	/** Disk target. Identity travels WITH content (the Obsidian TFile
	 *  discipline) — a save composed from this buffer can verify its
	 *  callback's path against this field instead of trusting a tab
	 *  record that may have been repurposed. */
	path: string;
	/** cid_cn extracted from props — the note's real identity (tab.id is a slot). */
	cid: string | null;
	/** Structured frontmatter half. Snapshot-cloned on set — never an
	 *  alias of a live $state proxy. */
	props: FrontmatterProperty[];
	/** Body half as CM6's immutable rope — assignment is O(1) ref swap,
	 *  toString() deferred to the write boundary. */
	body: Text;
	/** §CB-3 — EditorState captured at switch-time so undo history
	 *  survives tab switches. Unused in §CB-1/2. */
	paneState?: EditorState;
	updatedAt: number;
}

const buffers = new Map<string, NoteBuffer>();

/** Snapshot-clone props so the buffer never aliases a live reactive array
 *  (PropertyEditor's editableProps is a $state proxy). Props are small —
 *  this is nanoseconds, and it buys true snapshot semantics. */
function cloneProps(props: FrontmatterProperty[]): FrontmatterProperty[] {
	return props.map(p => ({
		...p,
		listItems: p.listItems ? [...p.listItems] : undefined,
		nestedObjects: p.nestedObjects ? p.nestedObjects.map(o => ({ ...o })) : undefined,
	}));
}

function cidOf(props: FrontmatterProperty[]): string | null {
	const p = props.find(p => p.key.toLowerCase() === 'cid_cn');
	return p?.value ? String(p.value) : null;
}

export function toText(s: string): Text {
	return Text.of(s.split('\n'));
}

/**
 * Create or replace a tab's buffer — the single entry point for every
 * writer site. Callers pass the SAME parsed pieces they compose the
 * legacy tab.content string from, so buffer and string cannot diverge
 * at the moment of writing.
 */
export function setBuffer(
	tabId: string,
	path: string,
	props: FrontmatterProperty[],
	body: string | Text,
): void {
	if (!tabId) return;
	const snapshot = cloneProps(props);
	buffers.set(tabId, {
		tabId,
		path,
		cid: cidOf(snapshot),
		props: snapshot,
		body: typeof body === 'string' ? toText(body) : body,
		// Preserve a §CB-3 paneState across content mirrors of the same tab
		paneState: buffers.get(tabId)?.paneState,
		updatedAt: Date.now(),
	});
}

/** Path-only identity update (folder rename, move) — content untouched. */
export function updateBufferPath(tabId: string, newPath: string): void {
	const b = buffers.get(tabId);
	if (b) {
		b.path = newPath;
		b.updatedAt = Date.now();
	}
}

export function getBuffer(tabId: string): NoteBuffer | undefined {
	return buffers.get(tabId);
}

export function deleteBuffer(tabId: string): void {
	buffers.delete(tabId);
}

/** Wholesale reset — universe switch / second-screen tab clear. */
export function clearAllBuffers(): void {
	buffers.clear();
}

export function bufferCount(): number {
	return buffers.size;
}

/**
 * §CB-1 DEV-ONLY drift probe, called right after each legacy-write +
 * mirror pair: if the buffer disagrees with what the legacy path just
 * recorded, a writer updated one side without the other — a missed
 * mirror site. Logs, never throws, never runs in release builds.
 *
 * Compares PARSED pieces, not raw strings: parse→serialize is not
 * byte-identical for legacy YAML (quote stripping, date normalization),
 * so the body is compared exactly while props are compared through the
 * caller's canonical serializer (buildFullContent with an empty body) —
 * passed in so this module stays a dependency leaf until §CB-2.
 */
export function parityProbe(
	tabId: string,
	legacy: { props: FrontmatterProperty[]; body: string },
	serialize: (props: FrontmatterProperty[], body: string) => string,
	where: string,
): void {
	if (!import.meta.env.DEV) return;
	const b = buffers.get(tabId);
	if (!b) {
		console.error(`[noteBuffers] PARITY: no buffer for tab=${tabId} at ${where} — missed creation site`);
		return;
	}
	const bodyMatch = b.body.toString() === legacy.body;
	const propsMatch = serialize(b.props, '') === serialize(legacy.props, '');
	if (!bodyMatch || !propsMatch) {
		console.error(
			`[noteBuffers] PARITY MISMATCH at ${where} tab=${tabId} path=${b.path} ` +
			`(body ${bodyMatch ? 'ok' : 'DRIFT'}, props ${propsMatch ? 'ok' : 'DRIFT'})`,
		);
	}
}
