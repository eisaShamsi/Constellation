/**
 * frontmatter — MIG-076 §CB-2: the YAML frontmatter parse/serialize cluster,
 * extracted VERBATIM from store.ts (no logic change) so the buffer layer
 * (noteBuffers.ts) can compose content without importing the store —
 * keeping the editor document-model a dependency leaf (no cycles).
 *
 * store.ts re-exports everything here, so every existing import path
 * (`from '$lib/libraries/store'`) keeps working unchanged.
 */

export type PropertyType = 'text' | 'number' | 'date' | 'datetime' | 'list' | 'link' | 'checkbox' | 'nested-object-list';

export interface FrontmatterProperty {
	key: string;
	value: string;
	type: PropertyType;
	listItems?: string[];
	/** MIG-022 §A.1 (PJ-041 cluster, 2026-05-11) — for the
	 *  `nested-object-list` type, holds the structured row data.
	 *  Each row is a `{ field: value }` map. Used today by the
	 *  `ikhtilāf` field which stores `[{ school, position }, ...]`;
	 *  generalized so future schema additions (e.g. `ḥadīth_chain`,
	 *  `citation_block`) can reuse the parser + serializer.
	 *
	 *  When this field is present, `value` carries a compact
	 *  one-line summary suitable for legacy display + search
	 *  matching ("Hanafī: permissible | Mālikī: discouraged"); the
	 *  authoritative source-of-truth is `nestedObjects`.
	 */
	nestedObjects?: Array<Record<string, string>>;
}

// Well-known property key sets (English + Arabic)
const LIST_KEYS = new Set([
	'tags', 'aliases', 'cssclasses', 'cssclass', 'related', 'categories', 'group',
	'الوسم', 'وسوم', 'المجموعة', 'ذات صلة', 'أسماء بديلة', 'تصنيفات',
	// MIG-022 §A.1 — gap-analysis §6.1 metadata extensions.
	// `domain` is a list ("[fiqh, photography, overland-travel]") of
	// per-note subject-matter tags. Per the gap analysis, the existing
	// `tags` field is the user's free-form folksonomy; `domain` is the
	// structured discipline/topic field for retrieval.
	'domain',
]);
const CHECKBOX_KEYS = new Set([
	'done', 'completed', 'draft', 'publish', 'published', 'pinned', 'archived', 'starred', 'todo',
	'favorite', 'featured', 'hidden',
	'مكتمل', 'منشور', 'مسودة', 'مثبت', 'مؤرشف', 'مميز', 'مخفي',
]);
const DATE_KEYS = new Set([
	'date', 'created', 'updated', 'modified', 'due', 'start', 'end', 'deadline', 'completed_date',
	'أنشئ', 'حُدث', 'تاريخ', 'تعديل', 'موعد', 'بداية', 'نهاية',
	// MIG-022 §A.1 — gap-analysis §6.1: ISO date of last epistemic
	// state revision. Distinct from `updated`/`modified` (file-system
	// touch); `updated_at` is the user's deliberate stance-revision
	// timestamp.
	'updated_at',
]);
// MIG-022 §A.1 — gap-analysis §6.1: list-of-objects support, primarily
// for `ikhtilāf` (structured scholarly disagreement). Each entry has a
// `school` field + a `position` field. The parser detects these by
// (a) the key matching IKHTILAF_KEYS, AND (b) the next line being an
// indented `- field: value` line. The §A.3 Properties panel renders
// these via the custom ikhtilāf widget per D-A4.α; raw consumers can
// read `nestedObjects` directly.
const IKHTILAF_KEYS = new Set([
	'ikhtilāf', 'ikhtilaf', 'الاختلاف',
]);

/** Normalize DD/MM/YYYY → YYYY-MM-DD for storage */
export function normalizeDateValue(value: string): string {
	const ddmmyyyy = value.match(/^(\d{1,2})\/(\d{1,2})\/(\d{4})$/);
	if (ddmmyyyy) {
		const [, d, m, y] = ddmmyyyy;
		return `${y}-${m.padStart(2, '0')}-${d.padStart(2, '0')}`;
	}
	return value;
}

