/**
 * G4 — the round-trip-safe frontmatter authority (Solve-the-Class: ONE parser +
 * serializer every reader/writer goes through, replacing the hand-rolled
 * `parseFrontmatter`/`reconstructFrontmatter` that silently drop nested maps +
 * block scalars and accumulate backslashes on quoted values).
 *
 * Built on eemeli `yaml` (the JS analog of Python's ruamel.yaml — the field's
 * battle-tested "edit one field, keep the rest" library; WA#5). Boss chose the
 * BYTE-PERFECT tier, so we drive the CST (Concrete Syntax Tree) directly:
 *   - pure round-trip           → `CST.stringify(parse(x)) === x` (byte-exact)
 *   - edit an existing scalar    → `CST.setScalarValue` (untouched keys byte-perfect)
 *   - add a scalar               → append a library-serialized `key: value` line
 *                                  (correct quoting/escaping; no accumulation)
 *   - remove a key               → splice its CST map item
 * Values are READ via `parseDocument` (easy `.get()`); the CST is the WRITE
 * authority. Runs on the save path only (Rule 6 — not the keystroke hot path).
 *
 * H1 (adversarial) — malformed YAML must NEVER lose content: `parseDocument`
 * never throws, but `CST.stringify` of a broken doc can misbehave, so on
 * `doc.errors.length > 0` we PASS THROUGH the original frontmatter bytes verbatim
 * and skip structural edits (the user's single change is applied only if it maps
 * to a safe scalar set; otherwise preserved-as-is + reported).
 *
 * This module is dark until the noteModel swap (G4 Phase 2, behind `useYamlDoc`).
 */
import { Parser, CST, parseDocument, stringify as yamlStringify, isSeq, isMap, isScalar, Scalar, YAMLSeq } from 'yaml';
import type { Document } from 'yaml';
import type { FrontmatterProperty, PropertyType } from '$lib/libraries/store';
import { samePropRow } from './propRow';

/**
 * Keys whose seq-of-maps value the system round-trips LOSSLESSLY as a structured,
 * editable list — MIG-022 §A.1 `ikhtilāf` (structured scholarly disagreement: each
 * row a `school` + `position`).
 *
 * This module owns the set because it owns the serializer that makes the claim true
 * (`serializeLine`'s `nested-object-list` branch). `store.parseFrontmatter` imports it
 * to route the same keys to the structured parser — one source of truth, and the
 * import direction (store → yamlDoc) is the one that already exists, so no cycle.
 *
 * Adding a key here is a PROMISE that both the parser and the serializer handle its
 * shape without dropping bytes. Every OTHER seq-of-maps is treated as an immutable
 * block (see `immutableBlockKeys`) precisely because that promise does not hold.
 */
export const STRUCTURED_LIST_KEYS: ReadonlySet<string> = new Set([
	'ikhtilāf', 'ikhtilaf', 'الاختلاف',
]);

/**
 * PJ-252 — **what a top-level frontmatter key's value IS.** One answer, for every reader
 * and every writer in the app.
 *
 *  - `list`   — a sequence of pure scalars. Editable, and `items` ARE its values.
 *  - `structured-list` — a `STRUCTURED_LIST_KEYS` seq-of-maps the system round-trips.
 *  - `block`  — a nested map, a seq holding any non-scalar, a block scalar, or a key
 *               written twice. Never rewritable from a flat projection.
 *  - `scalar` — everything else.
 */
export type FmValueShape =
	| { kind: 'list'; items: string[] }
	| { kind: 'structured-list' }
	| { kind: 'block' }
	| { kind: 'scalar' };

/** What the classifier keeps for itself: the on-disk sequence node (so an edited list's
 *  comments survive — `seqCarryingComments`) and a scalar's parsed JS value. */
type FmShapeNode = FmValueShape & { seq?: YAMLSeq; scalarValue?: unknown };

/** The ONE reading of "the text of this scalar node". `seqCarryingComments` matches items
 *  against values produced here, so a second spelling would silently stop carrying comments. */
