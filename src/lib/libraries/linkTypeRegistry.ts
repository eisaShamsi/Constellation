/**
 * Link-Type Registry (frontend mirror) — MIG-067 §C.
 *
 * The single frontend source of truth for the link-type vocabulary: the 8
 * built-in typed relations (supports, contradicts, causes, exemplifies,
 * generalizes, derives-from, part-of, supersedes) plus any user-defined types,
 * resolved + ordered + nested exactly as the Rust `LinkTypeRegistry` produces
 * them. Every surface that renders a link type — the editor's inline colors
 * (§E), the Base's per-type columns (§F), the Settings vocabulary editor (§G) —
 * reads it from here so they can never drift.
 *
 * Mirrors `propertyTypeRegistry.ts`: an in-memory cache seeded from the boot
 * bundle (no extra IPC), with an explicit reload + save path to the active
 * universe's `.constellation/link-types.json`. Adds a tiny observer seam so the
 * live-rendering consumers can refresh when the vocabulary changes (a §G save
 * or a universe switch) — the same spirit as `notifySettingsChanged`.
 */
import { invoke } from '@tauri-apps/api/core';

/** One resolved link type — the serialized shape of Rust `LinkTypeDef`. */
export interface LinkTypeDef {
	/** Slug id ([a-z0-9-]); also the stored `note_links.link_type` value. */
	id: string;
	/** Display label (localized by the render layer; defaults to the id). */
	label: string;
	/** Parent id when nested under one of the 8; `null` for a top-level type. */
	parent: string | null;
	/** Inline + badge color (hex). */
	color: string;
	/** Canonical sort key within its level. */
	order: number;
	/** True for the 8 seeds (grammar is immutable; only presentation overridable). */
	builtin: boolean;
	emoji: string | null;
	desc: string | null;
}

/** The 8 seed ids in canonical order — the fallback when nothing is loaded yet
 *  (keeps the editor/columns working before the bundle seeds, and if an IPC
 *  fails). Mirrors `link_types::SEED_IDS`. */
const SEED_IDS = [
	'supports', 'contradicts', 'causes', 'exemplifies',
	'generalizes', 'derives-from', 'part-of', 'supersedes',
] as const;

/** Neutral fallback color for an unknown id (matches the editor's default). */
const DEFAULT_COLOR = '#AAAAAA';

/** Resolved, ordered list (top-level types each followed by their children). */
let cache: LinkTypeDef[] = [];
/** id → def, for O(1) lookups built alongside the cache. */
let byId = new Map<string, LinkTypeDef>();
let loaded = false;

/** Observers notified after the vocabulary changes (re-seed / reload / save). */
const listeners = new Set<() => void>();

function rebuildIndex(): void {
	byId = new Map(cache.map((t) => [t.id, t]));
}

function notify(): void {
	for (const cb of listeners) {
		try { cb(); } catch { /* a bad subscriber must not break the others */ }
	}
}

/** Subscribe to vocabulary changes. Returns an unsubscribe fn (call in onDestroy). */
export function subscribe(cb: () => void): () => void {
	listeners.add(cb);
	return () => listeners.delete(cb);
}

/** Seed the cache from the boot-bundle response (`bundle.link_types`) — the
 *  fast path that avoids a separate `list_link_types` IPC at startup. */
export function seedFromBundle(data: unknown): void {
	if (Array.isArray(data)) {
		cache = data as LinkTypeDef[];
		rebuildIndex();
	}
	loaded = true;
	notify();
}

/** Reload the resolved registry from the active universe. Used on universe
 *  switch and as the fallback when the bundle had no link_types. */
export async function loadLinkTypes(): Promise<void> {
	try {
		const data = await invoke<LinkTypeDef[]>('list_link_types');
		if (Array.isArray(data)) {
			cache = data;
			rebuildIndex();
		}
		loaded = true;
	} catch {
		// Leave whatever we had (seed/last-good); never blank the vocabulary.
		loaded = true;
	}
	notify();
}

/** Persist the user-defined deltas to the active universe and re-seed from the
 *  resolved result so every surface reflects the change immediately. The
 *  backend re-materializes the Base aggregates under the new vocabulary. §G. */
export async function saveLinkTypes(deltas: LinkTypeDef[]): Promise<void> {
	await invoke('save_universe_link_types', { deltas });
	// The backend now holds the new registry; pull the resolved list back.
	await loadLinkTypes();
}

/** Whether the registry has been seeded/loaded at least once. */
export function isLoaded(): boolean {
	return loaded;
}

/** The full resolved list (top-level types each immediately followed by their
 *  children, in canonical order). Empty only before the first seed. */
export function getLinkTypes(): LinkTypeDef[] {
	return cache;
}

/** A single type by id, or undefined if unknown. */
export function getLinkType(id: string): LinkTypeDef | undefined {
	return byId.get(id);
}

/** True if `id` is a known type (seed or custom). */
export function isKnownLinkType(id: string): boolean {
	return byId.has(id) || (cache.length === 0 && (SEED_IDS as readonly string[]).includes(id));
}

/** Inline/badge color for a type id (neutral default for unknown ids). */
export function linkTypeColor(id: string): string {
	return byId.get(id)?.color ?? DEFAULT_COLOR;
}

/** Display label for a type id (falls back to the id itself). */
export function linkTypeLabel(id: string): string {
	return byId.get(id)?.label ?? id;
}

/** 1-based canonical rank (position in the resolved order). Unknown ids sort
 *  last (length + 1), matching the SQL sentinel the back-fill uses. */
export function linkTypeRank(id: string): number {
	const i = cache.findIndex((t) => t.id === id);
	return i >= 0 ? i + 1 : cache.length + 1;
}

/** Top-level types (no parent), in order. */
export function topLevelLinkTypes(): LinkTypeDef[] {
	return cache.filter((t) => t.parent == null);
}

/** Children nested under a given parent id, in order. */
export function linkTypeChildren(parentId: string): LinkTypeDef[] {
	return cache.filter((t) => t.parent === parentId);
}
