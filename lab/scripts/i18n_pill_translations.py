"""One-shot locale updater for the Living Link pill-styles UI.

Adds the settings.appearance.pill* keys to every locale. Idempotent —
safe to re-run. Translations are UI-reasonable; natives welcome to
polish them in a follow-up pass.
"""
import json, os

TR = {
    'ar': {
        'livingLinkPills': 'شارات الروابط الحية',
        'livingLinkPillsDesc': 'تخصيص ألوان وشكل شارات أنواع الروابط ورقائق المرور التي تظهر في لوحتي الروابط الخلفية والروابط الصادرة.',
        'pillFill': 'التعبئة',
        'pillText': 'النص',
        'pillRadius': 'انحناء الحواف',
        'pillRadiusDesc': 'مدى تدوير حواف الشارة (0 = حاد، 20 = دائري بالكامل).',
        'pillHeight': 'ارتفاع الشارة',
        'pillHeightDesc': 'الحجم العمودي لكل شارة.',
        'pillWeight': 'سماكة النص',
        'pillWeightDesc': 'سماكة خط تسميات الشارة (400 = عادي، 700 = عريض، 900 = عريض جداً).',
        'resetPillStyles': 'إعادة تعيين أنماط الشارات إلى الافتراضي',
    },
    'de': {
        'livingLinkPills': 'Living Link Badges',
        'livingLinkPillsDesc': 'Passe die Farben und die Form der Link-Typ-Badges und Durchlauf-Chips an, die in den Backlinks- und Ausgehende-Links-Bereichen erscheinen.',
        'pillFill': 'Füllung',
        'pillText': 'Text',
        'pillRadius': 'Eckenradius',
        'pillRadiusDesc': 'Wie abgerundet die Badge-Ecken sind (0 = scharf, 20 = vollständig rund).',
        'pillHeight': 'Badge-Höhe',
        'pillHeightDesc': 'Vertikale Größe jedes Badges.',
        'pillWeight': 'Textgewicht',
        'pillWeightDesc': 'Schriftgewicht der Badge-Beschriftungen (400 = normal, 700 = fett, 900 = extrafett).',
        'resetPillStyles': 'Badge-Stile zurücksetzen',
    },
    'es': {
        'livingLinkPills': 'Etiquetas de Enlaces Vivos',
        'livingLinkPillsDesc': 'Personaliza los colores y la forma de las etiquetas de tipo de enlace y los chips de recorrido que aparecen en los paneles Retroenlaces y Enlaces Salientes.',
        'pillFill': 'Relleno',
        'pillText': 'Texto',
        'pillRadius': 'Radio de esquina',
        'pillRadiusDesc': 'Qué tan redondeadas están las esquinas (0 = afilado, 20 = totalmente redondo).',
        'pillHeight': 'Altura de etiqueta',
        'pillHeightDesc': 'Tamaño vertical de cada etiqueta.',
        'pillWeight': 'Grosor del texto',
        'pillWeightDesc': 'Grosor de la fuente (400 = normal, 700 = negrita, 900 = extra negrita).',
        'resetPillStyles': 'Restablecer estilos de etiqueta',
    },
    'fa': {
        'livingLinkPills': 'نشان‌های پیوند زنده',
        'livingLinkPillsDesc': 'سفارشی‌سازی رنگ‌ها و شکل نشان‌های نوع پیوند و تراشه‌های پیمایش در پنل‌های بک‌لینک و پیوندهای خروجی.',
        'pillFill': 'پرکننده',
        'pillText': 'متن',
        'pillRadius': 'شعاع گوشه',
        'pillRadiusDesc': 'میزان گردی گوشه‌های نشان (0 = تیز، 20 = کاملاً گرد).',
        'pillHeight': 'ارتفاع نشان',
        'pillHeightDesc': 'اندازه عمودی هر نشان.',
        'pillWeight': 'ضخامت متن',
        'pillWeightDesc': 'وزن فونت برچسب‌ها (400 = معمولی، 700 = ضخیم، 900 = خیلی ضخیم).',
        'resetPillStyles': 'بازنشانی سبک‌های نشان به پیش‌فرض',
    },
    'fr': {
        'livingLinkPills': 'Pastilles de Liens Vivants',
        'livingLinkPillsDesc': 'Personnalisez les couleurs et la forme des badges de type de lien et des puces de traversée qui apparaissent dans les panneaux Rétroliens et Liens Sortants.',
        'pillFill': 'Remplissage',
        'pillText': 'Texte',
        'pillRadius': 'Rayon des coins',
        'pillRadiusDesc': 'Arrondi des coins (0 = net, 20 = entièrement rond).',
        'pillHeight': 'Hauteur de pastille',
        'pillHeightDesc': 'Taille verticale de chaque pastille.',
        'pillWeight': 'Épaisseur du texte',
        'pillWeightDesc': 'Graisse de la police (400 = normal, 700 = gras, 900 = extra-gras).',
        'resetPillStyles': 'Réinitialiser les styles de pastille',
    },
    'he': {
        'livingLinkPills': 'תגיות קישור חי',
        'livingLinkPillsDesc': 'התאמה אישית של הצבעים והצורה של תגיות סוג הקישור וצ׳יפי המעבר המופיעים בלוחות קישורים נכנסים וקישורים יוצאים.',
        'pillFill': 'מילוי',
        'pillText': 'טקסט',
        'pillRadius': 'רדיוס פינה',
        'pillRadiusDesc': 'כמה מעוגלות הפינות (0 = חד, 20 = עגול לגמרי).',
        'pillHeight': 'גובה תגית',
        'pillHeightDesc': 'גודל אנכי של כל תגית.',
        'pillWeight': 'עובי טקסט',
        'pillWeightDesc': 'משקל פונט של תוויות (400 = רגיל, 700 = מודגש, 900 = מודגש במיוחד).',
        'resetPillStyles': 'אפס סגנונות תגית לברירת מחדל',
    },
    'hi': {
        'livingLinkPills': 'लिविंग लिंक बैज',
        'livingLinkPillsDesc': 'बैकलिंक्स और आउटगोइंग लिंक्स पैनल में दिखाई देने वाले लिंक-प्रकार बैज और ट्रैवर्सल चिप्स के रंगों और आकार को अनुकूलित करें।',
        'pillFill': 'भरना',
        'pillText': 'पाठ',
        'pillRadius': 'कोना त्रिज्या',
        'pillRadiusDesc': 'बैज के कोने कितने गोल हैं (0 = तीव्र, 20 = पूर्ण गोल)।',
        'pillHeight': 'बैज ऊंचाई',
        'pillHeightDesc': 'प्रत्येक बैज का लंबवत आकार।',
        'pillWeight': 'पाठ वजन',
        'pillWeightDesc': 'बैज लेबल का फ़ॉन्ट वजन (400 = सामान्य, 700 = बोल्ड, 900 = अतिरिक्त बोल्ड)।',
        'resetPillStyles': 'बैज शैलियाँ डिफ़ॉल्ट पर रीसेट करें',
    },
    'ja': {
        'livingLinkPills': 'リビングリンクバッジ',
        'livingLinkPillsDesc': 'バックリンクと発信リンクのパネルに表示されるリンクタイプバッジと移動チップの色と形状をカスタマイズします。',
        'pillFill': '塗りつぶし',
        'pillText': 'テキスト',
        'pillRadius': '角の半径',
        'pillRadiusDesc': 'バッジの角の丸み (0 = 鋭角、20 = 完全な丸)。',
        'pillHeight': 'バッジの高さ',
        'pillHeightDesc': '各バッジの縦方向のサイズ。',
        'pillWeight': 'テキストの太さ',
        'pillWeightDesc': 'バッジラベルのフォントウェイト (400 = 標準、700 = 太字、900 = 極太)。',
        'resetPillStyles': 'バッジスタイルを既定値にリセット',
    },
    'ko': {
        'livingLinkPills': '리빙 링크 배지',
        'livingLinkPillsDesc': '백링크 및 아웃고잉 링크 패널에 나타나는 링크 유형 배지 및 탐색 칩의 색상과 모양을 사용자 지정합니다.',
        'pillFill': '채우기',
        'pillText': '텍스트',
        'pillRadius': '모서리 반경',
        'pillRadiusDesc': '배지 모서리의 둥글기 (0 = 날카로움, 20 = 완전히 둥금).',
        'pillHeight': '배지 높이',
        'pillHeightDesc': '각 배지의 세로 크기.',
        'pillWeight': '텍스트 굵기',
        'pillWeightDesc': '배지 레이블의 글꼴 두께 (400 = 보통, 700 = 굵게, 900 = 매우 굵게).',
        'resetPillStyles': '배지 스타일을 기본값으로 재설정',
    },
    'pt': {
        'livingLinkPills': 'Emblemas de Links Vivos',
        'livingLinkPillsDesc': 'Personalize as cores e a forma dos emblemas de tipo de link e chips de travessia que aparecem nos painéis Backlinks e Links de Saída.',
        'pillFill': 'Preenchimento',
        'pillText': 'Texto',
        'pillRadius': 'Raio do canto',
        'pillRadiusDesc': 'Quão arredondados são os cantos (0 = afiado, 20 = totalmente redondo).',
        'pillHeight': 'Altura do emblema',
        'pillHeightDesc': 'Tamanho vertical de cada emblema.',
        'pillWeight': 'Espessura do texto',
        'pillWeightDesc': 'Peso da fonte (400 = normal, 700 = negrito, 900 = extra negrito).',
        'resetPillStyles': 'Redefinir estilos de emblema',
    },
    'ru': {
        'livingLinkPills': 'Значки живых ссылок',
        'livingLinkPillsDesc': 'Настройте цвета и форму значков типов ссылок и чипов переходов, отображаемых на панелях «Обратные ссылки» и «Исходящие ссылки».',
        'pillFill': 'Заливка',
        'pillText': 'Текст',
        'pillRadius': 'Радиус углов',
        'pillRadiusDesc': 'Насколько скруглены углы (0 = острые, 20 = полностью круглые).',
        'pillHeight': 'Высота значка',
        'pillHeightDesc': 'Вертикальный размер каждого значка.',
        'pillWeight': 'Толщина текста',
        'pillWeightDesc': 'Насыщенность шрифта (400 = обычный, 700 = жирный, 900 = сверхжирный).',
        'resetPillStyles': 'Сбросить стили значков',
    },
    'tr': {
        'livingLinkPills': 'Canlı Bağlantı Rozetleri',
        'livingLinkPillsDesc': 'Geri Bağlantılar ve Giden Bağlantılar panellerinde görünen bağlantı türü rozetlerinin ve geçiş yongalarının renklerini ve şeklini özelleştirin.',
        'pillFill': 'Dolgu',
        'pillText': 'Metin',
        'pillRadius': 'Köşe yarıçapı',
        'pillRadiusDesc': 'Rozet köşelerinin yuvarlaklığı (0 = keskin, 20 = tamamen yuvarlak).',
        'pillHeight': 'Rozet yüksekliği',
        'pillHeightDesc': 'Her rozetin dikey boyutu.',
        'pillWeight': 'Metin kalınlığı',
        'pillWeightDesc': 'Rozet etiketlerinin yazı tipi ağırlığı (400 = normal, 700 = kalın, 900 = çok kalın).',
        'resetPillStyles': 'Rozet stillerini varsayılana sıfırla',
    },
    'ur': {
        'livingLinkPills': 'لیونگ لنک بیجز',
        'livingLinkPillsDesc': 'بیک لنکس اور آؤٹ گوئنگ لنکس پینلز میں ظاہر ہونے والے لنک قسم کے بیجز اور ٹریورسل چپس کے رنگ اور شکل کو حسب ضرورت بنائیں۔',
        'pillFill': 'بھرنا',
        'pillText': 'متن',
        'pillRadius': 'کنارے کا رداس',
        'pillRadiusDesc': 'بیج کے کنارے کتنے گول ہیں (0 = تیز، 20 = مکمل گول)۔',
        'pillHeight': 'بیج کی اونچائی',
        'pillHeightDesc': 'ہر بیج کا عمودی سائز۔',
        'pillWeight': 'متن کا وزن',
        'pillWeightDesc': 'بیج لیبلز کا فونٹ وزن (400 = عام، 700 = بولڈ، 900 = اضافی بولڈ)۔',
        'resetPillStyles': 'بیج اسٹائلز کو پہلے سے طے شدہ پر ری سیٹ کریں',
    },
    'zh': {
        'livingLinkPills': '活动链接徽章',
        'livingLinkPillsDesc': '自定义反向链接和传出链接面板中显示的链接类型徽章和遍历标签的颜色和形状。',
        'pillFill': '填充',
        'pillText': '文字',
        'pillRadius': '圆角半径',
        'pillRadiusDesc': '徽章边角的圆度 (0 = 直角, 20 = 完全圆形)。',
        'pillHeight': '徽章高度',
        'pillHeightDesc': '每个徽章的纵向尺寸。',
        'pillWeight': '文字粗细',
        'pillWeightDesc': '徽章标签的字体粗细 (400 = 普通, 700 = 加粗, 900 = 特粗)。',
        'resetPillStyles': '重置徽章样式为默认值',
    },
}

ROOT = os.path.join(os.path.dirname(__file__), '..', '..', 'src', 'lib', 'i18n')
ROOT = os.path.abspath(ROOT)

for lang, keys in TR.items():
    path = os.path.join(ROOT, lang + '.json')
    with open(path, 'r', encoding='utf-8') as f:
        data = json.load(f)
    if 'settings' in data and 'appearance' in data['settings']:
        for k, v in keys.items():
            data['settings']['appearance'][k] = v
        with open(path, 'w', encoding='utf-8') as f:
            json.dump(data, f, ensure_ascii=False, indent=2)
            f.write('\n')
        print('updated', lang)
    else:
        print('SKIP (missing settings.appearance):', lang)