const scalarText = (n: Scalar): string => (n.value == null ? '' : String(n.value));

/**
 * PJ-252 (APP-KILLER) — **the ONE classifier.** Adding a tag to a note whose `tags:` block
 * carried a comment line, or an item wrapped across two lines, DELETED the tags already in
 * that list, from the `.md`, with no error and a clean re-parse afterwards.
 *
 * Nothing was wrong with either half on its own. Two classifiers were answering the same
 * question about the same bytes, and they disagreed:
 *
 *  - `store.parseFrontmatter` worked on LINES. Its block extent absorbed a `#` comment (and a
 *    wrapped continuation), then required EVERY content line to be a bare `- item` before it
 *    would project an editable list. A comment fails that, so the key came back READ-ONLY with
 *    an EMPTY `value` and no `listItems`.
 *  - `immutableBlockKeys` here asked the `yaml` library, which attaches the comment as a
 *    comment and folds the wrapped item into ONE scalar — "all scalars", so NOT protected.
 *
 * Every list mutator then rebuilt from `p.listItems ?? (p.value ? split : [])` — `[]` on that
 * read-only projection — and the block was spliced out and re-appended holding only the new tag.
 *
 * This was the FOURTH shape of that one defect (2026-07-24 closed seq-of-maps, PJ-182 closed
 * block scalars), and it existed because each closure re-answered the question in a second
 * place. So the question is answered ONCE, here, by the library the writer composes with —
 * `store.parseFrontmatter` imports this and projects what it says. They cannot disagree,
 * because there is no longer a second opinion to disagree with.
 *
 * Two shapes this same pass found by running rather than by reading, both now covered by the
 * one answer: an inline `- history # why` was projected with the COMMENT INSIDE THE TAG VALUE
 * (and written back as the literal tag `"history   # why"`), and an inline flow list under an
 * unrecognised key (`whatever: [a, b]`) was typed `text`, so editing it wrote the sequence back
 * as the string `a, b`.
 *
 * On a YAML PARSE ERROR this returns an empty map and `store.parseFrontmatter` falls back to
 * its line scanner. That is safe by construction, not by luck: `composeFrontmatter`'s H1 branch
 * re-emits the frontmatter bytes verbatim and performs no structural edit at all, so there is
 * no write for a projection to be wrong for. `frontmatterIsRewritable` is what tells the user.
 * A key written TWICE lands there — the library calls it "Map keys must be unique" — which is
 * the right answer for it too: this map and the compose diff are both keyed by NAME, so a
 * rewrite would splice the first block and append a merge of both. Checked by running, not
 * assumed, and pinned by a test; the guard drafted for it here would never have fired.
 */
function classifyDoc(doc: Document.Parsed): Map<string, FmShapeNode> {
	const out = new Map<string, FmShapeNode>();
	if (doc.errors.length || !isMap(doc.contents)) return out;
	for (const pair of doc.contents.items) {
		const k = pair.key;
		if (!isScalar(k) || k.value == null) continue;
		const key = String(k.value);
		const v = pair.value;
		if (isSeq(v)) {
			if (v.items.every((it) => isScalar(it))) {
				// The ordinary tags/aliases list. The items are the LIBRARY's values, which is
				// what makes the folded wrapped item and the commented item come out right.
				out.set(key, { kind: 'list', items: v.items.map((it) => scalarText(it as Scalar)), seq: v });
			} else if (STRUCTURED_LIST_KEYS.has(key) || STRUCTURED_LIST_KEYS.has(key.toLowerCase())) {
				// MIG-101 — `ikhtilāf` is the one seq-of-maps the system round-trips losslessly:
				// `serializeLine` has a real `nested-object-list` branch and the panel edits it
				// through a structured widget. Exempt by KEY, and only in the shape that promise
				// covers — an `ikhtilāf` holding a nested MAP is a block like any other.
				out.set(key, { kind: 'structured-list' });
			} else {
				// 2026-07-24 inspection (APP-KILLER): a seq of maps —
				//   authors:
				//     - name: X
				//       role: Y
				// — projected as a TRUNCATED flat list (`- name: X` became a chip, `role: Y` was
				// dropped), and editing that chip rewrote the block from the truncation.
				out.set(key, { kind: 'block' });
			}
		} else if (isMap(v)) {
			// PJ-136 — a nested map. Its display `value` is a SUMMARY of its child keys, so
			// writing it would replace the whole block with that summary.
			out.set(key, { kind: 'block' });
		} else if (isScalar(v) && (v.type === 'BLOCK_LITERAL' || v.type === 'BLOCK_FOLDED')) {
			// PJ-182 — `desc: |` is `isScalar`, so neither test above saw it. Proven by running:
			// a props array that merely OMITTED the row deleted the block and both prose lines.
			out.set(key, { kind: 'block' });
		} else {
			out.set(key, { kind: 'scalar', scalarValue: isScalar(v) ? v.value : v ?? null });
		}
	}
	return out;
}

