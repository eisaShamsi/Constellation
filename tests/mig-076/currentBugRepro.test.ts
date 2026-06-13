/**
 * MIG-076 §C — CHARACTERIZATION of the CURRENT (pre-cure) defects, reproduced
 * against the REAL exported store primitives. These pass today by DEMONSTRATING
 * the broken behavior (green = defect confirmed present), pinning the diagnosis
 * the SME audit produced. They are DELETED when single content ownership lands
 * and removes the structures they document.
 *
 * Each maps to a named 2026-06-12 failure (audit Agent 2, file:line).
 */
import { describe, it, expect } from 'vitest';
import {
	setWriteAhead, getWriteAhead, clearWriteAhead,
	buildFullContent, parseFrontmatter,
} from '$lib/libraries/store';

describe('symptom 2 root — the write-ahead buffer structurally cannot tell stale from fresh', () => {
	it('a WAB entry carries NO freshness marker, so a stale snapshot looks valid on restore', () => {
		setWriteAhead('/repro/freshness.md', 'snapshot taken at focus-enter', 0, 0);
		const e = getWriteAhead('/repro/freshness.md');
		clearWriteAhead('/repro/freshness.md');
		expect(e).toBeDefined();
		// The defect: the entry has only {content, cursorPos, scrollTop} — no
		// version/mtime. resolveNoteContent therefore restores on identity (cid)
		// alone, with no way to reject an OLDER snapshot — the symptom-2 mechanism.
		const keys = Object.keys(e!);
		expect(keys).not.toContain('version');
		expect(keys).not.toContain('mtime');
		expect(keys.sort()).toEqual(['content', 'cursorPos', 'scrollTop']);
	});
});

describe('the landmine — composition from two sources has no identity guard', () => {
	it('frontmatter of note A + body of note B silently produces a frankenstein file', () => {
		const a = parseFrontmatter('---\ntitle: A\ncid_cn: NOTE_A\n---\nA body');
		const b = parseFrontmatter('---\ntitle: B\ncid_cn: NOTE_B\n---\nB body');
		// What the in-focus tab-switch teardown does today: compose mismatched halves.
		// buildFullContent has no identity parameter — nothing can refuse this.
		const frankenstein = buildFullContent(a.properties, b.body);
		expect(frankenstein).toContain('cid_cn: NOTE_A'); // says it's note A …
		expect(frankenstein).toContain('B body'); // … but carries note B's body — corruption
	});
});

describe('the §D title-rename poison — the unguarded flush has no path identity (RED, pre-cure)', () => {
	it('a stale flush for the renamed-away OLD path cannot be refused by the composition layer', () => {
		// Pre-cure mechanism behind BUG-023: after B is title-renamed (/b.md →
		// /B-renamed.md), a torn-down editor for B's OLD path composes via
		// buildFullContent — which takes NO target-path / identity parameter, so
		// NOTHING can refuse a write to the renamed-away old path (a resurrected
		// ghost) or to A's path (cross-identity). Under single ownership,
		// noteSession.save(id, oldPath) REFUSES exactly this — proven GREEN in
		// runtimeHarness Recipe H. This test pins the hole the guard fills.
		const b = parseFrontmatter('---\ntitle: B\ncid_cn: NOTE_B\n---\nB content');
		const ghost = buildFullContent(b.properties, b.body);
		// The composition is happy to be written to ANY path the stale caller still
		// holds — the layer is identity-blind. (§C's compose adds the missing guard.)
		expect(ghost).toContain('cid_cn: NOTE_B');
		expect(ghost).toContain('B content');
	});
});
