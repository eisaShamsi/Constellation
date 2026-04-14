/**
 * Constellation Style Settings Engine
 *
 * Full implementation of the Obsidian Style Settings spec.
 * Parses /* @settings * / blocks from CSS and provides types for rendering UI.
 *
 * Supports all 10 setting types:
 *   heading, info-text, class-toggle, class-select,
 *   variable-text, variable-number, variable-number-slider,
 *   variable-select, variable-color, variable-themed-color
 *
 * Plus: color-gradient, localization, format units, opacity.
 */

// ─── Types ────────────────────────────────────────────────

export type SettingType =
	| 'heading'
	| 'info-text'
	| 'class-toggle'
	| 'class-select'
	| 'variable-text'
	| 'variable-number'
	| 'variable-number-slider'
	| 'variable-select'
	| 'variable-color'
	| 'variable-themed-color'
	| 'color-gradient';

export interface StyleSettingOption {
	label: string;
	value: string;
}

export interface ColorAltFormat {
	id: string;
	format: string;
}

export interface StyleSetting {
	id: string;
	title: string;
	description?: string;
	type: SettingType;

	// Common
	default?: string;

	// heading
	level?: number;          // 1-6
	collapsed?: boolean;

	// info-text
	markdown?: boolean;

	// class-toggle
	addCommand?: boolean;

	// class-select
	allowEmpty?: boolean;
	options?: StyleSettingOption[];

	// variable-text
	quotes?: boolean;

	// variable-number / variable-number-slider
	format?: string;         // unit suffix: px, rem, %, em, etc.
	min?: number;
	max?: number;
	step?: number;

	// variable-select — uses options[]

	// variable-color
	opacity?: boolean;
	altFormat?: ColorAltFormat[];

	// variable-themed-color
	defaultLight?: string;
	defaultDark?: string;

	// color-gradient
	from?: string;
	to?: string;
	pad?: number;

	// Localization: title.ar, title.de, description.fr, etc.
	localizedTitles?: Record<string, string>;
	localizedDescriptions?: Record<string, string>;
}

export interface StyleSettingsBlock {
	name: string;
	id: string;
	settings: StyleSetting[];
}

// ─── Parser ───────────────────────────────────────────────

/**
 * Parse all /* @settings * / blocks from a CSS string.
 * Returns an array of setting blocks (a CSS file can have multiple).
 */
export function parseAllStyleSettings(css: string): StyleSettingsBlock[] {
	const blocks: StyleSettingsBlock[] = [];
	const regex = /\/\*\s*@settings\s*\n([\s\S]*?)\*\//g;
	let match;

	while ((match = regex.exec(css)) !== null) {
		try {
			const block = parseSettingsYAML(match[1]);
			if (block) blocks.push(block);
		} catch {
			// Skip malformed blocks
		}
	}

	return blocks;
}

/**
 * Parse a single @settings YAML block.
 */
function parseSettingsYAML(yaml: string): StyleSettingsBlock | null {
	const lines = yaml.split('\n');

	let name = '';
	let id = '';
	let inSettings = false;
	let settingsYAML = '';

	for (const line of lines) {
		const trimmed = line.trim();
		if (trimmed.startsWith('name:') && !inSettings) {
			name = extractValue(trimmed);
		} else if (trimmed.startsWith('id:') && !inSettings) {
			id = extractValue(trimmed);
		} else if (trimmed === 'settings:') {
			inSettings = true;
		} else if (inSettings) {
			settingsYAML += line + '\n';
		}
	}

	if (!name || !id) return null;

	const settings = parseSettingsArray(settingsYAML);
	return { name, id, settings };
}

/**
 * Parse the settings array from YAML.
 * Splits on top-level list items (lines starting with `    -` at the settings indent level).
 */
function parseSettingsArray(yaml: string): StyleSetting[] {
	const settings: StyleSetting[] = [];

	// Split into individual setting entries by detecting list item markers
	const entries = splitYAMLListItems(yaml);

	for (const entry of entries) {
		const setting = parseSettingEntry(entry);
		if (setting) settings.push(setting);
	}

	return settings;
}

/**
 * Split YAML text into individual list items.
 * Each item starts with a line matching /^\s{4}-\s/ or /^\s*-\s/.
 */
