/**
 * Style Presets (frontend) — MIG-069.
 *
 * Named, app-GLOBAL bundles of *style* configuration that a user can save, switch
 * between, export to a `.json` file, and share — reusable across every universe (the
 * VS Code Profiles model, the Local-First way). The presets array is stored app-global
 * at `{app_data_dir}/style-presets.json` via the `load_style_presets`/`save_style_presets`
 * Rust commands (a dumb JSON store — the SHAPE below is owned entirely by the frontend).
 *
 * A preset captures only the SECTIONS the user ticked (section-choosable). Each section
 * maps to real config: top-level appSettings fields, the nested pill shape, or the
 * link-type registry (the portable link palette). Capture/apply live in Phase B.
 */
import { invoke } from '@tauri-apps/api/core';
import { get } from 'svelte/store';
import { appSettings, updateSettings, type ConstellationTheme } from './store';
import { getLinkTypes, toLinkTypeDeltas, saveLinkTypes, type LinkTypeDef } from './linkTypeRegistry';

/** Bump the minor when adding sections (back-compatible); the major when the apply
 *  contract changes (so importers can refuse a too-new file gracefully). */
export const STYLE_PRESET_SCHEMA = 'constellation-style/1';

/** The choosable sections. Order here is the display order in the save dialog. */
export type SectionKey =
	| 'colorsTheme'
	| 'styleOverride'
	| 'fonts'
	| 'linkColors'
	| 'pillShape'
	| 'typedLinkDisplay'
	| 'skyView'
	| 'layout'
	| 'behaviour';

/** One stored Style preset. Only the ticked sections are present in `sections`. */
export interface StylePreset {
	id: string;
	name: string;
	icon?: string;
	/** MIG-070 §A — true for the DERIVED Styles that wrap a built-in / custom Theme (shown in
	 *  the unified list; applyable + duplicable, but not renamable/deletable as such). */
	builtin?: boolean;
	/** 'builtin' | 'theme' (derived from a custom Theme) | 'style' (a saved MIG-069 Style). */
	source?: 'builtin' | 'theme' | 'style';
	/** STYLE_PRESET_SCHEMA at save time — used to validate/upgrade on import. */
	schema: string;
	createdAt?: string;
	updatedAt?: string;
	/** Section key → its captured payload. Absent section = not captured (= leave the
	 *  current universe's that-aspect untouched on apply). */
	sections: Partial<Record<SectionKey, unknown>>;
}

/** How each section is captured/applied. `appSettingsKeys` are top-level AppSettings
 *  fields copied verbatim; `special` marks the two non-appSettings sources. */
export interface SectionDef {
	key: SectionKey;
	/** i18n key (en.json: styles.section.<key>). */
	labelKey: string;
	/** Top-level AppSettings keys this section captures verbatim. */
	appSettingsKeys: string[];
	/** Non-appSettings source: the link-type registry, or the nested pill shape. */
	special?: 'linkColors' | 'pillShape';
	/** Ticked by default when saving a new preset (the "look" sections; behaviour off). */
	defaultOn: boolean;
}

/**
 * The section catalogue. PRIVACY INVARIANT: `behaviour` deliberately excludes
 * `security`/`githubToken` (never share secrets) AND universe-specific folder PATHS
 * (defaultNoteFolder, templateFolder, …) — those point at a layout the recipient may
 * not have. Only portable toggles travel.
 */
export const SECTION_CATALOGUE: SectionDef[] = [
	{
		key: 'colorsTheme', labelKey: 'styles.section.colorsTheme', defaultOn: true,
		appSettingsKeys: ['colorScheme', 'accentColor', 'activeThemeId', 'customThemes', 'iconOverrides'],
	},
	{
		// MIG-070 §C Phase 6 — the per-Universe Style Setter look: the per-element CSS-var overrides
		// (chrome + Markdown elements) + the per-script fonts. Additive section (old presets without it
		// still apply); captured/applied verbatim by the generic appSettingsKeys path. Visual-only →
		// safe to share. Lets a saved Style carry the look you designed in the Setter, not just a theme.
		key: 'styleOverride', labelKey: 'styles.section.styleOverride', defaultOn: true,
		appSettingsKeys: ['styleOverride', 'perScriptFonts'],
	},
	{
		key: 'fonts', labelKey: 'styles.section.fonts', defaultOn: true,
		appSettingsKeys: [
			'interfaceFont', 'interfaceFontSize', 'textFont', 'monoFont', 'fontSize',
			'scriptFonts', 'fontMode', 'fontTheme', 'activeFontSetId', 'languageFontSets',
			'customFontSets', 'primaryScript', 'enableSecondaryScript', 'secondaryScript', 'numeralStyle',
		],
	},
	{ key: 'linkColors', labelKey: 'styles.section.linkColors', defaultOn: true, appSettingsKeys: [], special: 'linkColors' },
	{ key: 'pillShape', labelKey: 'styles.section.pillShape', defaultOn: true, appSettingsKeys: [], special: 'pillShape' },
	{
		key: 'typedLinkDisplay', labelKey: 'styles.section.typedLinkDisplay', defaultOn: true,
		appSettingsKeys: ['colourTypedLinks', 'showTypedLinkLabels'],
	},
	{ key: 'skyView', labelKey: 'styles.section.skyView', defaultOn: false, appSettingsKeys: ['skyView'] },
	{
		key: 'layout', labelKey: 'styles.section.layout', defaultOn: false,
		appSettingsKeys: ['panelPlacements', 'leftOfNoteWidth', 'rightOfNoteWidth', 'titleAlignment', 'focus'],
	},
	{
		key: 'behaviour', labelKey: 'styles.section.behaviour', defaultOn: false,
		appSettingsKeys: [
			'showLineNumbers', 'readableLineLength', 'tabSize', 'indentWithTabs', 'smartLists',
			'autoPairBrackets', 'autoPairMarkdown', 'spellcheck', 'showFloatingToolbar', 'foldHeading',
			'foldIndent', 'indentationGuides', 'alwaysFocusNewTabs', 'propertiesInDocument',
			'linkFormat', 'autoUpdateLinks', 'useWikilinks', 'confirmDelete', 'trashDestination',
		],
	},
];

