/**
 * PJ-068 v2 — shared data layer for the second-screen note-graph lenses.
 *
 * The Butterfly and the Ledger (and any future lens) ride ONE chassis: same typed-link
 * grouping, same note-statistics derivation, same relationship palette. This module is
 * the single source of that logic so the lenses stay in lock-step (one truth, tested once).
 * Colours are Flexoki defaults exposed as --rel-* CSS vars (Style-Setter controlled).
 */
import { parseFrontmatter } from '$lib/libraries/store';

export const REL_ORDER = [
	'supports', 'contradicts', 'causes', 'exemplifies', 'generalizes', 'derives-from', 'part-of', 'supersedes', 'associative',
] as const;
export type RelType = (typeof REL_ORDER)[number];

export const REL_DEFAULT: Record<string, string> = {
	supports: '#879A39', contradicts: '#D14D41', causes: '#DA702C', exemplifies: '#3AA99F',
	generalizes: '#4385BE', 'derives-from': '#8B7EC8', 'part-of': '#D0A215', supersedes: '#CE5D97', associative: '#B7B5AC',
};
export const relColor = (t: string) => `var(--rel-${t}, ${REL_DEFAULT[t] || REL_DEFAULT.associative})`;

const TIERW: Record<string, number> = { 'load-bearing': 1, established: 0.7, emerging: 0.42, stale: 0.2 };
export const tierW = (t?: string) => TIERW[(t || 'emerging').toLowerCase()] ?? 0.42;
export const clean = (n?: string) => (n || '').replace(/\.md$/, '');

export const STAGES = ['spark', 'birth', 'growth', 'maturity', 'dormancy', 'archival'];
export const MATS = ['seed', 'sapling', 'evergreen', 'canonical'];
export const CONF = ['hypothesis', 'evidence', 'established', 'contested'];
export const CONF_COLOR: Record<string, string> = {
	hypothesis: 'var(--text-faint, #9ca3af)', evidence: 'var(--rel-generalizes, #4385BE)',
	established: 'var(--rel-supports, #879A39)', contested: 'var(--rel-contradicts, #D14D41)',
};

export interface GLink {
	name?: string; target?: string; path?: string; libraryName?: string;
	linkType?: string; tier?: string; confidence?: string; annotation?: string; traversalCount?: number;
}

export const normalizeType = (t?: string): RelType => {
	const x = (t || 'associative').toLowerCase();
	return (REL_DEFAULT[x] ? x : 'associative') as RelType;
};

export function groupByType(items: GLink[]): Record<string, GLink[]> {
	const g: Record<string, GLink[]> = {};
	for (const it of items) { const t = normalizeType(it.linkType); (g[t] = g[t] || []).push(it); }
	return g;
}

/** Every note statistic the lenses + the gauge deck read — from frontmatter, the review
 *  IPC, and the resolved link rows. Rule-8 clean: no re-walk, just what's already in hand. */
export function deriveStats(content: string, review: any, backlinks: GLink[], outgoing: GLink[]) {
	const fm = parseFrontmatter(content || '');
	const propOf = (key: string) => fm.properties.find((p: any) => p.key.toLowerCase() === key.toLowerCase())?.value;
	const stage = String(propOf('stage') ?? '');
	const stratum = String(propOf('stratum') ?? '');
	const provenance = String(propOf('provenance') ?? '');
	const srcRaw = propOf('source') ?? propOf('sources');
	const source = Array.isArray(srcRaw) ? srcRaw.join(', ') : String(srcRaw ?? '');
	const tRaw = propOf('tags');
	const tags = (Array.isArray(tRaw) ? tRaw : (tRaw ? [String(tRaw)] : [])) as string[];
	const cid = String(propOf('cid_cn') ?? '');
	let created = '';
	if (/^\d{8}T/.test(cid)) created = cid.slice(0, 4) + '-' + cid.slice(4, 6) + '-' + cid.slice(6, 8);
	else { const d = propOf('created') ?? propOf('date'); created = d ? String(d).slice(0, 10) : ''; }

	const maturity = String(review?.maturity ?? '');
	const wordCount = (review?.word_count ?? null) as number | null;
	let reviewState: { key: string; sev: string } = { key: '', sev: '' };
	if (review) {
		if (review.is_stale) reviewState = { key: 'stale', sev: 'bad' };
		else if (review.never_reviewed) reviewState = { key: 'never', sev: 'mut' };
		else if ((review.days_overdue ?? 0) > 0) reviewState = { key: 'due', sev: 'warn' };
		else reviewState = { key: 'upToDate', sev: 'ok' };
	}

	const allLinks = [...backlinks, ...outgoing];
	const stageIdx = STAGES.indexOf(stage.toLowerCase());
	const matIdx = MATS.indexOf(maturity.toLowerCase());
	const mix: Record<string, number> = {};
	for (const l of allLinks) { const t = normalizeType(l.linkType); mix[t] = (mix[t] || 0) + 1; }
	const typeMix = REL_ORDER.filter((t) => mix[t]).map((t) => ({ type: t, count: mix[t], color: relColor(t) }));
	const supportsN = allLinks.filter((l) => normalizeType(l.linkType) === 'supports').length;
	const contradictsN = allLinks.filter((l) => normalizeType(l.linkType) === 'contradicts').length;
	const confMix = CONF.map((c) => ({ c, n: allLinks.filter((l) => (l.confidence || 'hypothesis') === c).length, color: CONF_COLOR[c] }));
	const loadBearing = allLinks.filter((l) => (l.tier || '') === 'load-bearing').length;
	const dominantConf = confMix.slice().sort((a, b) => b.n - a.n)[0]?.c || '';

	return {
		stage, stratum, provenance, source, tags, created, maturity, wordCount, reviewState,
		stageIdx, matIdx, typeMix, supportsN, contradictsN, confMix, loadBearing, dominantConf,
		totalIn: backlinks.length, totalOut: outgoing.length, totalLinks: allLinks.length,
	};
}

export type NoteStats = ReturnType<typeof deriveStats>;
