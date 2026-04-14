/**
 * Obsidian Community Theme Importer
 *
 * Fetches the Obsidian community theme registry, downloads theme CSS,
 * and converts to ConstellationTheme format.
 */
import type { ConstellationTheme } from '$lib/libraries/store';
import { parseAllStyleSettings } from './styleSettings';

export interface ObsidianThemeEntry {
	name: string;
	author: string;
	repo: string;
	screenshot: string;
	modes: string[]; // ["dark", "light"]
}

/** A Style Settings option parsed from theme CSS */
export interface StyleSettingsOption {
	id: string;
	title: string;
	type: 'variable-color' | 'variable-number' | 'variable-select' | 'variable-text' | 'variable-number-slider';
	default?: string;
	format?: string;
}

/** Preview colors extracted from a theme without full download */
export interface ThemePreviewColors {
	background: string;
	surface: string;
	text: string;
	accent: string;
	border: string;
}

const REGISTRY_URL = 'https://raw.githubusercontent.com/obsidianmd/obsidian-releases/master/community-css-themes.json';

/** Fetch the Obsidian community themes registry */
export async function fetchObsidianThemeList(): Promise<ObsidianThemeEntry[]> {
	const resp = await fetch(REGISTRY_URL);
	if (!resp.ok) throw new Error(`Failed to fetch theme list: ${resp.status}`);
	return resp.json();
}

/** Get the screenshot URL for a theme */
export function getScreenshotUrl(entry: ObsidianThemeEntry): string {
	return `https://raw.githubusercontent.com/${entry.repo}/HEAD/${entry.screenshot}`;
}

/** Download a theme's CSS from its GitHub repo */
export async function downloadThemeCSS(repo: string): Promise<string> {
	// Obsidian themes store their CSS in theme.css at repo root
	const url = `https://raw.githubusercontent.com/${repo}/HEAD/theme.css`;
	const resp = await fetch(url);
	if (!resp.ok) throw new Error(`Failed to download theme CSS: ${resp.status}`);
	return resp.text();
}

/**
 * Parse Obsidian theme CSS and extract color variables.
 * Maps to ConstellationTheme's 5 core colors.
 */
export function parseObsidianCSS(css: string, name: string, author: string, modes: string[]): ConstellationTheme[] {
	const themes: ConstellationTheme[] = [];

	// Parse Style Settings metadata (if present) — uses full parser
	const ssBlocks = parseAllStyleSettings(css);

	// Adapt CSS: extract only CSS variable declarations that Constellation uses.
	// Strip Obsidian-specific selectors (.workspace, .cm-s-obsidian, etc.)
	// Keep only :root, body, .theme-light, .theme-dark variable blocks.
	const adaptedCSS = adaptCSSForConstellation(css);

	// Extract variable blocks for light and dark themes
	const lightVars = extractVariables(css, '.theme-light');
	const darkVars = extractVariables(css, '.theme-dark');
	// Also check :root and body as fallbacks
	const rootVars = extractVariables(css, ':root');
	const bodyVars = extractVariables(css, 'body');

	const slug = name.toLowerCase().replace(/\s+/g, '-');
	const lightId = `obsidian-${slug}-light`;
	const darkId = `obsidian-${slug}-dark`;

	if (modes.includes('light') || Object.keys(lightVars).length > 0) {
		const vars = { ...rootVars, ...bodyVars, ...lightVars };
		themes.push({
			id: lightId,
			name: `${name} Light`,
			type: 'light',
			pairedThemeId: darkId, // auto-switch counterpart
			author,
			source: 'obsidian',
			colors: mapToColors(vars, 'light'),
			customCSS: adaptedCSS,
			styleSettingsBlocks: ssBlocks.length > 0 ? ssBlocks : undefined,
		});
	}

	if (modes.includes('dark') || Object.keys(darkVars).length > 0) {
		const vars = { ...rootVars, ...bodyVars, ...darkVars };
		themes.push({
			id: darkId,
			name: `${name} Dark`,
			type: 'dark',
			pairedThemeId: lightId, // auto-switch counterpart
			author,
			source: 'obsidian',
			colors: mapToColors(vars, 'dark'),
			customCSS: adaptedCSS,
			styleSettingsBlocks: ssBlocks.length > 0 ? ssBlocks : undefined,
		});
	}

	// If neither mode produced results, create one theme with raw CSS
	if (themes.length === 0) {
		const vars = { ...rootVars, ...bodyVars, ...lightVars, ...darkVars };
		const type = modes.includes('dark') ? 'dark' : 'light';
		themes.push({
			id: `obsidian-${slug}`,
			name,
			type,
			author,
			source: 'obsidian',
			colors: mapToColors(vars, type),
			customCSS: adaptedCSS,
			styleSettingsBlocks: ssBlocks.length > 0 ? ssBlocks : undefined,
		});
	}

	return themes;
}