/** Lookup a section def by key. */
export function sectionDef(key: SectionKey): SectionDef | undefined {
	return SECTION_CATALOGUE.find((s) => s.key === key);
}

/** Load all presets (app-global). Returns [] on any failure — never throws to the UI. */
export async function loadStylePresets(): Promise<StylePreset[]> {
	try {
		const data = await invoke<unknown>('load_style_presets');
		return Array.isArray(data) ? (data as StylePreset[]) : [];
	} catch {
		return [];
	}
}

/** Persist the full presets array (app-global). */
export async function saveStylePresets(presets: StylePreset[]): Promise<void> {
	await invoke('save_style_presets', { presets });
}

// ─── Engine: capture / apply (MIG-069 §B) ───

/** Deep clone so a preset never aliases the live settings (structuredClone, JSON fallback). */
function clone<T>(v: T): T {
	try { return structuredClone(v); } catch { return JSON.parse(JSON.stringify(v ?? null)); }
}

/** A unique preset id (crypto.randomUUID where available; robust fallback otherwise). */
function uid(): string {
	try { return crypto.randomUUID(); } catch { return 'p-' + Date.now().toString(36) + '-' + Math.random().toString(36).slice(2, 8); }
}

/** Capture the CURRENT universe's values for `sectionKeys` into a preset payload. */
export function captureCurrentStyle(sectionKeys: SectionKey[]): Partial<Record<SectionKey, unknown>> {
	const s = get(appSettings) as unknown as Record<string, unknown>;
	const out: Partial<Record<SectionKey, unknown>> = {};
	for (const key of sectionKeys) {
		const def = sectionDef(key);
		if (!def) continue;
		if (def.special === 'linkColors') {
			// Capture the FULL resolved palette (all 8 + customs), not just deltas, so apply
			// can MERGE this whole palette into another universe (replacing the 8's colours)
			// without needing that universe to already share the same overrides.
			out[key] = { deltas: getLinkTypes().map((t) => ({ ...t })) };
		} else if (def.special === 'pillShape') {
			const lp = s.linkPills as { shape?: unknown } | undefined;
			out[key] = { shape: clone(lp?.shape ?? null) };
		} else {
			const vals: Record<string, unknown> = {};
			for (const f of def.appSettingsKeys) vals[f] = clone(s[f]);
			out[key] = vals;
		}
	}
	return out;
}

/** Build a new preset from the current universe + the chosen sections. */
export function newPresetFromCurrent(name: string, sectionKeys: SectionKey[], icon?: string): StylePreset {
	const now = new Date().toISOString();
	return {
		id: uid(),
		name: name.trim() || 'Untitled style',
		icon,
		schema: STYLE_PRESET_SCHEMA,
		createdAt: now,
		updatedAt: now,
		sections: captureCurrentStyle(sectionKeys),
	};
}

/** A copy of a preset with a fresh id + name (for "Duplicate" / accepting an import). */
export function clonePreset(p: StylePreset, newName?: string): StylePreset {
	const now = new Date().toISOString();
	return { ...clone(p), id: uid(), name: newName ?? `${p.name} copy`, createdAt: now, updatedAt: now };
}

/** Apply a preset to the CURRENT universe. Each PRESENT section is written via the
 *  existing rails — appSettings sections merge into one `updateSettings` (auto-saves +
 *  notifies the second screen); the link-colours section goes through `saveLinkTypes`
 *  (re-seeds the registry → editor + panels update live). Sections ABSENT from the
 *  preset are left untouched (partial apply). */
