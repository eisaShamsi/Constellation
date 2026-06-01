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