/**
 * Extract CSS custom properties from a specific selector block.
 * Handles nested blocks, multiple occurrences of the same selector.
 */
/**
 * Brace-aware block extraction. Finds `selector { ... }` blocks where the
 * body may itself contain nested rules with their own `{}` pairs (the old
 * regex-based `[^}]+` approach truncated Minimal-style themes at the first
 * inner brace, losing most variable definitions).
 */
function extractSelectorBlocks(css: string, selector: string): string[] {
	const blocks: string[] = [];
	const escaped = selector.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');
	// Find the selector followed by `{` at start of declaration (not inside another rule)
	const headerRegex = new RegExp(escaped + '\\s*\\{', 'g');
	let m: RegExpExecArray | null;
	while ((m = headerRegex.exec(css)) !== null) {
		let i = m.index + m[0].length;
		let depth = 1;
		const start = i;
		while (i < css.length && depth > 0) {
			const ch = css[i];
			if (ch === '{') depth++;
			else if (ch === '}') depth--;
			i++;
		}
		if (depth === 0) blocks.push(css.slice(start, i - 1));
	}
	return blocks;
}

function extractVariables(css: string, selector: string): Record<string, string> {
	const vars: Record<string, string> = {};
	for (const block of extractSelectorBlocks(css, selector)) {
		// Walk the top-level declarations only: skip nested rules `{...}`
		let depth = 0;
		let declStart = 0;
		let i = 0;
		const topLevelPieces: string[] = [];
		while (i < block.length) {
			const ch = block[i];
			if (ch === '{') {
				if (depth === 0) {
					// Strip the nested rule's body so we don't collect declarations inside it
					topLevelPieces.push(block.slice(declStart, i).replace(/[^;]+$/, ''));
				}
				depth++;
			} else if (ch === '}') {
				depth--;
				if (depth === 0) declStart = i + 1;
			}
			i++;
		}
		if (depth === 0) topLevelPieces.push(block.slice(declStart));
		const decls = topLevelPieces.join(';');
		// Extract --variable: value pairs (value may contain parentheses & commas)
		const varRegex = /(--[\w-]+)\s*:\s*([^;]+?)\s*(?:;|$)/g;
		let varMatch: RegExpExecArray | null;
		while ((varMatch = varRegex.exec(decls)) !== null) {
			vars[varMatch[1].trim()] = varMatch[2].trim();
		}
	}
	return vars;
}

/**
 * Map extracted CSS variables to our 5 core colors.
 * Falls back to sensible defaults if variables are missing.
 */
function mapToColors(vars: Record<string, string>, type: 'light' | 'dark'): ConstellationTheme['colors'] {
	const defaults = type === 'light'
		? { background: '#ffffff', surface: '#f8fafc', text: '#1f2328', accent: '#7c3aed', border: '#e5e7eb' }
		: { background: '#1e1e2e', surface: '#2a2a3e', text: '#cdd6f4', accent: '#b4befe', border: '#45475a' };

	// Try multiple variable names for each color (Obsidian themes vary in naming)
	const background = resolveColor(vars,
		['--background-primary', '--bg', '--bg1', '--base00'],
		defaults.background
	);
	const surface = resolveColor(vars,
		['--background-secondary', '--bg-secondary', '--bg2', '--base01'],
		defaults.surface
	);
	const text = resolveColor(vars,
		['--text-normal', '--text', '--text-color', '--fg', '--base05'],
		defaults.text
	);
	const accent = resolveColor(vars,
		['--interactive-accent', '--accent', '--accent-color', '--color-accent', '--base0D'],
		defaults.accent
	);
	const border = resolveColor(vars,
		['--background-modifier-border', '--border', '--border-color', '--base02'],
		defaults.border
	);

	return { background, surface, text, accent, border };
}