function splitYAMLListItems(yaml: string): string[] {
	const items: string[] = [];
	let current = '';
	const lines = yaml.split('\n');

	for (const line of lines) {
		// Detect list item start: line that starts with optional whitespace then `-`
		if (/^\s*-\s/.test(line) && !line.trim().startsWith('- label:') && !line.trim().startsWith('- ')) {
			// Special handling: if this is a top-level list marker
			if (current.trim()) items.push(current);
			// Remove the leading `-` and keep the rest
			current = line.replace(/^\s*-\s*/, '    ') + '\n';
		} else if (/^\s{4,}-\s/.test(line) && current === '') {
			// First item
			current = line.replace(/^\s*-\s*/, '    ') + '\n';
		} else {
			current += line + '\n';
		}
	}
	if (current.trim()) items.push(current);

	return items;
}

/**
 * Parse a single setting entry from its YAML lines.
 */
function parseSettingEntry(yaml: string): StyleSetting | null {
	const props = parseYAMLObject(yaml);
	if (!props.id || !props.type) return null;

	const setting: StyleSetting = {
		id: props.id,
		title: props.title || props.id,
		type: props.type as SettingType,
	};

	// Optional common fields
	if (props.description) setting.description = props.description;
	if (props.default !== undefined) setting.default = String(props.default).replace(/^['"]|['"]$/g, '');

	// heading
	if (props.level) setting.level = parseInt(props.level);
	if (props.collapsed === 'true' || props.collapsed === true) setting.collapsed = true;

	// info-text
	if (props.markdown === 'true' || props.markdown === true) setting.markdown = true;

	// class-toggle
	if (props.addCommand === 'true' || props.addCommand === true) setting.addCommand = true;

	// class-select / variable-select
	if (props.allowEmpty === 'true' || props.allowEmpty === true) setting.allowEmpty = true;
	if (props.options) setting.options = parseOptions(props.options);

	// variable-text
	if (props.quotes === 'true' || props.quotes === true) setting.quotes = true;

	// variable-number / variable-number-slider
	if (props.format) setting.format = props.format;
	if (props.min !== undefined) setting.min = parseFloat(props.min);
	if (props.max !== undefined) setting.max = parseFloat(props.max);
	if (props.step !== undefined) setting.step = parseFloat(props.step);

	// variable-color
	if (props.opacity === 'true' || props.opacity === true) setting.opacity = true;
	if (props['alt-format']) setting.altFormat = parseAltFormat(props['alt-format']);

	// variable-themed-color
	if (props['default-light']) setting.defaultLight = String(props['default-light']).replace(/^['"]|['"]$/g, '');
	if (props['default-dark']) setting.defaultDark = String(props['default-dark']).replace(/^['"]|['"]$/g, '');

	// color-gradient
	if (props.from) setting.from = props.from;
	if (props.to) setting.to = props.to;
	if (props.pad) setting.pad = parseInt(props.pad);

	// Localization: title.ar, title.de, description.fr, etc.
	const titleLocales: Record<string, string> = {};
	const descLocales: Record<string, string> = {};
	for (const [key, val] of Object.entries(props)) {
		const titleMatch = key.match(/^title\.(\w+(?:-\w+)?)$/);
		if (titleMatch) titleLocales[titleMatch[1]] = String(val);
		const descMatch = key.match(/^description\.(\w+(?:-\w+)?)$/);
		if (descMatch) descLocales[descMatch[1]] = String(val);
	}
	if (Object.keys(titleLocales).length > 0) setting.localizedTitles = titleLocales;
	if (Object.keys(descLocales).length > 0) setting.localizedDescriptions = descLocales;

	return setting;
}

/**
 * Parse a YAML-like object from indented key: value lines.
 * Handles nested arrays (options) as raw strings for further parsing.
 */
function parseYAMLObject(yaml: string): Record<string, any> {
	const result: Record<string, any> = {};
	const lines = yaml.split('\n');
	let currentKey = '';
	let nestedContent = '';
	let inNested = false;

	for (const line of lines) {
		const trimmed = line.trim();
		if (!trimmed) continue;

		if (inNested) {
			// Check if this line is back at the same or higher indent level
			const indent = line.search(/\S/);
			if (indent <= 8 && !trimmed.startsWith('-') && trimmed.includes(':') && !trimmed.startsWith('label:') && !trimmed.startsWith('value:') && !trimmed.startsWith('id:') && !trimmed.startsWith('format:')) {
				// End of nested content
				result[currentKey] = nestedContent;
				inNested = false;
			} else {
				nestedContent += line + '\n';
				continue;
			}
		}

		const colonIdx = trimmed.indexOf(':');
		if (colonIdx === -1) continue;

		const key = trimmed.slice(0, colonIdx).trim();
		const val = trimmed.slice(colonIdx + 1).trim();

		if (val === '' || val === '|') {
			// Start of nested content (array or multiline)
			currentKey = key;
			nestedContent = '';
			inNested = true;
		} else {
			result[key] = val;
		}
	}

	if (inNested && currentKey) {
		result[currentKey] = nestedContent;
	}

	return result;
}

/**
 * Parse options array from YAML.
 * Handles both simple strings and label/value objects.
 */
function parseOptions(raw: any): StyleSettingOption[] {
	if (typeof raw !== 'string') return [];
	const options: StyleSettingOption[] = [];
	const lines = raw.split('\n').map(l => l.trim()).filter(l => l);

	let currentLabel = '';
	let currentValue = '';

	for (const line of lines) {
		if (line.startsWith('- label:')) {
			if (currentValue) {
				options.push({ label: currentLabel || currentValue, value: currentValue });
				currentLabel = '';
				currentValue = '';
			}
			currentLabel = extractValue(line.replace('- ', ''));
		} else if (line.startsWith('label:')) {
			currentLabel = extractValue(line);
		} else if (line.startsWith('value:')) {
			currentValue = extractValue(line);
		} else if (line.startsWith('- ')) {
			if (currentValue) {
				options.push({ label: currentLabel || currentValue, value: currentValue });
				currentLabel = '';
				currentValue = '';
			}
			// Simple string option
			const val = line.slice(2).trim();
			options.push({ label: val, value: val });
		}
	}

	// Flush last
	if (currentValue) {
		options.push({ label: currentLabel || currentValue, value: currentValue });
	}

	return options;
}

/**
 * Parse alt-format array for variable-color.
 */
function parseAltFormat(raw: any): ColorAltFormat[] {
	if (typeof raw !== 'string') return [];
	const formats: ColorAltFormat[] = [];
	const lines = raw.split('\n').map(l => l.trim()).filter(l => l);

	let currentId = '';
	let currentFormat = '';

	for (const line of lines) {
		if (line.startsWith('id:') || line.startsWith('- id:')) {
			if (currentId && currentFormat) {
				formats.push({ id: currentId, format: currentFormat });
			}
			currentId = extractValue(line.replace('- ', ''));
			currentFormat = '';
		} else if (line.startsWith('format:')) {
			currentFormat = extractValue(line);
		}
	}
	if (currentId && currentFormat) {
		formats.push({ id: currentId, format: currentFormat });
	}

	return formats;
}

/**
 * Extract value from a "key: value" line, stripping quotes.
 */
function extractValue(line: string): string {
	const idx = line.indexOf(':');
	if (idx === -1) return line.trim();
	return line.slice(idx + 1).trim().replace(/^['"]|['"]$/g, '');
}

// ─── CSS Output ───────────────────────────────────────────

/**
 * Generate CSS variable declarations from style settings values.
 * Handles format units, quotes, themed colors, etc.
 */
export function generateStyleSettingsCSS(
	blocks: StyleSettingsBlock[],
	values: Record<string, string>,
	themeType: 'light' | 'dark'
): { variables: Record<string, string>; classes: string[]; css: string } {
	const variables: Record<string, string> = {};
	const classes: string[] = [];
	const cssLines: string[] = [];

	for (const block of blocks) {
		for (const setting of block.settings) {
			const val = values[setting.id];

			switch (setting.type) {
				case 'class-toggle': {
					if (val === 'true' || (val === undefined && setting.default === 'true')) {
						classes.push(setting.id);
					}
					break;
				}

				case 'class-select': {
					const selected = val ?? setting.default;
					if (selected && selected !== 'none') {
						classes.push(selected);
					}
					break;
				}

				case 'variable-text': {
					const textVal = val ?? setting.default;
					if (textVal !== undefined) {
						variables[`--${setting.id}`] = setting.quotes ? `'${textVal}'` : textVal;
					}
					break;
				}

				case 'variable-number':
				case 'variable-number-slider': {
					const numVal = val ?? setting.default;
					if (numVal !== undefined) {
						const suffix = setting.format || '';
						variables[`--${setting.id}`] = `${numVal}${suffix}`;
					}
					break;
				}

				case 'variable-select': {
					const selVal = val ?? setting.default;
					if (selVal !== undefined) {
						variables[`--${setting.id}`] = selVal;
					}
					break;
				}

				case 'variable-color': {
					const colorVal = val ?? setting.default;
					if (colorVal) {
						variables[`--${setting.id}`] = formatColor(colorVal, setting.format || 'hex', setting.id, setting.opacity);
						// Alt formats
						if (setting.altFormat) {
							for (const alt of setting.altFormat) {
								variables[`--${alt.id}`] = formatColor(colorVal, alt.format, alt.id, setting.opacity);
							}
						}
					}
					break;
				}

				case 'variable-themed-color': {
					const lightVal = values[`${setting.id}@@light`] ?? setting.defaultLight;
					const darkVal = values[`${setting.id}@@dark`] ?? setting.defaultDark;
					const themedVal = themeType === 'dark' ? darkVal : lightVal;
					if (themedVal) {
						variables[`--${setting.id}`] = formatColor(themedVal, setting.format || 'hex', setting.id, setting.opacity);
					}
					break;
				}

				// heading, info-text, color-gradient handled elsewhere or no output
			}
		}
	}

	return { variables, classes, css: cssLines.join('\n') };
}

/**
 * Format a color value according to the specified format.
 */
function formatColor(hex: string, format: string, id: string, opacity?: boolean): string {
	// Normalize hex
	let h = hex.replace(/^['"]|['"]$/g, '');
	if (!h.startsWith('#')) h = `#${h}`;
	if (h.length === 4) h = `#${h[1]}${h[1]}${h[2]}${h[2]}${h[3]}${h[3]}`;

	const r = parseInt(h.slice(1, 3), 16);
	const g = parseInt(h.slice(3, 5), 16);
	const b = parseInt(h.slice(5, 7), 16);

	// Convert to HSL
	const rf = r / 255, gf = g / 255, bf = b / 255;
	const max = Math.max(rf, gf, bf), min = Math.min(rf, gf, bf);
	let hue = 0, sat = 0;
	const light = (max + min) / 2;
	if (max !== min) {
		const d = max - min;
		sat = light > 0.5 ? d / (2 - max - min) : d / (max + min);
		if (max === rf) hue = ((gf - bf) / d + (gf < bf ? 6 : 0)) / 6;
		else if (max === gf) hue = ((bf - rf) / d + 2) / 6;
		else hue = ((rf - gf) / d + 4) / 6;
	}
	const H = Math.round(hue * 360), S = Math.round(sat * 100), L = Math.round(light * 100);

	switch (format) {
		case 'hex': return opacity ? `${h}FF` : h;
		case 'rgb': return opacity ? `rgba(${r}, ${g}, ${b}, 1)` : `rgb(${r}, ${g}, ${b})`;
		case 'rgb-values': return opacity ? `${r}, ${g}, ${b}, 1` : `${r}, ${g}, ${b}`;
		case 'rgb-split': return h; // Caller handles split
		case 'hsl': return opacity ? `hsla(${H}, ${S}%, ${L}%, 1)` : `hsl(${H}, ${S}%, ${L}%)`;
		case 'hsl-values': return opacity ? `${H}, ${S}%, ${L}%, 1` : `${H}, ${S}%, ${L}%`;
		case 'hsl-split': return h; // Caller handles split
		case 'hsl-split-decimal': return h; // Caller handles split
		default: return h;
	}
}

// ─── Localization Helper ──────────────────────────────────

/**
 * Get the localized title for a setting, falling back to default title.
 */
export function getLocalizedTitle(setting: StyleSetting, locale: string): string {
	return setting.localizedTitles?.[locale] ?? setting.title;
}

/**
 * Get the localized description for a setting, falling back to default.
 */
export function getLocalizedDescription(setting: StyleSetting, locale: string): string | undefined {
	return setting.localizedDescriptions?.[locale] ?? setting.description;
}
