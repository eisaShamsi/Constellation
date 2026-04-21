#!/usr/bin/env python3
"""Add linkConfidence namespace + confidenceBackfill settings keys to 15 locales.

P5 deferred: right-click contest/force-promote menu + one-shot backfill button.
English strings already live in en.json; this script propagates translations
to the other 14 locales.
"""
import json
from pathlib import Path

# (setConfidence, rightClickHint, hypothesis, evidence, established, contested)
CONFIDENCE = {
    "en": ("Set confidence", "Right-click to set confidence", "Hypothesis", "Evidence", "Established", "Contested"),
    "ar": ("تعيين الثقة", "انقر بزر الفأرة الأيمن لتعيين الثقة", "فرضية", "دليل", "ثابت", "متنازع عليه"),
    "de": ("Vertrauen festlegen", "Rechtsklick zum Festlegen", "Hypothese", "Beleg", "Etabliert", "Umstritten"),
    "es": ("Establecer confianza", "Clic derecho para establecer confianza", "Hipótesis", "Evidencia", "Establecido", "Impugnado"),
    "fa": ("تنظیم اعتماد", "برای تنظیم اعتماد کلیک راست کنید", "فرضیه", "شواهد", "تثبیت‌شده", "مورد مناقشه"),
    "fr": ("Définir la confiance", "Clic droit pour définir la confiance", "Hypothèse", "Preuve", "Établi", "Contesté"),
    "he": ("הגדרת ביטחון", "לחץ ימני להגדרת ביטחון", "השערה", "עדות", "מבוסס", "שנוי במחלוקת"),
    "hi": ("आत्मविश्वास सेट करें", "आत्मविश्वास सेट करने के लिए राइट-क्लिक करें", "परिकल्पना", "साक्ष्य", "स्थापित", "विवादित"),
    "ja": ("信頼度を設定", "右クリックで信頼度を設定", "仮説", "証拠", "確立", "異議あり"),
    "ko": ("신뢰도 설정", "우클릭으로 신뢰도 설정", "가설", "증거", "확립됨", "이의 제기됨"),
    "pt": ("Definir confiança", "Clique direito para definir confiança", "Hipótese", "Evidência", "Estabelecido", "Contestado"),
    "ru": ("Задать уверенность", "Правый клик для настройки уверенности", "Гипотеза", "Свидетельство", "Подтверждено", "Оспаривается"),
    "tr": ("Güveni ayarla", "Güveni ayarlamak için sağ tıklayın", "Hipotez", "Kanıt", "Yerleşik", "İtirazlı"),
    "ur": ("اعتماد مقرر کریں", "اعتماد مقرر کرنے کیلئے رائٹ کلک کریں", "مفروضہ", "شواہد", "قائم", "متنازع"),
    "zh": ("设置置信度", "右键设置置信度", "假设", "证据", "已确立", "有争议"),
}

