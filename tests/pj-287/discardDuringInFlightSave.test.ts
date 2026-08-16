/**
 * PJ-287 — A WRITE COMPOSED FROM A MODEL THAT NO LONGER EXISTS MUST NOT BE REPORTED AS SUCCESS.
 *
 * ## The failure, in the user's terms
 *
 * A save fails because something holds the `.md` (a sync agent, a scanner) and the save-health
 * banner appears. The ~10 s auto-retry fires and parks on `await write`. **In that window the user
 * clicks "Discard my changes."** `discardFailedSave` re-seeds the tab from disk — correctly, that
 * is the user's stated intent — which mints a NEW model under the SAME tab id. Then the parked
 * write lands.
 *
 * `saveUnchained` guards the two model mutations on lineage (`expectGen`, added by PJ-207 §15), so
 * `markSaved` and `noteDiskSynced` both correctly refuse. **But those guards return `void`**, and
 * the function goes on to `clearNetIf`, `onSuccess`, and `return r` — success. So:
 *
 * - the bytes the user explicitly discarded are on disk,
 * - the version they chose to keep survives only in an in-memory model that reports CLEAN, and is
 *   therefore never written back — gone for good when the tab closes,
 * - the crash-recovery net is cleared, removing the last other copy,
 * - `onSuccess` reindexes and re-embeds the index from the discarded content,
 * - the banner clears, which reads as "your save went through."
 *
 * An explicit decision of the user's, silently inverted.
 *
 * ## Why this test drives the session layer directly
 *
 * The mechanism is entirely `saveUnchained`'s: compose captures a lineage, the write parks, the
 * id-slot is re-seeded, the write resolves. Driving it here — with a writer the test releases by
 * hand — makes the interleaving deterministic rather than a matter of timing, which is what
 * Reproduce-First asks for. `open(id, path, content)` on an id that already has a model is exactly
 * what `discardFailedSave` reaches through `reloadTabsFromDisk({discardLocalEdits: true})`.
 */
import { describe, it, expect, vi } from 'vitest';
import * as S from '$lib/editor/noteSession';
import { getModel, isDirty, hasUnsavedRecovery } from '$lib/editor/noteModel';

const file = (body: string) => `---\ntitle: A\ncid_cn: C_A\n---\n${body}`;

/** A writer the test releases by hand, so the "parked write" window is exact. */
function gatedWriter() {
	let release!: () => void;
	const gate = new Promise<void>((r) => (release = r));
	const writes: Array<{ path: string; content: string }> = [];
	return {
		writes,
		release: () => release(),
		write: async (path: string, content: string) => {
			await gate;
			writes.push({ path, content });
		},
	};
}