/**
 * Resolve a scalar var() chain. Follows up to 8 hops.
 * Returns the final literal value (number / string / hex / rgb()).
 */
function resolveVarChain(vars: Record<string, string>, expr: string, depth = 0): string {
	if (depth > 8) return expr;
	const trimmed = expr.trim();
	const m = trimmed.match(/^var\(\s*(--[\w-]+)(?:\s*,\s*([^)]+))?\s*\)$/);
	if (!m) return trimmed;
	const name = m[1];
	const fallback = m[2]?.trim();
	if (vars[name] !== undefined) return resolveVarChain(vars, vars[name], depth + 1);
	if (fallback !== undefined) return resolveVarChain(vars, fallback, depth + 1);
	return trimmed;
}

/**
 * Fully resolve a value that may contain inner var() references
 * (e.g. "hsl(var(--base-h), var(--base-s), var(--base-l))").
 * Replaces each var() occurrence with its resolved literal.
 */
function resolveValue(vars: Record<string, string>, val: string, depth = 0): string {
	if (depth > 8) return val;
	let out = val;
	let changed = true;
	let hops = 0;
	while (changed && hops++ < 8) {
		changed = false;
		out = out.replace(/var\(\s*(--[\w-]+)(?:\s*,\s*([^)]*))?\s*\)/g, (_, name, fb) => {
			if (vars[name] !== undefined) { changed = true; return vars[name]; }
			if (fb !== undefined && fb.trim() !== '') { changed = true; return fb; }
			return '#000000'; // unresolvable → safe default
		});
	}
	return out.trim();
}

/**
 * Resolve a color from CSS variables, trying multiple names.
 * Handles var() references (including chained and HSL-split patterns),
 * hsl(), rgb(), and hex values.
 */
function resolveColor(vars: Record<string, string>, candidates: string[], fallback: string): string {
	for (const name of candidates) {
		const raw = vars[name];
		if (!raw) continue;

		// Expand any var() references (handles HSL-split themes like Minimal)
		const val = raw.includes('var(') ? resolveValue(vars, raw) : raw;
		if (!val) continue;

		// Handle hex
		if (val.startsWith('#')) {
			if (val.length === 4) { // #RGB → #RRGGBB
				return `#${val[1]}${val[1]}${val[2]}${val[2]}${val[3]}${val[3]}`;
			}
			if (val.length === 5) { // #RGBA → #RRGGBB (drop alpha)
				return `#${val[1]}${val[1]}${val[2]}${val[2]}${val[3]}${val[3]}`;
			}
			if (val.length === 7 || val.length === 9) return val.slice(0, 7);
			continue; // malformed hex, try next candidate
		}

		// Handle rgb/rgba
		if (val.startsWith('rgb')) {
			const nums = val.match(/[\d.]+/g);
			if (nums && nums.length >= 3) {
				const r = Math.round(parseFloat(nums[0]));
				const g = Math.round(parseFloat(nums[1]));
				const b = Math.round(parseFloat(nums[2]));
				return `#${r.toString(16).padStart(2, '0')}${g.toString(16).padStart(2, '0')}${b.toString(16).padStart(2, '0')}`;
			}
		}

		// Handle hsl/hsla — robust to commas, spaces, slashes, percent signs
		if (val.startsWith('hsl')) {
			const nums = val.match(/[\d.]+/g);
			if (nums && nums.length >= 3) {
				return hslToHex(parseFloat(nums[0]), parseFloat(nums[1]), parseFloat(nums[2]));
			}
		}

		// Handle plain numbers (might be part of a color)
		if (/^\d/.test(val) && val.includes(',')) {
			const nums = val.split(',').map(n => parseFloat(n.trim()));
			if (nums.length >= 3 && nums.every(n => !isNaN(n))) {
				if (nums[0] <= 360 && nums[1] <= 100 && nums[2] <= 100) {
					return hslToHex(nums[0], nums[1], nums[2]);
				}
				if (nums.every(n => n <= 255)) {
					return `#${nums.slice(0, 3).map(n => Math.round(n).toString(16).padStart(2, '0')).join('')}`;
				}
			}
		}
	}
	return fallback;
}

/**
 * Adapt Obsidian CSS for Constellation.
 *
 * Three-stage adaptation:
 * 1. Preserve @import and @font-face declarations (fonts the theme needs)
 * 2. Extract ALL CSS variables from compatible selectors (not just known ones)
 * 3. Map Obsidian component selectors to Constellation equivalents via class shim
 */
