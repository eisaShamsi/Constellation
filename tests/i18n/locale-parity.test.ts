/**
 * Locale parity guard — the 15 i18n bundles may never silently drift again.
 *
 * PROVENANCE (2026-08-01). CLAUDE.md's standing order: every user-facing string
 * goes through `$t()` and all 15 locale files are updated together. That
 * discipline slipped and nothing caught it, because a missing key is INVISIBLE
 * at build time — `t()` silently falls back active-locale → en → raw key, and
 * `resolvePluralForm` silently falls back category → other → one. A sweep found:
 *
 *   • en.json missing 11 LIVE keys. This is the severe direction: en is the
 *     terminal fallback, so a key absent from en renders the RAW KEY PATH
 *     ("styleSetter.labels.note_graph") in ALL 15 languages, not just English.
 *   • the 13 non-en/ar locales each missing 62 live keys (sources.*,
 *     classifierScan.*, taxonomyTreePicker.*, searchBadges.concept) — these
 *     rendered English, i.e. silently untranslated.
 *   • four locales with WRONG CLDR plural category sets (below).
 *
 * The three suites here fail on each of those independently.
 *
 * WHY NOT "diff against en.json" (or ar.json): the drift ran in BOTH directions
 * — ar carried keys en lacked and vice versa — so neither file is the reference.
 * The reference is the UNION across all 15, minus documented exemptions. That is
 * computed by scripts/i18n-parity.mjs, which this test imports rather than
 * reimplements, so `npm test` and `node scripts/i18n-parity.mjs` can never
 * disagree about what parity means.
 */
import { describe, it, expect } from 'vitest';
import {
	LOCALES,
	EXEMPT_PREFIXES,
	analyse,
	loadAll,
	cldrCategories,
	referenceKeys,
} from '../../scripts/i18n-parity.mjs';

const maps = loadAll();
const { reference, report } = analyse(maps);

/** Render a bounded, readable failure message — full lists blow up the reporter. */
function sample(keys: string[], max = 12): string {
	if (keys.length <= max) return keys.join(', ');
	return `${keys.slice(0, max).join(', ')} … (+${keys.length - max} more)`;
}

describe('i18n locale parity — ordinary keys', () => {
	it('resolves a non-trivial reference key set (guards against a broken loader)', () => {
		// If flatten/loadAll ever silently returns nothing, every other assertion
		// below would vacuously pass. Anchor the suite to a real magnitude.
		expect(reference.size).toBeGreaterThan(3000);
		expect(LOCALES).toHaveLength(15);
	});

	it.each(LOCALES)('%s carries every key in the reference set', (loc) => {
		const { missing } = report[loc];
		expect(
			missing,
			`${loc}.json is missing ${missing.length} key(s): ${sample(missing)}\n` +
			`Every user-facing string must exist in all 15 locales (CLAUDE.md § i18n).\n` +
			`Run: node scripts/i18n-parity.mjs`
		).toEqual([]);
	});

	it.each(LOCALES)('%s carries no key outside the reference set', (loc) => {
		const { extra } = report[loc];
		expect(
			extra,
			`${loc}.json has ${extra.length} key(s) no other locale defines: ${sample(extra)}\n` +
			`Either add them to the other 14 locales, or delete them as orphans.`
		).toEqual([]);
	});
});