describe('PJ-287 — discard during an in-flight save', () => {
	it('does NOT report success for a write whose model was replaced mid-flight', async () => {
		const id = 'tab-1';
		const P = '/lib/A.md';
		const w = gatedWriter();
		const clearNetIf = vi.fn();
		const onSuccess = vi.fn();
		const setNet = vi.fn();

		// The note is open with disk content, and the user has typed.
		S.open(id, P, file('disk version — the one the user chooses to KEEP'));
		S.editBody(id, 'the edit the user will DISCARD', P);
		expect(isDirty(id)).toBe(true);

		// The retry fires and parks on the write.
		const inFlight = S.save(id, P, { write: w.write, setNet, clearNetIf, onSuccess }, 'retry_save');

		// …and in that window the user clicks "Discard my changes": the tab is re-seeded from
		// disk under the SAME id, which mints a new model generation.
		S.open(id, P, file('disk version — the one the user chooses to KEEP'));
		const freshGen = getModel(id)!.gen;

		// The parked write now lands.
		w.release();
		const outcome = await inFlight;

		// The write itself did happen — that is unavoidable, it was already awaited.
		expect(w.writes).toHaveLength(1);
		expect(w.writes[0].content).toContain('DISCARD');

		// But NOTHING may ratify it.
		expect(outcome.ok).toBe(false);
		expect((outcome as { reason: string }).reason).toBe('superseded');
		expect(onSuccess).not.toHaveBeenCalled(); // no reindex/re-embed/broadcast of discarded text
		// The net-clear DOES run, and must: it is a compare-and-clear against the bytes that were
		// just written, which are provably on disk by now. What protects the kept version is not
		// skipping this call — that was my own wrong exception, and it stranded a stale entry on
		// every superseded path — but the fact that the kept version was stashed FIRST, so the
		// net no longer holds `r.content` and this clear matches nothing.
		expect(clearNetIf).toHaveBeenCalledWith(P, expect.stringContaining('DISCARD'));

		// And the model the user kept must not be left believing it matches disk — disk now holds
		// the superseded bytes, so it needs writing back.
		expect(getModel(id)!.gen).toBe(freshGen); // the fresh model is untouched…
		expect(hasUnsavedRecovery(id)).toBe(true); // …but flagged as content disk does not have
	});

	/**
	 * **The residue, caught by the gate on the first cut of this fix.** Refusing to ratify was
	 * correct and still left the kept version living ONLY in memory: `discardFailedSave` clears
	 * the net before this write resolves, so closing the tab disposes the model and that version
	 * exists nowhere — while the file holds the bytes the user discarded, with nothing recording
	 * the split. Two obligations follow from that, and both are pinned here.
	 */
	it('re-stashes the kept version and records the divergence', async () => {
		const id = 'tab-5';
		const P = '/lib/D.md';
		const w = gatedWriter();
		const net: Array<{ path: string; content: string }> = [];
		const onSuperseded = vi.fn();
		const KEPT = 'the version the user chose to KEEP';

		S.open(id, P, file(KEPT));
		S.editBody(id, 'the edit being discarded', P);
		const inFlight = S.save(
			id,
			P,
			{ write: w.write, setNet: (path, content) => net.push({ path, content }), onSuperseded },
			'retry_save',
		);
		S.open(id, P, file(KEPT)); // the discard re-seeds from disk
		w.release();
		await inFlight;

		// The net's LAST entry is the kept version — not the discarded edit stashed before the
		// write. That is what survives a close, and what a reopen restores.
		expect(net.at(-1)!.path).toBe(P);
		expect(net.at(-1)!.content).toContain(KEPT);
		expect(net.at(-1)!.content).not.toContain('being discarded');

		// And the divergence is reported, so it stays decidable after the fact.
		expect(onSuperseded).toHaveBeenCalledTimes(1);
		expect(onSuperseded.mock.calls[0][0].path).toBe(P);
	});

	it('still reports success on the ordinary path, where the lineage holds', async () => {
		const id = 'tab-2';
		const P = '/lib/B.md';
		const w = gatedWriter();
		const clearNetIf = vi.fn();
		const onSuccess = vi.fn();

		S.open(id, P, file('before'));
		S.editBody(id, 'after', P);
		const inFlight = S.save(id, P, { write: w.write, clearNetIf, onSuccess }, 'editor_save');
		w.release();
		const outcome = await inFlight;

		expect(outcome.ok).toBe(true);
		expect(clearNetIf).toHaveBeenCalled();
		expect(onSuccess).toHaveBeenCalled();
		expect(isDirty(id)).toBe(false); // marked clean, because the write really is this model's
		expect(hasUnsavedRecovery(id)).toBe(false);
	});

	/**
	 * **The case the first cut of this fix got wrong, found by the per-build gate.**
	 *
	 * "The lineage no longer holds" covers THREE states, not one: the model is gone, the model is
	 * the same note re-seeded (a new generation), or **the slot now holds a DIFFERENT NOTE** —
	 * which is the ordinary outcome of clicking another note in the sidebar, because the departure
	 * flush is skipped for a model that reads clean while its write is still parked.
	 *
	 * The repair line stamped `r.content` as the slot's disk baseline with no identity guard, so
	 * note A's bytes became note B's baseline and B was flagged as holding unsaved recovery.
	 * B's `adoptDisk` then refuses every external change, `diskDiffersFromBaseline` reports true
	 * for B's own untouched bytes — and, silently, merely viewing B stashes its net as real work,
	 * re-arming the PJ-181 stale-net class for a note that has none. A fix for a cross-note bleed
	 * that introduced a cross-note bleed.
	 */
	it('never stamps one note bytes onto another when the slot holds a DIFFERENT note', async () => {
		const idA = 'tab-4';
		const A = '/lib/A.md';
		const B = '/lib/B.md';
		const w = gatedWriter();

		S.open(idA, A, file('A on disk'));
		S.editBody(idA, 'A edited', A);
		const inFlight = S.save(idA, A, { write: w.write }, 'editor_save');

		// The user clicks note B; the id-slot is re-seeded with a DIFFERENT note.
		S.open(idA, B, file('B on disk, untouched'));
		const bBaselineBefore = getModel(idA)!.diskBaseline;

		w.release();
		const outcome = await inFlight;

		expect(outcome.ok).toBe(false); // still not ratified — that part was right
		// …and note B is left exactly as it was.
		expect(getModel(idA)!.diskBaseline).toBe(bBaselineBefore);
		expect(getModel(idA)!.diskBaseline).not.toContain('A edited');
		expect(hasUnsavedRecovery(idA)).toBe(false); // B has no unsaved recovery; saying it does re-arms PJ-181
	});

	/**
	 * **When the re-seed read back the very bytes we wrote, there is nothing to repair** — and
	 * "repairing" anyway is how the third finding on this fix arose. `write_note` is async while
	 * `read_note` is sync, so a reload issued after the write lands can observe it and still
	 * deliver its response first; the model is then re-seeded with exactly the written content.
	 *
	 * Stashing that into the recovery net records ALREADY-DURABLE content as "work the disk never
	 * had". Nothing clears it (the model is clean, so no save ever runs), it survives the tab, and
	 * on the next open after an external edit it beats the NEWER file — then gets written back
	 * over that edit. The PJ-181 app-killer, re-armed by a repair for a different one.
	 */
	it('does nothing when the re-seeded model already matches what was written', async () => {
		const id = 'tab-6';
		const P = '/lib/E.md';
		const w = gatedWriter();
		const net: Array<{ path: string; content: string }> = [];
		const onSuperseded = vi.fn();

		S.open(id, P, file('before'));
		S.editBody(id, 'the edit', P);
		const inFlight = S.save(
			id,
			P,
			{ write: w.write, setNet: (path, content) => net.push({ path, content }), onSuperseded },
			'retry_save',
		);
		w.release();
		await Promise.resolve();
		// The reload read the bytes the write had just landed, and re-seeded from them.
		const composed = net.at(-1)!.content; // exactly what the write put on disk
		S.open(id, P, composed);
		const outcome = await inFlight;

		expect(outcome.ok).toBe(false); // the lineage still broke — nothing is ratified
		// …but there is no divergence, so no phantom recovery and no false report.
		expect(hasUnsavedRecovery(id)).toBe(false);
		expect(onSuperseded).not.toHaveBeenCalled();
		expect(net.filter((n) => n.content === composed)).toHaveLength(1); // only the pre-write stash
	});

	/** A write for a note whose tab was CLOSED mid-flight is the same class: nothing to ratify. */
	it('does not report success when the model is gone entirely', async () => {
		const id = 'tab-3';
		const P = '/lib/C.md';
		const w = gatedWriter();
		const clearNetIf = vi.fn();
		const onSuccess = vi.fn();

		S.open(id, P, file('before'));
		S.editBody(id, 'after', P);
		const inFlight = S.save(id, P, { write: w.write, clearNetIf, onSuccess }, 'close_flush');
		S.close(id); // the tab closes while the write is parked
		w.release();
		const outcome = await inFlight;

		expect(outcome.ok).toBe(false);
		expect(onSuccess).not.toHaveBeenCalled();
		// **And the compare-and-clear MUST still run.** Not clearing it was my own invented
		// exception, and it was wrong: this is "clear ONLY IF the net still holds exactly the
		// bytes just written", which by now are provably on disk. Skipping it stranded the
		// pre-write stash — unflagged, so it reads as real unsaved work — on a note whose tab is
		// gone, where nothing re-stashes and nothing prunes a closed note's net. That entry then
		// beats a NEWER file on the next open and is written back over it.
		expect(clearNetIf).toHaveBeenCalledWith(P, expect.stringContaining('after'));
	});
});
