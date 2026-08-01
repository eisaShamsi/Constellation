/**
 * Property Type Registry — persists user-assigned property types across sessions.
 * When a user changes a property's type via the PropertyEditor, it's remembered
 * library-wide so the same key is detected as that type in all notes.
 *
 * Uses an in-memory cache that syncs to the active Universe's property-types.json.
 */
import { invoke } from '@tauri-apps/api/core';
import { activeUniverseRootSync, type PropertyType } from './store';

// In-memory cache: { libraryName: { key: type } }
let cache: Record<string, Record<string, PropertyType>> = {};
let loaded = false;
let saveTimer: ReturnType<typeof setTimeout> | null = null;
/** 2026-08-01 inspection — the universe root captured when the debounce ARMED, so a timer
 *  firing after a universe switch writes to the universe the assignment belonged to. */
let pendingRoot: string | undefined;

/**
 * MIG-108 inspection (APP-KILLER) — a FAILED READ must never present as "you have none",
 * because the next write turns that emptiness into the truth on disk.
 *
 * `loaded` used to be set to `true` on the CATCH path too, with an empty cache. So a
 * momentary lock on property-types.json (a sync tool, antivirus, a half-written file — and
 * the boot bundle maps ANY read error to `{}` silently) left the registry looking empty, and
 * the user's very next property-type assignment atomically wrote `{one entry}` over every
 * assignment in the universe. Byte-for-byte the collections bug PJ-187 fixed; this sibling
 * store never got the fix (a Whole-Ecosystem gap).
 *
 * The rule, identical to collections: `loaded` means "a read SUCCEEDED", nothing else, and
 * a write is refused until it does.
 */
let loadError: string | null = null;

/** True while writes are refused: either the read FAILED, or no successful read has
 *  happened yet (the ambiguous empty-bundle case — `{}` means both "empty" and "failed"). */
export function propertyTypesUnavailable(): boolean {
	return !loaded;
}

/**
 * Safety inspection 2026-08-01 — RESET on a universe switch.
 *
 * `cache` and `loaded` are module-globals that were never cleared, and `seedFromBundle`
 * deliberately no-ops on an empty object (it cannot tell "empty" from "failed"). Together
 * that carried universe A's ENTIRE registry into universe B: still `loaded`, so writes were
 * allowed, and the first type assignment in B atomically wrote A's registry over B's
 * property-types.json. The latch must mean "a read succeeded FOR THE UNIVERSE WE ARE IN".
 *
 * Called at the start of every load — which is what a universe switch performs.
 */
function resetForUniverseSwitch(): void {
	cache = {};
	loaded = false;
	loadError = null;
}

/** Load all property types from the active universe into the cache. */
export async function loadPropertyTypes(): Promise<void> {
	// 2026-08-01 inspection — a universe switch re-loads through here; a pending debounced
	// write still belongs to the OUTGOING universe. Flush it (with its arm-time root) before
	// the cache is replaced, or the timer fires later and writes the new universe's registry
	// over the old universe's file (or vice versa).
	await flushPendingPropertyTypesSave();
	// The outgoing universe's registry must not survive into the incoming one (see above).
	// Ordered AFTER the flush: the flush still belongs to the universe we are leaving.
	resetForUniverseSwitch();
	try {
		const data = await invoke<Record<string, Record<string, string>>>('read_universe_property_types');
		if (data && typeof data === 'object') {
			cache = data as Record<string, Record<string, PropertyType>>;
		}
		loaded = true;
		loadError = null;
	} catch (e) {
		// NOT loaded: leave the cache alone and refuse to persist over the file.
		loadError = String(e);
		console.error('[propertyTypes] read failed — property types are READ-ONLY this session:', e);
	}
}

/** Seed the cache from a boot-bundle response so initializeApp can avoid
 *  a separate read_universe_property_types IPC. Effectively identical to
 *  loadPropertyTypes but skips the invoke. */
