/**
 * PJ-229 — app-global preferences that survive a restart.
 *
 * Backed by `{app_data_dir}/app-prefs.json` (`src-tauri/src/app_prefs.rs`), a sibling of
 * the universe registry — NOT inside any universe, because its first tenant is the
 * interface language and that must not change when the user switches universe.
 *
 * ## Why this exists
 *
 * The language lived only in `localStorage`, which PJ-110 proved this app can lose (the
 * leveldb orphan-wipe). On 2026-08-08 the Boss closed Constellation in Arabic and it
 * reopened in English. localStorage is now a CACHE — see `reconcileLocaleFromDisk` in
 * `$lib/i18n` — and this file is the record.
 *
 * ## The read-succeeded latch
 *
 * `save` refuses until a `load` has succeeded. Writing a merged object on top of a read
 * we could not perform would flatten preferences that are still on disk — the same
 * failure `style_presets` documents, and the same discipline `settingsLoaded` already
 * applies in `libraries/store.ts`. A session that cannot read simply does not persist.
 */
import { invoke } from '@tauri-apps/api/core';

type Prefs = Record<string, unknown>;

let snapshot: Prefs = {};
let loaded = false;

/** Read the whole object once at boot. Never throws — a failure means "do not persist". */
export async function loadAppPrefs(): Promise<Prefs> {
	try {
		const v = await invoke<Prefs>('load_app_prefs');
		snapshot = v && typeof v === 'object' ? v : {};
		loaded = true;
	} catch (e) {
		// Deliberately not `loaded = true`: see the latch note above.
		console.warn('[appPrefs] could not read app-prefs.json; this session will not persist', e);
	}
	return snapshot;
}

/**
 * Merge a patch into the snapshot and write the whole object.
 *
 * No debounce: these are rare, deliberate acts (choosing a language), and the 300 ms
 * debounce the per-universe settings save uses is a loss window there is no reason to
 * inherit for a once-in-a-while write.
 */
export async function saveAppPrefs(patch: Prefs): Promise<boolean> {
	if (!loaded) return false;
	snapshot = { ...snapshot, ...patch };
	try {
		await invoke('save_app_prefs', { prefs: snapshot });
		return true;
	} catch (e) {
		// PJ-207 §15 — the caller is TOLD. This used to end at a console.warn, which a release
		// build has no console for: the interface language changed on screen, the write failed,
		// and the next launch quietly came back in the old language — which is the exact
		// complaint (PJ-110, 2026-08-08) this file was written to fix.
		console.warn('[appPrefs] could not save app-prefs.json', e);
		return false;
	}
}

/** The value read at boot, for callers that need it before any save. */
export function getAppPref<T>(key: string): T | undefined {
	return snapshot[key] as T | undefined;
}
