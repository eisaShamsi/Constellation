/**
 * MIG-093 §C — the Quick Switcher's pure ranking module.
 *
 * Concept (the horse): Ctrl+O answers "take me to the note I can name" —
 * titles + aliases only, in-memory, zero IPC per keystroke.
 *
 * The banded scoring model is the researched industry pattern (workflow
 * `wf_7786efda-5db`): VS Code's fuzzyScorer reserves hard score BANDS so no
 * fuzzy hit can ever outrank an exact title; fzf/fzy score subsequences with
 * position bonuses + gap penalties. Bands here:
 *
 *   EXACT (1<<18)  — folded query === folded title/alias
 *   PREFIX (1<<17) — folded title starts with the query (+ shortness boost,
 *                    so "islam" ranks the note "Islam" above "Islamic Art")
 *   WORDB (1<<16)  — the query matches starting at a word boundary
 *                    ("art" → "Islamic Art"); multi-word queries land here
 *                    when every word matches (all-words-must-match, VS Code
 *                    semantics)
 *   FUZZY (0)      — subsequence match with fzf-style bonuses; includes
 *                    plain mid-word substrings (a consecutive run scores
 *                    near the tier's top). Skipped for 1-2-char queries
 *                    (noise guard) and for multi-word queries.
 *
 * Tie-breakers within a band: recency → match compactness (span) → shorter
 * title → locale-aware alphabetical. Alias hits carry a small within-band
 * penalty so an equal title hit edges them.
 *
 * Fold-for-matching, display-raw: candidates carry a pre-folded string
 * (computed ONCE per cache refresh — never per keystroke).
 */
import { foldForMatch } from './searchFold';

export interface SwitcherCandidate {
	name: string;
	path: string;
	libraryName: string;
	/** foldForMatch(name), precomputed by the host once per cache refresh. */
	folded: string;
	/** Set when this candidate row represents an alias of the note. */
	alias?: string;
	/** foldForMatch(alias), precomputed. */
	aliasFolded?: string;
}

export interface RankedHit {
	candidate: SwitcherCandidate;
	score: number;
	/** first..last matched char span in the folded text (compactness tie-break). */
	span: number;
}

export const BAND_EXACT = 1 << 18;
export const BAND_PREFIX = 1 << 17;
export const BAND_WORDB = 1 << 16;
const ALIAS_PENALTY = 8;

/** Word boundary = start, or the previous char is not a letter/digit. */
function isSep(ch: string): boolean {
	return !/[\p{L}\p{N}]/u.test(ch);
}

/** fzf-style subsequence score: position bonuses + consecutive-run bonus +
 *  capped gap penalties. Null when q is not a subsequence of s. */
function fuzzyScore(q: string, s: string): { score: number; span: number } | null {
	let si = 0;
	let score = 0;
	let first = -1;
	let last = -1;
	let run = 0;
	for (const ch of q) {
		const idx = s.indexOf(ch, si);
		if (idx < 0) return null;
		if (first < 0) first = idx;
		let bonus = 2; // base per-match
		if (idx === 0) bonus += 8;
		else if (isSep(s[idx - 1])) bonus += 6;
		if (last >= 0 && idx === last + 1) {
			run += 1;
			bonus += Math.min(run, 3) * 4; // consecutive-run bonus, capped
		} else {
			if (last >= 0) score -= Math.min(idx - last - 1, 5); // capped gap penalty
			run = 0;
		}
		score += bonus;
		last = idx;
		si = idx + 1;
	}
	return { score: Math.max(score, 1), span: last - first + 1 };
}