/**
 * The one classifier's public face — `store.parseFrontmatter` projects what this says.
 * The import direction (store → yamlDoc) is the one that already exists, so no cycle.
 * The internal `seq` / `scalarValue` fields ride along; the ReadonlyMap return type is what
 * keeps them out of the contract, so no copy is needed to hide them.
 */
export function classifyFrontmatterValues(rawYaml: string): ReadonlyMap<string, FmValueShape> {
	return classifyDoc(parseDocument(rawYaml));
}

export interface FmDoc {
	/** Everything after the closing `---` fence (the note body), verbatim. */
	body: string;
	/** The original YAML block text (between the fences), for passthrough + errors. */
	rawYaml: string;
	/** The editable projection for the PropertyEditor (top-level scalars + simple lists). */
	props: FrontmatterProperty[];
	/** True when the source YAML has parse errors — H1 passthrough mode. */
	hasErrors: boolean;
	/** True when the source actually had a `---` frontmatter fence. */
	hadFence: boolean;
}

/** Split a note into its frontmatter YAML block + body, tolerant of leading blank lines. */
export function splitFrontmatter(content: string): { yaml: string; body: string; hadFence: boolean } {
	// The opening fence must be `---` on its own line (allow a leading BOM/blank lines? No —
	// match the existing parser: the FIRST line must be `---`). Keep it strict for parity.
	const nl = content.indexOf('\n');
	const firstLine = (nl === -1 ? content : content.slice(0, nl)).trim();
	if (firstLine !== '---') {
		return { yaml: '', body: content, hadFence: false };
	}
	const rest = content.slice(nl + 1);
	// Find the closing fence: a line that is exactly `---`. NOTE: do NOT accept a
	// `...` YAML end-marker — the legacy parseFrontmatter (store.ts) closes ONLY on
	// `---`, and the noteModel diff relies on both parsers agreeing on the frontmatter
	// region. Accepting `...` here (but not there) made base.rawYaml and base.props
	// describe different regions → frontmatter duplication (G4 review Finding #1).
	const lines = rest.split('\n');
	let closeIdx = -1;
	for (let i = 0; i < lines.length; i++) {
		const t = lines[i].trim();
		if (t === '---') { closeIdx = i; break; }
	}
	if (closeIdx === -1) {
		// No closing fence — treat the whole thing as body (no frontmatter). Matches
		// the hand-rolled parser, which required a second `---`.
		return { yaml: '', body: content, hadFence: false };
	}
	const yaml = lines.slice(0, closeIdx).join('\n');
	// Body is everything after the closing fence line, preserving the newline that
	// followed it (the fence line + its trailing newline are consumed).
	const body = lines.slice(closeIdx + 1).join('\n');
	return { yaml: yaml ? yaml + '\n' : '', body, hadFence: true };
}

/** Light property-type inference for the display projection (kept minimal + local — the
 *  authoritative encoding is owned by the YAML serializer, this only tags the UI editor). */
