/**
 * Constellation Core Style Settings.
 *
 * A native, theme-agnostic set of Style Settings blocks that expose every
 * key styling surface of Constellation (colors, typography, layout, UI
 * components, editor). These blocks are rendered in the Style Settings tab
 * on top of any theme-specific blocks that an imported Obsidian theme may
 * bring. User values are stored on the active theme's styleSettingsValues.
 *
 * The `id` of each setting corresponds to the CSS custom property name
 * without the leading `--`. `generateStyleSettingsCSS` emits `--{id}: value;`
 * which overrides whatever was produced by `deriveThemeVariables`.
 */

import type { StyleSettingsBlock } from './styleSettings';

export const CONSTELLATION_CORE_BLOCKS: StyleSettingsBlock[] = [
	{
		name: 'Constellation — Colors',
		id: 'constellation-colors',
		settings: [
			{ id: 'ss-colors-bg', type: 'heading', title: 'Background & Surfaces', level: 2 },
			{ id: 'background-primary', type: 'variable-color', title: 'Background (primary)', description: 'Main editor and pane background', format: 'hex', default: '' },
			{ id: 'background-primary-alt', type: 'variable-color', title: 'Background (alt)', description: 'Alternate background (stripes, subtle contrasts)', format: 'hex', default: '' },
			{ id: 'background-secondary', type: 'variable-color', title: 'Surface (secondary)', description: 'Sidebar, panels, modal background', format: 'hex', default: '' },
			{ id: 'background-secondary-alt', type: 'variable-color', title: 'Surface (alt)', format: 'hex', default: '' },
			{ id: 'background-modifier-hover', type: 'variable-color', title: 'Hover background', format: 'hex', default: '' },
			{ id: 'background-modifier-border', type: 'variable-color', title: 'Border', description: 'Dividers and outlines', format: 'hex', default: '' },
			{ id: 'background-modifier-form-field', type: 'variable-color', title: 'Input background', format: 'hex', default: '' },

			{ id: 'ss-colors-text', type: 'heading', title: 'Text', level: 2 },
			{ id: 'text-normal', type: 'variable-color', title: 'Text (normal)', format: 'hex', default: '' },
			{ id: 'text-muted', type: 'variable-color', title: 'Text (muted)', description: 'Secondary text, captions', format: 'hex', default: '' },
			{ id: 'text-faint', type: 'variable-color', title: 'Text (faint)', description: 'Tertiary, placeholders', format: 'hex', default: '' },
			{ id: 'text-on-accent', type: 'variable-color', title: 'Text on accent', format: 'hex', default: '' },
			{ id: 'text-error', type: 'variable-color', title: 'Error', format: 'hex', default: '' },
			{ id: 'text-warning', type: 'variable-color', title: 'Warning', format: 'hex', default: '' },
			{ id: 'text-success', type: 'variable-color', title: 'Success', format: 'hex', default: '' },

			{ id: 'ss-colors-accent', type: 'heading', title: 'Accent', level: 2 },
			{ id: 'interactive-accent', type: 'variable-color', title: 'Accent', description: 'Primary interactive color (buttons, links, active states)', format: 'hex', default: '' },
			{ id: 'interactive-accent-hover', type: 'variable-color', title: 'Accent (hover)', format: 'hex', default: '' },
			{ id: 'text-accent', type: 'variable-color', title: 'Accent text', format: 'hex', default: '' },
		],
	},
	{
		name: 'Constellation — Typography',
		id: 'constellation-typography',
		settings: [
			{ id: 'ss-type-size', type: 'heading', title: 'Font sizes', level: 2 },
			{ id: 'font-interface-size', type: 'variable-number-slider', title: 'Interface font size', description: 'Sidebar, toolbars, menus', min: 11, max: 20, step: 1, default: '14', format: 'px' },
			{ id: 'font-text-size', type: 'variable-number-slider', title: 'Note font size', description: 'Body text in the editor', min: 12, max: 24, step: 1, default: '16', format: 'px' },
			{ id: 'font-monospace-size', type: 'variable-number-slider', title: 'Code font size', min: 10, max: 20, step: 1, default: '14', format: 'px' },

			{ id: 'ss-type-head', type: 'heading', title: 'Headings', level: 2 },
			{ id: 'h1-size', type: 'variable-number-slider', title: 'H1 size', min: 18, max: 48, step: 1, default: '32', format: 'px' },
			{ id: 'h2-size', type: 'variable-number-slider', title: 'H2 size', min: 16, max: 40, step: 1, default: '26', format: 'px' },
			{ id: 'h3-size', type: 'variable-number-slider', title: 'H3 size', min: 14, max: 32, step: 1, default: '22', format: 'px' },
			{ id: 'h4-size', type: 'variable-number-slider', title: 'H4 size', min: 14, max: 28, step: 1, default: '18', format: 'px' },
			{ id: 'h5-size', type: 'variable-number-slider', title: 'H5 size', min: 12, max: 24, step: 1, default: '16', format: 'px' },
			{ id: 'h6-size', type: 'variable-number-slider', title: 'H6 size', min: 11, max: 22, step: 1, default: '14', format: 'px' },
			{ id: 'heading-weight', type: 'variable-number-slider', title: 'Heading weight', min: 300, max: 900, step: 100, default: '700' },

			{ id: 'ss-type-rhythm', type: 'heading', title: 'Rhythm', level: 2 },
			{ id: 'line-height-normal', type: 'variable-number-slider', title: 'Line height', min: 1.1, max: 2.2, step: 0.05, default: '1.6' },
			{ id: 'line-height-tight', type: 'variable-number-slider', title: 'Tight line height', description: 'For headings and dense UI', min: 1.0, max: 1.8, step: 0.05, default: '1.3' },
			{ id: 'paragraph-spacing', type: 'variable-number-slider', title: 'Paragraph spacing', min: 0, max: 32, step: 1, default: '12', format: 'px' },
		],
	},
	{
		name: 'Constellation — Layout & Shape',
		id: 'constellation-layout',
		settings: [
			{ id: 'ss-layout-radius', type: 'heading', title: 'Corners', level: 2 },
			{ id: 'radius-s', type: 'variable-number-slider', title: 'Small radius', description: 'Buttons, tags, inputs', min: 0, max: 20, step: 1, default: '4', format: 'px' },
			{ id: 'radius-m', type: 'variable-number-slider', title: 'Medium radius', description: 'Cards, panels', min: 0, max: 24, step: 1, default: '8', format: 'px' },
			{ id: 'radius-l', type: 'variable-number-slider', title: 'Large radius', description: 'Modals, popovers', min: 0, max: 32, step: 1, default: '12', format: 'px' },

			{ id: 'ss-layout-borders', type: 'heading', title: 'Borders & shadows', level: 2 },
			{ id: 'border-width', type: 'variable-number-slider', title: 'Border width', min: 0, max: 4, step: 1, default: '1', format: 'px' },
			{ id: 'shadow-s', type: 'variable-text', title: 'Shadow (small)', description: 'CSS box-shadow value', default: '0 1px 2px rgba(0,0,0,0.1)' },
			{ id: 'shadow-l', type: 'variable-text', title: 'Shadow (large)', description: 'CSS box-shadow value', default: '0 4px 16px rgba(0,0,0,0.12)' },

			{ id: 'ss-layout-editor', type: 'heading', title: 'Editor width', level: 2 },
			{ id: 'file-line-width', type: 'variable-number-slider', title: 'Readable line length', description: 'Max width of the note content', min: 40, max: 120, step: 1, default: '70', format: 'ch' },
			{ id: 'file-margins', type: 'variable-number-slider', title: 'Note side margins', min: 0, max: 80, step: 1, default: '24', format: 'px' },
		],
	},
	{
		name: 'Constellation — Components',
		id: 'constellation-components',
		settings: [
			{ id: 'ss-comp-side', type: 'heading', title: 'Sidebar', level: 2 },
			{ id: 'sidebar-width', type: 'variable-number-slider', title: 'Sidebar width', min: 180, max: 420, step: 2, default: '260', format: 'px' },
			{ id: 'sidebar-bg', type: 'variable-color', title: 'Sidebar background', format: 'hex', default: '' },

			{ id: 'ss-comp-dock', type: 'heading', title: 'Ribbon dock (left icons)', level: 2 },
			{ id: 'dock-width', type: 'variable-number-slider', title: 'Dock width', min: 32, max: 72, step: 1, default: '40', format: 'px' },
			{ id: 'dock-btn-size', type: 'variable-number-slider', title: 'Dock button size', min: 24, max: 56, step: 1, default: '32', format: 'px' },
			{ id: 'dock-icon-size', type: 'variable-number-slider', title: 'Dock icon size', min: 12, max: 32, step: 1, default: '18', format: 'px' },
			{ id: 'dock-btn-radius', type: 'variable-number-slider', title: 'Dock button radius', min: 0, max: 16, step: 1, default: '4', format: 'px' },
			{ id: 'dock-btn-color', type: 'variable-color', title: 'Dock icon color', format: 'hex', default: '' },
			{ id: 'dock-bg', type: 'variable-color', title: 'Dock background', format: 'hex', default: '' },

			{ id: 'ss-comp-sbt', type: 'heading', title: 'Sidebar action toolbar', description: 'Row of buttons at the top of the sidebar (new note, table, folder, capture).', level: 2 },
			{ id: 'sidebar-toolbar-height', type: 'variable-number-slider', title: 'Toolbar min height', min: 26, max: 60, step: 1, default: '34', format: 'px' },
			{ id: 'sidebar-toolbar-bg', type: 'variable-color', title: 'Toolbar background', format: 'hex', default: '' },
			{ id: 'sidebar-btn-size', type: 'variable-number-slider', title: 'Button size', min: 20, max: 40, step: 1, default: '26', format: 'px' },
			{ id: 'sidebar-icon-size', type: 'variable-number-slider', title: 'Icon size', min: 10, max: 28, step: 1, default: '16', format: 'px' },
			{ id: 'sidebar-btn-radius', type: 'variable-number-slider', title: 'Button radius', min: 0, max: 14, step: 1, default: '3', format: 'px' },
			{ id: 'sidebar-btn-color', type: 'variable-color', title: 'Button icon color', format: 'hex', default: '' },

			{ id: 'ss-comp-layout', type: 'heading', title: 'Layout bar (pane toggles)', description: 'Left sidebar, split, and right sidebar toggle buttons above the note area.', level: 2 },
			{ id: 'layout-bar-bg', type: 'variable-color', title: 'Layout bar background', format: 'hex', default: '' },
			{ id: 'layout-bar-height', type: 'variable-number-slider', title: 'Layout bar height', min: 26, max: 60, step: 1, default: '34', format: 'px' },
			{ id: 'layout-btn-size', type: 'variable-number-slider', title: 'Toggle button size', min: 20, max: 44, step: 1, default: '28', format: 'px' },
			{ id: 'layout-icon-size', type: 'variable-number-slider', title: 'Toggle icon size', min: 10, max: 28, step: 1, default: '14', format: 'px' },
			{ id: 'layout-btn-radius', type: 'variable-number-slider', title: 'Toggle button radius', min: 0, max: 14, step: 1, default: '4', format: 'px' },
			{ id: 'layout-btn-color', type: 'variable-color', title: 'Toggle icon color', format: 'hex', default: '' },
			{ id: 'layout-btn-active-color', type: 'variable-color', title: 'Toggle icon color (active)', format: 'hex', default: '' },

			{ id: 'ss-comp-top', type: 'heading', title: 'Top bar / Tab strip', level: 2 },
			{ id: 'topbar-height', type: 'variable-number-slider', title: 'Top bar min height', description: 'Leave small to match tab size; raise to add padding above tabs', min: 28, max: 80, step: 1, default: '38', format: 'px' },
			{ id: 'topbar-bg', type: 'variable-color', title: 'Top bar background', format: 'hex', default: '' },
			{ id: 'tab-font-size', type: 'variable-number-slider', title: 'Tab font size', min: 10, max: 20, step: 1, default: '13', format: 'px' },
			{ id: 'tab-height', type: 'variable-number-slider', title: 'Tab height', min: 22, max: 48, step: 1, default: '26', format: 'px' },
			{ id: 'tab-bg', type: 'variable-color', title: 'Tab background (inactive)', format: 'hex', default: '' },
			{ id: 'tab-color', type: 'variable-color', title: 'Tab text (inactive)', format: 'hex', default: '' },
			{ id: 'tab-active-bg', type: 'variable-color', title: 'Tab background (active)', format: 'hex', default: '' },
			{ id: 'tab-active-color', type: 'variable-color', title: 'Tab text (active)', format: 'hex', default: '' },
			{ id: 'tab-border', type: 'variable-color', title: 'Tab border', format: 'hex', default: '' },

			{ id: 'ss-comp-ft', type: 'heading', title: 'File explorer (left sidebar)', description: 'Universe notes, child universes, libraries, folders, and notes in the left sidebar.', level: 2 },
			{ id: 'ft-universe-font-size', type: 'variable-number-slider', title: 'Universe notes size', description: 'Root Universe notes entry at the top of the sidebar', min: 10, max: 22, step: 1, default: '13', format: 'px' },
			{ id: 'ft-universe-weight', type: 'variable-number-slider', title: 'Universe notes weight', min: 300, max: 900, step: 100, default: '600' },
			{ id: 'ft-universe-color', type: 'variable-color', title: 'Universe notes color', format: 'hex', default: '' },
			{ id: 'ft-cuniverse-font-size', type: 'variable-number-slider', title: 'Child universe size', description: 'Linked child universes shown in the sidebar', min: 10, max: 22, step: 1, default: '13', format: 'px' },
			{ id: 'ft-cuniverse-weight', type: 'variable-number-slider', title: 'Child universe weight', min: 300, max: 900, step: 100, default: '600' },
			{ id: 'ft-cuniverse-color', type: 'variable-color', title: 'Child universe color', format: 'hex', default: '' },
			{ id: 'ft-library-font-size', type: 'variable-number-slider', title: 'Library name size', min: 10, max: 22, step: 1, default: '13', format: 'px' },
			{ id: 'ft-library-weight', type: 'variable-number-slider', title: 'Library name weight', min: 300, max: 900, step: 100, default: '600' },
			{ id: 'ft-library-color', type: 'variable-color', title: 'Library name color', format: 'hex', default: '' },
			{ id: 'ft-font-size', type: 'variable-number-slider', title: 'Folder & note size', min: 10, max: 20, step: 1, default: '13', format: 'px' },
			{ id: 'ft-folder-weight', type: 'variable-number-slider', title: 'Folder weight', min: 300, max: 900, step: 100, default: '400' },
			{ id: 'ft-folder-color', type: 'variable-color', title: 'Folder color', format: 'hex', default: '' },
			{ id: 'ft-file-weight', type: 'variable-number-slider', title: 'Note weight', min: 300, max: 900, step: 100, default: '400' },
			{ id: 'ft-file-color', type: 'variable-color', title: 'Note color', format: 'hex', default: '' },
			{ id: 'ft-row-padding-y', type: 'variable-number-slider', title: 'Row spacing (vertical)', min: 0, max: 10, step: 1, default: '2', format: 'px' },

			{ id: 'ss-comp-rs', type: 'heading', title: 'Right sidebar (inspector)', description: 'Inspector pane on the right and its tab row (properties, backlinks, tags, tasks, calendar, etc.).', level: 2 },
			{ id: 'right-sidebar-bg', type: 'variable-color', title: 'Right sidebar background', format: 'hex', default: '' },
			{ id: 'rs-tabs-bg', type: 'variable-color', title: 'Tab row background', format: 'hex', default: '' },
			{ id: 'rs-tab-height', type: 'variable-number-slider', title: 'Tab row height', min: 24, max: 56, step: 1, default: '30', format: 'px' },
			{ id: 'rs-icon-size', type: 'variable-number-slider', title: 'Tab icon size', min: 10, max: 28, step: 1, default: '16', format: 'px' },
			{ id: 'rs-tab-color', type: 'variable-color', title: 'Tab icon color', format: 'hex', default: '' },
			{ id: 'rs-tab-active-color', type: 'variable-color', title: 'Tab icon color (active)', format: 'hex', default: '' },

			{ id: 'ss-comp-status', type: 'heading', title: 'Status bar', level: 2 },
			{ id: 'statusbar-height', type: 'variable-number-slider', title: 'Status bar height', min: 18, max: 48, step: 1, default: '24', format: 'px' },
			{ id: 'statusbar-font-size', type: 'variable-number-slider', title: 'Status bar font size', min: 9, max: 18, step: 1, default: '11', format: 'px' },
			{ id: 'statusbar-bg', type: 'variable-color', title: 'Status bar background', format: 'hex', default: '' },
			{ id: 'statusbar-color', type: 'variable-color', title: 'Status bar text', format: 'hex', default: '' },

			{ id: 'ss-comp-tabs', type: 'heading', title: 'Tab shape', level: 2 },
			{ id: 'tab-radius', type: 'variable-number-slider', title: 'Tab radius', min: 0, max: 16, step: 1, default: '6', format: 'px' },

			{ id: 'ss-comp-buttons', type: 'heading', title: 'Buttons', level: 2 },
			{ id: 'button-radius', type: 'variable-number-slider', title: 'Button radius', min: 0, max: 24, step: 1, default: '6', format: 'px' },
			{ id: 'button-padding-x', type: 'variable-number-slider', title: 'Button padding (horizontal)', min: 4, max: 32, step: 1, default: '12', format: 'px' },
			{ id: 'button-padding-y', type: 'variable-number-slider', title: 'Button padding (vertical)', min: 2, max: 20, step: 1, default: '6', format: 'px' },

			{ id: 'ss-comp-tags', type: 'heading', title: 'Tags & callouts', level: 2 },
			{ id: 'tag-radius', type: 'variable-number-slider', title: 'Tag radius', min: 0, max: 24, step: 1, default: '12', format: 'px' },
			{ id: 'tag-bg', type: 'variable-color', title: 'Tag background', format: 'hex', default: '' },
			{ id: 'tag-color', type: 'variable-color', title: 'Tag text', format: 'hex', default: '' },
			{ id: 'callout-radius', type: 'variable-number-slider', title: 'Callout radius', min: 0, max: 24, step: 1, default: '8', format: 'px' },
		],
	},
	{
		name: 'Constellation — Editor',
		id: 'constellation-editor',
		settings: [
			{ id: 'ss-ed-link', type: 'heading', title: 'Links', level: 2 },
			{ id: 'link-color', type: 'variable-color', title: 'Link color', format: 'hex', default: '' },
			{ id: 'link-color-hover', type: 'variable-color', title: 'Link color (hover)', format: 'hex', default: '' },
			{ id: 'link-decoration', type: 'variable-select', title: 'Link decoration', default: 'underline', options: [
				{ label: 'None', value: 'none' },
				{ label: 'Underline', value: 'underline' },
				{ label: 'Dotted', value: 'underline dotted' },
			]},

			{ id: 'ss-ed-code', type: 'heading', title: 'Code & blocks', level: 2 },
			{ id: 'code-background', type: 'variable-color', title: 'Inline code background', format: 'hex', default: '' },
			{ id: 'code-normal', type: 'variable-color', title: 'Code text', format: 'hex', default: '' },
			{ id: 'code-block-radius', type: 'variable-number-slider', title: 'Code block radius', min: 0, max: 20, step: 1, default: '6', format: 'px' },

			{ id: 'ss-ed-quote', type: 'heading', title: 'Blockquote', level: 2 },
			{ id: 'blockquote-border-color', type: 'variable-color', title: 'Blockquote bar', format: 'hex', default: '' },
			{ id: 'blockquote-border-width', type: 'variable-number-slider', title: 'Blockquote bar width', min: 1, max: 8, step: 1, default: '3', format: 'px' },

			{ id: 'ss-ed-cursor', type: 'heading', title: 'Cursor & selection', level: 2 },
			{ id: 'caret-color', type: 'variable-color', title: 'Cursor color', format: 'hex', default: '' },
			{ id: 'text-selection', type: 'variable-color', title: 'Selection background', format: 'hex', default: '' },
		],
	},
];

/** Precomputed set of core block ids — reuse instead of re-allocating. */
export const CORE_BLOCK_IDS: ReadonlySet<string> = new Set(
	CONSTELLATION_CORE_BLOCKS.map(b => b.id),
);

/** Strip core-block copies a previous bug may have stored on a theme. */
export function stripCoreBlocks(blocks: StyleSettingsBlock[] | undefined): StyleSettingsBlock[] {
	if (!blocks || blocks.length === 0) return [];
	return blocks.filter(b => !CORE_BLOCK_IDS.has(b.id));
}

/** Core blocks + theme's own blocks (deduplicated). Single source of truth. */
export function getEffectiveStyleBlocks(theme: { styleSettingsBlocks?: StyleSettingsBlock[] } | null | undefined): StyleSettingsBlock[] {
	return [...CONSTELLATION_CORE_BLOCKS, ...stripCoreBlocks(theme?.styleSettingsBlocks)];
}
