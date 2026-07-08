/**
 * noteModel — MIG-076 §C (rebuilt 2026-06-12, Boss order "solve it for good"):
 * SINGLE CONTENT OWNERSHIP — the one authority for an open note's content.
 *
 * THE source of truth, not a mirror. It replaces the three drifting copies
 * whose mismatch is the entire content-integrity disease (BUG-012/015/019/023,
 * F2, the §C series): the editor's live text, the stale `tab.content` copy,
 * and the separate write-ahead buffer. With one owner, there is no second
 * copy to seed a view from, and content is never assembled from drifting
 * parts at a fragile lifecycle moment.
 *
 * Industry pattern (SME-audit-confirmed): VS Code `TextFileEditorModel`
 * (one model per resource, views are disposable), Emacs/Vim buffer-vs-window,
 * Obsidian's TFile-keyed data, CM6's state-per-buffer guidance. Saves read
 * the MODEL, never the view — so view teardown carries no content.
 *
 * DELIBERATELY NON-REACTIVE (a plain module Map). Writing it from any
 * lifecycle moment is inert — it announces nothing — which is exactly the
 * §C-2 lesson (a store update inside a `{#key}` teardown re-enters the render
 * the store drives). `openTabs` keeps METADATA only once integration lands.
 *
 * This module is built and proven in ISOLATION first (tests/mig-076/
 * noteModel.test.ts). Integration into the live components — the step that
 * failed as §CB — comes later behind a runtime view-vs-disk harness + a Boss
 * test, per the Editor-Surface Gate Checklist (CLAUDE.md).
 *
 * Note: parseFrontmatter/buildFullContent are imported from the store today;
 * at integration they move to a leaf module to keep this a dependency leaf
 * (no store→noteModel cycle). Nothing live imports this module yet.
 *
 * Invariants (each from a named 2026-06 failure; proven in the test file):
 *   I1 always-current  — a view's change is readable immediately; no stale
 *                        copy exists to seed an empty body from (symptom 1).
 *   I2 freshness       — unsaved edits are never clobbered; a clean model
 *                        adopts only genuinely-new external content, never an
 *                        echo or a stale snapshot (symptom 2).
 *   I3 identity-bound  — a save composes THIS note's content for THIS note's
 *                        path, or REFUSES (the in-focus-switch cross-note write).
 *   I4 single source   — content = buildFullContent(model.props, body): one
 *                        place, deterministic, no external re-fetch.
 *   I5 independence     — models never share content; touching B can't change A.
 */
import { Text } from '@codemirror/state';
import { parseFrontmatter, buildFullContent, type FrontmatterProperty } from '$lib/libraries/store';
import { splitFrontmatter, composeFrontmatter } from '$lib/editor/yamlDoc';

/**
 * G4 Phase 2 — the model's WRITE goes through the round-trip-safe yamlDoc CST
 * (byte-perfect: only edited keys change; nested maps, block scalars, quoted
 * values, comments in untouched keys are preserved verbatim) instead of the
 * hand-rolled buildFullContent that drops/corrupts them. THE UNIFICATION: the
 * model diffs base→current using the SAME `parseFrontmatter` projection the
 * PropertyEditor uses, so an unedited key projects identically on both sides →
 * no diff → its real value is preserved untouched (consistency, not losslessness,
 * is what makes it safe). One const toggle → instant revert if a Boss test fails.
 */
const USE_YAML_DOC = true;

