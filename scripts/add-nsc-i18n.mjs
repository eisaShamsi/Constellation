#!/usr/bin/env node
/**
 * MIG-040 (NSC) — add the `nsc.summary` label across all 15 locales.
 * Shown as the small caption on each Cataloger / Source Review card's
 * summary block. Idempotent; tab indent + trailing newline (project convention).
 */
import fs from 'fs';
import path from 'path';
import { fileURLToPath } from 'url';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const I18N_DIR = path.resolve(__dirname, '..', 'src', 'lib', 'i18n');

const SUMMARY = {
	en: 'Summary',
	ar: 'ملخّص',
	de: 'Zusammenfassung',
	es: 'Resumen',
	fa: 'خلاصه',
	fr: 'Résumé',
	he: 'סיכום',
	hi: 'सारांश',
	ja: '要約',
	ko: '요약',
	pt: 'Resumo',
	ru: 'Резюме',
	tr: 'Özet',
	ur: 'خلاصہ',
	zh: '摘要',
};

let updated = 0;
for (const [locale, summary] of Object.entries(SUMMARY)) {
	const file = path.join(I18N_DIR, `${locale}.json`);
	if (!fs.existsSync(file)) {
		console.error(`MISSING: ${file}`);
		continue;
	}
	const json = JSON.parse(fs.readFileSync(file, 'utf8'));
	json.nsc = json.nsc || {};
	json.nsc.summary = summary;
	fs.writeFileSync(file, JSON.stringify(json, null, '\t') + '\n', 'utf8');
	updated++;
	console.log(`  ✓ ${locale}.json`);
}
console.log(`\nDone — ${updated}/${Object.keys(SUMMARY).length} locale files updated.`);
