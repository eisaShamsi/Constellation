#!/usr/bin/env python3
"""Generate the Arabic Constellation Concept Paper v2.0 PDF with RTL support."""

from reportlab.lib.pagesizes import A4
from reportlab.lib.styles import ParagraphStyle
from reportlab.lib.units import mm
from reportlab.lib.colors import HexColor, white
from reportlab.lib.enums import TA_CENTER, TA_RIGHT, TA_JUSTIFY
from reportlab.platypus import (
    SimpleDocTemplate, Paragraph, Spacer, Table, TableStyle,
    PageBreak, HRFlowable
)
from reportlab.pdfgen import canvas
from reportlab.pdfbase import pdfmetrics
from reportlab.pdfbase.ttfonts import TTFont
import arabic_reshaper
from bidi.algorithm import get_display

# Colors
DARK_BLUE = HexColor('#1a2332')
GOLD = HexColor('#d4a574')
LIGHT_GOLD = HexColor('#f5e6d3')
MED_BLUE = HexColor('#2c3e50')
LIGHT_BLUE = HexColor('#eef2f7')
ACCENT = HexColor('#c9956b')
TEXT_COLOR = HexColor('#2c3e50')
LIGHT_TEXT = HexColor('#5a6a7a')

# Register Arabic font
pdfmetrics.registerFont(TTFont('Tahoma', 'C:/Windows/Fonts/tahoma.ttf'))
pdfmetrics.registerFont(TTFont('Tahoma-Bold', 'C:/Windows/Fonts/tahomabd.ttf'))

W, H = A4


def ar(text):
    """Reshape and reorder Arabic text for PDF rendering."""
    reshaped = arabic_reshaper.reshape(text)
    return get_display(reshaped)


def ar_bold(text):
    """Wrap Arabic text with bold tags after reshaping."""
    return ar(text)


class NumberedCanvas(canvas.Canvas):
    def __init__(self, *args, **kwargs):
        canvas.Canvas.__init__(self, *args, **kwargs)
        self._saved_page_states = []

    def showPage(self):
        self._saved_page_states.append(dict(self.__dict__))
        self._startPage()

    def save(self):
        num_pages = len(self._saved_page_states)
        for i, state in enumerate(self._saved_page_states):
            self.__dict__.update(state)
            if i > 0:
                self.setFont('Tahoma', 8)
                self.setFillColor(LIGHT_TEXT)
                bidi_text = ar(f'كونستلاشن — ورقة المفهوم  |  صفحة {i} من {num_pages - 1}')
                self.drawCentredString(W / 2, 20 * mm, bidi_text)
                self.setStrokeColor(GOLD)
                self.setLineWidth(0.5)
                self.line(30 * mm, H - 15 * mm, W - 30 * mm, H - 15 * mm)
            canvas.Canvas.showPage(self)
        canvas.Canvas.save(self)


def create_styles():
    s = {}
    s['title'] = ParagraphStyle(
        'Title', fontName='Tahoma-Bold', fontSize=36, textColor=DARK_BLUE,
        alignment=TA_CENTER, spaceAfter=12, leading=50
    )
    s['subtitle'] = ParagraphStyle(
        'Subtitle', fontName='Tahoma', fontSize=16, textColor=GOLD,
        alignment=TA_CENTER, spaceAfter=6, leading=24
    )
    s['h1'] = ParagraphStyle(
        'H1', fontName='Tahoma-Bold', fontSize=20, textColor=DARK_BLUE,
        spaceBefore=28, spaceAfter=14, leading=32, alignment=TA_RIGHT
    )
    s['h2'] = ParagraphStyle(
        'H2', fontName='Tahoma-Bold', fontSize=15, textColor=MED_BLUE,
        spaceBefore=20, spaceAfter=10, leading=24, alignment=TA_RIGHT
    )
    s['h3'] = ParagraphStyle(
        'H3', fontName='Tahoma-Bold', fontSize=12, textColor=ACCENT,
        spaceBefore=14, spaceAfter=8, leading=20, alignment=TA_RIGHT
    )
    s['body'] = ParagraphStyle(
        'Body', fontName='Tahoma', fontSize=10.5, textColor=TEXT_COLOR,
        alignment=TA_RIGHT, spaceAfter=8, leading=20
    )
    s['body_bold'] = ParagraphStyle(
        'BodyBold', fontName='Tahoma-Bold', fontSize=10.5, textColor=TEXT_COLOR,
        alignment=TA_RIGHT, spaceAfter=8, leading=20
    )
    s['quote'] = ParagraphStyle(
        'Quote', fontName='Tahoma', fontSize=11, textColor=ACCENT,
        alignment=TA_CENTER, spaceBefore=12, spaceAfter=12, leading=20,
        leftIndent=40, rightIndent=40
    )
    s['toc_item'] = ParagraphStyle(
        'TOC', fontName='Tahoma', fontSize=11, textColor=MED_BLUE,
        spaceBefore=4, spaceAfter=4, leading=20, alignment=TA_RIGHT,
        rightIndent=20
    )
    s['toc_title'] = ParagraphStyle(
        'TOCTitle', fontName='Tahoma-Bold', fontSize=16, textColor=DARK_BLUE,
        spaceBefore=10, spaceAfter=20, leading=26, alignment=TA_RIGHT
    )
    s['bullet'] = ParagraphStyle(
        'Bullet', fontName='Tahoma', fontSize=10.5, textColor=TEXT_COLOR,
        spaceAfter=4, leading=20, alignment=TA_RIGHT, rightIndent=12
    )
    s['table_cell'] = ParagraphStyle(
        'TableCell', fontName='Tahoma', fontSize=9, textColor=TEXT_COLOR,
        alignment=TA_RIGHT, leading=16
    )
    s['table_header'] = ParagraphStyle(
        'TableHeader', fontName='Tahoma-Bold', fontSize=9.5, textColor=GOLD,
        alignment=TA_RIGHT, leading=16
    )
    return s


def gold_hr():
    return HRFlowable(width="100%", thickness=1.5, color=GOLD, spaceBefore=6, spaceAfter=6)


def p(style, text):
    return Paragraph(ar(text), style)


def make_table(headers, rows, col_widths=None):
    data = [headers] + rows
    avail = W - 60 * mm
    if not col_widths:
        col_widths = [avail / len(headers)] * len(headers)

    t = Table(data, colWidths=col_widths, repeatRows=1)
    style_cmds = [
        ('BACKGROUND', (0, 0), (-1, 0), DARK_BLUE),
        ('TEXTCOLOR', (0, 0), (-1, 0), GOLD),
        ('FONTNAME', (0, 0), (-1, -1), 'Tahoma'),
        ('FONTSIZE', (0, 0), (-1, 0), 9.5),
        ('FONTSIZE', (0, 1), (-1, -1), 9),
        ('TEXTCOLOR', (0, 1), (-1, -1), TEXT_COLOR),
        ('ALIGN', (0, 0), (-1, -1), 'RIGHT'),
        ('VALIGN', (0, 0), (-1, -1), 'TOP'),
        ('TOPPADDING', (0, 0), (-1, -1), 6),
        ('BOTTOMPADDING', (0, 0), (-1, -1), 6),
        ('LEFTPADDING', (0, 0), (-1, -1), 8),
        ('RIGHTPADDING', (0, 0), (-1, -1), 8),
        ('GRID', (0, 0), (-1, -1), 0.5, HexColor('#dde3ea')),
        ('LINEBELOW', (0, 0), (-1, 0), 1.5, GOLD),
    ]
    for i in range(1, len(data)):
        bg = LIGHT_BLUE if i % 2 == 0 else white
        style_cmds.append(('BACKGROUND', (0, i), (-1, i), bg))
    t.setStyle(TableStyle(style_cmds))
    return t


