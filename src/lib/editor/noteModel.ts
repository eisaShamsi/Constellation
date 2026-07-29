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
import { sameList, sameNested, cloneRows, type SetPropOpts } from './propRow';

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
	/**
	 * PJ-070 — the exact bytes last SYNCED with disk (set at open, at adoptDisk, and on a
	 * durable save). The watcher's dirty-conflict discriminator compares an incoming external
	 * change against this: a DIRTY model + `disk !== diskBaseline` = a genuine external edit
	 * (→ `.conflict` sidecar), while `disk === diskBaseline` is a spurious fs touch or our own
	 * echo (→ no sidecar). Distinct from `base` (the frontmatter write-base) — this is the whole
	 * file. Full-string compare, off the keystroke hot path (only the 300ms watcher flush).
	 */
	diskBaseline: string;
}

const models = new Map<string, NoteModel>();

export function toText(s: string): Text {
	return Text.of(s.split('\n'));
}

/**
 * Deep-copy property rows. EXPORTED (MIG-107 Slice 6): PropertyEditor was hand-rolling this and its
 * copy missed `nestedObjects`, so the panel's seed aliased the model's nested rows. One clone that
 * tracks `FrontmatterProperty`'s shape, not several that must each remember every field.
 */
export function cloneProps(props: FrontmatterProperty[]): FrontmatterProperty[] {
	return props.map((p) => ({
		...p,
		listItems: p.listItems ? [...p.listItems] : undefined,
		nestedObjects: p.nestedObjects ? p.nestedObjects.map((o) => ({ ...o })) : undefined,
	}));
}

// PJ-182 — row equality now lives in `propRow.ts`, a leaf module `yamlDoc` can import
// without a runtime cycle. Re-exported here so every existing consumer keeps one import.
// (It used to live here as "the lowest layer that needs it" — then the COMPOSER needed it,
// and the composer is lower still. Its absence there is what dropped an ikhtilāf write.)
export { sameList, sameNested, samePropRow, cloneRows } from './propRow';
export type { SetPropOpts } from './propRow';

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
	// APP-KILLER #2 (LL-023 tripwire, dev-only) — replacing a still-DIRTY model onto a
	// DIFFERENT path silently discards the outgoing note's unsaved edits (the nav-loss class).
	// Every departure site (openNoteTab reuse / loadTabHistoryEntry / closeTab / renameItem)
	// flushes first via flushIfDirty; a same-path re-seed (reloadTabsFromDisk adopt) is fine.
	// This catches a FUTURE unguarded replace site in dev before it reintroduces the loss.
	if (import.meta.env.DEV) {
		const prev = models.get(id);
		if (prev && prev.path !== path && isDirty(id)) {
			console.warn(
				`[noteModel] openModel is replacing a DIRTY model onto a different path with no flush: ` +
				`${prev.path} → ${path} (id ${id}). Flush the outgoing model first (flushIfDirty) — APP-KILLER #2.`,
			);
		}
	}
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
		diskBaseline: content, // PJ-070 — what's on disk at open
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
	if (next === m.body) return; // O(1) ref check — the per-keystroke Text path (onDocChange), Rule 1.
	// PJ-070 — the STRING form of setBody is ONLY the cold flush/teardown pushes (handleFlush,
	// FocusPane onflush, saveTabContent), never the per-keystroke Text path above. A no-op push there
	// (identical content — e.g. a merely-VIEWED note's teardown flush re-pushing the unchanged body)
	// must NOT bump version: a spuriously-dirty clean model makes adoptDisk refuse the next external
	// edit (reintroducing the PJ-070 clobber on background/focus notes + raising phantom `.conflict`
	// sidecars) and makes flushAllDirtyTabs re-write untouched notes on a universe switch. The content
	// compare is O(N) but runs ONLY on these cold string paths, so the keystroke hot path stays O(1).
	if (typeof body === 'string' && next.eq(m.body)) return;
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