describe('i18n locale parity — CLDR plural categories ($tn)', () => {
	/**
	 * MIG-087 made i18n plural-aware: `$tn('plurals.<noun>', count)` resolves a
	 * category via `Intl.PluralRules` and looks up `plurals.<noun>.<category>`.
	 *
	 * Plurals are deliberately NOT union-governed. The correct category set is a
	 * property of the LANGUAGE, not of the union of all locale files:
	 *   ar → zero one two few many other      he → one two other
	 *   ru → one few many other               es/fr/pt → one many other
	 *   en/de/fa/ur/hi/tr → one other         zh/ja/ko → other
	 * A union rule would force `plurals.characters.two` into English, where
	 * Intl.PluralRules('en') can never select it — a permanently dead key. So we
	 * assert against Intl.PluralRules itself: the exact engine the runtime uses,
	 * so this test cannot disagree with production.
	 *
	 * Both directions matter and both were real:
	 *   MISSING → the runtime falls back category → other → one and renders
	 *     grammatically WRONG text rather than crashing, so it ships unnoticed.
	 *     ru had NO `other` at all; es/fr/pt had no `many`; ar had no `zero`.
	 *   EXTRA → a category the language lacks is unreachable dead weight that
	 *     misleads the next translator into maintaining it.
	 */
	it.each(LOCALES)('%s defines exactly its CLDR categories for every plural noun', (loc) => {
		const { pluralMissing, pluralExtra } = report[loc];
		expect(
			{ missing: pluralMissing, extra: pluralExtra },
			`${loc}.json plural categories are wrong (CLDR for ${loc}: ${cldrCategories(loc).join(', ')}).\n` +
			(pluralMissing.length ? `  missing: ${sample(pluralMissing)}\n` : '') +
			(pluralExtra.length ? `  dead (unreachable by Intl.PluralRules): ${sample(pluralExtra)}\n` : '') +
			`A missing category makes $tn() silently render the wrong grammatical form.`
		).toEqual({ missing: [], extra: [] });
	});

	it('every plural form is a non-empty string', () => {
		const bad: string[] = [];
		for (const loc of LOCALES) {
			for (const [key, value] of maps[loc]) {
				if (!key.startsWith('plurals.')) continue;
				if (typeof value !== 'string' || !value.trim()) bad.push(`${loc}:${key}`);
			}
		}
		expect(bad, `Empty plural forms: ${sample(bad)}`).toEqual([]);
	});

	it('plural forms that show a number use the {count} placeholder', () => {
		// A form containing a literal digit but no {count} is almost always a
		// translator hardcoding "2 notes" instead of "{count} notes".
		const suspicious: string[] = [];
		for (const loc of LOCALES) {
			for (const [key, value] of maps[loc]) {
				if (!key.startsWith('plurals.') || typeof value !== 'string') continue;
				if (/[0-9٠-٩۰-۹]/.test(value) && !value.includes('{count}')) {
					suspicious.push(`${loc}:${key} = "${value}"`);
				}
			}
		}
		expect(suspicious, `Hardcoded digits in plural forms: ${sample(suspicious)}`).toEqual([]);
	});
});

