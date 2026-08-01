#!/usr/bin/env node
/**
 * i18n parity report — the authoritative locale key-set diff.
 *
 * WHY THIS EXISTS (2026-08-01). CLAUDE.md's standing order is that every
 * user-facing string goes through `$t()` and all 15 locale files move together.
 * That discipline slipped: a sweep found en.json missing 11 live keys and the
 * 13 non-en/ar locales missing 62 each. Because `t()` falls back
 * active-locale → en → raw-key, a key missing from a NON-en locale renders
 * English (degraded), but a key missing from EN renders the RAW KEY PATH in all
 * 15 languages (broken). en.json is therefore the highest-severity locale, not
 * the lowest — which is why neither ar.json nor en.json alone can be "the
 * reference". The reference is the UNION, minus the exemptions below.
 *
 * TWO KEY SPACES, TWO DIFFERENT RULES:
 *
 *   1. Ordinary keys — governed by the UNION of all locales. Every locale must
 *      carry every key any locale carries.
 *
 *   2. `plurals.*` — NOT union-governed. Per MIG-087 each noun is a CLDR
 *      category map consumed by `$tn()` via `Intl.PluralRules`. The correct
 *      category set is a property of the LANGUAGE, not of the union:
 *        ar → zero one two few many other      he → one two other
 *        ru → one few many other               es/fr/pt → one many other
 *        en/de/fa/ur/hi/tr → one other         zh/ja/ko → other
 *      Taking the union here would force `plurals.characters.two` into English,
 *      where `Intl.PluralRules('en')` can never select it — a permanently dead
 *      key. So plurals are checked against `Intl.PluralRules` itself: the same
 *      engine the runtime uses, so the test cannot disagree with production.
 *
 * Usage:
 *   node scripts/i18n-parity.mjs            # human-readable report
 *   node scripts/i18n-parity.mjs --json     # machine-readable
 *   node scripts/i18n-parity.mjs --keys     # print the reference key set
 *
 * Exit code is 0 on parity, 1 on drift — usable as a pre-commit gate.
 * The vitest guard `tests/i18n/locale-parity.test.ts` imports this module's
 * helpers so the script and the test can never disagree.
 */
import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const HERE = path.dirname(fileURLToPath(import.meta.url));
export const I18N_DIR = path.resolve(HERE, '..', 'src', 'lib', 'i18n');

/**
 * The repo type-checks .js/.mjs (`checkJs: true`, `strict: true` in
 * tsconfig.json), and tests/i18n/locale-parity.test.ts imports this module — so
 * these JSDoc annotations are load-bearing, not decoration.
 *
 * @typedef {'en'|'ar'|'fa'|'he'|'ur'|'es'|'fr'|'de'|'zh'|'ja'|'ko'|'pt'|'ru'|'hi'|'tr'} Locale
 * @typedef {Map<string, unknown>} LocaleMap
 * @typedef {Record<Locale, LocaleMap>} LocaleMaps
 * @typedef {object} ReportEntry
 * @property {number} total
 * @property {string[]} missing
 * @property {string[]} extra
 * @property {string[]} pluralMissing
 * @property {string[]} pluralExtra
 * @property {Intl.LDMLPluralRule[]} cldr
 * @typedef {Record<Locale, ReportEntry>} Report
 */

/**
 * Locale order mirrors SUPPORTED_LOCALES in src/lib/i18n/index.ts.
 * @type {Locale[]}
 */
export const LOCALES = [
	'en', 'ar', 'fa', 'he', 'ur', 'es', 'fr', 'de',
	'zh', 'ja', 'ko', 'pt', 'ru', 'hi', 'tr',
];

