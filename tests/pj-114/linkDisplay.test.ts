/**
 * PJ-114 Phase 1 §3b — the shared living-link display helpers.
 *
 * What this file protects:
 *
 *   1. The relative-time buckets and their BOUNDARIES. The two panels each carried
 *      a private, byte-identical, hardcoded-English copy of this logic; the whole
 *      point of §3b is that there is now one. A boundary drift here is invisible in
 *      the UI (a tooltip) and would never be caught by eye.
 *
 *   2. The future-timestamp clamp. The old code's `Math.floor` on a negative
 *      fraction rendered "-1d ago"; without the clamp the localized form would say
 *      "tomorrow" — a confident false statement about the user's own knowledge.
 *      Reachable via clock skew across synced devices and hand-edited files.
 *
 *   3. The unknown-tier guard. `tIn` returns the KEY PATH on a miss, so a tier the
 *      map doesn't know must degrade to its raw word, never leak "ccs.tier.foo"
 *      into a user-visible tooltip.
 *
 *   4. i18n parity across all 15 locales for the two namespaces this feature reuses
 *      (`ccs.tier.*`, `plurals.walks`). This is the regression most likely to bite
 *      later — nothing in this repo enforces locale-file parity (no CI check, no
 *      lint rule, and the TS structural check is deliberately disabled), and a 16th
 *      locale or a namespace rename would silently untranslate the chip.
 */
import { describe, it, expect } from 'vitest';
import {
	formatRelativeDays,
	linkTierLabel,
	traversalTooltip,
	walkCountLabel,
} from '$lib/links/linkDisplay';
import { LINK_STALE_DAYS } from '$lib/libraries/store';
import { detectDir } from '$lib/utils';

import ar from '$lib/i18n/ar.json';
import de from '$lib/i18n/de.json';
import en from '$lib/i18n/en.json';
import es from '$lib/i18n/es.json';
import fa from '$lib/i18n/fa.json';
import fr from '$lib/i18n/fr.json';
import he from '$lib/i18n/he.json';
import hi from '$lib/i18n/hi.json';
import ja from '$lib/i18n/ja.json';
import ko from '$lib/i18n/ko.json';
import pt from '$lib/i18n/pt.json';
import ru from '$lib/i18n/ru.json';
import tr from '$lib/i18n/tr.json';
import ur from '$lib/i18n/ur.json';
import zh from '$lib/i18n/zh.json';

const LOCALES: Record<string, any> = { ar, de, en, es, fa, fr, he, hi, ja, ko, pt, ru, tr, ur, zh };

/** An ISO stamp `days` in the past. The extra second keeps `Math.floor` stable
 *  against the milliseconds that elapse between building the stamp and reading it. */
const daysAgo = (days: number) => new Date(Date.now() - (days * 86_400_000 + 1000)).toISOString();

describe('formatRelativeDays — empty and malformed input', () => {
	// The two private copies both returned '' here, and the caller then omitted the
	// clause entirely. That contract must survive the extraction.
	it('returns empty string for missing, empty, or unparseable timestamps', () => {
		expect(formatRelativeDays(undefined, 'en')).toBe('');
		expect(formatRelativeDays('', 'en')).toBe('');
		expect(formatRelativeDays('not-a-date', 'en')).toBe('');
		expect(formatRelativeDays('2026-13-45T99:99:99Z', 'en')).toBe('');
	});
});