function detectPropertyType(key: string, value: string): PropertyType {
	const k = key.toLowerCase();

	// MIG-022 §A.1 — nested-object-list detection (highest priority).
	// `ikhtilāf` and its transliterations route to the structured
	// nested-object-list parser. The parseFrontmatter caller checks
	// the same set BEFORE entering the simple-list branch.
	if (IKHTILAF_KEYS.has(key) || IKHTILAF_KEYS.has(k)) return 'nested-object-list';

	// List detection (highest priority for known keys)
	if (LIST_KEYS.has(k)) return 'list';
	if (value.startsWith('[') && value.endsWith(']')) return 'list';

	// Link detection
	if (/^\[\[.*\]\]$/.test(value)) return 'link';

	// Checkbox / boolean detection
	const lv = value.toLowerCase();
	if (lv === 'true' || lv === 'false') return 'checkbox';
	if (CHECKBOX_KEYS.has(k) && value === '') return 'checkbox';

	// Datetime detection (with time component)
	if (/^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}(:\d{2})?$/.test(value)) return 'datetime';

	// Date detection (date only, including DD/MM/YYYY)
	if (/^\d{4}-\d{2}-\d{2}$/.test(value)) return 'date';
	if (/^\d{1,2}\/\d{1,2}\/\d{4}$/.test(value)) return 'date';
	if (DATE_KEYS.has(k) && value) return 'date';

	// Number detection
	if (/^-?\d+(\.\d+)?$/.test(value) && value !== '') return 'number';

	return 'text';
}

