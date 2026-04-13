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

	if (modes.includes('light') || Object.keys(lightVars).length > 0) {
		const vars = { ...rootVars, ...bodyVars, ...lightVars };
		themes.push({
			id: `obsidian-${name.toLowerCase().replace(/\s+/g, '-')}-light`,
			name: `${name} Light`,
			type: 'light',
			colors: mapToColors(vars, 'light'),
			customCSS: adaptedCSS,
		});
	}

	if (modes.includes('dark') || Object.keys(darkVars).length > 0) {
		const vars = { ...rootVars, ...bodyVars, ...darkVars };
		themes.push({
			id: `obsidian-${name.toLowerCase().replace(/\s+/g, '-')}-dark`,
			name: `${name} Dark`,
			type: 'dark',
			colors: mapToColors(vars, 'dark'),
			customCSS: adaptedCSS,
		});
	}

	// If neither mode produced results, create one theme with raw CSS
	if (themes.length === 0) {
		const vars = { ...rootVars, ...bodyVars, ...lightVars, ...darkVars };
		const type = modes.includes('dark') ? 'dark' : 'light';
		themes.push({
			id: `obsidian-${name.toLowerCase().replace(/\s+/g, '-')}`,
			name,
			type,
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
 * Extracts only CSS variable declarations from compatible selectors.
 * Strips Obsidian-specific component selectors (.workspace, .nav-*, etc.)
 * that don't exist in Constellation.
 */
function adaptCSSForConstellation(css: string): string {
	const lines: string[] = [];
	// Extract variable blocks from selectors we support
	const supportedSelectors = [':root', 'body', '.theme-light', '.theme-dark'];

	for (const selector of supportedSelectors) {
		const vars = extractVariables(css, selector);
		if (Object.keys(vars).length === 0) continue;

		// Only include variables that Constellation uses (our CSS variable namespace)
		const constellationVarPrefixes = [
			'--background-', '--text-', '--interactive-', '--accent-',
			'--scrollbar-', '--shadow-', '--color-', '--font-',
			'--code-', '--star-', '--line-height',
		];

		const filtered: Record<string, string> = {};
		for (const [key, val] of Object.entries(vars)) {
			if (constellationVarPrefixes.some(p => key.startsWith(p))) {
				filtered[key] = val;
			}
		}

		if (Object.keys(filtered).length > 0) {
			lines.push(`${selector} {`);
			for (const [key, val] of Object.entries(filtered)) {
				lines.push(`  ${key}: ${val};`);
			}
			lines.push('}');
		}
	}

	return lines.join('\n');
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