describe('formatRelativeDays — the day bucket (under LINK_STALE_DAYS)', () => {
	it('renders today / yesterday deictics, preserving the old strings in English', () => {
		expect(formatRelativeDays(daysAgo(0), 'en')).toBe('today');
		expect(formatRelativeDays(daysAgo(1), 'en')).toBe('yesterday');
	});

	it('renders elapsed days, not weeks, right up to the stale threshold', () => {
		expect(formatRelativeDays(daysAgo(3), 'en')).toBe('3 days ago');
		expect(formatRelativeDays(daysAgo(14), 'en')).toBe('14 days ago');
		expect(formatRelativeDays(daysAgo(LINK_STALE_DAYS - 1), 'en')).toBe('89 days ago');
	});

	it('localizes the day bucket in every locale without leaking English', () => {
		expect(formatRelativeDays(daysAgo(0), 'ar')).toBe('اليوم');
		expect(formatRelativeDays(daysAgo(1), 'he')).toBe('אתמול');
		expect(formatRelativeDays(daysAgo(14), 'ja')).toBe('14 日前');
		expect(formatRelativeDays(daysAgo(14), 'ru')).toBe('14 дней назад');
	});
});

describe('formatRelativeDays — the month and year buckets', () => {
	it('switches to months at exactly LINK_STALE_DAYS', () => {
		expect(formatRelativeDays(daysAgo(LINK_STALE_DAYS), 'en')).toBe('3 months ago');
		expect(formatRelativeDays(daysAgo(364), 'en')).toBe('12 months ago');
	});

	it('switches to years at exactly 365 days', () => {
		expect(formatRelativeDays(daysAgo(365), 'en')).toBe('1 year ago');
		expect(formatRelativeDays(daysAgo(800), 'en')).toBe('2 years ago');
	});

	it('uses elapsed-duration phrasing, never a calendar claim', () => {
		// `numeric:'auto'` would render these as "last month" / "last year" — a
		// different and false statement about elapsed time, and in ru/ja a
		// prepositional phrase rather than a label.
		expect(formatRelativeDays(daysAgo(365), 'en')).not.toBe('last year');
		expect(formatRelativeDays(daysAgo(365), 'ru')).toBe('1 год назад');
		expect(formatRelativeDays(daysAgo(100), 'ja')).toBe('3 か月前');
	});

	it('avoids the Hebrew parenthesised-numeral CLDR artifact at every bucket', () => {
		// he months 1 and 2 render as "לפני חודש (1)" / "לפני חודשיים (2)". Extending
		// the day bucket to LINK_STALE_DAYS steps over both windows; months start at 3.
		for (const d of [0, 1, 2, 14, 89, 90, 200, 364, 365, 800]) {
			expect(formatRelativeDays(daysAgo(d), 'he')).not.toMatch(/\(\d+\)/);
		}
	});
});

describe('formatRelativeDays — future timestamps clamp to today', () => {
	// Pre-existing bug this fixes: Math.floor of a negative fraction rounds toward
	// −∞, so any stamp ahead of the clock produced "-1d ago". Localized, that would
	// have become the confident falsehood "tomorrow".
	it('never reports a future traversal', () => {
		const inOneHour = new Date(Date.now() + 3_600_000).toISOString();
		const inThirtyDays = new Date(Date.now() + 30 * 86_400_000).toISOString();
		expect(formatRelativeDays(inOneHour, 'en')).toBe('today');
		expect(formatRelativeDays(inThirtyDays, 'en')).toBe('today');
		expect(formatRelativeDays(inThirtyDays, 'ar')).toBe('اليوم');
	});
});

describe('formatRelativeDays — digits stay Latin in every locale', () => {
	// Persian defaults to the `arabext` numbering system (۳ روز پیش) while the rest
	// of the app interpolates counts with plain String(count). One tooltip must not
	// mix ۳ and 3, so the formatter pins `-u-nu-latn` via the locale tag.
	it('emits Latin digits for fa', () => {
		const out = formatRelativeDays(daysAgo(14), 'fa');
		expect(out).toContain('14');
		expect(out).not.toMatch(/[۰-۹]/);
	});

	it('emits Latin digits in every supported locale', () => {
		for (const loc of Object.keys(LOCALES)) {
			const out = formatRelativeDays(daysAgo(14), loc);
			expect(out, `locale ${loc}`).toMatch(/14/);
		}
	});
});