function inferType(node: unknown): PropertyType {
	if (typeof node === 'boolean') return 'checkbox';
	if (typeof node === 'number') return 'number';
	if (Array.isArray(node)) return 'list';
	return 'text';
}

/** Project the top-level scalar + simple-list keys into editable FrontmatterProperty[].
 *  Complex values (nested maps, block-scalar strings kept multi-line, seqs of maps) are
 *  PRESERVED in the CST and intentionally NOT projected as editable here (Boss decision:
 *  preserve + read-only for now); compose applies a diff to the CST so they are never lost. */
function projectProps(yaml: string): FrontmatterProperty[] {
	const doc = parseDocument(yaml);
	if (doc.errors.length || !isMap(doc.contents)) return [];
	// PJ-252 — this was a THIRD place answering "is this key a list, or a block?". It now
	// reads the one classifier like everyone else.
	const out: FrontmatterProperty[] = [];
	for (const [key, shape] of classifyDoc(doc)) {
		if (shape.kind === 'list') {
			out.push({ key, value: shape.items.join(', '), type: 'list', listItems: shape.items });
		} else if (shape.kind === 'scalar') {
			const v = shape.scalarValue;
			out.push({ key, value: v == null ? '' : String(v), type: inferType(v) });
		}
		// `block` / `structured-list` → not projected; preserved verbatim by the CST diff.
	}
	return out;
}

/**
 * PJ-207 §15 — decode ONE quoted YAML scalar token (`'O''Brien'`, `"The \\"Real\\" Thing"`) to the
 * string it actually denotes, or `null` when the token does not parse as one.
 *
 * The store's frontmatter projection stripped the quotes and KEPT the escapes, so the escape
 * SYNTAX travelled through the app as if it were data. Untouched keys survived only because both
 * sides of the compose diff share that projection — but the moment the user edits such a key,
 * `serializeLine` re-encodes the syntax as literal text and the note's real value is replaced by
 * its source form: an alias that no longer resolves, a title carrying literal backslashes. It also
 * compounded, because `yamlStringify` re-quotes its own previous output on every subsequent edit.
 *
 * The escape table belongs to the parser, not to a regex in a line-scanner (WA#5), so this routes
 * the decode to the SAME `yaml` library the composer serializes with — read and write then agree
 * by construction instead of by two hand-rolled tables staying in step.
 */
export function decodeQuotedScalar(token: string): string | null {
	try {
		const doc = parseDocument(token);
		if (doc.errors.length) return null;
		const v = doc.contents;
		return isScalar(v) && typeof v.value === 'string' ? v.value : null;
	} catch {
		return null; // a malformed token keeps the caller's fallback — a read path must never throw
	}
}

/** Parse note content into the FmDoc authority. */
export function parseFrontmatterDoc(content: string): FmDoc {
	const { yaml, body, hadFence } = splitFrontmatter(content);
	if (!hadFence) {
		return { body: content, rawYaml: '', props: [], hasErrors: false, hadFence: false };
	}
	const doc = parseDocument(yaml);
	const hasErrors = doc.errors.length > 0;
	return { body, rawYaml: yaml, props: hasErrors ? [] : projectProps(yaml), hasErrors, hadFence: true };
}

/**
 * Top-level keys whose value is a BLOCK this compose must never rewrite.
 *
 * PJ-136 — the authority for "this key holds a block" is the FILE, not the props array
 * handed to `composeFrontmatter`. Those props can be derived from PropertyEditor's
 * `tab.content` cache, which `reconstructFrontmatter` writes WITHOUT a nested block's
 * children, so the key comes back typed as ordinary text. Keying the refusal off the file
 * makes it independent of every upstream projection.
 *
 * PJ-252 — and it is now the SAME reading of the file that the panel's own parser projects
 * from (`classifyDoc`), rather than a second, separately-worded answer to the same question.
 * The shapes this refuses, and why each one had to be learned the hard way, are documented
 * there.
 *
 * The test is stated the CLOSED way — refuse anything not on the writable list — so that a
 * fifth `FmValueShape` added to the union arrives REFUSED rather than silently writable. A
 * new kind is then a visible "why can I not edit this?", never another silent deletion.
 */
