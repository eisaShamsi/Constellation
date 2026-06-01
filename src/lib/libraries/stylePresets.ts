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
import { appSettings, updateSettings } from './store';
import { getLinkTypes, toLinkTypeDeltas, saveLinkTypes, type LinkTypeDef } from './linkTypeRegistry';

/** Bump the minor when adding sections (back-compatible); the major when the apply
 *  contract changes (so importers can refuse a too-new file gracefully). */
export const STYLE_PRESET_SCHEMA = 'constellation-style/1';

/** The choosable sections. Order here is the display order in the save dialog. */
export type SectionKey =
	| 'colorsTheme'
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
			out[key] = { deltas: toLinkTypeDeltas(getLinkTypes()) };
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
			Object.assign(partial, payload as Record<string, unknown>);
		}
	}
	// Apply link colours FIRST (registry → live editor/panel rebuild) while the editor is
	// idle, THEN appSettings — so the appSettings reactivity can't race the registry's live
	// update (the Boss-found "themes apply live, link colours need a relaunch" bug).
	if (linkDeltas) await saveLinkTypes(linkDeltas);
	if (Object.keys(partial).length) updateSettings(partial as Parameters<typeof updateSettings>[0]);
}

/** Which sections a preset carries (catalogue order) — for display + apply summaries. */
export function presetSectionKeys(preset: StylePreset): SectionKey[] {
	return SECTION_CATALOGUE.map((s) => s.key).filter((k) => k in preset.sections);
}

/** Structural validation for an IMPORTED preset (defensive: never apply garbage). */
export function isValidPreset(x: unknown): x is StylePreset {
	if (!x || typeof x !== 'object') return false;
	const p = x as Record<string, unknown>;
	return (
		typeof p.name === 'string' &&
		typeof p.schema === 'string' &&
		p.schema.startsWith('constellation-style/') &&
		typeof p.sections === 'object' && p.sections != null && !Array.isArray(p.sections)
	);
}