describe('i18n locale parity — value integrity', () => {
	/**
	 * Deliberately-empty strings, enumerated one by one so that any NEW empty
	 * value still fails.
	 *
	 * `styleSetter.labels.an` is the indefinite article in the Style Setter's
	 * bold-text sample — StyleSetter.svelte:1475 renders `{L('An')} {L('apple')}`
	 * ("An **apple**"). Hebrew, Japanese and Korean have no indefinite article,
	 * so the correct rendering is nothing at all and the translator left it empty.
	 *
	 * KNOWN BUG (out of scope here — component code, 2026-08-01): `L()` treats
	 * '' as a miss (`!v || v === key ? en : v`) and falls back to the English, so
	 * these three currently render "An りんご" / "An 사과" rather than dropping the
	 * article. Fixing that means distinguishing "absent" from "intentionally
	 * empty" in L() — a one-line change in StyleSetter.svelte, not a data change:
	 * no value in this file can produce an empty rendering while L() stands.
	 */
	const INTENTIONALLY_EMPTY = new Set([
		'he:styleSetter.labels.an',
		'ja:styleSetter.labels.an',
		'ko:styleSetter.labels.an',
	]);

	it('no locale has an unexplained empty or whitespace-only string', () => {
		const bad: string[] = [];
		for (const loc of LOCALES) {
			for (const [key, value] of maps[loc]) {
				if (typeof value !== 'string' || value.trim()) continue;
				if (INTENTIONALLY_EMPTY.has(`${loc}:${key}`)) continue;
				bad.push(`${loc}:${key}`);
			}
		}
		expect(bad, `Empty values: ${sample(bad)}`).toEqual([]);
	});

	it('every allowlisted empty string is still actually empty', () => {
		// If a translator later fills one of these in, the waiver is stale and
		// should be deleted rather than left to accumulate.
		const stale = [...INTENTIONALLY_EMPTY].filter((entry) => {
			const split = entry.indexOf(':');
			const loc = entry.slice(0, split) as (typeof LOCALES)[number];
			const v = maps[loc]?.get(entry.slice(split + 1));
			return typeof v !== 'string' || v.trim().length > 0;
		});
		expect(stale, `Stale empty-string waivers — remove them: ${sample(stale)}`).toEqual([]);
	});

	it('every translation preserves the placeholders of its English source', () => {
		/**
		 * `t()` interpolates `{name}`-style params by literal string replace. A
		 * translation that drops `{N}` renders a sentence with the number missing;
		 * one that invents `{n}` renders the literal braces to the user. Both are
		 * silent. Compare each locale's placeholder SET against en's.
		 */
		const en = maps.en;
		const placeholders = (s: string) => (s.match(/\{[a-zA-Z_][a-zA-Z0-9_]*\}/g) ?? []).sort();
		const bad: string[] = [];
		for (const loc of LOCALES) {
			if (loc === 'en') continue;
			for (const [key, value] of maps[loc]) {
				if (key.startsWith('plurals.')) continue; // {count} handled above
				const source = en.get(key);
				if (typeof source !== 'string' || typeof value !== 'string') continue;
				const want = placeholders(source);
				const got = placeholders(value);
				if (want.join(',') !== got.join(',')) {
					bad.push(`${loc}:${key} expected [${want}] got [${got}]`);
				}
			}
		}
		expect(bad, `Placeholder mismatches:\n  ${bad.slice(0, 10).join('\n  ')}`).toEqual([]);
	});
});

describe('i18n parity guard — self-test', () => {
	/**
	 * The suites above all assert "the list is empty". If the analyser ever broke
	 * and returned empty lists unconditionally, they would pass while the repo
	 * rotted. These two inject synthetic drift into an in-memory copy and assert
	 * the analyser SEES it — so a green run is evidence, not an absence.
	 */
	it('detects a deleted key', () => {
		const mutated = loadAll();
		const victim = [...mutated.de.keys()].find((k) => k.startsWith('sources.review.'))!;
		mutated.de.delete(victim);
		expect(analyse(mutated).report.de.missing).toContain(victim);
	});

	it('detects a plural category that the language cannot select', () => {
		const mutated = loadAll();
		mutated.ja.set('plurals.notes.few', '{count} ノート'); // ja is `other`-only
		expect(analyse(mutated).report.ja.pluralExtra).toContain('plurals.notes.few');
	});
});

describe('i18n parity guard — exemptions stay honest', () => {
	/**
	 * `sight.v5.*` is exempt because SightV5.svelte was DELETED and
	 * SIGHT_V5_ENABLED retired (MIG-028), while src/lib/sight/engine.ts keeps
	 * retired-engine key paths on disk as an architectural-history record.
	 * Exempting a namespace is a standing waiver, so it must expire the moment
	 * the code comes back: if anything imports Sight v5 again, the exemption is
	 * stale and this fails, forcing the 13 missing locales to be filled.
	 */
	it('the sight.v5 exemption still describes retired code', () => {
		expect(EXEMPT_PREFIXES).toEqual(['sight.v5.']);
		// The exemption must actually be doing something — if the keys are gone
		// from disk entirely, delete the exemption rather than leaving it to rot.
		const exempted = [...maps.en.keys()].filter((k) => k.startsWith('sight.v5.'));
		expect(
			exempted.length,
			'No sight.v5.* keys remain on disk — remove the exemption from scripts/i18n-parity.mjs.'
		).toBeGreaterThan(0);
	});

	it('exempted keys are excluded from the reference set', () => {
		const ref = referenceKeys(maps);
		expect([...ref].some((k) => k.startsWith('sight.v5.'))).toBe(false);
	});
});
