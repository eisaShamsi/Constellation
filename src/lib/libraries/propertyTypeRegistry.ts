/**
 * Property Type Registry — persists user-assigned property types across sessions.
 * When a user changes a property's type via the PropertyEditor, it's remembered
 * library-wide so the same key is detected as that type in all notes.
 *
 * Uses an in-memory cache that syncs to the active Universe's property-types.json.
 */
import { invoke } from '@tauri-apps/api/core';
import type { PropertyType } from './store';

// In-memory cache: { libraryName: { key: type } }
let cache: Record<string, Record<string, PropertyType>> = {};
let loaded = false;
let saveTimer: ReturnType<typeof setTimeout> | null = null;

/** Load all property types from the active universe into the cache. */
export async function loadPropertyTypes(): Promise<void> {
	try {
		const data = await invoke<Record<string, Record<string, string>>>('read_universe_property_types');
		if (data && typeof data === 'object') {
			cache = data as Record<string, Record<string, PropertyType>>;
		}
		loaded = true;
	} catch {
		cache = {};
		loaded = true;
	}
}

/** Seed the cache from a boot-bundle response so initializeApp can avoid
 *  a separate read_universe_property_types IPC. Effectively identical to
 *  loadPropertyTypes but skips the invoke. */
export function seedFromBundle(data: unknown): void {
	if (data && typeof data === 'object') {
		cache = data as Record<string, Record<string, PropertyType>>;
	}
	loaded = true;
}

/** Persist the cache to the active universe (debounced). */
function persistPropertyTypes() {
	if (saveTimer) clearTimeout(saveTimer);
	saveTimer = setTimeout(() => {
		// Safety Audit G6 (W1-12): surface a failed persist (was .catch(()=>{}), so a
		// save failure silently lost the user's property-type assignment).
		invoke('save_universe_property_types', { types: cache }).catch((e) => {
			console.error('[propertyTypes] persist failed — type assignment not saved to disk:', e);
		});
	}, 500);
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
