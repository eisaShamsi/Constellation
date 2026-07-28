/**
 * MIG-107 Slice 4 — turning a panel's edited rows into INTENTS.
 *
 * The Properties panel keeps local rows while you type (it must — a half-typed key or an
 * uncommitted value is that panel's business, not a fact about the note). At commit time those rows
 * have to reach the shared authority. Doing it the old way — handing over the whole array — is the
 * defect: an array assembled from one panel's view silently deletes whatever another writer changed
 * in the meantime (PJ-174 AK-2/AK-3).
 *
 * ## The rule that makes this safe
 *
 * **A commit may ADD and SET freely, but it may only REMOVE a key the panel actually knew about.**
 *
 * `seededKeys` is what the panel was showing when it last read the model. A key that appeared
 * afterwards — a tag added from the file-tree menu, a property set in the *other* panel — is
 * therefore not in that set, and **there is no code path by which this commit can delete it.** The
 * loss is not guarded against; it is unrepresentable.
 *
 * That is also why removal is derived here rather than taken from "keys missing from the local
 * rows": missing-from-local is exactly the condition an unseen key satisfies.
 *
 * ## Why this is a pure planner
 *
 * `plan()` computes the operations from three plain inputs and returns them. It touches nothing, so
 * every rule above is testable without a component, a model, or a DOM — which is the whole reason
 * this logic lives here instead of inside the `.svelte` file (the noteSession discipline: drag the
 * glue into the light).
 */
import type { FrontmatterProperty } from '$lib/libraries/store';

export type PropOp =
	| { op: 'set'; key: string; value: string; type: FrontmatterProperty['type']; listItems?: string[] }
	| { op: 'add'; prop: FrontmatterProperty }
	| { op: 'remove'; key: string }
	| { op: 'order'; key: string; beforeKey: string | null };

/** A row is committable once it has a real key — a blank row stays the panel's own business. */
const committable = (p: FrontmatterProperty) => !!p.key && !!p.key.trim();

const sameList = (a?: string[], b?: string[]) =>
	JSON.stringify(a ?? null) === JSON.stringify(b ?? null);

/**
 * Work out what the model must be told, given what the panel is showing.
 *
 * @param localRows   the panel's current rows (may include a blank in-progress row)
 * @param modelProps  the authority as it stands RIGHT NOW (may contain keys the panel never saw)
 * @param seededKeys   the keys the panel was showing when it last read the model
 * @param touchedKeys  the keys the USER actually edited since that read. A commit may only SET
 *                     these — `seededKeys` protects keys the panel never saw, this protects keys it
 *                     saw but did not touch, whose value another writer may have moved on since.
 *                     Omit to allow every row (the legacy whole-array meaning).
 */
