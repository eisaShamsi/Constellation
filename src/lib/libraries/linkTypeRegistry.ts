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
import { writable } from 'svelte/store';

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
export const SEED_IDS = [
	'supports', 'contradicts', 'causes', 'exemplifies',
	'generalizes', 'derives-from', 'part-of', 'supersedes',
] as const;

/** Neutral fallback color for an unknown id (matches the editor's default). */
const DEFAULT_COLOR = '#AAAAAA';

/** The 8 built-in defaults (id → presentation), mirroring Rust `seeds()`. Used to
 *  compute minimal deltas: a seed is persisted only when its presentation differs
 *  from the default, so a future change to a default still reaches the user. */
export const SEED_DEFAULTS: Record<string, { label: string; color: string; order: number }> = {
	supports: { label: 'Supports', color: '#4A9EFF', order: 1 },
	contradicts: { label: 'Contradicts', color: '#FF4A4A', order: 2 },
	causes: { label: 'Causes', color: '#FF8C42', order: 3 },
	exemplifies: { label: 'Exemplifies', color: '#4AFF88', order: 4 },
	generalizes: { label: 'Generalizes', color: '#A44AFF', order: 5 },
	'derives-from': { label: 'Derives From', color: '#FFD700', order: 6 },
	'part-of': { label: 'Part Of', color: '#AAAAAA', order: 7 },
	supersedes: { label: 'Supersedes', color: '#5B7A8A', order: 8 },
};

/** Reduce a working list to the minimal deltas for `link-types.json`: every custom
 *  type, plus only the seeds whose label / colour / order differs from the default. */
export function toLinkTypeDeltas(types: LinkTypeDef[]): LinkTypeDef[] {
	return types.filter((t) => {
		const d = SEED_DEFAULTS[t.id];
		if (!d) return true; // custom → always a delta
		return t.label !== d.label || t.color !== d.color || t.order !== d.order;
	});
}

/** Resolved, ordered list (top-level types each followed by their children). */
let cache: LinkTypeDef[] = [];
/** id → def, for O(1) lookups built alongside the cache. */
let byId = new Map<string, LinkTypeDef>();
let loaded = false;

/** Observers notified after the vocabulary changes (re-seed / reload / save). */
const listeners = new Set<() => void>();

/** Svelte-reactive mirror of the resolved registry — emits the current list on every
 *  change so components can derive colours/labels that update LIVE when a type is
 *  recoloured in the §G editor (MIG-067: the registry is the single colour source). */
const _store = writable<LinkTypeDef[]>([]);
export const linkTypesStore = { subscribe: _store.subscribe };

function rebuildIndex(): void {
	byId = new Map(cache.map((t) => [t.id, t]));
}

function notify(): void {
	_store.set(cache);
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

/** True if `id` is a known typed act (seed or custom). */
export function isKnownLinkType(id: string): boolean {
	return byId.has(id) || (cache.length === 0 && (SEED_IDS as readonly string[]).includes(id));
}

/** True if `id` is a recognized stored `link_type` value — a typed act OR the
 *  null/default `associative`. Mirror of Rust `is_link_type_value`: the panels
 *  and editor membership checks historically accepted `associative` alongside
 *  the typed acts, so they use this (not `isKnownLinkType`) to stay byte-identical
 *  while still recognizing custom types. */
export function isLinkTypeValue(id: string): boolean {
	return id === 'associative' || isKnownLinkType(id);
}

/** Strip a predicate-first `type::` prefix from a wikilink's inner text when the
 *  prefix is a KNOWN link type. `supports::Apple` → `Apple` (any `|display` and
 *  `#fragment` are preserved); `Apple|alias`, `Apple`, and `C++::vector` (unknown
 *  prefix) pass through unchanged. Used by the editor- and HTML-click resolvers so
 *  a typed link `[[type::target]]` opens its TARGET, not a note literally named
 *  "type::target". Guarded by isLinkTypeValue so a `::` inside a real note name is
 *  never mistaken for a type prefix. */
export function stripLinkTypePrefix(inner: string): string {
	const i = inner.indexOf('::');
	if (i > 0 && isLinkTypeValue(inner.slice(0, i).trim().toLowerCase())) {
		return inner.slice(i + 2);
	}
	return inner;
}

/** MIG-075 follow-up — the ONE definition of "a null type": ids that mean
 *  untyped / the open question rather than a typed cognitive act.
 *  `associative` is the canonical null id (MIG-067), `relates` the legacy
 *  one; empty/undefined count as null defensively. Callers decide what
 *  null MEANS for them (untyped tint, default weight, …); membership is
 *  defined here once. (Rust mirror: link_types.rs::is_null_type.) */
export function isNullLinkType(id: string | undefined | null): boolean {
	return !id || id === 'associative' || id === 'relates';
}

/** Inline/badge color for a type id (neutral default for unknown ids). */
export function linkTypeColor(id: string): string {
	return byId.get(id)?.color ?? DEFAULT_COLOR;
}

/** Readable text colour (black/white) auto-contrasted against a type's fill colour, so
 *  pills/badges stay legible for ANY type — built-in or custom — without a separate
 *  per-type text store. Threshold 0.7 reproduces the 8 built-ins' original text colours. */
export function linkTypeTextColor(id: string): string {
	const hex = linkTypeColor(id).replace('#', '');
	if (hex.length < 6) return '#ffffff';
	const r = parseInt(hex.slice(0, 2), 16), g = parseInt(hex.slice(2, 4), 16), b = parseInt(hex.slice(4, 6), 16);
	const lum = (0.299 * r + 0.587 * g + 0.114 * b) / 255;
	return lum > 0.7 ? '#000000' : '#ffffff';
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
