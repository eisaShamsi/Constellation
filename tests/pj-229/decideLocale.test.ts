/**
 * PJ-229 — the interface language must survive a restart.
 *
 * Origin: 2026-08-08, the Boss closed Constellation in Arabic and it reopened in
 * English. The locale lived only in `localStorage['constellation-locale']`, and PJ-110
 * had already proved this app can lose that store (the leveldb orphan-wipe). It is now
 * written to `{app_data_dir}/app-prefs.json`, with localStorage demoted to the cache
 * that keeps text direction correct on the very first paint.
 *
 * `decideLocale` is the whole decision, extracted pure so it can be pinned without a
 * filesystem, a Tauri round-trip, or a component mount (this repo has no mount harness).
 */
import { describe, it, expect } from 'vitest';
import { decideLocale } from '../../src/lib/i18n';

describe('PJ-229 — deciding the interface language at boot', () => {
	it('lets the durable record win over the cache', () => {
		// The failing case, exactly: the cache says English, disk says Arabic.
		expect(decideLocale('ar', 'en')).toEqual({ locale: 'ar', adopt: false });
	});

	it('does not rewrite the file when disk and cache already agree', () => {
		expect(decideLocale('ar', 'ar')).toEqual({ locale: 'ar', adopt: false });
	});

	it('adopts the existing choice the first time, instead of resetting to English', () => {
		// An upgrading user already picked Arabic; app-prefs.json does not exist yet.
		// Getting this wrong would mean the change that makes the language durable is
		// itself what discards it.
		expect(decideLocale(undefined, 'ar')).toEqual({ locale: 'ar', adopt: true });
		expect(decideLocale(null, 'ar')).toEqual({ locale: 'ar', adopt: true });
	});

	it('ignores a value it does not recognise, and does not overwrite it either', () => {
		// Hand-edited, or written by a future version that knows more locales. Keep what
		// we have; do not act, and do not clobber a file we did not understand.
		for (const bad of ['klingon', '', 'EN', 42, {}, []]) {
			expect(decideLocale(bad, 'ar')).toEqual({ locale: 'ar', adopt: false });
		}
	});
});
