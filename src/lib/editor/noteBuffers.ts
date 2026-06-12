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
import { invoke } from '@tauri-apps/api/core';
// §CB-2 — the frontmatter cluster lives in a sibling leaf now, so this
// module can parse/compose itself with zero store dependency.
import { parseFrontmatter, buildFullContent, type FrontmatterProperty } from '$lib/editor/frontmatter';

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

/**
 * §CB-2 — create-if-absent (or re-seed when the buffer demonstrably
 * belongs to a different path than the host's CURRENT target). Hosts
 * that never pass through openNoteTab — index preview, dashboard,
 * ad-hoc TabLike objects — get their buffer here, seeded from their own
 * content copy. Genuine stale-callback refusals are untouched: those
 * are filtered by the callers' `filePath !== tab.path` guards BEFORE
 * this runs, so a path mismatch here can only mean "the host moved its
 * slot to a new note without openNoteTab", where the host's copy IS
 * the truth.
 */
export function ensureBuffer(tabId: string, path: string, content: string): void {
	if (!tabId) return;
	const b = buffers.get(tabId);
	if (b && b.path === path) return;
	const fp = parseFrontmatter(content || '');
	setBuffer(tabId, path, fp.properties, fp.body);
}

/**
 * §CB-2 — update the BODY half only (the pane's hand-off path). Props,
 * path, cid, paneState untouched. No buffer → dev-warn and drop: a body
 * without an identity is exactly the Frankenstein ingredient this
 * migration exists to refuse.
 */
export function setBufferBody(tabId: string, body: string | Text): void {
	const b = buffers.get(tabId);
	if (!b) {
		if (import.meta.env.DEV) console.error(`[noteBuffers] setBufferBody: no buffer for tab=${tabId}`);
		return;
	}
	b.body = typeof body === 'string' ? toText(body) : body;
	b.updatedAt = Date.now();
}

/**
 * §CB-2 — update the PROPS half only (PropertyEditor / stage-promote
 * path). Body untouched; cid re-extracted (props carry identity).
 */
export function setBufferProps(tabId: string, props: FrontmatterProperty[]): void {
	const b = buffers.get(tabId);
	if (!b) {
		if (import.meta.env.DEV) console.error(`[noteBuffers] setBufferProps: no buffer for tab=${tabId}`);
		return;
	}
	b.props = cloneProps(props);
	b.cid = cidOf(b.props);
	b.updatedAt = Date.now();
}

export type ComposeResult =
	| { ok: true; content: string; body: string; path: string; cid: string | null }
	| { ok: false; reason: 'no_buffer' | 'path_mismatch'; bufferPath?: string };

/**
 * §CB-2 — THE single content source for every editor-originated save.
 * Composes {props + body} from ONE buffer object, and only after the
 * caller's intended target path matches the buffer's own identity —
 * content and target travel together (the Obsidian TFile discipline).
 * A mismatch means the callback outlived its tab slot (wikilink click,
 * Alt-nav repurposing): composing would manufacture a Frankenstein
 * write, so it is REFUSED and journaled into the same write-journal
 * stream the Rust gate uses (one forensic timeline).
 */
export function composeBuffer(tabId: string, expectPath: string, surface: string): ComposeResult {
	const b = buffers.get(tabId);
	if (!b) {
		journalRefusal(surface, expectPath, 'no_buffer');
		return { ok: false, reason: 'no_buffer' };
	}
	if (expectPath && b.path !== expectPath) {
		journalRefusal(surface, expectPath, 'path_mismatch');
		return { ok: false, reason: 'path_mismatch', bufferPath: b.path };
	}
	const body = b.body.toString();
	return { ok: true, content: buildFullContent(b.props, body), body, path: b.path, cid: b.cid };
}

function journalRefusal(surface: string, path: string, reason: string): void {
	console.error(`[noteBuffers] compose REFUSED (${reason}) surface=${surface} path=${path}`);
	invoke('journal_compose_refusal', { surface, path, reason }).catch(() => {});
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
 * canonical serializer (buildFullContent with an empty body).
 */
export function parityProbe(
	tabId: string,
	legacy: { props: FrontmatterProperty[]; body: string },
	where: string,
): void {
	if (!import.meta.env.DEV) return;
	const b = buffers.get(tabId);
	if (!b) {
		console.error(`[noteBuffers] PARITY: no buffer for tab=${tabId} at ${where} — missed creation site`);
		return;
	}
	const bodyMatch = b.body.toString() === legacy.body;
	const propsMatch = buildFullContent(b.props, '') === buildFullContent(legacy.props, '');
	if (!bodyMatch || !propsMatch) {
		console.error(
			`[noteBuffers] PARITY MISMATCH at ${where} tab=${tabId} path=${b.path} ` +
			`(body ${bodyMatch ? 'ok' : 'DRIFT'}, props ${propsMatch ? 'ok' : 'DRIFT'})`,
		);
	}
}