const WRITABLE_KINDS: ReadonlySet<FmValueShape['kind']> = new Set(['list', 'scalar', 'structured-list']);
function immutableBlockKeys(shapes: Map<string, FmShapeNode>): Set<string> {
	const out = new Set<string>();
	for (const [key, v] of shapes) if (!WRITABLE_KINDS.has(v.kind)) out.add(key);
	return out;
}

/** Index of the top-level map-item whose key scalar === `key`, or -1. */
function findItemIndex(cst: CST.Document, key: string): number {
	const coll = cst.value as CST.BlockMap | undefined;
	if (!coll || !('items' in coll)) return -1;
	return coll.items.findIndex((it) => {
		const k = it.key;
		return k != null && 'source' in k && CST.resolveAsScalar(k as CST.FlowScalar)?.value === key;
	});
}

/** The CST map-item for a top-level scalar `key`, if present. */
function findItem(cst: CST.Document, key: string): CST.CollectionItem | undefined {
	const coll = cst.value as CST.BlockMap | undefined;
	const idx = findItemIndex(cst, key);
	return idx === -1 || !coll ? undefined : coll.items[idx];
}

/**
 * PJ-252 — rebuild a scalar sequence's items from `items`, carrying each SURVIVING item's own
 * comments, and the sequence's leading comment, across the rewrite.
 *
 * A list key takes the splice-and-append path below (there is no in-place scalar edit for a
 * seq), so before this the whole block — including anything the user had written in it — was
 * re-emitted from the bare values. That was invisible while a commented list was projected
 * read-only; the moment the one classifier makes those lists editable, it would have traded a
 * destroyed list for a destroyed comment. The library carries the comments itself once the
 * original item nodes are reused, so this is a lookup, not a serializer.
 *
 * Matching is by VALUE, so a comment follows its item through an add, a removal or a reorder;
 * a deleted item takes its comment with it, and a new item has none. A value that appears
 * twice keeps the first occurrence's comment for both — the alternative is to guess.
 */
function seqCarryingComments(orig: YAMLSeq, items: string[]): YAMLSeq {
	const byValue = new Map<string, Scalar>();
	for (const it of orig.items) {
		if (!isScalar(it)) continue;
		const k = scalarText(it); // the same reading `classifyDoc` produced `items` with
		if (!byValue.has(k)) byValue.set(k, it);
	}
	const next = new YAMLSeq();
	next.commentBefore = orig.commentBefore;
	next.comment = orig.comment;
	next.items = items.map((v) => byValue.get(v) ?? new Scalar(v));
	return next;
}

/** Serialize a single `key: value` pair to a YAML line via the library (correct quoting).
 *  `origSeq` is the key's sequence node as it exists on disk, when it has one — see
 *  `seqCarryingComments`. */