/**
 * Key paths deliberately excluded from the parity contract.
 *
 * `sight.v5.*` — Sight v5's component (`SightV5.svelte`) was DELETED and
 * `SIGHT_V5_ENABLED` retired by MIG-028 (2026-05-18). src/lib/sight/engine.ts
 * states the standing policy: retired-engine i18n key paths are "RETAINED as
 * architectural-history record". They stay on disk in en/ar as that record, but
 * demanding 13 more translations of a deleted engine's UI would enshrine ~520
 * dead strings. Exempt, not deleted — Boss-ruled 2026-08-01.
 *
 * If v5 is ever revived, delete its entry here and the test will immediately
 * demand the 13 missing locales.
 */
export const EXEMPT_PREFIXES = ['sight.v5.'];

/**
 * Editorial metadata embedded in the JSON (`_comment`, `_translation_note`) —
 * never rendered, never looked up by `$t()`. Any key whose LAST segment starts
 * with `_` is documentation for translators, not a string. Excluded from parity
 * so a note added to one locale doesn't demand 14 fake translations of it.
 */
/** @param {string} key */
export const isEditorialKey = (key) => (key.split('.').pop() ?? '').startsWith('_');

/**
 * Flatten a nested locale object to dot-notation leaf keys.
 * @param {Record<string, unknown>} obj
 * @param {string} [prefix]
 * @param {LocaleMap} [out]
 * @returns {LocaleMap}
 */
export function flatten(obj, prefix = '', out = new Map()) {
	for (const [k, v] of Object.entries(obj)) {
		const key = prefix ? `${prefix}.${k}` : k;
		if (v && typeof v === 'object' && !Array.isArray(v)) {
			flatten(/** @type {Record<string, unknown>} */ (v), key, out);
		} else out.set(key, v);
	}
	return out;
}

/**
 * @param {Locale} loc
 * @returns {Record<string, unknown>}
 */
export function loadLocale(loc) {
	return JSON.parse(fs.readFileSync(path.join(I18N_DIR, `${loc}.json`), 'utf8'));
}

/** @returns {LocaleMaps} */
export function loadAll() {
	return /** @type {LocaleMaps} */ (
		Object.fromEntries(LOCALES.map((l) => [l, flatten(loadLocale(l))]))
	);
}

/** @param {string} key */
const isExempt = (key) =>
	EXEMPT_PREFIXES.some((p) => key.startsWith(p)) || isEditorialKey(key);

/**
 * CLDR cardinal categories for a locale, straight from the runtime's own engine.
 * @param {Locale} loc
 * @returns {Intl.LDMLPluralRule[]}
 */
export function cldrCategories(loc) {
	return new Intl.PluralRules(loc, { type: 'cardinal' }).resolvedOptions().pluralCategories;
}

/**
 * The authoritative NON-plural reference key set: the union across every
 * locale, minus exemptions. Union — not "whatever en has" — because the drift
 * ran in both directions.
 * @param {LocaleMaps} maps
 * @returns {Set<string>}
 */
export function referenceKeys(maps) {
	const union = new Set();
	for (const loc of LOCALES) {
		for (const key of maps[loc].keys()) {
			if (!key.startsWith('plurals.') && !isExempt(key)) union.add(key);
		}
	}
	return union;
}

/**
 * The set of plural NOUNS (`plurals.<noun>`) any locale defines.
 * @param {LocaleMaps} maps
 * @returns {Set<string>}
 */
export function pluralNouns(maps) {
	const nouns = new Set();
	for (const loc of LOCALES) {
		for (const key of maps[loc].keys()) {
			if (key.startsWith('plurals.')) {
				const noun = key.split('.')[1];
				if (noun) nouns.add(noun);
			}
		}
	}
	return nouns;
}

/**
 * Full drift analysis. Returns per-locale `missing` / `extra` for ordinary keys
 * and `pluralMissing` / `pluralExtra` for CLDR category mismatches.
 * @param {LocaleMaps} [maps]
 * @returns {{ reference: Set<string>, nouns: string[], report: Report }}
 */
