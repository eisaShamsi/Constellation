#!/usr/bin/env node
/**
 * MIG-039 (Cataloger note-picker) — add three new cataloger strings across
 * all 15 locales:
 *   cataloger.classifyNote  — button label "Classify a note…"
 *   cataloger.searchNotes   — search-input placeholder
 *   cataloger.noNotesFound  — empty state message
 */
import fs from 'fs';
import path from 'path';
import { fileURLToPath } from 'url';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const I18N_DIR = path.resolve(__dirname, '..', 'src', 'lib', 'i18n');

const STRINGS = {
  en: { classifyNote: 'Classify a note…', searchNotes: 'Search notes…', noNotesFound: 'No notes found' },
  ar: { classifyNote: 'صنِّف ملاحظة…',   searchNotes: 'ابحث في الملاحظات…', noNotesFound: 'لا توجد ملاحظات' },
  de: { classifyNote: 'Notiz klassifizieren…', searchNotes: 'Notizen suchen…', noNotesFound: 'Keine Notizen gefunden' },
  es: { classifyNote: 'Clasificar una nota…', searchNotes: 'Buscar notas…', noNotesFound: 'No se encontraron notas' },
  fa: { classifyNote: 'دسته‌بندی یادداشت…', searchNotes: 'جستجوی یادداشت‌ها…', noNotesFound: 'یادداشتی یافت نشد' },
  fr: { classifyNote: 'Classer une note…', searchNotes: 'Rechercher des notes…', noNotesFound: 'Aucune note trouvée' },
  he: { classifyNote: 'סווג פתק…', searchNotes: 'חפש פתקים…', noNotesFound: 'לא נמצאו פתקים' },
  hi: { classifyNote: 'नोट वर्गीकृत करें…', searchNotes: 'नोट्स खोजें…', noNotesFound: 'कोई नोट नहीं मिला' },
  ja: { classifyNote: 'ノートを分類…', searchNotes: 'ノートを検索…', noNotesFound: 'ノートが見つかりません' },
  ko: { classifyNote: '노트 분류…', searchNotes: '노트 검색…', noNotesFound: '노트를 찾을 수 없음' },
  pt: { classifyNote: 'Classificar uma nota…', searchNotes: 'Pesquisar notas…', noNotesFound: 'Nenhuma nota encontrada' },
  ru: { classifyNote: 'Классифицировать заметку…', searchNotes: 'Поиск заметок…', noNotesFound: 'Заметки не найдены' },
  tr: { classifyNote: 'Notu sınıflandır…', searchNotes: 'Notları ara…', noNotesFound: 'Not bulunamadı' },
  ur: { classifyNote: 'نوٹ درجہ بندی کریں…', searchNotes: 'نوٹس تلاش کریں…', noNotesFound: 'کوئی نوٹ نہیں ملا' },
  zh: { classifyNote: '分类笔记…', searchNotes: '搜索笔记…', noNotesFound: '未找到笔记' },
};

let updated = 0;
for (const [locale, strings] of Object.entries(STRINGS)) {
  const file = path.join(I18N_DIR, `${locale}.json`);
  if (!fs.existsSync(file)) {
    console.error(`MISSING: ${file}`);
    continue;
  }
  const json = JSON.parse(fs.readFileSync(file, 'utf8'));
  json.cataloger = json.cataloger || {};
  json.cataloger.classifyNote  = strings.classifyNote;
  json.cataloger.searchNotes   = strings.searchNotes;
  json.cataloger.noNotesFound  = strings.noNotesFound;
  fs.writeFileSync(file, JSON.stringify(json, null, '\t') + '\n', 'utf8');
  updated++;
  console.log(`  ✓ ${locale}.json`);
}
console.log(`\nDone — ${updated}/${Object.keys(STRINGS).length} locale files updated.`);