export function plan(
	localRows: FrontmatterProperty[],
	modelProps: FrontmatterProperty[],
	seededKeys: ReadonlySet<string>,
	touchedKeys?: ReadonlySet<string>,
): PropOp[] {
	const ops: PropOp[] = [];
	const rows = localRows.filter(committable);
	const localKeys = new Set(rows.map((p) => p.key));
	const byModelKey = new Map(modelProps.map((p) => [p.key, p]));

	// REMOVE — only keys this panel was actually showing. A key that arrived after the panel's last
	// read is absent from `seededKeys`, so it can never be selected here. This single line is what
	// makes another writer's frontmatter safe from this commit.
	for (const key of seededKeys) {
		if (!localKeys.has(key) && byModelKey.has(key)) ops.push({ op: 'remove', key });
	}

	// ADD / SET — a key the model does not have is new; one it has is a value change (the intent
	// itself no-ops when nothing actually differs, so an unchanged row costs nothing).
	for (const row of rows) {
		const cur = byModelKey.get(row.key);
		if (!cur) {
			ops.push({ op: 'add', prop: { ...row } });
			continue;
		}
		// A key the user did not touch is left exactly as the model has it — even if this panel is
		// displaying something older. Writing it back is how a value another writer changed gets
		// silently reverted on a key both panels show.
		if (touchedKeys && !touchedKeys.has(row.key)) continue;
		if (cur.value !== row.value || cur.type !== row.type || !sameList(cur.listItems, row.listItems)) {
			ops.push({
				op: 'set', key: row.key, value: row.value, type: row.type,
				...(row.listItems ? { listItems: [...row.listItems] } : {}),
			});
		}
	}

	// ORDER — make the model's order match what the user is looking at, for the keys the panel knows
	// about. Order does not survive to disk for existing keys (`propsContract.test.ts` pins that a
	// shuffle alone is a byte-identical write), so this exists purely so the two panels agree on
	// screen — including after a rename, which is a remove + an add and would otherwise jump the row
	// to the bottom.
	//
	// ★ PJ-174 #1f — an earlier version of this comment claimed keys the panel never saw are "never
	// moved". That was FALSE, and the inspection caught it: a trailing `beforeKey: null` means
	// "move to the END of the model array", which pushes any foreign key sitting after it. Anchoring
	// only BETWEEN two known keys keeps every operation inside the panel's own span, so an unseen
	// key keeps its position as well as its value.
	for (let i = 0; i < rows.length - 1; i++) {
		ops.push({ op: 'order', key: rows[i].key, beforeKey: rows[i + 1].key });
	}

	return ops;
}


/**
 * Which keys the user has actually edited, derived by comparing the panel's CURRENT rows against
 * the rows it was seeded with.
 *
 * ★ Why derived and not hand-marked. The first version of this had the panel call `touchedKeys.add`
 * from its edit handlers — and it was wired at 3 of the panel's **16** mutation sites. The tag
 * editor was one of the 13 that were missed, so adding a tag in one panel never reached the model
 * at all: the Boss saw it in the panel he typed it into and nowhere else. Hand-marking is a list
 * that must be kept complete forever, by everyone, including whoever adds the 17th site.
 *
 * Comparing against the seed cannot be forgotten: any row that differs is touched, whatever code
 * changed it, and a site added tomorrow is covered the day it is written.
 */
export function touchedSince(
	seededRows: FrontmatterProperty[],
	localRows: FrontmatterProperty[],
): Set<string> {
	const seededByKey = new Map(seededRows.filter(committable).map((p) => [p.key, p]));
	const touched = new Set<string>();
	for (const row of localRows.filter(committable)) {
		const was = seededByKey.get(row.key);
		if (!was || was.value !== row.value || was.type !== row.type || !sameList(was.listItems, row.listItems)) {
			touched.add(row.key);
		}
	}
	// A key that was seeded and is now gone was edited too — the user deleted it. (Removal is
	// governed by `seededKeys` in `plan`; this keeps the two notions consistent.)
	for (const key of seededByKey.keys()) {
		if (!localRows.some((p) => p.key === key)) touched.add(key);
	}
	return touched;
}

/** The intent functions this planner drives, injected so the planner stays pure and testable. */
export interface IntentSink {
	setValue: (key: string, value: string, opts?: { listItems?: string[]; type?: FrontmatterProperty['type'] }) => boolean;
	add: (prop: FrontmatterProperty) => boolean;
	remove: (key: string) => boolean;
	order: (key: string, beforeKey: string | null) => boolean;
}

/** Apply a plan. Returns whether anything actually changed, so a caller can skip a pointless write. */
export function apply(ops: PropOp[], sink: IntentSink): boolean {
	let changed = false;
	for (const op of ops) {
		switch (op.op) {
			case 'remove': changed = sink.remove(op.key) || changed; break;
			case 'add': changed = sink.add(op.prop) || changed; break;
			case 'set':
				changed = sink.setValue(op.key, op.value, { listItems: op.listItems, type: op.type }) || changed;
				break;
			case 'order': changed = sink.order(op.key, op.beforeKey) || changed; break;
		}
	}
	return changed;
}
