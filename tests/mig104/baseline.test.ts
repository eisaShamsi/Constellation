/**
 * MIG-104 Slice 0 — the harness's own proof. No product code exists yet; these tests pin the
 * CONTRACTS every later slice is built against, so a later slice cannot quietly redefine them.
 *
 * Glob-driven vitest (PJ-157) collects this automatically — no registration step.
 */
import { describe, it, expect } from 'vitest';
import { encodeLine, decodeLines, foldLinkLife, ledgerKeyPath, LEDGER } from './harness';

describe('MIG-104 ledger encoding — the Q5 OS-portability contract', () => {
	it('writes exactly one LF-terminated line, no CRLF and no BOM', () => {
		const line = encodeLine({ k: 'C1>C2', n: 3 });
		expect(line.endsWith('\n')).toBe(true);
		expect(line).not.toContain('\r');
		expect(line.charCodeAt(0)).not.toBe(0xfeff);
		expect(line.split('\n').filter(Boolean)).toHaveLength(1);
	});

	it('round-trips byte-exactly, which is what portability across Windows and macOS means', () => {
		const rec = { k: 'C1>C2', n: 7, conf: 'evidence', at: 1785131672 };
		const { records, skipped } = decodeLines(encodeLine(rec));
		expect(skipped).toBe(0);
		expect(records[0]).toEqual(rec);
	});

	it('keys on a Universe-relative forward-slashed NFC path — never a drive letter', () => {
		const key = ledgerKeyPath(
			'E:\\Constellation Universes\\Eisa Cognitive Knowledge',
			'E:\\Constellation Universes\\Eisa Cognitive Knowledge\\Daily Notes\\2026-06-17.md',
		);
		expect(key).toBe('Daily Notes/2026-06-17.md');
		expect(key).not.toMatch(/^[A-Za-z]:/);
		expect(key).not.toContain('\\');
		expect(key.normalize('NFC')).toBe(key);
	});
});

describe('MIG-104 corrupt-store contract', () => {
	it('one unparseable line costs ONE line and is COUNTED — never the whole file', () => {
		const text = encodeLine({ k: 'a', n: 1 }) + '{ this is a torn tail\n' + encodeLine({ k: 'b', n: 2 });
		const { records, skipped } = decodeLines(text);
		expect(records).toHaveLength(2);
		expect(skipped).toBe(1);
	});

	it('a torn final line (killed mid-append) loses only that line', () => {
		const text = encodeLine({ k: 'a', n: 1 }) + '{"k":"b","n":';
		const { records, skipped } = decodeLines(text);
		expect(records).toHaveLength(1);
		expect(skipped).toBe(1);
	});
});

describe('MIG-104 link-life fold — idempotent by arithmetic, not by rule', () => {
	it('folds an absolute count to the MAX, so replay can never ratchet it down', () => {
		const recs = [{ k: 'L', n: 5 }, { k: 'L', n: 2 }, { k: 'L', n: 9 }];
		expect(foldLinkLife(recs).get('L')!.n).toBe(9);
	});

	it('is order-independent (commutative) — a merge cannot change the answer', () => {
		const a = [{ k: 'L', n: 3 }, { k: 'L', n: 8 }];
		const b = [{ k: 'L', n: 8 }, { k: 'L', n: 3 }];
		expect(foldLinkLife(a).get('L')).toEqual(foldLinkLife(b).get('L'));
	});

	it('is idempotent — a duplicated region folds to the same answer as one copy', () => {
		const once = [{ k: 'L', n: 4, conf: 'evidence' }];
		const twice = [...once, ...once];
		expect(foldLinkLife(twice).get('L')).toEqual(foldLinkLife(once).get('L'));
	});

	it('lets a LATER decision win while the count still folds to max', () => {
		const folded = foldLinkLife([
			{ k: 'L', n: 9, status: 'active' },
			{ k: 'L', n: 4, status: 'archived' },
		]).get('L')!;
		expect(folded.status).toBe('archived');
		expect(folded.n).toBe(9);
	});
});

describe('MIG-104 note-history — the stream that must NEVER fold', () => {
	it('keeps every event, because the record IS the payload', () => {
		// The live shape (hid 8251/8252/8253): a property being typed, one character at a time.
		// Folding by note identity would collapse a thought into a single keystroke.
		const typing = [
			{ hid: 8251, cid: 'C1', to: 'ma' },
			{ hid: 8252, cid: 'C1', to: 'mas' },
			{ hid: 8253, cid: 'C1', to: 'masadir' },
		];
		const { records } = decodeLines(typing.map(encodeLine).join(''));
		expect(records).toHaveLength(3);
		// If anyone ever routes this stream through the link-life fold, the loss is visible:
		expect(foldLinkLife(typing.map((t) => ({ ...t, k: t.cid }))).size).toBe(1);
	});

	it('orders by the row ordinal, never by timestamp (765 timestamp groups collide live)', () => {
		const rows = [
			{ hid: 3, cid: 'C1', at: 1785131672 },
			{ hid: 1, cid: 'C1', at: 1785131672 },
			{ hid: 2, cid: 'C1', at: 1785131672 },
		];
		const ordered = [...rows].sort((a, b) => a.hid - b.hid).map((r) => r.hid);
		expect(ordered).toEqual([1, 2, 3]);
		const byTime = [...rows].sort((a, b) => a.at - b.at).map((r) => r.hid);
		expect(byTime).not.toEqual([1, 2, 3]); // identical timestamps preserve input order
	});
});

describe('MIG-104 file names are fixed by the Plan', () => {
	it('names the three ledger files', () => {
		expect(LEDGER.tail).toBe('earned.jsonl');
		expect(LEDGER.snapshot).toBe('earned.snapshot.jsonl');
		expect(LEDGER.history).toBe('note-history.jsonl');
	});
});