function serializeLine(key: string, prop: FrontmatterProperty, origSeq?: YAMLSeq): string {
	let value: unknown;
	// MIG-101 safety-inspection fix (2026-07-20) — APP-KILLER. This branch was
	// MISSING, so a `nested-object-list` fell through to `value = prop.value`,
	// the flat ` | `-joined SUMMARY string. Editing one row of a structured
	// ikhtilāf block spliced the whole block-seq out of the CST and wrote a
	// single scalar over it; on reopen the parser's nested branch requires an
	// EMPTY value, so it never fired and every row was gone from the .md with no
	// error. The legacy `reconstructFrontmatter` serialized this correctly — the
	// G4 swap dropped it. Emitting the rows lets the library produce the same
	// block form the parser expects:
	//   ikhtilāf:
	//     - school: Hanafī
	//       position: permissible
	// Empty row-sets deliberately fall through to the scalar path, matching the
	// legacy `&& prop.nestedObjects.length > 0` guard.
	if (prop.type === 'nested-object-list' && prop.nestedObjects?.length) value = prop.nestedObjects;
	else if (prop.type === 'list') {
		const items = prop.listItems ?? (prop.value ? prop.value.split(',').map((s) => s.trim()) : []);
		value = origSeq ? seqCarryingComments(origSeq, items) : items;
	}
	else if (prop.type === 'number' && prop.value.trim() !== '' && !Number.isNaN(Number(prop.value))) value = Number(prop.value);
	else if (prop.type === 'checkbox') value = prop.value === 'true' || prop.value === 'yes';
	else value = prop.value;
	// yamlStringify of a single-key object yields `key: value\n` with correct quoting/escaping.
	// lineWidth:0 disables line-FOLDING (G4 Phase 4 C1) so every emitted value stays on ONE
	// line — the Rust index reader's tolerant line-scanner then decodes it without needing
	// multi-line continuation reconstruction (folded quoted values would be mis-read).
	return yamlStringify({ [key]: value }, { lineWidth: 0 });
}

/**
 * Compose the full note content by applying the diff between `oldProps` (the base
 * projection as parsed) and `newProps` (after the user's edit) onto a FRESH CST
 * re-parsed from the original frontmatter, then re-attaching the body. Untouched
 * keys — nested maps, block scalars, comments — stay byte-perfect.
 *
 * PURE: it never mutates `fm`; it re-parses `fm.rawYaml` each call. Because the
 * diff is always base→current applied to the base bytes, repeated composes (even
 * after intermediate saves) stay correct and byte-stable. `bodyOverride` lets the
 * noteModel supply the live edited body (the model owns body separately).
 *
 * H1: on parse errors, pass the original frontmatter bytes through verbatim — no
 * content is ever lost.
 */
export function composeContent(
	fm: FmDoc,
	oldProps: FrontmatterProperty[],
	newProps: FrontmatterProperty[],
	bodyOverride?: string,
): string {
	return composeFrontmatter(fm.rawYaml, fm.hadFence, oldProps, newProps, bodyOverride ?? fm.body);
}

/**
 * PJ-207 §15 — can this note's frontmatter be REWRITTEN at all?
 *
 * `composeFrontmatter`'s H1 branch below preserves malformed YAML byte-for-byte rather than risk
 * rewriting it — which is right for the FILE and catastrophic for the USER, because it discards
 * every pending property intent while the save path reports success. The model calls this at open
 * time so it can REFUSE those intents instead, and the panel's existing "could not be saved"
 * banner does the telling.
 *
 * Cheaper than `parseFrontmatterDoc` on purpose: no `projectProps` pass, since the caller only
 * needs the yes/no.
 */
export function frontmatterIsRewritable(rawYaml: string): boolean {
	if (!rawYaml.trim()) return true; // empty fence — nothing to misparse
	return parseDocument(rawYaml).errors.length === 0;
}

/**
 * The core byte-perfect write. Applies the diff between `oldProps` (the base
 * projection) and `newProps` (after edits) onto a FRESH CST re-parse of `rawYaml`,
 * then re-attaches `body`. Untouched keys stay byte-perfect.
 *
 * THE UNIFICATION (G4 Phase 2): the projection need not be lossless, only
 * CONSISTENT — the noteModel supplies BOTH oldProps and newProps from the SAME
 * `parseFrontmatter` projection the PropertyEditor uses, so an unedited key
 * (block scalar, quoted value, nested map) projects identically on both sides →
 * no diff → the CST preserves the REAL value untouched. Only keys the user
 * actually edited are written, via `serializeLine` (correct quoting).
 *
 * H1: on parse errors, pass the original frontmatter bytes through verbatim.
 */
