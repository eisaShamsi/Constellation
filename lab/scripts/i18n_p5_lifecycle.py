#!/usr/bin/env python3
"""P5 slice 2: add settings.appearance.linkLifecycle* + days keys to all 15 locales."""
import json
from pathlib import Path

# Each locale: (heading, section-desc, toggle-label, toggle-desc, slider-label, slider-desc, days-word)
T = {
    "en": (
        "Living Link Lifecycle",
        "Links you haven't followed in a while drift down the Backlinks / Outgoing / Most-Traveled sort. The decay is a display concern only — the raw traversal counts in the database stay intact.",
        "Apply weight decay to link sorts",
        "When off, links sort by raw traversal count only (no recency weighting).",
        "Decay half-life",
        "Days after which an untouched link's effective weight halves. Lower = faster drop-off; higher = slower.",
        "days",
    ),
    "ar": (
        "دورة حياة الروابط الحيّة",
        "الروابط التي لم تتصفّحها منذ فترة تنزلق نحو أسفل قوائم الروابط الواردة / الصادرة / الأكثر ترددًا. هذا التأثير بصري فقط — أعداد التنقّل الخام في قاعدة البيانات تبقى كما هي.",
        "تطبيق انحلال الوزن على ترتيب الروابط",
        "عند الإيقاف، تُرتَّب الروابط بحسب عدد التنقّل الخام فقط (بدون مراعاة الحداثة).",
        "نصف عمر الانحلال",
        "عدد الأيام التي يتضاعف نصف الوزن الفعّال لرابط غير مستعمَل بعدها. كلّما قلّ، زاد الانحلال.",
        "يوم",
    ),
    "de": (
        "Living-Link-Lebenszyklus",
        "Links, die Sie länger nicht verfolgt haben, rutschen in Rückverweise / Ausgehende / Meist begangen nach unten. Der Zerfall ist nur eine Anzeigesache — die rohen Traversal-Zählungen in der Datenbank bleiben unangetastet.",
        "Gewichtszerfall auf Link-Sortierung anwenden",
        "Wenn aus, werden Links nur nach roher Traversal-Zahl sortiert (keine Aktualitätsgewichtung).",
        "Halbwertszeit des Zerfalls",
        "Tage, nach denen sich das effektive Gewicht eines unbenutzten Links halbiert. Niedriger = schnellerer Abfall.",
        "Tage",
    ),
    "es": (
        "Ciclo de vida del enlace vivo",
        "Los enlaces que no has seguido en un tiempo bajan en los listados de Retrovínculos / Salientes / Más transitados. Este decaimiento es solo visual — los recuentos brutos de travesía en la base de datos permanecen intactos.",
        "Aplicar decaimiento de peso al orden de los enlaces",
        "Cuando está desactivado, los enlaces se ordenan solo por recuento bruto de travesía (sin ponderar la frescura).",
        "Vida media del decaimiento",
        "Días tras los cuales el peso efectivo de un enlace sin uso se reduce a la mitad. Menor = caída más rápida.",
        "días",
    ),
    "fa": (
        "چرخهٔ عمر پیوند زنده",
        "پیوندهایی که مدتی دنبالشان نکرده‌اید در فهرست‌های پیوندهای ورودی / خروجی / پرترددترین به پایین می‌خزند. این افت فقط نمایشی است — شمارش خام در پایگاه داده دست‌نخورده می‌ماند.",
        "اعمال افت وزن روی ترتیب پیوندها",
        "وقتی خاموش است، پیوندها فقط بر اساس شمارش خام ترتیب می‌گیرند (بدون ملاحظهٔ تازگی).",
        "نیمه‌عمر افت",
        "روزی که پس از آن وزن مؤثر پیوند بی‌استفاده نصف می‌شود. کمتر = افت سریع‌تر.",
        "روز",
    ),
    "fr": (
        "Cycle de vie des liens vivants",
        "Les liens que vous n'avez pas suivis depuis un moment descendent dans les listes Rétroliens / Sortants / Plus empruntés. Cet affaiblissement n'est qu'un concept d'affichage — les décomptes bruts de traversée restent intacts.",
        "Appliquer la décroissance du poids au tri des liens",
        "Quand désactivé, les liens sont triés uniquement par décompte brut (aucune pondération par fraîcheur).",
        "Demi-vie de la décroissance",
        "Nombre de jours après lesquels le poids effectif d'un lien inutilisé est divisé par deux. Plus bas = chute plus rapide.",
        "jours",
    ),
    "he": (
        "מחזור חיי הקישור החי",
        "קישורים שלא עקבת אחריהם זמן-מה נדחקים מטה ברשימות הקישורים הנכנסים / היוצאים / הנפוצים ביותר. הדעיכה היא עניין תצוגה בלבד — מוני המעברים הגולמיים במסד הנתונים נשארים בשלמותם.",
        "החל דעיכת משקל על מיון קישורים",
        "כשמכובה, הקישורים ממוינים רק לפי מונה המעברים הגולמי (ללא שקלול רעננות).",
        "חצי-חיי הדעיכה",
        "מספר הימים שלאחריהם המשקל האפקטיבי של קישור בלתי-משומש נחצה. נמוך יותר = נפילה מהירה יותר.",
        "ימים",
    ),
    "hi": (
        "जीवित कड़ी का जीवन-चक्र",
        "जिन कड़ियों का आप कुछ समय से अनुसरण नहीं कर रहे, वे बैकलिंक्स / आउटगोइंग / सबसे अधिक यात्रा सूचियों में नीचे खिसक जाती हैं। यह क्षय केवल प्रदर्शन संबंधी है — डेटाबेस में कच्चे यात्रा आँकड़े अछूते रहते हैं।",
        "कड़ी क्रम पर भार क्षय लागू करें",
        "बंद होने पर कड़ियाँ केवल कच्ची यात्रा संख्या से क्रमित होती हैं (ताज़गी भार के बिना)।",
        "क्षय अर्ध-आयु",
        "वे दिन जिनके बाद अप्रयुक्त कड़ी का प्रभावी भार आधा हो जाता है। कम = तेज़ गिरावट।",
        "दिन",
    ),
    "ja": (
        "リビングリンクのライフサイクル",
        "しばらく辿っていないリンクは、被リンク / 発リンク / 最多通過の並びで下へ沈みます。これは表示上の扱いで、データベースの生のトラバース回数は維持されます。",
        "リンクの並びに重み減衰を適用",
        "オフのときは、リンクは生のトラバース回数のみで並びます（新しさは加味されません）。",
        "減衰の半減期",
        "未使用リンクの実効重みが半分になるまでの日数。小さいほど素早く沈みます。",
        "日",
    ),
    "ko": (
        "리빙 링크 라이프사이클",
        "한동안 따라가지 않은 링크는 역링크 / 나가는 링크 / 가장 많이 이동 목록에서 아래로 밀립니다. 이 감쇠는 표시 계층의 문제일 뿐, 데이터베이스의 원시 이동 횟수는 그대로 유지됩니다.",
        "링크 정렬에 가중치 감쇠 적용",
        "끄면 링크는 원시 이동 횟수만으로 정렬됩니다(최신성 가중 없음).",
        "감쇠 반감기",
        "사용되지 않은 링크의 유효 가중치가 절반이 되기까지의 일수. 낮을수록 빠르게 감쇠.",
        "일",
    ),
    "pt": (
        "Ciclo de vida da ligação viva",
        "Ligações que você não seguiu há algum tempo descem nas listas de Retroligações / Ligações de saída / Mais percorridas. Este decaimento é apenas visual — as contagens brutas de travessia no banco de dados permanecem intactas.",
        "Aplicar decaimento de peso à ordenação de ligações",
        "Quando desativado, as ligações são ordenadas apenas pela contagem bruta de travessia (sem ponderação por recência).",
        "Meia-vida do decaimento",
        "Dias após os quais o peso efetivo de uma ligação não usada cai pela metade. Menor = queda mais rápida.",
        "dias",
    ),
    "ru": (
        "Жизненный цикл живой ссылки",
        "Ссылки, по которым вы давно не переходили, опускаются в списках обратных / исходящих / наиболее пройденных. Это только визуальное затухание — сырые счётчики переходов в базе данных остаются нетронутыми.",
        "Применять затухание веса к сортировке ссылок",
        "Когда выключено, ссылки сортируются только по сырому счётчику переходов (без учёта свежести).",
        "Период полураспада",
        "Дни, по истечении которых эффективный вес неиспользуемой ссылки падает вдвое. Меньше = быстрее спад.",
        "дн.",
    ),
    "tr": (
        "Yaşayan Bağlantı Yaşam Döngüsü",
        "Bir süredir takip etmediğiniz bağlantılar Geri Bağlantılar / Dışa Bağlantılar / En Çok Geçilen listelerinde aşağı kayar. Bu sönümleme yalnızca görüntü katmanındadır — veritabanındaki ham geçiş sayıları bozulmaz.",
        "Bağlantı sıralamasında ağırlık sönümlemesi uygula",
        "Kapalıyken bağlantılar yalnızca ham geçiş sayısına göre sıralanır (tazelik ağırlıklandırılmaz).",
        "Sönümleme yarı ömrü",
        "Kullanılmayan bir bağlantının etkin ağırlığının yarıya düşmesi için geçen gün sayısı. Düşük = hızlı düşüş.",
        "gün",
    ),
    "ur": (
        "زندہ لنک کا چکرِ حیات",
        "جن لنکس کو آپ نے کچھ عرصے سے نہیں چلایا وہ بیک لنکس / باہر جانے والے / سب سے زیادہ چلے ہوئے کی فہرستوں میں نیچے سرکتے ہیں۔ یہ زوال صرف ڈسپلے کی بات ہے — ڈیٹا بیس میں خام گنتیاں برقرار رہتی ہیں۔",
        "لنکس کی ترتیب پر وزن کا زوال لاگو کریں",
        "بند ہونے پر لنکس صرف خام چال کی گنتی سے ترتیب پاتے ہیں (تازگی کا وزن نہیں)۔",
        "زوال کی نصف عمر",
        "وہ دن جن کے بعد غیر استعمال شدہ لنک کا مؤثر وزن آدھا رہ جاتا ہے۔ کم = تیز کمی۔",
        "دن",
    ),
    "zh": (
        "活链接生命周期",
        "一段时间未点开的链接会在反向链接 / 外向链接 / 最常通行列表中下沉。这只是显示层面的衰减——数据库中的原始通过计数保持不变。",
        "对链接排序应用权重衰减",
        "关闭时，链接仅按原始通过次数排序（不加入新鲜度权重）。",
        "衰减半衰期",
        "未使用链接的有效权重减半所需的天数。数值越小下降越快。",
        "天",
    ),
}

root = Path(__file__).resolve().parents[2] / "src" / "lib" / "i18n"
for locale, (head, desc, tg, tgd, sl, sld, days) in T.items():
    p = root / f"{locale}.json"
    data = json.loads(p.read_text(encoding="utf-8"))
    a = data.setdefault("settings", {}).setdefault("appearance", {})
    a["linkLifecycle"] = head
    a["linkLifecycleDesc"] = desc
    a["decayEnabled"] = tg
    a["decayEnabledDesc"] = tgd
    a["halfLifeDays"] = sl
    a["halfLifeDaysDesc"] = sld
    a["days"] = days
    p.write_text(json.dumps(data, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
    print(f"{locale}: linkLifecycle={head!r}")