describe('linkTierLabel', () => {
	it('translates every lifecycle tier', () => {
		expect(linkTierLabel('fresh', 'en')).toBe('Fresh');
		expect(linkTierLabel('emerging', 'en')).toBe('Emerging');
		expect(linkTierLabel('established', 'en')).toBe('Established');
		expect(linkTierLabel('stale', 'en')).toBe('Stale');
		expect(linkTierLabel('established', 'ar')).toBe('راسخ');
	});

	it('accepts both spellings of the load-bearing tier', () => {
		// store.ts types it 'load-bearing'; the Rust-side CCS snapshot uses
		// 'load_bearing'. Both spellings exist in the codebase today.
		expect(linkTierLabel('load-bearing', 'en')).toBe('Load-bearing');
		expect(linkTierLabel('load_bearing', 'en')).toBe('Load-bearing');
		expect(linkTierLabel('load-bearing', 'ar')).toBe('ركيزة');
	});

	it('defaults an absent tier to emerging, matching the chip class', () => {
		expect(linkTierLabel(undefined, 'en')).toBe('Emerging');
		expect(linkTierLabel('', 'en')).toBe('Emerging');
	});

	it('degrades an unknown tier to its raw word, never a key path', () => {
		expect(linkTierLabel('wobbly', 'en')).toBe('wobbly');
		expect(linkTierLabel('wobbly', 'en')).not.toContain('ccs.tier');
	});
});

describe('traversalTooltip', () => {
	const strip = (s: string) => s;

	// Boss test 2026-07-18: an earlier cut wrapped this in FSI…PDI (U+2068/U+2069).
	// The tooltip box was then measured WITH the invisible characters and painted
	// WITHOUT them, so the box width belonged to a different string — the English box
	// was too wide and the Arabic text ran past it. They were never needed: all three
	// segments are composed in one locale, so there is no internal reordering hazard.
	// This test is the guard against a well-meaning re-add.
	it('contains no invisible bidi control characters', () => {
		for (const loc of ['en', 'ar', 'he', 'ja']) {
			const out = traversalTooltip(3, 'established', daysAgo(2), loc);
			expect(out, `locale ${loc}`).not.toMatch(/[⁦-⁩‎‏؜]/);
		}
	});

	it('reads count · tier · recency', () => {
		expect(strip(traversalTooltip(3, 'established', daysAgo(2), 'en')))
			.toBe('3 walks · Established · 2 days ago');
	});

	it('is plural-aware via the shared CLDR vocabulary', () => {
		expect(strip(traversalTooltip(1, 'emerging', '', 'en'))).toBe('1 walk · Emerging');
		// Arabic distinguishes one/two/few/many; the shared plurals.walks carries all.
		expect(strip(traversalTooltip(1, 'emerging', '', 'ar'))).toContain('عبور واحد');
		expect(strip(traversalTooltip(2, 'emerging', '', 'ar'))).toContain('عبوران');
	});

	it('omits the recency clause when the link has never been traversed on record', () => {
		const out = strip(traversalTooltip(2, 'emerging', undefined, 'en'));
		expect(out).toBe('2 walks · Emerging');
		expect(out.split('·')).toHaveLength(2);
	});

	it('normalises an undefined count rather than rendering "undefined"', () => {
		expect(strip(traversalTooltip(undefined, undefined, undefined, 'en')))
			.toBe('0 walks · Emerging');
	});

	it('carries no English into a non-English tooltip', () => {
		const out = strip(traversalTooltip(3, 'load-bearing', daysAgo(2), 'ar'));
		expect(out).not.toMatch(/walk|Traversed|Last|ago|bearing/i);
		expect(out).toContain('ركيزة');
	});
});