// ─── MIG-107 Slice 2: the PROPERTY INTENTS ──────────────────────────────────────────────────────
//
// **Why these exist.** `setProps` replaces the WHOLE array. That is correct for a caller that means
// "replace everything" (there are exactly two — `replaceContent` and `adoptDisk`, both of which also
// re-base), and it is the defect for a caller that means "change one property", because the array it
// submits was assembled from a projection of the file as it looked when the note was opened. Two such
// callers cancel each other out: MIG-107 §3.5.1, reproduced in `tests/pj-174/propsOwnership.test.ts`.
//
// An intent names the ONE property it is about and applies to whatever the model currently holds, so
// adding a tag cannot touch `stage` — the loss becomes unrepresentable rather than guarded against.
//
// **Keyed by KEY, and that was measured, not assumed** (MIG-107 §5.4, `propsContract.test.ts`):
// `composeFrontmatter` addresses frontmatter by key via `Map`s, so the persisted form structurally
// cannot hold duplicates and position is not identity. A row id would invent an identity the file
// format cannot store.
//
// **These do NOT announce.** This module stays deliberately non-reactive (see the header, and the
// §C-2 lesson: a store update inside a `{#key}` teardown re-enters the render the store drives). The
// reactive signal that lets panels observe changes lives OUTSIDE, in `propsSignal.ts`, ticked by the
// `noteSession` wrappers — so the choice of which call sites announce stays explicit and reviewable.
//
// Every intent: identity-guarded like `setProps`, a no-op when the model is absent, and it bumps
// `version` (→ dirty) ONLY when it actually changed something — a no-op edit must not mark the note
// unsaved. Each returns whether it mutated, so callers can skip pointless saves.

/** Set (or update) ONE property's value. Returns false when the key is absent or unchanged. */
export function setPropValue(
	id: string,
	key: string,
	value: string,
	opts?: SetPropOpts,
	expectPath?: string,
): boolean {
	const m = models.get(id);
	if (!m) return false;
	if (expectPath !== undefined && m.path !== expectPath) return false;
	const i = m.props.findIndex((p) => p.key === key);
	if (i === -1) return false;
	const cur = m.props[i];
	const nextType = opts?.type ?? cur.type;
	const nextItems = opts?.listItems;
	const nextNested = opts?.nestedObjects;
	// `undefined` means "not supplying list items" — the scalar case, and by far the common one.
	// Comparing `cur.listItems` to itself was guaranteed-equal work on every value edit.
	const sameItems = nextItems === undefined || sameList(cur.listItems, nextItems);
	const sameRows = nextNested === undefined || sameNested(cur.nestedObjects, nextNested);
	if (cur.value === value && cur.type === nextType && sameItems && sameRows) return false; // no-op: stay clean
	const next = m.props.slice(); // entries are REPLACED, never mutated in place — no deep clone needed
	next[i] = {
		...cur,
		value,
		type: nextType,
		...(nextItems ? { listItems: [...nextItems] } : {}),
		...(nextNested ? { nestedObjects: cloneRows(nextNested) } : {}),
	};
	m.props = next;
	m.cid = cidOf(m.props);
	m.version++;
	return true;
}

/**
 * Add a NEW property. Refuses an empty key and refuses to overwrite an existing one — a collision is
 * the caller's to resolve, never a silent last-wins (MIG-107 §5.4). Refusing the empty key is also
 * what closes PJ-178: a half-typed panel row can no longer reach the file as a literal `"": ""`.
 */
export function addProp(id: string, prop: FrontmatterProperty, expectPath?: string): boolean {
	const m = models.get(id);
	if (!m) return false;
	if (expectPath !== undefined && m.path !== expectPath) return false;
	if (!prop.key || !prop.key.trim()) return false;
	if (m.props.some((p) => p.key === prop.key)) return false;
	m.props = [...m.props, ...cloneProps([prop])]; // deep-clone only the INCOMING row (caller may alias it)
	m.cid = cidOf(m.props);
	m.version++;
	return true;
}

/** Remove ONE property. Returns false when the key was not there. */
export function removeProp(id: string, key: string, expectPath?: string): boolean {
	const m = models.get(id);
	if (!m) return false;
	if (expectPath !== undefined && m.path !== expectPath) return false;
	if (!m.props.some((p) => p.key === key)) return false;
	m.props = m.props.filter((p) => p.key !== key);
	m.cid = cidOf(m.props);
	m.version++;
	return true;
}

/**
 * Rename a property's KEY in place (compose turns this into a remove + an add — §5.4).
 * **Refuses a collision** rather than silently overwriting the other property: per the Boss-approved
 * ruling, a rename onto an existing key is reported to the user, not resolved by last-wins.
 */
