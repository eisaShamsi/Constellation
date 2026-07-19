/**
 * PJ-114 Phase 1 §3b — the ONE source of truth for how a living link's *state*
 * is put into words.
 *
 * Concept (the horse): a link's accumulated traversal is evidence of how much a
 * connection has actually carried thought. The `×N` chip beside the type pills is
 * where the user reads that evidence — so it must speak the user's language, from
 * one place, and never leak an internal token.
 *
 * Before this module the two link panels each carried a byte-identical private
 * `fmtTraversed` that hardcoded English ('today', '3d ago'), and each built the
 * chip tooltip from a hardcoded English template. Three copies of the same words,
 * none of them translated.
 *
 * VOCABULARY IS REUSED, NOT RE-INVENTED. Both halves of the tooltip already
 * existed, fully translated in all 15 locales, and are already on screen in the
 * CCS panel (`CCSView.svelte:268/281` builds a tooltip of this exact shape):
 *   - the traversal count  → `plurals.walks`  (CLDR plural-aware; "walk" is this
 *                            app's user-facing word for a traversal)
 *   - the lifecycle tier   → `ccs.tier.*`     (fresh/emerging/established/
 *                            loadBearing/stale, all 15 locales)
 * Adding a second translation of either would have guaranteed drift — the CCS
 * panel calling a traversal "عبور" while this chip called it something else for
 * the identical event. So this module adds ZERO new i18n keys.
 *
 * The relative time uses `Intl.RelativeTimeFormat` rather than hand-written
 * strings, matching the house precedent set by `Intl.PluralRules` in the i18n
 * core (WA#5 — the battle-tested standard, not hand-rolled).
 */
import { tIn, tnIn } from '$lib/i18n';
import { LINK_STALE_DAYS, type LinkLifecycle } from '$lib/libraries/store';
// The link-type name resolver. `relLabelIn` is the THREE-branch form
// (`linkTypes.<id>` translation → the registry's user-given label for a custom
// type → the raw id) and is already shared by the three cockpit lenses and
// `LinkTypePill`. The panels' private copies were the weaker two-branch form,
// which is why a custom link type rendered its label in the pill and its raw
// slug in the annotation ON THE SAME ROW. Importing the existing resolver fixes
// that divergence without minting a fourth copy.
// NOTE: `relLabelIn`'s canonical home should migrate here once §7–§10 make
// `$lib/links/` the real home of the living-link layer; re-pointing the three
// Boss-validated cockpit lenses is not this step's business.
import { relLabelIn } from '$lib/cockpitGraphData';

const MS_PER_DAY = 86_400_000;

/**
 * `Intl.RelativeTimeFormat` instances are expensive to construct relative to how
 * often this runs, so they are cached at MODULE scope, per (locale, numeric mode).
 *
 * This cache is load-bearing, not a nicety: `VirtualList` keys its each-block by
 * SLOT index, so every visible row's snippet re-runs on every scroll tick (~45–55
 * rows per event on a hub note). Constructing a formatter per call would be
 * milliseconds per frame. Do not "simplify" this into the function body.
 *
 * A construction failure memoises its fallback (the shape used by
 * `pluralRulesCache` in the i18n core) so a bad tag never retries per row.
 */
const rtfCache: Record<string, Intl.RelativeTimeFormat | null> = {};

function rtf(loc: string, numeric: 'auto' | 'always'): Intl.RelativeTimeFormat | null {
	const key = `${loc}|${numeric}`;
	if (key in rtfCache) return rtfCache[key];
	let made: Intl.RelativeTimeFormat | null = null;
	try {
		// `-u-nu-latn` pins Latin digits through the Unicode extension subtag. It must
		// go in the TAG, not the options bag: TypeScript's `RelativeTimeFormatOptions`
		// has no `numberingSystem` member, so the options form is a TS2353 error under
		// this repo's `strict` (verified against the repo's own tsc) even though it
		// works at runtime. The tag form is the house idiom — see `calendar-rim.ts:91`
		// (`${locale}-u-ca-${cal}`).
		//
		// Why it matters: `fa` resolves to the `arabext` numbering system by default
		// (۳ روز پیش) while the rest of the app interpolates counts with plain
		// `String(count)` — one tooltip would have mixed ۳ and 3. (Bare `ar` already
		// resolves to `latn`; region-tagged `ar-EG`/`ar-SA` would not, which is the
		// second reason to pin it rather than rely on the default.)
		made = new Intl.RelativeTimeFormat(`${loc}-u-nu-latn`, { numeric });
	} catch {
		try {
			made = new Intl.RelativeTimeFormat(loc, { numeric });
		} catch {
			made = null;
		}
	}
	rtfCache[key] = made;
	return made;
}