function adaptCSSForConstellation(css: string): string {
	const lines: string[] = [];

	// ── Stage 1: Preserve @import and @font-face ──
	// These are essential for themes that bundle custom fonts
	const importRegex = /@import\s+(?:url\()?[^;]+;/g;
	let importMatch;
	while ((importMatch = importRegex.exec(css)) !== null) {
		lines.push(importMatch[0]);
	}
	const fontFaceRegex = /@font-face\s*\{[^}]+\}/g;
	let fontMatch;
	while ((fontMatch = fontFaceRegex.exec(css)) !== null) {
		lines.push(fontMatch[0]);
	}

	// ── Stage 2: Extract ALL CSS variables from compatible selectors ──
	// Keep every --variable, not just known prefixes. Unknown vars do no harm
	// and may be referenced by the theme's own component styles.
	const supportedSelectors = [':root', 'body', '.theme-light', '.theme-dark'];

	for (const selector of supportedSelectors) {
		const vars = extractVariables(css, selector);
		if (Object.keys(vars).length === 0) continue;

		lines.push(`${selector} {`);
		for (const [key, val] of Object.entries(vars)) {
			lines.push(`  ${key}: ${val};`);
		}
		lines.push('}');
	}

	// ── Stage 3: Extract CodeMirror syntax highlighting ──
	// Map Obsidian's .cm-* classes to our --code-* variables
	const cmMappings = extractCodeMirrorColors(css);
	if (Object.keys(cmMappings).length > 0) {
		lines.push(':root {');
		for (const [key, val] of Object.entries(cmMappings)) {
			lines.push(`  ${key}: ${val};`);
		}
		lines.push('}');
	}

	// ── Stage 4: CSS Class Shim ──
	// Map Obsidian's component selectors to Constellation equivalents
	lines.push(generateClassShim(css));

	return lines.join('\n');
}

/**
 * Extract CodeMirror syntax colors from Obsidian theme CSS.
 * Maps .cm-keyword { color: X } → --code-keyword: X
 */
function extractCodeMirrorColors(css: string): Record<string, string> {
	const mapping: Record<string, string> = {};
	const cmClasses: Record<string, string> = {
		'cm-keyword': '--code-keyword',
		'cm-string': '--code-string',
		'cm-number': '--code-number',
		'cm-comment': '--code-comment',
		'cm-def': '--code-function',
		'cm-builtin': '--code-builtin',
		'cm-type': '--code-type',
		'cm-tag': '--code-tag',
		'cm-attribute': '--code-attr',
		'cm-variable': '--code-variable',
		'cm-meta': '--code-meta',
		'cm-operator': '--code-keyword',
		'cm-property': '--code-attr',
		'cm-qualifier': '--code-type',
		'cm-atom': '--code-number',
	};

	for (const [cmClass, cssVar] of Object.entries(cmClasses)) {
		// Match: .cm-keyword { color: #xxx } or .cm-s-obsidian .cm-keyword { color: #xxx }
		const regex = new RegExp(`\\.(?:cm-s-obsidian\\s+)?\\.${cmClass}\\s*\\{[^}]*color:\\s*([^;]+);`, 'g');
		const match = regex.exec(css);
		if (match) {
			const color = match[1].trim();
			if (!color.includes('var(')) { // skip variable references
				mapping[cssVar] = color;
			}
		}
	}

	return mapping;
}

/**
 * Generate CSS class shim that maps Obsidian selectors to Constellation.
 * This allows Obsidian component-level styles to partially work.
 */