export function renamePropKey(id: string, oldKey: string, newKey: string, expectPath?: string): boolean {
	const m = models.get(id);
	if (!m) return false;
	if (expectPath !== undefined && m.path !== expectPath) return false;
	if (!newKey || !newKey.trim() || oldKey === newKey) return false;
	const i = m.props.findIndex((p) => p.key === oldKey);
	if (i === -1) return false;
	if (m.props.some((p) => p.key === newKey)) return false; // collision — caller surfaces it
	const next = m.props.slice();
	next[i] = { ...next[i], key: newKey };
	m.props = next;
	m.cid = cidOf(m.props);
	m.version++;
	return true;
}

/**
 * Move `key` to the position currently held by `beforeKey` (or to the end when `beforeKey` is null).
 *
 * Order does NOT survive to disk for existing keys — compose rewrites values in place and only
 * APPENDS genuinely new ones (`propsContract.test.ts` pins that shuffling alone is a byte-identical
 * write). It is preserved here because it is what the panel displays, and because a future
 * order-preserving serializer should not need the panel rewritten to support it.
 */
export function reorderProps(id: string, key: string, beforeKey: string | null, expectPath?: string): boolean {
	const m = models.get(id);
	if (!m) return false;
	if (expectPath !== undefined && m.path !== expectPath) return false;
	const from = m.props.findIndex((p) => p.key === key);
	if (from === -1) return false;
	// Decide BEFORE allocating. `plan` used to emit an order op per adjacent pair, and each one
	// cloned the whole array only to discover it changed nothing — O(N^2) spreads per commit for a
	// result that was thrown away every time. Establish the move is real first, then copy once.
	const to = beforeKey === null
		? m.props.length - 1
		: m.props.findIndex((p) => p.key === beforeKey) - (m.props.findIndex((p) => p.key === beforeKey) > from ? 1 : 0);
	if (beforeKey !== null && m.props.findIndex((p) => p.key === beforeKey) === -1) return false; // unknown anchor
	if (to === from) return false; // already where it belongs — no allocation, no version bump
	const next = m.props.slice();
	const [moved] = next.splice(from, 1);
	next.splice(to, 0, moved);
	m.props = next;
	m.version++;
	return true;
}

/**
 * PJ-088 — replace the model's ENTIRE content (frontmatter + body) from an authored/merged source,
 * RE-BASING so compose emits it byte-consistently. Distinct from setProps+setBody: those leave the
 * G4 write-base (`m.base`) at its open-time bytes, so compose would diff the stale base against the
 * merged props — a violation of the UNIFICATION invariant (old/new props must project the SAME
 * source) that silently DROPS non-projectable frontmatter (nested maps / block scalars) the merge
 * changed. Re-basing to the merged content means compose applies a zero diff and emits the merged
 * frontmatter verbatim. Marks dirty (version++), path-guarded — flows through the durability gate
 * and stays dirty-until-durable on a failed save.
 */
export function replaceContent(id: string, content: string, expectPath?: string): void {
	const m = models.get(id);
	if (!m) return;
	if (expectPath !== undefined && m.path !== expectPath) return;
	const { properties, body } = parseFrontmatter(content);
	m.props = cloneProps(properties);
	m.cid = cidOf(m.props);
	m.body = toText(body);
	m.base = baseOf(content, properties); // re-base to the merged source → compose emits it verbatim (no stale-base diff)
	m.version++;
}

/** Identity update for rename/move — content untouched. */
export function setPath(id: string, path: string): void {
	const m = models.get(id);
	if (m) m.path = path;
}

/**
 * PJ-102b — mark a model whose content was seeded from the write-ahead NET (a
 * crash / failed-save recovery) as what it truthfully is: DIRTY (the recovered
 * delta IS unsaved work — the autosave/retry must persist it, the save-health
 * banner must stay red until a durable write, a departure must flush it) with
 * `diskBaseline` set to the ACTUAL on-disk bytes (so the freshness arbiter
 * sees a phantom event as a phantom and a genuine external edit as a genuine
 * conflict). Without this the model was born "clean" on content disk never had
 * — a lie every downstream arbiter then acted on (the Boss-hit clobber).
 */