describe('walkCountLabel — the editor chip says less, in the same words', () => {
	// The editor's inline ×N chip knows only the count (linkTraversalMapField is a
	// Map<string, number>), so it must not default a tier it was never told.
	it('renders the count alone, with no tier and no recency', () => {
		expect(walkCountLabel(3, 'en')).toBe('3 walks');
		expect(walkCountLabel(1, 'en')).toBe('1 walk');
		expect(walkCountLabel(3, 'en')).not.toContain('·');
		expect(walkCountLabel(3, 'en')).not.toMatch(/Emerging|Fresh/);
	});

	it('is the same vocabulary the panel tooltip uses', () => {
		// If these ever diverge, the app is calling the identical event two things.
		const fromPanel = traversalTooltip(3, 'emerging', undefined, 'ar').split(' · ')[0];
		expect(fromPanel).toContain(walkCountLabel(3, 'ar'));
	});

	it('carries no English into a non-English note', () => {
		expect(walkCountLabel(3, 'ar')).not.toMatch(/walk|Traversed|time/i);
		expect(walkCountLabel(3, 'ja')).toBe('3回の通過');
	});

	it('normalises an undefined count', () => {
		expect(walkCountLabel(undefined, 'en')).toBe('0 walks');
	});
});

describe('link tooltip text direction — resolved by dominance, not first character', () => {
	// Boss test 2026-07-18: an Arabic NSC summary rendered LTR inside the app-drawn tooltip
	// under an English interface — short final line flush left, sentence period on the wrong
	// side. `linkTip.ts` sets the box's `dir` from the CONTENT via `detectDir`.
	//
	// Why `detectDir` and not `dir="auto"`: PJ-106 §A1 already replaced auto for this exact
	// reason — auto resolves from the first STRONG CHARACTER, so a Latin-first/Arabic-dominant
	// string comes out LTR. The screenshot that reported the bug contained precisely that shape
	// (a row titled "Arabic music" with an Arabic summary). These cases lock the distinction.
	const ARABIC_SUMMARY =
		'محمد عبد الوهاب هو مغني وملحن وممثل مصري، يعدّ أحد أعلام الموسيقى العربية، لقّب بموسيقار الأجيال';

	it('an Arabic summary resolves RTL', () => {
		expect(detectDir(ARABIC_SUMMARY)).toBe('rtl');
	});

	it('a Latin-first but Arabic-dominant summary still resolves RTL (dir="auto" would not)', () => {
		expect(detectDir(`Arabic music — ${ARABIC_SUMMARY}`)).toBe('rtl');
	});

	it('an English summary resolves LTR even when it opens with an Arabic word', () => {
		expect(detectDir('الموسيقى: a long English summary sentence about this note and its sources'))
			.toBe('ltr');
	});

	it('the app-composed chip tooltip resolves to its own interface language', () => {
		expect(detectDir(traversalTooltip(3, 'established', daysAgo(2), 'en'))).toBe('ltr');
		expect(detectDir(traversalTooltip(3, 'established', daysAgo(2), 'ar'))).toBe('rtl');
		expect(detectDir(traversalTooltip(3, 'established', daysAgo(2), 'he'))).toBe('rtl');
	});
});

describe('i18n parity — the reused namespaces exist in all 15 locales', () => {
	const TIERS = ['fresh', 'emerging', 'established', 'loadBearing', 'stale'];

	it.each(Object.keys(LOCALES))('%s has every ccs.tier key as a non-empty string', (loc) => {
		const tier = LOCALES[loc]?.ccs?.tier;
		expect(tier, `${loc} is missing ccs.tier`).toBeTruthy();
		for (const t of TIERS) {
			expect(typeof tier[t], `${loc}.ccs.tier.${t}`).toBe('string');
			expect(tier[t].length, `${loc}.ccs.tier.${t} is empty`).toBeGreaterThan(0);
		}
	});

	it.each(Object.keys(LOCALES))('%s has plurals.walks with at least an "other" form', (loc) => {
		const walks = LOCALES[loc]?.plurals?.walks;
		expect(walks, `${loc} is missing plurals.walks`).toBeTruthy();
		// `resolvePluralForm` falls back category → other → one, so one of those must exist.
		expect(Boolean(walks.other || walks.one), `${loc}.plurals.walks has no usable form`).toBe(true);
	});
});