export function composeFrontmatter(
	rawYaml: string,
	hadFence: boolean,
	oldProps: FrontmatterProperty[],
	newProps: FrontmatterProperty[],
	body: string,
): string {
	// G4 review #2 — match the note's dominant line ending on the FENCE lines (the
	// CST/rawYaml/body already preserve their own EOL); hardcoding `---\n` on a
	// CRLF note produced mixed EOL + a spurious diff on the first save.
	const eol = (rawYaml || body).includes('\r\n') ? '\r\n' : '\n';
	if (!hadFence) {
		// No frontmatter fence. If properties were added (PropertyEditor on a plain
		// .md), CREATE a fenced block from them (legacy buildFullContent parity).
		if (newProps.length === 0) return body;
		// PJ-136 — a nested map cannot reach here (there is no source YAML to have
		// parsed one from), but never let its display SUMMARY be serialized as a scalar.
		const yamlText = newProps
			.filter((p) => p.type !== 'nested-map')
			.map((p) => serializeLine(p.key, p).replace(/\r?\n/g, eol))
			.join('');
		return `---${eol}${yamlText}---${eol}${body}`;
	}
	// PJ-252 — ONE parse feeds both the H1 error gate and the classification the refusal and the
	// list serializer read, so nothing here can form a second opinion about these bytes.
	const doc = parseDocument(rawYaml);
	if (doc.errors.length) return `---${eol}${rawYaml}---${eol}${body}`; // H1
	const shapes = classifyDoc(doc);

	let cst: CST.Document | null = null;
	for (const tok of new Parser().parse(rawYaml)) {
		if (tok.type === 'document') { cst = tok; break; }
	}
	if (!cst) {
		// No document token — the fence is EMPTY (`---\n---`) or whitespace-only. The
		// YAML parsed WITHOUT errors (H1 above already caught malformed input), so it is
		// safely editable: build a block from newProps (adds the first tag/property on an
		// empty-fence note). G4 review Finding 1 — empty-fence tag-add was a no-op.
		if (newProps.length === 0) return `---${eol}${rawYaml}---${eol}${body}`;
		// PJ-136 — a nested map cannot reach here (there is no source YAML to have
		// parsed one from), but never let its display SUMMARY be serialized as a scalar.
		const yamlText = newProps
			.filter((p) => p.type !== 'nested-map')
			.map((p) => serializeLine(p.key, p).replace(/\r?\n/g, eol))
			.join('');
		return `---${eol}${yamlText}---${eol}${body}`;
	}

	// PJ-136 — a `nested-map` property is IMMUTABLE here. Its `value` is a summary of
	// its child keys for display, not its content, so writing it would replace the
	// whole block with that summary; and its row must never be spliceable, or a UI
	// that merely stops listing it would delete it.
	//
	// This is enforced in the WRITE PATH rather than in the widget on purpose. A
	// read-only widget protects the data only as long as every caller keeps it
	// read-only; refusing here means the block survives however the panel behaves —
	// the same reason `adoptDisk` grew an identity guard instead of trusting callers.
	// The authoritative bytes stay untouched in the CST and are emitted verbatim.
	// The set is read from the FILE, never from the props arrays. That distinction is
	// the whole fix: a props array can arrive claiming `source` is ordinary text —
	// `reconstructFrontmatter` drops a nested block's children when PropertyEditor
	// caches `tab.content`, and re-parsing that cache re-projects the key as `text`.
	// Trusting the props array there let the SET/ADD branch splice the block out and
	// append `source: ""`. The file always knows; ask it.
	const immutableKeys = immutableBlockKeys(shapes);
	const oldByKey = new Map(
		oldProps.filter((p) => !immutableKeys.has(p.key)).map((p) => [p.key, p]),
	);
	const newByKey = new Map(
		newProps.filter((p) => !immutableKeys.has(p.key)).map((p) => [p.key, p]),
	);

	// REMOVE — keys present before, gone now: splice their CST map item.
	const coll = cst.value as CST.BlockMap | undefined;
	if (coll && 'items' in coll) {
		for (const key of oldByKey.keys()) {
			if (!newByKey.has(key) && !immutableKeys.has(key)) {
				const idx = findItemIndex(cst, key);
				if (idx !== -1) coll.items.splice(idx, 1);
			}
		}
	}

	// SET (existing scalar → byte-perfect in-place) or ADD (new key / shape change → append).
	const addLines: string[] = [];
	for (const [key, np] of newByKey) {
		const op = oldByKey.get(key);
		// PJ-182 — this was the LAST LINK, and it had not been moved. The check used to be
		// `value === value && type === type && JSON.stringify(listItems) === …`, which
		// decides a `nested-object-list` from `value` — a ` | `-joined DISPLAY SUMMARY.
		// Delete a row from an ikhtilāf block without the summary happening to change and
		// the write was dropped: `touchedSince`, `plan` and `setPropValue` had all been
		// taught to carry the rows, and the composer still said "unchanged". It was also
		// the fifth spelling of list equality, in the very codebase where `sameList`
		// exists to collapse them. One predicate now, in `propRow.ts`.
		if (op && samePropRow(op, np)) {
			continue; // unchanged — never rewrite a key the user did not edit
		}
		const item = findItem(cst, key);
		// Safety inspection 2026-08-01 — `nested-object-list` must never take the
		// scalar SET fast-path. When the key pre-exists on disk as a SCALAR (a flat
		// `ikhtilāf: old` later given structured rows by the panel widget),
		// `setScalarValue` wrote `np.value` — the ` | `-joined DISPLAY SUMMARY — and
		// the user's rows never reached disk. Excluded exactly like `list`, so it
		// falls through to the splice-and-append branch below, which replaces the
		// scalar item and emits `serializeLine`'s block rows. (Type-union sweep:
		// `nested-map` is the only other block-shaped type, but `serializeLine` has
		// no block branch for it by design — its bytes are preserved verbatim in the
		// CST via `immutableBlockKeys`, read from the FILE, so it never reaches this
		// loop. Every remaining type is genuinely scalar.)
		if (item && item.value && 'type' in item.value && item.value.type === 'scalar' && np.type !== 'list' && np.type !== 'nested-object-list') {
			CST.setScalarValue(item.value as CST.FlowScalar, np.value);
		} else {
			const idx = findItemIndex(cst, key);
			if (idx !== -1) (cst.value as CST.BlockMap).items.splice(idx, 1);
			// PJ-252 — hand the key's ON-DISK sequence to the serializer so the user's comments
			// inside an edited list survive the splice-and-append.
			addLines.push(serializeLine(key, np, shapes.get(key)?.seq));
		}
	}

	// PJ-252 — a note whose edited key was its FIRST property gained a blank line under the
	// opening `---` on every edit. Pre-existing (measured at HEAD: byte-identical output), and
	// the exact twin of the blank line `ensure_cid_cn` was leaving in the Rust writer — one
	// concern, two surfaces, so it is fixed here rather than left as the odd one out.
	//
	// Splicing that key empties the CST residue, and `+= eol` on an EMPTY string is what
	// invented the line. Note what that means, because it is the whole subtlety: at this point
	// a blank line the user typed and a blank line the splice left are indistinguishable —
	// both are simply gone from the residue. HEAD preserved the user's blank only by the same
	// accident that fabricated one when there was none. So the user's blank is restored from
	// `rawYaml`, the file's own bytes, which is the only place it still exists.
	let yamlText = CST.stringify(cst);
	const leadingBlank = /^\r?\n/.test(rawYaml);
	if (yamlText) {
		if (!leadingBlank) yamlText = yamlText.replace(/^\r?\n/, '');
		if (!yamlText.endsWith('\n')) yamlText += eol;
	} else if (leadingBlank) {
		yamlText = eol;
	}
	// Appended (added-key) lines use the note's EOL for consistency.
	for (const line of addLines) {
		const l = line.replace(/\r?\n/g, eol);
		yamlText += l.endsWith(eol) ? l : l + eol;
	}
	return `---${eol}${yamlText}---${eol}${body}`;
}