export async function applyPreset(preset: StylePreset): Promise<void> {
	const partial: Record<string, unknown> = {};
	let linkDeltas: LinkTypeDef[] | null = null;
	for (const k of Object.keys(preset.sections) as SectionKey[]) {
		const def = sectionDef(k);
		const payload = preset.sections[k];
		if (!def || payload == null) continue;
		if (def.special === 'linkColors') {
			const d = (payload as { deltas?: unknown }).deltas;
			if (Array.isArray(d)) linkDeltas = d as LinkTypeDef[];
		} else if (def.special === 'pillShape') {
			const shape = (payload as { shape?: unknown }).shape;
			const cur = (get(appSettings) as { linkPills?: Record<string, unknown> }).linkPills ?? {};
			partial.linkPills = { ...cur, shape };
		} else {
			// Allow-list: only write the fields this section OWNS (symmetric with capture),
			// so a hand-edited/foreign file can't inject arbitrary settings keys (audit §F).
			const pl = payload as Record<string, unknown>;
			for (const f of def.appSettingsKeys) if (f in pl) partial[f] = pl[f];
		}
	}
	// MIG-070 §A — applying a colours/theme section MERGES its customThemes into the universe's
	// library (the Style wins on id), never REPLACES it — so switching theme can't drop the
	// universe's other custom themes (the same non-destructive rule as the link colours below).
	if (Array.isArray(partial.customThemes)) {
		const curThemes = (get(appSettings) as { customThemes?: ConstellationTheme[] }).customThemes ?? [];
		const byId = new Map(curThemes.map((t) => [t.id, t] as [string, ConstellationTheme]));
		for (const t of partial.customThemes as ConstellationTheme[]) byId.set(t.id, t);
		partial.customThemes = [...byId.values()];
	}
	// Apply link colours FIRST (registry → live editor/panel rebuild) while the editor is
	// idle, THEN appSettings — so the appSettings reactivity can't race the registry's live
	// update (the Boss-found "themes apply live, link colours need a relaunch" bug).
	if (linkDeltas) {
		// MERGE the Style's palette into the current universe's link types — never replace —
		// so applying a Style can't DELETE custom types this universe already has (audit §F:
		// the headline data-loss risk). The Style wins on id conflicts; the universe keeps
		// its own non-conflicting custom types.
		const byId = new Map(getLinkTypes().map((d) => [d.id, { ...d } as LinkTypeDef]));
		for (const d of linkDeltas) byId.set(d.id, d);
		await saveLinkTypes(toLinkTypeDeltas([...byId.values()]));
	}
	if (Object.keys(partial).length) updateSettings(partial as Parameters<typeof updateSettings>[0]);
}

/** Which sections a preset carries (catalogue order) — for display + apply summaries. */
export function presetSectionKeys(preset: StylePreset): SectionKey[] {
	return SECTION_CATALOGUE.map((s) => s.key).filter((k) => k in preset.sections);
}

// ─── MIG-071 — Themes removed. The unify-Themes-+-Styles scaffold (themeToStyle / unifiedStyleList /
//     isUserStyle / isBaseStyle / resolveActiveBase) and the preview-card helper (stylePreview /
//     StylePreview) are gone: there are no themes to assemble, and the gallery/preview cards that
//     used them were removed. Saved Styles ARE the looks now; they apply via applyPreset (below). ───

/** The newest schema MAJOR this build can apply; a file from a newer major is refused. */
const SUPPORTED_MAJOR = 1;

/** Structural validation for an IMPORTED preset (defensive: never apply garbage, and
 *  refuse a file from a newer Constellation whose apply-contract we don't understand). */
export function isValidPreset(x: unknown): x is StylePreset {
	if (!x || typeof x !== 'object') return false;
	const p = x as Record<string, unknown>;
	if (typeof p.name !== 'string' || typeof p.schema !== 'string') return false;
	if (!p.schema.startsWith('constellation-style/')) return false;
	const major = parseInt(p.schema.slice('constellation-style/'.length), 10);
	if (!Number.isFinite(major) || major > SUPPORTED_MAJOR) return false; // too new → refuse gracefully
	return typeof p.sections === 'object' && p.sections != null && !Array.isArray(p.sections);
}

// ─── Export / import (MIG-069 §D) ───

/** A filesystem-safe stem from a preset name. */
function slugName(n: string): string {
	return (n || 'style').trim().replace(/[^\w-]+/g, '-').replace(/^-+|-+$/g, '') || 'style';
}

/** Export a preset to a user-chosen `.json` file. Returns true if saved, false if cancelled. */
export async function exportPreset(preset: StylePreset): Promise<boolean> {
	return await invoke<boolean>('export_style_preset', { preset, suggestedName: slugName(preset.name) });
}

/** Import a preset from a user-chosen `.json` file: returns a validated preset with a FRESH
 *  id (keeping its name), `null` if cancelled, or throws if the file isn't a valid style. */
export async function importPreset(): Promise<StylePreset | null> {
	const v = await invoke<unknown>('import_style_preset');
	if (v == null) return null; // cancelled
	if (!isValidPreset(v)) throw new Error('invalid-style');
	return clonePreset(v as StylePreset, (v as StylePreset).name);
}
