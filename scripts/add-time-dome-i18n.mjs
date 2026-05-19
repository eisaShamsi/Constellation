#!/usr/bin/env node
/**
 * MIG-037 P1 — Add Time Dome i18n entries across all 15 locales.
 *
 * Adds:
 *   - sight.v6.tradition.list.time-dome.{name, tooltip, scope}
 *   - sight.v6.tradition.family.time
 *
 * One-shot script; safe to re-run (idempotent — overwrites existing
 * entries if any). After this lands, the entries are part of the
 * locale files themselves and this script can be deleted, or kept
 * as a record of the additions.
 */
import fs from 'fs';
import path from 'path';
import { fileURLToPath } from 'url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const I18N_DIR = path.resolve(__dirname, '..', 'src', 'lib', 'i18n');

// Per-locale translations. English is the canonical source; other
// languages are best-effort native equivalents using the project's
// established convention (right native term for the language, not
// a transliteration — per the feedback_full_localization_everything
// memory rule that drove MIG-026 §λ).
const TRANSLATIONS = {
	en: {
		name: 'Time Dome',
		tooltip: 'Time Dome — when did you create your knowledge? Stars positioned by their creation month around the calendar rim.',
		scope: 'For temporal analysis of your knowledge — which months produced the most notes, where are the gaps, how creation rhythm maps onto maturity strata.',
		family: 'Time',
	},
	ar: {
		name: 'قبة الزمن',
		tooltip: 'قبة الزمن — متى أنشأت معرفتك؟ النجوم موضوعة حسب شهر إنشائها حول حلقة التقويم.',
		scope: 'للتحليل الزمني لمعرفتك — أي الشهور أنتجت أكثر الملاحظات، أين الفجوات، كيف يرتبط إيقاع الإنشاء بطبقات النضج.',
		family: 'الزمن',
	},
	de: {
		name: 'Zeit-Kuppel',
		tooltip: 'Zeit-Kuppel — wann haben Sie Ihr Wissen geschaffen? Sterne nach Erstellungsmonat um den Kalenderrand angeordnet.',
		scope: 'Für die zeitliche Analyse Ihres Wissens — welche Monate brachten die meisten Notizen hervor, wo sind die Lücken, wie der Erstellungsrhythmus mit den Reifeschichten zusammenpasst.',
		family: 'Zeit',
	},
	es: {
		name: 'Cúpula del Tiempo',
		tooltip: 'Cúpula del Tiempo — ¿cuándo creó su conocimiento? Estrellas posicionadas por su mes de creación alrededor del calendario.',
		scope: 'Para el análisis temporal de su conocimiento — qué meses produjeron más notas, dónde están los vacíos, cómo se relaciona el ritmo de creación con los estratos de madurez.',
		family: 'Tiempo',
	},
	fa: {
		name: 'گنبد زمان',
		tooltip: 'گنبد زمان — دانش خود را چه زمانی ساختید؟ ستاره‌ها بر اساس ماه آفرینش‌شان دور حلقه تقویم قرار گرفته‌اند.',
		scope: 'برای تحلیل زمانی دانش شما — کدام ماه‌ها بیشترین یادداشت‌ها را تولید کردند، شکاف‌ها کجاست، چگونه ریتم آفرینش با لایه‌های پختگی مطابقت دارد.',
		family: 'زمان',
	},
	fr: {
		name: 'Dôme du Temps',
		tooltip: 'Dôme du Temps — quand avez-vous créé vos connaissances ? Étoiles positionnées selon leur mois de création autour du calendrier.',
		scope: 'Pour l’analyse temporelle de vos connaissances — quels mois ont produit le plus de notes, où sont les lacunes, comment le rythme de création s’aligne sur les strates de maturité.',
		family: 'Temps',
	},
	he: {
		name: 'כיפת הזמן',
		tooltip: 'כיפת הזמן — מתי יצרת את הידע שלך? הכוכבים ממוקמים לפי חודש היצירה סביב טבעת הלוח.',
		scope: 'לניתוח זמני של הידע שלך — אילו חודשים הפיקו הכי הרבה הערות, היכן הפערים, כיצד קצב היצירה מתמפה על שכבות הבשלות.',
		family: 'זמן',
	},
	hi: {
		name: 'समय गुम्बद',
		tooltip: 'समय गुम्बद — आपने अपना ज्ञान कब बनाया? तारे उनके निर्माण माह के अनुसार कैलेंडर रिम के चारों ओर रखे गए हैं।',
		scope: 'अपने ज्ञान के समय-संबंधी विश्लेषण के लिए — किन महीनों में सबसे अधिक नोट्स बने, अंतराल कहाँ हैं, सृजन की लय परिपक्वता स्तरों के साथ कैसे संगत है।',
		family: 'समय',
	},
	ja: {
		name: '時間ドーム',
		tooltip: '時間ドーム — いつ知識を作りましたか？ 星はカレンダーリムに沿って作成月で配置されます。',
		scope: '知識の時間分析のために — どの月に最も多くのノートが作成されたか、ギャップはどこか、創作リズムが成熟階層にどうマッピングされるか。',
		family: '時間',
	},
	ko: {
		name: '시간 돔',
		tooltip: '시간 돔 — 언제 지식을 만들었나요? 별이 생성 월에 따라 달력 림 주위에 배치됩니다.',
		scope: '지식의 시간 분석을 위해 — 어떤 달에 가장 많은 노트가 생성되었는지, 공백은 어디인지, 창작 리듬이 성숙 계층에 어떻게 매핑되는지.',
		family: '시간',
	},
	pt: {
		name: 'Cúpula do Tempo',
		tooltip: 'Cúpula do Tempo — quando você criou seu conhecimento? Estrelas posicionadas pelo mês de criação ao redor do calendário.',
		scope: 'Para análise temporal do seu conhecimento — quais meses produziram mais notas, onde estão as lacunas, como o ritmo de criação se mapeia nos estratos de maturidade.',
		family: 'Tempo',
	},
	ru: {
		name: 'Купол Времени',
		tooltip: 'Купол Времени — когда вы создали свои знания? Звёзды расположены по месяцу создания вокруг календарной кромки.',
		scope: 'Для временного анализа ваших знаний — в каких месяцах создано больше заметок, где пробелы, как ритм создания соотносится со слоями зрелости.',
		family: 'Время',
	},
	tr: {
		name: 'Zaman Kubbesi',
		tooltip: 'Zaman Kubbesi — bilginizi ne zaman oluşturdunuz? Yıldızlar oluşturulma ayına göre takvim çerçevesi etrafında yerleştirilir.',
		scope: 'Bilginizin zamansal analizi için — hangi aylarda en çok not üretildi, boşluklar nerede, oluşturma ritmi olgunluk katmanlarıyla nasıl eşleşiyor.',
		family: 'Zaman',
	},
	ur: {
		name: 'گنبدِ وقت',
		tooltip: 'گنبدِ وقت — آپ نے اپنا علم کب بنایا؟ ستارے ان کے بنائے جانے کے مہینے کے مطابق تقویم کے کنارے پر رکھے گئے ہیں۔',
		scope: 'اپنے علم کے زمانی تجزیہ کے لیے — کن مہینوں میں سب سے زیادہ نوٹس بنے، خلا کہاں ہیں، تخلیق کی تال پختگی کی تہوں سے کیسے میل کھاتی ہے۔',
		family: 'وقت',
	},
	zh: {
		name: '时间穹顶',
		tooltip: '时间穹顶 — 您何时创建了知识？星星按创建月份排列在日历边缘周围。',
		scope: '对您知识的时间分析 — 哪些月份产生了最多笔记，差距在哪里，创建节奏如何映射到成熟度层级。',
		family: '时间',
	},
};