export function seedFromBundle(data: unknown): void {
	// Safety inspection 2026-08-01 — the bundle is the universe-SWITCH path too, and this
	// function no-ops on an empty object, so without this reset the OUTGOING universe's
	// registry stayed latched and live: the first type assignment in the new universe wrote
	// the old universe's whole registry over its property-types.json. Reset first, then let
	// only a non-empty payload re-latch; the ambiguous empty case defers to the explicit
	// `loadPropertyTypes()` fallback its caller runs, which can tell empty from failed.
	resetForUniverseSwitch();
	// The bundle carries `{}` both for "genuinely empty" and for "the read FAILED"
	// (boot_bundle.rs maps any error through unwrap_or) — indistinguishable here. A
	// NON-EMPTY object proves the read succeeded and may latch; an empty one proves
	// nothing, so it must NOT — the explicit loadPropertyTypes() fallback decides, and it
	// can tell them apart. (`{}` is truthy AND an object, so the obvious guard latches on
	// exactly the ambiguous case — which is how this was caught.)
	if (data && typeof data === 'object' && Object.keys(data as object).length > 0) {
		cache = data as Record<string, Record<string, PropertyType>>;
		loaded = true;
		loadError = null;
	}
}

/** Persist the cache to the active universe (debounced). */
function persistPropertyTypes() {
	// The latch is `loaded` — "a read SUCCEEDED" — not merely "no error was recorded". A
	// bundle that carried an ambiguous `{}` leaves no error AND no proof; writing then would
	// replace the whole registry with one entry, which is the bug this guard exists for.
	if (!loaded) {
		console.error(
			'[propertyTypes] refusing to write: no successful read yet',
			loadError ? `(read failed: ${loadError})` : '(awaiting the explicit read)',
		);
		return;
	}
	if (saveTimer) clearTimeout(saveTimer);
	else pendingRoot = activeUniverseRootSync(); // first arm of this window
	saveTimer = setTimeout(() => {
		saveTimer = null;
		const universeRoot = pendingRoot;
		pendingRoot = undefined;
		// Safety Audit G6 (W1-12): surface a failed persist (was .catch(()=>{}), so a
		// save failure silently lost the user's property-type assignment).
		invoke('save_universe_property_types', { types: cache, universeRoot }).catch((e) => {
			console.error('[propertyTypes] persist failed — type assignment not saved to disk:', e);
		});
	}, 500);
}

/** 2026-08-01 inspection — fire the pending debounced write NOW (app close / universe
 *  switch): an assignment made inside the 500ms window must never silently revert. */
export async function flushPendingPropertyTypesSave(): Promise<void> {
	if (!saveTimer) return;
	clearTimeout(saveTimer);
	saveTimer = null;
	const universeRoot = pendingRoot ?? activeUniverseRootSync();
	pendingRoot = undefined;
	try {
		await invoke('save_universe_property_types', { types: cache, universeRoot });
	} catch (e) {
		console.error('[propertyTypes] persist flush failed — type assignment not saved to disk:', e);
	}
}

/** Get all registered property types for a library */
export function getRegisteredTypes(libraryName: string): Map<string, PropertyType> {
	const libraryTypes = cache[libraryName];
	if (!libraryTypes) return new Map();
	return new Map(Object.entries(libraryTypes));
}

/** Get a single registered type for a key in a library */
export function getRegisteredType(libraryName: string, key: string): PropertyType | undefined {
	return cache[libraryName]?.[key.toLowerCase()] as PropertyType | undefined;
}

/** Set a property type for a key in a library */
export function setRegisteredType(libraryName: string, key: string, type: PropertyType): void {
	if (!cache[libraryName]) cache[libraryName] = {};
	cache[libraryName][key.toLowerCase()] = type;
	persistPropertyTypes();
}

/** Remove a registered type for a key */
export function removeRegisteredType(libraryName: string, key: string): void {
	if (cache[libraryName]) {
		delete cache[libraryName][key.toLowerCase()];
		persistPropertyTypes();
	}
}
