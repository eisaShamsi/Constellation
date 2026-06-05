<script lang="ts">
	/**
	 * Constellation Style Setter (CSS) — MIG-070, standalone, built from scratch.
	 *
	 * A full-page "design studio": your real interface in the centre, click any part to style
	 * it, controls on the right, surfaces + theme cards on the left. Edits go to a DRAFT (CSS
	 * variable overrides scoped to the preview wrapper — the live app is untouched); **Apply**
	 * copies the draft onto the real <body>. Deliberately independent of the old MIG-069 style
	 * code, and it renders ONE preview (never a gallery of heavy cards — that froze the old panel).
	 *
	 * Iteration 1: live-edit the core variables (accent · backgrounds · text · link · fonts).
	 * Iteration 2 §3: every Markdown element editable — Headers H1–H6, bold, italic, strikethrough,
	 * inline code, blockquote — each with colour / size / weight, mapped to the REAL editor vars in
	 * `livePreview.ts`'s `livePreviewTheme` + `constellationStyleSettings.ts`. The centre preview is
	 * a richer mini-note that renders those elements so each is clickable. (Blockquote bar, list
	 * markers, and table rendering are §3C — they need new editor decorations, cross-checked first.)
	 */
	import { onMount } from 'svelte';
	import { get } from 'svelte/store';
	import { styleSetterOpen, closeStyleSetter } from '$lib/stores/styleSetter';
	import { appSettings, mergeStyleOverride, clearAllStyleOverride, addStyleSwatch, removeStyleSwatch, setPerScriptFont, updateSettings, setLiveStyleDraft, clearLiveStyleDraft } from '$lib/libraries/store';
	// §C Phase 5 — link styling reuses the EXISTING single source: the §G Link-Types editor (one save
	// path → Backlinks/Outgoing/editor recolour live). Display toggles + pill shape are appSettings.
	import LinkTypesEditor from './LinkTypesEditor.svelte';
	// MIG-070 §C Phase 6 — named, reusable Styles (the frozen MIG-069 SAVE/APPLY engine, reused as-is —
	// the same calls StylePresetsPanel uses). NOTE: we deliberately do NOT import unifiedStyleList /
	// stylePreview here — rendering BUILTIN_THEMES through themeToStyle as a gallery of self-portrait
	// cards is the documented main-thread FREEZE shape that the clean-slate Setter exists to avoid
	// (orientation v2.49; LL-014). The Setter lists only the user's SAVED styles, as lightweight rows.
	import { loadStylePresets, saveStylePresets, newPresetFromCurrent, applyPreset, SECTION_CATALOGUE, type StylePreset } from '$lib/libraries/stylePresets';

	// A control writes one REAL app CSS variable. `color` → hex; `select` → a stack/keyword;
	// `range` → a number + unit (e.g. `32px`, or `700` when unit is '').
	type Ctrl =
		| { label: string; type: 'color'; var: string }
		| { label: string; type: 'select'; var: string; options: [string, string][] }
		| { label: string; type: 'range'; var: string; min: number; max: number; step: number; unit: string; def: number }
		| { label: string; type: 'scriptfont'; script: string; options: [string, string][] }  // writes appSettings.perScriptFonts[script]
		// §C Phase 5 — settings-backed controls (write appSettings immediately, like scriptfont — NOT
		// the per-Universe styleOverride draft): a boolean toggle, and the link-pill shape (radius/height
		// as ranges, fontWeight as a select). They flow through updateSettings → every panel reacts.
		| { label: string; type: 'toggle'; setting: 'colourTypedLinks' | 'showTypedLinkLabels' }
		| { label: string; type: 'pillrange'; prop: 'radius' | 'height'; min: number; max: number; step: number; unit: string }
		| { label: string; type: 'pillselect'; prop: 'fontWeight'; options: [string, string][] };

	// §C Phase 4 — a curated typeface list (cross-platform stacks). A full installed-fonts list +
	// per-script fonts + font-theme/numerals (from the Language tab) are the deeper follow-up.
	const FONTS: [string, string][] = [
		['System', 'ui-sans-serif, system-ui, "Segoe UI", sans-serif'],
		['Serif', 'ui-serif, Georgia, "Times New Roman", serif'],
		['Mono', 'ui-monospace, "Courier New", monospace'],
		['Segoe UI', '"Segoe UI", system-ui, sans-serif'],
		['Calibri', 'Calibri, "Segoe UI", sans-serif'],
		['Helvetica', '"Helvetica Neue", Helvetica, Arial, sans-serif'],
		['Verdana', 'Verdana, Geneva, sans-serif'],
		['Tahoma', 'Tahoma, sans-serif'],
		['Trebuchet', '"Trebuchet MS", sans-serif'],
		['Georgia', 'Georgia, "Times New Roman", serif'],
		['Times', '"Times New Roman", Times, serif'],
		['Garamond', 'Garamond, "EB Garamond", Georgia, serif'],
		['Consolas', 'Consolas, "Courier New", monospace'],
		['Courier', '"Courier New", Courier, monospace'],
	];
	const DECOR: [string, string][] = [
		['Underline', 'underline'],
		['None', 'none'],
		['Dotted', 'underline dotted'],
	];
	// Border styles (shared by every element's border controls — §3B full set).
	const BORDER_STYLE: [string, string][] = [
		['Solid', 'solid'], ['Dashed', 'dashed'], ['Dotted', 'dotted'], ['None', 'none'],
	];
	// §C Phase 4.2 — per-script font choices (each script its own face). The interface LANGUAGE
	// stays in Settings → Language (a locale setting, not styling) — Eisa's call.
	const AR_FONTS: [string, string][] = [['System default', ''], ['Noto Naskh Arabic', '"Noto Naskh Arabic"'], ['Amiri', 'Amiri'], ['Scheherazade New', '"Scheherazade New"'], ['Cairo', 'Cairo'], ['Dubai', 'Dubai'], ['Tahoma', 'Tahoma'], ['Segoe UI', '"Segoe UI"'], ['Traditional Arabic', '"Traditional Arabic"']];
	const HE_FONTS: [string, string][] = [['System default', ''], ['Noto Sans Hebrew', '"Noto Sans Hebrew"'], ['David', 'David'], ['Frank Ruehl', 'FrankRuehl'], ['Arial', 'Arial'], ['Times New Roman', '"Times New Roman"']];
	const CJK_FONTS: [string, string][] = [['System default', ''], ['Noto Sans CJK SC', '"Noto Sans CJK SC"'], ['Microsoft YaHei', '"Microsoft YaHei"'], ['SimSun', 'SimSun'], ['Malgun Gothic', '"Malgun Gothic"'], ['MS Gothic', '"MS Gothic"']];
	const DEV_FONTS: [string, string][] = [['System default', ''], ['Noto Sans Devanagari', '"Noto Sans Devanagari"'], ['Mangal', 'Mangal'], ['Nirmala UI', '"Nirmala UI"']];
	const CYR_FONTS: [string, string][] = [['System default', ''], ['Noto Sans', '"Noto Sans"'], ['Segoe UI', '"Segoe UI"'], ['Times New Roman', '"Times New Roman"'], ['Arial', 'Arial']];
	// Shared "weight (all headings)" control — `--heading-weight` is one var for every H level.
	const HW: Ctrl = { label: 'Weight (all headings)', type: 'range', var: '--heading-weight', min: 300, max: 900, step: 100, unit: '', def: 700 };

	// Element key → its name + controls (each control writes a REAL app CSS variable). The
	// heading/emphasis/code/quote vars are read by `livePreviewTheme` in livePreview.ts; sizes
	// default to the `constellationStyleSettings.ts` catalog defaults so the preview looks right
	// before any edit, and colours default to `inherit` (unset = today's look, no regression).
	const ELEMENTS: Record<string, { name: string; controls: Ctrl[] }> = {
		// Interface (the app chrome) is the FIRST core element (Eisa). Its text colour writes the
		// global --text-normal; the NOTE has its own --editor-text-color, so styling the note no
		// longer bleeds into the file tree / sidebars (which fall back to --text-normal).
		interface: { name: 'Interface', controls: [
			{ label: 'Interface text colour', type: 'color', var: '--text-normal' },
			{ label: 'Interface font', type: 'select', var: '--font-interface-theme', options: FONTS },
			{ label: 'Panel background', type: 'color', var: '--background-secondary' } ] },
		accent:  { name: 'Accent',   controls: [{ label: 'Accent colour', type: 'color', var: '--interactive-accent' }] },
		// §3B — File tree (#6), full set. Background = the Interface panel background (shared);
		// these are the tree-specific knobs. Row separators default to 0 width (invisible).
		fileTree: { name: 'File tree', controls: [
			{ label: 'Text colour', type: 'color', var: '--ft-master-color' },
			{ label: 'Font', type: 'select', var: '--ft-master-font-family', options: FONTS },
			{ label: 'Font size', type: 'range', var: '--ft-master-font-size', min: 10, max: 22, step: 1, unit: 'px', def: 13 },
			{ label: 'Font weight', type: 'range', var: '--ft-master-weight', min: 300, max: 900, step: 100, unit: '', def: 400 },
			{ label: 'Row spacing', type: 'range', var: '--ft-master-row-padding-y', min: 0, max: 12, step: 1, unit: 'px', def: 2 },
			{ label: 'Row radius', type: 'range', var: '--ft-row-radius', min: 0, max: 14, step: 1, unit: 'px', def: 3 },
			{ label: 'Separator width', type: 'range', var: '--ft-border-width', min: 0, max: 4, step: 1, unit: 'px', def: 0 },
			{ label: 'Separator style', type: 'select', var: '--ft-border-style', options: BORDER_STYLE },
			{ label: 'Separator colour', type: 'color', var: '--ft-border-color' } ] },
		// §3B G1 — sidebar row TYPES, each its own element (Eisa). Each overrides the File-tree
		// master (--ft-master-*); unset = follows the master = today's look.
		library: { name: 'Library', controls: [
			{ label: 'Text colour', type: 'color', var: '--ft-library-color' },
			{ label: 'Font', type: 'select', var: '--ft-library-font-family', options: FONTS },
			{ label: 'Font size', type: 'range', var: '--ft-library-font-size', min: 10, max: 22, step: 1, unit: 'px', def: 13 },
			{ label: 'Font weight', type: 'range', var: '--ft-library-weight', min: 300, max: 900, step: 100, unit: '', def: 600 } ] },
		folder: { name: 'Folder', controls: [
			{ label: 'Text colour', type: 'color', var: '--ft-folder-color' },
			{ label: 'Font', type: 'select', var: '--ft-folder-font-family', options: FONTS },
			{ label: 'Font size', type: 'range', var: '--ft-folder-font-size', min: 10, max: 22, step: 1, unit: 'px', def: 13 },
			{ label: 'Font weight', type: 'range', var: '--ft-folder-weight', min: 300, max: 900, step: 100, unit: '', def: 400 } ] },
		cuniverse: { name: 'cUniverse', controls: [
			{ label: 'Text colour', type: 'color', var: '--ft-cuniverse-color' },
			{ label: 'Font', type: 'select', var: '--ft-cuniverse-font-family', options: FONTS },
			{ label: 'Font size', type: 'range', var: '--ft-cuniverse-font-size', min: 10, max: 22, step: 1, unit: 'px', def: 13 },
			{ label: 'Font weight', type: 'range', var: '--ft-cuniverse-weight', min: 300, max: 900, step: 100, unit: '', def: 600 } ] },
		// §3B — Universe switcher ("◊ Universe", sidebar foot) + Status bar (bottom).
		universe: { name: 'Universe bar', controls: [
			{ label: 'Text colour', type: 'color', var: '--universe-bar-color' },
			{ label: 'Background', type: 'color', var: '--universe-bar-bg' },
			{ label: 'Font', type: 'select', var: '--universe-bar-font-family', options: FONTS },
			{ label: 'Text size', type: 'range', var: '--universe-bar-font-size', min: 9, max: 18, step: 1, unit: 'px', def: 12 } ] },
		statusbar: { name: 'Status bar', controls: [
			{ label: 'Background', type: 'color', var: '--statusbar-bg' },
			{ label: 'Text colour', type: 'color', var: '--statusbar-color' },
			{ label: 'Font size', type: 'range', var: '--statusbar-font-size', min: 9, max: 18, step: 1, unit: 'px', def: 11 },
			{ label: 'Height', type: 'range', var: '--statusbar-height', min: 18, max: 48, step: 1, unit: 'px', def: 24 } ] },
		noteBg:  { name: 'Note background', controls: [{ label: 'Background', type: 'color', var: '--background-primary' }] },
		text:    { name: 'Body text', controls: [
			{ label: 'Text colour', type: 'color', var: '--editor-text-color' },
			{ label: 'Note font', type: 'select', var: '--font-text-theme', options: FONTS },
			{ label: 'Text size', type: 'range', var: '--font-text-size', min: 11, max: 28, step: 1, unit: 'px', def: 16 } ] },
		link:    { name: 'Link', controls: [
			{ label: 'Link colour', type: 'color', var: '--link-color' },
			{ label: 'Underline', type: 'select', var: '--link-decoration', options: DECOR } ] },
		h1: { name: 'Heading 1', controls: [
			{ label: 'Colour', type: 'color', var: '--h1-color' },
			{ label: 'Size', type: 'range', var: '--h1-size', min: 18, max: 60, step: 1, unit: 'px', def: 32 }, HW ] },
		h2: { name: 'Heading 2', controls: [
			{ label: 'Colour', type: 'color', var: '--h2-color' },
			{ label: 'Size', type: 'range', var: '--h2-size', min: 16, max: 48, step: 1, unit: 'px', def: 26 }, HW ] },
		h3: { name: 'Heading 3', controls: [
			{ label: 'Colour', type: 'color', var: '--h3-color' },
			{ label: 'Size', type: 'range', var: '--h3-size', min: 14, max: 40, step: 1, unit: 'px', def: 22 }, HW ] },
		h4: { name: 'Heading 4', controls: [
			{ label: 'Colour', type: 'color', var: '--h4-color' },
			{ label: 'Size', type: 'range', var: '--h4-size', min: 13, max: 32, step: 1, unit: 'px', def: 18 }, HW ] },
		h5: { name: 'Heading 5', controls: [
			{ label: 'Colour', type: 'color', var: '--h5-color' },
			{ label: 'Size', type: 'range', var: '--h5-size', min: 12, max: 28, step: 1, unit: 'px', def: 16 }, HW ] },
		h6: { name: 'Heading 6', controls: [
			{ label: 'Colour', type: 'color', var: '--h6-color' },
			{ label: 'Size', type: 'range', var: '--h6-size', min: 11, max: 24, step: 1, unit: 'px', def: 14 }, HW ] },
		bold:   { name: 'Bold', controls: [
			{ label: 'Colour', type: 'color', var: '--bold-color' },
			{ label: 'Weight', type: 'range', var: '--bold-weight', min: 500, max: 900, step: 100, unit: '', def: 700 } ] },
		italic: { name: 'Italic', controls: [{ label: 'Colour', type: 'color', var: '--italic-color' }] },
		strike: { name: 'Strikethrough', controls: [
			{ label: 'Line colour', type: 'color', var: '--strikethrough-color' },
			{ label: 'Line thickness', type: 'range', var: '--strikethrough-thickness', min: 1, max: 6, step: 1, unit: 'px', def: 1 } ] },
		code:   { name: 'Inline code', controls: [
			{ label: 'Background', type: 'color', var: '--code-background' },
			{ label: 'Text colour', type: 'color', var: '--code-normal' },
			{ label: 'Code font', type: 'select', var: '--font-monospace-theme', options: FONTS },
			{ label: 'Code size', type: 'range', var: '--font-monospace-size', min: 10, max: 22, step: 1, unit: 'px', def: 14 } ] },
		quote:  { name: 'Blockquote', controls: [{ label: 'Text colour', type: 'color', var: '--blockquote-text-color' }] },
		// §C Phase 3 — global/foundational look (catalog vars already consumed app-wide).
		gBackgrounds: { name: 'Backgrounds', controls: [
			{ label: 'Background (alt)', type: 'color', var: '--background-primary-alt' },
			{ label: 'Surface (alt)', type: 'color', var: '--background-secondary-alt' },
			{ label: 'Hover background', type: 'color', var: '--background-modifier-hover' },
			{ label: 'Border / dividers', type: 'color', var: '--background-modifier-border' },
			{ label: 'Input field', type: 'color', var: '--background-modifier-form-field' } ] },
		gTextShades: { name: 'Text shades', controls: [
			{ label: 'Muted text', type: 'color', var: '--text-muted' },
			{ label: 'Faint text', type: 'color', var: '--text-faint' },
			{ label: 'Text on accent', type: 'color', var: '--text-on-accent' } ] },
		gStatus: { name: 'Status colours', controls: [
			{ label: 'Error', type: 'color', var: '--text-error' },
			{ label: 'Warning', type: 'color', var: '--text-warning' },
			{ label: 'Success', type: 'color', var: '--text-success' } ] },
		gAccent: { name: 'Accent shades', controls: [
			{ label: 'Accent (hover)', type: 'color', var: '--interactive-accent-hover' },
			{ label: 'Accent text', type: 'color', var: '--text-accent' } ] },
		gType: { name: 'Type & rhythm', controls: [
			{ label: 'Interface font size', type: 'range', var: '--font-interface-size', min: 11, max: 20, step: 1, unit: 'px', def: 14 },
			{ label: 'Line height', type: 'range', var: '--line-height-normal', min: 1.1, max: 2.2, step: 0.05, unit: '', def: 1.6 },
			{ label: 'Tight line height', type: 'range', var: '--line-height-tight', min: 1.0, max: 1.8, step: 0.05, unit: '', def: 1.3 },
			{ label: 'Paragraph spacing', type: 'range', var: '--paragraph-spacing', min: 0, max: 32, step: 1, unit: 'px', def: 12 } ] },
		gShape: { name: 'Shape & corners', controls: [
			{ label: 'Small radius', type: 'range', var: '--radius-s', min: 0, max: 20, step: 1, unit: 'px', def: 4 },
			{ label: 'Medium radius', type: 'range', var: '--radius-m', min: 0, max: 24, step: 1, unit: 'px', def: 8 },
			{ label: 'Large radius', type: 'range', var: '--radius-l', min: 0, max: 32, step: 1, unit: 'px', def: 12 },
			{ label: 'Border width', type: 'range', var: '--border-width', min: 0, max: 4, step: 1, unit: 'px', def: 1 },
			{ label: 'Reading width', type: 'range', var: '--file-line-width', min: 40, max: 120, step: 1, unit: 'ch', def: 70 },
			{ label: 'Note margins', type: 'range', var: '--file-margins', min: 0, max: 80, step: 1, unit: 'px', def: 24 } ] },
		// §C Phase 4.2 — interface language + per-script fonts (Latin = the Interface/Note/Code font
		// pickers; these are the non-Latin scripts, each rendered in its own font via the engine).
		fonts: { name: 'Per-script fonts', controls: [
			{ label: 'Arabic font', type: 'scriptfont', script: 'arabic', options: AR_FONTS },
			{ label: 'Hebrew font', type: 'scriptfont', script: 'hebrew', options: HE_FONTS },
			{ label: 'CJK font (中日韓)', type: 'scriptfont', script: 'cjk', options: CJK_FONTS },
			{ label: 'Devanagari font', type: 'scriptfont', script: 'devanagari', options: DEV_FONTS },
			{ label: 'Cyrillic font', type: 'scriptfont', script: 'cyrillic', options: CYR_FONTS } ] },
		// §C Phase 3 — chrome Components (existing catalog vars, consumed by the app chrome).
		cDock: { name: 'Ribbon dock', controls: [
			{ label: 'Background', type: 'color', var: '--dock-bg' },
			{ label: 'Icon colour', type: 'color', var: '--dock-btn-color' },
			{ label: 'Dock width', type: 'range', var: '--dock-width', min: 32, max: 72, step: 1, unit: 'px', def: 40 },
			{ label: 'Button size', type: 'range', var: '--dock-btn-size', min: 24, max: 56, step: 1, unit: 'px', def: 32 },
			{ label: 'Icon size', type: 'range', var: '--dock-icon-size', min: 12, max: 32, step: 1, unit: 'px', def: 18 },
			{ label: 'Button radius', type: 'range', var: '--dock-btn-radius', min: 0, max: 16, step: 1, unit: 'px', def: 4 } ] },
		cToolbar: { name: 'Sidebar toolbar', controls: [
			{ label: 'Background', type: 'color', var: '--sidebar-toolbar-bg' },
			{ label: 'Icon colour', type: 'color', var: '--sidebar-btn-color' },
			{ label: 'Toolbar height', type: 'range', var: '--sidebar-toolbar-height', min: 26, max: 60, step: 1, unit: 'px', def: 34 },
			{ label: 'Button size', type: 'range', var: '--sidebar-btn-size', min: 20, max: 40, step: 1, unit: 'px', def: 26 },
			{ label: 'Icon size', type: 'range', var: '--sidebar-icon-size', min: 10, max: 28, step: 1, unit: 'px', def: 16 },
			{ label: 'Button radius', type: 'range', var: '--sidebar-btn-radius', min: 0, max: 14, step: 1, unit: 'px', def: 3 } ] },
		cLayoutBar: { name: 'Layout bar', controls: [
			{ label: 'Background', type: 'color', var: '--layout-bar-bg' },
			{ label: 'Icon colour', type: 'color', var: '--layout-btn-color' },
			{ label: 'Icon colour (active)', type: 'color', var: '--layout-btn-active-color' },
			{ label: 'Bar height', type: 'range', var: '--layout-bar-height', min: 26, max: 60, step: 1, unit: 'px', def: 34 },
			{ label: 'Button size', type: 'range', var: '--layout-btn-size', min: 20, max: 44, step: 1, unit: 'px', def: 28 },
			{ label: 'Icon size', type: 'range', var: '--layout-icon-size', min: 10, max: 28, step: 1, unit: 'px', def: 14 },
			{ label: 'Button radius', type: 'range', var: '--layout-btn-radius', min: 0, max: 14, step: 1, unit: 'px', def: 4 } ] },
		cTabs: { name: 'Top bar & tabs', controls: [
			{ label: 'Top bar background', type: 'color', var: '--topbar-bg' },
			{ label: 'Top bar height', type: 'range', var: '--topbar-height', min: 28, max: 80, step: 1, unit: 'px', def: 38 },
			{ label: 'Tab background', type: 'color', var: '--tab-bg' },
			{ label: 'Tab text', type: 'color', var: '--tab-color' },
			{ label: 'Active tab background', type: 'color', var: '--tab-active-bg' },
			{ label: 'Active tab text', type: 'color', var: '--tab-active-color' },
			{ label: 'Tab border', type: 'color', var: '--tab-border' },
			{ label: 'Tab font size', type: 'range', var: '--tab-font-size', min: 10, max: 20, step: 1, unit: 'px', def: 13 },
			{ label: 'Tab height', type: 'range', var: '--tab-height', min: 22, max: 48, step: 1, unit: 'px', def: 26 },
			{ label: 'Tab radius', type: 'range', var: '--tab-radius', min: 0, max: 16, step: 1, unit: 'px', def: 6 } ] },
		cRightSidebar: { name: 'Right sidebar', controls: [
			{ label: 'Background', type: 'color', var: '--right-sidebar-bg' },
			{ label: 'Tab row background', type: 'color', var: '--rs-tabs-bg' },
			{ label: 'Tab icon colour', type: 'color', var: '--rs-tab-color' },
			{ label: 'Tab icon (active)', type: 'color', var: '--rs-tab-active-color' },
			{ label: 'Tab row height', type: 'range', var: '--rs-tab-height', min: 24, max: 56, step: 1, unit: 'px', def: 30 },
			{ label: 'Tab icon size', type: 'range', var: '--rs-icon-size', min: 10, max: 28, step: 1, unit: 'px', def: 16 } ] },
		cButtons: { name: 'Buttons', controls: [
			{ label: 'Radius', type: 'range', var: '--button-radius', min: 0, max: 24, step: 1, unit: 'px', def: 6 },
			{ label: 'Padding (horizontal)', type: 'range', var: '--button-padding-x', min: 4, max: 32, step: 1, unit: 'px', def: 12 },
			{ label: 'Padding (vertical)', type: 'range', var: '--button-padding-y', min: 2, max: 20, step: 1, unit: 'px', def: 6 } ] },
		cTags: { name: 'Tags & callouts', controls: [
			{ label: 'Tag background', type: 'color', var: '--tag-bg' },
			{ label: 'Tag text', type: 'color', var: '--tag-color' },
			{ label: 'Tag radius', type: 'range', var: '--tag-radius', min: 0, max: 24, step: 1, unit: 'px', def: 12 },
			{ label: 'Callout radius', type: 'range', var: '--callout-radius', min: 0, max: 24, step: 1, unit: 'px', def: 8 } ] },
		cSidebar: { name: 'Sidebar shell', controls: [
			{ label: 'Width', type: 'range', var: '--sidebar-width', min: 180, max: 420, step: 2, unit: 'px', def: 260 },
			{ label: 'Background', type: 'color', var: '--sidebar-bg' } ] },
		// §C Phase 5 + redesign — Links is ONE integrated surface (left sidebar + one right space, no
		// centre — Eisa). The display controls (toggles + pill shape) sit above the §G Link-Types editor
		// (the single colour source); each editor row shows its LIVE pill, so the control IS the preview
		// (the pill reflects colour + shape live) — no separate pill row / in-editor block to duplicate.
		// All write their own stores directly (registry / appSettings), bypassing the CSS-var draft.
		links: { name: 'Links', controls: [
			{ label: 'Colour typed links', type: 'toggle', setting: 'colourTypedLinks' },
			{ label: 'Show type labels', type: 'toggle', setting: 'showTypedLinkLabels' },
			{ label: 'Pill corner radius', type: 'pillrange', prop: 'radius', min: 0, max: 20, step: 1, unit: 'px' },
			{ label: 'Pill height', type: 'pillrange', prop: 'height', min: 14, max: 32, step: 1, unit: 'px' },
			{ label: 'Pill label weight', type: 'pillselect', prop: 'fontWeight', options: [['Normal', '400'], ['Semibold', '600'], ['Bold', '700'], ['Extra bold', '900']] } ] },
	};
	// §3B — the left rail is organised into CATEGORIES (a.k.a. Surfaces), each grouping its
	// elements (Eisa). Interface + Editor both preview the main app window ('editor' surface);
	// the heavy plugins are their own preview surfaces.
	const CATEGORIES: { key: string; name: string; surface: string; elements: string[] }[] = [
		{ key: 'interface', name: 'Interface', surface: 'editor', elements: ['interface', 'fileTree', 'library', 'folder', 'cuniverse', 'universe', 'statusbar'] },
		{ key: 'components', name: 'Components', surface: 'editor', elements: ['cDock', 'cToolbar', 'cLayoutBar', 'cTabs', 'cRightSidebar', 'cButtons', 'cTags', 'cSidebar'] },
		{ key: 'editor', name: 'Editor', surface: 'editor', elements: ['noteBg', 'text', 'accent', 'link', 'h1', 'h2', 'h3', 'h4', 'h5', 'h6', 'bold', 'italic', 'strike', 'code', 'quote'] },
		{ key: 'global', name: 'Global', surface: 'editor', elements: ['gBackgrounds', 'gTextShades', 'gStatus', 'gAccent', 'gType', 'gShape', 'fonts'] },
		{ key: 'links', name: 'Links', surface: 'editor', elements: ['links'] },
		{ key: 'sky', name: 'Sky View', surface: 'sky', elements: ['accent', 'link'] },
		{ key: 'org', name: 'OrgChart', surface: 'org', elements: ['accent', 'link'] },
		{ key: 'index', name: 'Index', surface: 'index', elements: ['accent'] },
		{ key: 'cataloger', name: 'Cataloger', surface: 'cataloger', elements: ['accent'] },
		{ key: 'shell', name: 'Shell', surface: 'shell', elements: ['accent'] },
	];
	// element key → its (first) category, so clicking a part in the preview opens the right category.
	const CATEGORY_OF: Record<string, string> = {};
	for (const c of CATEGORIES) for (const e of c.elements) if (!(e in CATEGORY_OF)) CATEGORY_OF[e] = c.key;

	// MIG-070 §C Phase 6 — named, reusable Styles. The gallery is the UNIFIED list (built-in theme
	// Styles + this Universe's custom-theme Styles + the user's saved Styles); clicking one APPLIES it
	// (non-destructive merge), "+ Save current" captures the look incl. the new styleOverride section.
	let savedStyles = $state<StylePreset[]>([]);

	let activeSurface = $state('editor');
	let activeCategory = $state('interface');
	let selected = $state<string | null>(null);
	// §C — the colour control most recently touched, so a clicked palette swatch knows where to apply.
	let activeColorVar = $state<string | null>(null);
	let draftName = $state('Untitled style');
	/** The draft: CSS-var → override value. Scoped to the preview wrapper; Apply → <body>. */
	let draft = $state<Record<string, string>>({});

	// §C redesign (Eisa) — the panel is RESIZABLE; its size persists across opens (localStorage, a
	// pure UI pref). And only the **Editor** category keeps the 3-zone layout (left · centre note
	// preview · right controls); every OTHER category is 2-zone (left sidebar + one wide right space,
	// no centre — the controls integrate their own preview, e.g. the live pill in each Links row).
	let panelW = $state(1180);
	let panelH = $state(760);
	const twoZone = $derived(activeCategory !== 'editor');

	const draftStyle = $derived(Object.entries(draft).map(([k, v]) => `${k}:${v}`).join(';'));
	const sel = $derived(selected ? ELEMENTS[selected] ?? null : null);

	// §C — the centre preview replicates the EXACT selected element (Eisa). Note/tree/global
	// elements share a sample shape; chrome widgets each have their own. (Heavy surfaces —
	// sky/org/index — keep their own alt preview, keyed on activeSurface below.)
	const NOTE_ELS = new Set(['noteBg', 'text', 'h1', 'h2', 'h3', 'h4', 'h5', 'h6', 'bold', 'italic', 'strike', 'code', 'quote', 'link', 'accent']);
	const TREE_ELS = new Set(['interface', 'fileTree', 'library', 'folder', 'cuniverse']);
	const GLOBAL_ELS = new Set(['gBackgrounds', 'gTextShades', 'gStatus', 'gAccent', 'gType', 'gShape']);
	const pk = $derived(
		!selected ? 'none'
		: NOTE_ELS.has(selected) ? 'note'
		: TREE_ELS.has(selected) ? 'tree'
		: GLOBAL_ELS.has(selected) ? 'global'
		: selected,
	);

	function hexOf(c: string): string {
		c = (c || '').trim();
		if (c.startsWith('#')) return c.length === 4 ? '#' + [...c.slice(1)].map((x) => x + x).join('') : c;
		const m = c.match(/rgba?\((\d+)[,\s]+(\d+)[,\s]+(\d+)/);
		if (m) return '#' + [m[1], m[2], m[3]].map((x) => (+x).toString(16).padStart(2, '0')).join('');
		return '#888888';
	}
	/** Current value of a var: the draft override, else the live value — read from <body>,
	    where the app sets its theme vars (:root would not see them and the swatch would be grey). */
	function curVal(v: string): string {
		if (v in draft) return draft[v];
		try { return getComputedStyle(document.body).getPropertyValue(v).trim(); } catch { return ''; }
	}
	/** Numeric current value for a range control: the draft/live value parsed, else the catalog
	    default. Reads `draft` first so the slider + readout track edits live. */
	function curNum(v: string, def: number): number {
		const raw = curVal(v);
		const n = parseFloat(raw);
		return Number.isFinite(n) ? n : def;
	}
	function setVar(v: string, val: string) { draft = { ...draft, [v]: val }; }
	/** §C — apply a saved swatch to the most-recently-touched colour control (or the selected
	 *  element's first colour control if none touched yet). */
	function applySwatch(hex: string) {
		const target = activeColorVar ?? sel?.controls.find((c) => c.type === 'color')?.var ?? null;
		if (target) { activeColorVar = target; setVar(target, hex); }
	}
	// §C Phase 5 — link display (appSettings, written immediately like scriptfont; NOT the draft).
	// Pill shape feeds the LinkTypePill + Backlinks/Outgoing panels; toggles gate the in-editor look.
	const pillShape = $derived($appSettings.linkPills?.shape ?? { radius: 10, height: 20, fontWeight: 700 });
	function setPillShape(prop: 'radius' | 'height' | 'fontWeight', val: number) {
		const lp = get(appSettings).linkPills;
		const shape = { ...lp.shape, [prop]: val } as typeof lp.shape;
		updateSettings({ linkPills: { ...lp, shape } });
	}
	function setToggle(setting: 'colourTypedLinks' | 'showTypedLinkLabels', val: boolean) {
		if (setting === 'colourTypedLinks') updateSettings({ colourTypedLinks: val });
		else updateSettings({ showTypedLinkLabels: val });
	}
	function selectEl(key: string) {
		selected = key;
		// keep the open category in sync with the clicked preview part (stay if already here).
		const cur = CATEGORIES.find((c) => c.key === activeCategory);
		if (cur && cur.elements.includes(key)) return;
		const c = CATEGORIES.find((c) => c.elements.includes(key));
		if (c) activeCategory = c.key;
	}
	function pickCategory(c: { key: string; surface: string; elements: string[] }) {
		activeCategory = c.key; activeSurface = c.surface; selected = c.elements[0] ?? null;
	}
	// §C Phase 6 — apply a saved/derived Style (non-destructive merge via the MIG-069 engine), then
	// reflect its look in the draft so the Setter + live preview show it.
	async function applyStyle(p: StylePreset) {
		await applyPreset(p);
		draft = { ...(get(appSettings).styleOverride ?? {}) };
		draftName = p.name;
	}
	// Save the CURRENT look as a named Style: Keep first (so the unsaved draft is captured into
	// styleOverride), then capture the default-on sections (incl. the styleOverride/Setter look).
	async function saveAsStyle() {
		keep();
		const keys = SECTION_CATALOGUE.filter((s) => s.defaultOn).map((s) => s.key);
		savedStyles = [...savedStyles, newPresetFromCurrent(draftName, keys)];
		await saveStylePresets($state.snapshot(savedStyles) as StylePreset[]);
	}

	// §C redesign — drag the corner grip to resize the panel; clamp to the viewport, persist on release.
	function startResize(e: PointerEvent) {
		e.preventDefault();
		const startX = e.clientX, startY = e.clientY, startW = panelW, startH = panelH;
		function move(ev: PointerEvent) {
			panelW = Math.max(760, Math.min(window.innerWidth * 0.97, startW + (ev.clientX - startX)));
			panelH = Math.max(460, Math.min(window.innerHeight * 0.95, startH + (ev.clientY - startY)));
		}
		function up() {
			window.removeEventListener('pointermove', move);
			window.removeEventListener('pointerup', up);
			try { localStorage.setItem('cn-style-setter-size', JSON.stringify({ w: panelW, h: panelH })); } catch { /* ignore */ }
		}
		window.addEventListener('pointermove', move);
		window.addEventListener('pointerup', up);
	}

	/** hex → HSL (mirrors the app's own hexToHSL; inlined so the Setter stays standalone). */
	function hexToHSL(hex: string): { h: number; s: number; l: number } | null {
		const h6 = hexOf(hex);
		const r = parseInt(h6.slice(1, 3), 16) / 255, g = parseInt(h6.slice(3, 5), 16) / 255, b = parseInt(h6.slice(5, 7), 16) / 255;
		if ([r, g, b].some((x) => Number.isNaN(x))) return null;
		const max = Math.max(r, g, b), min = Math.min(r, g, b);
		let h = 0, s = 0; const l = (max + min) / 2;
		if (max !== min) {
			const d = max - min;
			s = l > 0.5 ? d / (2 - max - min) : d / (max + min);
			if (max === r) h = ((g - b) / d + (g < b ? 6 : 0)) / 6;
			else if (max === g) h = ((b - r) / d + 2) / 6;
			else h = ((r - g) / d + 4) / 6;
		}
		return { h: Math.round(h * 360), s: Math.round(s * 100), l: Math.round(l * 100) };
	}

	// MIG-070 §C — the draft with the accent decomposed (the accent is also consumed as
	// --accent-h/s/l + --text-accent + --interactive-accent-hover). Used by BOTH Keep (persist) and
	// the live-preview layer, so the real app and the saved look apply identically.
	function mergedDraft(): Record<string, string> {
		const merged: Record<string, string> = { ...draft };
		const acc = draft['--interactive-accent'];
		if (acc) {
			const hsl = hexToHSL(acc);
			if (hsl) {
				merged['--accent-h'] = String(hsl.h);
				merged['--accent-s'] = `${hsl.s}%`;
				merged['--accent-l'] = `${hsl.l}%`;
				merged['--text-accent'] = `hsl(${hsl.h}, ${hsl.s}%, ${hsl.l}%)`;
				merged['--interactive-accent-hover'] = `hsl(${hsl.h}, ${hsl.s}%, ${Math.max(0, hsl.l - 8)}%)`;
			}
		}
		return merged;
	}
	// MIG-070 §C Option E — KEEP: persist the draft as the per-Universe styleOverride (ONE settings
	// update; the +layout $effect writes it to <body> — survives reload + theme switch), then drop the
	// transient live layer (styleOverride now carries the look).
	function keep() {
		mergeStyleOverride(mergedDraft());
		clearLiveStyleDraft();
	}
	// DISCARD: abandon the unsaved edits — re-seed the draft from the saved look + drop the live layer
	// (the +layout $effect reverts the REAL app to styleOverride). The saved look on disk is untouched.
	function discard() {
		draft = { ...(get(appSettings).styleOverride ?? {}) };
		clearLiveStyleDraft();
	}
	function resetDraft() {
		draft = {};
		selected = null;
		clearLiveStyleDraft();
		// MIG-070 §C — clear the persisted per-Universe override → back to the pure theme look.
		clearAllStyleOverride();
	}

	// MIG-070 §C Option E — LIVE preview: while the Setter is open on a non-Editor category, push the
	// draft to the REAL app via the transient layer (the real chrome IS the preview). On the Editor
	// category (its centre note-preview suffices) or when closed, clear it so the app shows the saved
	// look. One-directional (reads draft/twoZone/open, writes the external live store) → no loop.
	$effect(() => {
		if ($styleSetterOpen && twoZone) setLiveStyleDraft(mergedDraft());
		else clearLiveStyleDraft();
	});

	// MIG-070 §C — when the Setter opens, seed the draft from the persisted per-Universe override
	// so the controls reflect the live look (not a blank slate). Rising-edge only, so editing
	// while open isn't clobbered by the seed.
	let _ssWasOpen = false;
	$effect(() => {
		if ($styleSetterOpen && !_ssWasOpen) draft = { ...(get(appSettings).styleOverride ?? {}) };
		_ssWasOpen = $styleSetterOpen;
	});

	onMount(() => {
		// §C Phase 6 — load the user's saved Styles for the gallery (app-global; the unified list adds
		// the built-in/custom theme Styles read-time).
		loadStylePresets().then((s) => { savedStyles = s; });
		// §C redesign — restore the last panel size the user dragged to (pure UI pref).
		try {
			const s = JSON.parse(localStorage.getItem('cn-style-setter-size') || 'null');
			if (s && typeof s.w === 'number' && typeof s.h === 'number') { panelW = s.w; panelH = s.h; }
		} catch { /* ignore */ }
		// Capture phase + stopImmediatePropagation so Escape closes ONLY the Setter, never the
		// Settings modal underneath it. No-op (and doesn't swallow Escape) when the Setter is shut.
		function onKey(e: KeyboardEvent) {
			if (e.key === 'Escape' && get(styleSetterOpen)) {
				e.preventDefault();
				e.stopImmediatePropagation();
				closeStyleSetter();
			}
		}
		window.addEventListener('keydown', onKey, true);
		return () => window.removeEventListener('keydown', onKey, true);
	});
</script>

{#if $styleSetterOpen}
	<div class="ss-overlay" class:ss-overlay--live={twoZone} role="dialog" aria-label="Style Setter">
		<div class="ss" class:ss--twozone={twoZone} style="width:{panelW}px;height:{panelH}px;{draftStyle}">
			<!-- Top bar -->
			<header class="ss-top">
				<span class="ss-brand"><span class="ss-star">✦</span> Style Setter</span>
				{#if twoZone}<span class="ss-livetag" title="Your edits show on the real app live — Keep to save, Discard to revert">● live</span>{/if}
				<span class="ss-draft">draft: <input class="ss-dname" bind:value={draftName} /></span>
				<span class="ss-spacer"></span>
				<button class="ss-btn" onclick={resetDraft} title="Clear all overrides — back to the theme default">Reset</button>
				<button class="ss-btn" onclick={discard} title="Abandon unsaved changes (the real app reverts)">Discard</button>
				<button class="ss-btn ss-primary" onclick={keep} title="Save this look (per-Universe)">Keep</button>
				<button class="ss-btn ss-icon" aria-label="Close" onclick={closeStyleSetter}>✕</button>
			</header>

			<!-- Left rail: surfaces + themes -->
			<aside class="ss-left">
				<div class="ss-rlabel">Surfaces</div>
				{#each CATEGORIES as cat (cat.key)}
					<button class="ss-surface" class:active={activeCategory === cat.key} onclick={() => pickCategory(cat)}>
						<span class="ss-sdot"></span> {cat.name}
					</button>
					{#if activeCategory === cat.key}
						{#each cat.elements as elKey (elKey)}
							<button class="ss-elhead" class:active={selected === elKey} onclick={() => selectEl(elKey)}>{ELEMENTS[elKey].name}</button>
						{/each}
					{/if}
				{/each}
				<div class="ss-divider"></div>
				<div class="ss-rlabel">Saved styles</div>
				<div class="ss-stylelist">
					{#each savedStyles as p (p.id)}
						<button class="ss-srow" onclick={() => applyStyle(p)} title={'Apply ' + p.name} dir="auto">{p.name}</button>
					{/each}
					{#if !savedStyles.length}<div class="ss-srow-empty">Design a look, then save it as a named style you can reuse.</div>{/if}
					<button class="ss-srow ss-srow-save" onclick={saveAsStyle}>+ Save current as a style</button>
				</div>
			</aside>

			<!-- Center: focused preview of the SELECTED element (Eisa §C) -->
			<main class="ss-center">
				<div class="ss-hint">{selected ? 'Previewing: ' + (sel?.name ?? '') : 'Select an element on the left to preview & style it'}</div>
				<div class="ss-stage">
					{#if activeSurface !== 'editor'}
						<div class="ss-prev-alt">
							<div class="ss-alt-title">{CATEGORIES.find((c) => c.surface === activeSurface)?.name}</div>
							{#if activeSurface === 'sky' || activeSurface === 'org'}
								<div class="ss-sky">
									<button class="ss-node ss-hot" class:ss-sel={selected === 'accent'} onclick={() => selectEl('accent')} aria-label="accent"></button>
									<button class="ss-node b ss-hot" class:ss-sel={selected === 'link'} onclick={() => selectEl('link')} aria-label="link"></button>
								</div>
							{:else if activeSurface === 'index'}
								<div class="ss-idx">
									<div class="ss-irow"><button class="ss-ibar ss-hot" style="width:70%" class:ss-sel={selected === 'accent'} onclick={() => selectEl('accent')} aria-label="accent"></button> apple</div>
									<div class="ss-irow"><span class="ss-ibar" style="width:45%"></span> banana</div>
									<div class="ss-irow"><span class="ss-ibar" style="width:30%"></span> carrot</div>
								</div>
							{/if}
							<div class="ss-alt-note">representative snapshot · re-colours with your edits</div>
						</div>
					{:else if pk === 'none'}
						<div class="ss-focus ss-focus-empty">Pick an element on the left — its preview appears here.</div>
					{:else if pk === 'note'}
						<div class="ss-focus ss-fcard ss-fnote">
							<span class="ss-title ss-hot2" class:ss-sel={selected === 'text' || selected === 'noteBg'} onclick={() => selectEl('text')}>Apple (Fruit)</span>
							<span class="ss-h1 ss-hot2" class:ss-sel={selected === 'h1'} onclick={() => selectEl('h1')}>Heading one</span>
							<span class="ss-h2 ss-hot2" class:ss-sel={selected === 'h2'} onclick={() => selectEl('h2')}>Heading two</span>
							<span class="ss-h3 ss-hot2" class:ss-sel={selected === 'h3'} onclick={() => selectEl('h3')}>Heading three</span>
							<span class="ss-body">
								An <b class="ss-bold ss-hot2" class:ss-sel={selected === 'bold'} onclick={() => selectEl('bold')}>apple</b>
								a day pairs with a <i class="ss-italic ss-hot2" class:ss-sel={selected === 'italic'} onclick={() => selectEl('italic')}>crisp</i>
								<span class="ss-link ss-hot2" class:ss-sel={selected === 'link'} onclick={() => selectEl('link')}>[[Banana]]</span>
								<span class="ss-pill ss-hot2" class:ss-sel={selected === 'accent'} onclick={() => selectEl('accent')}>supports</span>
								— see <code class="ss-code ss-hot2" class:ss-sel={selected === 'code'} onclick={() => selectEl('code')}>juice()</code>,
								<s class="ss-strike ss-hot2" class:ss-sel={selected === 'strike'} onclick={() => selectEl('strike')}>an old note</s>.
							</span>
							<span class="ss-quote ss-hot2" class:ss-sel={selected === 'quote'} onclick={() => selectEl('quote')}>“An apple a day keeps the doctor away.”</span>
							<span class="ss-hrow">
								<span class="ss-h4 ss-hot2" class:ss-sel={selected === 'h4'} onclick={() => selectEl('h4')}>H4</span>
								<span class="ss-h5 ss-hot2" class:ss-sel={selected === 'h5'} onclick={() => selectEl('h5')}>H5</span>
								<span class="ss-h6 ss-hot2" class:ss-sel={selected === 'h6'} onclick={() => selectEl('h6')}>H6</span>
							</span>
						</div>
					{:else if pk === 'tree'}
						<div class="ss-focus ss-fcard ss-ftree">
							<span class="ss-lib ss-hot2" class:ss-sel={selected === 'library' || selected === 'interface'} onclick={() => selectEl('library')}>📚 My Library</span>
							<span class="ss-folder ss-hot2" class:ss-sel={selected === 'folder'} onclick={() => selectEl('folder')}>📁 Ideas</span>
							<span class="ss-file ss-hot2" class:ss-sel={selected === 'fileTree'} onclick={() => selectEl('fileTree')}>Apple (Fruit)</span>
							<span class="ss-file dim ss-hot2" class:ss-sel={selected === 'fileTree'} onclick={() => selectEl('fileTree')}>Banana</span>
							<span class="ss-cuniverse ss-hot2" class:ss-sel={selected === 'cuniverse'} onclick={() => selectEl('cuniverse')}>✦ Linked Universe</span>
						</div>
					{:else if pk === 'universe'}
						<div class="ss-focus ss-fcard"><span class="ss-univ" style="margin-top:0">◇ Universe</span></div>
					{:else if pk === 'statusbar'}
						<div class="ss-focus ss-fcard ss-fstrip"><div class="ss-statusbar2"><span>Library · Note</span><span>7,660 notes · ✦ Universe</span></div></div>
					{:else if pk === 'cDock'}
						<div class="ss-focus"><div class="ss-fdock"><i></i><i></i><i></i><i></i></div></div>
					{:else if pk === 'cToolbar'}
						<div class="ss-focus ss-fcard ss-fstrip"><div class="ss-ftoolbar"><b></b><b></b><b></b><b></b></div></div>
					{:else if pk === 'cLayoutBar'}
						<div class="ss-focus ss-fcard ss-fstrip"><div class="ss-flayout"><b></b><b class="on"></b><b></b></div></div>
					{:else if pk === 'cTabs'}
						<div class="ss-focus ss-fcard ss-fstrip"><div class="ss-ftabs"><span class="t">Note A</span><span class="t on">Note B</span><span class="t">Note C</span></div></div>
					{:else if pk === 'cRightSidebar'}
						<div class="ss-focus"><div class="ss-frs"><div class="ss-frs-tabs"><i></i><i class="on"></i><i></i></div><div class="ss-frs-body"></div></div></div>
					{:else if pk === 'cButtons'}
						<div class="ss-focus ss-frow"><button class="ss-fbtn">Save</button><button class="ss-fbtn ghost">Cancel</button></div>
					{:else if pk === 'cTags'}
						<div class="ss-focus ss-fcol"><div class="ss-frow"><span class="ss-ftag">#idea</span><span class="ss-ftag">#fruit</span></div><div class="ss-fcallout"><b>ℹ︎ Note</b> A callout box.</div></div>
					{:else if pk === 'cSidebar'}
						<div class="ss-focus"><div class="ss-fsidebar"><span></span><span></span><span></span><span></span></div></div>
					{:else if pk === 'global'}
						<div class="ss-focus ss-fcard ss-fglobal">
							<span class="ss-h2">Heading</span>
							<span class="ss-body">Body text with a <span class="ss-pill">pill</span> and <span class="ss-ftag">#tag</span>.</span>
							<div class="ss-frow"><button class="ss-fbtn">Button</button><span class="ss-fbox"></span></div>
						</div>
					{:else if pk === 'fonts'}
						<div class="ss-focus ss-fcard ss-fnote">
							<span class="ss-body">The quick brown fox jumps over the lazy dog. AaBbCc 0123</span>
							<span class="ss-body" dir="rtl">نص عربي — رؤية وتراث ومصادر المعرفة</span>
							<span class="ss-body">中文 · 日本語 · 한국어 · हिन्दी · Кириллица</span>
							<span class="ss-note-hint">Per-script fonts apply across the whole app — open a note in that script to see your chosen font. (This preview uses your Latin font.)</span>
						</div>
					{/if}
				</div>
			</main>

			<!-- Right rail: controls for the selected element -->
			<aside class="ss-right">
				{#if sel}
					<div class="ss-rlabel">Selected element</div>
					<div class="ss-selname">{sel.name}</div>
						{#each sel.controls as c (c.label)}
						<div class="ss-ctrl">
							{#if c.type === 'range'}
								<label for={'ss-' + c.var}>{c.label}<span class="ss-rval">{curNum(c.var, c.def)}{c.unit}</span></label>
								<input id={'ss-' + c.var} type="range" min={c.min} max={c.max} step={c.step}
									value={curNum(c.var, c.def)}
									oninput={(e) => setVar(c.var, (e.currentTarget as HTMLInputElement).value + c.unit)} />
							{:else if c.type === 'color'}
								<label for={'ss-' + c.var}>{c.label}</label>
								<input id={'ss-' + c.var} type="color" value={hexOf(curVal(c.var))}
									onfocus={() => activeColorVar = c.var}
									oninput={(e) => { activeColorVar = c.var; setVar(c.var, (e.currentTarget as HTMLInputElement).value); }}
									onchange={(e) => addStyleSwatch((e.currentTarget as HTMLInputElement).value)} />
							{:else if c.type === 'scriptfont'}
								<label for={'ss-sf-' + c.script}>{c.label}</label>
								<select id={'ss-sf-' + c.script} value={$appSettings.perScriptFonts?.[c.script] ?? ''} onchange={(e) => setPerScriptFont(c.script, (e.currentTarget as HTMLSelectElement).value)}>
									{#each c.options as [lbl, val] (lbl)}<option value={val}>{lbl}</option>{/each}
								</select>
							{:else if c.type === 'toggle'}
								<label class="ss-toggle">
									<span>{c.label}</span>
									<input type="checkbox" checked={$appSettings[c.setting]}
										onchange={(e) => setToggle(c.setting, (e.currentTarget as HTMLInputElement).checked)} />
								</label>
							{:else if c.type === 'pillrange'}
								<label for={'ss-pill-' + c.prop}>{c.label}<span class="ss-rval">{pillShape[c.prop]}{c.unit}</span></label>
								<input id={'ss-pill-' + c.prop} type="range" min={c.min} max={c.max} step={c.step}
									value={pillShape[c.prop]}
									oninput={(e) => setPillShape(c.prop, parseInt((e.currentTarget as HTMLInputElement).value))} />
							{:else if c.type === 'pillselect'}
								<label for={'ss-pill-' + c.prop}>{c.label}</label>
								<select id={'ss-pill-' + c.prop} value={String(pillShape[c.prop])}
									onchange={(e) => setPillShape(c.prop, parseInt((e.currentTarget as HTMLSelectElement).value))}>
									{#each c.options as [lbl, val] (val)}<option value={val}>{lbl}</option>{/each}
								</select>
							{:else}
								<label for={'ss-' + c.var}>{c.label}</label>
								<select id={'ss-' + c.var} value={curVal(c.var)} onchange={(e) => setVar(c.var, (e.currentTarget as HTMLSelectElement).value)}>
									{#each c.options as [lbl, val] (val)}<option value={val}>{lbl}</option>{/each}
								</select>
							{/if}
						</div>
					{/each}
					{#if selected === 'links'}
						<LinkTypesEditor embedded />
					{/if}
					{#if ($appSettings.styleSwatches ?? []).length && sel.controls.some((c) => c.type === 'color')}
						<div class="ss-rlabel">Saved colours</div>
						<div class="ss-swatches">
							{#each $appSettings.styleSwatches as sw (sw)}
								<button class="ss-sw" style="background:{sw}" title={sw + ' — click to apply · right-click to remove'} aria-label={sw} onclick={() => applySwatch(sw)} oncontextmenu={(e) => { e.preventDefault(); removeStyleSwatch(sw); }}></button>
							{/each}
						</div>
					{/if}
				{:else}
					<div class="ss-empty"><span class="ss-big">⊹</span>Click any part of the interface to style it. Its controls appear here, and changes show instantly.</div>
				{/if}
			</aside>
			<!-- §C redesign — corner grip to resize the whole panel (size persists across opens). -->
			<button class="ss-resize" aria-label="Resize panel" title="Drag to resize" onpointerdown={startResize}></button>
		</div>
	</div>
{/if}

<style>
	.ss-overlay {
		position: fixed; inset: 0; z-index: 9000; display: flex; align-items: center; justify-content: center;
		background: rgba(6, 6, 12, 0.62); backdrop-filter: blur(2px); padding: 16px;
	}
	/* §C Option E — LIVE mode (non-Editor categories): the overlay no longer dims or blocks the app, so
	   the real chrome shows AND stays interactive; the panel docks to the right, faintly translucent, so
	   the live app IS the preview. The Editor category keeps the centred, dimmed modal (rule above). */
	.ss-overlay--live { background: none; backdrop-filter: none; pointer-events: none; justify-content: flex-end; padding: 10px; }
	.ss-overlay--live .ss { pointer-events: auto; }
	.ss-overlay--live .ss--twozone { background: color-mix(in srgb, var(--c-bg) 92%, transparent); backdrop-filter: blur(7px); box-shadow: -14px 0 60px rgba(0,0,0,.45); }
	.ss-livetag { font-size: 11px; font-weight: 700; color: #4ade80; letter-spacing: .04em; white-space: nowrap; }
	.ss {
		/* Chrome follows the theme being edited (the .ss element carries the draft + inherits the
		   app theme), with the original dark studio look as fallback (MIG-070 §iter2-#2, Eisa). */
		--c-bg: var(--background-primary, #15151f); --c-surface: var(--background-secondary, #1d1d2a); --c-surface2: var(--background-modifier-hover, #24243440); --c-text: var(--text-normal, #cfd0e0);
		--c-muted: var(--text-muted, #8a8ba0); --c-border: var(--background-modifier-border, #2c2c3e); --c-accent: var(--interactive-accent, #7c6cff);
		max-width: 97vw; max-height: 95vh; background: var(--c-bg); position: relative;
		border: 1px solid var(--c-border); border-radius: 14px; overflow: hidden; color: var(--c-text);
		display: grid; grid-template-rows: 52px 1fr; grid-template-columns: 210px 1fr 248px;
		grid-template-areas: "top top top" "left center right"; box-shadow: 0 30px 80px rgba(0,0,0,.55);
		font-family: ui-sans-serif, system-ui, "Segoe UI", sans-serif;
	}
	/* §C redesign — 2-zone for every category EXCEPT Editor: left sidebar + one wide right space,
	   no centre column (the Editor's rendered note keeps the 3-zone grid above). */
	.ss--twozone { grid-template-columns: 210px 1fr; grid-template-areas: "top top" "left right"; }
	.ss--twozone .ss-center { display: none; }
	.ss--twozone .ss-right { padding: 18px 22px; }
	.ss--twozone .ss-ctrl, .ss--twozone .ss-selname, .ss--twozone .ss-rlabel, .ss--twozone .ss-swatches { max-width: 560px; }
	.ss--twozone .ss-right :global(.lte) { max-width: 620px; }
	/* Resize grip (bottom-right corner) — drag to resize the whole panel; the size persists. */
	.ss-resize { position: absolute; right: 0; bottom: 0; width: 18px; height: 18px; padding: 0; border: none; background: none; cursor: nwse-resize; z-index: 6; touch-action: none; opacity: .5; }
	.ss-resize:hover { opacity: .9; }
	.ss-resize::after { content: ""; position: absolute; right: 3px; bottom: 3px; width: 9px; height: 9px; border-right: 2px solid var(--c-muted); border-bottom: 2px solid var(--c-muted); border-bottom-right-radius: 3px; }
	.ss-top { grid-area: top; display: flex; align-items: center; gap: 12px; padding: 0 16px; border-bottom: 1px solid var(--c-border); background: var(--c-surface); }
	.ss-brand { font-weight: 700; } .ss-star { color: var(--c-accent); }
	.ss-draft { color: var(--c-muted); font-size: 13px; }
	.ss-dname { font: inherit; font-size: 13px; color: var(--c-text); background: var(--c-surface2); border: 1px solid var(--c-border); border-radius: 6px; padding: 3px 8px; width: 140px; }
	.ss-spacer { flex: 1; }
	.ss-btn { font: inherit; font-size: 13px; padding: 6px 13px; border-radius: 7px; border: 1px solid var(--c-border); background: var(--c-surface2); color: var(--c-text); cursor: pointer; }
	.ss-btn:hover { border-color: var(--c-accent); }
	.ss-primary { background: var(--c-accent); border-color: var(--c-accent); color: #fff; font-weight: 600; }
	.ss-icon { padding: 6px 9px; }
	.ss-left { grid-area: left; border-right: 1px solid var(--c-border); background: var(--c-surface); display: flex; flex-direction: column; padding: 12px 10px; gap: 5px; overflow-y: auto; }
	.ss-rlabel { font-size: 11px; text-transform: uppercase; letter-spacing: .07em; color: var(--c-muted); margin: 6px 4px 2px; }
	.ss-surface { display: flex; align-items: center; gap: 9px; padding: 7px 9px; border-radius: 8px; cursor: pointer; font: inherit; font-size: 13.5px; color: var(--c-text); background: none; border: none; text-align: left; }
	.ss-surface:hover { background: var(--c-surface2); }
	.ss-surface.active { background: color-mix(in srgb, var(--c-accent) 22%, transparent); color: #fff; }
	.ss-sdot { width: 7px; height: 7px; border-radius: 50%; background: currentColor; opacity: .55; flex: none; }
	.ss-surface.active .ss-sdot { opacity: 1; background: var(--c-accent); }
	/* §3B — element rows nested under their open category (Surface). */
	.ss-elhead { display: block; width: 100%; text-align: start; padding: 5px 9px 5px 26px; border-radius: 7px; cursor: pointer; font: inherit; font-size: 12.5px; color: var(--c-muted); background: none; border: none; }
	.ss-elhead:hover { background: var(--c-surface2); color: var(--c-text); }
	.ss-elhead.active { background: color-mix(in srgb, var(--c-accent) 20%, transparent); color: #fff; }
	.ss-divider { height: 1px; background: var(--c-border); margin: 8px 2px; }
	/* §C Phase 6 — saved styles as a LIGHTWEIGHT name list. NOT a gallery of stylePreview cards over
	   unifiedStyleList/BUILTIN_THEMES — that is the documented main-thread freeze shape (v2.49; LL-014). */
	.ss-stylelist { display: flex; flex-direction: column; gap: 3px; }
	.ss-srow { text-align: start; padding: 6px 9px; border-radius: 7px; border: 1px solid var(--c-border); background: var(--c-surface2); color: var(--c-text); font: inherit; font-size: 12.5px; cursor: pointer; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }
	.ss-srow:hover { border-color: var(--c-accent); }
	.ss-srow-save { border-style: dashed; color: var(--c-muted); text-align: center; }
	.ss-srow-empty { font-size: 11.5px; color: var(--c-muted); padding: 4px 6px; line-height: 1.4; }
	.ss-center { grid-area: center; display: flex; flex-direction: column; align-items: center; justify-content: center; padding: 20px; gap: 10px; background: var(--background-secondary, #14141c); }
	.ss-hint { font-size: 12px; color: var(--c-muted); }
	.ss-stage { position: relative; }
	/* The mini interface — uses the REAL app vars (overridden by the draft on .ss). */
	.ss-prev { width: 560px; height: 360px; border-radius: 10px; overflow: hidden; display: grid; grid-template-columns: 124px 1fr; grid-template-rows: 1fr auto; grid-template-areas: "side main" "status status"; background: var(--background-primary, #fbfbfa); box-shadow: 0 14px 40px rgba(0,0,0,.45); border: 1px solid rgba(0,0,0,.25); }
	.ss-side { grid-area: side; overflow: hidden; background: var(--background-secondary, #f1f1ef); color: var(--text-normal, #2e3338); padding: 12px 10px; display: flex; flex-direction: column; gap: 8px; border: none; text-align: left; font-family: var(--font-interface-theme, inherit); }
	.ss-file { font-size: var(--ft-master-font-size, 11.5px); color: var(--ft-master-color, var(--text-normal, #2e3338)); font-weight: var(--ft-master-weight, 400); font-family: var(--ft-master-font-family, inherit); padding: var(--ft-master-row-padding-y, 1px) 4px; border-radius: var(--ft-row-radius, 3px); border-bottom: var(--ft-border-width, 0px) var(--ft-border-style, solid) var(--ft-border-color, var(--background-modifier-border, #ddd)); display: flex; align-items: center; gap: 6px; } .ss-file.dim { opacity: .55; }
	/* §3B G1 — sidebar row types; each reads its own --ft-{type}-* with the File-tree master as fallback. */
	.ss-lib { font-size: var(--ft-library-font-size, var(--ft-master-font-size, 11.5px)); color: var(--ft-library-color, var(--ft-master-color, var(--text-normal, #2e3338))); font-weight: var(--ft-library-weight, var(--ft-master-weight, 600)); font-family: var(--ft-library-font-family, var(--ft-master-font-family, inherit)); display: flex; align-items: center; gap: 6px; }
	.ss-folder { font-size: var(--ft-folder-font-size, var(--ft-master-font-size, 11.5px)); color: var(--ft-folder-color, var(--ft-master-color, var(--text-muted, #6b7280))); font-weight: var(--ft-folder-weight, var(--ft-master-weight, 400)); font-family: var(--ft-folder-font-family, var(--ft-master-font-family, inherit)); display: flex; align-items: center; gap: 6px; padding-inline-start: 8px; }
	.ss-cuniverse { font-size: var(--ft-cuniverse-font-size, var(--ft-master-font-size, 11.5px)); color: var(--ft-cuniverse-color, var(--ft-master-color, var(--interactive-accent, #7c3aed))); font-weight: var(--ft-cuniverse-weight, var(--ft-master-weight, 600)); font-family: var(--ft-cuniverse-font-family, var(--ft-master-font-family, inherit)); display: flex; align-items: center; gap: 6px; }
	/* §3B — Universe switcher footer (sidebar foot) + Status bar strip (window bottom). */
	.ss-univ { margin-top: auto; display: flex; align-items: center; gap: 5px; font-size: var(--universe-bar-font-size, 11px); color: var(--universe-bar-color, var(--text-normal, #2e3338)); background: var(--universe-bar-bg, transparent); font-family: var(--universe-bar-font-family, inherit); border-top: 1px solid rgba(0,0,0,.08); padding-top: 5px; }
	.ss-statusbar { grid-area: status; display: flex; align-items: center; justify-content: space-between; gap: 8px; padding: 0 8px; min-height: var(--statusbar-height, 22px); background: var(--statusbar-bg, var(--background-secondary, #ececed)); color: var(--statusbar-color, var(--text-muted, #6b7280)); font-size: var(--statusbar-font-size, 10px); border: none; border-top: 1px solid rgba(0,0,0,.12); cursor: pointer; text-align: start; }
	.ss-file::before { content: ""; width: 6px; height: 6px; border-radius: 50%; background: var(--interactive-accent, #7c3aed); flex: none; } .ss-file.dim::before { background: currentColor; opacity: .4; }
	/* The note body scrolls if the chosen heading sizes overflow — the preview shows REAL sizes. */
	.ss-main { grid-area: main; background: var(--background-primary, #fbfbfa); color: var(--editor-text-color, var(--text-normal, #2e3338)); padding: 16px 18px; text-align: left; border: none; font-family: var(--font-text-theme, inherit); display: flex; flex-direction: column; gap: 7px; overflow-y: auto; }
	.ss-title { display: block; font-weight: 800; font-size: 18px; color: var(--editor-text-color, var(--text-normal, #2e3338)); }
	/* Headings read their own size/colour vars, with the catalog defaults + inherit as fallbacks
	   so the preview matches a real note before any edit. Weight is shared (--heading-weight). */
	/* Colour fallbacks mirror the real note's markdownHighlightStyle (heading #d73a49, bold
	   #e36209, italic #7c3aed, code #16a34a) so the preview looks like an actual Constellation
	   note before any edit; the controls override via the same --vars the real editor reads. */
	.ss-h1 { display: block; font-size: var(--h1-size, 32px); color: var(--h1-color, #d73a49); font-weight: var(--heading-weight, 700); line-height: 1.2; }
	.ss-h2 { display: block; font-size: var(--h2-size, 26px); color: var(--h2-color, #d73a49); font-weight: var(--heading-weight, 700); line-height: 1.2; }
	.ss-h3 { display: block; font-size: var(--h3-size, 22px); color: var(--h3-color, #d73a49); font-weight: var(--heading-weight, 600); line-height: 1.2; }
	.ss-h4 { font-size: var(--h4-size, 18px); color: var(--h4-color, #d73a49); font-weight: var(--heading-weight, 600); }
	.ss-h5 { font-size: var(--h5-size, 16px); color: var(--h5-color, #d73a49); font-weight: var(--heading-weight, 600); }
	.ss-h6 { font-size: var(--h6-size, 14px); color: var(--h6-color, #d73a49); font-weight: var(--heading-weight, 600); }
	.ss-hrow { display: flex; align-items: baseline; gap: 14px; }
	.ss-body { display: block; font-size: var(--font-text-size, 14px); line-height: 1.7; color: var(--editor-text-color, var(--text-normal, #2e3338)); }
	.ss-note-hint { display: block; font-size: 12px; line-height: 1.6; color: var(--text-muted, #8a8a8a); opacity: .85; max-width: 92%; margin-top: 4px; }
	.ss-bold { font-weight: var(--bold-weight, 700); color: var(--bold-color, #e36209); }
	.ss-italic { font-style: italic; color: var(--italic-color, #7c3aed); }
	.ss-strike { text-decoration: line-through; text-decoration-color: var(--strikethrough-color, currentColor); text-decoration-thickness: var(--strikethrough-thickness, 1px); opacity: 0.7; }
	.ss-code { font-family: var(--font-monospace-theme, ui-monospace, "Courier New", monospace); font-size: var(--font-monospace-size, 13px); background: var(--code-background, rgba(0,0,0,.07)); color: var(--code-normal, #16a34a); border-radius: var(--radius-s, 3px); padding: 1px 5px; }
	.ss-link { color: var(--link-color, var(--interactive-accent, #2f6fed)); text-decoration: var(--link-decoration, underline); }
	.ss-pill { display: inline-flex; align-items: center; background: var(--interactive-accent, #4a9eff); color: #fff; font-size: 11px; font-weight: 700; padding: 1px 8px; border-radius: 9px; text-transform: lowercase; }
	.ss-quote { display: block; color: var(--blockquote-text-color, var(--text-muted, #8a8a8a)); font-style: italic; border-inline-start: 3px solid color-mix(in srgb, var(--blockquote-text-color, var(--text-muted, #8a8a8a)) 60%, transparent); padding-inline-start: 9px; }
	/* Hover/selected rings drawn INSIDE the element (inset box-shadow) so they're never clipped
	   by the preview's overflow:hidden — that was why the edge-touching sidebar/note showed nothing. */
	.ss-hot { cursor: pointer; } .ss-hot:hover { box-shadow: inset 0 0 0 2px #9d8dff; }
	.ss-hot2 { cursor: pointer; border-radius: 3px; } .ss-hot2:hover { outline: 2px dashed #9d8dff; outline-offset: 2px; }
	.ss-sel { box-shadow: inset 0 0 0 2.5px #b9acff !important; }
	.ss-hot2.ss-sel { box-shadow: none !important; outline: 2.5px solid #b9acff !important; outline-offset: 2px; }
	.ss-prev-alt { width: 560px; height: 360px; border-radius: 10px; background: var(--background-primary, #fbfbfa); color: var(--text-normal, #2e3338); box-shadow: 0 14px 40px rgba(0,0,0,.45); border: 1px solid rgba(0,0,0,.25); display: flex; align-items: center; justify-content: center; flex-direction: column; gap: 16px; }
	.ss-alt-title { font-weight: 700; font-size: 15px; color: var(--interactive-accent, #7c3aed); }
	.ss-alt-note { font-size: 11.5px; color: var(--text-normal, #6b7280); opacity: .7; max-width: 70%; text-align: center; }
	.ss-sky { display: flex; gap: 22px; }
	.ss-node { width: 34px; height: 34px; border-radius: 50%; border: none; cursor: pointer; background: var(--interactive-accent, #7c3aed); box-shadow: 0 0 0 4px color-mix(in srgb, var(--interactive-accent, #7c3aed) 25%, transparent); }
	.ss-node.b { background: var(--link-color, #2f6fed); box-shadow: 0 0 0 4px color-mix(in srgb, var(--link-color, #2f6fed) 25%, transparent); }
	.ss-idx { width: 70%; display: flex; flex-direction: column; gap: 8px; }
	.ss-irow { display: flex; align-items: center; gap: 8px; font-size: 12px; }
	.ss-ibar { height: 7px; background: var(--interactive-accent, #7c3aed); border-radius: 3px; border: none; cursor: pointer; }
	.ss-right { grid-area: right; border-left: 1px solid var(--c-border); background: var(--c-surface); padding: 14px; overflow-y: auto; }
	.ss-selname { font-size: 16px; font-weight: 700; margin-bottom: 14px; }
	.ss-ctrl { margin-bottom: 14px; }
	.ss-ctrl label { display: flex; justify-content: space-between; align-items: baseline; font-size: 12px; color: var(--c-muted); margin-bottom: 5px; }
	.ss-rval { font-variant-numeric: tabular-nums; color: var(--c-text); font-weight: 600; }
	.ss-ctrl input[type=color] { width: 100%; height: 30px; border: 1px solid var(--c-border); border-radius: 6px; background: none; cursor: pointer; }
	.ss-ctrl input[type=range] { width: 100%; accent-color: var(--c-accent); cursor: pointer; }
	.ss-ctrl select { width: 100%; padding: 6px 8px; border-radius: 6px; border: 1px solid var(--c-border); background: var(--c-surface2); color: var(--c-text); font: inherit; font-size: 13px; }
	.ss-ctrl select option { background: var(--c-surface); color: var(--c-text); }
	/* §C — saved colour palette (swatches). Click to apply to the active colour control; right-click to remove. */
	.ss-swatches { display: flex; flex-wrap: wrap; gap: 6px; margin: 4px 0 10px; }
	.ss-sw { width: 22px; height: 22px; border-radius: 5px; border: 1px solid var(--c-border); cursor: pointer; padding: 0; }
	.ss-sw:hover { outline: 2px solid var(--c-accent); outline-offset: 1px; }
	/* §C — focused per-element preview: the centre shows JUST the selected element (Eisa). */
	.ss-focus { min-width: 460px; min-height: 300px; display: flex; flex-direction: column; align-items: center; justify-content: center; gap: 14px; }
	.ss-focus-empty { color: var(--c-muted); font-size: 13px; }
	.ss-fcard { background: var(--background-primary, #fbfbfa); color: var(--editor-text-color, var(--text-normal, #2e3338)); border: 1px solid rgba(0,0,0,.18); border-radius: 12px; box-shadow: 0 14px 40px rgba(0,0,0,.22); padding: 22px 26px; display: flex; flex-direction: column; gap: 9px; min-width: 320px; max-width: 460px; text-align: left; align-items: stretch; }
	.ss-fnote { font-family: var(--font-text-theme, inherit); }
	.ss-ftree { background: var(--background-secondary, #f1f1ef); font-family: var(--font-interface-theme, inherit); gap: 7px; }
	.ss-fstrip { padding: 14px; min-width: 380px; }
	.ss-statusbar2 { display: flex; align-items: center; justify-content: space-between; gap: 12px; width: 100%; padding: 0 10px; min-height: var(--statusbar-height, 24px); background: var(--statusbar-bg, var(--background-secondary, #ececed)); color: var(--statusbar-color, var(--text-muted, #6b7280)); font-size: var(--statusbar-font-size, 11px); border-radius: 6px; }
	.ss-fdock { background: var(--dock-bg, var(--background-secondary, #ececed)); width: var(--dock-width, 44px); border-radius: 10px; padding: 12px 0; display: flex; flex-direction: column; align-items: center; gap: 14px; box-shadow: 0 14px 40px rgba(0,0,0,.22); }
	.ss-fdock i { width: var(--dock-icon-size, 18px); height: var(--dock-icon-size, 18px); border-radius: var(--dock-btn-radius, 4px); background: var(--dock-btn-color, var(--text-muted, #888)); display: block; }
	.ss-ftoolbar { display: flex; gap: 8px; padding: 6px; background: var(--sidebar-toolbar-bg, var(--background-secondary, #ececed)); border-radius: 8px; min-height: var(--sidebar-toolbar-height, 34px); align-items: center; }
	.ss-ftoolbar b { width: var(--sidebar-btn-size, 26px); height: var(--sidebar-btn-size, 26px); border-radius: var(--sidebar-btn-radius, 3px); background: color-mix(in srgb, var(--sidebar-btn-color, var(--text-muted, #888)) 22%, transparent); border: 1px solid color-mix(in srgb, var(--sidebar-btn-color, var(--text-muted, #888)) 45%, transparent); display: block; }
	.ss-flayout { display: flex; gap: 8px; padding: 6px; background: var(--layout-bar-bg, var(--background-secondary, #ececed)); border-radius: 8px; min-height: var(--layout-bar-height, 34px); align-items: center; }
	.ss-flayout b { width: var(--layout-btn-size, 28px); height: var(--layout-btn-size, 28px); border-radius: var(--layout-btn-radius, 4px); background: color-mix(in srgb, var(--layout-btn-color, var(--text-muted, #888)) 20%, transparent); border: 1px solid color-mix(in srgb, var(--layout-btn-color, var(--text-muted, #888)) 40%, transparent); display: block; }
	.ss-flayout b.on { background: color-mix(in srgb, var(--layout-btn-active-color, var(--interactive-accent, #7c3aed)) 30%, transparent); border-color: var(--layout-btn-active-color, var(--interactive-accent, #7c3aed)); }
	.ss-ftabs { display: flex; gap: 4px; padding: 8px 8px 0; background: var(--topbar-bg, var(--background-secondary, #ececed)); border-radius: 8px 8px 0 0; align-items: flex-end; }
	.ss-ftabs .t { font-size: var(--tab-font-size, 13px); height: var(--tab-height, 26px); display: flex; align-items: center; padding: 0 10px; border-radius: var(--tab-radius, 6px) var(--tab-radius, 6px) 0 0; background: var(--tab-bg, #dcdce0); color: var(--tab-color, var(--text-muted, #555)); border: 1px solid var(--tab-border, transparent); border-bottom: none; }
	.ss-ftabs .t.on { background: var(--tab-active-bg, var(--background-primary, #fff)); color: var(--tab-active-color, var(--text-normal, #222)); }
	.ss-frs { display: flex; flex-direction: column; width: 220px; height: 200px; background: var(--right-sidebar-bg, var(--background-secondary, #f1f1ef)); border-radius: 10px; box-shadow: 0 14px 40px rgba(0,0,0,.22); overflow: hidden; }
	.ss-frs-tabs { display: flex; gap: 10px; padding: 0 10px; align-items: center; height: var(--rs-tab-height, 30px); background: var(--rs-tabs-bg, var(--background-secondary-alt, #e8e8ec)); }
	.ss-frs-tabs i { width: var(--rs-icon-size, 16px); height: var(--rs-icon-size, 16px); border-radius: 3px; background: var(--rs-tab-color, var(--text-muted, #888)); display: block; }
	.ss-frs-tabs i.on { background: var(--rs-tab-active-color, var(--interactive-accent, #7c3aed)); }
	.ss-frs-body { flex: 1; }
	.ss-frow { display: flex; gap: 10px; align-items: center; }
	.ss-fcol { display: flex; flex-direction: column; gap: 12px; align-items: center; }
	.ss-fbtn { font: inherit; font-size: 13px; background: var(--interactive-accent, #7c6cff); color: var(--text-on-accent, #fff); border: none; border-radius: var(--button-radius, 6px); padding: var(--button-padding-y, 6px) var(--button-padding-x, 12px); cursor: default; }
	.ss-fbtn.ghost { background: var(--background-secondary, #ececed); color: var(--text-normal, #333); border: 1px solid var(--background-modifier-border, #ccc); }
	.ss-ftag { display: inline-flex; align-items: center; font-size: 12px; background: var(--tag-bg, color-mix(in srgb, var(--interactive-accent, #7c3aed) 12%, transparent)); color: var(--tag-color, var(--interactive-accent, #7c3aed)); border-radius: var(--tag-radius, 12px); padding: 2px 9px; }
	.ss-fcallout { background: color-mix(in srgb, var(--interactive-accent, #4a9eff) 8%, transparent); border-inline-start: 3px solid var(--interactive-accent, #4a9eff); border-radius: var(--callout-radius, 8px); padding: 8px 12px; font-size: 13px; color: var(--text-normal, #333); }
	.ss-fsidebar { width: clamp(120px, var(--sidebar-width, 260px), 320px); height: 200px; background: var(--sidebar-bg, var(--background-secondary, #f1f1ef)); border-radius: 10px; box-shadow: 0 14px 40px rgba(0,0,0,.22); padding: 14px 12px; display: flex; flex-direction: column; gap: 12px; }
	.ss-fsidebar span { height: 9px; border-radius: 4px; background: color-mix(in srgb, var(--text-normal, #888) 18%, transparent); display: block; }
	.ss-fsidebar span:nth-child(1) { width: 80%; } .ss-fsidebar span:nth-child(2) { width: 60%; } .ss-fsidebar span:nth-child(3) { width: 72%; } .ss-fsidebar span:nth-child(4) { width: 50%; }
	.ss-fbox { width: 44px; height: 30px; border-radius: var(--radius-m, 8px); background: var(--background-secondary, #ececed); border: var(--border-width, 1px) solid var(--background-modifier-border, #ccc); display: inline-block; }
	.ss-empty { color: var(--c-muted); font-size: 13px; line-height: 1.6; margin-top: 28px; text-align: center; }
	.ss-big { font-size: 26px; opacity: .5; display: block; margin-bottom: 8px; }
	/* §C Phase 5 — Links category: the settings-backed toggle control (used in the right space). */
	.ss-toggle { display: flex; align-items: center; justify-content: space-between; gap: 10px; font-size: 13px; color: var(--c-text); cursor: pointer; }
	.ss-toggle input { width: 16px; height: 16px; accent-color: var(--c-accent); cursor: pointer; flex: none; }
</style>
