/**
 * PJ-068 v2 — shared data layer for the second-screen note-graph lenses.
 *
 * The Butterfly and the Ledger (and any future lens) ride ONE chassis: same typed-link
 * grouping, same note-statistics derivation, same palette. This module is the single source
 * of that logic so the lenses stay in lock-step (one truth, tested once).
 *
 * COLOUR (Boss ruling 2026-07-10 — "registry base + Style Setter override", the same
 * unify-on-demand pattern as MIG-088's Cognitive colours): the **Link Types registry** is the
 * base, so a `supports` wedge is the same colour as a `supports` pill and recolouring the
 * vocabulary once moves every surface. A Style-Setter `--rel-<id>` var overrides the graph
 * alone, on demand. The lenses previously carried their own Flexoki palette — a second palette
 * for the same eight relationships, which disagreed with the pills (supports was blue as a pill
 * and green as a wedge). That drift is gone.
 *
 * The registry is a PER-WINDOW cache seeded from the main window's boot bundle, so the second
 * screen must `loadLinkTypes()` itself (SecondScreenPage) or every wedge renders neutral grey.
 *
 * Structural (parent/TOC) links never reach here: `get_backlink_rows` / `get_outgoing_rows`
 * exclude that lane in Rust (PJ-065), keeping the graph a purely cognitive surface.
 */
import { parseFrontmatter } from '$lib/libraries/store';
import { linkTypeColor, linkTypeRank, isNullLinkType, getLinkType } from '$lib/libraries/linkTypeRegistry';
import { tIn } from '$lib/i18n';

/** The null/default relationship — an untyped link. Not a registry type; always sorts last. */
export const NULL_TYPE = 'associative';
const NULL_RANK = 1e6;

/** Registry colour as the base, a Style-Setter `--rel-<id>` var as the override. Returns a CSS
 *  value valid in an SVG `fill`/`stroke`, so no per-theme branching is needed. */
export const relColor = (id: string) => `var(--rel-${id}, ${linkTypeColor(id)})`;

/** The type's display name IN THE UI LANGUAGE. Same resolver as LinkTypePill so the wedge, the row
 *  and the pill always read the same word: the `linkTypes.<id>` translation, else the registry's
 *  user-given label (a custom type), else the raw id. Pass `$locale` from the component so the
 *  label re-renders when the language changes. */
export const relLabelIn = (loc: string, id: string) => {
	const key = `linkTypes.${id.toLowerCase()}`;
	const tr = tIn(loc, key);
	return tr !== key ? tr : (getLinkType(id)?.label ?? id);
};

/** A localized value from a fixed vocabulary (stage / maturity), falling back to the raw value. */
export const vocabIn = (loc: string, prefix: string, value: string) => {
	if (!value) return '';
	const key = `${prefix}.${value.toLowerCase()}`;
	const tr = tIn(loc, key);
	return tr !== key ? tr : value;
};

const TIERW: Record<string, number> = { 'load-bearing': 1, established: 0.7, emerging: 0.42, stale: 0.2 };
export const tierW = (t?: string) => TIERW[(t || 'emerging').toLowerCase()] ?? 0.42;
export const clean = (n?: string) => (n || '').replace(/\.md$/, '');

export const STAGES = ['spark', 'birth', 'growth', 'maturity', 'dormancy', 'archival'];
export const MATS = ['seed', 'sapling', 'evergreen', 'canonical'];
export const CONF = ['hypothesis', 'evidence', 'established', 'contested'];
/** Confidence is NOT a link type — it has its own shared cognitive vars (the Style Setter's
 *  "Cognitive colours → Confidence" category, MIG-088). Borrowing --rel-supports for
 *  "established" made a confidence level change colour when you recoloured a relationship. */
export const CONF_COLOR: Record<string, string> = {
	hypothesis: 'var(--confidence-hypothesis, var(--text-faint, #9ca3af))',
	evidence: 'var(--confidence-evidence, #4385BE)',
	established: 'var(--confidence-established, #879A39)',
	contested: 'var(--confidence-contested, #D14D41)',
};

export interface GLink {
	name?: string; target?: string; path?: string; libraryName?: string;
	linkType?: string; tier?: string; confidence?: string; annotation?: string;
	traversalCount?: number; lastTraversed?: string; weight?: number;
}

/** The Orrery's six recency orbits, inner (recent/warm) → outer (cold). A link with
 *  traversalCount === 0 was NEVER actually walked (its lastTraversed is only its birth-seeded
 *  timestamp), so it pins to the outermost NEVER shell regardless of age — the reliable
 *  cold-by-definition sentinel. Otherwise the shell is the age of its last traversal. */
export const RECENCY_SHELLS = [
	{ key: 'today', maxDays: 1 }, { key: 'week', maxDays: 7 }, { key: 'month', maxDays: 30 },
	{ key: 'quarter', maxDays: 90 }, { key: 'older', maxDays: Infinity }, { key: 'never', maxDays: -1 },
] as const;
export const NEVER_SHELL = 5;

export function recencyShell(lk: GLink, nowMs: number): number {
	if (!lk.traversalCount) return NEVER_SHELL;              // never actually walked → cold rim
	const t = lk.lastTraversed ? Date.parse(lk.lastTraversed) : NaN;
	if (!isFinite(t)) return NEVER_SHELL;
	const days = (nowMs - t) / 86_400_000;
	for (let i = 0; i < 5; i++) if (days <= RECENCY_SHELLS[i].maxDays) return i;
	return 4;                                                // OLDER (>90d but genuinely walked)
}

/** Earned weight — logarithmic in traversals (the app's own growth curve). Falls back to the
 *  lifecycle tier when a raw count is absent, so a load-bearing link always reads as heavy. */
export function earnedWeight(lk: GLink): number {
	if (typeof lk.weight === 'number' && lk.weight > 0) return lk.weight;
	const c = lk.traversalCount ?? 0;
	return c > 0 ? 1 + Math.log(1 + c) : 0.5 + tierW(lk.tier);
}

/** Keep a link's own type id — including a user's CUSTOM type, which used to collapse into
 *  "associative" (mislabelled and miscoloured). Only genuinely null/untyped links become NULL_TYPE. */
export const normalizeType = (t?: string): string => {
	const x = (t || '').toLowerCase();
	return isNullLinkType(x) ? NULL_TYPE : x;
};

/** Order the types present on a note by the registry's own canonical rank; unknown ids sort after
 *  the known ones, and the untyped bucket always sits last. Never drops a type (and so never a link). */
export function orderTypes(present: string[]): string[] {
	const rank = (id: string) => (id === NULL_TYPE ? NULL_RANK : linkTypeRank(id));
	return present.slice().sort((a, b) => rank(a) - rank(b) || a.localeCompare(b));
}

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
	const typeMix = orderTypes(Object.keys(mix)).map((t) => ({ type: t, count: mix[t], color: relColor(t) }));
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
