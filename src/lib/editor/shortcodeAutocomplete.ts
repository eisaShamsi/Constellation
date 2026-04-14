/**
 * Inline `:shortcode:` autocomplete for the emoji/icon library.
 *
 * Triggered when the user types `:` followed by 1+ letters inside the
 * editor. Matches across:
 *   - Emoji shortcodes from emojibase-data (all supported locales are
 *     searched simultaneously, so `:heart:`, `:قلب:`, and `:coeur:` all
 *     hit ❤️)
 *   - Emoji labels/tags (multi-language via the locale-specific
 *     emojibase files — English is always included as a baseline)
 *   - Lucide icon names in kebab-case (e.g. `:calendar:` → 📅 or the
 *     Lucide calendar SVG)
 *
 * Lazy-loaded: data is fetched on the first `:` keystroke, cached for
 * the session.
 */

import type { CompletionContext, CompletionResult } from '@codemirror/autocomplete';

interface ShortcodeMatch {
	keyword: string;          // lowercased searchable keyword
	label: string;            // human-readable label shown in the popup
	insertion: string;        // unicode emoji OR inline SVG
	detail: string;           // short hint (emoji / icon / lucide)
	boost: number;            // sort weight
}

let _entries: ShortcodeMatch[] | null = null;
let _loading: Promise<ShortcodeMatch[]> | null = null;

// Locales available via emojibase-data (v16)
const EMOJI_LOCALES = [
	'en', 'de', 'es', 'fr', 'it', 'ja', 'ko', 'pt', 'ru', 'nl',
	'pl', 'sv', 'th', 'uk', 'hi', 'da', 'fi', 'et', 'hu', 'nb',
	'bn', 'lt', 'ms',
];

/**
 * Build the flat list of shortcode → insertion entries. Multi-locale:
 * every emoji contributes one row per locale's label + tag set, so
 * searching in any supported language finds it.
 */
async function buildEntries(): Promise<ShortcodeMatch[]> {
	const seen = new Set<string>();
	const out: ShortcodeMatch[] = [];

	// Load all emoji locales in parallel; the list is small (~23 files × 200 KB).
	// Locale import failures are silently ignored — search still works via the
	// locales that loaded.
	const localePromises = EMOJI_LOCALES.map(async (loc) => {
		try {
			const mod = await import(`emojibase-data/${loc}/compact.json`);
			return { loc, data: mod.default as any[] };
		} catch {
			return null;
		}
	});
	const localeSets = (await Promise.all(localePromises)).filter(Boolean) as { loc: string; data: any[] }[];

	// One row per (emoji × keyword). We dedupe by (keyword + unicode) so
	// identical keywords across locales don't duplicate popup entries.
	for (const { data } of localeSets) {
		for (const e of data) {
			if (!e.unicode) continue;
			const label = (e.label ?? '').toLowerCase();
			const tags: string[] = Array.isArray(e.tags) ? e.tags.map((t: string) => t.toLowerCase()) : [];
			const keywords = new Set<string>([label, ...tags]);
			for (const raw of keywords) {
				if (!raw) continue;
				const kw = raw.replace(/[^\p{L}\p{N}\-_]+/gu, '-').replace(/^-+|-+$/g, '');
				if (!kw) continue;
				const dedupKey = `${kw}|${e.unicode}`;
				if (seen.has(dedupKey)) continue;
				seen.add(dedupKey);
				out.push({
					keyword: kw,
					label: `${e.unicode}  ${e.label}`,
					insertion: e.unicode,
					detail: 'emoji',
					boost: label === raw ? 5 : 0,
				});
			}
		}
	}

	// Vector icons — insert as :set-name: shortcodes. The editor's live-preview
	// widget renders them as inline SVG. Keeps the .md file small and
	// readable; matches how emoji work (raw character, decorated).
	try {
		const { loadAllIcons } = await import('./iconSets');
		const icons = await loadAllIcons();
		const boostPerSet: Record<string, number> = {
			lucide: 0, feather: -1, heroicons: -2, phosphor: -3,
		};
		for (const icon of icons) {
			const shortcode = `:${icon.set}-${icon.name}:`;
			// Short form — `:heart:` lists every set that has a matching name.
			out.push({
				keyword: icon.name,
				label: `⎔  ${icon.id}`,
				insertion: shortcode,
				detail: icon.set,
				boost: -5 + (boostPerSet[icon.set] ?? -3),
			});
			// Namespaced form — `:lucide-heart:`, `:phosphor-heart:`, etc.
			out.push({
				keyword: `${icon.set}-${icon.name}`,
				label: `⎔  ${icon.id}`,
				insertion: shortcode,
				detail: icon.set,
				boost: -7,
			});
		}
	} catch { /* Optional — continue without vector icons */ }

	return out;
}

async function getEntries(): Promise<ShortcodeMatch[]> {
	if (_entries) return _entries;
	if (!_loading) _loading = buildEntries().then(e => { _entries = e; return e; });
	return _loading;
}

/** CompletionSource for CodeMirror 6. */
export async function shortcodeCompletion(ctx: CompletionContext): Promise<CompletionResult | null> {
	// Match `:word` where word is non-whitespace, letters/digits/Arabic/etc.
	const match = ctx.matchBefore(/:[\p{L}\p{N}_-]{1,40}/u);
	if (!match) return null;
	const query = match.text.slice(1).toLowerCase();
	if (query.length === 0 && !ctx.explicit) return null;

	const entries = await getEntries();

	// Rank: exact prefix hits first, then substring hits, alphabetical within.
	const prefix: ShortcodeMatch[] = [];
	const substring: ShortcodeMatch[] = [];
	for (const e of entries) {
		if (e.keyword.startsWith(query)) prefix.push(e);
		else if (e.keyword.includes(query)) substring.push(e);
		if (prefix.length + substring.length >= 500) break;
	}
	const seenKeyInsert = new Set<string>();
	const ranked = [...prefix, ...substring]
		.filter(e => {
			// Avoid showing duplicate (keyword, insertion) pairs — cross-locale
			// labels can produce them
			const k = `${e.keyword}|${e.insertion}`;
			if (seenKeyInsert.has(k)) return false;
			seenKeyInsert.add(k);
			return true;
		})
		.slice(0, 40);

	return {
		from: match.from,
		options: ranked.map((e) => ({
			label: `:${e.keyword}:`,
			displayLabel: e.label,
			detail: e.detail,
			apply: e.insertion,
			boost: e.boost + (e.keyword === query ? 10 : 0),
		})),
		validFor: /^:[\p{L}\p{N}_-]*$/u,
	};
}
