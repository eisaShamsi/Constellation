#!/usr/bin/env node
/**
 * MIG-040 (NSC backfill) — `nscBackfill.*` strings across all 15 locales:
 * the status-bar progress strip + the manual "Build all summaries" button in
 * the Cataloger. (The backfill is MANUAL only — Boss decision 2026-05-21 — so
 * the earlier Settings-toggle strings `settingName`/`settingDesc` are dropped;
 * this script REPLACES the `nscBackfill` object so they're removed cleanly.)
 * Idempotent; tab indent + trailing newline (project convention).
 */
import fs from 'fs';
import path from 'path';
import { fileURLToPath } from 'url';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const I18N_DIR = path.resolve(__dirname, '..', 'src', 'lib', 'i18n');

const STRINGS = {
	en: {
		label: 'Building note summaries…',
		done: 'Summaries ready',
		cancelled: 'Summary build cancelled',
		error: 'Summary build error',
		cancelling: 'Cancelling…',
		cancel: 'Cancel',
		cancelTitle: 'Cancel summary build',
		buildNow: 'Build all summaries',
		buildNowTitle: 'Pre-compute a summary for every note that lacks one. Runs in the background; progress shows in the status bar; cancel any time.',
	},
	ar: {
		label: 'جارٍ إنشاء ملخّصات الملاحظات…',
		done: 'الملخّصات جاهزة',
		cancelled: 'أُلغي إنشاء الملخّصات',
		error: 'خطأ في إنشاء الملخّصات',
		cancelling: 'جارٍ الإلغاء…',
		cancel: 'إلغاء',
		cancelTitle: 'إلغاء إنشاء الملخّصات',
		buildNow: 'أنشئ كل الملخّصات',
		buildNowTitle: 'احسب مسبقًا ملخّصًا لكل ملاحظة لا تملك واحدًا. يعمل في الخلفية، ويظهر التقدّم في شريط الحالة، ويمكنك إلغاؤه في أي وقت.',
	},
	de: {
		label: 'Notizzusammenfassungen werden erstellt…',
		done: 'Zusammenfassungen bereit',
		cancelled: 'Erstellung abgebrochen',
		error: 'Fehler bei der Erstellung',
		cancelling: 'Wird abgebrochen…',
		cancel: 'Abbrechen',
		cancelTitle: 'Erstellung abbrechen',
		buildNow: 'Alle Zusammenfassungen erstellen',
		buildNowTitle: 'Berechnet im Hintergrund für jede Notiz ohne Zusammenfassung eine. Fortschritt in der Statusleiste; jederzeit abbrechbar.',
	},
	es: {
		label: 'Generando resúmenes de notas…',
		done: 'Resúmenes listos',
		cancelled: 'Generación de resúmenes cancelada',
		error: 'Error al generar resúmenes',
		cancelling: 'Cancelando…',
		cancel: 'Cancelar',
		cancelTitle: 'Cancelar la generación',
		buildNow: 'Generar todos los resúmenes',
		buildNowTitle: 'Calcula por adelantado un resumen para cada nota que no lo tiene. Se ejecuta en segundo plano; el progreso aparece en la barra de estado; puedes cancelar cuando quieras.',
	},
	fa: {
		label: 'در حال ساخت خلاصهٔ یادداشت‌ها…',
		done: 'خلاصه‌ها آماده‌اند',
		cancelled: 'ساخت خلاصه لغو شد',
		error: 'خطا در ساخت خلاصه',
		cancelling: 'در حال لغو…',
		cancel: 'لغو',
		cancelTitle: 'لغو ساخت خلاصه',
		buildNow: 'ساخت همهٔ خلاصه‌ها',
		buildNowTitle: 'برای هر یادداشتی که خلاصه ندارد، از پیش خلاصه می‌سازد. در پس‌زمینه اجرا می‌شود؛ پیشرفت در نوار وضعیت دیده می‌شود و هر زمان می‌توانید لغو کنید.',
	},
	fr: {
		label: 'Création des résumés de notes…',
		done: 'Résumés prêts',
		cancelled: 'Création annulée',
		error: 'Erreur lors de la création',
		cancelling: 'Annulation…',
		cancel: 'Annuler',
		cancelTitle: 'Annuler la création',
		buildNow: 'Générer tous les résumés',
		buildNowTitle: 'Précalcule un résumé pour chaque note qui n’en a pas. S’exécute en arrière-plan ; progression dans la barre d’état ; annulable à tout moment.',
	},
	he: {
		label: 'בונה תקצירי הערות…',
		done: 'התקצירים מוכנים',
		cancelled: 'בניית התקצירים בוטלה',
		error: 'שגיאה בבניית התקצירים',
		cancelling: 'מבטל…',
		cancel: 'ביטול',
		cancelTitle: 'ביטול בניית התקצירים',
		buildNow: 'בניית כל התקצירים',
		buildNowTitle: 'מחשב מראש תקציר לכל הערה שאין לה. פועל ברקע; ההתקדמות מוצגת בשורת המצב; ניתן לבטל בכל עת.',
	},
	hi: {
		label: 'नोट सारांश बनाए जा रहे हैं…',
		done: 'सारांश तैयार हैं',
		cancelled: 'सारांश निर्माण रद्द किया गया',
		error: 'सारांश निर्माण में त्रुटि',
		cancelling: 'रद्द किया जा रहा है…',
		cancel: 'रद्द करें',
		cancelTitle: 'सारांश निर्माण रद्द करें',
		buildNow: 'सभी सारांश बनाएँ',
		buildNowTitle: 'हर उस नोट के लिए सारांश पहले से तैयार करता है जिसमें नहीं है। पृष्ठभूमि में चलता है; प्रगति स्थिति-पट्टी में दिखती है; कभी भी रद्द कर सकते हैं।',
	},
	ja: {
		label: 'ノートの要約を作成中…',
		done: '要約の準備ができました',
		cancelled: '要約の作成をキャンセルしました',
		error: '要約の作成エラー',
		cancelling: 'キャンセル中…',
		cancel: 'キャンセル',
		cancelTitle: '要約の作成をキャンセル',
		buildNow: 'すべての要約を作成',
		buildNowTitle: '要約のない各ノートの要約を事前計算します。バックグラウンドで実行され、進捗はステータスバーに表示され、いつでもキャンセルできます。',
	},
	ko: {
		label: '노트 요약 생성 중…',
		done: '요약 준비 완료',
		cancelled: '요약 생성이 취소됨',
		error: '요약 생성 오류',
		cancelling: '취소하는 중…',
		cancel: '취소',
		cancelTitle: '요약 생성 취소',
		buildNow: '모든 요약 생성',
		buildNowTitle: '요약이 없는 각 노트의 요약을 미리 계산합니다. 백그라운드에서 실행되며 진행 상황은 상태 표시줄에 표시되고 언제든지 취소할 수 있습니다.',
	},
	pt: {
		label: 'Gerando resumos das notas…',
		done: 'Resumos prontos',
		cancelled: 'Geração de resumos cancelada',
		error: 'Erro ao gerar resumos',
		cancelling: 'Cancelando…',
		cancel: 'Cancelar',
		cancelTitle: 'Cancelar a geração',
		buildNow: 'Gerar todos os resumos',
		buildNowTitle: 'Pré-calcula um resumo para cada nota que não tem um. Executa em segundo plano; o progresso aparece na barra de status; pode cancelar a qualquer momento.',
	},
	ru: {
		label: 'Создание сводок заметок…',
		done: 'Сводки готовы',
		cancelled: 'Создание сводок отменено',
		error: 'Ошибка создания сводок',
		cancelling: 'Отмена…',
		cancel: 'Отмена',
		cancelTitle: 'Отменить создание сводок',
		buildNow: 'Создать все сводки',
		buildNowTitle: 'Заранее вычисляет сводку для каждой заметки без неё. Работает в фоне; прогресс виден в строке состояния; можно отменить в любой момент.',
	},
	tr: {
		label: 'Not özetleri oluşturuluyor…',
		done: 'Özetler hazır',
		cancelled: 'Özet oluşturma iptal edildi',
		error: 'Özet oluşturma hatası',
		cancelling: 'İptal ediliyor…',
		cancel: 'İptal',
		cancelTitle: 'Özet oluşturmayı iptal et',
		buildNow: 'Tüm özetleri oluştur',
		buildNowTitle: 'Özeti olmayan her not için özeti önceden hesaplar. Arka planda çalışır; ilerleme durum çubuğunda görünür; istediğiniz zaman iptal edebilirsiniz.',
	},
	ur: {
		label: 'نوٹ کے خلاصے بنائے جا رہے ہیں…',
		done: 'خلاصے تیار ہیں',
		cancelled: 'خلاصہ سازی منسوخ کر دی گئی',
		error: 'خلاصہ سازی میں خرابی',
		cancelling: 'منسوخ کیا جا رہا ہے…',
		cancel: 'منسوخ کریں',
		cancelTitle: 'خلاصہ سازی منسوخ کریں',
		buildNow: 'تمام خلاصے بنائیں',
		buildNowTitle: 'ہر اُس نوٹ کا خلاصہ پہلے سے تیار کرتا ہے جس میں نہیں۔ پس منظر میں چلتا ہے؛ پیش رفت اسٹیٹس بار میں نظر آتی ہے؛ کسی بھی وقت منسوخ کر سکتے ہیں۔',
	},
	zh: {
		label: '正在生成笔记摘要…',
		done: '摘要已就绪',
		cancelled: '已取消摘要生成',
		error: '摘要生成出错',
		cancelling: '正在取消…',
		cancel: '取消',
		cancelTitle: '取消摘要生成',
		buildNow: '生成全部摘要',
		buildNowTitle: '为每条没有摘要的笔记预先计算摘要。在后台运行；进度显示在状态栏；可随时取消。',
	},
};

let updated = 0;
for (const [locale, strings] of Object.entries(STRINGS)) {
	const file = path.join(I18N_DIR, `${locale}.json`);
	if (!fs.existsSync(file)) {
		console.error(`MISSING: ${file}`);
		continue;
	}
	const json = JSON.parse(fs.readFileSync(file, 'utf8'));
	json.nscBackfill = { ...strings }; // REPLACE (drops obsolete setting* keys)
	fs.writeFileSync(file, JSON.stringify(json, null, '\t') + '\n', 'utf8');
	updated++;
	console.log(`  ✓ ${locale}.json`);
}
console.log(`\nDone — ${updated}/${Object.keys(STRINGS).length} locale files updated.`);