# (backfill, desc, btn, running, resultTemplate)
BACKFILL = {
    "en": (
        "Back-fill link confidence",
        "Promote existing links that already crossed a traversal threshold (≥3 → evidence, ≥10 → established) but never ran through the auto-promotion rule. One-shot; safe to run multiple times. Never downgrades; preserves user-set contested.",
        "Run back-fill", "Running…",
        "Promoted {total} link(s) (→evidence: {evidence}, →established: {established}).",
    ),
    "ar": (
        "إعادة تعبئة ثقة الروابط",
        "ترقية الروابط التي تجاوزت حد التنقلات (≥3 → دليل، ≥10 → ثابت) لكنها لم تمر بقاعدة الترقية التلقائية. تشغيل آمن مرات عديدة. لا يُخفض الدرجة ويحافظ على 'متنازع عليه' الذي حدده المستخدم.",
        "تشغيل إعادة التعبئة", "قيد التشغيل…",
        "تمت ترقية {total} رابطًا (→دليل: {evidence}، →ثابت: {established}).",
    ),
    "de": (
        "Link-Vertrauen nachtragen",
        "Stuft bestehende Links hoch, die die Schwelle schon überschritten haben (≥3 → Beleg, ≥10 → Etabliert), aber nie auto-gepromoted wurden. Einmalig; mehrfach sicher ausführbar. Stuft nie herab; erhält nutzerdefiniertes 'Umstritten'.",
        "Nachtrag ausführen", "Läuft…",
        "{total} Link(s) hochgestuft (→Beleg: {evidence}, →Etabliert: {established}).",
    ),
    "es": (
        "Rellenar confianza de enlaces",
        "Promueve enlaces existentes que ya cruzaron un umbral (≥3 → evidencia, ≥10 → establecido) pero nunca pasaron por la regla de auto-promoción. Único disparo; seguro ejecutar varias veces. Nunca degrada; conserva 'impugnado' del usuario.",
        "Ejecutar relleno", "Ejecutando…",
        "Se promovieron {total} enlace(s) (→evidencia: {evidence}, →establecido: {established}).",
    ),
    "fa": (
        "پرکردن اعتماد پیوندها",
        "ارتقای پیوندهایی که از آستانه گذشته‌اند (۳+ → شواهد، ۱۰+ → تثبیت‌شده) اما از قاعدهٔ ارتقای خودکار نگذشته‌اند. یک‌بار اجرا؛ اجرای مکرر امن است. هرگز تنزل نمی‌دهد؛ «مورد مناقشه» کاربر حفظ می‌شود.",
        "اجرای پرکردن", "در حال اجرا…",
        "{total} پیوند ارتقا یافت (→شواهد: {evidence}، →تثبیت‌شده: {established}).",
    ),
    "fr": (
        "Remplir la confiance des liens",
        "Promeut les liens existants qui ont déjà franchi un seuil (≥3 → preuve, ≥10 → établi) mais n'ont jamais traversé la règle d'auto-promotion. Un seul coup ; peut être relancé sans risque. Ne rétrograde jamais ; conserve le 'contesté' de l'utilisateur.",
        "Exécuter le remplissage", "En cours…",
        "{total} lien(s) promu(s) (→preuve: {evidence}, →établi: {established}).",
    ),
    "he": (
        "מילוי ביטחון קישורים",
        "קידום קישורים שכבר חצו סף (≥3 → עדות, ≥10 → מבוסס) אך לא עברו דרך חוקי הקידום האוטומטיים. הפעלה חד־פעמית; בטוח להפעיל שוב. לא מוריד; משמר 'שנוי במחלוקת' שנקבע על־ידי המשתמש.",
        "הפעל מילוי", "פועל…",
        "קודמו {total} קישור(ים) (→עדות: {evidence}, →מבוסס: {established}).",
    ),
    "hi": (
        "लिंक आत्मविश्वास बैक-फिल करें",
        "मौजूदा लिंक को प्रमोट करें जो ट्रेवर्सल थ्रेशोल्ड पार कर चुके हैं (≥3 → साक्ष्य, ≥10 → स्थापित) लेकिन ऑटो-प्रमोशन नियम से कभी नहीं गुजरे। एक बार; कई बार चलाना सुरक्षित। कभी डाउनग्रेड नहीं; उपयोगकर्ता-सेट 'विवादित' सुरक्षित।",
        "बैक-फिल चलाएँ", "चल रहा है…",
        "{total} लिंक प्रमोट हुए (→साक्ष्य: {evidence}, →स्थापित: {established}).",
    ),
    "ja": (
        "リンク信頼度の一括補完",
        "閾値を既に超えた既存リンク (≥3 → 証拠、≥10 → 確立) を昇格。自動昇格ルールをまだ通っていないものが対象。単発実行、複数回実行も安全。降格はせず、ユーザー設定の「異議あり」を保持。",
        "補完を実行", "実行中…",
        "{total}個のリンクを昇格 (→証拠: {evidence}, →確立: {established})。",
    ),
    "ko": (
        "링크 신뢰도 채우기",
        "이미 임계값을 넘은 기존 링크 (≥3 → 증거, ≥10 → 확립됨) 를 승격합니다. 자동 승격 규칙을 거치지 않은 것들 대상. 한 번 실행; 여러 번 실행해도 안전. 절대 강등하지 않으며 사용자 지정 '이의 제기됨'을 보존.",
        "채우기 실행", "실행 중…",
        "{total}개 링크 승격 (→증거: {evidence}, →확립됨: {established}).",
    ),
    "pt": (
        "Preencher confiança dos links",
        "Promove links existentes que já cruzaram um limite (≥3 → evidência, ≥10 → estabelecido) mas nunca passaram pela regra de auto-promoção. Disparo único; seguro rodar várias vezes. Nunca rebaixa; preserva 'contestado' do usuário.",
        "Executar preenchimento", "Executando…",
        "{total} link(s) promovido(s) (→evidência: {evidence}, →estabelecido: {established}).",
    ),
    "ru": (
        "Заполнить уверенность ссылок",
        "Повышает существующие ссылки, уже перешедшие порог (≥3 → свидетельство, ≥10 → подтверждено), но не прошедшие автоповышение. Однократный запуск; безопасно запускать многократно. Не понижает; сохраняет заданное пользователем «оспаривается».",
        "Запустить заполнение", "Выполняется…",
        "Повышено {total} ссылок (→свидетельство: {evidence}, →подтверждено: {established}).",
    ),
    "tr": (
        "Bağlantı güvenini doldur",
        "Eşiği geçmiş (≥3 → kanıt, ≥10 → yerleşik) ama otomatik yükseltme kuralından geçmemiş mevcut bağlantıları yükseltir. Tek atış; birden çok kez güvenle çalıştırılabilir. Asla düşürmez; kullanıcının 'itirazlı' seçimini korur.",
        "Doldurmayı çalıştır", "Çalışıyor…",
        "{total} bağlantı yükseltildi (→kanıt: {evidence}, →yerleşik: {established}).",
    ),
    "ur": (
        "لنک اعتماد بھرنا",
        "موجودہ لنکس جو پہلے ہی حد عبور کر چکے ہیں (≥3 → شواہد، ≥10 → قائم) لیکن خودکار ترقی سے نہیں گزرے، انہیں ترقی دیں۔ ایک بار؛ متعدد بار چلانا محفوظ۔ کبھی کم نہیں کرتا؛ صارف کا مقرر کردہ 'متنازع' محفوظ رہتا ہے۔",
        "بھرنا چلائیں", "چل رہا ہے…",
        "{total} لنک ترقی پائی (→شواہد: {evidence}، →قائم: {established})۔",
    ),
    "zh": (
        "补填链接置信度",
        "对已跨越阈值（≥3 → 证据，≥10 → 已确立）但从未经过自动提升规则的现有链接进行提升。一次性；可多次安全运行。从不降级；保留用户设置的「有争议」。",
        "运行补填", "运行中…",
        "已提升 {total} 条链接（→证据：{evidence}，→已确立：{established}）。",
    ),
}

root = Path(__file__).resolve().parents[2] / "src" / "lib" / "i18n"

for locale in CONFIDENCE.keys():
    p = root / f"{locale}.json"
    data = json.loads(p.read_text(encoding="utf-8"))

    # linkConfidence namespace (top-level).
    setc, hint, hyp, ev, est, con = CONFIDENCE[locale]
    data["linkConfidence"] = {
        "setConfidence": setc,
        "rightClickHint": hint,
        "hypothesis": hyp,
        "evidence": ev,
        "established": est,
        "contested": con,
    }

    # settings.appearance backfill keys.
    appearance = data.setdefault("settings", {}).setdefault("appearance", {})
    name, desc, btn, run, res = BACKFILL[locale]
    appearance["confidenceBackfill"] = name
    appearance["confidenceBackfillDesc"] = desc
    appearance["confidenceBackfillBtn"] = btn
    appearance["confidenceBackfillRunning"] = run
    appearance["confidenceBackfillResult"] = res

    p.write_text(json.dumps(data, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
    print(f"{locale}: linkConfidence + confidenceBackfill* written")

print("\nAll 15 locales updated.")
