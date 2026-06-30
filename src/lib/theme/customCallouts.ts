/**
 * Custom callout types — MIG-089 Phase B.
 *
 * A per-Universe registry of user-defined callout types. Each entry gives a
 * sanitised trigger `slug` (the `[!slug]` word), a friendly `name`, a `color`
 * (hex, injected inline as --callout-color by calloutPlugin) and an `icon`
 * (an emoji char or a "lucide:heart"-style ref, resolved like icon overrides).
 *
 * Stored in appSettings.customCallouts → per-Universe settings.json, so the
 * callout vocabulary travels with the Universe (the Obsidian per-vault norm).
 * Reads are synchronous (peek*) for use inside the CM6 plugin; writes go
 * through updateSettings (the cross-window settings-propagation path).
 */

import { get } from 'svelte/store';
import { appSettings } from '$lib/libraries/store';
import { isBuiltinCalloutType } from '$lib/editor/calloutFamilies';

export interface CustomCallout {
	/** Sanitised trigger word — the `[!slug]` in markdown. Lowercase, hyphenated. */
	slug: string;
	/** Friendly display name shown in the Style Setter list. */
	name: string;
	/** Hex colour injected inline as --callout-color. */
	color: string;
	/** Icon ref — an emoji char or a namespaced id ("lucide:heart"). */
	icon: string;
}

/** Sanitise a raw trigger word into a safe callout slug: lowercase, every run of
 *  non-[a-z0-9] collapses to a single hyphen, leading/trailing hyphens stripped.
 *  (The slug becomes a `data-callout` attribute value, so it must be DOM-safe.) */
export function sanitizeCalloutSlug(raw: string): string {
	return (raw || '').toLowerCase().replace(/[^a-z0-9]+/g, '-').replace(/^-+|-+$/g, '');
}

/** Synchronous read of the whole registry (empty array if unset). */
export function peekCustomCallouts(): CustomCallout[] {
	return get(appSettings).customCallouts ?? [];
}

/** Synchronous lookup of one custom callout by its slug, or null. */
export function peekCustomCallout(slug: string): CustomCallout | null {
	const s = (slug || '').toLowerCase();
	return peekCustomCallouts().find((c) => c.slug === s) ?? null;
}

export type SlugStatus = 'ok' | 'empty' | 'builtin' | 'duplicate';

/** Validate a candidate slug for the Add form. `existing` lets edit-in-place
 *  exclude the row being edited from the duplicate check. */
export function slugStatus(slug: string, existing?: string): SlugStatus {
	if (!slug) return 'empty';
	if (isBuiltinCalloutType(slug)) return 'builtin';
	if (slug !== existing && peekCustomCallouts().some((c) => c.slug === slug)) return 'duplicate';
	return 'ok';
}

function persist(list: CustomCallout[]) {
	import('$lib/libraries/store').then(({ updateSettings }) => {
		updateSettings({ customCallouts: list });
	});
}

/** Add or replace (by slug) a custom callout. */
export function addCustomCallout(c: CustomCallout) {
	const list = peekCustomCallouts().filter((x) => x.slug !== c.slug);
	persist([...list, c]);
}

/** Patch an existing custom callout (matched by slug). */
export function updateCustomCallout(slug: string, patch: Partial<CustomCallout>) {
	persist(peekCustomCallouts().map((c) => (c.slug === slug ? { ...c, ...patch } : c)));
}

/** Remove a custom callout (its `[!slug]` text in notes is untouched and reverts to the note look). */
export function removeCustomCallout(slug: string) {
	persist(peekCustomCallouts().filter((c) => c.slug !== slug));
}