const locales = Object.keys(TRANSLATIONS);
let updated = 0;

for (const locale of locales) {
	const file = path.join(I18N_DIR, `${locale}.json`);
	if (!fs.existsSync(file)) {
		console.error(`MISSING: ${file}`);
		continue;
	}
	const raw = fs.readFileSync(file, 'utf8');
	const json = JSON.parse(raw);

	// Defensive: walk down sight.v6.tradition.list, creating nodes
	// if absent. Same for sight.v6.tradition.family.
	json.sight = json.sight || {};
	json.sight.v6 = json.sight.v6 || {};
	json.sight.v6.tradition = json.sight.v6.tradition || {};
	json.sight.v6.tradition.list = json.sight.v6.tradition.list || {};
	json.sight.v6.tradition.family = json.sight.v6.tradition.family || {};

	const tr = TRANSLATIONS[locale];
	json.sight.v6.tradition.list['time-dome'] = {
		name: tr.name,
		tooltip: tr.tooltip,
		scope: tr.scope,
	};
	json.sight.v6.tradition.family.time = tr.family;

	// Preserve trailing newline + use tab indent (matches project
	// convention seen in en.json).
	const out = JSON.stringify(json, null, '\t') + '\n';
	fs.writeFileSync(file, out, 'utf8');
	updated++;
	console.log(`  ✓ ${locale}.json`);
}

console.log(`\nDone — ${updated}/${locales.length} locale files updated.`);