def tc(style, text):
    """Table cell with Arabic reshaping."""
    return Paragraph(ar(text), style)


def build_pdf():
    import os
    output = os.path.join(os.path.dirname(__file__), "Constellation — Concept Paper (Arabic).pdf")

    doc = SimpleDocTemplate(
        output, pagesize=A4,
        leftMargin=30 * mm, rightMargin=30 * mm,
        topMargin=25 * mm, bottomMargin=25 * mm
    )

    S = create_styles()
    story = []
    avail = W - 60 * mm

    # ===== COVER PAGE =====
    story.append(Spacer(1, 80))
    story.append(HRFlowable(width="40%", thickness=2, color=GOLD, spaceBefore=0, spaceAfter=20))
    story.append(p(S['title'], 'كونستلاشن'))
    story.append(Spacer(1, 8))
    story.append(p(S['subtitle'], 'ورقة المفهوم'))
    story.append(Spacer(1, 4))
    story.append(p(S['subtitle'], 'الإصدار ٢.٠ — مارس ٢٠٢٦'))
    story.append(Spacer(1, 20))
    story.append(HRFlowable(width="40%", thickness=2, color=GOLD, spaceBefore=0, spaceAfter=40))

    tagline = ParagraphStyle(
        'Tagline', fontName='Tahoma', fontSize=13, textColor=GOLD,
        alignment=TA_CENTER, leading=24
    )
    story.append(p(tagline, 'منصة مستقلة لإدارة المعرفة'))
    story.append(Spacer(1, 30))

    desc = ParagraphStyle(
        'CoverDesc', fontName='Tahoma', fontSize=11, textColor=LIGHT_TEXT,
        alignment=TA_CENTER, leading=20
    )
    story.append(p(desc,
        'إدارة المعرفة متعددة الخزائن. قواعد بيانات وإدارة مهام وتقويم '
        'وذكاء اصطناعي وقوالب واستيراد مدمج. غير مدمّر. بدون إضافات.'
    ))
    story.append(Spacer(1, 40))

    dev_style = ParagraphStyle(
        'DevName', fontName='Tahoma', fontSize=10, textColor=LIGHT_TEXT,
        alignment=TA_CENTER, leading=16
    )
    story.append(Paragraph('Developed by Eisa ALSHAMSI', dev_style))
    story.append(p(dev_style, 'طور بواسطة: عيسى الشامسي'))
    story.append(Spacer(1, 20))

    footer = ParagraphStyle(
        'CoverFooter', fontName='Tahoma', fontSize=9, textColor=LIGHT_TEXT,
        alignment=TA_CENTER, leading=14
    )
    story.append(Paragraph('Tauri v2 + SvelteKit + Svelte 5 + Rust', footer))
    story.append(p(footer, 'مفتوح المصدر — رخصة MIT'))
    story.append(PageBreak())

    # ===== TABLE OF CONTENTS =====
    story.append(p(S['toc_title'], 'المحتويات'))
    story.append(gold_hr())
    toc_items = [
        ('١', 'ما هو كونستلاشن؟'),
        ('٢', 'المشكلة التي يحلها كونستلاشن'),
        ('٣', 'البنية الأساسية: نموذج الكون'),
        ('٤', 'ما يقدمه كونستلاشن'),
        ('٥', 'لمن صُمّم كونستلاشن؟'),
        ('٦', 'المزايا التقنية'),
        ('٧', 'معايير التحقق من التطوير'),
        ('٨', 'المشهد التنافسي'),
        ('٩', 'خارطة الطريق'),
        ('١٠', 'الخاتمة'),
    ]
    for num, title in toc_items:
        story.append(p(S['toc_item'], f'{title}  .{num}'))
    story.append(PageBreak())

    # ===== SECTION 1: ما هو كونستلاشن؟ =====
    story.append(p(S['h1'], '١. ما هو كونستلاشن؟'))
    story.append(gold_hr())
    story.append(p(S['body'],
        'كونستلاشن منصة سطح مكتب لإدارة المعرفة مصممة للأشخاص الذين يفكرون بملاحظات مترابطة. '
        'يخزّن كل شيء كملفات ماركداون قياسية على نظام الملفات المحلي — '
        'بدون حسابات سحابية، وبدون قيود على المورّد، وبدون اشتراك مطلوب.'
    ))
    story.append(p(S['body'],
        'يقدم كونستلاشن مفهوم الكون — مساحة عمل محمولة ومكتفية ذاتياً '
        'توحّد خزائن متعددة من ملفات الماركداون وقواعد بيانات منظمة ومساعدة ذكاء اصطناعي '
        'وإدارة مهام وعرض تقويم في تجربة واحدة متماسكة. '
        'حيث تمنحك الأدوات الأخرى دفتراً واحداً، يمنحك كونستلاشن نظاماً مترابطاً.'
    ))
    story.append(p(S['body_bold'],
        'الأساس التقني: Tauri v2 (واجهة خلفية بلغة Rust) + SvelteKit + Svelte 5. '
        'أداء أصلي، حجم ثنائي صغير، تشغيل كامل بدون إنترنت، بدون عبء Electron.'
    ))

    # ===== SECTION 2: المشكلة التي يحلها كونستلاشن =====
    story.append(p(S['h1'], '٢. المشكلة التي يحلها كونستلاشن'))
    story.append(gold_hr())
    story.append(p(S['body'],
        'إدارة المعرفة اليوم مجزأة. يواجه المستخدمون مجموعة مشتركة من المشكلات '
        'بغض النظر عن الأداة التي يستخدمونها:'
    ))

    t2h = [
        tc(S['table_header'], 'التكلفة'),
        tc(S['table_header'], 'ما يفعله المستخدمون اليوم'),
        tc(S['table_header'], 'المشكلة'),
    ]
    t2r = [
        [tc(S['table_cell'], 'روابط مفقودة، جهد مكرر'), tc(S['table_cell'], 'نسخ ولصق يدوي بين التطبيقات'), tc(S['table_cell'], 'ملاحظات متناثرة عبر أدوات متعددة')],
        [tc(S['table_cell'], 'تبديل السياق، لا بحث أو ربط عبر المشاريع'), tc(S['table_cell'], 'إغلاق مشروع لفتح آخر'), tc(S['table_cell'], 'دفتر/خزنة واحدة في كل مرة')],
        [tc(S['table_cell'], 'المهام منفصلة عن الملاحظات التي أنشأتها'), tc(S['table_cell'], 'تطبيق مهام منفصل (Todoist، Things، إلخ)'), tc(S['table_cell'], 'إدارة المهام مفقودة')],
        [tc(S['table_cell'], 'البيانات خارج نظام المعرفة'), tc(S['table_cell'], 'تصدير لجداول بيانات أو أدوات منفصلة'), tc(S['table_cell'], 'لا عروض قواعد بيانات')],
        [tc(S['table_cell'], 'انقطاع سير العمل، لا صيغ في الملاحظات'), tc(S['table_cell'], 'تحرير الجداول في جدول بيانات ثم لصقها'), tc(S['table_cell'], 'تحرير جداول جامد')],
        [tc(S['table_cell'], 'الملاحظات اليومية والمهام غير مرئية في مكان واحد'), tc(S['table_cell'], 'تطبيق تقويم منفصل'), tc(S['table_cell'], 'التقويم منفصل')],
        [tc(S['table_cell'], 'عدم اتساق، وقت إعداد ضائع'), tc(S['table_cell'], 'نسخ ولصق يدوي أو صيغ خاصة بالأداة'), tc(S['table_cell'], 'أنظمة القوالب')],
        [tc(S['table_cell'], 'لا تكامل مع ملاحظاتك الفعلية'), tc(S['table_cell'], 'أداة ذكاء اصطناعي منفصلة، نسخ السياق يدوياً'), tc(S['table_cell'], 'الذكاء الاصطناعي كفكرة لاحقة')],
        [tc(S['table_cell'], 'الاحتكاك يمنع الترحيل، البيانات تبقى محتجزة'), tc(S['table_cell'], 'تحويل يدوي أو سكريبتات خاصة بالصيغة'), tc(S['table_cell'], 'الاستيراد من أدوات أخرى')],
    ]
    story.append(make_table(t2h, t2r, [avail * 0.30, avail * 0.35, avail * 0.35]))
    story.append(Spacer(1, 8))

    story.append(p(S['body'],
        'بعض الأدوات تحل بعض هذه المشكلات. لا أداة تحلها جميعاً. ينتهي المستخدمون بتجميع '
        'رقع من التطبيقات والإضافات والحلول البديلة — ليصبحوا مُدمجي أنظمة بدلاً من عاملي معرفة.'
    ))
    story.append(p(S['body_bold'],
        'كونستلاشن يلغي ضريبة التكامل. كل قدرة مذكورة أعلاه مدمجة ومختبرة معاً وتُشحن كتجربة موحدة.'
    ))

    # Non-Destructive
    story.append(p(S['h2'], 'غير مدمّر بالتصميم'))
    story.append(p(S['body'],
        'كونستلاشن مبني على مبدأ أساسي: ملفاتك لا تُعدَّل أبداً بدون إجراء صريح منك. '
        'يقرأ مجلدات الماركداون الحالية كما هي تماماً — لا يحقن بيانات وصفية، ولا يعيد كتابة '
        'الـ frontmatter، ولا يغيّر هيكل المجلدات، ولا ينشئ ملفات تكوين مخفية داخل خزائنك. '
        'ملفات الماركداون الخاصة بك تبقى نقية ومحمولة ومتوافقة تماماً مع أي محرر نصوص أو أداة تقرأ ماركداون القياسي.'
    ))
    story.append(p(S['body'],
        'هذا يعني أن تبني كونستلاشن لا ينطوي على أي مخاطر. وجّهه إلى مجلدات ملاحظاتك الحالية، '
        'استكشف كل ميزة، وإن قررت يوماً استخدام أداة مختلفة — لم يتغير شيء. '
        'لا ترحيل، لا تحويل، ولا تنظيف مطلوب. كونستلاشن نافذة على معرفتك، وليس قفلاً عليها.'
    ))

    # ===== SECTION 3: البنية الأساسية =====
    story.append(p(S['h1'], '٣. البنية الأساسية: نموذج الكون'))
    story.append(gold_hr())
    story.append(p(S['body'],
        'المفهوم المعماري المحدد لكونستلاشن هو الكون — دليل محمول يملك جميع تكوين المستخدم وحالة مساحة العمل، '
        'منفصل عن ملاحظاتك.'
    ))
    story.append(p(S['h3'], 'لماذا هذا مهم'))
    bullets = [
        'قابلية النقل: انسخ دليل الكون إلى جهاز آخر وكل شيء ينتقل معك — الإعدادات، الإشارات المرجعية، مساحات العمل، تعريفات قواعد البيانات. الخزائن نفسها مجرد مجلدات ملفات ماركداون تعيش أينما تريد.',
        'تعدد الخزائن بالتصميم: يمكن للكون تسجيل أي عدد من الخزائن. البحث وعرض الرسم البياني ومسح المهام والروابط الخلفية وقواعد البيانات تعمل جميعها عبر حدود الخزائن أصلياً.',
        'التسلسل الهرمي: يمكن للأكوان أن تشير إلى أكوان فرعية، موروثةً خزائنها. كون قائد الفريق يمكن أن يتضمن كون الفريق المشترك بالإضافة إلى كونه الشخصي — مع منع المراجع الدائرية.',
        'بدون قيود: الكون ملفات JSON في مجلد. الخزائن ملفات ماركداون في مجلدات. غادر في أي وقت — ملاحظاتك ملفات قياسية يمكن لأي أداة قراءتها.',
    ]
    for b in bullets:
        story.append(p(S['bullet'], f'{b}  ●'))

    # ===== SECTION 4: ما يقدمه كونستلاشن =====
    story.append(p(S['h1'], '٤. ما يقدمه كونستلاشن'))
    story.append(gold_hr())

    story.append(p(S['h2'], '٤.١ القدرات التي تميّز كونستلاشن'))
    t4h = [
        tc(S['table_header'], 'التفاصيل'),
        tc(S['table_header'], 'القدرة'),
    ]
    t4r = [
        [tc(S['table_cell'], 'فتح وبحث وربط ورسم بياني عبر خزائن متعددة في نفس الوقت في نافذة واحدة'), tc(S['table_cell'], 'مساحة عمل متعددة الخزائن حقيقية')],
        [tc(S['table_cell'], 'كل التكوين ينتقل في دليل واحد محمول. انتقل بين الأجهزة ومساحة عملك بأكملها تتبعك'), tc(S['table_cell'], 'قابلية نقل الكون')],
        [tc(S['table_cell'], 'تكوين مساحات عمل هرمية — خزنة الفريق تغذي كونك الشخصي تلقائياً'), tc(S['table_cell'], 'أكوان فرعية')],
        [tc(S['table_cell'], 'رؤية أي ملاحظات في أي خزنة ترتبط بالملاحظة الحالية — ليس مقتصراً على خزنة واحدة'), tc(S['table_cell'], 'روابط خلفية عبر الخزائن')],
        [tc(S['table_cell'], 'رسم بياني واحد للمعرفة يعرض الروابط عبر كل خزائنك'), tc(S['table_cell'], 'رسم بياني عبر الخزائن')],
        [tc(S['table_cell'], 'عرض مهام شامل يجمع المهام من كل خزنة مع التصفية حسب الخزنة والأولوية وتاريخ الاستحقاق والبحث النصي'), tc(S['table_cell'], 'مسح مهام موحد')],
        [tc(S['table_cell'], 'عروض قواعد بيانات غير مدمرة مع أوضاع جدول وبطاقات وقوائم والتصفية والفرز والتحرير المباشر — بدون أدوات خارجية'), tc(S['table_cell'], 'قواعد بيانات مدمجة (Bases)')],
        [tc(S['table_cell'], 'SUM و AVG و COUNT و MIN و MAX مع مراجع الخلايا والنطاقات، تُقيَّم في مكانها داخل جداول الماركداون'), tc(S['table_cell'], 'صيغ الجداول')],
        [tc(S['table_cell'], 'OpenAI و Anthropic و Google Gemini و Ollama المحلي من واجهة واحدة، مع ٨ مهارات جاهزة — متكاملة مباشرة مع ملاحظاتك'), tc(S['table_cell'], 'ذكاء اصطناعي متعدد المزودين')],
        [tc(S['table_cell'], 'نافذة ثانوية مستقلة بالكامل توسّع مساحة عملك عبر شاشتين — حرر وتصفح واعرض الرسوم البيانية وأدر المهام جنباً إلى جنب بدون قيود. ليست مجرد لوحة مرجعية؛ مساحة عمل ثانية كاملة'), tc(S['table_cell'], 'شاشة ثانية')],
        [tc(S['table_cell'], 'الإنجليزية والعربية والألمانية والإسبانية والفرنسية والعبرية والهندية واليابانية والكورية والبرتغالية والروسية والتركية والأردية والصينية والفارسية — جميعها بدعم RTL كامل'), tc(S['table_cell'], '١٥ لغة عند الإطلاق')],
        [tc(S['table_cell'], 'تشفير الخزنة في حالة السكون، قفل الخمول بـ PIN، تخزين مفاتيح API في سلسلة مفاتيح النظام'), tc(S['table_cell'], 'طبقة أمان')],
        [tc(S['table_cell'], 'لا يعدّل ملفات الخزنة أبداً بدون إجراء صريح من المستخدم. تبنّي بدون مخاطر — جرّب كونستلاشن وانتقل بين الأدوات بحرية بدون أي أثر'), tc(S['table_cell'], 'وصول غير مدمّر للخزائن')],
    ]
    story.append(make_table(t4h, t4r, [avail * 0.65, avail * 0.35]))

    story.append(p(S['h2'], '٤.٢ كل شيء مدمج'))
    story.append(p(S['body'],
        'الميزات التي تحتاج الأدوات الأخرى إلى إضافات أو تطبيقات خارجية لتحقيقها تأتي مدمجة في كونستلاشن:'
    ))
    t42h = [
        tc(S['table_header'], 'كونستلاشن (مدمج)'),
        tc(S['table_header'], 'كيف يتعامل الآخرون'),
        tc(S['table_header'], 'الميزة'),
    ]
    t42r = [
        [tc(S['table_cell'], 'محلل استعلامات أصلي (TABLE، LIST، TASK، CALENDAR)'), tc(S['table_cell'], 'إضافات أو سكريبتات خارجية'), tc(S['table_cell'], 'الاستعلامات المنظمة')],
        [tc(S['table_cell'], 'مسح شامل للخزنة، تبديل، تواريخ استحقاق، أولوية، وسوم'), tc(S['table_cell'], 'تطبيقات مهام منفصلة أو إضافات'), tc(S['table_cell'], 'إدارة المهام')],
        [tc(S['table_cell'], 'عرض شهري مع نقاط الملاحظات/المهام، إنشاء ملاحظة يومية'), tc(S['table_cell'], 'إضافات تقويم منفصلة'), tc(S['table_cell'], 'شريط التقويم')],
        [tc(S['table_cell'], 'عمليات صفوف/أعمدة، فرز، نقل، صيغ'), tc(S['table_cell'], 'جداول ماركداون أساسية أو جداول بيانات'), tc(S['table_cell'], 'جداول متقدمة')],
        [tc(S['table_cell'], 'متغيرات القوالب (التاريخ، الوقت، العنوان، المجلد، الخزنة، المؤشر)'), tc(S['table_cell'], 'نسخ ولصق يدوي أو صيغ خاصة بالأداة'), tc(S['table_cell'], 'القوالب')],
        [tc(S['table_cell'], '٧ صيغ: مجلدات MD، Notion، Bear، Evernote، HTML، CSV، نص عادي'), tc(S['table_cell'], 'سكريبتات تحويل يدوية'), tc(S['table_cell'], 'استيراد الملاحظات')],
        [tc(S['table_cell'], 'محسّنة مع دعم عبر الخزائن والإشارات غير المرتبطة'), tc(S['table_cell'], 'أساسية أو تعتمد على إضافات'), tc(S['table_cell'], 'لوحة الروابط الخلفية')],
        [tc(S['table_cell'], 'عُقد عبر الخزائن، تحكم بالقوى، تجميع'), tc(S['table_cell'], 'خزنة واحدة فقط في معظم الأدوات'), tc(S['table_cell'], 'عرض الرسم البياني')],
        [tc(S['table_cell'], 'تحليل تكرار الوسوم، تجميع شامل للخزائن'), tc(S['table_cell'], 'تطبيقات أساسية'), tc(S['table_cell'], 'متصفح الوسوم')],
    ]
    story.append(make_table(t42h, t42r, [avail * 0.38, avail * 0.30, avail * 0.32]))

    # 4.3 Import From Anywhere
    story.append(p(S['h2'], '٤.٣ استيراد من أي مكان'))
    story.append(p(S['body'],
        'المستورد المدمج في كونستلاشن يدعم الترحيل من:'
    ))
    t43h = [
        tc(S['table_header'], 'ما يتم استيراده'),
        tc(S['table_header'], 'المصدر'),
    ]
    t43r = [
        [tc(S['table_cell'], 'تسجيل مباشر كخزنة — لا تحويل مطلوب'), tc(S['table_cell'], 'مجلدات ماركداون')],
        [tc(S['table_cell'], 'تنظيف معرّفات Hex، تحويل الروابط الداخلية إلى وصلات ويكي'), tc(S['table_cell'], 'تصدير Notion')],
        [tc(S['table_cell'], 'تحويل صيغة Bear إلى ماركداون قياسي'), tc(S['table_cell'], 'ملاحظات Bear')],
        [tc(S['table_cell'], 'تحويل ENML إلى ماركداون، حفظ الوسوم والتواريخ كـ frontmatter'), tc(S['table_cell'], 'Evernote (.enex)')],
        [tc(S['table_cell'], 'تحويل إلى ماركداون نظيف'), tc(S['table_cell'], 'ملفات HTML')],
        [tc(S['table_cell'], 'كل صف يصبح ملاحظة مع خصائص frontmatter'), tc(S['table_cell'], 'ملفات CSV')],
        [tc(S['table_cell'], 'استيراد مباشر مع امتداد ماركداون'), tc(S['table_cell'], 'ملفات نص عادي')],
    ]
    story.append(make_table(t43h, t43r, [avail * 0.60, avail * 0.40]))
    story.append(p(S['body'],
        'ملاحظاتك الحالية من أي أداة تصبح مواطنة من الدرجة الأولى في كونستلاشن بدون فقدان البنية أو البيانات الوصفية.'
    ))

    # 4.4 What Constellation Does Not Do (Yet)
    story.append(p(S['h2'], '٤.٤ ما لا يفعله كونستلاشن (بعد)'))
    story.append(p(S['body'], 'الشفافية مهمة. هذه القدرات غير متوفرة حالياً في كونستلاشن:'))
    t44h = [
        tc(S['table_header'], 'الحالة'),
        tc(S['table_header'], 'الميزة'),
    ]
    t44r = [
        [tc(S['table_cell'], 'ليس بعد — سطح المكتب فقط (Windows، macOS، Linux)'), tc(S['table_cell'], 'تطبيقات الجوال (iOS/Android)')],
        [tc(S['table_cell'], 'غير مدمج — استخدم Git أو Syncthing أو أي حل مزامنة ملفات'), tc(S['table_cell'], 'المزامنة السحابية')],
        [tc(S['table_cell'], 'ليس بعد'), tc(S['table_cell'], 'اللوحة اللانهائية / السبورة')],
        [tc(S['table_cell'], 'ليس بعد'), tc(S['table_cell'], 'توضيح PDF')],
        [tc(S['table_cell'], 'ليس بعد'), tc(S['table_cell'], 'التسجيل الصوتي')],
        [tc(S['table_cell'], 'ليس بعد'), tc(S['table_cell'], 'واجهة برمجة إضافات الطرف الثالث')],
    ]
    story.append(make_table(t44h, t44r, [avail * 0.55, avail * 0.45]))

    # ===== SECTION 5: لمن صُمّم كونستلاشن؟ =====
    story.append(p(S['h1'], '٥. لمن صُمّم كونستلاشن؟'))
    story.append(gold_hr())

    personas = [
        ('٥.١ المحترف متعدد المشاريع',
         'مستشار أو باحث أو عامل معرفة يحتفظ بمجموعات ملاحظات منفصلة لعملاء ومشاريع ومجالات حياتية مختلفة.',
         'يجب إغلاق مشروع لفتح آخر. لا يمكن البحث عبر المجموعات. لا يمكن رؤية الروابط بين ملاحظات مشروع عميل وملاحظات بحث في مجلد منفصل.',
         'سجّل كل الخزائن في كون واحد. بحث ورسم بياني ومسح مهام وربط عبر الجميع في نفس الوقت.'),
        ('٥.٢ المستخدم المتقدم المتعب من الأدوات',
         'مستخدم متقدم يشغّل تطبيقات وإضافات متعددة يقضي وقتاً كبيراً في إدارة التحديثات وحل التعارضات وتصحيح الأعطال.',
         'كل تحديث لأداة يمثل مخاطرة. إدارة المهام وقواعد البيانات والقوالب والتقويم والذكاء الاصطناعي كلها أنظمة منفصلة يصنعها فرق مختلفة بجداول زمنية مختلفة.',
         'الكل مدمج ومختبر معاً ويُحدَّث كوحدة واحدة. صفر إدارة إضافات.'),
        ('٥.٣ عامل المعرفة العربي / RTL',
         'مستخدم يعمل أساساً بالعربية أو العبرية أو الفارسية أو الأردية ويحتاج نظام تدوين ملاحظات يتعامل مع RTL كأولوية من الدرجة الأولى.',
         'دعم RTL غير متسق في معظم الأدوات. المحررات تفترض LTR. مفاتيح التاريخ والقوائم لا تتعرف على المكافئات العربية. عناصر الواجهة تتعطل في التخطيطات المعكوسة.',
         '١٥ لغة بما فيها ٤ لغات RTL. كشف مفاتيح الخصائص العربية (التاريخ، القوائم، مربعات الاختيار). انعكاس واجهة كامل. جداول ونماذج ومحررات وتقويم متوافقة مع RTL.'),
        ('٥.٤ قائد الفريق أو باني المنظمة',
         'مدير أو قائد فريق يريد مشاركة قاعدة معرفية مع أعضاء الفريق مع الحفاظ على ملاحظات شخصية بشكل منفصل.',
         'لا مفهوم لتكوين مساحات العمل في معظم الأدوات. مجموعات الملاحظات المشتركة تحتاج إعداداً يدوياً لكل شخص.',
         'أنشئ كوناً للفريق مع خزائن مشتركة. كل عضو يضيف كون الفريق ككون فرعي لكونه الشخصي. خزائن الفريق تظهر تلقائياً بجانب الخزائن الشخصية.'),
        ('٥.٥ الباحث المعزز بالذكاء الاصطناعي',
         'باحث أو طالب يريد مساعدة ذكاء اصطناعي مدمجة مباشرة في سير عمل تدوين ملاحظاته — تلخيص، أسئلة وأجوبة، مساعدة كتابية، ترجمة.',
         'يجب استخدام أداة ذكاء اصطناعي منفصلة، ونسخ السياق يدوياً، ولصق النتائج. أو تثبيت إضافات AI متنافسة بواجهات غير متسقة وإدارة مفاتيح API منفصلة.',
         'لوحة إعدادات AI واحدة. أربعة خيارات مزودين (بما فيهم Ollama المحلي للخصوصية). ثماني مهارات جاهزة. مفاتيح API مخزنة في سلسلة مفاتيح النظام، وليس في ملفات تكوين نصية.'),
        ('٥.٦ المستخدم المهتم بالأمان',
         'محترف يتعامل مع ملاحظات حساسة (قانونية، طبية، مالية، شخصية) يحتاج تشفيراً وتحكماً في الوصول.',
         'معظم تطبيقات الملاحظات لا تقدم تشفيراً مدمجاً، ولا قفل خمول، وتخزن مفاتيح API في ملفات تكوين نصية عادية.',
         'تشفير الخزنة في حالة السكون، قفل الخمول بـ PIN، تخزين مفاتيح API في سلسلة مفاتيح النظام.'),
        ('٥.٧ المستخدم المهاجر',
         'شخص ينتقل من Notion أو Evernote أو Bear أو أداة أخرى يريد امتلاك بياناته محلياً بدون فقدان سنوات من الملاحظات المتراكمة.',
         'الترحيل مؤلم. صيغ التصدير غير متسقة. الروابط الداخلية تنكسر. البيانات الوصفية تضيع. كثير من المستخدمين يبقون محتجزين لأن تكاليف التبديل مرتفعة جداً.',
         'المستورد المدمج يتعامل مع ٧ صيغ. معرّفات Notion Hex تُنظَّف، الروابط تُحوَّل إلى وصلات ويكي، ENML من Evernote يصبح ماركداون مع frontmatter. ترحيل بنقرة واحدة، صفر فقدان بيانات.'),
    ]
    for title, profile, pain, answer in personas:
        story.append(p(S['h2'], title))
        story.append(p(S['body'], f'الملف الشخصي: {profile}'))
        story.append(p(S['body'], f'المعاناة اليوم: {pain}'))
        story.append(p(S['body'], f'حل كونستلاشن: {answer}'))

    # ===== SECTION 6: المزايا التقنية =====
    story.append(p(S['h1'], '٦. المزايا التقنية'))
    story.append(gold_hr())
    techs = [
        ('٦.١ الأداء',
         'الواجهة الخلفية بلغة Rust في كونستلاشن تنفذ عمليات الملفات ومسح الروابط واستخراج المهام واستعلامات قواعد البيانات '
         'بسرعة أصلية. العمليات الثقيلة — مسح المهام على مستوى الخزنة والاستعلامات المنظمة وبناء الرسم البياني — '
         'تُنفَّذ في عملية Rust وتُعيد نتائج منظمة للواجهة الأمامية. المحرر لا يتنافس أبداً مع المعالجة في الخلفية على الموارد.'),
        ('٦.٢ حجم الثنائي واستخدام الموارد',
         'Tauri v2 يستخدم عرض الويب الأصلي للنظام بدلاً من تجميع Chromium. '
         'النتيجة حجم ثنائي أصغر بكثير واستهلاك ذاكرة أقل مقارنة بالبدائل المبنية على Electron.'),
        ('٦.٣ نموذج الأمان',
         'واجهة Tauri الخلفية بلغة Rust توفر حدود أمان طبيعية. '
         'الوصول لنظام الملفات يتم من خلال أوامر Tauri صريحة — الواجهة الأمامية لا تستطيع الوصول لملفات عشوائية. '
         'منع اجتياز المسار مفروض في طبقة Rust (فحوصات القننة على جميع عمليات الملفات).'),
        ('٦.٤ سيادة البيانات',
         'جميع البيانات على نظام ملفات المستخدم بصيغ قياسية: '
         'ملاحظات كملفات ماركداون مع YAML frontmatter، قواعد بيانات كملفات JSON بامتداد .base، '
         'التكوين كملفات JSON في دليل الكون، والمرفقات كملفات صور/PDF قياسية في مجلدات الخزنة. '
         'لا تتبع. لا اعتماد على السحابة. لا حساب مطلوب.'),
    ]
    for title, body in techs:
        story.append(p(S['h2'], title))
        story.append(p(S['body'], body))

    # ===== SECTION 7: معايير التحقق من التطوير =====
    story.append(p(S['h1'], '٧. معايير التحقق من التطوير'))
    story.append(gold_hr())

    story.append(p(S['h2'], '٧.١ الوعد الأساسي: "ملاحظاتك، على طريقتك"'))
    v1h = [tc(S['table_header'], 'النتيجة المتوقعة'), tc(S['table_header'], 'الاختبار')]
    v1r = [
        [tc(S['table_cell'], 'كل الملاحظات مرئية، frontmatter مُحلَّل، الروابط محلولة'), tc(S['table_cell'], 'فتح أي مجلد من ملفات ماركداون')],
        [tc(S['table_cell'], 'الملف على القرص يُحدَّث، قابل للقراءة بأي أداة ماركداون'), tc(S['table_cell'], 'تحرير ملاحظة وحفظها')],
        [tc(S['table_cell'], 'YAML frontmatter صحيح، صيغة قياسية'), tc(S['table_cell'], 'إنشاء ملاحظة مع frontmatter')],
        [tc(S['table_cell'], 'حل صحيح عبر الملفات والمجلدات'), tc(S['table_cell'], 'حل وصلات الويكي [[wikilinks]]')],
        [tc(S['table_cell'], 'عرض غني لصيغ ماركداون الموسعة'), tc(S['table_cell'], 'عرض التنبيهات والتمييز والرياضيات والمخططات')],
    ]
    story.append(make_table(v1h, v1r, [avail * 0.55, avail * 0.45]))

    story.append(p(S['h2'], '٧.٢ وعد تعدد الخزائن: "كون من المعرفة"'))
    v2h = [tc(S['table_header'], 'النتيجة المتوقعة'), tc(S['table_header'], 'الاختبار')]
    v2r = [
        [tc(S['table_cell'], 'الكل يظهر في مستكشف الملفات بألوان مميزة'), tc(S['table_cell'], 'تسجيل ٣+ خزائن')],
        [tc(S['table_cell'], 'نتائج من كل الخزائن، مصنفة حسب المصدر'), tc(S['table_cell'], 'البحث عبر الخزائن')],
        [tc(S['table_cell'], 'عُقد من كل الخزائن، حواف عبر الخزائن مرئية'), tc(S['table_cell'], 'الرسم البياني عبر الخزائن')],
        [tc(S['table_cell'], 'ملاحظة في الخزنة أ تعرض روابط خلفية من الخزنة ب'), tc(S['table_cell'], 'الروابط الخلفية عبر الخزائن')],
        [tc(S['table_cell'], 'عرض المهام الشامل يجمع كل الخزائن'), tc(S['table_cell'], 'المهام عبر الخزائن')],
    ]
    story.append(make_table(v2h, v2r, [avail * 0.55, avail * 0.45]))

    story.append(p(S['h2'], '٧.٣ وعد الكل في واحد: "كل شيء مدمج"'))
    v3h = [tc(S['table_header'], 'النتيجة المتوقعة'), tc(S['table_header'], 'الاختبار')]
    v3r = [
        [tc(S['table_cell'], 'استعلامات TABLE و LIST و TASK تعرض النتائج'), tc(S['table_cell'], 'استعلام منظم في ملاحظة')],
        [tc(S['table_cell'], 'التبديل في الشريط الجانبي يحدّث الملف على القرص'), tc(S['table_cell'], 'تبديل مربع اختيار المهمة')],
        [tc(S['table_cell'], 'الأيام ذات الملاحظات/المهام تعرض مؤشرات بصرية'), tc(S['table_cell'], 'مؤشرات نقاط التقويم')],
        [tc(S['table_cell'], '=SUM(A1:A5) تحسب بشكل صحيح'), tc(S['table_cell'], 'تقييم صيغ الجداول')],
        [tc(S['table_cell'], 'المتغيرات تُستبدل بالتاريخ والوقت والعنوان الحالي'), tc(S['table_cell'], 'إدراج القالب')],
        [tc(S['table_cell'], 'معرّفات Hex تُزال، الروابط تُحوَّل إلى وصلات ويكي'), tc(S['table_cell'], 'الاستيراد من تصدير Notion')],
    ]
    story.append(make_table(v3h, v3r, [avail * 0.55, avail * 0.45]))

    story.append(p(S['h2'], '٧.٤ وعد تجربة المستخدم: "يعمل للجميع"'))
    v4h = [tc(S['table_header'], 'النتيجة المتوقعة'), tc(S['table_header'], 'الاختبار')]
    v4r = [
        [tc(S['table_cell'], 'واجهة كاملة بالعربية، تخطيط RTL، شريط جانبي معكوس'), tc(S['table_cell'], 'التبديل إلى العربية')],
        [tc(S['table_cell'], 'مفاتيح التاريخ/القائمة/مربع الاختيار مكتشفة بشكل صحيح'), tc(S['table_cell'], 'إنشاء ملاحظة بخصائص عربية')],
        [tc(S['table_cell'], 'كل الإعدادات والإشارات المرجعية ومساحات العمل مُستعادة'), tc(S['table_cell'], 'فتح التطبيق على جهاز جديد بنسخة الكون')],
        [tc(S['table_cell'], 'الملاحظات غير قابلة للوصول حتى إدخال PIN صحيح'), tc(S['table_cell'], 'قفل التطبيق، إدخال PIN')],
    ]
    story.append(make_table(v4h, v4r, [avail * 0.55, avail * 0.45]))

    # ===== SECTION 8: المشهد التنافسي =====
    story.append(p(S['h1'], '٨. المشهد التنافسي'))
    story.append(gold_hr())
    story.append(p(S['body'],
        'كونستلاشن يشغل موقعاً فريداً في مجال إدارة المعرفة: محلي أولاً، متعدد الخزائن، كل شيء مدمج، ومتعدد اللغات.'
    ))

    t8h = [
        tc(S['table_header'], 'Bear'),
        tc(S['table_header'], 'Roam'),
        tc(S['table_header'], 'Logseq'),
        tc(S['table_header'], 'Notion'),
        tc(S['table_header'], 'Obsidian'),
        tc(S['table_header'], 'كونستلاشن'),
        tc(S['table_header'], 'البُعد'),
    ]
    t8r = [
        [tc(S['table_cell'], 'iCloud'), tc(S['table_cell'], 'سحابي'), tc(S['table_cell'], 'محلي'), tc(S['table_cell'], 'سحابي'), tc(S['table_cell'], 'محلي'), tc(S['table_cell'], 'ملفات محلية'), tc(S['table_cell'], 'ملكية البيانات')],
        [tc(S['table_cell'], 'كامل'), tc(S['table_cell'], 'لا'), tc(S['table_cell'], 'كامل'), tc(S['table_cell'], 'محدود'), tc(S['table_cell'], 'كامل'), tc(S['table_cell'], 'كامل'), tc(S['table_cell'], 'بدون إنترنت')],
        [tc(S['table_cell'], 'ملكي'), tc(S['table_cell'], 'ملكي'), tc(S['table_cell'], 'MD/EDN'), tc(S['table_cell'], 'ملكي'), tc(S['table_cell'], 'MD قياسي'), tc(S['table_cell'], 'MD قياسي'), tc(S['table_cell'], 'صيغة الملفات')],
        [tc(S['table_cell'], 'لا'), tc(S['table_cell'], 'واحد'), tc(S['table_cell'], 'واحد'), tc(S['table_cell'], 'لا (مساحات)'), tc(S['table_cell'], 'نافذة لكل خزنة'), tc(S['table_cell'], 'أصلي (الكون)'), tc(S['table_cell'], 'تعدد الخزائن')],
        [tc(S['table_cell'], 'لا'), tc(S['table_cell'], 'لا'), tc(S['table_cell'], 'لا'), tc(S['table_cell'], 'لا'), tc(S['table_cell'], 'لا'), tc(S['table_cell'], 'نعم'), tc(S['table_cell'], 'بحث عبر الخزائن')],
        [tc(S['table_cell'], 'لا'), tc(S['table_cell'], 'لا'), tc(S['table_cell'], 'لا'), tc(S['table_cell'], 'لا'), tc(S['table_cell'], 'لا'), tc(S['table_cell'], 'نعم'), tc(S['table_cell'], 'رسم بياني عبر الخزائن')],
        [tc(S['table_cell'], 'لا'), tc(S['table_cell'], 'استعلامات'), tc(S['table_cell'], 'استعلامات (محدودة)'), tc(S['table_cell'], 'أصلي'), tc(S['table_cell'], 'يحتاج إضافة'), tc(S['table_cell'], 'مدمج (Bases)'), tc(S['table_cell'], 'عروض قواعد البيانات')],
        [tc(S['table_cell'], 'لا'), tc(S['table_cell'], 'أساسي'), tc(S['table_cell'], 'يحتاج إضافة'), tc(S['table_cell'], 'أساسي'), tc(S['table_cell'], 'يحتاج إضافة'), tc(S['table_cell'], 'مدمج'), tc(S['table_cell'], 'إدارة المهام')],
        [tc(S['table_cell'], 'لا'), tc(S['table_cell'], 'يحتاج إضافة'), tc(S['table_cell'], 'يحتاج إضافة'), tc(S['table_cell'], 'مدمج (مزود واحد)'), tc(S['table_cell'], 'يحتاج إضافة'), tc(S['table_cell'], 'مدمج (٤ مزودين)'), tc(S['table_cell'], 'تكامل الذكاء الاصطناعي')],
        [tc(S['table_cell'], 'لا'), tc(S['table_cell'], 'لا'), tc(S['table_cell'], 'لا'), tc(S['table_cell'], 'محدود'), tc(S['table_cell'], 'يحتاج إضافة'), tc(S['table_cell'], 'مدمج'), tc(S['table_cell'], 'صيغ الجداول')],
        [tc(S['table_cell'], 'محدود'), tc(S['table_cell'], 'محدود'), tc(S['table_cell'], 'محدود'), tc(S['table_cell'], 'مدمج (محدود)'), tc(S['table_cell'], 'يحتاج إضافة'), tc(S['table_cell'], '٧ صيغ مدمجة'), tc(S['table_cell'], 'مصادر الاستيراد')],
        [tc(S['table_cell'], 'لا'), tc(S['table_cell'], 'محدود'), tc(S['table_cell'], 'محدود'), tc(S['table_cell'], 'محدود'), tc(S['table_cell'], 'جهد مجتمعي'), tc(S['table_cell'], '١٥ لغة، ٤ RTL'), tc(S['table_cell'], 'العربية / RTL')],
        [tc(S['table_cell'], 'اشتراك'), tc(S['table_cell'], 'اشتراك'), tc(S['table_cell'], 'مجاني'), tc(S['table_cell'], 'Freemium + اشتراك'), tc(S['table_cell'], 'Freemium'), tc(S['table_cell'], 'مجاني / مفتوح المصدر'), tc(S['table_cell'], 'التسعير')],
        [tc(S['table_cell'], 'أصلي (Apple)'), tc(S['table_cell'], 'تطبيق ويب'), tc(S['table_cell'], 'Electron'), tc(S['table_cell'], 'تطبيق ويب'), tc(S['table_cell'], 'Electron'), tc(S['table_cell'], 'Tauri (Rust + عرض ويب أصلي)'), tc(S['table_cell'], 'البنية')],
    ]
    col_w = avail / 7
    story.append(make_table(t8h, t8r, [col_w] * 7))

    story.append(p(S['h2'], 'موقع كونستلاشن'))
    story.append(p(S['body'],
        'كونستلاشن لا ينافس بكونه "نسخة أفضل من X". ينافس بكونه منصة إدارة معرفة متكاملة '
        'تلغي الحاجة لتجميع حزمة من الأدوات. المستخدمون القادمون من أي أداة — أو من لا أداة على الإطلاق — '
        'يمكنهم البدء مع كونستلاشن والحصول على كل ما يحتاجونه من اليوم الأول.'
    ))
    story.append(p(S['body'],
        'لمستخدمي الأدوات الحالية القائمة على ماركداون، الانتقال سلس: وجّه كونستلاشن إلى مجلداتك الحالية وكل شيء يعمل. '
        'لمستخدمي الأدوات الملكية، المستورد المدمج يتولى التحويل.'
    ))

    # ===== SECTION 9: خارطة الطريق =====
    story.append(p(S['h1'], '٩. خارطة الطريق'))
    story.append(gold_hr())

    story.append(p(S['h2'], 'أولوية عالية (تعزيز المميزات الأساسية)'))
    for item in [
        'صقل تجربة تعدد الخزائن — نقل/نسخ عبر الخزائن، إعدادات مخصصة لكل خزنة',
        'أداء قواعد البيانات على نطاق واسع — معالجة +١٠٠٠٠ ملاحظة بكفاءة',
        'توسيع مهارات الذكاء الاصطناعي — منشئ مهارات مخصص، أسئلة وأجوبة واعية بالسياق',
        'تطبيق جوال مرافق — متصفح خزائن للقراءة فقط لـ iOS/Android',
    ]:
        story.append(p(S['bullet'], f'{item}  ●'))

    story.append(p(S['h2'], 'أولوية متوسطة (توسيع المنصة)'))
    for item in [
        'لوحة لانهائية / سبورة — لوحة لانهائية مع ملاحظات مضمنة',
        'توضيح PDF — تمييز وتعليق ملفات PDF داخل الخزائن',
        'نشر / تصدير موقع ثابت — إنشاء مواقع ويب من محتوى الخزنة',
        'بروتوكول Constellation URI — ربط عميق بملاحظات وعروض محددة',
    ]:
        story.append(p(S['bullet'], f'{item}  ●'))

    story.append(p(S['h2'], 'أولوية أدنى (رؤية مستقبلية)'))
    for item in [
        'التسجيل الصوتي والنسخ',
        'واجهة برمجة الإضافات — السماح بإضافات طرف ثالث (بنطاق محدد بعناية)',
        'التحرير التعاوني — تحرير متعدد المستخدمين في الوقت الحقيقي عبر CRDT',
    ]:
        story.append(p(S['bullet'], f'{item}  ●'))

    # ===== SECTION 10: الخاتمة =====
    story.append(p(S['h1'], '١٠. الخاتمة'))
    story.append(gold_hr())
    story.append(p(S['body'],
        'كونستلاشن موجود لأن إدارة المعرفة لا ينبغي أن تتطلب تكامل أنظمة. '
        'منصة تدوين الملاحظات يجب أن تشحن بالأدوات التي يحتاجها مستخدموها — '
        'قواعد بيانات ومهام وتقاويم وقوالب ومستورِدات وذكاء اصطناعي ودعم تعدد الخزائن — '
        'مختبرة معاً ومُحدَّثة معاً وجاهزة للاستخدام فوراً.'
    ))
    story.append(p(S['body'],
        'لعامل المعرفة الذي بنى سير عمله عبر أدوات متعددة ويشعر باحتكاك إدارة تلك الحزمة، '
        'يقدم كونستلاشن بديلاً موحداً يعمل مع ملفات ماركداون القياسية، ولا يتطلب أي تكوين، '
        'ويوفر قدرات لا تقدمها أي أداة واحدة حالياً — '
        'مساحات عمل حقيقية متعددة الخزائن، وكل شيء عبر الخزائن، وتكوين محمول قائم على الكون.'
    ))
    story.append(p(S['body'],
        'الملفات ملكك. الصيغة ماركداون. والباب مفتوح دائماً.'
    ))
    story.append(Spacer(1, 20))
    story.append(HRFlowable(width="30%", thickness=2, color=GOLD, spaceBefore=10, spaceAfter=10))

    oss = ParagraphStyle(
        'OSS', fontName='Tahoma', fontSize=10, textColor=LIGHT_TEXT,
        alignment=TA_CENTER, leading=18
    )
    story.append(p(oss, 'كونستلاشن مفتوح المصدر تحت رخصة MIT'))
    story.append(Paragraph('Developed by Eisa ALSHAMSI', oss))
    story.append(p(oss, 'طور بواسطة: عيسى الشامسي'))
    story.append(Paragraph('github.com/eisaAlshamsi/Constellation', oss))

    # ===== LEGAL NOTICE =====
    story.append(PageBreak())
    story.append(p(S['h1'], 'إشعار قانوني'))
    story.append(gold_hr())

    disclaimer = ParagraphStyle(
        'Disclaimer', fontName='Tahoma', fontSize=9.5, textColor=LIGHT_TEXT,
        alignment=TA_RIGHT, spaceAfter=8, leading=18
    )

    story.append(p(S['h2'], 'إقرار العلامات التجارية'))
    story.append(Paragraph(ar(
        'جميع أسماء المنتجات والشعارات والعلامات التجارية المذكورة في هذه الوثيقة هي ملك لأصحابها. '
        '"أوبسيديان" علامة تجارية لشركة Dynalist Inc. '
        '"Notion" علامة تجارية لشركة Notion Labs, Inc. '
        '"Bear" علامة تجارية لشركة Shiny Frog Ltd. '
        '"Evernote" علامة تجارية لشركة Bending Spoons S.p.A. '
        '"Logseq" علامة تجارية لشركة Logseq, Inc. '
        '"Roam" علامة تجارية لشركة Roam Research, Inc.'
    ), disclaimer))
    story.append(Paragraph(ar(
        'كونستلاشن مشروع مستقل وليس تابعاً أو مدعوماً أو مرعىً من أي من الشركات المذكورة أعلاه. '
        'جميع الإشارات إلى منتجات الطرف الثالث في هذه الوثيقة هي لأغراض المقارنة الفعلية '
        'ووصف التوافقية فقط، بموجب الاستخدام العادل الاسمي.'
    ), disclaimer))

    story.append(p(S['h2'], 'بيان الملكية الفكرية'))
    story.append(Paragraph(ar(
        'كونستلاشن برنامج أصلي طُوِّر بشكل مستقل. لا يحتوي على أو يدمج أو يُشتق من أي كود مصدري '
        'لأي تطبيق طرف ثالث. كونستلاشن يقرأ ويكتب ملفات ماركداون قياسية مع YAML frontmatter — صيغ مفتوحة '
        'وغير ملكية. صيغة وصلات الويكي ([[رابط]]) نشأت من برمجيات الويكي وليست ملكية '
        'لأي مورّد. التوافقية على مستوى الملفات مع أدوات ماركداون المختلفة تتم من خلال عمليات نظام ملفات '
        'قياسية على صيغ مفتوحة، وليس من خلال الهندسة العكسية أو استخدام واجهات برمجة ملكية.'
    ), disclaimer))

    story.append(p(S['h2'], 'الامتثال للمصادر المفتوحة'))
    story.append(Paragraph(ar(
        'كونستلاشن مرخص بموجب رخصة MIT. جميع التبعيات من الطرف الثالث تُستخدم وفقاً '
        'لتراخيص المصادر المفتوحة الخاصة بها. يُحتفظ بمراجعة كاملة للتبعيات في مستودع المشروع.'
    ), disclaimer))

    doc.build(story, canvasmaker=NumberedCanvas)
    print(f"PDF generated: {output}")
    return output


if __name__ == '__main__':
    build_pdf()
