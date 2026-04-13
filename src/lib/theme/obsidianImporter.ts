/**
 * Obsidian Community Theme Importer
 *
 * Fetches the Obsidian community theme registry, downloads theme CSS,
 * and converts to ConstellationTheme format.
 */
import type { ConstellationTheme } from '$lib/libraries/store';

export interface ObsidianThemeEntry {
	name: string;
	author: string;
	repo: string;
	screenshot: string;
	modes: string[]; // ["dark", "light"]
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
		});
	}

	return themes;
}

/**
 * Extract CSS custom properties from a specific selector block.
 * Handles nested blocks, multiple occurrences of the same selector.
 */
function extractVariables(css: string, selector: string): Record<string, string> {
	const vars: Record<string, string> = {};
	// Match all blocks for this selector
	const escaped = selector.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');
	const regex = new RegExp(escaped + '\\s*\\{([^}]+)\\}', 'g');
	let match;
	while ((match = regex.exec(css)) !== null) {
		const block = match[1];
		// Extract --variable: value pairs
		const varRegex = /(--[\w-]+)\s*:\s*([^;]+);/g;
		let varMatch;
		while ((varMatch = varRegex.exec(block)) !== null) {
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
 * Resolve a color from CSS variables, trying multiple names.
 * Handles var() references, hsl(), rgb(), and hex values.
 */
function resolveColor(vars: Record<string, string>, candidates: string[], fallback: string): string {
	for (const name of candidates) {
		const val = vars[name];
		if (!val) continue;

		// Skip values that reference other variables (can't resolve without full context)
		if (val.includes('var(')) continue;

		// Handle hex
		if (val.startsWith('#')) return val.length === 4
			? `#${val[1]}${val[1]}${val[2]}${val[2]}${val[3]}${val[3]}` // expand shorthand
			: val;

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

		// Handle hsl/hsla
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