export function analyse(maps = loadAll()) {
	const reference = referenceKeys(maps);
	const nouns = [...pluralNouns(maps)].sort();
	/** @type {Report} */
	const report = /** @type {Report} */ ({});

	for (const loc of LOCALES) {
		const have = maps[loc];
		const ordinary = new Set(
			[...have.keys()].filter((k) => !k.startsWith('plurals.') && !isExempt(k))
		);

		const missing = [...reference].filter((k) => !ordinary.has(k)).sort();
		const extra = [...ordinary].filter((k) => !reference.has(k)).sort();

		// Plurals: compare against the LANGUAGE's CLDR categories, not the union.
		const cats = cldrCategories(loc);
		const pluralMissing = [];
		const pluralExtra = [];
		for (const noun of nouns) {
			for (const c of cats) {
				if (!have.has(`plurals.${noun}.${c}`)) pluralMissing.push(`plurals.${noun}.${c}`);
			}
			// A category the language does NOT have is a dead key: Intl.PluralRules
			// can never select it, so $tn() would never reach it.
			for (const k of have.keys()) {
				if (!k.startsWith(`plurals.${noun}.`)) continue;
				const cat = k.slice(`plurals.${noun}.`.length);
				if (cat.includes('.')) continue;
				if (!(/** @type {string[]} */ (cats)).includes(cat)) pluralExtra.push(k);
			}
		}

		report[loc] = {
			total: have.size,
			missing,
			extra,
			pluralMissing: pluralMissing.sort(),
			pluralExtra: pluralExtra.sort(),
			cldr: cats,
		};
	}

	return { reference, nouns, report };
}

/**
 * True when every locale is in parity.
 * @param {Report} report
 */
export function isClean(report) {
	return Object.values(report).every(
		(r) => !r.missing.length && !r.extra.length && !r.pluralMissing.length && !r.pluralExtra.length
	);
}

// ── CLI ──────────────────────────────────────────────────────────────────────
if (import.meta.url === `file://${process.argv[1]}` || process.argv[1]?.endsWith('i18n-parity.mjs')) {
	const { reference, nouns, report } = analyse();

	if (process.argv.includes('--keys')) {
		[...reference].sort().forEach((k) => console.log(k));
		process.exit(0);
	}
	if (process.argv.includes('--json')) {
		console.log(JSON.stringify({ reference: [...reference].sort(), nouns, report }, null, 2));
		process.exit(isClean(report) ? 0 : 1);
	}

	console.log(`Reference key set: ${reference.size} keys (union − exemptions)`);
	console.log(`Plural nouns: ${nouns.length}\n`);
	console.log('locale  total  missing  extra  plural∆  CLDR categories');
	console.log('─'.repeat(72));
	for (const loc of LOCALES) {
		const r = report[loc];
		const pd = r.pluralMissing.length + r.pluralExtra.length;
		const flag = r.missing.length || r.extra.length || pd ? ' ✗' : ' ✓';
		console.log(
			`${loc.padEnd(6)} ${String(r.total).padStart(5)} ${String(r.missing.length).padStart(8)}` +
			` ${String(r.extra.length).padStart(6)} ${String(pd).padStart(8)}  ${r.cldr.join(',')}${flag}`
		);
	}

	for (const loc of LOCALES) {
		const r = report[loc];
		if (!r.missing.length && !r.extra.length && !r.pluralMissing.length && !r.pluralExtra.length) continue;
		console.log(`\n── ${loc} ──`);
		if (r.missing.length) console.log(`  missing (${r.missing.length}):\n    ${r.missing.join('\n    ')}`);
		if (r.extra.length) console.log(`  extra (${r.extra.length}):\n    ${r.extra.join('\n    ')}`);
		if (r.pluralMissing.length) console.log(`  plural missing (${r.pluralMissing.length}):\n    ${r.pluralMissing.join('\n    ')}`);
		if (r.pluralExtra.length) console.log(`  plural extra/dead (${r.pluralExtra.length}):\n    ${r.pluralExtra.join('\n    ')}`);
	}

	const clean = isClean(report);
	console.log(`\n${clean ? '✓ All 15 locales in parity.' : '✗ Locale drift detected.'}`);
	process.exit(clean ? 0 : 1);
}