/** Score one folded text against the (single-word) folded query. */
function scoreOne(q: string, folded: string): { score: number; span: number } | null {
	if (folded === q) return { score: BAND_EXACT, span: q.length };
	if (folded.startsWith(q)) {
		// Shortness boost: the more of the title the query covers, the higher.
		return { score: BAND_PREFIX + Math.round((q.length / folded.length) * 100), span: q.length };
	}
	// Word-boundary start ("art" → "islamic art").
	let from = 1;
	while (from < folded.length) {
		const idx = folded.indexOf(q, from);
		if (idx < 0) break;
		if (isSep(folded[idx - 1])) {
			return { score: BAND_WORDB + Math.max(0, 60 - idx), span: q.length };
		}
		from = idx + 1;
	}
	// Fuzzy subsequence (covers mid-word substrings too) — 3+ char queries only.
	if (q.length >= 3) return fuzzyScore(q, folded);
	return null;
}

/** Multi-word: every word must match; boundary-anchored words score higher. */
function scoreMulti(words: string[], folded: string): { score: number; span: number } | null {
	let score = 0;
	let lo = Infinity;
	let hi = -1;
	let allBoundary = true;
	for (const w of words) {
		const idx = folded.indexOf(w);
		if (idx < 0) return null;
		const boundary = idx === 0 || isSep(folded[idx - 1]);
		if (!boundary) allBoundary = false;
		score += boundary ? 60 : 25;
		lo = Math.min(lo, idx);
		hi = Math.max(hi, idx + w.length);
	}
	// All words at boundaries → the WORDB band; otherwise below it, but the
	// all-words-must-match guarantee already filters hard.
	const band = allBoundary ? BAND_WORDB : 0;
	return { score: band + score + Math.round((words.join('').length / folded.length) * 20), span: hi - lo };
}

/**
 * Rank candidates against a raw query. `recencyIndex` maps path → recency
 * position (0 = most recent); missing paths get no recency boost.
 */
export function rankSwitcher(
	rawQuery: string,
	candidates: SwitcherCandidate[],
	opts?: { recencyIndex?: Map<string, number>; limit?: number; collator?: Intl.Collator }
): RankedHit[] {
	const q = foldForMatch(rawQuery.trim());
	if (!q) return [];
	const words = q.split(/\s+/).filter(Boolean);
	const multi = words.length > 1;
	const limit = opts?.limit ?? 50;
	const recency = opts?.recencyIndex;
	const collator = opts?.collator ?? new Intl.Collator(undefined, { sensitivity: 'base' });

	const hits: RankedHit[] = [];
	for (const c of candidates) {
		const folded = c.alias ? (c.aliasFolded ?? '') : c.folded;
		if (!folded) continue;
		const m = multi ? scoreMulti(words, folded) : scoreOne(q, folded);
		if (!m) continue;
		const score = c.alias ? m.score - ALIAS_PENALTY : m.score;
		hits.push({ candidate: c, score, span: m.span });
	}

	hits.sort((a, b) => {
		if (b.score !== a.score) return b.score - a.score;
		const ra = recency?.get(a.candidate.path) ?? Infinity;
		const rb = recency?.get(b.candidate.path) ?? Infinity;
		if (ra !== rb) return ra - rb;
		if (a.span !== b.span) return a.span - b.span;
		const la = a.candidate.folded.length;
		const lb = b.candidate.folded.length;
		if (la !== lb) return la - lb;
		return collator.compare(a.candidate.name, b.candidate.name);
	});

	// One row per note: an alias hit and a title hit for the same path keep
	// only the best-scoring row (sorted order makes the first the best).
	const seen = new Set<string>();
	const out: RankedHit[] = [];
	for (const h of hits) {
		if (seen.has(h.candidate.path)) continue;
		seen.add(h.candidate.path);
		out.push(h);
		if (out.length >= limit) break;
	}
	return out;
}

/** True when some candidate's folded title/alias equals the folded query —
 *  drives the "Create note" pinned row's visibility. */
export function hasExactMatch(rawQuery: string, candidates: SwitcherCandidate[]): boolean {
	const q = foldForMatch(rawQuery.trim());
	if (!q) return false;
	return candidates.some(c => (c.alias ? c.aliasFolded === q : c.folded === q));
}
