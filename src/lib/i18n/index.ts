import { writable, derived, get } from 'svelte/store';
import en from './en.json';
import ar from './ar.json';
import fa from './fa.json';
import he from './he.json';
import ur from './ur.json';
import es from './es.json';
import fr from './fr.json';
import de from './de.json';
import zh from './zh.json';
import ja from './ja.json';
import ko from './ko.json';
import pt from './pt.json';
import ru from './ru.json';
import hi from './hi.json';
import tr from './tr.json';

export type Locale = 'en' | 'ar' | 'fa' | 'he' | 'ur' | 'es' | 'fr' | 'de' | 'zh' | 'ja' | 'ko' | 'pt' | 'ru' | 'hi' | 'tr';
export type Direction = 'ltr' | 'rtl';

/** All supported locales with native labels */
export const SUPPORTED_LOCALES: { code: Locale; label: string }[] = [
	{ code: 'en', label: 'English' },
	{ code: 'ar', label: 'العربية' },
	{ code: 'fa', label: 'فارسی' },
	{ code: 'he', label: 'עברית' },
	{ code: 'ur', label: 'اردو' },
	{ code: 'es', label: 'Español' },
	{ code: 'fr', label: 'Français' },
	{ code: 'de', label: 'Deutsch' },
	{ code: 'zh', label: '中文' },
	{ code: 'ja', label: '日本語' },
	{ code: 'ko', label: '한국어' },
	{ code: 'pt', label: 'Português' },
	{ code: 'ru', label: 'Русский' },
	{ code: 'hi', label: 'हिन्दी' },
	{ code: 'tr', label: 'Türkçe' },
];

const RTL_LOCALES = new Set<Locale>(['ar', 'fa', 'he', 'ur']);

// §120: cast each non-en locale through `unknown` to bypass strict structural
// matching. The runtime fallback chain in `t` (active locale → en → key) handles
// the missing-key case gracefully; TypeScript's structural check would otherwise
// require every locale to include every key from en.json the moment one is added.
const translations: Record<Locale, typeof en> = {
	en,
	ar: ar as unknown as typeof en,
	fa: fa as unknown as typeof en,
	he: he as unknown as typeof en,
	ur: ur as unknown as typeof en,
	es: es as unknown as typeof en,
	fr: fr as unknown as typeof en,
	de: de as unknown as typeof en,
	zh: zh as unknown as typeof en,
	ja: ja as unknown as typeof en,
	ko: ko as unknown as typeof en,
	pt: pt as unknown as typeof en,
	ru: ru as unknown as typeof en,
	hi: hi as unknown as typeof en,
	tr: tr as unknown as typeof en,
};

const STORAGE_KEY = 'constellation-locale';

const VALID_LOCALES = new Set<string>(SUPPORTED_LOCALES.map(l => l.code));

function getInitialLocale(): Locale {
	if (typeof window !== 'undefined') {
		const saved = localStorage.getItem(STORAGE_KEY);
		if (saved && VALID_LOCALES.has(saved)) return saved as Locale;
	}
	return 'en';
}

export const locale = writable<Locale>(getInitialLocale());

export const dir = derived<typeof locale, Direction>(locale, ($locale) =>
	RTL_LOCALES.has($locale) ? 'rtl' : 'ltr'
);

export const isRTL = derived(locale, ($locale) => RTL_LOCALES.has($locale));

// Persist locale changes
locale.subscribe(($locale) => {
	if (typeof window !== 'undefined') {
		localStorage.setItem(STORAGE_KEY, $locale);
		document.documentElement.lang = $locale;
		document.documentElement.dir = RTL_LOCALES.has($locale) ? 'rtl' : 'ltr';
	}
});

/**
 * Get a translated string by dot-notation key with optional interpolation.
 * Usage: lookup(obj, 'dialogs.confirmDelete', { name: 'My Note' })
 *   → "Are you sure you want to delete "My Note"?"
 */
function lookup(obj: Record<string, unknown>, path: string, params?: Record<string, string>): string {
	const keys = path.split('.');
	let current: unknown = obj;
	for (const key of keys) {
		if (current && typeof current === 'object' && key in current) {
			current = (current as Record<string, unknown>)[key];
		} else {
			return path; // fallback: return the key itself
		}
	}
	let result = typeof current === 'string' ? current : path;
	if (params) {
		for (const [key, value] of Object.entries(params)) {
			result = result.replace(`{${key}}`, value);
		}
	}
	return result;
}

/**
 * Plural-aware lookup (MIG-087 / Boss-directed grammatical-number, 2026-06-26).
 *
 * Counted nouns must agree with the count per each language's grammar — a bug
 * Eisa caught in the status bar ("19 مكتبات" / "7659 ملاحظات" used the plural
 * form unconditionally, wrong Arabic for 1, 2, and 11+). We resolve the form via
 * the Unicode CLDR plural categories exposed natively by `Intl.PluralRules` (the
 * same engine ICU/CLDR use — WA#5: the battle-tested standard, not hand-rolled).
 *
 * Arabic uses all of one/two/few/many/other (+zero); Russian one/few/many;
 * Hebrew one/two/other; most languages one/other; CJK only other.
 *
 * The locale data lives under the `plurals.<noun>` namespace as a category map,
 * e.g. plurals.notes = { one, two?, few?, many?, other, zero? }. Each form is the
 * COMPLETE rendered phrase (with `{count}` where the digit should appear) so each
 * locale controls whether the number shows — Arabic 1/2 are word-only ("ملاحظة" /
 * "ملاحظتان"), 3+ carry the number ("{count} ملاحظات", "{count} ملاحظة").
 */
