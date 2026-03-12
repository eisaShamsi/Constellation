/**
 * Property Type Registry — persists user-assigned property types across sessions.
 * When a user changes a property's type via the PropertyEditor, it's remembered
 * vault-wide so the same key is detected as that type in all notes.
 *
 * Uses an in-memory cache that syncs to the active Universe's property-types.json.
 */
import { invoke } from '@tauri-apps/api/core';
import type { PropertyType } from './store';

// In-memory cache: { vaultName: { key: type } }
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

/** Persist the cache to the active universe (debounced). */
function persistPropertyTypes() {
	if (saveTimer) clearTimeout(saveTimer);
	saveTimer = setTimeout(() => {
		invoke('save_universe_property_types', { types: cache }).catch(() => {});
	}, 500);
}

/** Get all registered property types for a vault */
export function getRegisteredTypes(vaultName: string): Map<string, PropertyType> {
	const vaultTypes = cache[vaultName];
	if (!vaultTypes) return new Map();
	return new Map(Object.entries(vaultTypes));
}

/** Get a single registered type for a key in a vault */
export function getRegisteredType(vaultName: string, key: string): PropertyType | undefined {
	return cache[vaultName]?.[key.toLowerCase()] as PropertyType | undefined;
}

/** Set a property type for a key in a vault */
export function setRegisteredType(vaultName: string, key: string, type: PropertyType): void {
	if (!cache[vaultName]) cache[vaultName] = {};
	cache[vaultName][key.toLowerCase()] = type;
	persistPropertyTypes();
}

/** Remove a registered type for a key */
export function removeRegisteredType(vaultName: string, key: string): void {
	if (cache[vaultName]) {
		delete cache[vaultName][key.toLowerCase()];
		persistPropertyTypes();
	}
}