function generateClassShim(css: string): string {
	// Obsidian → Constellation class mappings
	const SELECTOR_MAP: Record<string, string> = {
		// Layout
		'.workspace': '.app-root',
		'.workspace-leaf': '.pane',
		'.workspace-leaf-content': '.pane',
		'.workspace-tabs': '.tabs-row',
		'.workspace-tab-header': '.tab',
		// Sidebar
		'.nav-folder': '.library-section',
		'.nav-folder-title': '.tree-folder',
		'.nav-file': '.tree-file',
		'.nav-file-title': '.tree-file',
		'.nav-folder-collapse-indicator': '.v-chev',
		// Editor
		'.markdown-source-view': '.pane',
		'.markdown-preview-view': '.pane',
		'.markdown-rendered': '.pane',
		'.cm-s-obsidian': '.cm-editor',
		'.cm-content': '.cm-content',
		// UI
		'.view-header': '.sight2-header',
		'.view-header-title': '.sight2-title',
		'.status-bar': '.statusbar',
		'.modal': '.settings-overlay',
		'.setting-item': '.setting-item',
		'.search-input-container': '.search-input',
		// Tags
		'.tag': '.s-tag',
		// Callouts
		'.callout': '.callout',
		'.callout-title': '.callout-title',
	};

	const shimLines: string[] = [];

	// For each mapping, check if the Obsidian selector has styles in the CSS
	// and create an alias rule
	for (const [obsSelector, constSelector] of Object.entries(SELECTOR_MAP)) {
		const escaped = obsSelector.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');
		const regex = new RegExp(`${escaped}\\s*\\{([^}]+)\\}`, 'g');
		let match;
		while ((match = regex.exec(css)) !== null) {
			const block = match[1].trim();
			// Only include style declarations (not nested selectors)
			const declarations = block.split(';')
				.map(d => d.trim())
				.filter(d => d && d.includes(':') && !d.startsWith('//') && !d.startsWith('/*'));

			if (declarations.length > 0) {
				shimLines.push(`${constSelector} {`);
				for (const decl of declarations) {
					shimLines.push(`  ${decl};`);
				}
				shimLines.push('}');
			}
		}
	}

	return shimLines.join('\n');
}

function hslToHex(h: number, s: number, l: number): string {
	s /= 100;
	l /= 100;
	const a = s * Math.min(l, 1 - l);
	const f = (n: number) => {
		const k = (n + h / 30) % 12;
		const color = l - a * Math.max(Math.min(k - 3, 9 - k, 1), -1);
		return Math.round(255 * color).toString(16).padStart(2, '0');
	};
	return `#${f(0)}${f(8)}${f(4)}`;
}

// ─── #6: Theme Preview ─────────────────────────────────────

/**
 * Extract preview colors from theme CSS without creating a full theme.
 * Used for showing a mini preview before the user commits to importing.
 */
export function extractPreviewColors(css: string, type: 'light' | 'dark'): ThemePreviewColors {
	const selector = type === 'dark' ? '.theme-dark' : '.theme-light';
	const vars = {
		...extractVariables(css, ':root'),
		...extractVariables(css, 'body'),
		...extractVariables(css, selector),
	};
	return mapToColors(vars, type);
}

// ─── #7: Style Settings Metadata Parser ────────────────────

/**
 * Parse the Style Settings plugin metadata from theme CSS.
 * Obsidian themes like Minimal, AnuPpuccin, Things, etc. embed a
 * /* @settings block that defines user-customizable options.
 *
 * Format:
 * /* @settings
 * name: Theme Name
 * id: theme-id
 * settings:
 *   - id: color-bg
 *     title: Background color
 *     type: variable-color
 *     default: '#1e1e2e'
 * * /
 */
export function parseStyleSettings(css: string): StyleSettingsOption[] {
	const options: StyleSettingsOption[] = [];

	// Find the @settings block in CSS comments
	const settingsRegex = /\/\*\s*@settings\s*\n([\s\S]*?)\*\//g;
	const match = settingsRegex.exec(css);
	if (!match) return options;

	const yaml = match[1];

	// Find the settings: array section
	const settingsStart = yaml.indexOf('settings:');
	if (settingsStart === -1) return options;

	const settingsBlock = yaml.slice(settingsStart + 'settings:'.length);

	// Parse each setting entry (simplified YAML parser for the known format)
	const entries = settingsBlock.split(/\n\s*-\s+/).filter(e => e.trim());

	for (const entry of entries) {
		const lines = entry.split('\n').map(l => l.trim()).filter(l => l);
		const obj: Record<string, string> = {};

		for (const line of lines) {
			const colonIdx = line.indexOf(':');
			if (colonIdx === -1) continue;
			const key = line.slice(0, colonIdx).trim();
			const val = line.slice(colonIdx + 1).trim().replace(/^['"]|['"]$/g, '');
			obj[key] = val;
		}

		if (obj.id && obj.title && obj.type) {
			options.push({
				id: obj.id,
				title: obj.title,
				type: obj.type as StyleSettingsOption['type'],
				default: obj.default,
				format: obj.format,
			});
		}
	}

	return options;
}
