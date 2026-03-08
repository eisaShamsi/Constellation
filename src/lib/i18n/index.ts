import { writable, derived, get } from 'svelte/store';
import en from './en.json';
import ar from './ar.json';

export type Locale = 'en' | 'ar';
export type Direction = 'ltr' | 'rtl';

const translations: Record<Locale, typeof en> = { en, ar };

const STORAGE_KEY = 'constellation-locale';

function getInitialLocale(): Locale {
	if (typeof window !== 'undefined') {
		const saved = localStorage.getItem(STORAGE_KEY);
		if (saved === 'en' || saved === 'ar') return saved;
	}
	return 'en';
}

export const locale = writable<Locale>(getInitialLocale());

export const dir = derived<typeof locale, Direction>(locale, ($locale) =>
	$locale === 'ar' ? 'rtl' : 'ltr'
);

export const isRTL = derived(locale, ($locale) => $locale === 'ar');

// Persist locale changes
locale.subscribe(($locale) => {
	if (typeof window !== 'undefined') {
		localStorage.setItem(STORAGE_KEY, $locale);
		document.documentElement.lang = $locale;
		document.documentElement.dir = $locale === 'ar' ? 'rtl' : 'ltr';
	}
});

/**
 * Get a translated string by dot-notation key.
 * Usage: t('app.name') → "Constellation"
 */
function lookup(obj: Record<string, unknown>, path: string): string {
	const keys = path.split('.');
	let current: unknown = obj;
	for (const key of keys) {
		if (current && typeof current === 'object' && key in current) {
			current = (current as Record<string, unknown>)[key];
		} else {
			return path; // fallback: return the key itself
		}
	}
	return typeof current === 'string' ? current : path;
}

/**
 * Reactive translation store.
 * Usage in Svelte: $t('app.tagline')
 */
export const t = derived(locale, ($locale) => {
	return (key: string): string => lookup(translations[$locale], key);
});

export function setLocale(newLocale: Locale) {
	locale.set(newLocale);
}

export function toggleLocale() {
	locale.update((current) => (current === 'en' ? 'ar' : 'en'));
}
