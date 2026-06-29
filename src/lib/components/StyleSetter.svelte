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
	import { onMount, onDestroy } from 'svelte';
	import { get } from 'svelte/store';
	import { styleSetterOpen, closeStyleSetter, styleSetterInspectRequest, styleSetterCategoryRequest } from '$lib/stores/styleSetter';
	// MIG-070 §C polish (Item A) — real font choices: the shared catalogue (curated floor + the user's
	// installed fonts via queryLocalFonts), reused from Settings. Drives the font pickers + live preview.
	import { systemFonts, ensureSystemFonts, fontFamilyValue } from '$lib/fonts';
	import { appSettings, mergeStyleOverride, clearAllStyleOverride, addStyleSwatch, removeStyleSwatch, renameStyleSwatch, setPerScriptFont, updateSettings, setLiveStyleDraft, clearLiveStyleDraft } from '$lib/libraries/store';
	// §C Phase 5 — link styling reuses the EXISTING single source: the §G Link-Types editor (one save
	// path → Backlinks/Outgoing/editor recolour live). Display toggles + pill shape are appSettings.
	import LinkTypesEditor from './LinkTypesEditor.svelte';
	// MIG-081 §C.2d — the real CalendarPanel as the Calendar category's centre preview. It reads
	// the draft --cal-* (set on the .ss root) → recolours live; engine loads lazily on open.
	import CalendarPanel from './CalendarPanel.svelte';
	// MIG-070 §C Phase 6 — named, reusable Styles (the frozen MIG-069 SAVE/APPLY engine, reused as-is —
	// the same calls StylePresetsPanel uses). NOTE: we deliberately do NOT import unifiedStyleList /
	// stylePreview here — rendering BUILTIN_THEMES through themeToStyle as a gallery of self-portrait
	// cards is the documented main-thread FREEZE shape that the clean-slate Setter exists to avoid
	// (orientation v2.49; LL-014). The Setter lists only the user's SAVED styles, as lightweight rows.
	import { loadStylePresets, saveStylePresets, newPresetFromCurrent, applyPreset, exportPreset, importPreset, SECTION_CATALOGUE, type StylePreset } from '$lib/libraries/stylePresets';
	import { t } from '$lib/i18n';

	// MIG-072 follow-up — Style Setter i18n. Every control label / group + category name / chrome
	// string renders through L(en): it looks up `styleSetter.labels.<slug>` and FALLS BACK to the
	// English text, so any untranslated string shows exactly as today (zero regression). Identical
	// English strings (the many "Background", "Button size", …) dedupe to one shared key.
	function ssSlug(s: string): string {
		return (s || '').toLowerCase().replace(/[^a-z0-9]+/g, '_').replace(/^_+|_+$/g, '');
	}
	function L(en: string | undefined | null): string {
		if (!en) return en ?? '';
		const key = 'styleSetter.labels.' + ssSlug(en);
		const v = $t(key);
		// svelte-i18n returns the KEY ITSELF on a miss (truthy — `|| en` never fired), so an
		// unkeyed label used to render as a raw `styleSetter.labels.*` string. Treat key-echo
		// as a miss and fall back to the English text.
		return !v || v === key ? en : v;
	}

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
		| { label: string; type: 'pillselect'; prop: 'fontWeight'; options: [string, string][] }
		// §C Phase 9 wiring-audit — font sizes are driven by appSettings (interfaceFontSize/fontSize),
		// which +layout applies AFTER styleOverride. A CSS-var control would be stomped, so these write
		// appSettings directly (single source of truth — no duplicate var↔setting path).
		| { label: string; type: 'appnum'; setting: 'interfaceFontSize' | 'fontSize'; min: number; max: number; step: number; unit: string; def: number };

	// §C Phase 4 / §C-polish Item A — the three generic stacks stay as the top choices (with proper
	// generic fallbacks); the user's INSTALLED fonts (or the curated floor) are appended live via
	// `fontOptions` below. `FONTS` is what the ELEMENTS map references for font `select` controls; the
	// template swaps in `fontOptions` (generics + installed) at render so every font picker is real.
	const FONTS: [string, string][] = [
		['System', 'ui-sans-serif, system-ui, "Segoe UI", sans-serif'],
		['Serif', 'ui-serif, Georgia, "Times New Roman", serif'],
		['Mono', 'ui-monospace, "Courier New", monospace'],
	];
	// The live font-picker options: the 3 generics on top, then every installed/curated family (each
	// value safely quoted for CSS). A font `select` (var contains "font") renders THIS list, not FONTS.
	const fontOptions = $derived<[string, string][]>([
		...FONTS,
		...$systemFonts.map((f) => [f, fontFamilyValue(f)] as [string, string]),
	]);
	const DECOR: [string, string][] = [
		['Underline', 'underline'],
		['None', 'none'],
		['Dotted', 'underline dotted'],
	];
	// Border styles (shared by every element's border controls — §3B full set).
	const BORDER_STYLE: [string, string][] = [
		['Solid', 'solid'], ['Dashed', 'dashed'], ['Dotted', 'dotted'], ['None', 'none'],
	];
	// MIG-072 §2 — per-ring frame style (Sky View node rings). Solid first = the default look.
	const FRAME_STYLE_OPTS: [string, string][] = [['Solid', 'solid'], ['Dotted', 'dotted']];
	// §C Phase 9 gap-close — shadows are box-shadow CSS strings; a non-technical user picks a
	// preset, not a raw value. "Default" is FIRST so an unset var (curVal '') displays it (= the
	// catalog default), and its value matches the catalog so a set-to-default var reflects too.
	const SHADOW_S_OPTS: [string, string][] = [
		['Default', '0 1px 2px rgba(0,0,0,0.1)'], ['None', 'none'],
		['Soft', '0 2px 6px rgba(0,0,0,0.14)'], ['Medium', '0 3px 10px rgba(0,0,0,0.18)'], ['Strong', '0 5px 16px rgba(0,0,0,0.24)'],
	];
	const SHADOW_L_OPTS: [string, string][] = [
		['Default', '0 4px 16px rgba(0,0,0,0.12)'], ['None', 'none'],
		['Soft', '0 6px 20px rgba(0,0,0,0.16)'], ['Medium', '0 8px 28px rgba(0,0,0,0.2)'], ['Strong', '0 12px 36px rgba(0,0,0,0.26)'],
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
			{ label: 'Panel background', type: 'color', var: '--background-secondary' },
			{ label: 'Centre zone background', type: 'color', var: '--center-zone-bg' } ] },
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
		// MIG-070 §C — the Universe / Libraries panel (the switcher popup: Universe header, Own Libraries
		// list with colour dots + counts). Its bg/text/header default to the interface look; override here.
		universePanel: { name: 'Universe panel', controls: [
			{ label: 'Background', type: 'color', var: '--universe-panel-bg' },
			{ label: 'Text colour', type: 'color', var: '--universe-panel-color' },
			{ label: 'Section headers', type: 'color', var: '--universe-panel-header-color' } ] },
		noteBg:  { name: 'Note background', controls: [{ label: 'Background', type: 'color', var: '--background-primary' }] },
		text:    { name: 'Body text', controls: [
			{ label: 'Text colour', type: 'color', var: '--editor-text-color' },
			{ label: 'Note font', type: 'select', var: '--font-text-theme', options: FONTS },
			{ label: 'Text size', type: 'appnum', setting: 'fontSize', min: 11, max: 28, step: 1, unit: 'px', def: 16 } ] },
		link:    { name: 'Link', controls: [
			{ label: 'Link colour', type: 'color', var: '--link-color' },
			{ label: 'Hover colour', type: 'color', var: '--link-color-hover' },
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
			{ label: 'Code size', type: 'range', var: '--font-monospace-size', min: 10, max: 22, step: 1, unit: 'px', def: 14 },
			{ label: 'Block radius', type: 'range', var: '--code-block-radius', min: 0, max: 20, step: 1, unit: 'px', def: 6 } ] },
		quote:  { name: 'Blockquote', controls: [
			{ label: 'Text colour', type: 'color', var: '--blockquote-text-color' },
			{ label: 'Bar colour', type: 'color', var: '--blockquote-border-color' },
			{ label: 'Bar width', type: 'range', var: '--blockquote-border-width', min: 1, max: 8, step: 1, unit: 'px', def: 3 } ] },
		caret:  { name: 'Cursor & selection', controls: [
			{ label: 'Cursor colour', type: 'color', var: '--caret-color' },
			{ label: 'Selection colour', type: 'color', var: '--text-selection' } ] },
		// MIG-070 §C — the NOTE summary (the italic NSC headline under the title) + the breadcrumb path
		// line. Each writes its dedicated NotePane var (defaults to today's look when unset). The
		// breadcrumb's colour defaults to the interface text (§3B); set it here to override.
		summary: { name: 'Note summary', controls: [
			{ label: 'Colour', type: 'color', var: '--summary-color' },
			{ label: 'Font', type: 'select', var: '--summary-font', options: FONTS },
			{ label: 'Text size', type: 'range', var: '--summary-size', min: 11, max: 24, step: 1, unit: 'px', def: 15 },
			{ label: 'Thickness', type: 'range', var: '--summary-weight', min: 300, max: 900, step: 100, unit: '', def: 400 },
			{ label: 'Italic', type: 'select', var: '--summary-style', options: [['Italic', 'italic'], ['Normal', 'normal']] } ] },
		breadcrumb: { name: 'Breadcrumb', controls: [
			{ label: 'Colour', type: 'color', var: '--breadcrumb-color' },
			{ label: 'Text size', type: 'range', var: '--breadcrumb-size', min: 9, max: 18, step: 1, unit: 'px', def: 12 } ] },
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
			{ label: 'Interface font size', type: 'appnum', setting: 'interfaceFontSize', min: 11, max: 20, step: 1, unit: 'px', def: 14 },
			{ label: 'Line height', type: 'range', var: '--line-height-normal', min: 1.1, max: 2.2, step: 0.05, unit: '', def: 1.6 },
			{ label: 'Tight line height', type: 'range', var: '--line-height-tight', min: 1.0, max: 1.8, step: 0.05, unit: '', def: 1.3 } ] },
			// §C Phase 9 wiring-audit — "Paragraph spacing" removed: the editor is a live-preview SOURCE
			// editor (lines, not paragraphs); paragraph gaps are blank lines, so there's no honest CSS hook.
		gShape: { name: 'Shape & corners', controls: [
			{ label: 'Small radius', type: 'range', var: '--radius-s', min: 0, max: 20, step: 1, unit: 'px', def: 4 },
			{ label: 'Medium radius', type: 'range', var: '--radius-m', min: 0, max: 24, step: 1, unit: 'px', def: 8 },
			{ label: 'Large radius', type: 'range', var: '--radius-l', min: 0, max: 32, step: 1, unit: 'px', def: 12 },
			{ label: 'Border width', type: 'range', var: '--border-width', min: 0, max: 4, step: 1, unit: 'px', def: 1 },
			{ label: 'Reading width', type: 'range', var: '--file-line-width', min: 600, max: 1600, step: 20, unit: 'px', def: 1200 },
			{ label: 'Note margins', type: 'range', var: '--file-margins', min: 0, max: 96, step: 4, unit: 'px', def: 48 },
			{ label: 'Small shadow', type: 'select', var: '--shadow-s', options: SHADOW_S_OPTS },
			{ label: 'Large shadow', type: 'select', var: '--shadow-l', options: SHADOW_L_OPTS } ] },
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
			{ label: 'Tab radius', type: 'range', var: '--tab-radius', min: 0, max: 16, step: 1, unit: 'px', def: 6 },
			{ label: 'Tab left offset', type: 'range', var: '--tab-bar-offset', min: 0, max: 64, step: 1, unit: 'px', def: 32 } ] },
		cRightSidebar: { name: 'Right sidebar', controls: [
			{ label: 'Background', type: 'color', var: '--right-sidebar-bg' },
			{ label: 'Tab row background', type: 'color', var: '--rs-tabs-bg' },
			{ label: 'Tab icon colour', type: 'color', var: '--rs-tab-color' },
			{ label: 'Tab icon (active)', type: 'color', var: '--rs-tab-active-color' },
			{ label: 'Tab row height', type: 'range', var: '--rs-tab-height', min: 24, max: 56, step: 1, unit: 'px', def: 30 },
			{ label: 'Tab icon size', type: 'range', var: '--rs-icon-size', min: 10, max: 28, step: 1, unit: 'px', def: 16 } ] },
		// MIG-080 §H — PER-PANEL right-sidebar text size. Each control writes --rs-text-scale-<tab>;
		// .rs-inner applies the ACTIVE tab's token to --rs-scale, which the 181 font-size wraps read.
		// Text-only (spacing untouched); default 100 = no change. Live two-zone edit (Components surface).
		cRsText: { name: 'Right Sidebar Text', controls: [
			{ label: 'Properties',    type: 'range', var: '--rs-text-scale-properties',   min: 70, max: 140, step: 5, unit: '', def: 100 },
			{ label: 'Backlinks',     type: 'range', var: '--rs-text-scale-backlinks',    min: 70, max: 140, step: 5, unit: '', def: 100 },
			{ label: 'Tags',          type: 'range', var: '--rs-text-scale-tags',         min: 70, max: 140, step: 5, unit: '', def: 100 },
			{ label: 'Sky View',      type: 'range', var: '--rs-text-scale-star',         min: 70, max: 140, step: 5, unit: '', def: 100 },
			{ label: 'Tasks',         type: 'range', var: '--rs-text-scale-tasks',        min: 70, max: 140, step: 5, unit: '', def: 100 },
			{ label: 'Health',        type: 'range', var: '--rs-text-scale-health',       min: 70, max: 140, step: 5, unit: '', def: 100 },
			{ label: 'Provenance',    type: 'range', var: '--rs-text-scale-provenance',   min: 70, max: 140, step: 5, unit: '', def: 100 },
			{ label: 'Review',        type: 'range', var: '--rs-text-scale-review',       min: 70, max: 140, step: 5, unit: '', def: 100 },
			{ label: '360.3D',        type: 'range', var: '--rs-text-scale-inspector360', min: 70, max: 140, step: 5, unit: '', def: 100 },
			{ label: 'Source Review', type: 'range', var: '--rs-text-scale-sourceReview', min: 70, max: 140, step: 5, unit: '', def: 100 } ] },
		cButtons: { name: 'Buttons', controls: [
			{ label: 'Radius', type: 'range', var: '--button-radius', min: 0, max: 24, step: 1, unit: 'px', def: 6 },
			{ label: 'Padding (horizontal)', type: 'range', var: '--button-padding-x', min: 4, max: 32, step: 1, unit: 'px', def: 12 },
			{ label: 'Padding (vertical)', type: 'range', var: '--button-padding-y', min: 2, max: 20, step: 1, unit: 'px', def: 6 } ] },
		cTags: { name: 'Tags & callouts', controls: [
			{ label: 'Tag background', type: 'color', var: '--tag-bg' },
			{ label: 'Tag text', type: 'color', var: '--tag-color' },
			{ label: 'Tag radius', type: 'range', var: '--tag-radius', min: 0, max: 24, step: 1, unit: 'px', def: 12 },
			{ label: 'Callout radius', type: 'range', var: '--callout-radius', min: 0, max: 24, step: 1, unit: 'px', def: 8 } ] },
		// MIG-088 Phase 1 — the Frontmatter (Properties panel) pills, previously hardcoded. Each var
		// falls back to today's value in PropertyEditor (--background-modifier-border-focus / #fff /
		// --pill-radius / --pill-height) so the look is byte-identical until the user edits a control.
		pTags: { name: 'Property tags', controls: [
			{ label: 'Tag background', type: 'color', var: '--pe-tag-bg' },
			{ label: 'Tag text', type: 'color', var: '--pe-tag-text-color' },
			{ label: 'Tag radius', type: 'range', var: '--pe-tag-radius', min: 0, max: 20, step: 1, unit: 'px', def: 10 },
			{ label: 'Height', type: 'range', var: '--pe-tag-height', min: 14, max: 32, step: 1, unit: 'px', def: 20 } ] },
		pTaxo: { name: 'Taxonomy pills', controls: [
			{ label: 'Background', type: 'color', var: '--pe-taxo-bg' },
			{ label: 'Text', type: 'color', var: '--pe-taxo-text-color' },
			{ label: 'Radius', type: 'range', var: '--pe-taxo-radius', min: 0, max: 20, step: 1, unit: 'px', def: 10 } ] },
		// MIG-088 §2 — Cognitive colours: the shared cognitive-vocabulary palettes (Maturity, Confidence,
		// Origin, Stage, Match-category). "Unify on demand" (Boss 2026-06-29): each surface keeps its own
		// colour as a fallback until the user sets the shared var here, then ALL surfaces snap to it.
		cogMaturity: { name: 'Maturity', controls: [
			{ label: 'Seed', type: 'color', var: '--maturity-seed' },
			{ label: 'Sapling', type: 'color', var: '--maturity-sapling' },
			{ label: 'Evergreen', type: 'color', var: '--maturity-evergreen' },
			{ label: 'Canonical', type: 'color', var: '--maturity-canonical' },
			{ label: 'Wilting', type: 'color', var: '--maturity-wilting' } ] },
		cogConfidence: { name: 'Confidence', controls: [
			{ label: 'Hypothesis', type: 'color', var: '--confidence-hypothesis' },
			{ label: 'Evidence', type: 'color', var: '--confidence-evidence' },
			{ label: 'Established', type: 'color', var: '--confidence-established' },
			{ label: 'Contested', type: 'color', var: '--confidence-contested' } ] },
		cogOrigin: { name: 'Origin', controls: [
			{ label: 'Received', type: 'color', var: '--origin-received' },
			{ label: 'Discovered', type: 'color', var: '--origin-discovered' },
			{ label: 'Mixed', type: 'color', var: '--origin-mixed' },
			{ label: 'None', type: 'color', var: '--origin-none' } ] },
		cogStage: { name: 'Stage', controls: [
			{ label: 'Spark', type: 'color', var: '--stage-spark' },
			{ label: 'Birth', type: 'color', var: '--stage-birth' },
			{ label: 'Growth', type: 'color', var: '--stage-growth' },
			{ label: 'Maturity', type: 'color', var: '--stage-maturity' },
			{ label: 'Dormancy', type: 'color', var: '--stage-dormancy' },
			{ label: 'Archival', type: 'color', var: '--stage-archival' } ] },
		// §C Phase 9 wiring-audit — "Width" removed: the sidebar is sized by its drag-resize handle
		// (a JS inline width), which a CSS var can't override. Background is a per-sidebar override.
		cSidebar: { name: 'Sidebar shell', controls: [
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
		// MIG-070 §C — the Sky View graph CANVAS background. Its own var so it's independent of the
		// panel/sidebar surface (--background-secondary): recolour the graph without touching the
		// chrome. Consumed by GraphMindView's .gm-container and LocalSkyView's .local-star (both have
		// a transparent canvas, so this colour IS the visible background). Unset = panel surface = today.
		skyCanvas: { name: 'Canvas', controls: [{ label: 'Background', type: 'color', var: '--skyview-bg' }] },
		// MIG-072 §2 — Sky View NODE colours. Each node's base fill is its library/folder colour (set
		// per-library); "Default node colour" is the fallback for un-coloured nodes. The rings/glows layer
		// on top by note property. (Selection ring is intentionally the selected library's colour — not a
		// static style — so it has no control here.) Consumed by skyPalette.ts → setPalette → graphEngine.
		// Each ring has its own colour + frame width + solid/dotted (Eisa 2026-06-08). Width is a multiplier
		// (1 = today); the FRAME_STYLE_OPTS are shared. Frame vars are --skyview-frame-<id>-width/-style.
		skyNodes: { name: 'Nodes', controls: [
			{ label: 'Default node colour', type: 'color', var: '--skyview-node-default' },
			{ label: 'First ring gap (all)', type: 'range', var: '--skyview-ring-base', min: 0.5, max: 6, step: 0.5, unit: '', def: 1.5 },
			{ label: 'Gap between rings (all)', type: 'range', var: '--skyview-ring-gap', min: 1.5, max: 7, step: 0.1, unit: '', def: 2.6 },
			{ label: 'Open-note ring', type: 'color', var: '--skyview-ring-active' },
			{ label: 'Open-note width', type: 'range', var: '--skyview-frame-active-width', min: 0.5, max: 3, step: 0.25, unit: '', def: 1.5 },
			{ label: 'Open-note style', type: 'select', var: '--skyview-frame-active-style', options: FRAME_STYLE_OPTS },
			{ label: 'Pinned ring', type: 'color', var: '--skyview-ring-pinned' },
			{ label: 'Pinned width', type: 'range', var: '--skyview-frame-pinned-width', min: 0.5, max: 3, step: 0.25, unit: '', def: 1.5 },
			{ label: 'Pinned style', type: 'select', var: '--skyview-frame-pinned-style', options: FRAME_STYLE_OPTS },
			{ label: 'Orphan ring', type: 'color', var: '--skyview-ring-orphan' },
			{ label: 'Orphan width', type: 'range', var: '--skyview-frame-orphan-width', min: 0.5, max: 3, step: 0.25, unit: '', def: 1.5 },
			{ label: 'Orphan style', type: 'select', var: '--skyview-frame-orphan-style', options: FRAME_STYLE_OPTS },
			{ label: 'Dimmed opacity', type: 'range', var: '--skyview-dim-alpha', min: 0, max: 0.5, step: 0.02, unit: '', def: 0.12 } ] },
		// Seed has no maturity ring in the graph (by design — youngest state, no emphasis), so no Seed
		// control here; --skyview-maturity-seed stays in the palette only as the unknown-state fallback.
		skyMaturity: { name: 'Maturity rings', controls: [
			{ label: 'Sapling', type: 'color', var: '--skyview-maturity-sapling' },
			{ label: 'Sapling width', type: 'range', var: '--skyview-frame-sapling-width', min: 0.5, max: 3, step: 0.25, unit: '', def: 1.5 },
			{ label: 'Sapling style', type: 'select', var: '--skyview-frame-sapling-style', options: FRAME_STYLE_OPTS },
			{ label: 'Evergreen', type: 'color', var: '--skyview-maturity-evergreen' },
			{ label: 'Evergreen width', type: 'range', var: '--skyview-frame-evergreen-width', min: 0.5, max: 3, step: 0.25, unit: '', def: 1.5 },
			{ label: 'Evergreen style', type: 'select', var: '--skyview-frame-evergreen-style', options: FRAME_STYLE_OPTS },
			{ label: 'Canonical', type: 'color', var: '--skyview-maturity-canonical' },
			{ label: 'Canonical width', type: 'range', var: '--skyview-frame-canonical-width', min: 0.5, max: 3, step: 0.25, unit: '', def: 1.5 },
			{ label: 'Canonical style', type: 'select', var: '--skyview-frame-canonical-style', options: FRAME_STYLE_OPTS },
			{ label: 'Wilting', type: 'color', var: '--skyview-maturity-wilting' },
			{ label: 'Wilting width', type: 'range', var: '--skyview-frame-wilting-width', min: 0.5, max: 3, step: 0.25, unit: '', def: 1.5 },
			{ label: 'Wilting style', type: 'select', var: '--skyview-frame-wilting-style', options: FRAME_STYLE_OPTS } ] },
		skyGlow: { name: 'Glows & MOC', controls: [
			{ label: 'Received glow', type: 'color', var: '--skyview-glow-received' },
			{ label: 'Discovered glow', type: 'color', var: '--skyview-glow-discovered' },
			{ label: 'Map-of-content ring', type: 'color', var: '--skyview-moc-ring' },
			{ label: 'MOC width', type: 'range', var: '--skyview-frame-moc-width', min: 0.5, max: 3, step: 0.25, unit: '', def: 1.5 },
			{ label: 'MOC style', type: 'select', var: '--skyview-frame-moc-style', options: FRAME_STYLE_OPTS },
			{ label: 'Glow strength', type: 'range', var: '--skyview-glow-alpha', min: 0, max: 0.4, step: 0.02, unit: '', def: 0.06 },
			{ label: 'Stratum glow strength', type: 'range', var: '--skyview-stratum-alpha', min: 0, max: 0.4, step: 0.02, unit: '', def: 0.08 } ] },
		// MIG-072 §3 — links & overlays. "Edges" = the graph connections (named distinctly from the
		// typed-link colour editor in the Links category). Untyped/hover edges, direction arrows, semantic
		// (AI) links, cluster bubbles, + the in-scope opacity sliders.
		skyLinks: { name: 'Edges', controls: [
			{ label: 'Untyped edge', type: 'color', var: '--skyview-edge-normal' },
			{ label: 'Untyped edge opacity', type: 'range', var: '--skyview-edge-normal-alpha', min: 0, max: 1, step: 0.02, unit: '', def: 0.2 },
			{ label: 'Hover edge (untyped)', type: 'color', var: '--skyview-edge-highlight' },
			{ label: 'Outgoing arrow', type: 'color', var: '--skyview-arrow-out' },
			{ label: 'Incoming arrow', type: 'color', var: '--skyview-arrow-in' },
			{ label: 'Semantic (AI) link', type: 'color', var: '--skyview-semantic' },
			{ label: 'Semantic opacity', type: 'range', var: '--skyview-semantic-alpha', min: 0, max: 1, step: 0.02, unit: '', def: 0.6 },
			{ label: 'Cluster bubble', type: 'color', var: '--skyview-cluster' } ] },
		skyOverlays: { name: 'Overlays', controls: [
			{ label: 'Trail path', type: 'color', var: '--skyview-trail' },
			{ label: 'Badge: Title', type: 'color', var: '--skyview-badge-title' },
			{ label: 'Badge: Content', type: 'color', var: '--skyview-badge-content' },
			{ label: 'Badge: Tag', type: 'color', var: '--skyview-badge-tag' },
			{ label: 'Badge: Property', type: 'color', var: '--skyview-badge-property' },
			{ label: 'Badge: Wikilink', type: 'color', var: '--skyview-badge-wikilink' },
			{ label: 'Badge: Semantic', type: 'color', var: '--skyview-badge-semantic' },
			{ label: 'Badge: Structured', type: 'color', var: '--skyview-badge-structured' } ] },
		// MIG-072 §4 — node labels (full font, Eisa) + the 3D axis gizmo. Label size overrides the Sky
		// View ⚙ "Label font size" when set; unset = the ⚙ value (Predecessor reconciliation).
		skyLabels: { name: 'Labels', controls: [
			{ label: 'Label colour', type: 'color', var: '--skyview-label' },
			{ label: 'Label font', type: 'select', var: '--skyview-label-font', options: FONTS },
			{ label: 'Label size', type: 'range', var: '--skyview-label-size', min: 8, max: 28, step: 1, unit: 'px', def: 12 },
			{ label: 'Label thickness', type: 'range', var: '--skyview-label-weight', min: 300, max: 900, step: 100, unit: '', def: 400 } ] },
		skyGizmo: { name: '3D gizmo', controls: [
			{ label: 'X axis', type: 'color', var: '--skyview-gizmo-x' },
			{ label: 'Y axis', type: 'color', var: '--skyview-gizmo-y' },
			{ label: 'Z axis', type: 'color', var: '--skyview-gizmo-z' },
			{ label: 'Centre dot', type: 'color', var: '--skyview-gizmo-centre' } ] },
		// MIG-075 FU-3 — the CNS gravity well's minimal wired set (key = its
		// data-style-target, so the ⌖ inspect jumps here). Background applies
		// live; the two hover-label colors are read once at CNS mount
		// (Perf Rule 3) so they apply on the next CNS open.
		cns: { name: 'Nervous System (CNS)', controls: [
			{ label: 'Background', type: 'color', var: '--cns-bg' },
			{ label: 'Hover label background', type: 'color', var: '--cns-label-bg' },
			{ label: 'Hover label text', type: 'color', var: '--cns-label-text' },
			{ label: 'Text size', type: 'range', var: '--cns-label-size', min: 9, max: 24, step: 1, unit: 'px', def: 12 } ] },
		// MIG-081 §C.2d — the rich Calendar (CalendarPanel). Every token is consumed as
		// var(--cal-X, default) in CalendarPanel, so these apply live on Apply (and in the
		// centre preview, which renders the real CalendarPanel under the draft wrapper).
		calendar: { name: 'Calendar', controls: [
			{ label: 'Calendar font', type: 'select', var: '--cal-font', options: FONTS },
			{ label: 'Header gradient start', type: 'color', var: '--cal-header-from' },
			{ label: 'Header gradient end', type: 'color', var: '--cal-header-to' },
			{ label: 'Month pill background', type: 'color', var: '--cal-pill-bg' },
			{ label: 'Month pill text', type: 'color', var: '--cal-pill-text' },
			{ label: 'Month pill border', type: 'color', var: '--cal-pill-border' },
			{ label: 'Month pill size', type: 'range', var: '--cal-pill-size', min: 14, max: 44, step: 1, unit: 'px', def: 24 },
			{ label: 'Sacred-month pill start', type: 'color', var: '--cal-pill-sacred-from' },
			{ label: 'Sacred-month pill end', type: 'color', var: '--cal-pill-sacred-to' },
			{ label: 'Sacred-month pill text', type: 'color', var: '--cal-pill-sacred-text' },
			{ label: 'Subtitle size', type: 'range', var: '--cal-subtitle-size', min: 8, max: 24, step: 1, unit: 'px', def: 13 },
			{ label: 'Today button size', type: 'range', var: '--cal-today-size', min: 9, max: 22, step: 1, unit: 'px', def: 14 },
			{ label: 'Nav arrow size', type: 'range', var: '--cal-nav-size', min: 12, max: 40, step: 1, unit: 'px', def: 22 },
			{ label: 'Day number', type: 'color', var: '--cal-primary-color' },
			{ label: 'Day number size', type: 'range', var: '--cal-day-size', min: 12, max: 36, step: 1, unit: 'px', def: 19 },
			{ label: 'Cross-reference date', type: 'color', var: '--cal-sub-color' },
			{ label: 'Cross-reference date size', type: 'range', var: '--cal-subdate-size', min: 7, max: 22, step: 1, unit: 'px', def: 11 },
			{ label: 'Moon glyph', type: 'color', var: '--cal-moon-color' },
			{ label: 'Moon glyph size', type: 'range', var: '--cal-moon-size', min: 7, max: 22, step: 1, unit: 'px', def: 12 },
			{ label: 'Today gradient start', type: 'color', var: '--cal-today-from' },
			{ label: 'Today gradient end', type: 'color', var: '--cal-today-to' },
			{ label: 'Today text', type: 'color', var: '--cal-today-text' },
			{ label: 'Cell background', type: 'color', var: '--cal-cell-bg' },
			{ label: 'Weekday header', type: 'color', var: '--cal-weekday-color' },
			{ label: 'Weekday header size', type: 'range', var: '--cal-weekday-size', min: 8, max: 22, step: 1, unit: 'px', def: 12 },
			{ label: 'Week number', type: 'color', var: '--cal-wk-color' },
			{ label: 'Week number size', type: 'range', var: '--cal-week-size', min: 7, max: 20, step: 1, unit: 'px', def: 12 },
			{ label: 'Grid lines', type: 'color', var: '--cal-grid-border' },
			{ label: 'Daily-note dot', type: 'color', var: '--cal-daily-dot' },
			{ label: 'Note dot', type: 'color', var: '--cal-note-dot' },
			{ label: 'Task dot', type: 'color', var: '--cal-task-dot' },
			{ label: 'Dot size', type: 'range', var: '--cal-dot-size', min: 4, max: 12, step: 1, unit: 'px', def: 6 },
			{ label: 'Holiday dot', type: 'color', var: '--cal-event-holiday' },
			{ label: 'Observance dot', type: 'color', var: '--cal-event-observance' },
			{ label: 'Special-day dot', type: 'color', var: '--cal-event-special' } ] },
		// MIG-080 §C.3 — Global Tasks (the universe task agenda). Each surface uses
		// var(--gt-X, var(--theme-var, default)) → follows the universe theme by
		// default, these override it. (Live two-zone — open the Tasks view to see edits.)
		globalTasks: { name: 'Global Tasks', controls: [
			{ label: 'Background', type: 'color', var: '--gt-bg' },
			{ label: 'Surface', type: 'color', var: '--gt-surface' },
			{ label: 'Text', type: 'color', var: '--gt-text' },
			{ label: 'Muted text', type: 'color', var: '--gt-muted' },
			{ label: 'Accent', type: 'color', var: '--gt-accent' },
			{ label: 'Border', type: 'color', var: '--gt-border' },
			{ label: 'Row hover', type: 'color', var: '--gt-hover' },
			{ label: 'Overdue date', type: 'color', var: '--gt-overdue' },
			{ label: 'Due-today date', type: 'color', var: '--gt-today' },
			{ label: 'Text size', type: 'range', var: '--gt-text-scale', min: 70, max: 140, step: 5, unit: '', def: 100 } ] },
		// MIG-080 §D-text — Cataloger (the universe review queue) text size. Scales the
		// CatalogerView chrome (calc(X * var(--cat-scale,1))) AND the embedded
		// SourceReviewPanel cards (via .cataloger-queue { --rs-scale: var(--cat-scale) }).
		cataloger: { name: 'Cataloger', controls: [
			{ label: 'Text size', type: 'range', var: '--cat-text-scale', min: 70, max: 140, step: 5, unit: '', def: 100 } ] },
	};
	// §3B — the left rail is organised into CATEGORIES (a.k.a. Surfaces), each grouping its
	// elements (Eisa). Interface + Editor both preview the main app window ('editor' surface);
	// the heavy plugins are their own preview surfaces.
	const CATEGORIES: { key: string; name: string; surface: string; elements: string[] }[] = [
		{ key: 'interface', name: 'Interface', surface: 'editor', elements: ['interface', 'fileTree', 'library', 'folder', 'cuniverse', 'universe', 'universePanel', 'statusbar'] },
		{ key: 'components', name: 'Components', surface: 'editor', elements: ['cDock', 'cToolbar', 'cLayoutBar', 'cTabs', 'cRightSidebar', 'cRsText', 'cButtons', 'cTags', 'cSidebar'] },
		{ key: 'editor', name: 'Editor', surface: 'editor', elements: ['noteBg', 'text', 'breadcrumb', 'summary', 'accent', 'link', 'h1', 'h2', 'h3', 'h4', 'h5', 'h6', 'bold', 'italic', 'strike', 'code', 'quote', 'caret'] },
		{ key: 'frontmatter', name: 'Properties', surface: 'editor', elements: ['pTags', 'pTaxo'] },
		{ key: 'global', name: 'Global', surface: 'editor', elements: ['gBackgrounds', 'gTextShades', 'gStatus', 'gAccent', 'gType', 'gShape', 'fonts'] },
		{ key: 'links', name: 'Links', surface: 'editor', elements: ['links'] },
		{ key: 'cognitive', name: 'Cognitive colours', surface: 'editor', elements: ['cogMaturity', 'cogConfidence', 'cogOrigin', 'cogStage'] },
		{ key: 'sky', name: 'Sky View', surface: 'sky', elements: ['skyCanvas', 'skyNodes', 'skyMaturity', 'skyGlow', 'skyLinks', 'skyOverlays', 'skyLabels', 'skyGizmo'] },
		{ key: 'cns', name: 'CNS', surface: 'cns', elements: ['cns'] },
		{ key: 'calendar', name: 'Calendar', surface: 'calendar', elements: ['calendar'] },
		{ key: 'globalTasks', name: 'Global Tasks', surface: 'editor', elements: ['globalTasks'] },
		{ key: 'org', name: 'OrgChart', surface: 'org', elements: ['accent', 'link'] },
		{ key: 'index', name: 'Index', surface: 'index', elements: ['accent'] },
		{ key: 'cataloger', name: 'Cataloger', surface: 'cataloger', elements: ['accent', 'cataloger'] },
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
	// §C-polish Item B — expand the saved-colour grid into named rows (name / rename / delete).
	let managingSwatches = $state(false);
	// §C-polish Item B — the swatch pending delete-confirmation. Delete is a deliberate two-step
	// (✕ → Remove/Cancel) in Manage mode only — never an accidental right-click (Eisa, 2026-06-07).
	let confirmDeleteHex = $state<string | null>(null);
	let draftName = $state('Untitled style');
	/** The draft: CSS-var → override value. Scoped to the preview wrapper; Apply → <body>. */
	let draft = $state<Record<string, string>>({});

	// §C redesign (Eisa) — the panel is RESIZABLE; its size persists across opens (localStorage, a
	// pure UI pref). And only the **Editor** category keeps the 3-zone layout (left · centre note
	// preview · right controls); every OTHER category is 2-zone (left sidebar + one wide right space,
	// no centre — the controls integrate their own preview, e.g. the live pill in each Links row).
	let panelW = $state(1180);
	let panelH = $state(760);
	// MIG-072 §2 — Sky View uses the CENTRE preview (three-zone), like the Editor: a focused, labelled
	// bubble preview (ss-skyprev) beats hunting a ring-change in a 7,600-node live graph. The chrome
	// surfaces keep the docked two-zone live-behind. (Editor was already three-zone.)
	// MIG-075 FU-3 — CNS joins the three-zone set for the same reason, plus a harder one: the well's
	// hover-label vars are read once at canvas mount, so the live-behind app CANNOT preview them —
	// only the mini gravity-well (ss-cnsprev), which reads the draft vars as CSS, can.
	// MIG-088 Phase 1 — Properties (frontmatter) is THREE-zone: its dedicated centre preview shows the
	// pill mimic. (Two-zone relies on the live app showing through behind the right-anchored panel — but
	// the panel would occlude the right-sidebar Properties panel, so live-behind can't preview it.)
	const twoZone = $derived(activeCategory !== 'editor' && activeCategory !== 'frontmatter' && activeCategory !== 'cognitive' && activeCategory !== 'sky' && activeCategory !== 'cns' && activeCategory !== 'calendar');

	const draftStyle = $derived(Object.entries(draft).map(([k, v]) => `${k}:${v}`).join(';'));
	const sel = $derived(selected ? ELEMENTS[selected] ?? null : null);

	// §C — the centre preview replicates the EXACT selected element (Eisa). Note/tree/global
	// elements share a sample shape; chrome widgets each have their own. (Heavy surfaces —
	// sky/org/index — keep their own alt preview, keyed on activeSurface below.)
	const NOTE_ELS = new Set(['noteBg', 'text', 'breadcrumb', 'summary', 'h1', 'h2', 'h3', 'h4', 'h5', 'h6', 'bold', 'italic', 'strike', 'code', 'quote', 'link', 'accent']);
	const TREE_ELS = new Set(['interface', 'fileTree', 'library', 'folder', 'cuniverse']);
	const GLOBAL_ELS = new Set(['gBackgrounds', 'gTextShades', 'gStatus', 'gAccent', 'gType', 'gShape']);
	const pk = $derived(
		!selected ? 'none'
		: NOTE_ELS.has(selected) ? 'note'
		: TREE_ELS.has(selected) ? 'tree'
		: GLOBAL_ELS.has(selected) ? 'global'
		: selected,
	);

	// MIG-072 §2 — live "stacked example" canvas for the Sky View Nodes group: a node carrying maturity +
	// MOC + open-note rings, drawn from the current draft so the spacing (first gap / gap between) + each
	// ring's width + style show LIVE. Mirrors the engine's strokeRing dot logic so the preview is faithful.
	let stackCanvas = $state<HTMLCanvasElement | null>(null);
	function ssDrawRing(ctx: CanvasRenderingContext2D, cx: number, cy: number, r: number, width: number, color: string, dotted: boolean) {
		if (dotted) {
			const dotR = Math.max(0.8, width * 0.7);
			const count = Math.max(6, Math.min(48, Math.round((2 * Math.PI * r) / (dotR * 5))));
			ctx.fillStyle = color;
			for (let i = 0; i < count; i++) { const a = (i / count) * 2 * Math.PI; ctx.beginPath(); ctx.arc(cx + r * Math.cos(a), cy + r * Math.sin(a), dotR, 0, 2 * Math.PI); ctx.fill(); }
		} else {
			ctx.beginPath(); ctx.arc(cx, cy, r, 0, 2 * Math.PI); ctx.lineWidth = width; ctx.strokeStyle = color; ctx.stroke();
		}
	}
	function ssDrawStack() {
		const cv = stackCanvas; if (!cv) return;
		const ctx = cv.getContext('2d'); if (!ctx) return;
		ctx.clearRect(0, 0, cv.width, cv.height);
		const cx = cv.width / 2, cy = cv.height / 2, r = 13, S = 3.0;
		const num = (v: string, d: number) => { const x = parseFloat(draft[v] ?? ''); return Number.isNaN(x) ? d : x; };
		const col = (v: string, d: string) => draft[v] || d;
		const dot = (v: string) => draft[v] === 'dotted';
		ctx.beginPath(); ctx.arc(cx, cy, r, 0, 2 * Math.PI); ctx.fillStyle = col('--skyview-node-default', '#a78bfa'); ctx.fill();
		const rings = [
			{ c: col('--skyview-maturity-evergreen', '#16a34a'), w: 1.5 * num('--skyview-frame-evergreen-width', 1.5), d: dot('--skyview-frame-evergreen-style') },
			{ c: col('--skyview-moc-ring', '#f59e0b'), w: 1.5 * num('--skyview-frame-moc-width', 1.5), d: dot('--skyview-frame-moc-style') },
			{ c: col('--skyview-ring-active', '#333333'), w: 2 * num('--skyview-frame-active-width', 1.5), d: dot('--skyview-frame-active-style') },
		];
		let rr = r + num('--skyview-ring-base', 1.5) * S;
		for (const ring of rings) { ssDrawRing(ctx, cx, cy, rr, ring.w, ring.c, ring.d); rr += num('--skyview-ring-gap', 2.6) * S; }
	}
	$effect(() => { if (activeSurface === 'sky') { void draft; ssDrawStack(); } });

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
	// §C Phase 9 wiring-audit — font-size controls write appSettings (the working path, applied by
	// +layout as --font-ui-size / --font-text-size). Restores sizing that Phase 9.3 inadvertently broke.
	function setAppNum(setting: 'interfaceFontSize' | 'fontSize', val: number) {
		if (setting === 'interfaceFontSize') updateSettings({ interfaceFontSize: val });
		else updateSettings({ fontSize: val });
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

	// §C Option E item D — inspect-to-style. Toggle ON → the Setter goes click-through + hides its
	// panel so the REAL app is hoverable; the nearest `[data-style-target]` under the cursor is
	// highlighted + named; clicking it jumps the Setter to that element's controls and exits inspect.
	let inspecting = $state(false);
	let inspectRect = $state<{ x: number; y: number; w: number; h: number; label: string } | null>(null);
	function inspectTargetAt(target: EventTarget | null): HTMLElement | null {
		return ((target as HTMLElement)?.closest?.('[data-style-target]') as HTMLElement | null) ?? null;
	}
	function onInspectMove(e: PointerEvent) {
		if (!inspecting) return;
		const el = inspectTargetAt(e.target);
		const key = el?.getAttribute('data-style-target') ?? '';
		if (!el || !ELEMENTS[key]) { inspectRect = null; return; }
		const r = el.getBoundingClientRect();
		inspectRect = { x: r.left, y: r.top, w: r.width, h: r.height, label: L(ELEMENTS[key].name) };
	}
	function onInspectClick(e: MouseEvent) {
		if (!inspecting) return;
		const t = e.target as HTMLElement;
		if (t?.closest?.('.ss-inspect-banner')) return; // let the banner's Cancel button work
		// Capture (swallow) every other click while inspecting, so a stray click can't trigger an app
		// action (open a note, switch a tab). A click ON a tagged element jumps to its controls + exits.
		e.preventDefault();
		e.stopPropagation();
		const el = inspectTargetAt(t);
		const key = el?.getAttribute('data-style-target') ?? '';
		if (el && ELEMENTS[key]) {
			stopInspect();
			const cat = CATEGORIES.find((c) => c.elements.includes(key));
			if (cat) { activeCategory = cat.key; activeSurface = cat.surface; }
			selected = key;
		}
	}
	function startInspect() {
		if (inspecting) return;
		inspecting = true; inspectRect = null;
		window.addEventListener('pointermove', onInspectMove, true);
		window.addEventListener('click', onInspectClick, true);
	}
	function stopInspect() {
		if (!inspecting) return;
		inspecting = false; inspectRect = null;
		window.removeEventListener('pointermove', onInspectMove, true);
		window.removeEventListener('click', onInspectClick, true);
	}
	function toggleInspect() { if (inspecting) stopInspect(); else startInspect(); }
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

	// MIG-071 audit HIGH — restore saved-style IMPORT (its only caller, StylePresetsPanel, was deleted
	// in §K, so Export had become a dead end). Opens a file picker, validates, appends with a fresh id.
	async function importStyle() {
		try {
			const p = await importPreset();
			if (!p) return; // cancelled
			savedStyles = [...savedStyles, p];
			await saveStylePresets($state.snapshot(savedStyles) as StylePreset[]);
		} catch { /* invalid style file — ignore */ }
	}

	// §C Phase 6.3 — saved-style CRUD in the Setter (rename / delete / export), mirroring the
	// Settings → Styles panel. Lightweight rows only — NO stylePreview/unifiedStyleList (LL-032).
	let renamingId = $state<string | null>(null);
	let renameValue = $state('');
	function startRename(p: StylePreset) { renamingId = p.id; renameValue = p.name; }
	async function confirmRename() {
		const id = renamingId;
		savedStyles = savedStyles.map((s) => (s.id === id ? { ...s, name: renameValue.trim() || s.name } : s));
		renamingId = null;
		await saveStylePresets($state.snapshot(savedStyles) as StylePreset[]);
	}
	async function removeStyle(p: StylePreset) {
		savedStyles = savedStyles.filter((s) => s.id !== p.id);
		await saveStylePresets($state.snapshot(savedStyles) as StylePreset[]);
	}
	// §C — UPDATE an existing style with the CURRENT look (overwrite its captured sections, keep its
	// id + name). Keep() first so the unsaved draft is captured. Distinct from "+ Save current as a
	// style" (which makes a NEW style). A brief ✓ confirms the overwrite.
	let updatedId = $state<string | null>(null);
	let _updTimer: ReturnType<typeof setTimeout> | null = null;
	async function updateStyle(p: StylePreset) {
		keep();
		const keys = SECTION_CATALOGUE.filter((s) => s.defaultOn).map((s) => s.key);
		const fresh = newPresetFromCurrent(p.name, keys);
		savedStyles = savedStyles.map((s) => (s.id === p.id ? { ...fresh, id: p.id, createdAt: p.createdAt ?? fresh.createdAt } : s));
		await saveStylePresets($state.snapshot(savedStyles) as StylePreset[]);
		updatedId = p.id;
		if (_updTimer) clearTimeout(_updTimer);
		_updTimer = setTimeout(() => { if (updatedId === p.id) updatedId = null; }, 1500);
	}
	onDestroy(() => { if (_updTimer) clearTimeout(_updTimer); stopInspect(); if (typeof document !== 'undefined') document.body.classList.remove('ss-inspecting'); });

	// §C item 3 (REMOVED) — a built-in-theme picker in the Setter froze it AGAIN, even as a plain
	// `<select>` over BUILTIN_THEMES (2026-06-05). LL-032 strengthened: rendering BUILTIN_THEMES /
	// themes ANYWHERE in the Setter's render path freezes it (mechanism unreproducible). The theme
	// stays settable from Settings → Appearance; the Setter never touches BUILTIN_THEMES.

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
	// §C item D — never leave inspect on once the Setter closes (reads only $styleSetterOpen; stopInspect
	// self-guards, so this neither loops nor reads what it writes).
	$effect(() => { if (!$styleSetterOpen) stopInspect(); });
	// §C item D — the dock shortcut: open straight into inspect mode (no Settings in the way).
	$effect(() => {
		if ($styleSetterInspectRequest && $styleSetterOpen) { styleSetterInspectRequest.set(false); startInspect(); }
	});
	// MIG-007 — open straight to a category (the Links Settings hub deep-links to the Links category's
	// Link-Type editor). Reads request + open, clears the request, then navigates via pickCategory.
	$effect(() => {
		if ($styleSetterCategoryRequest && $styleSetterOpen) {
			const target = CATEGORIES.find((cat) => cat.key === $styleSetterCategoryRequest);
			styleSetterCategoryRequest.set(null);
			if (target) pickCategory(target);
		}
	});
	// §C item D — while inspecting, hide the Settings modal too (the Setter is often opened from it),
	// so the REAL app is fully hoverable however inspect was triggered. DOM-class side effect (not a
	// reactive write), so no loop; the global CSS rule below does the hiding.
	$effect(() => { if (typeof document !== 'undefined') document.body.classList.toggle('ss-inspecting', inspecting); });

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
		// §C-polish Item A — populate the font pickers with the user's installed fonts (once/session).
		ensureSystemFonts();
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
				if (inspecting) stopInspect();   // §C item D — Esc exits inspect first, then closes the Setter
				else closeStyleSetter();
			}
		}
		window.addEventListener('keydown', onKey, true);
		return () => window.removeEventListener('keydown', onKey, true);
	});
