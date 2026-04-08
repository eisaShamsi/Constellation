// ─── Constellation Universe — Store (IPC bridge) ───

import { invoke } from '@tauri-apps/api/core';

export interface UniverseEntry {
	id: string;
	name: string;
	path: string;
	created: string;
}

export interface UniverseMeta {
	name: string;
	created: string;
	version: number;
	children: string[];
}

/** List all known universes from the registry. */
export async function listUniverses(): Promise<UniverseEntry[]> {
	return await invoke('list_universes');
}

/** Create a new universe with directory structure at path/name. */
export async function createUniverse(name: string, path: string): Promise<UniverseEntry> {
	return await invoke('create_universe', { name, path });
}

/** Set the active universe by ID. */
export async function setActiveUniverse(id: string): Promise<void> {
	return await invoke('set_active_universe', { id });
}

/** Get the current active universe path. */
export async function getActiveUniversePath(): Promise<string | null> {
	return await invoke('get_active_universe_path');
}

/** Remove a universe from the registry (does NOT delete files). */
export async function removeUniverseFromRegistry(id: string): Promise<void> {
	return await invoke('remove_universe_from_registry', { id });
}

/** Check if migration from legacy app_data_dir storage is needed. */
export async function checkMigrationNeeded(): Promise<boolean> {
	return await invoke('check_migration_needed');
}

/** Add a child universe path to the active universe. */
export async function addChildUniverse(childPath: string): Promise<void> {
	return await invoke('add_child_universe', { childPath });
}

/** Remove a child universe path from the active universe. */
export async function removeChildUniverse(childPath: string): Promise<void> {
	return await invoke('remove_child_universe', { childPath });
}

/** Resolve the full merged library list for the active universe. */
export async function resolveUniverseLibraries(): Promise<{ id: string; name: string; path: string }[]> {
	return await invoke('resolve_universe_libraries');
}

/** Link a folder as a single-library universe and register it. */
export async function linkLibraryAsUniverse(path: string): Promise<UniverseEntry> {
	return await invoke('link_library_as_universe', { path });
}

/** Open an existing universe directory (must contain universe.json). */
export async function openExistingUniverse(path: string): Promise<UniverseEntry> {
	return await invoke('open_existing_universe', { path });
}

/** Migrate legacy data from app_data_dir to a new universe directory. */
export async function migrateLegacyData(name: string, universePath: string): Promise<UniverseEntry> {
	return await invoke('migrate_legacy_data', { name, universePath });
}

export interface ChildUniverseInfo {
	name: string;
	path: string;
	library_count: number;
}

/** Get info about child universes of the active universe. */
export async function getChildUniverses(): Promise<ChildUniverseInfo[]> {
	return await invoke('get_child_universes');
}

// ─── Universe Data File I/O ───

export async function readUniverseSettings(): Promise<Record<string, unknown>> {
	return await invoke('read_universe_settings');
}

export async function saveUniverseSettings(settings: Record<string, unknown>): Promise<void> {
	return await invoke('save_universe_settings', { settings });
}

export async function readUniverseBookmarks(): Promise<unknown[]> {
	return await invoke('read_universe_bookmarks');
}

export async function saveUniverseBookmarks(bookmarks: unknown[]): Promise<void> {
	return await invoke('save_universe_bookmarks', { bookmarks });
}

export async function readUniverseWorkspaces(): Promise<unknown[]> {
	return await invoke('read_universe_workspaces');
}

export async function saveUniverseWorkspaces(workspaces: unknown[]): Promise<void> {
	return await invoke('save_universe_workspaces', { workspaces });
}

export async function readUniversePropertyTypes(): Promise<Record<string, Record<string, string>>> {
	return await invoke('read_universe_property_types');
}

export async function saveUniversePropertyTypes(types: Record<string, Record<string, string>>): Promise<void> {
	return await invoke('save_universe_property_types', { types });
}