const pluralRulesCache: Partial<Record<string, Intl.PluralRules>> = {};
function pluralCategory(loc: string, count: number): Intl.LDMLPluralRule {
	let pr = pluralRulesCache[loc];
	if (!pr) {
		try {
			pr = new Intl.PluralRules(loc, { type: 'cardinal' });
		} catch {
			pr = new Intl.PluralRules('en', { type: 'cardinal' });
		}
		pluralRulesCache[loc] = pr;
	}
	return pr.select(count);
}

/** Resolve the best-matching plural form string for (loc, key, count), or null. */
function resolvePluralForm(loc: Locale, key: string, count: number): string | null {
	const cat = pluralCategory(loc, count);
	const dict = translations[loc] ?? translations.en;
	// category → other → one : graceful fallback when a locale omits a category.
	for (const candidate of [`${key}.${cat}`, `${key}.other`, `${key}.one`]) {
		const r = lookup(dict as Record<string, unknown>, candidate);
		if (r !== candidate) return r;
	}
	return null;
}

/** Interpolate {count} (and any extra params) into a resolved plural form. */
function interpCount(form: string, count: number, params?: Record<string, string | number>): string {
	let result = form.replace(/\{count\}/g, String(count));
	if (params) {
		for (const [k, v] of Object.entries(params)) {
			result = result.replace(`{${k}}`, String(v));
		}
	}
	return result;
}

/**
 * Reactive translation store with interpolation support.
 * Usage in Svelte: $t('app.tagline')
 * With params: $t('dialogs.confirmDelete', { name: 'My Note' })
 *
 * §120: falls back to en.json when the active locale is missing a key.
 * Previously, missing keys returned the literal path ("inspector360.untyped"),
 * which broke `$t(key) || fallback` chains because the literal is truthy
 * — the same bug that forced the Untyped label hardcode in §104/§113.
 * With the fallback chain, missing keys in non-en locales display the
 * English string instead of the key, and partial localization stays
 * graceful while translators backfill.
 */
export const t = derived(locale, ($locale) => {
	return (key: string, params?: Record<string, string>): string => {
		const localeResult = lookup(translations[$locale] ?? translations.en, key, params);
		if (localeResult !== key) return localeResult;
		if ($locale !== 'en') {
			const enResult = lookup(translations.en, key, params);
			if (enResult !== key) return enResult;
		}
		return key;
	};
});

/**
 * Reactive PLURAL-aware translation store (MIG-087).
 * Usage in Svelte: $tn('plurals.notes', count)  →  e.g. "ملاحظتان" / "{count} ملاحظات"
 * Picks the CLDR plural category for the active locale, falls back active→en→key.
 */
export const tn = derived(locale, ($locale) => {
	return (key: string, count: number, params?: Record<string, string | number>): string => {
		let form = resolvePluralForm($locale, key, count);
		if (form === null && $locale !== 'en') form = resolvePluralForm('en', key, count);
		if (form === null) return key;
		return interpCount(form, count, params);
	};
});

export function setLocale(newLocale: Locale) {
	locale.set(newLocale);
}

/**
 * Translate a key in a SPECIFIC locale (not the reactive UI locale), with the same
 * active-locale → en → key fallback chain as `t`. Used where the content language
 * differs from the UI language — e.g. typed-link labels that must read in the
 * NOTE's own language regardless of the interface language (MIG-067 §E.2).
 */
export function tIn(loc: string, key: string, params?: Record<string, string>): string {
	const l = (loc in translations ? loc : 'en') as Locale;
	const localeResult = lookup(translations[l] ?? translations.en, key, params);
	if (localeResult !== key) return localeResult;
	if (l !== 'en') {
		const enResult = lookup(translations.en, key, params);
		if (enResult !== key) return enResult;
	}
	return key;
}

/**
 * Plural-aware translate in a SPECIFIC locale (non-reactive sibling of `tIn`).
 * Same active-locale → en → key fallback as `tn`. For surfaces that render in a
 * locale other than the reactive UI locale.
 */
export function tnIn(loc: string, key: string, count: number, params?: Record<string, string | number>): string {
	const l = (loc in translations ? loc : 'en') as Locale;
	let form = resolvePluralForm(l, key, count);
	if (form === null && l !== 'en') form = resolvePluralForm('en', key, count);
	if (form === null) return key;
	return interpCount(form, count, params);
}

/** Get the searchOps map for the current locale (for query canonicalization). */
export function getSearchOps(): Record<string, string> | null {
	const loc = get(locale);
	const trans = translations[loc] as Record<string, unknown>;
	return (trans?.searchOps as Record<string, string>) ?? null;
}