export interface NoteModel {
	/** Open-note session id (today this equals the tab id). */
	readonly id: string;
	/** Disk target — the note's identity. Updated only by setPath (rename/move). */
	path: string;
	/** cid_cn extracted from props — the durable identity (path can change). */
	cid: string | null;
	/** Frontmatter half. Snapshot-cloned on every set — never aliases a $state proxy. */
	props: FrontmatterProperty[];
	/** Body half as CM6's immutable rope — O(1) assignment, toString() at save only. */
	body: Text;
	/** Monotonic content counter — bumped on every real change; the freshness arbiter. */
	version: number;
	/** The version last composed for a successful disk save — drives isDirty. */
	savedVersion: number;
	/**
	 * G4 — the byte-perfect write base: the ORIGINAL frontmatter bytes (`rawYaml`)
	 * + the `parseFrontmatter` projection of them (`props`). compose diffs
	 * base.props → props and applies to a fresh re-parse of base.rawYaml, so an
	 * unedited key stays byte-perfect. Same projection as `props`/the PropertyEditor.
	 * Null on the legacy path (USE_YAML_DOC off).
	 */
	base: { rawYaml: string; hadFence: boolean; props: FrontmatterProperty[] } | null;
}

const models = new Map<string, NoteModel>();

export function toText(s: string): Text {
	return Text.of(s.split('\n'));
}

function cloneProps(props: FrontmatterProperty[]): FrontmatterProperty[] {
	return props.map((p) => ({
		...p,
		listItems: p.listItems ? [...p.listItems] : undefined,
		nestedObjects: p.nestedObjects ? p.nestedObjects.map((o) => ({ ...o })) : undefined,
	}));
}

function cidOf(props: FrontmatterProperty[]): string | null {
	const p = props.find((x) => x.key.toLowerCase() === 'cid_cn');
	return p?.value ? String(p.value) : null;
}

/** G4 — capture the byte-perfect write base from a note's disk content (or null on legacy). */
function baseOf(content: string, props: FrontmatterProperty[]): NoteModel['base'] {
	if (!USE_YAML_DOC) return null;
	const { yaml, hadFence } = splitFrontmatter(content);
	return { rawYaml: yaml, hadFence, props: cloneProps(props) };
}

/** G4 — the ONE serialize (I4). Byte-perfect diff-apply via yamlDoc when the base is
 *  present, else the legacy rebuild. Every reader/writer of the model composes here. */
function composeModel(m: NoteModel): string {
	if (USE_YAML_DOC && m.base) {
		return composeFrontmatter(m.base.rawYaml, m.base.hadFence, m.base.props, m.props, m.body.toString());
	}
	return buildFullContent(m.props, m.body.toString());
}

/** Open (create) the model for a note from its on-disk content. */
export function openModel(id: string, path: string, content: string): NoteModel {
	const { properties, body } = parseFrontmatter(content);
	const m: NoteModel = {
		id,
		path,
		cid: cidOf(properties),
		props: cloneProps(properties),
		body: toText(body),
		version: 0,
		savedVersion: 0,
		base: baseOf(content, properties),
	};
	models.set(id, m);
	return m;
}

export function getModel(id: string): NoteModel | undefined {
	return models.get(id);
}
export function hasModel(id: string): boolean {
	return models.has(id);
}
export function closeModel(id: string): void {
	models.delete(id);
}
export function clearAllModels(): void {
	models.clear();
}
export function modelCount(): number {
	return models.size;
}

/**
 * Editor → model: replace the body (I1). Designed for the keystroke hot path:
 * accepts the CM6 `Text` rope directly (O(1) ref assignment, never toString()).
 * The guard is a pure reference check — CM6 produces a NEW doc object on every
 * real change, so an unchanged push (same ref) no-ops, and we never pay an
 * O(N) content comparison per keystroke (CLAUDE.md Rule 1).
 */
export function setBody(id: string, body: string | Text, expectPath?: string): void {
	const m = models.get(id);
	if (!m) return;
	// Identity guard on the WRITE-IN side (mirror of compose's read-out guard):
	// a caller addressing a DIFFERENT path than the model now holds is stale
	// (e.g. a torn-down editor's last flush after its tab was repurposed) and
	// must NOT poison the repurposed model. This closes the new-note-while-open
	// identity leak the 2026-06-12 Boss test surfaced.
	if (expectPath !== undefined && m.path !== expectPath) return;
	const next = typeof body === 'string' ? toText(body) : body;
	if (next === m.body) return;
	m.body = next;
	m.version++;
}

