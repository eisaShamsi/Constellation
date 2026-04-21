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

const translations: Record<Locale, typeof en> = {
	en, ar, fa: fa as typeof en, he: he as typeof en, ur: ur as typeof en,
	es: es as typeof en, fr: fr as typeof en, de: de as typeof en,
	zh: zh as typeof en, ja: ja as typeof en, ko: ko as typeof en,
	pt: pt as typeof en, ru: ru as typeof en, hi: hi as typeof en,
	tr: tr as typeof en,
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
 */
export const t = derived(locale, ($locale) => {
	return (key: string, params?: Record<string, string>): string =>
		lookup(translations[$locale] ?? translations.en, key, params);
});

export function setLocale(newLocale: Locale) {
	locale.set(newLocale);
}
