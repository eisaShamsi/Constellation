/**
 * MIG-093 §C — the Quick Switcher ranking model (pinned).
 *
 * The Boss-reproduced failure this pins forever: query `islam` on a corpus
 * full of "X in Islam" titles MUST rank the note titled exactly "Islam" #1
 * (the old switcher filled its cap in cache order and buried/dropped it).
 * Bands: exact > prefix > word-boundary > fuzzy — no fuzzy hit may ever
 * outrank an exact title (the VS Code invariant).
 */
import { describe, it, expect } from 'vitest';
import { rankSwitcher, hasExactMatch, BAND_EXACT, BAND_PREFIX, BAND_WORDB, type SwitcherCandidate } from '$lib/switcherRank';
import { foldForMatch } from '$lib/searchFold';

function cand(name: string, path?: string, alias?: string): SwitcherCandidate {
	return {
		name,
		path: path ?? `${name}.md`,
		libraryName: 'Lib',
		folded: foldForMatch(name),
		...(alias ? { alias, aliasFolded: foldForMatch(alias) } : {}),
	};
}

describe('MIG-093 §C — the pinned Boss case', () => {
	it('query "islam": exact title "Islam" ranks #1, above every "X in Islam"', () => {
		const corpus = [
			cand('Abraham in Islam'),
			cand('Adam in Islam'),
			cand('Ansar (Islam)'),
			cand('Astronomy in the medieval Islamic world'),
			cand('Islam'),
			cand('Islamic Art'),
			cand('Christian influences on the Islamic world'),
		];
		const hits = rankSwitcher('islam', corpus);
		expect(hits[0].candidate.name).toBe('Islam');
		// prefix beats word-boundary: "Islamic Art" above "Abraham in Islam"
		expect(hits[1].candidate.name).toBe('Islamic Art');
		expect(hits.map(h => h.candidate.name)).toContain('Abraham in Islam');
	});
});

describe('MIG-093 §C — bands', () => {
	it('exact > prefix > word-boundary > fuzzy, structurally', () => {
		const corpus = [cand('Knowledge'), cand('Knowledge Graph'), cand('Tacit Knowledge'), cand('Klnowledgey xyz')];
		const hits = rankSwitcher('knowledge', corpus);
		expect(hits[0].candidate.name).toBe('Knowledge'); // exact
		expect(hits[0].score).toBeGreaterThanOrEqual(BAND_EXACT);
		expect(hits[1].candidate.name).toBe('Knowledge Graph'); // prefix
		expect(hits[1].score).toBeGreaterThanOrEqual(BAND_PREFIX);
		expect(hits[2].candidate.name).toBe('Tacit Knowledge'); // word boundary
		expect(hits[2].score).toBeGreaterThanOrEqual(BAND_WORDB);
	});

	it('prefix shortness boost: "Window" above "Window Actions Reference"', () => {
		const hits = rankSwitcher('window', [cand('Window Actions Reference'), cand('Windows')]);
		expect(hits[0].candidate.name).toBe('Windows'); // shorter prefix candidate wins
	});

	it('1-2 char queries skip fuzzy (prefix/boundary only)', () => {
		const corpus = [cand('AI'), cand('Brain'), cand('xAy')];
		const hits = rankSwitcher('ai', corpus);
		expect(hits.map(h => h.candidate.name)).toEqual(['AI']); // no fuzzy 'xAy', no 'brAIn'... (Brain has "ai" mid-word: substring ≠ boundary → excluded)
	});

	it('multi-word: all words must match; boundary-anchored ranks in the WORDB band', () => {
		const corpus = [cand('Islamic Art History'), cand('Islamic Pottery'), cand('History of Art')];
		const hits = rankSwitcher('islamic art', corpus);
		expect(hits.map(h => h.candidate.name)).toEqual(['Islamic Art History']);
		expect(hits[0].score).toBeGreaterThanOrEqual(BAND_WORDB);
	});
});

describe('MIG-093 §C — Arabic (folded matching)', () => {
	it('query without diacritics/hamza matches a decorated title', () => {
		const corpus = [cand('المَعْرِفَة العربية'), cand('إسلام')];
		expect(rankSwitcher('المعرفة', corpus)[0].candidate.name).toBe('المَعْرِفَة العربية');
		expect(rankSwitcher('اسلام', corpus)[0].candidate.name).toBe('إسلام');
	});
});

describe('MIG-093 §C — aliases + tie-breakers + dedupe', () => {
	it('an alias hit surfaces its note, but an equal title hit edges it', () => {
		const corpus = [cand('Epistemology', 'e.md'), cand('Theory of Knowledge', 'tok.md', 'Epistemology')];
		const hits = rankSwitcher('epistemology', corpus);
		expect(hits[0].candidate.path).toBe('e.md'); // title beats alias at the same tier
		expect(hits[1].candidate.path).toBe('tok.md');
		expect(hits[1].candidate.alias).toBe('Epistemology');
	});

	it('one row per note even when both title and alias match', () => {
		const corpus = [cand('Islam', 'i.md'), cand('Islam', 'i.md', 'Islamic faith')];
		const hits = rankSwitcher('islam', corpus);
		expect(hits.filter(h => h.candidate.path === 'i.md')).toHaveLength(1);
	});

	it('recency breaks ties within a band', () => {
		const corpus = [cand('Note Alpha', 'a.md'), cand('Note Beta', 'b.md')];
		const hits = rankSwitcher('note', corpus, { recencyIndex: new Map([['b.md', 0]]) });
		expect(hits[0].candidate.path).toBe('b.md');
	});

	it('caps at limit', () => {
		const corpus = Array.from({ length: 80 }, (_, i) => cand(`Islam topic ${i}`, `t${i}.md`));
		expect(rankSwitcher('islam', corpus, { limit: 50 })).toHaveLength(50);
	});
});

describe('MIG-093 §C — hasExactMatch (the Create-row gate)', () => {
	it('true on folded equality, false otherwise', () => {
		const corpus = [cand('إسلام')];
		expect(hasExactMatch('اسلام', corpus)).toBe(true);
		expect(hasExactMatch('اسلا', corpus)).toBe(false);
	});
});