/** Props editor → model: replace props, re-extract identity (I1). */
export function setProps(id: string, props: FrontmatterProperty[], expectPath?: string): void {
	const m = models.get(id);
	if (!m) return;
	// Identity guard (see setBody): a stale PropertyEditor teardown save for the
	// PREVIOUS note must not write its props into a model whose tab has already
	// been repurposed to a new note — the exact poison behind the new-note leak.
	if (expectPath !== undefined && m.path !== expectPath) return;
	m.props = cloneProps(props);
	m.cid = cidOf(m.props);
	m.version++;
}

/** Identity update for rename/move — content untouched. */
export function setPath(id: string, path: string): void {
	const m = models.get(id);
	if (m) m.path = path;
}

export type ComposeResult =
	| { ok: true; content: string; path: string; cid: string | null; version: number }
	| { ok: false; reason: 'no_model' | 'path_mismatch'; modelPath?: string };

/**
 * The ONE composition (I3 + I4). Composes from the model's own fields, only
 * when the caller's intended target path matches the model's identity. A
 * mismatch means the callback outlived its tab slot (wikilink click, Alt-nav,
 * in-focus switch) — composing would manufacture a cross-note write, so it is
 * REFUSED. The caller journals the refusal (the gate's one timeline).
 */
export function compose(id: string, expectPath: string): ComposeResult {
	const m = models.get(id);
	if (!m) return { ok: false, reason: 'no_model' };
	if (expectPath && m.path !== expectPath) return { ok: false, reason: 'path_mismatch', modelPath: m.path };
	return {
		ok: true,
		content: composeModel(m),
		path: m.path,
		cid: m.cid,
		version: m.version,
	};
}

/**
 * Record that `version` was persisted to disk (clears dirty up to that point).
 * `expectPath` (APP-KILLER #2, 2026-07-08) path-guards the mark-clean exactly as
 * compose/setBody guard the read-out / write-in: a save that RESOLVES after its id-slot
 * was re-seeded to a DIFFERENT note (nav / reuse) must not stamp its old version onto the
 * new model — that would poison savedVersion and hide the new note's first edits from
 * autosave (a silent loss). A path mismatch is a no-op; omitting expectPath keeps the
 * legacy unguarded behavior for callers that never swap under an id.
 */
export function markSaved(id: string, version: number, expectPath?: string): void {
	const m = models.get(id);
	if (!m) return;
	if (expectPath !== undefined && m.path !== expectPath) return;
	if (version > m.savedVersion) m.savedVersion = version;
}

/** Unsaved edits exist beyond the last persisted version. */
export function isDirty(id: string): boolean {
	const m = models.get(id);
	return m ? m.version > m.savedVersion : false;
}

/**
 * External-change reconciliation (I2 freshness). Adopt disk content ONLY when:
 *   - the model has NO unsaved edits (a dirty model always wins — local edits
 *     are never silently clobbered; a true conflict is §E's dialog), AND
 *   - the disk content actually DIFFERS from what this model would write (an
 *     identical payload is our own write echoing back through the watcher —
 *     ignore it).
 * A clean model + different disk = a genuine external edit (second screen,
 * another app) → adopt. There is no "stale older snapshot" path because a
 * clean model already equals the last disk write; this is what structurally
 * removes symptom 2 rather than guarding against it.
 */
export function adoptDisk(id: string, diskContent: string): boolean {
	const m = models.get(id);
	if (!m) return false;
	if (isDirty(id)) return false;
	if (composeModel(m) === diskContent) return false; // our own echo
	const { properties, body } = parseFrontmatter(diskContent);
	m.props = cloneProps(properties);
	m.cid = cidOf(m.props);
	m.body = toText(body);
	m.base = baseOf(diskContent, properties);
	m.version++;
	m.savedVersion = m.version; // disk IS the saved state
	return true;
}
