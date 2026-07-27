/**
 * MIG-104 — the Earned-Life Ledger reproduction harness.
 *
 * Slice 0 (2026-07-27). Every later slice APPENDS its named failure recipe here, so the
 * ledger's guarantees are proven by replay rather than by argument (Reproduce-First).
 *
 * What this harness exists to make cheap:
 *   - building a temp Universe with a `.constellation/` dir, the way `conn.path()` finds it;
 *   - appending ledger lines and reading them back BYTE-EXACTLY, which is the whole of the
 *     Q5 OS-portability contract (`\n` endings, NFC, forward slashes, no drive letters);
 *   - asserting the two fold algebras, which differ per stream and must never be confused:
 *       * link-life   FOLDS  — `n` = max, decisions = latest, commutative + idempotent;
 *       * note-history NEVER FOLDS — the record IS the payload (three rows recording
 *         `ma` → `mas` → `masadir` are a thought being typed; folding them destroys it).
 */

export const LEDGER = {
	/** The append-only tail: every mutation is an append, so a write can never clobber. */
	tail: 'earned.jsonl',
	/** One line per earned link, current state — bounded by earned count, never by history. */
	snapshot: 'earned.snapshot.jsonl',
	/** The note-history stream. Never folded, never compacted. */
	history: 'note-history.jsonl',
} as const;

/** A ledger record as it appears on disk. Keys are deliberately short — this is written per event. */
export type LedgerLine = Record<string, unknown>;

/**
 * Serialize one record the way the Rust appender must: compact JSON, one line, `\n`.
 * No trailing spaces, no CRLF, no BOM — a ledger written on Windows must read
 * byte-identically on macOS (Boss ruling Q5: the Universe is portable, never concurrent).
 */
export function encodeLine(rec: LedgerLine): string {
	return JSON.stringify(rec) + '\n';
}

/** Parse a ledger file's text, skipping (and counting) unparseable lines — never throwing. */
export function decodeLines(text: string): { records: LedgerLine[]; skipped: number } {
	const records: LedgerLine[] = [];
	let skipped = 0;
	for (const line of text.split('\n')) {
		if (!line.trim()) continue;
		try {
			records.push(JSON.parse(line));
		} catch {
			// The corrupt-store contract: ONE bad line costs one line, never the file.
			// The count is surfaced, never swallowed.
			skipped++;
		}
	}
	return { records, skipped };
}

/**
 * The link-life fold. Absolute `n` + max means this is idempotent BY ARITHMETIC, not by
 * rule — a duplicated region, a re-appended restored copy, or a "keep both" merge all fold
 * to the same answer. Later decisions win; `n` never ratchets down.
 */
export function foldLinkLife(records: LedgerLine[]): Map<string, LedgerLine> {
	const out = new Map<string, LedgerLine>();
	for (const r of records) {
		const key = String(r.k ?? '');
		if (!key) continue;
		const prev = out.get(key);
		if (!prev) {
			out.set(key, { ...r });
			continue;
		}
		const merged: LedgerLine = { ...prev, ...r };
		// `n` is written ABSOLUTE, so the fold is max — never a sum, never a decrement.
		const pn = Number(prev.n ?? 0);
		const rn = Number(r.n ?? 0);
		merged.n = Math.max(pn, rn);
		out.set(key, merged);
	}
	return out;
}

/**
 * Normalize a path for a ledger KEY per Q5: relative to the Universe root, forward slashes,
 * NFC. Never absolute, never a drive letter — those do not survive the Universe moving
 * between machines or operating systems.
 */
export function ledgerKeyPath(universeRoot: string, absPath: string): string {
	const norm = (s: string) => s.replace(/\\/g, '/');
	const root = norm(universeRoot).replace(/\/+$/, '');
	const p = norm(absPath);
	const rel = p.toLowerCase().startsWith(root.toLowerCase() + '/') ? p.slice(root.length + 1) : p;
	return rel.normalize('NFC');
}
