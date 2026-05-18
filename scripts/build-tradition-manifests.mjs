#!/usr/bin/env node
/**
 * MIG-026 Phase ι.2 + §λ-fix-1 — build-time generation of the
 * tradition-manifest lookup map (multi-locale).
 *
 * Reads every `docs/traditions/<id>.md` (English source, the canonical
 * Phase ι.1 deliverable) PLUS every `docs/traditions/<lang>/<id>.md`
 * across the 14 non-English locales shipped in Phase λ.2.b. Writes a
 * TypeScript constant map keyed by locale → tradition id → markdown to
 * `src/lib/sight/v6/traditions/_manifests.generated.ts`. The chip's
 * ⓘ button + SightV6's manifest modal consume this map via
 * `getManifest(id, locale)` which falls back to English when a
 * locale's translation is missing.
 *
 * Source of truth: the markdown files at `docs/traditions/**\/*.md`.
 * The generated TS file is a build artifact that ships in the bundle
 * so manifests are available offline + with zero IPC. The file is
 * committed to git for first-clone-friendliness and CI determinism.
 *
 * Run modes:
 *   - automatic — fires before `npm run build` + `npm run dev` via
 *     the prebuild/predev hooks in package.json
 *   - manual    — `node scripts/build-tradition-manifests.mjs` after
 *     editing manifest .md files
 *
 * Verification: re-running produces a byte-identical file. The script
 * sorts filenames + locale codes alphabetically so the output order
 * is stable across runs.
 */
import { readdirSync, readFileSync, writeFileSync, mkdirSync, existsSync, statSync } from 'node:fs';
import { resolve, dirname } from 'node:path';
import { fileURLToPath } from 'node:url';

const __dirname = dirname(fileURLToPath(import.meta.url));
const REPO_ROOT = resolve(__dirname, '..');
const TRADITIONS_DIR = resolve(REPO_ROOT, 'docs/traditions');
const OUT_DIR = resolve(REPO_ROOT, 'src/lib/sight/v6/traditions');
const OUT_FILE = resolve(OUT_DIR, '_manifests.generated.ts');

// Supported locales mirror src/lib/i18n/index.ts. Keep in sync if a
// new locale is added there.
const LOCALES = [
	'en', 'ar', 'fa', 'he', 'ur', 'es', 'fr', 'de',
	'zh', 'ja', 'ko', 'pt', 'ru', 'hi', 'tr',
];

if (!existsSync(TRADITIONS_DIR)) {
	console.error(`[build-tradition-manifests] ERROR: ${TRADITIONS_DIR} not found.`);
	process.exit(1);
}

/**
 * Read all <id>.md files from a directory. Returns an array of
 * { id, content } sorted by id. Empty array if the directory doesn't
 * exist or has no manifest files (e.g. a locale that hasn't been
 * translated yet).
 */
function readLocaleManifests(dir) {
	if (!existsSync(dir) || !statSync(dir).isDirectory()) {
		return [];
	}
	return readdirSync(dir)
		.filter((f) => f.endsWith('.md') && f !== 'README.md')
		.sort()
		.map((f) => {
			const id = f.replace(/\.md$/, '');
			const content = readFileSync(resolve(dir, f), 'utf-8');
			return { id, content };
		});
}

// English manifests live at the top level of docs/traditions/ (no
// subfolder). Each non-en locale lives under docs/traditions/<lang>/.
const perLocale = {};
for (const lang of LOCALES) {
	const dir = lang === 'en' ? TRADITIONS_DIR : resolve(TRADITIONS_DIR, lang);
	perLocale[lang] = readLocaleManifests(dir);
}

if (perLocale.en.length === 0) {
	console.error(`[build-tradition-manifests] ERROR: no English manifests found in ${TRADITIONS_DIR}`);
	process.exit(1);
}

// Identifier-vs-string-literal key choice: TraditionId allows kebab-case
// strings like 'ibn-rushd-burhan' which are not valid TS identifiers,
// so quote them. Plain identifiers (aristotelian, pramana, etc.) stay
// unquoted for readability.
function keyToken(id) {
	return /^[a-z][a-zA-Z0-9]*$/.test(id) ? id : `'${id}'`;
}

// Escape backticks, backslashes, and ${} interpolation so the markdown
// content survives inside a TS template literal. Markdown bodies can
// contain backticks (code spans) so this is non-optional.
function escapeForTemplate(s) {
	return s
		.replace(/\\/g, '\\\\')
		.replace(/`/g, '\\`')
		.replace(/\$\{/g, '\\${');
}

const tsLines = [
	'/**',
	' * GENERATED — do not edit by hand.',
	' *',
	' * Regenerated from docs/traditions/**.md by',
	" * scripts/build-tradition-manifests.mjs (auto-runs via the `prebuild`",
	" * + `predev` npm scripts; can be invoked manually after editing any",
	' * manifest .md file).',
	' *',
	' * MIG-026 Phase ι.2 + §λ-fix-1 — multi-locale manifest content',
	' * backing the ⓘ disclosure button in the tradition chip dropdown.',
	' * Inner record is locale-specific; getManifest(id, locale) falls',
	' * back to English when a locale\'s translation is missing.',
	' */',
	'',
	"import type { TraditionId } from '../types';",
	'',
	'/** Locale codes — kept in sync with src/lib/i18n/index.ts Locale union. */',
	"export type ManifestLocale = " + LOCALES.map((l) => `'${l}'`).join(' | ') + ';',
	'',
	'/** Per-locale per-tradition manifest content map. The English entry is',
	' *  guaranteed complete (24/24 traditions); other locales may be partial',
	" *  during incremental translation, in which case getManifest()'s",
	' *  fallback chain serves the English content. */',
	'export const MANIFESTS: Record<ManifestLocale, Partial<Record<TraditionId, string>>> = {',
];

for (const lang of LOCALES) {
	tsLines.push(`\t${lang}: {`);
	for (const { id, content } of perLocale[lang]) {
		tsLines.push(`\t\t${keyToken(id)}: \`${escapeForTemplate(content)}\`,`);
	}
	tsLines.push('\t},');
}
tsLines.push('};');
tsLines.push('');
tsLines.push('/**');
tsLines.push(' * Look up a manifest by tradition id + locale.');
tsLines.push(' *');
tsLines.push(' * Fallback chain:');
tsLines.push(' *   1. requested locale + requested id');
tsLines.push(' *   2. English + requested id (when a locale skipped this manifest)');
tsLines.push(' *   3. English + Aristotelian (last-resort defensive — should not happen)');
tsLines.push(' */');
tsLines.push('export function getManifest(id: TraditionId, locale: ManifestLocale = \'en\'): string {');
tsLines.push('\tconst forLocale = MANIFESTS[locale]?.[id];');
tsLines.push('\tif (forLocale) return forLocale;');
tsLines.push('\tconst fallbackEn = MANIFESTS.en[id];');
tsLines.push('\tif (fallbackEn) return fallbackEn;');
tsLines.push('\treturn MANIFESTS.en.aristotelian ?? \'\';');
tsLines.push('}');
tsLines.push('');

mkdirSync(OUT_DIR, { recursive: true });
writeFileSync(OUT_FILE, tsLines.join('\n'), 'utf-8');

// Coverage stats
const enCount = perLocale.en.length;
const localeStats = LOCALES
	.filter((l) => l !== 'en')
	.map((l) => `${l}=${perLocale[l].length}/${enCount}`)
	.join(' ');

console.log(
	`[build-tradition-manifests] wrote ${OUT_FILE} (en=${enCount} manifests; coverage: ${localeStats}; ` +
		`${tsLines.join('\n').length} bytes)`,
);
