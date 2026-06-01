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

/** Get the searchOps map for the current locale (for query canonicalization). */
export function getSearchOps(): Record<string, string> | null {
	const loc = get(locale);
	const trans = translations[loc] as Record<string, unknown>;
	return (trans?.searchOps as Record<string, string>) ?? null;
}