export function markRecoveredFromNet(id: string, trueDiskContent: string | null): void {
	const m = models.get(id);
	if (!m) return;
	// Only adopt a REAL baseline — never fabricate one (a '' sentinel would make the
	// dirty-branch discriminator see EVERY real disk as a "genuine change" and raise a
	// spurious .conflict sidecar on the first phantom event after a disk-unreachable
	// open). With the baseline left at the recovered bytes, the dirty guard alone
	// still prevents every clobber; a sidecar in that unverifiable corner is honest.
	if (trueDiskContent !== null) m.diskBaseline = trueDiskContent;
	m.version++; // dirty: version now exceeds savedVersion
}

/**
 * PJ-102b (the restore half) — set the model's diskBaseline to the ACTUAL on-disk
 * bytes WITHOUT dirtying it. The MIG-100 session restore seeds a wab-recovered tab
 * born-clean by design (Gate #8: a restore performs zero write-class IPCs) — but
 * seeding it with baseline = the recovered bytes was a lie the phantom-guard can't
 * see through: a phantom watcher event then "adopted" stale disk and clearWriteAhead
 * DESTROYED the preserved net. With the TRUE baseline, a phantom event (disk ===
 * baseline) is refused and the net survives; only a genuinely-changed disk adopts.
 */
export function setDiskBaseline(id: string, trueDiskContent: string): void {
	const m = models.get(id);
	if (m) m.diskBaseline = trueDiskContent;
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
export function adoptDisk(id: string, diskContent: string, expectPath?: string): boolean {
	const m = models.get(id);
	if (!m) return false;
	// PJ-070 / 2026-07-21 inspection — the IDENTITY guard, matching `markSaved`.
	// adoptDisk was the only model mutator without one, so a caller holding a stale
	// (path, id) pairing — one captured before an await, with an in-place navigation
	// landing in the gap — would write one note's disk content into another note's
	// model. Every other write path already proves identity before mutating; this one
	// now does too, so the protection does not depend on each caller being careful.
	if (expectPath !== undefined && m.path !== expectPath) return false;
	if (isDirty(id)) return false;
	if (composeModel(m) === diskContent) return false; // our own echo
	// PJ-102b — the PHANTOM-EVENT guard: a clean model whose baseline already equals
	// the incoming disk means NOTHING actually changed on disk — the watcher event is
	// a phantom (AV/indexer touch, a suppressed-echo leak). Without this, a model
	// seeded from RECOVERED content (net-restore: content ≠ disk by design, baseline
	// = the true stale disk) would "adopt" that stale disk on the first phantom event
	// — silently reverting the recovery. A genuine external edit still adopts (its
	// disk differs from the baseline).
	if (diskContent === m.diskBaseline) return false;
	const { properties, body } = parseFrontmatter(diskContent);
	m.props = cloneProps(properties);
	m.cid = cidOf(m.props);
	m.body = toText(body);
	m.base = baseOf(diskContent, properties);
	m.diskBaseline = diskContent; // PJ-070 — disk is now the synced baseline
	m.version++;
	m.savedVersion = m.version; // disk IS the saved state
	return true;
}

/**
 * PJ-070 — re-baseline after a DURABLE save. Called by noteSession.save's success branch with
 * the exact bytes just written (= what `read_note` will now return), so the model knows the
 * current on-disk truth. Path-guarded exactly like markSaved (a save that resolves after an
 * id-swap must not stamp its old content onto the new model's baseline).
 */
export function noteDiskSynced(id: string, content: string, expectPath?: string): void {
	const m = models.get(id);
	if (!m) return;
	if (expectPath !== undefined && m.path !== expectPath) return;
	m.diskBaseline = content;
}

/**
 * PJ-070 — does an incoming external disk change GENUINELY differ from what this model last
 * synced with disk? The watcher's dirty-conflict arbiter: only a true difference (not a spurious
 * fs touch, not our own echo) on a DIRTY model warrants the `.conflict` sidecar. Returns false
 * when no model exists (nothing to conflict with).
 */
export function diskDiffersFromBaseline(id: string, disk: string): boolean {
	const m = models.get(id);
	if (!m) return false;
	return disk !== m.diskBaseline;
}
