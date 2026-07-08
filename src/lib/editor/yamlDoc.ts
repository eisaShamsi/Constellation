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
import { Parser, CST, parseDocument, stringify as yamlStringify, isSeq, isMap, isScalar } from 'yaml';
import type { FrontmatterProperty, PropertyType } from '$lib/libraries/store';

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
	// Find the closing fence: a line that is exactly `---` (or `...`).
	const lines = rest.split('\n');
	let closeIdx = -1;
	for (let i = 0; i < lines.length; i++) {
		const t = lines[i].trim();
		if (t === '---' || t === '...') { closeIdx = i; break; }
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
	const out: FrontmatterProperty[] = [];
	for (const pair of doc.contents.items) {
		const keyNode = pair.key;
		const key = isScalar(keyNode) && keyNode.value != null ? String(keyNode.value) : '';
		if (!key) continue;
		const node = pair.value; // the value NODE (not a materialized JS value)
		if (isSeq(node)) {
			// A sequence of SCALARS → editable list; a seq of maps → preserved in CST (skip).
			if (node.items.every((it) => isScalar(it))) {
				const items = node.items.map((it) => String((it as { value?: unknown }).value ?? ''));
				out.push({ key, value: items.join(', '), type: 'list', listItems: items });
			}
			// else: seq-of-maps → not projected (preserved by the CST diff).
		} else if (isMap(node)) {
			// Nested map → preserved in the CST, not editable here (Boss decision).
			continue;
		} else {
			// Scalar (or null/empty).
			const jsVal = isScalar(node) ? node.value : node == null ? null : node;
			out.push({ key, value: jsVal == null ? '' : String(jsVal), type: inferType(jsVal) });
		}
	}
	return out;
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

/** Serialize a single `key: value` pair to a YAML line via the library (correct quoting). */
function serializeLine(key: string, prop: FrontmatterProperty): string {
	let value: unknown;
	if (prop.type === 'list') value = prop.listItems ?? (prop.value ? prop.value.split(',').map((s) => s.trim()) : []);
	else if (prop.type === 'number' && prop.value.trim() !== '' && !Number.isNaN(Number(prop.value))) value = Number(prop.value);
	else if (prop.type === 'checkbox') value = prop.value === 'true' || prop.value === 'yes';
	else value = prop.value;
	// yamlStringify of a single-key object yields `key: value\n` with correct quoting/escaping.
	return yamlStringify({ [key]: value });
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
	const body = bodyOverride ?? fm.body;
	if (!fm.hadFence) {
		// A note that had NO frontmatter fence. If properties were added (e.g. via the
		// PropertyEditor on a plain .md), CREATE a fresh fenced block from them —
		// matching the legacy buildFullContent, which would build a block from props.
		// With no props, it stays fenceless (body only).
		if (newProps.length === 0) return body;
		const yamlText = newProps.map((p) => serializeLine(p.key, p)).join('');
		return `---\n${yamlText}---\n${body}`;
	}
	if (fm.hasErrors) return `---\n${fm.rawYaml}---\n${body}`; // H1

	let cst: CST.Document | null = null;
	for (const tok of new Parser().parse(fm.rawYaml)) {
		if (tok.type === 'document') { cst = tok; break; }
	}
	if (!cst) return `---\n${fm.rawYaml}---\n${body}`;

	const oldByKey = new Map(oldProps.map((p) => [p.key, p]));
	const newByKey = new Map(newProps.map((p) => [p.key, p]));

	// REMOVE — keys present before, gone now: splice their CST map item.
	const coll = cst.value as CST.BlockMap | undefined;
	if (coll && 'items' in coll) {
		for (const key of oldByKey.keys()) {
			if (!newByKey.has(key)) {
				const idx = findItemIndex(cst, key);
				if (idx !== -1) coll.items.splice(idx, 1);
			}
		}
	}

	// SET (existing scalar → byte-perfect in-place) or ADD (new key / shape change → append).
	const addLines: string[] = [];
	for (const [key, np] of newByKey) {
		const op = oldByKey.get(key);
		if (op && op.value === np.value && op.type === np.type &&
			JSON.stringify(op.listItems ?? null) === JSON.stringify(np.listItems ?? null)) {
			continue; // unchanged
		}
		const item = findItem(cst, key);
		if (item && item.value && 'type' in item.value && item.value.type === 'scalar' && np.type !== 'list') {
			CST.setScalarValue(item.value as CST.FlowScalar, np.value);
		} else {
			const idx = findItemIndex(cst, key);
			if (idx !== -1) (cst.value as CST.BlockMap).items.splice(idx, 1);
			addLines.push(serializeLine(key, np));
		}
	}

	let yamlText = CST.stringify(cst);
	if (!yamlText.endsWith('\n')) yamlText += '\n';
	for (const line of addLines) yamlText += line.endsWith('\n') ? line : line + '\n';
	return `---\n${yamlText}---\n${body}`;
}
