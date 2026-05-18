#!/usr/bin/env node
/**
 * MIG-026 Phase ι.2 — build-time generation of the tradition-manifest
 * lookup map.
 *
 * Reads every `docs/traditions/<id>.md` (excluding README.md) and
 * writes a TypeScript constant map to
 * `src/lib/sight/v6/traditions/_manifests.generated.ts`. The chip's
 * ⓘ button + SightV6's manifest modal consume this map.
 *
 * The source of truth is the markdown files at `docs/traditions/*.md`.
 * The generated TS file is a build artifact that ships in the bundle
 * so the manifests are available offline + with zero IPC. The file is
 * committed to git for first-clone-friendliness and CI determinism.
 *
 * Run modes:
 *   - automatic — fires before `npm run build` via the `prebuild` hook
 *     in package.json
 *   - manual    — `node scripts/build-tradition-manifests.mjs` after
 *     editing manifest .md files
 *
 * Verification: re-running produces a byte-identical file. The script
 * sorts filenames alphabetically so the output order is stable across
 * runs.
 */
import { readdirSync, readFileSync, writeFileSync, mkdirSync, existsSync } from 'node:fs';
import { resolve, dirname } from 'node:path';
import { fileURLToPath } from 'node:url';

const __dirname = dirname(fileURLToPath(import.meta.url));
const REPO_ROOT = resolve(__dirname, '..');
const TRADITIONS_DIR = resolve(REPO_ROOT, 'docs/traditions');
const OUT_DIR = resolve(REPO_ROOT, 'src/lib/sight/v6/traditions');
const OUT_FILE = resolve(OUT_DIR, '_manifests.generated.ts');

if (!existsSync(TRADITIONS_DIR)) {
	console.error(`[build-tradition-manifests] ERROR: ${TRADITIONS_DIR} not found.`);
	process.exit(1);
}

// Read all <id>.md files except README.md, sorted alphabetically for
// deterministic output. README is the human-facing index, not a
// tradition manifest, so it stays out of the runtime map.
const files = readdirSync(TRADITIONS_DIR)
	.filter((f) => f.endsWith('.md') && f !== 'README.md')
	.sort();

if (files.length === 0) {
	console.error(`[build-tradition-manifests] ERROR: no manifest files found in ${TRADITIONS_DIR}`);
	process.exit(1);
}

const manifests = files.map((f) => {
	const id = f.replace(/\.md$/, '');
	const content = readFileSync(resolve(TRADITIONS_DIR, f), 'utf-8');
	return { id, content };
});

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
	' * Regenerated from docs/traditions/*.md by',
	' * scripts/build-tradition-manifests.mjs (auto-runs via the',
	" * `prebuild` npm script + can be invoked manually after editing",
	' * any manifest .md file).',
	' *',
	' * MIG-026 Phase ι.2 — manifest content backing the ⓘ disclosure',
	' * button in the tradition chip dropdown.',
	' */',
	'',
	"import type { TraditionId } from '../types';",
	'',
	'export const MANIFESTS: Record<TraditionId, string> = {',
];

for (const { id, content } of manifests) {
	tsLines.push(`\t${keyToken(id)}: \`${escapeForTemplate(content)}\`,`);
}

tsLines.push('};');
tsLines.push('');
tsLines.push('/**');
tsLines.push(' * Look up a manifest by tradition id. Falls back to the Aristotelian');
tsLines.push(' * manifest if the requested id is unknown (defensive — should not');
tsLines.push(' * happen in practice since TraditionId is the source-of-truth union).');
tsLines.push(' */');
tsLines.push('export function getManifest(id: TraditionId): string {');
tsLines.push('\treturn MANIFESTS[id] ?? MANIFESTS.aristotelian;');
tsLines.push('}');
tsLines.push('');

mkdirSync(OUT_DIR, { recursive: true });
writeFileSync(OUT_FILE, tsLines.join('\n'), 'utf-8');

console.log(
	`[build-tradition-manifests] wrote ${OUT_FILE} (${manifests.length} manifests, ` +
		`${tsLines.join('\n').length} bytes)`,
);
