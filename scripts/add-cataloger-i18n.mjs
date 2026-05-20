#!/usr/bin/env node
/**
 * MIG-039 — Add "The Cataloger" i18n entries across all 15 locales.
 *
 * Adds:
 *   - ribbon.cataloger          (left-dock button tooltip)
 *   - commands.cataloger        (command palette entry)
 *   - cataloger.title           (full-page view header)
 *   - cataloger.tagline         (one-line description under the header)
 *   - cece.queueShowMore        ("Show more" — render-cap footer, MIG-039 perf fix)
 *   - cece.queueShowingCount    ("Showing {shown} of {total}")
 *
 * Naming decision (CECE Concept Paper §10, Eisa 2026-05-19):
 *   - English user-facing brand stays "The Cataloger".
 *   - ar = المُصنِّف (the *classifier* sense, Eisa's choice — not مُفهرِس).
 *   - The other 13 locales follow the CLASSIFIER sense, not the literal
 *     library-"cataloger" word (per Concept Paper §10 candidate list +
 *     the full-localization standing order: right native equivalent).
 *
 * Internal engine name stays "CECE" — this only touches user-facing chrome.
 *
 * One-shot, idempotent (overwrites if re-run). Writes with tab indent +
 * trailing newline to match the project convention (see en.json + the
 * MIG-037 add-time-dome-i18n.mjs template).
 */
import fs from 'fs';
import path from 'path';
import { fileURLToPath } from 'url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const I18N_DIR = path.resolve(__dirname, '..', 'src', 'lib', 'i18n');

// name        — the classifier-sense brand (Concept Paper §10).
// tagline     — "Classify each note by its kind of knowledge and its source."
// showMore    — "Show more" (render-cap footer button).
// showingCount— "Showing {shown} of {total}" ({shown}/{total} interpolated).
// No claim of "AI"/"LLM" anywhere (CECE ships as a heuristic ensemble, §5).
const TRANSLATIONS = {
	en: { name: 'The Cataloger', tagline: 'Classify each note by its kind of knowledge and its source.', showMore: 'Show more', showingCount: 'Showing {shown} of {total}' },
	ar: { name: 'المُصنِّف', tagline: 'صنّف كل ملاحظة حسب نوع معرفتها ومصدرها.', showMore: 'عرض المزيد', showingCount: 'عرض {shown} من {total}' },
	de: { name: 'Klassifikator', tagline: 'Klassifiziere jede Notiz nach ihrer Wissensart und ihrer Quelle.', showMore: 'Mehr anzeigen', showingCount: '{shown} von {total} angezeigt' },
	es: { name: 'Clasificador', tagline: 'Clasifica cada nota por su tipo de conocimiento y su fuente.', showMore: 'Mostrar más', showingCount: 'Mostrando {shown} de {total}' },
	fa: { name: 'دسته‌بند', tagline: 'هر یادداشت را بر اساس نوع دانش و منبع آن دسته‌بندی کنید.', showMore: 'نمایش بیشتر', showingCount: 'نمایش {shown} از {total}' },
	fr: { name: 'Classificateur', tagline: 'Classez chaque note selon son type de connaissance et sa source.', showMore: 'Afficher plus', showingCount: 'Affichage de {shown} sur {total}' },
	he: { name: 'המסווג', tagline: 'סווג כל פתק לפי סוג הידע שלו ומקורו.', showMore: 'הצג עוד', showingCount: 'מציג {shown} מתוך {total}' },
	hi: { name: 'वर्गीकारक', tagline: 'प्रत्येक नोट को उसके ज्ञान के प्रकार और स्रोत के अनुसार वर्गीकृत करें।', showMore: 'और दिखाएँ', showingCount: '{total} में से {shown} दिखा रहे हैं' },
	ja: { name: '分類器', tagline: '各ノートを知識の種類と出典で分類します。', showMore: 'もっと表示', showingCount: '{total} 件中 {shown} 件を表示' },
	ko: { name: '분류기', tagline: '각 노트를 지식의 종류와 출처에 따라 분류합니다.', showMore: '더 보기', showingCount: '{total}개 중 {shown}개 표시' },
	pt: { name: 'Classificador', tagline: 'Classifique cada nota pelo seu tipo de conhecimento e pela sua fonte.', showMore: 'Mostrar mais', showingCount: 'Mostrando {shown} de {total}' },
	ru: { name: 'Классификатор', tagline: 'Классифицируйте каждую заметку по виду знания и его источнику.', showMore: 'Показать ещё', showingCount: 'Показано {shown} из {total}' },
	tr: { name: 'Sınıflandırıcı', tagline: 'Her notu bilgi türüne ve kaynağına göre sınıflandırın.', showMore: 'Daha fazla göster', showingCount: '{total} öğeden {shown} tanesi gösteriliyor' },
	ur: { name: 'درجہ بند', tagline: 'ہر نوٹ کو اس کے علم کی نوعیت اور ماخذ کے مطابق درجہ بند کریں۔', showMore: 'مزید دکھائیں', showingCount: '{total} میں سے {shown} دکھائے جا رہے ہیں' },
	zh: { name: '分类器', tagline: '按知识类型与来源对每条笔记进行分类。', showMore: '显示更多', showingCount: '显示 {total} 条中的 {shown} 条' },
};

const locales = Object.keys(TRANSLATIONS);
let updated = 0;

for (const locale of locales) {
	const file = path.join(I18N_DIR, `${locale}.json`);
	if (!fs.existsSync(file)) {
		console.error(`MISSING: ${file}`);
		continue;
	}
	const json = JSON.parse(fs.readFileSync(file, 'utf8'));
	const tr = TRANSLATIONS[locale];

	json.ribbon = json.ribbon || {};
	json.commands = json.commands || {};
	json.cece = json.cece || {};
	json.ribbon.cataloger = tr.name;
	json.commands.cataloger = tr.name;
	json.cataloger = { title: tr.name, tagline: tr.tagline };
	json.cece.queueShowMore = tr.showMore;
	json.cece.queueShowingCount = tr.showingCount;

	fs.writeFileSync(file, JSON.stringify(json, null, '\t') + '\n', 'utf8');
	updated++;
	console.log(`  ✓ ${locale}.json`);
}

console.log(`\nDone — ${updated}/${locales.length} locale files updated.`);