/**
 * A localized "how long ago" label for an ISO-8601 timestamp.
 *
 * Returns `''` for an empty or unparseable value — the caller then omits the
 * clause entirely, which is exactly what the two private copies did.
 *
 * BUCKETS (chosen from the measured output of all 15 locales, not by taste):
 *   - under `LINK_STALE_DAYS` (90) → the DAY unit with `numeric:'auto'`, giving
 *     today / yesterday / the day-before-yesterday deictics, then "N days ago".
 *   - 90 days … under a year     → the MONTH unit with `numeric:'always'`.
 *   - a year and beyond          → the YEAR unit with `numeric:'always'`.
 *
 * The 90-day boundary is not arbitrary: it is `LINK_STALE_DAYS`, the threshold at
 * which `linkLifecycle` calls a link stale. Inside the living window the user gets
 * day resolution; once a link has gone cold, coarser units read better.
 *
 * Two deliberate choices behind the `numeric` modes, both measured across all 15
 * locales:
 *   - `'auto'` on the day unit is what produces today/yesterday. It is NOT used on
 *     month/year, where at magnitude 1 it switches from elapsed duration to a
 *     CALENDAR claim — "last month" for 44 elapsed days is a different (and false)
 *     statement, and in ru/ja it emits a prepositional phrase ("в прошлом месяце")
 *     rather than a label.
 *   - `'always'` is not used on the day unit: it renders 0 as "in 0 days" and, in
 *     Hebrew, appends a parenthesised numeral to the dual forms.
 * Extending the day bucket to 90 days also steps over the only remaining CLDR data
 * artifact in the matrix — Hebrew months 1 and 2 ("לפני חודש (1)"). At months 3+
 * every locale is clean.
 */
export function formatRelativeDays(iso: string | undefined, loc: string): string {
	if (!iso) return '';
	const parsed = Date.parse(iso);
	if (Number.isNaN(parsed)) return '';

	// Clamp at zero. `Math.floor` of a negative fraction rounds toward −∞, so a
	// timestamp even 1 ms ahead of the clock yielded "-1d ago" before this. A future
	// stamp is reachable in practice — `last_traversed` is written per-device and the
	// Library syncs across machines with skewed clocks (Git/Syncthing/iCloud), and the
	// file is hand-editable on disk (File Over App). Degrading to "today" is the honest
	// reading; without the clamp the localized form would confidently say "tomorrow".
	const days = Math.max(0, Math.floor((Date.now() - parsed) / MS_PER_DAY));

	if (days < LINK_STALE_DAYS) {
		const f = rtf(loc, 'auto');
		return f ? f.format(-days, 'day') : `${days}d`;
	}
	if (days < 365) {
		const f = rtf(loc, 'always');
		const months = Math.floor(days / 30);
		return f ? f.format(-months, 'month') : `${months}mo`;
	}
	const f = rtf(loc, 'always');
	const years = Math.floor(days / 365);
	return f ? f.format(-years, 'year') : `${years}y`;
}

/**
 * The lifecycle tier as a word, in `loc`. Reuses the already-translated
 * `ccs.tier.*` namespace (see the module header).
 *
 * Both spellings of the load-bearing tier are accepted on purpose: `store.ts`
 * types the tier as `'load-bearing'` (hyphen) while the Rust-side CCS snapshot
 * uses `load_bearing` (underscore), and both spellings exist in the codebase today.
 */