// ─── Frontmatter parsing ───
export function parseFrontmatter(content: string): { properties: FrontmatterProperty[]; body: string; rawYaml?: string } {
	const lines = content.split('\n');
	if (lines[0]?.trim() !== '---') {
		return { properties: [], body: content };
	}

	let endIndex = -1;
	for (let i = 1; i < lines.length; i++) {
		if (lines[i].trim() === '---') {
			endIndex = i;
			break;
		}
	}

	if (endIndex === -1) {
		return { properties: [], body: content };
	}

	const yamlLines = lines.slice(1, endIndex);
	const rawYaml = yamlLines.join('\n');
	const properties: FrontmatterProperty[] = [];

	let i = 0;
	while (i < yamlLines.length) {
		const line = yamlLines[i];
		const colonIdx = line.indexOf(':');

		if (colonIdx > 0 && !line.startsWith(' ') && !line.startsWith('\t')) {
			const key = line.substring(0, colonIdx).trim();
			let value = line.substring(colonIdx + 1).trim();

			// MIG-022 §A.1 — nested-object-list (e.g. ikhtilāf):
			//   ikhtilāf:
			//     - school: Hanafī
			//       position: permissible
			//     - school: Mālikī
			//       position: discouraged
			// Detect when the key is in IKHTILAF_KEYS AND the next line
			// is an indented `- field:` start. Each item gathers its
			// continuation lines (also indented but without `- `) into
			// a single Record<string, string>.
			if (
				!value &&
				(IKHTILAF_KEYS.has(key) || IKHTILAF_KEYS.has(key.toLowerCase())) &&
				i + 1 < yamlLines.length &&
				/^\s+-\s/.test(yamlLines[i + 1])
			) {
				i++;
				const nestedObjects: Array<Record<string, string>> = [];
				while (i < yamlLines.length) {
					const cur = yamlLines[i];
					if (/^\s+-\s/.test(cur)) {
						// New row begins. The first field is on this line:
						// "    - school: Hanafī"
						const obj: Record<string, string> = {};
						const firstFieldLine = cur.replace(/^\s+-\s*/, '');
						const firstColon = firstFieldLine.indexOf(':');
						if (firstColon > 0) {
							const fkey = firstFieldLine.substring(0, firstColon).trim();
							let fval = firstFieldLine.substring(firstColon + 1).trim();
							if ((fval.startsWith('"') && fval.endsWith('"')) || (fval.startsWith("'") && fval.endsWith("'"))) {
								fval = fval.slice(1, -1);
							}
							if (fkey) obj[fkey] = fval;
						}
						i++;
						// Gather continuation lines (indented, no leading dash) until
						// either next list-item starts or non-indented line appears.
						while (i < yamlLines.length) {
							const cont = yamlLines[i];
							if (/^\s+-\s/.test(cont)) break; // next row
							if (!/^\s/.test(cont)) break; // back to top-level key
							const contColon = cont.indexOf(':');
							if (contColon > 0) {
								const fkey = cont.substring(0, contColon).trim();
								let fval = cont.substring(contColon + 1).trim();
								if ((fval.startsWith('"') && fval.endsWith('"')) || (fval.startsWith("'") && fval.endsWith("'"))) {
									fval = fval.slice(1, -1);
								}
								if (fkey) obj[fkey] = fval;
							}
							i++;
						}
						nestedObjects.push(obj);
					} else {
						break;
					}
				}
				if (key) {
					// Compact display string for legacy consumers + search:
					// "Hanafī: permissible | Mālikī: discouraged"
					const summary = nestedObjects
						.map((o) => Object.entries(o).map(([k, v]) => `${k}: ${v}`).join(' / '))
						.join(' | ');
					properties.push({
						key,
						value: summary,
						type: 'nested-object-list',
						nestedObjects,
					});
				}
				continue;
			}

			// Multi-line list: key:\n  - item1\n  - item2
			const listItems: string[] = [];
			if (!value && i + 1 < yamlLines.length && /^\s+-\s/.test(yamlLines[i + 1])) {
				i++;
				while (i < yamlLines.length && /^\s+-\s/.test(yamlLines[i])) {
					let item = yamlLines[i].replace(/^\s+-\s*/, '').trim();
					if ((item.startsWith('"') && item.endsWith('"')) || (item.startsWith("'") && item.endsWith("'"))) {
						item = item.slice(1, -1);
					}
					listItems.push(item);
					i++;
				}
				if (key) {
					properties.push({ key, value: listItems.join(', '), type: 'list', listItems });
				}
				continue;
			}

			// Strip quotes
			if ((value.startsWith('"') && value.endsWith('"')) || (value.startsWith("'") && value.endsWith("'"))) {
				value = value.slice(1, -1);
			}

			// Inline list: [a, b, c]
			let parsedListItems: string[] | undefined;
			if (value.startsWith('[') && value.endsWith(']')) {
				parsedListItems = value.slice(1, -1)
					.split(',')
					.map(s => s.trim().replace(/^["']|["']$/g, ''))
					.filter(Boolean);
				value = parsedListItems.join(', ');
			}

			const type = detectPropertyType(key, value);
			// Normalize DD/MM/YYYY dates to YYYY-MM-DD for storage
			if ((type === 'date' || type === 'datetime') && value) {
				value = normalizeDateValue(value);
			}
			if (key) {
				properties.push({
					key,
					value,
					type,
					listItems: parsedListItems ?? (type === 'list' ? value.split(',').map(s => s.trim()).filter(Boolean) : undefined)
				});
			}
		}
		i++;
	}

	const body = lines.slice(endIndex + 1).join('\n');
	return { properties, body, rawYaml };
}

/** MIG-022 §A.1 — shared YAML value quoter. Used by reconstructFrontmatter
 *  for both flat values and nested-object-list field values. Strings with
 *  YAML special chars get double-quoted with embedded `"` escaped. */
function quoteIfNeeded(v: string): string {
	if (v === '') return '""';
	const needsQuoting = /[:{}\[\],&*?|>!%@`#]/.test(v) ||
		v.startsWith("'") || v.startsWith('"') ||
		v === 'true' || v === 'false' ||
		v === 'null' || v === 'yes' || v === 'no';
	if (needsQuoting) return `"${v.replace(/"/g, '\\"')}"`;
	return v;
}

export function reconstructFrontmatter(properties: FrontmatterProperty[]): string {
	if (properties.length === 0) return '';

	const lines: string[] = ['---'];
	for (const prop of properties) {
		if (prop.type === 'nested-object-list' && prop.nestedObjects && prop.nestedObjects.length > 0) {
			// MIG-022 §A.1 — write nested-object-list back as YAML:
			//   ikhtilāf:
			//     - school: Hanafī
			//       position: permissible
			//     - school: Mālikī
			//       position: discouraged
			// Field order within each object follows insertion order
			// (Object.entries preserves the order parseFrontmatter
			// captured). Quote values that contain YAML special chars.
			lines.push(`${prop.key}:`);
			for (const obj of prop.nestedObjects) {
				const entries = Object.entries(obj);
				if (entries.length === 0) continue;
				const [firstKey, firstVal] = entries[0];
				lines.push(`  - ${firstKey}: ${quoteIfNeeded(firstVal)}`);
				for (const [k, v] of entries.slice(1)) {
					lines.push(`    ${k}: ${quoteIfNeeded(v)}`);
				}
			}
		} else if (prop.type === 'list' && prop.listItems && prop.listItems.length > 0) {
			lines.push(`${prop.key}:`);
			for (const item of prop.listItems) {
				lines.push(`  - ${item}`);
			}
		} else if (prop.type === 'checkbox') {
			// Write bare YAML boolean (unquoted true/false)
			lines.push(`${prop.key}: ${prop.value === 'true' ? 'true' : 'false'}`);
		} else if (prop.type === 'date' || prop.type === 'datetime' || prop.type === 'number' || prop.type === 'link') {
			lines.push(`${prop.key}: ${prop.value}`);
		} else {
			const v = prop.value;
			const needsQuoting = /[:{}\[\],&*?|>!%@`#]/.test(v) ||
				v.startsWith("'") || v.startsWith('"') ||
				v === '' || v === 'true' || v === 'false' ||
				v === 'null' || v === 'yes' || v === 'no';
			if (needsQuoting && v !== '') {
				lines.push(`${prop.key}: "${v.replace(/"/g, '\\"')}"`);
			} else {
				lines.push(`${prop.key}: ${v}`);
			}
		}
	}
	lines.push('---');
	return lines.join('\n');
}

export function buildFullContent(properties: FrontmatterProperty[], body: string): string {
	const frontmatter = reconstructFrontmatter(properties);
	if (!frontmatter) return body;
	return frontmatter + '\n' + body;
}