</script>

{#if $styleSetterOpen}
	{#if inspecting}
		<!-- §C item D — inspect overlay: a banner + a highlight box over the hovered chrome element.
		     The Setter panel is hidden + click-through while inspecting, so the real app is hoverable. -->
		<div class="ss-inspect-banner">⌖ Inspecting — click a part of the app to style it, or <button class="ss-inspect-cancel" onclick={stopInspect}>Cancel (Esc)</button></div>
		{#if inspectRect}
			<div class="ss-inspect-hl" style="left:{inspectRect.x}px; top:{inspectRect.y}px; width:{inspectRect.w}px; height:{inspectRect.h}px"><span class="ss-inspect-label">{inspectRect.label}</span></div>
		{/if}
	{/if}
	<div class="ss-overlay" class:ss-overlay--live={twoZone} class:ss-overlay--inspect={inspecting} role="dialog" aria-label={L('Style Setter')}>
		<div class="ss" class:ss--twozone={twoZone} style="width:{panelW}px;height:{panelH}px;{draftStyle}">
			<!-- Top bar -->
			<header class="ss-top">
				<span class="ss-brand"><span class="ss-star">✦</span> {L('Style Setter')}</span>
				{#if twoZone}<span class="ss-livetag" title={L('Your edits show on the real app live — Keep to save, Discard to revert')}>● {L('live')}</span>{/if}
				<span class="ss-draft">{L('draft')}: <input class="ss-dname" bind:value={draftName} /></span>
				<span class="ss-spacer"></span>
				<button class="ss-btn" class:ss-primary={inspecting} onclick={toggleInspect} title={L('Inspect — hover the real app and click a part to jump to its controls')}>⌖ {L('Inspect')}</button>
				<button class="ss-btn" onclick={resetDraft} title={L('Clear all overrides — back to the theme default')}>{L('Reset')}</button>
				<button class="ss-btn" onclick={discard} title={L('Abandon unsaved changes (the real app reverts)')}>{L('Discard')}</button>
				<button class="ss-btn ss-primary" onclick={keep} title={L('Save this look (per-Universe)')}>{L('Keep')}</button>
				<button class="ss-btn ss-icon" aria-label={L('Close')} onclick={closeStyleSetter}>✕</button>
			</header>

			<!-- Left rail: surfaces + themes -->
			<aside class="ss-left">
				<div class="ss-rlabel">{L('Surfaces')}</div>
				{#each CATEGORIES as cat (cat.key)}
					<button class="ss-surface" class:active={activeCategory === cat.key} onclick={() => pickCategory(cat)}>
						<span class="ss-sdot"></span> {L(cat.name)}
					</button>
					{#if activeCategory === cat.key}
						{#each cat.elements as elKey (elKey)}
							<button class="ss-elhead" class:active={selected === elKey} onclick={() => selectEl(elKey)}>{L(ELEMENTS[elKey].name)}</button>
						{/each}
					{/if}
				{/each}
				<div class="ss-divider"></div>
				<div class="ss-rlabel">{L('Saved styles')}</div>
				<div class="ss-stylelist">
					{#each savedStyles as p (p.id)}
						{#if renamingId === p.id}
							<div class="ss-srow-wrap">
								<input class="ss-srow-rename" bind:value={renameValue} dir="auto"
									onkeydown={(e) => { if (e.key === 'Enter') confirmRename(); if (e.key === 'Escape') renamingId = null; }} />
								<button class="ss-srow-ic" title={L('Save name')} aria-label={L('Save name')} onclick={confirmRename}>✓</button>
							</div>
						{:else}
							<div class="ss-srow-wrap">
								<button class="ss-srow" onclick={() => applyStyle(p)} title={'Apply ' + p.name} dir="auto">{p.name}</button>
								<span class="ss-srow-actions">
									<button class="ss-srow-ic" class:ss-srow-ok={updatedId === p.id} title={L('Update this style with the current look')} aria-label={L('Update')} onclick={() => updateStyle(p)}>{updatedId === p.id ? '✓' : '↻'}</button>
									<button class="ss-srow-ic" title={L('Export')} aria-label={L('Export')} onclick={() => exportPreset(p)}>⤓</button>
									<button class="ss-srow-ic" title={L('Rename')} aria-label={L('Rename')} onclick={() => startRename(p)}>✎</button>
									<button class="ss-srow-ic ss-srow-del" title={L('Delete')} aria-label={L('Delete')} onclick={() => removeStyle(p)}>✕</button>
								</span>
							</div>
						{/if}
					{/each}
					{#if !savedStyles.length}<div class="ss-srow-empty">{L('Design a look, then save it as a named style you can reuse.')}</div>{/if}
					<button class="ss-srow ss-srow-save" onclick={saveAsStyle}>+ {L('Save current as a style')}</button>
					<button class="ss-srow ss-srow-save" onclick={importStyle}>↥ {L('Import a style')}</button>
				</div>
			</aside>

			<!-- Center: focused preview of the SELECTED element (Eisa §C) -->
			<main class="ss-center">
				<div class="ss-hint">{selected ? L('Previewing') + ': ' + (sel?.name ? L(sel.name) : '') : L('Select an element on the left to preview & style it')}</div>
				<div class="ss-stage">
					{#if activeSurface !== 'editor'}
						<div class="ss-prev-alt" class:ss-prev-alt--sky={activeSurface === 'sky'} class:ss-prev-alt--cns={activeSurface === 'cns'} class:ss-prev-alt--calendar={activeSurface === 'calendar'}>
							<div class="ss-alt-title">{L(CATEGORIES.find((c) => c.surface === activeSurface)?.name)}</div>
							{#if activeSurface === 'sky'}
								<!-- MIG-072 §2 — live Sky View preview. Each bubble reads its --skyview-* var, so it
								     recolours as you pick. Click a group to style it; click the empty canvas for the
								     background colour. The card shows the canvas colour (--skyview-bg). -->
								<div class="ss-skyprev ss-hot" class:ss-sel={selected === 'skyCanvas'}
									role="button" tabindex="0" aria-label={L('canvas background')}
									onclick={() => selectEl('skyCanvas')}
									onkeydown={(e) => { if (e.key === 'Enter' || e.key === ' ') { e.preventDefault(); selectEl('skyCanvas'); } }}>
									<!-- MIG-072 §2 — live stacked example: a node with maturity + MOC + open-note rings,
									     drawn from the draft so the spacing/width/style controls show live. -->
									<div class="ssn-stackdemo ss-hot" class:ss-sel={selected === 'skyNodes'}
										role="button" tabindex="0" aria-label={L('stacked node example')}
										onclick={(e) => { e.stopPropagation(); selectEl('skyNodes'); }}
										onkeydown={(e) => { if (e.key === 'Enter' || e.key === ' ') { e.preventDefault(); e.stopPropagation(); selectEl('skyNodes'); } }}>
										<canvas bind:this={stackCanvas} width="280" height="240"></canvas>
										<div class="ssn-stack-cap">{L('Stacked example · maturity → MOC → open-note (live spacing)')}</div>
									</div>
									<div class="ssn-group ss-hot" class:ss-sel={selected === 'skyNodes'}
										role="button" tabindex="0" aria-label={L('nodes')}
										onclick={(e) => { e.stopPropagation(); selectEl('skyNodes'); }}
										onkeydown={(e) => { if (e.key === 'Enter' || e.key === ' ') { e.preventDefault(); e.stopPropagation(); selectEl('skyNodes'); } }}>
										<div class="ssn-title">{L('Nodes')}</div>
										<div class="ssn-row">
											<div class="ssn-cell"><span class="ssn-bub"></span><span class="ssn-cap">{L('Default')}</span></div>
											<div class="ssn-cell"><span class="ssn-bub ssn-active"></span><span class="ssn-cap">{L('Open note')}</span></div>
											<div class="ssn-cell"><span class="ssn-bub ssn-pinned"></span><span class="ssn-cap">{L('Pinned')}</span></div>
											<div class="ssn-cell"><span class="ssn-bub ssn-orphan"></span><span class="ssn-cap">{L('Orphan')}</span></div>
										</div>
									</div>
									<div class="ssn-group ss-hot" class:ss-sel={selected === 'skyMaturity'}
										role="button" tabindex="0" aria-label={L('maturity rings')}
										onclick={(e) => { e.stopPropagation(); selectEl('skyMaturity'); }}
										onkeydown={(e) => { if (e.key === 'Enter' || e.key === ' ') { e.preventDefault(); e.stopPropagation(); selectEl('skyMaturity'); } }}>
										<div class="ssn-title">{L('Maturity rings')}</div>
										<div class="ssn-row">
											<div class="ssn-cell"><span class="ssn-bub ssn-mat-sapling"></span><span class="ssn-cap">{L('Sapling')}</span></div>
											<div class="ssn-cell"><span class="ssn-bub ssn-mat-evergreen"></span><span class="ssn-cap">{L('Evergreen')}</span></div>
											<div class="ssn-cell"><span class="ssn-bub ssn-mat-canonical"></span><span class="ssn-cap">{L('Canonical')}</span></div>
											<div class="ssn-cell"><span class="ssn-bub ssn-mat-wilting"></span><span class="ssn-cap">{L('Wilting')}</span></div>
										</div>
									</div>
									<div class="ssn-group ss-hot" class:ss-sel={selected === 'skyGlow'}
										role="button" tabindex="0" aria-label={L('glows and moc')}
										onclick={(e) => { e.stopPropagation(); selectEl('skyGlow'); }}
										onkeydown={(e) => { if (e.key === 'Enter' || e.key === ' ') { e.preventDefault(); e.stopPropagation(); selectEl('skyGlow'); } }}>
										<div class="ssn-title">{L('Glows & MOC')}</div>
										<div class="ssn-row">
											<div class="ssn-cell"><span class="ssn-bub ssn-glow-recv"></span><span class="ssn-cap">{L('Received')}</span></div>
											<div class="ssn-cell"><span class="ssn-bub ssn-glow-disc"></span><span class="ssn-cap">{L('Discovered')}</span></div>
											<div class="ssn-cell"><span class="ssn-bub ssn-moc"></span><span class="ssn-cap">{L('MOC')}</span></div>
										</div>
									</div>
									<div class="ssn-group ss-hot" class:ss-sel={selected === 'skyLinks'}
										role="button" tabindex="0" aria-label={L('edges')}
										onclick={(e) => { e.stopPropagation(); selectEl('skyLinks'); }}
										onkeydown={(e) => { if (e.key === 'Enter' || e.key === ' ') { e.preventDefault(); e.stopPropagation(); selectEl('skyLinks'); } }}>
										<div class="ssn-title">{L('Edges')}</div>
										<div class="ssn-row">
											<div class="ssn-cell"><span class="ssn-line ssn-edge-normal"></span><span class="ssn-cap">{L('Untyped')}</span></div>
											<div class="ssn-cell"><span class="ssn-line ssn-edge-hover"></span><span class="ssn-cap">{L('Hover')}</span></div>
											<div class="ssn-cell"><span class="ssn-line ssn-edge-semantic"></span><span class="ssn-cap">{L('Semantic')}</span></div>
											<div class="ssn-cell"><span class="ssn-arrow ssn-arrow-out">➜</span><span class="ssn-cap">{L('Out')}</span></div>
											<div class="ssn-cell"><span class="ssn-arrow ssn-arrow-in">➜</span><span class="ssn-cap">{L('In')}</span></div>
											<div class="ssn-cell"><span class="ssn-cbub ssn-cluster"></span><span class="ssn-cap">{L('Cluster')}</span></div>
										</div>
									</div>
									<div class="ssn-group ss-hot" class:ss-sel={selected === 'skyOverlays'}
										role="button" tabindex="0" aria-label={L('overlays')}
										onclick={(e) => { e.stopPropagation(); selectEl('skyOverlays'); }}
										onkeydown={(e) => { if (e.key === 'Enter' || e.key === ' ') { e.preventDefault(); e.stopPropagation(); selectEl('skyOverlays'); } }}>
										<div class="ssn-title">{L('Overlays')}</div>
										<div class="ssn-row">
											<div class="ssn-cell"><span class="ssn-line ssn-trail"></span><span class="ssn-cap">{L('Trail')}</span></div>
											<div class="ssn-cell"><span class="ssn-badge ssn-b-title">T</span><span class="ssn-cap">{L('Title')}</span></div>
											<div class="ssn-cell"><span class="ssn-badge ssn-b-content">C</span><span class="ssn-cap">{L('Content')}</span></div>
											<div class="ssn-cell"><span class="ssn-badge ssn-b-tag">#</span><span class="ssn-cap">{L('Tag')}</span></div>
											<div class="ssn-cell"><span class="ssn-badge ssn-b-property">P</span><span class="ssn-cap">{L('Property')}</span></div>
											<div class="ssn-cell"><span class="ssn-badge ssn-b-wikilink">W</span><span class="ssn-cap">{L('Wikilink')}</span></div>
											<div class="ssn-cell"><span class="ssn-badge ssn-b-semantic">S</span><span class="ssn-cap">{L('Semantic')}</span></div>
											<div class="ssn-cell"><span class="ssn-badge ssn-b-structured">?</span><span class="ssn-cap">{L('Struct.')}</span></div>
										</div>
									</div>
									<div class="ssn-group ss-hot" class:ss-sel={selected === 'skyLabels'}
										role="button" tabindex="0" aria-label={L('labels')}
										onclick={(e) => { e.stopPropagation(); selectEl('skyLabels'); }}
										onkeydown={(e) => { if (e.key === 'Enter' || e.key === ' ') { e.preventDefault(); e.stopPropagation(); selectEl('skyLabels'); } }}>
										<div class="ssn-title">{L('Labels')}</div>
										<div class="ssn-row"><span class="ssn-labelsample">{L('Apple (Fruit)')}</span></div>
									</div>
									<div class="ssn-group ss-hot" class:ss-sel={selected === 'skyGizmo'}
										role="button" tabindex="0" aria-label={L('3d gizmo')}
										onclick={(e) => { e.stopPropagation(); selectEl('skyGizmo'); }}
										onkeydown={(e) => { if (e.key === 'Enter' || e.key === ' ') { e.preventDefault(); e.stopPropagation(); selectEl('skyGizmo'); } }}>
										<div class="ssn-title">{L('3D gizmo')}</div>
										<div class="ssn-row">
											<div class="ssn-cell"><span class="ssn-gz ssn-gz-x">X</span><span class="ssn-cap">{L('X')}</span></div>
											<div class="ssn-cell"><span class="ssn-gz ssn-gz-y">Y</span><span class="ssn-cap">{L('Y')}</span></div>
											<div class="ssn-cell"><span class="ssn-gz ssn-gz-z">Z</span><span class="ssn-cap">{L('Z')}</span></div>
											<div class="ssn-cell"><span class="ssn-gz-dot"></span><span class="ssn-cap">{L('Centre')}</span></div>
										</div>
									</div>
								</div>
							{:else if activeSurface === 'org'}
								<div class="ss-sky">
									<button class="ss-node ss-hot" class:ss-sel={selected === 'accent'} onclick={() => selectEl('accent')} aria-label={L('accent')}></button>
									<button class="ss-node b ss-hot" class:ss-sel={selected === 'link'} onclick={() => selectEl('link')} aria-label={L('link')}></button>
								</div>
							{:else if activeSurface === 'index'}
								<div class="ss-idx">
									<div class="ss-irow"><button class="ss-ibar ss-hot" style="width:70%" class:ss-sel={selected === 'accent'} onclick={() => selectEl('accent')} aria-label={L('accent')}></button> apple</div>
									<div class="ss-irow"><span class="ss-ibar" style="width:45%"></span> banana</div>
									<div class="ss-irow"><span class="ss-ibar" style="width:30%"></span> carrot</div>
								</div>
							{:else if activeSurface === 'cns'}
								<!-- MIG-075 FU-3 — a mini gravity-well: the card IS the preview.
								     Background + the hover-label chip read the three --cns-* vars
								     live off the draft (the Links-pill lesson: no second copy). -->
								<button class="ss-cnsprev ss-hot" class:ss-sel={selected === 'cns'} onclick={() => selectEl('cns')} aria-label={L('Nervous System (CNS)')}>
									<span class="ss-cns-ring r1"></span>
									<span class="ss-cns-ring r2"></span>
									<span class="ss-cns-ring r3"></span>
									<span class="ss-cns-label">{L('Apple (Fruit)')}</span>
								</button>
							{:else if activeSurface === 'calendar'}
								<!-- §C.2d — the REAL CalendarPanel, filling the centre zone (full-center-zone rule).
								     It inherits the draft --cal-* from .ss, so it recolours live as you pick. One
								     element → any click selects it (header nav still works to scrub months). -->
								<div class="ss-calprev ss-hot" class:ss-sel={selected === 'calendar'}
									role="button" tabindex="0" aria-label={L('Calendar')}
									onclick={() => selectEl('calendar')}
									onkeydown={(e) => { if (e.key === 'Enter' || e.key === ' ') { e.preventDefault(); selectEl('calendar'); } }}>
									<CalendarPanel
										primarySystem={$appSettings.calendarPrimarySystem ?? 'gregorian'}
										secondarySystem={$appSettings.calendarSecondarySystem ?? 'none'}
										weekStart={$appSettings.calendarWeekStart ?? 0}
										showWeekNumbers={$appSettings.calendarShowWeekNumbers ?? true}
										corrections={$appSettings.calendarCorrections ?? {}}
										calculationMode={$appSettings.calendarCalculationMode ?? 'astronomical'}
										noteEntries={{}}
										taskEntries={{}}
										onDayClick={() => selectEl('calendar')}
									/>
								</div>
							{/if}
							<div class="ss-alt-note">{L('representative snapshot · re-colours with your edits')}</div>
						</div>
					{:else if pk === 'none'}
						<div class="ss-focus ss-focus-empty">Pick an element on the left — its preview appears here.</div>
					{:else if pk === 'note'}
						<div class="ss-focus ss-fcard ss-fnote">
							<span class="ss-breadcrumb ss-hot2" class:ss-sel={selected === 'breadcrumb'} onclick={() => selectEl('breadcrumb')}>📚 {L('My Library')} / {L('Apple (Fruit)')}</span>
							<span class="ss-title ss-hot2" class:ss-sel={selected === 'text' || selected === 'noteBg'} onclick={() => selectEl('text')}>{L('Apple (Fruit)')}</span>
							<span class="ss-summary ss-hot2" class:ss-sel={selected === 'summary'} onclick={() => selectEl('summary')}>{L('A crisp pome fruit — sweet, tart, and endlessly useful.')}</span>
							<span class="ss-h1 ss-hot2" class:ss-sel={selected === 'h1'} onclick={() => selectEl('h1')}>{L('Heading one')}</span>
							<span class="ss-h2 ss-hot2" class:ss-sel={selected === 'h2'} onclick={() => selectEl('h2')}>{L('Heading two')}</span>
							<span class="ss-h3 ss-hot2" class:ss-sel={selected === 'h3'} onclick={() => selectEl('h3')}>{L('Heading three')}</span>
							<span class="ss-body">
								{L('An')} <b class="ss-bold ss-hot2" class:ss-sel={selected === 'bold'} onclick={() => selectEl('bold')}>{L('apple')}</b>
								{L('a day pairs with a')} <i class="ss-italic ss-hot2" class:ss-sel={selected === 'italic'} onclick={() => selectEl('italic')}>{L('crisp')}</i>
								<span class="ss-link ss-hot2" class:ss-sel={selected === 'link'} onclick={() => selectEl('link')}>[[{L('Banana')}]]</span>
								<span class="ss-pill ss-hot2" class:ss-sel={selected === 'accent'} onclick={() => selectEl('accent')}>{L('supports')}</span>
								— {L('see')} <code class="ss-code ss-hot2" class:ss-sel={selected === 'code'} onclick={() => selectEl('code')}>juice()</code>,
								<s class="ss-strike ss-hot2" class:ss-sel={selected === 'strike'} onclick={() => selectEl('strike')}>{L('an old note')}</s>.
							</span>
							<span class="ss-quote ss-hot2" class:ss-sel={selected === 'quote'} onclick={() => selectEl('quote')}>“{L('An apple a day keeps the doctor away.')}”</span>
							<span class="ss-hrow">
								<span class="ss-h4 ss-hot2" class:ss-sel={selected === 'h4'} onclick={() => selectEl('h4')}>H4</span>
								<span class="ss-h5 ss-hot2" class:ss-sel={selected === 'h5'} onclick={() => selectEl('h5')}>H5</span>
								<span class="ss-h6 ss-hot2" class:ss-sel={selected === 'h6'} onclick={() => selectEl('h6')}>H6</span>
							</span>
						</div>
					{:else if pk === 'tree'}
						<div class="ss-focus ss-fcard ss-ftree">
							<span class="ss-lib ss-hot2" class:ss-sel={selected === 'library' || selected === 'interface'} onclick={() => selectEl('library')}>📚 {L('My Library')}</span>
							<span class="ss-folder ss-hot2" class:ss-sel={selected === 'folder'} onclick={() => selectEl('folder')}>📁 {L('Ideas')}</span>
							<span class="ss-file ss-hot2" class:ss-sel={selected === 'fileTree'} onclick={() => selectEl('fileTree')}>{L('Apple (Fruit)')}</span>
							<span class="ss-file dim ss-hot2" class:ss-sel={selected === 'fileTree'} onclick={() => selectEl('fileTree')}>{L('Banana')}</span>
							<span class="ss-cuniverse ss-hot2" class:ss-sel={selected === 'cuniverse'} onclick={() => selectEl('cuniverse')}>✦ {L('Linked Universe')}</span>
						</div>
					{:else if pk === 'universe'}
						<div class="ss-focus ss-fcard"><span class="ss-univ" style="margin-top:0">◇ {L('Universe')}</span></div>
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
					{:else if pk === 'pTags'}
						<div class="ss-focus ss-fcard ss-feprops">
							<div class="ss-fep-title">{L('Apple (Fruit)')}</div>
							<div class="ss-fep-row ss-fep-hot"><span class="ss-fep-key"># tags</span><div class="ss-fep-vals"><span class="ss-petag">fruit</span><span class="ss-petag">orchard</span><span class="ss-petag">pome</span></div></div>
							<div class="ss-fep-row"><span class="ss-fep-key">maturity</span><span class="ss-fep-plain">evergreen</span></div>
							<div class="ss-fep-row"><span class="ss-fep-key">domain</span><div class="ss-fep-vals"><span class="ss-petaxo">Botany</span></div></div>
						</div>
					{:else if pk === 'pTaxo'}
						<div class="ss-focus ss-fcard ss-feprops">
							<div class="ss-fep-title">{L('Apple (Fruit)')}</div>
							<div class="ss-fep-row"><span class="ss-fep-key"># tags</span><div class="ss-fep-vals"><span class="ss-petag">fruit</span></div></div>
							<div class="ss-fep-row ss-fep-hot"><span class="ss-fep-key">domain</span><div class="ss-fep-vals"><span class="ss-petaxo">Botany</span><span class="ss-petaxo">Horticulture</span></div></div>
							<div class="ss-fep-row ss-fep-hot"><span class="ss-fep-key">field</span><div class="ss-fep-vals"><span class="ss-petaxo">Pomology</span></div></div>
						</div>
					{:else if pk === 'cogMaturity'}
						<div class="ss-focus ss-fcard ss-fcog">
							<div class="ss-fep-title">{L('Maturity')}</div>
							<div class="ss-cog-row"><span class="ss-cog-bar" style="background:var(--maturity-seed, #9ca3af)"></span><span class="ss-cog-lbl">{L('Seed')}</span><span class="ss-cog-note">360.3D</span></div>
							<div class="ss-cog-row"><span class="ss-cog-bar" style="background:var(--maturity-sapling, #4ade80)"></span><span class="ss-cog-lbl">{L('Sapling')}</span><span class="ss-cog-note">{L('File tree')} · {L('Top bar & tabs')}</span></div>
							<div class="ss-cog-row"><span class="ss-cog-bar" style="background:var(--maturity-evergreen, #16a34a)"></span><span class="ss-cog-lbl">{L('Evergreen')}</span></div>
							<div class="ss-cog-row"><span class="ss-cog-bar" style="background:var(--maturity-canonical, #f59e0b)"></span><span class="ss-cog-lbl">{L('Canonical')}</span></div>
							<div class="ss-cog-row"><span class="ss-cog-bar" style="background:var(--maturity-wilting, rgba(22,163,74,0.4))"></span><span class="ss-cog-lbl">{L('Wilting')}</span></div>
						</div>
					{:else if pk === 'cogConfidence'}
						<div class="ss-focus ss-fcard ss-fcog">
							<div class="ss-fep-title">{L('Confidence')}</div>
							<div class="ss-cog-row"><span class="ss-cog-bar" style="background:var(--confidence-hypothesis, color-mix(in srgb, var(--interactive-accent, #7c3aed) 14%, transparent))"></span><span class="ss-cog-lbl">{L('Hypothesis')}</span></div>
							<div class="ss-cog-row"><span class="ss-cog-bar" style="background:var(--confidence-evidence, color-mix(in srgb, var(--interactive-accent, #7c3aed) 40%, transparent))"></span><span class="ss-cog-lbl">{L('Evidence')}</span></div>
							<div class="ss-cog-row"><span class="ss-cog-bar" style="background:var(--confidence-established, var(--interactive-accent, #7c3aed))"></span><span class="ss-cog-lbl">{L('Established')}</span></div>
							<div class="ss-cog-row"><span class="ss-cog-bar" style="background:var(--confidence-contested, #d97706)"></span><span class="ss-cog-lbl">{L('Contested')}</span></div>
						</div>
					{:else if pk === 'cogOrigin'}
						<div class="ss-focus ss-fcard ss-fcog">
							<div class="ss-fep-title">{L('Origin')}</div>
							<div class="ss-cog-row"><span class="ss-cog-bar" style="background:var(--origin-received, #4A9EFF)"></span><span class="ss-cog-lbl">{L('Received')}</span></div>
							<div class="ss-cog-row"><span class="ss-cog-bar" style="background:var(--origin-discovered, #FFB347)"></span><span class="ss-cog-lbl">{L('Discovered')}</span></div>
							<div class="ss-cog-row"><span class="ss-cog-bar" style="background:var(--origin-mixed, #A78BFA)"></span><span class="ss-cog-lbl">{L('Mixed')}</span></div>
							<div class="ss-cog-row"><span class="ss-cog-bar" style="background:var(--origin-none, #9ca3af)"></span><span class="ss-cog-lbl">{L('None')}</span></div>
						</div>
					{:else if pk === 'cogStage'}
						<div class="ss-focus ss-fcard ss-fcog">
							<div class="ss-fep-title">{L('Stage')}</div>
							<div class="ss-cog-row"><span class="ss-cog-bar" style="background:var(--stage-spark, #a78bfa)"></span><span class="ss-cog-lbl">{L('Spark')}</span></div>
							<div class="ss-cog-row"><span class="ss-cog-bar" style="background:var(--stage-birth, #94a3b8)"></span><span class="ss-cog-lbl">{L('Birth')}</span></div>
							<div class="ss-cog-row"><span class="ss-cog-bar" style="background:var(--stage-growth, #16a34a)"></span><span class="ss-cog-lbl">{L('Growth')}</span></div>
							<div class="ss-cog-row"><span class="ss-cog-bar" style="background:var(--stage-maturity, #7c3aed)"></span><span class="ss-cog-lbl">{L('Maturity')}</span></div>
							<div class="ss-cog-row"><span class="ss-cog-bar" style="background:var(--stage-dormancy, #f59e0b)"></span><span class="ss-cog-lbl">{L('Dormancy')}</span></div>
							<div class="ss-cog-row"><span class="ss-cog-bar" style="background:var(--stage-archival, #ef4444)"></span><span class="ss-cog-lbl">{L('Archival')}</span></div>
						</div>
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
					<div class="ss-rlabel">{L('Selected element')}</div>
					<div class="ss-selname">{L(sel.name)}</div>
						{#if GLOBAL_ELS.has(selected ?? '')}
							<!-- §C Option E item C — inline composite preview for the Global atoms. A sample card
							     built from the Global CSS vars; the .ss wrapper carries the draft, so every shade /
							     radius / border / accent edit shows here LIVE via the CSS cascade (no JS). -->
							<div class="ss-gprev">
								<div class="ss-gprev-card">
									<span class="ss-gprev-title">Aa Heading</span>
									<span class="ss-gprev-muted">Muted body text · <span class="ss-gprev-faint">faint caption</span></span>
									<span class="ss-gprev-row">
										<span class="ss-gprev-chip">accent</span>
										<span class="ss-gprev-tag">#tag</span>
										<span class="ss-gprev-st ss-gprev-err">error</span>
										<span class="ss-gprev-st ss-gprev-warn">warn</span>
										<span class="ss-gprev-st ss-gprev-ok">ok</span>
									</span>
								</div>
							</div>
						{/if}
						{#each sel.controls as c (c.label)}
						<div class="ss-ctrl">
							{#if c.type === 'range'}
								<label for={'ss-' + c.var}>{L(c.label)}<span class="ss-rval">{curNum(c.var, c.def)}{c.unit}</span></label>
								<input id={'ss-' + c.var} type="range" min={c.min} max={c.max} step={c.step}
									value={curNum(c.var, c.def)}
									oninput={(e) => setVar(c.var, (e.currentTarget as HTMLInputElement).value + c.unit)} />
							{:else if c.type === 'color'}
								<label for={'ss-' + c.var}>{L(c.label)}</label>
								<input id={'ss-' + c.var} type="color" value={hexOf(curVal(c.var))}
									onfocus={() => activeColorVar = c.var}
									oninput={(e) => { activeColorVar = c.var; setVar(c.var, (e.currentTarget as HTMLInputElement).value); }}
									onchange={(e) => addStyleSwatch((e.currentTarget as HTMLInputElement).value)} />
							{:else if c.type === 'scriptfont'}
								<label for={'ss-sf-' + c.script}>{L(c.label)}</label>
								<select id={'ss-sf-' + c.script} value={$appSettings.perScriptFonts?.[c.script] ?? ''} onchange={(e) => setPerScriptFont(c.script, (e.currentTarget as HTMLSelectElement).value)}>
									{#each c.options as [lbl, val] (lbl)}<option value={val}>{L(lbl)}</option>{/each}
								</select>
							{:else if c.type === 'toggle'}
								<label class="ss-toggle">
									<span>{L(c.label)}</span>
									<input type="checkbox" checked={$appSettings[c.setting]}
										onchange={(e) => setToggle(c.setting, (e.currentTarget as HTMLInputElement).checked)} />
								</label>
							{:else if c.type === 'pillrange'}
								<label for={'ss-pill-' + c.prop}>{L(c.label)}<span class="ss-rval">{pillShape[c.prop]}{c.unit}</span></label>
								<input id={'ss-pill-' + c.prop} type="range" min={c.min} max={c.max} step={c.step}
									value={pillShape[c.prop]}
									oninput={(e) => setPillShape(c.prop, parseInt((e.currentTarget as HTMLInputElement).value))} />
							{:else if c.type === 'pillselect'}
								<label for={'ss-pill-' + c.prop}>{L(c.label)}</label>
								<select id={'ss-pill-' + c.prop} value={String(pillShape[c.prop])}
									onchange={(e) => setPillShape(c.prop, parseInt((e.currentTarget as HTMLSelectElement).value))}>
									{#each c.options as [lbl, val] (val)}<option value={val}>{L(lbl)}</option>{/each}
								</select>
							{:else if c.type === 'appnum'}
								<label for={'ss-an-' + c.setting}>{L(c.label)}<span class="ss-rval">{$appSettings[c.setting] ?? c.def}{c.unit}</span></label>
								<input id={'ss-an-' + c.setting} type="range" min={c.min} max={c.max} step={c.step}
									value={$appSettings[c.setting] ?? c.def}
									oninput={(e) => setAppNum(c.setting, parseInt((e.currentTarget as HTMLInputElement).value))} />
							{:else}
								<!-- §C-polish Item A — a font picker (var contains "font") renders the live list
								     (generics + installed fonts) with each option in its OWN face; other selects
								     (underline / border / shadow / italic) keep their fixed options. -->
								{@const isFont = c.var.includes('font')}
								{@const opts = isFont ? fontOptions : c.options}
								<label for={'ss-' + c.var}>{L(c.label)}</label>
								<select id={'ss-' + c.var} value={curVal(c.var)} onchange={(e) => setVar(c.var, (e.currentTarget as HTMLSelectElement).value)}>
									{#each opts as [lbl, val] (val)}<option value={val} style={isFont ? `font-family:${val}` : ''}>{L(lbl)}</option>{/each}
								</select>
							{/if}
						</div>
					{/each}
					{#if selected === 'links'}
						<LinkTypesEditor embedded />
					{/if}
					{#if ($appSettings.styleSwatches ?? []).length && sel.controls.some((c) => c.type === 'color')}
						<div class="ss-rlabel ss-rlabel-row">
							<span>{L('Saved colours')}</span>
							<button class="ss-manage-tog" class:active={managingSwatches} onclick={() => { managingSwatches = !managingSwatches; confirmDeleteHex = null; }} title={L('Name, rename or remove saved colours')}>{managingSwatches ? L('Done') : L('Manage')}</button>
						</div>
						<div class="ss-swatches">
							{#each $appSettings.styleSwatches as sw (sw.hex)}
								<button class="ss-sw" style="background:{sw.hex}" title={(sw.name ? sw.name + ' · ' : '') + sw.hex + ' — click to apply (rename / remove via Manage)'} aria-label={sw.name || sw.hex} onclick={() => applySwatch(sw.hex)}></button>
							{/each}
						</div>
						{#if managingSwatches}
							<!-- §C-polish Item B — name / rename / delete saved colours (rows mirror the saved-styles
							     rows). Delete is a deliberate two-step (✕ → Remove / Cancel) — no accidental right-click. -->
							<div class="ss-swatch-rows">
								{#each $appSettings.styleSwatches as sw (sw.hex)}
									{#if confirmDeleteHex === sw.hex}
										<div class="ss-swatch-row ss-swatch-confirm">
											<span class="ss-swatch-chip" style="background:{sw.hex}"></span>
											<span class="ss-confirm-text">{L('Remove')} {sw.name || sw.hex}?</span>
											<button class="ss-confirm-yes" onclick={() => { removeStyleSwatch(sw.hex); confirmDeleteHex = null; }}>Remove</button>
											<button class="ss-confirm-no" onclick={() => confirmDeleteHex = null}>Cancel</button>
										</div>
									{:else}
										<div class="ss-swatch-row">
											<span class="ss-swatch-chip" style="background:{sw.hex}" title={sw.hex}></span>
											<input class="ss-swatch-name" placeholder={sw.hex} value={sw.name ?? ''} dir="auto"
												onchange={(e) => renameStyleSwatch(sw.hex, (e.currentTarget as HTMLInputElement).value)} />
											<button class="ss-srow-ic ss-srow-del" title={L('Remove colour')} aria-label={L('Remove colour')} onclick={() => confirmDeleteHex = sw.hex}>✕</button>
										</div>
									{/if}
								{/each}
							</div>
						{/if}
					{/if}
				{:else}
					<div class="ss-empty"><span class="ss-big">⊹</span>Click any part of the interface to style it. Its controls appear here, and changes show instantly.</div>
				{/if}
			</aside>
			<!-- §C redesign — corner grip to resize the whole panel (size persists across opens). -->
			<button class="ss-resize" aria-label={L('Resize panel')} title={L('Drag to resize')} onpointerdown={startResize}></button>
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
	/* §C item D — inspect mode: hide + click-through the panel so the real app is hoverable; a banner +
	   a highlight box (over the hovered chrome element) sit above everything. */
	.ss-overlay--inspect { background: none; backdrop-filter: none; pointer-events: none; }
	.ss-overlay--inspect .ss { display: none; }
	.ss-inspect-banner { position: fixed; top: 12px; left: 50%; transform: translateX(-50%); z-index: 9200; pointer-events: auto; background: var(--interactive-accent, #7c6cff); color: #fff; font: 13px ui-sans-serif, system-ui, sans-serif; padding: 7px 14px; border-radius: 999px; box-shadow: 0 8px 30px rgba(0,0,0,.4); display: flex; align-items: center; gap: 8px; }
	.ss-inspect-cancel { font: inherit; font-weight: 600; background: rgba(255,255,255,.2); color: #fff; border: none; border-radius: 999px; padding: 3px 10px; cursor: pointer; }
	.ss-inspect-cancel:hover { background: rgba(255,255,255,.32); }
	.ss-inspect-hl { position: fixed; z-index: 9150; pointer-events: none; border: 2px solid var(--interactive-accent, #7c6cff); background: color-mix(in srgb, var(--interactive-accent, #7c6cff) 14%, transparent); border-radius: 3px; }
	.ss-inspect-label { position: absolute; top: -22px; left: 0; background: var(--interactive-accent, #7c6cff); color: #fff; font: 11px ui-sans-serif, system-ui, sans-serif; font-weight: 600; padding: 2px 7px; border-radius: 5px; white-space: nowrap; }
	/* §C item D — while inspecting, hide the Settings modal (the Setter is often opened from it) so the
	   real app is fully hoverable. Global — the modal lives outside this component. */
	:global(body.ss-inspecting .settings-overlay) { display: none !important; }
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
	.ss-surface { display: flex; align-items: center; gap: 9px; padding: 7px 9px; border-radius: 8px; cursor: pointer; font: inherit; font-size: 13.5px; color: var(--c-text); background: none; border: none; text-align: start; }
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
	/* §C Phase 6.3 — per-row CRUD (export / rename / delete), revealed on hover. */
	.ss-srow-wrap { display: flex; align-items: center; gap: 2px; }
	.ss-srow-wrap .ss-srow { flex: 1; min-width: 0; }
	.ss-srow-actions { display: none; flex: none; }
	.ss-srow-wrap:hover .ss-srow-actions { display: flex; }
	.ss-srow-ic { background: none; border: none; color: var(--c-muted); cursor: pointer; font-size: 12px; line-height: 1; padding: 4px 5px; border-radius: 5px; }
	.ss-srow-ic:hover { background: var(--c-surface2); color: var(--c-text); }
	.ss-srow-del:hover { color: var(--text-error, #e5484d); }
	.ss-srow-ok { color: #4ade80 !important; }
	/* §C Option E item C — inline composite Global preview: built from the Global CSS vars, so it
	   reflects the draft live (the .ss wrapper carries the draft). No JS — pure CSS cascade. */
	.ss-gprev { margin-bottom: 14px; }
	.ss-gprev-card { background: var(--background-primary-alt, var(--background-primary, #fbfbfa)); border: var(--border-width, 1px) solid var(--background-modifier-border, #ddd); border-radius: var(--radius-m, 8px); padding: 12px 14px; display: flex; flex-direction: column; gap: 7px; }
	.ss-gprev-title { font-size: 15px; font-weight: 700; color: var(--text-normal, #2e3338); }
	.ss-gprev-muted { font-size: 12.5px; color: var(--text-muted, #8a8a8a); line-height: var(--line-height-normal, 1.6); }
	.ss-gprev-faint { color: var(--text-faint, #aaa); }
	.ss-gprev-row { display: flex; flex-wrap: wrap; gap: 6px; align-items: center; }
	.ss-gprev-chip { font-size: 11px; padding: 2px 9px; border-radius: var(--radius-s, 4px); background: var(--interactive-accent-hover, var(--interactive-accent, #7c3aed)); color: var(--text-on-accent, #fff); }
	.ss-gprev-tag { font-size: 11px; padding: 2px 9px; border-radius: var(--radius-l, 12px); background: var(--background-modifier-hover, rgba(0,0,0,.06)); color: var(--text-accent, var(--interactive-accent, #7c3aed)); }
	.ss-gprev-st { font-size: 11px; font-weight: 600; }
	.ss-gprev-err { color: var(--text-error, #e5484d); }
	.ss-gprev-warn { color: var(--text-warning, #f5a623); }
	.ss-gprev-ok { color: var(--text-success, #30a46c); }
	.ss-srow-rename { flex: 1; min-width: 0; font: inherit; font-size: 12.5px; padding: 5px 8px; border: 1px solid var(--c-accent); border-radius: 6px; background: var(--c-bg); color: var(--c-text); outline: none; }
	.ss-center { grid-area: center; display: flex; flex-direction: column; align-items: center; justify-content: center; padding: 20px; gap: 10px; background: var(--background-secondary, #14141c); }
	.ss-hint { font-size: 12px; color: var(--c-muted); }
	.ss-stage { position: relative; flex: 1; align-self: stretch; min-height: 0; display: flex; align-items: center; justify-content: center; }
	/* The mini interface — uses the REAL app vars (overridden by the draft on .ss). */
	.ss-prev { width: 560px; height: 360px; border-radius: 10px; overflow: hidden; display: grid; grid-template-columns: 124px 1fr; grid-template-rows: 1fr auto; grid-template-areas: "side main" "status status"; background: var(--background-primary, #fbfbfa); box-shadow: 0 14px 40px rgba(0,0,0,.45); border: 1px solid rgba(0,0,0,.25); }
	.ss-side { grid-area: side; overflow: hidden; background: var(--background-secondary, #f1f1ef); color: var(--text-normal, #2e3338); padding: 12px 10px; display: flex; flex-direction: column; gap: 8px; border: none; text-align: start; font-family: var(--font-interface-theme, inherit); }
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
	.ss-main { grid-area: main; background: var(--background-primary, #fbfbfa); color: var(--editor-text-color, var(--text-normal, #2e3338)); padding: 16px 18px; text-align: start; border: none; font-family: var(--font-text-theme, inherit); display: flex; flex-direction: column; gap: 7px; overflow-y: auto; }
	.ss-title { display: block; font-weight: 800; font-size: 18px; color: var(--editor-text-color, var(--text-normal, #2e3338)); }
	/* §C — note breadcrumb + summary previews, reading the same vars as NotePane's `.e-breadcrumb`/`.e-summary`. */
	.ss-breadcrumb { display: block; font-size: var(--breadcrumb-size, 12px); color: var(--breadcrumb-color, var(--text-normal, #2e3338)); opacity: .9; }
	.ss-summary { display: block; font-style: var(--summary-style, italic); font-size: var(--summary-size, 15px); color: var(--summary-color, var(--text-muted, #8a8a8a)); font-family: var(--summary-font, inherit); font-weight: var(--summary-weight, 400); line-height: 1.5; }
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
	/* Sky View preview uses the FULL centre zone (it has the most to show) instead of the fixed card. */
	.ss-prev-alt--sky { width: 100%; height: 100%; max-width: 1100px; }
	/* Style Setter Preview Rule (Eisa, 2026-06-11): take advantage of the ENTIRE centre
	   zone — never squeeze an element mimicry into a tiny box. Every preview card
	   stretches to the stage like --sky; the mimicry inside scales to the card. */
	.ss-prev-alt--cns { width: 100%; height: 100%; max-width: 1100px; padding: 18px 22px; }
	/* §C.2d — Calendar preview fills the whole centre zone (full-center-zone rule); the real
	   CalendarPanel scales to it. The wrapper is the single click-target (selects calendar). */
	.ss-prev-alt--calendar { width: 100%; height: 100%; max-width: 1100px; padding: 14px 18px; }
	.ss-calprev { align-self: stretch; flex: 1; width: 100%; min-height: 0; overflow: auto; display: flex; flex-direction: column; align-items: center; justify-content: flex-start; border-radius: 10px; border: 2px solid transparent; cursor: pointer; }
	.ss-calprev.ss-sel { border-color: #b9acff; }
	.ss-calprev :global(.cal-root) { max-width: 100%; }
	.ss-alt-title { font-weight: 700; font-size: 15px; color: var(--interactive-accent, #7c3aed); }
	.ss-alt-note { font-size: 11.5px; color: var(--text-normal, #6b7280); opacity: .7; max-width: 70%; text-align: center; }
	.ss-sky { display: flex; gap: 22px; }
	/* MIG-072 §2 — live Sky View preview card. Shows the chosen canvas colour (--skyview-bg) as the
	   backdrop, with bubble groups that each read their --skyview-* var so the preview recolours live.
	   Caption/title colours are fixed mid-tones so they stay readable on ANY canvas colour. */
	/* 2-column grid: the live stacked example (col 1, spans all rows) beside the per-type groups (col 2),
	   so the Nodes / Maturity / Glows&MOC groups all stay visible at once (no clip, no scroll). */
	.ss-skyprev { align-self: stretch; flex: 1; margin: 6px 0; border-radius: 10px; background: var(--skyview-bg, var(--background-secondary, #f1f1ef)); border: 2px solid transparent; display: grid; grid-template-columns: auto 1fr; column-gap: 16px; row-gap: 6px; align-content: center; padding: 12px 16px; cursor: pointer; transition: background 0.12s; overflow-y: auto; }
	.ss-skyprev > .ssn-group { grid-column: 2; }
	.ssn-group { border-radius: 9px; padding: 7px 9px; border: 2px solid transparent; cursor: pointer; }
	.ssn-group.ss-sel { border-color: #b9acff; background: rgba(185,172,255,0.10); }
	.ssn-title { font-size: 11px; font-weight: 700; color: #8a93a6; margin-bottom: 7px; letter-spacing: 0.02em; }
	.ssn-row { display: flex; flex-wrap: wrap; gap: 14px 18px; align-items: flex-start; }
	.ssn-cell { display: flex; flex-direction: column; align-items: center; gap: 5px; width: 56px; }
	.ssn-bub { width: 24px; height: 24px; border-radius: 50%; background: var(--skyview-node-default, #a78bfa); display: inline-block; flex: none; }
	.ssn-cap { font-size: 9.5px; color: #9aa3b2; text-align: center; }
	/* MIG-072 §2 — live stacked-node example (shows ring spacing/width/style as you drag the controls). */
	.ssn-stackdemo { grid-column: 1; grid-row: 1 / 4; align-self: center; display: flex; flex-direction: column; align-items: center; gap: 4px; padding: 6px; border-radius: 10px; border: 2px solid transparent; cursor: pointer; }
	.ssn-stackdemo.ss-sel { border-color: #b9acff; background: rgba(185,172,255,0.10); }
	.ssn-stack-cap { font-size: 10px; color: #8a93a6; text-align: center; }
	/* Ring frames use the `border` shorthand so each ring's live width + solid/dotted + colour show
	   per-ring (--skyview-frame-<id>-width/-style + the colour var). Glows stay box-shadow (halos). */
	.ssn-active, .ssn-pinned, .ssn-orphan,
	.ssn-mat-sapling, .ssn-mat-evergreen, .ssn-mat-canonical, .ssn-mat-wilting, .ssn-moc { box-sizing: border-box; }
	.ssn-active   { border: calc(var(--skyview-frame-active-width, 1.5) * 2.5px) var(--skyview-frame-active-style, solid) var(--skyview-ring-active, #333333); }
	.ssn-pinned   { border: calc(var(--skyview-frame-pinned-width, 1.5) * 2.5px) var(--skyview-frame-pinned-style, solid) var(--skyview-ring-pinned, #06b6d4); }
	.ssn-orphan   { border: calc(var(--skyview-frame-orphan-width, 1.5) * 2.5px) var(--skyview-frame-orphan-style, solid) var(--skyview-ring-orphan, #94a3b8); }
	.ssn-mat-sapling   { border: calc(var(--skyview-frame-sapling-width, 1.5) * 2.5px) var(--skyview-frame-sapling-style, solid) var(--skyview-maturity-sapling, #4ade80); }
	.ssn-mat-evergreen { border: calc(var(--skyview-frame-evergreen-width, 1.5) * 2.5px) var(--skyview-frame-evergreen-style, solid) var(--skyview-maturity-evergreen, #16a34a); }
	.ssn-mat-canonical { border: calc(var(--skyview-frame-canonical-width, 1.5) * 2.5px) var(--skyview-frame-canonical-style, solid) var(--skyview-maturity-canonical, #f59e0b); }
	.ssn-mat-wilting   { border: calc(var(--skyview-frame-wilting-width, 1.5) * 2.5px) var(--skyview-frame-wilting-style, solid) var(--skyview-maturity-wilting, #16a34a); }
	.ssn-moc { border: calc(var(--skyview-frame-moc-width, 1.5) * 2.5px) var(--skyview-frame-moc-style, solid) var(--skyview-moc-ring, #f59e0b); }
	.ssn-glow-recv { box-shadow: 0 0 11px 5px var(--skyview-glow-received, #4a9eff); }
	.ssn-glow-disc { box-shadow: 0 0 11px 5px var(--skyview-glow-discovered, #ffb347); }
	/* §3 — edge + overlay samples (live, read the --skyview-* vars). */
	.ssn-line { width: 30px; border-top-width: 3px; border-top-style: solid; margin: 10px 0 11px; display: inline-block; }
	.ssn-edge-normal { border-top-color: var(--skyview-edge-normal, #bcccdc); }
	.ssn-edge-hover { border-top-color: var(--skyview-edge-highlight, #f97316); }
	.ssn-edge-semantic { border-top-style: dotted; border-top-color: var(--skyview-semantic, #6366f1); }
	.ssn-trail { border-top-color: var(--skyview-trail, #ff6b6b); }
	.ssn-arrow { font-size: 17px; font-weight: 700; line-height: 24px; }
	.ssn-arrow-out { color: var(--skyview-arrow-out, #22c55e); }
	.ssn-arrow-in { color: var(--skyview-arrow-in, #ef4444); display: inline-block; transform: scaleX(-1); }
	.ssn-cbub { width: 24px; height: 16px; border-radius: 50%; background: color-mix(in srgb, var(--skyview-cluster, #7c3aed) 18%, transparent); border: 1px solid var(--skyview-cluster, #7c3aed); display: inline-block; }
	.ssn-badge { width: 18px; height: 18px; border-radius: 3px; color: #fff; font-size: 10px; font-weight: 700; display: inline-flex; align-items: center; justify-content: center; }
	.ssn-b-title { background: var(--skyview-badge-title, #3b82f6); }
	.ssn-b-content { background: var(--skyview-badge-content, #16a34a); }
	.ssn-b-tag { background: var(--skyview-badge-tag, #f472b6); }
	.ssn-b-property { background: var(--skyview-badge-property, #f59e0b); }
	.ssn-b-wikilink { background: var(--skyview-badge-wikilink, #60a5fa); }
	.ssn-b-semantic { background: var(--skyview-badge-semantic, #7c3aed); }
	.ssn-b-structured { background: var(--skyview-badge-structured, #94a3b8); }
	/* §4 — label sample (live colour/font/size/weight) + 3D gizmo sample. */
	.ssn-labelsample { color: var(--skyview-label, var(--text-normal, #1e293b)); font-family: var(--skyview-label-font, system-ui); font-size: var(--skyview-label-size, 14px); font-weight: var(--skyview-label-weight, 400); white-space: nowrap; }
	.ssn-gz { font-weight: 700; font-size: 18px; line-height: 24px; }
	.ssn-gz-x { color: var(--skyview-gizmo-x, #ef4444); }
	.ssn-gz-y { color: var(--skyview-gizmo-y, #22c55e); }
	.ssn-gz-z { color: var(--skyview-gizmo-z, #3b82f6); }
	.ssn-gz-dot { width: 13px; height: 13px; border-radius: 50%; background: var(--skyview-gizmo-centre, #333); display: inline-block; margin: 6px 0; }
	.ss-node { width: 34px; height: 34px; border-radius: 50%; border: none; cursor: pointer; background: var(--interactive-accent, #7c3aed); box-shadow: 0 0 0 4px color-mix(in srgb, var(--interactive-accent, #7c3aed) 25%, transparent); }
	.ss-node.b { background: var(--link-color, #2f6fed); box-shadow: 0 0 0 4px color-mix(in srgb, var(--link-color, #2f6fed) 25%, transparent); }
	.ss-idx { width: 70%; display: flex; flex-direction: column; gap: 8px; }
	.ss-irow { display: flex; align-items: center; gap: 8px; font-size: 12px; }
	.ss-ibar { height: 7px; background: var(--interactive-accent, #7c3aed); border-radius: 3px; border: none; cursor: pointer; }
	/* MIG-075 FU-3 — the gravity-well preview: reads the --cns-* vars live. Preview Rule
	   (Eisa, 2026-06-11): the well FILLS the card (flex: 1, stretched), and the percentile
	   rings scale to the available height — no fixed thumbnail pixels. */
	.ss-cnsprev {
		position: relative; align-self: stretch; flex: 1; min-height: 0;
		border: 1px solid var(--c-border);
		border-radius: 12px; cursor: pointer; overflow: hidden;
		background: var(--cns-bg, var(--background-primary, #fafafa));
		display: grid; place-items: center;
	}
	.ss-cns-ring { position: absolute; border: 1px dashed rgba(148, 163, 184, 0.5); border-radius: 50%; height: 28%; aspect-ratio: 1; max-width: 94%; }
	.ss-cns-ring.r2 { height: 56%; }
	.ss-cns-ring.r3 { height: 84%; }
	.ss-cns-label {
		/* font + em-padding track the "Text size" control like the real well's box does */
		position: relative; font-size: var(--cns-label-size, 12px); padding: 0.3em 0.9em; border-radius: 6px;
		background: var(--cns-label-bg, rgba(30,30,40,0.9));
		color: var(--cns-label-text, #ffffff);
	}
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
	/* §C-polish Item B — the "Manage" toggle + named-swatch rows (mirror the saved-styles row look). */
	.ss-rlabel-row { display: flex; align-items: center; justify-content: space-between; }
	.ss-manage-tog { font: inherit; font-size: 10.5px; text-transform: none; letter-spacing: 0; color: var(--c-muted); background: none; border: 1px solid var(--c-border); border-radius: 6px; padding: 2px 8px; cursor: pointer; }
	.ss-manage-tog:hover { color: var(--c-text); border-color: var(--c-accent); }
	.ss-manage-tog.active { color: #fff; background: var(--c-accent); border-color: var(--c-accent); }
	.ss-swatch-rows { display: flex; flex-direction: column; gap: 4px; margin: 2px 0 12px; }
	.ss-swatch-row { display: flex; align-items: center; gap: 7px; }
	.ss-swatch-chip { width: 20px; height: 20px; border-radius: 5px; border: 1px solid var(--c-border); flex: none; }
	.ss-swatch-name { flex: 1; min-width: 0; font: inherit; font-size: 12.5px; padding: 4px 8px; border: 1px solid var(--c-border); border-radius: 6px; background: var(--c-surface2); color: var(--c-text); outline: none; }
	.ss-swatch-name:focus { border-color: var(--c-accent); }
	/* §C-polish Item B — deliberate two-step delete confirm (replaces the accidental right-click). */
	.ss-swatch-confirm { gap: 6px; }
	.ss-confirm-text { flex: 1; min-width: 0; font-size: 12px; color: var(--c-text); overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
	.ss-confirm-yes { font: inherit; font-size: 11.5px; padding: 3px 9px; border-radius: 6px; border: 1px solid var(--text-error, #e5484d); background: var(--text-error, #e5484d); color: #fff; cursor: pointer; flex: none; }
	.ss-confirm-no { font: inherit; font-size: 11.5px; padding: 3px 9px; border-radius: 6px; border: 1px solid var(--c-border); background: var(--c-surface2); color: var(--c-text); cursor: pointer; flex: none; }
	/* §C — focused per-element preview: the centre shows JUST the selected element (Eisa). */
	.ss-focus { min-width: 460px; min-height: 300px; display: flex; flex-direction: column; align-items: center; justify-content: center; gap: 14px; }
	.ss-focus-empty { color: var(--c-muted); font-size: 13px; }
	.ss-fcard { background: var(--background-primary, #fbfbfa); color: var(--editor-text-color, var(--text-normal, #2e3338)); border: 1px solid rgba(0,0,0,.18); border-radius: 12px; box-shadow: 0 14px 40px rgba(0,0,0,.22); padding: 22px 26px; display: flex; flex-direction: column; gap: 9px; min-width: 320px; max-width: 460px; text-align: start; align-items: stretch; }
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
	/* MIG-088 Phase 1 — Properties (frontmatter) centre preview: a mini Properties panel. The pills
	   mirror PropertyEditor's .pe-tag / .pe-taxo-pill, reading the same draft vars with the same
	   fallbacks so they re-colour live as the user edits. .ss-fep-hot rings the selected element's row. */
	.ss-feprops { gap: 9px; align-items: stretch; }
	.ss-fep-title { font-size: 17px; font-weight: 700; color: var(--text-normal, #2e3338); margin-bottom: 3px; text-align: start; }
	.ss-fep-row { display: flex; align-items: center; gap: 12px; min-height: 24px; padding: 2px 6px; border-radius: 6px; }
	.ss-fep-hot { background: color-mix(in srgb, var(--interactive-accent, #7c3aed) 9%, transparent); }
	.ss-fep-key { color: var(--text-muted, #888); font-size: 13px; min-width: 84px; text-align: start; flex-shrink: 0; }
	.ss-fep-vals { display: inline-flex; flex-wrap: wrap; gap: 4px; }
	.ss-fep-plain { color: var(--text-normal, #2e3338); font-size: 13px; }
	.ss-petag { display: inline-flex; align-items: center; height: var(--pe-tag-height, 20px); padding: 0 8px; border-radius: var(--pe-tag-radius, 10px); background: var(--pe-tag-bg, var(--background-modifier-border-focus, #555)); color: var(--pe-tag-text-color, #fff); font-size: 12px; font-weight: 700; white-space: nowrap; }
	.ss-petaxo { display: inline-flex; align-items: center; height: 20px; padding: 0 8px; border-radius: var(--pe-taxo-radius, 10px); background: var(--pe-taxo-bg, var(--background-modifier-border-focus, #555)); color: var(--pe-taxo-text-color, #fff); font-size: 12px; font-weight: 700; white-space: nowrap; border-inline-start: 3px solid var(--interactive-accent, #7c3aed); }
	/* MIG-088 §2 — Cognitive-colours legend preview: a swatch per state reading the shared draft var. */
	.ss-fcog { gap: 7px; align-items: stretch; min-width: 360px; }
	.ss-cog-row { display: flex; align-items: center; gap: 10px; min-height: 26px; }
	.ss-cog-bar { width: 34px; height: 16px; border-radius: 4px; flex-shrink: 0; }
	.ss-cog-lbl { color: var(--text-normal, #2e3338); font-size: 14px; font-weight: 600; min-width: 92px; text-align: start; }
	.ss-cog-note { color: var(--text-muted, #888); font-size: 11.5px; }
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