const TIER_I18N_KEY: Record<string, string> = {
	fresh: 'fresh',
	emerging: 'emerging',
	established: 'established',
	'load-bearing': 'loadBearing',
	load_bearing: 'loadBearing',
	stale: 'stale',
};

export function linkTierLabel(tier: string | undefined, loc: string): string {
	const raw = tier || 'emerging';
	const key = `ccs.tier.${TIER_I18N_KEY[raw] ?? raw}`;
	const tr = tIn(loc, key);
	// Same guard as the panels' original `typeName`: `tIn` returns the key path on a
	// miss, so an unrecognised tier must fall back to its raw word (today's behaviour,
	// which interpolated the tier verbatim) rather than leaking "ccs.tier.whatever"
	// into a user-visible tooltip.
	return tr !== key ? tr : raw;
}

/**
 * NO BIDI ISOLATES HERE — deliberately, and this comment is the guard against
 * re-adding them.
 *
 * The first cut of this wrapped the whole tooltip in FSI…PDI (U+2068/U+2069) to give
 * a `title` attribute the `dir="auto"` behaviour it cannot otherwise have. On the
 * Boss test (2026-07-18) that produced two defects at once: in English the tooltip
 * box was measured WIDER than the text it drew (a gap on one side), and in Arabic the
 * text ran past the box. One cause — the box is measured with the invisible
 * characters and painted without them, so the width belongs to a different string.
 *
 * The isolates were never needed: `traversalTooltip` composes all three segments in
 * ONE locale, so the string is directionally homogeneous and has no internal
 * reordering hazard for isolates to defend against.
 */

/**
 * The traversal count as a phrase — "3 walks" — via the shared, CLDR plural-aware
 * `plurals.walks`.
 *
 * Exported on its own because the editor's inline `×N` chip
 * (`livePreview.ts`'s `WikilinkTraversalChipWidget`) knows ONLY the count: its
 * `linkTraversalMapField` is a `Map<string, number>`, carrying no tier and no
 * last-traversed. It must therefore say less than the panel tooltip — never
 * default a tier it has not been told.
 */
export function walkCountLabel(count: number | undefined, loc: string): string {
	return tnIn(loc, 'plurals.walks', count ?? 0);
}

/**
 * The living-link state chip's tooltip: `3 walks · Established · 14 days ago`.
 *
 * All three segments render in `loc` — pass `$locale` (the UI language) from the
 * component so it re-renders on a language switch. This is chrome, not authored
 * content: the type pills and the annotation keep following the NOTE's language
 * (the §H note-language principle), while this diagnostic follows the interface,
 * matching the `linkConfidence.rightClickHint` tooltip already on the same element.
 *
 * The count and tier are normalised here rather than at each call site, because the
 * row types declare both as optional.
 */
export function traversalTooltip(
	count: number | undefined,
	tier: string | undefined,
	lastTraversedIso: string | undefined,
	loc: string,
): string {
	const segments = [walkCountLabel(count, loc), linkTierLabel(tier, loc)];
	const when = formatRelativeDays(lastTraversedIso, loc);
	if (when) segments.push(when);
	return segments.join(' · ');
}

/**
 * A link type's display name in `loc`. Re-exported so the link panels reach the
 * shared three-branch resolver instead of their own two-branch copies.
 */
export const linkTypeNameIn = (id: string, loc: string): string => relLabelIn(loc, id);

/**
 * MIG-022 §A.4.d — the annotation slot sometimes carries a known link-type name
 * (e.g. "supersedes") rather than a user-written annotation: legacy index data plus
 * the `search.rs::parse_typed_links` path that treats pipe-aliases as annotation.
 * Localize it when it matches a known type; otherwise pass it through verbatim.
 */
export function displayAnnotationIn(annotation: string | undefined, loc: string): string {
	return annotation ? linkTypeNameIn(annotation, loc) : (annotation ?? '');
}

/** Re-exported so a consumer can type a tier without importing the whole store. */
export type { LinkLifecycle };
