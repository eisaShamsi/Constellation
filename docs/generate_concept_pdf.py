"""
Generate a professionally formatted PDF for the Constellation Concept Paper.
Dark blue (#1a2332) and gold/amber (#d4a574) constellation theme.
"""

from reportlab.lib.pagesizes import A4
from reportlab.lib.units import inch, mm
from reportlab.lib.colors import HexColor, white, black, Color
from reportlab.lib.styles import ParagraphStyle
from reportlab.lib.enums import TA_LEFT, TA_CENTER, TA_JUSTIFY, TA_RIGHT
from reportlab.platypus import (
    BaseDocTemplate, PageTemplate, Frame, Paragraph, Spacer, Table, TableStyle,
    PageBreak, KeepTogether, NextPageTemplate, Flowable
)
from reportlab.pdfgen import canvas
from reportlab.pdfbase import pdfmetrics
from reportlab.pdfbase.ttfonts import TTFont
import os

# ─── Colors ───
DARK_BLUE = HexColor("#1a2332")
GOLD = HexColor("#d4a574")
LIGHT_GOLD = HexColor("#f5e6d0")
MEDIUM_BLUE = HexColor("#2a3a52")
LIGHT_BLUE = HexColor("#e8edf4")
SOFT_BLUE = HexColor("#3d5a80")
TEXT_DARK = HexColor("#1a1a2e")
TEXT_MEDIUM = HexColor("#4a4a5a")
TEXT_LIGHT = HexColor("#6a6a7a")
WHITE = white
ROW_ALT = HexColor("#f7f9fc")
BORDER_LIGHT = HexColor("#d0d8e4")
ACCENT_RED = HexColor("#c44536")
ACCENT_GREEN = HexColor("#2d6a4f")

PAGE_W, PAGE_H = A4
MARGIN_L = 60
MARGIN_R = 60
MARGIN_T = 65
MARGIN_B = 70
CONTENT_W = PAGE_W - MARGIN_L - MARGIN_R


# ─── Custom Flowables ───

class HorizontalRule(Flowable):
    def __init__(self, width, color=GOLD, thickness=1.5):
        super().__init__()
        self.width = width
        self.color = color
        self.thickness = thickness

    def wrap(self, availWidth, availHeight):
        return (self.width, self.thickness + 6)

    def draw(self):
        self.canv.setStrokeColor(self.color)
        self.canv.setLineWidth(self.thickness)
        self.canv.line(0, 3, self.width, 3)


class GoldAccentBox(Flowable):
    """A left-bordered quote/callout box."""
    def __init__(self, text, width, style):
        super().__init__()
        self.text = text
        self.width = width
        self.style = style
        self._para = Paragraph(text, style)

    def wrap(self, availWidth, availHeight):
        w, h = self._para.wrap(self.width - 30, availHeight)
        self.para_h = h
        return (self.width, h + 16)

    def draw(self):
        # Gold left border
        self.canv.setStrokeColor(GOLD)
        self.canv.setLineWidth(3)
        self.canv.line(0, 0, 0, self.para_h + 16)
        # Light background
        self.canv.setFillColor(LIGHT_GOLD)
        self.canv.rect(3, 0, self.width - 3, self.para_h + 16, fill=1, stroke=0)
        # Text
        self._para.drawOn(self.canv, 18, 8)


class CodeBlock(Flowable):
    """Monospace code block with background."""
    def __init__(self, text, width):
        super().__init__()
        self.text = text
        self.width = width
        self.lines = text.strip().split('\n')
        self.line_h = 13
        self.padding = 12

    def wrap(self, availWidth, availHeight):
        h = len(self.lines) * self.line_h + self.padding * 2
        return (self.width, h)

    def draw(self):
        h = len(self.lines) * self.line_h + self.padding * 2
        # Background
        self.canv.setFillColor(HexColor("#f0f2f5"))
        self.canv.roundRect(0, 0, self.width, h, 4, fill=1, stroke=0)
        # Border
        self.canv.setStrokeColor(BORDER_LIGHT)
        self.canv.setLineWidth(0.5)
        self.canv.roundRect(0, 0, self.width, h, 4, fill=0, stroke=1)
        # Text
        self.canv.setFillColor(TEXT_DARK)
        self.canv.setFont("Courier", 8.5)
        y = h - self.padding - 2
        for line in self.lines:
            self.canv.drawString(self.padding, y, line)
            y -= self.line_h


# ─── Styles ───

def make_styles():
    s = {}

    s['body'] = ParagraphStyle(
        'body', fontName='Helvetica', fontSize=10, leading=15,
        textColor=TEXT_DARK, alignment=TA_JUSTIFY, spaceAfter=8,
        spaceBefore=2
    )
    s['body_bold'] = ParagraphStyle(
        'body_bold', parent=s['body'], fontName='Helvetica-Bold'
    )
    s['section_title'] = ParagraphStyle(
        'section_title', fontName='Helvetica-Bold', fontSize=18, leading=24,
        textColor=DARK_BLUE, spaceBefore=28, spaceAfter=6
    )
    s['subsection_title'] = ParagraphStyle(
        'subsection_title', fontName='Helvetica-Bold', fontSize=13, leading=18,
        textColor=SOFT_BLUE, spaceBefore=18, spaceAfter=6
    )
    s['subsubsection'] = ParagraphStyle(
        'subsubsection', fontName='Helvetica-Bold', fontSize=11, leading=15,
        textColor=MEDIUM_BLUE, spaceBefore=12, spaceAfter=4
    )
    s['bullet'] = ParagraphStyle(
        'bullet', fontName='Helvetica', fontSize=10, leading=15,
        textColor=TEXT_DARK, leftIndent=20, bulletIndent=8,
        spaceAfter=4, alignment=TA_LEFT
    )
    s['quote'] = ParagraphStyle(
        'quote', fontName='Helvetica-Oblique', fontSize=10.5, leading=16,
        textColor=MEDIUM_BLUE, leftIndent=15, rightIndent=15,
        spaceBefore=4, spaceAfter=4, alignment=TA_LEFT
    )
    s['toc_entry'] = ParagraphStyle(
        'toc_entry', fontName='Helvetica', fontSize=11, leading=20,
        textColor=DARK_BLUE, leftIndent=10
    )
    s['toc_title'] = ParagraphStyle(
        'toc_title', fontName='Helvetica-Bold', fontSize=20, leading=26,
        textColor=DARK_BLUE, spaceAfter=20, alignment=TA_CENTER
    )
    s['footer_text'] = ParagraphStyle(
        'footer_text', fontName='Helvetica-Oblique', fontSize=9, leading=12,
        textColor=TEXT_LIGHT, alignment=TA_CENTER
    )
    s['table_header'] = ParagraphStyle(
        'table_header', fontName='Helvetica-Bold', fontSize=8.5, leading=12,
        textColor=WHITE, alignment=TA_LEFT
    )
    s['table_cell'] = ParagraphStyle(
        'table_cell', fontName='Helvetica', fontSize=8.5, leading=12,
        textColor=TEXT_DARK, alignment=TA_LEFT
    )
    s['table_cell_bold'] = ParagraphStyle(
        'table_cell_bold', fontName='Helvetica-Bold', fontSize=8.5, leading=12,
        textColor=TEXT_DARK, alignment=TA_LEFT
    )
    s['profile_label'] = ParagraphStyle(
        'profile_label', fontName='Helvetica-Bold', fontSize=10, leading=14,
        textColor=GOLD, spaceBefore=6, spaceAfter=2
    )
    return s


# ─── Page Templates ───

def cover_page(canvas_obj, doc):
    c = canvas_obj
    w, h = PAGE_W, PAGE_H

    # Full dark blue background
    c.setFillColor(DARK_BLUE)
    c.rect(0, 0, w, h, fill=1, stroke=0)

    # Decorative gold lines (constellation feel)
    c.setStrokeColor(GOLD)
    c.setLineWidth(0.3)
    import random
    random.seed(42)
    for _ in range(15):
        x1 = random.uniform(50, w - 50)
        y1 = random.uniform(100, h - 100)
        x2 = x1 + random.uniform(-120, 120)
        y2 = y1 + random.uniform(-120, 120)
        c.line(x1, y1, x2, y2)

    # Small gold dots (stars)
    c.setFillColor(GOLD)
    for _ in range(40):
        x = random.uniform(30, w - 30)
        y = random.uniform(80, h - 80)
        r = random.uniform(1, 2.5)
        c.circle(x, y, r, fill=1, stroke=0)

    # Title block
    c.setFillColor(GOLD)
    c.setFont("Helvetica-Bold", 48)
    c.drawCentredString(w / 2, h / 2 + 60, "Constellation")

    # Subtitle
    c.setFont("Helvetica", 18)
    c.setFillColor(HexColor("#a0b4cc"))
    c.drawCentredString(w / 2, h / 2 + 15, "Concept Paper")

    # Gold rule
    c.setStrokeColor(GOLD)
    c.setLineWidth(2)
    rule_w = 200
    c.line(w / 2 - rule_w / 2, h / 2 - 10, w / 2 + rule_w / 2, h / 2 - 10)

    # Date
    c.setFont("Helvetica", 14)
    c.setFillColor(HexColor("#8899aa"))
    c.drawCentredString(w / 2, h / 2 - 40, "March 2026")

    # Version
    c.setFont("Helvetica", 11)
    c.setFillColor(HexColor("#667788"))
    c.drawCentredString(w / 2, h / 2 - 65, "Version 1.0")

    # Tagline at bottom
    c.setFont("Helvetica-Oblique", 12)
    c.setFillColor(GOLD)
    c.drawCentredString(w / 2, 80, "A Vault of Vaults  \u2022  A Map of Maps")

    # Bottom line
    c.setStrokeColor(GOLD)
    c.setLineWidth(0.5)
    c.line(80, 60, w - 80, 60)

    c.setFont("Helvetica", 9)
    c.setFillColor(HexColor("#556677"))
    c.drawCentredString(w / 2, 42, "Open Source  \u2022  MIT License  \u2022  github.com/eisaShamsi/Constellation")


def normal_page(canvas_obj, doc):
    c = canvas_obj
    w, h = PAGE_W, PAGE_H
    page_num = doc.page

    # Top gold accent line
    c.setStrokeColor(GOLD)
    c.setLineWidth(1.5)
    c.line(MARGIN_L, h - 40, w - MARGIN_R, h - 40)

    # Header
    c.setFont("Helvetica", 8)
    c.setFillColor(TEXT_LIGHT)
    c.drawString(MARGIN_L, h - 33, "Constellation \u2014 Concept Paper")
    c.drawRightString(w - MARGIN_R, h - 33, "March 2026")

    # Footer
    c.setStrokeColor(BORDER_LIGHT)
    c.setLineWidth(0.5)
    c.line(MARGIN_L, MARGIN_B - 15, w - MARGIN_R, MARGIN_B - 15)

    c.setFont("Helvetica", 9)
    c.setFillColor(TEXT_LIGHT)
    c.drawCentredString(w / 2, MARGIN_B - 32, str(page_num))


# ─── Table Builder ───

def build_table(headers, rows, col_widths, styles):
    """Build a styled table with alternating rows."""
    header_cells = [Paragraph(h, styles['table_header']) for h in headers]
    data = [header_cells]

    for row in rows:
        cells = []
        for i, cell in enumerate(row):
            if i == 0 and cell.startswith('<b>'):
                cells.append(Paragraph(cell, styles['table_cell_bold']))
            else:
                cells.append(Paragraph(cell, styles['table_cell']))
        data.append(cells)

    t = Table(data, colWidths=col_widths, repeatRows=1)

    style_cmds = [
        # Header
        ('BACKGROUND', (0, 0), (-1, 0), DARK_BLUE),
        ('TEXTCOLOR', (0, 0), (-1, 0), WHITE),
        ('FONTNAME', (0, 0), (-1, 0), 'Helvetica-Bold'),
        ('FONTSIZE', (0, 0), (-1, 0), 9),
        ('BOTTOMPADDING', (0, 0), (-1, 0), 8),
        ('TOPPADDING', (0, 0), (-1, 0), 8),
        # Body
        ('FONTNAME', (0, 1), (-1, -1), 'Helvetica'),
        ('FONTSIZE', (0, 1), (-1, -1), 8.5),
        ('TOPPADDING', (0, 1), (-1, -1), 6),
        ('BOTTOMPADDING', (0, 1), (-1, -1), 6),
        ('LEFTPADDING', (0, 0), (-1, -1), 8),
        ('RIGHTPADDING', (0, 0), (-1, -1), 8),
        # Grid
        ('LINEBELOW', (0, 0), (-1, 0), 1.5, GOLD),
        ('LINEBELOW', (0, 1), (-1, -2), 0.5, BORDER_LIGHT),
        ('LINEBELOW', (0, -1), (-1, -1), 1, DARK_BLUE),
        ('VALIGN', (0, 0), (-1, -1), 'TOP'),
    ]

    # Alternating row colors
    for i in range(1, len(data)):
        if i % 2 == 0:
            style_cmds.append(('BACKGROUND', (0, i), (-1, i), ROW_ALT))

    t.setStyle(TableStyle(style_cmds))
    return t


# ─── Build Document ───

def build_pdf():
    output_path = os.path.join(os.path.dirname(__file__), "Constellation \u2014 Concept Paper.pdf")

    doc = BaseDocTemplate(
        output_path,
        pagesize=A4,
        leftMargin=MARGIN_L,
        rightMargin=MARGIN_R,
        topMargin=MARGIN_T,
        bottomMargin=MARGIN_B,
        title="Constellation \u2014 Concept Paper",
        author="Constellation Team",
        subject="Knowledge Management Application Concept Paper",
    )

    frame_content = Frame(
        MARGIN_L, MARGIN_B,
        CONTENT_W, PAGE_H - MARGIN_T - MARGIN_B,
        id='normal'
    )

    # Cover uses full-page drawing, no frame content
    cover_frame = Frame(MARGIN_L, MARGIN_B, CONTENT_W, PAGE_H - MARGIN_T - MARGIN_B, id='cover')

    doc.addPageTemplates([
        PageTemplate(id='cover', frames=[cover_frame], onPage=cover_page),
        PageTemplate(id='content', frames=[frame_content], onPage=normal_page),
    ])

    S = make_styles()
    story = []

    # ─── Cover Page ───
    story.append(NextPageTemplate('content'))
    story.append(PageBreak())

    # ─── Table of Contents ───
    story.append(Paragraph("Table of Contents", S['toc_title']))
    story.append(Spacer(1, 10))

    toc_items = [
        ("1.", "What Is Constellation?"),
        ("2.", "The Problem Constellation Solves"),
        ("3.", "Core Architecture: The Universe Model"),
        ("4.", "Feature Comparison: Constellation vs. Obsidian"),
        ("5.", "Who Is Constellation For?"),
        ("6.", "Technical Advantages"),
        ("7.", "Development Validation Criteria"),
        ("8.", "Competitive Positioning"),
        ("9.", "Roadmap Implications"),
        ("10.", "Conclusion"),
    ]
    for num, title in toc_items:
        story.append(Paragraph(
            f'<font color="{GOLD.hexval()}">{num}</font>&nbsp;&nbsp;&nbsp;{title}',
            S['toc_entry']
        ))

    story.append(Spacer(1, 20))
    story.append(HorizontalRule(CONTENT_W))
    story.append(PageBreak())

    # ═══════════════════════════════════════════
    # Section 1: What Is Constellation?
    # ═══════════════════════════════════════════
    story.append(Paragraph("1. What Is Constellation?", S['section_title']))
    story.append(HorizontalRule(CONTENT_W, GOLD, 1))
    story.append(Spacer(1, 8))

    story.append(Paragraph(
        "Constellation is a desktop application for knowledge management, note-taking, and personal "
        "information organization. It reads and writes standard Obsidian-compatible Markdown files stored "
        "on the user\u2019s own file system \u2014 no cloud accounts, no vendor lock-in, no subscription required.",
        S['body']
    ))
    story.append(Paragraph(
        "Where Obsidian gives you a vault, Constellation gives you a <b>universe</b> \u2014 a portable, "
        "self-contained workspace that unifies multiple vaults, structured databases, AI assistance, task "
        "management, and calendar views into a single coherent experience. Everything Obsidian requires "
        "community plugins to achieve, Constellation ships as built-in, first-class functionality.",
        S['body']
    ))
    story.append(Paragraph(
        "<b>Technical foundation:</b> Tauri v2 (Rust backend) + SvelteKit + Svelte 5. Native performance, "
        "small binary size, full offline operation, no Electron overhead.",
        S['body']
    ))

    # ═══════════════════════════════════════════
    # Section 2: The Problem
    # ═══════════════════════════════════════════
    story.append(Paragraph("2. The Problem Constellation Solves", S['section_title']))
    story.append(HorizontalRule(CONTENT_W, GOLD, 1))
    story.append(Spacer(1, 8))

    story.append(Paragraph(
        "Obsidian is an excellent Markdown editor. But as a knowledge management <i>system</i>, it has "
        "structural limitations that its plugin ecosystem patches over rather than solves:",
        S['body']
    ))

    story.append(Spacer(1, 6))
    t2 = build_table(
        ["Problem", "Obsidian\u2019s Answer", "The Cost"],
        [
            ["One vault at a time", "Community workarounds", "Context-switching, no cross-vault search or linking"],
            ["Task management", "Tasks plugin (community)", "Separate install, separate updates, potential breakage"],
            ["Database views", "Dataview plugin (community)", "Complex query language, performance issues at scale"],
            ["Table editing", "Advanced Tables plugin (community)", "Basic functionality, no formulas"],
            ["Calendar integration", "Calendar plugin (community)", "Limited to daily notes, no task integration"],
            ["Templates", "Templater plugin (community)", "Complex syntax, security concerns with JS execution"],
            ["Note importing", "Importer plugin (community)", "Separate install, limited format support"],
            ["AI assistance", "Multiple competing plugins", "API key management across plugins, inconsistent UX"],
        ],
        [CONTENT_W * 0.22, CONTENT_W * 0.35, CONTENT_W * 0.43],
        S
    )
    story.append(t2)
    story.append(Spacer(1, 10))

    story.append(Paragraph(
        "Obsidian users routinely install 15\u201330 plugins to reach a functional workflow. Each plugin is "
        "maintained by a different developer, updated on a different schedule, and can break on any Obsidian "
        "update. The user becomes a systems integrator.",
        S['body']
    ))
    story.append(Paragraph(
        "<b>Constellation eliminates the integration tax.</b> Every capability listed above is built in, "
        "tested together, and ships as a unified experience.",
        S['body']
    ))

    # ═══════════════════════════════════════════
    # Section 3: Universe Model
    # ═══════════════════════════════════════════
    story.append(Paragraph("3. Core Architecture: The Universe Model", S['section_title']))
    story.append(HorizontalRule(CONTENT_W, GOLD, 1))
    story.append(Spacer(1, 8))

    story.append(Paragraph(
        "Constellation\u2019s defining architectural concept is the <b>Universe</b> \u2014 a portable directory "
        "that owns all user data and configuration.",
        S['body']
    ))
    story.append(Spacer(1, 4))

    story.append(CodeBlock(
        "MyUniverse/\n"
        "  universe.json          # Identity and metadata\n"
        "  vaults.json            # Registered vault paths\n"
        "  settings.json          # All preferences\n"
        "  bookmarks.json         # Saved bookmarks\n"
        "  workspaces.json        # Tab layouts\n"
        "  property-types.json    # Custom property mappings\n"
        "  bases/                 # Workspace-level databases",
        CONTENT_W
    ))
    story.append(Spacer(1, 10))

    story.append(Paragraph("Why This Matters", S['subsection_title']))

    benefits = [
        ("<b>Portability.</b> Copy the universe directory to another machine and everything follows \u2014 "
         "settings, bookmarks, workspaces, database definitions. The vaults themselves are just folders of "
         "Markdown files."),
        ("<b>Multi-vault by design.</b> A universe can register any number of vaults. Search, graph view, "
         "task scanning, backlinks, and databases all operate across vault boundaries natively."),
        ("<b>Hierarchy.</b> Universes can reference child universes, inheriting their vaults. A team lead\u2019s "
         "universe can include a shared team universe plus a personal universe \u2014 with circular reference "
         "prevention built in."),
        ("<b>No lock-in.</b> The universe is JSON files in a folder. The vaults are Markdown files in folders. "
         "Walk away at any time, open the same vaults in Obsidian, and nothing is lost."),
    ]
    for b in benefits:
        story.append(Paragraph(f"\u2022  {b}", S['bullet']))

    # ═══════════════════════════════════════════
    # Section 4: Feature Comparison
    # ═══════════════════════════════════════════
    story.append(Paragraph("4. Feature Comparison: Constellation vs. Obsidian", S['section_title']))
    story.append(HorizontalRule(CONTENT_W, GOLD, 1))
    story.append(Spacer(1, 8))

    # 4.1
    story.append(Paragraph("4.1  What Constellation Does That Obsidian Cannot (Even With Plugins)", S['subsection_title']))
    story.append(Spacer(1, 4))

    t41 = build_table(
        ["Capability", "Details"],
        [
            ["<b>True multi-vault workspace</b>", "Open, search, link, and graph across multiple vaults simultaneously. Obsidian can only open one vault at a time per window."],
            ["<b>Universe portability</b>", "All configuration travels in a single portable directory. Obsidian settings are per-vault and non-portable."],
            ["<b>Child universes</b>", "Compose workspaces hierarchically \u2014 a team vault feeds into your personal universe automatically."],
            ["<b>Cross-vault backlinks</b>", "See which notes in any vault link to the current note. Obsidian backlinks are vault-scoped."],
            ["<b>Cross-vault graph</b>", "One graph showing connections across all your vaults. Obsidian\u2019s graph is single-vault."],
            ["<b>Unified task scanning</b>", "Global Tasks view aggregates tasks from every vault with filtering by vault, priority, due date, and text search."],
            ["<b>Built-in Bases (databases)</b>", "Non-destructive database views with table/card/list modes, filtering, sorting, inline editing \u2014 no plugin needed."],
            ["<b>Table formulas</b>", "=SUM(), =AVG(), =COUNT(), =MIN(), =MAX() with cell references and ranges, evaluated in-place."],
            ["<b>Multi-provider AI</b>", "OpenAI, Anthropic, Google Gemini, and Ollama (local) from one interface, with 8 pre-built skills."],
            ["<b>Second screen</b>", "Dedicated secondary window for reference browsing while editing in the primary window."],
            ["<b>15 languages at launch</b>", "English, Arabic, German, Spanish, French, Hebrew, Hindi, Japanese, Korean, Portuguese, Russian, Turkish, Urdu, Chinese, Farsi \u2014 all with full RTL support."],
            ["<b>Security layer</b>", "Vault encryption at rest, idle lock with PIN, API key storage in OS keyring."],
        ],
        [CONTENT_W * 0.30, CONTENT_W * 0.70],
        S
    )
    story.append(t41)
    story.append(Spacer(1, 12))

    # 4.2
    story.append(Paragraph("4.2  What Constellation Matches (Built-In, No Plugins Required)", S['subsection_title']))
    story.append(Paragraph(
        "Every feature below requires a community plugin in Obsidian. In Constellation, it ships out of the box:",
        S['body']
    ))
    story.append(Spacer(1, 4))

    t42 = build_table(
        ["Feature", "Obsidian (Plugin Required)", "Constellation (Built-In)"],
        [
            ["Dataview queries", "Dataview plugin", "Native DQL parser (TABLE, LIST, TASK, CALENDAR queries)"],
            ["Task management", "Tasks plugin", "Vault-wide scanning, toggle, due dates, priority, tags"],
            ["Calendar sidebar", "Calendar plugin", "Month view with note/task dots, daily note creation"],
            ["Advanced tables", "Advanced Tables plugin", "Row/column operations, sorting, move, formulas"],
            ["Templates", "Templater plugin", "Template variables (date, time, title, folder, vault, cursor)"],
            ["Note importer", "Importer plugin", "7 formats: Markdown, Notion, Bear, Evernote, HTML, CSV, Plain Text"],
            ["Backlinks panel", "Core (basic)", "Enhanced with cross-vault support and unlinked mentions"],
            ["Graph view", "Core (basic)", "Enhanced with cross-vault nodes, force controls, grouping"],
            ["Tag browser", "Core (basic)", "Tag frequency, vault-wide aggregation"],
        ],
        [CONTENT_W * 0.18, CONTENT_W * 0.32, CONTENT_W * 0.50],
        S
    )
    story.append(t42)
    story.append(Spacer(1, 12))

    # 4.3
    story.append(Paragraph("4.3  What Obsidian Does That Constellation Does Not (Yet)", S['subsection_title']))
    story.append(Paragraph("Transparency matters. These are Obsidian capabilities not currently in Constellation:", S['body']))
    story.append(Spacer(1, 4))

    t43 = build_table(
        ["Feature", "Status"],
        [
            ["Mobile apps (iOS/Android)", "Not yet \u2014 desktop only (Windows, macOS, Linux)"],
            ["Obsidian Sync / Publish", "Not planned \u2014 use Git, Syncthing, or any file sync"],
            ["Community plugin ecosystem", "Not applicable \u2014 features are built-in"],
            ["Canvas (infinite whiteboard)", "Not yet"],
            ["Obsidian URI protocol", "Not yet"],
            ["PDF annotation", "Not yet"],
            ["Audio recording", "Not yet"],
        ],
        [CONTENT_W * 0.40, CONTENT_W * 0.60],
        S
    )
    story.append(t43)

    # ═══════════════════════════════════════════
    # Section 5: Who Is Constellation For?
    # ═══════════════════════════════════════════
    story.append(Paragraph("5. Who Is Constellation For?", S['section_title']))
    story.append(HorizontalRule(CONTENT_W, GOLD, 1))
    story.append(Spacer(1, 8))

    personas = [
        ("5.1  The Multi-Vault Professional",
         "Consultant, researcher, or knowledge worker who maintains separate vaults for different clients, projects, or life domains.",
         "Must close one vault to open another. Cannot search across vaults. Cannot see connections between client A\u2019s project notes and research notes in a separate vault.",
         "Register all vaults in one universe. Search, graph, task scan, and link across all of them simultaneously."),
        ("5.2  The Plugin-Fatigued Power User",
         "Current Obsidian user running 20+ plugins who spends significant time managing plugin updates, resolving conflicts, and debugging breakage after Obsidian updates.",
         "Every plugin update is a risk. Dataview, Tasks, Templater, Calendar, Advanced Tables, and Importer are all maintained by different people on different schedules.",
         "All of these are built-in, tested together, and updated as one unit. Zero plugin management."),
        ("5.3  The Arabic/RTL Knowledge Worker",
         "User who works primarily in Arabic, Hebrew, Farsi, or Urdu and needs a note-taking system that treats RTL as a first-class concern.",
         "RTL support is inconsistent. Some plugins break in RTL. Property editors assume LTR. Date keys and list keys don\u2019t recognize Arabic equivalents.",
         "15 languages including 4 RTL languages. Arabic property key detection. Full UI mirroring. RTL-aware tables, forms, editors, and calendar."),
        ("5.4  The Team Lead or Organization Builder",
         "Manager or team lead who wants to share a knowledge base with team members while maintaining personal notes separately.",
         "No concept of workspace composition. Shared vaults require manual setup per person.",
         "Create a team universe with shared vaults. Each team member adds the team universe as a child of their personal universe. Team vaults appear automatically alongside personal vaults."),
        ("5.5  The AI-Augmented Researcher",
         "Researcher or student who wants AI assistance integrated directly into their note-taking workflow.",
         "Must choose between competing AI plugins, configure API keys in each, and deal with inconsistent interfaces.",
         "One AI settings panel. Four provider options (including local Ollama for privacy). Eight pre-built skills. API keys stored in the OS keyring."),
        ("5.6  The Security-Conscious User",
         "Professional handling sensitive notes (legal, medical, financial, personal) who needs encryption and access control.",
         "No built-in encryption. No idle lock. API keys stored in plaintext plugin configs.",
         "Vault encryption at rest, idle lock with PIN, API key storage in OS keyring."),
    ]

    for title, profile, pain, answer in personas:
        story.append(Paragraph(title, S['subsection_title']))
        story.append(Paragraph(f"<b>Profile:</b> {profile}", S['body']))
        story.append(Paragraph(f"<b>Pain point with Obsidian:</b> {pain}", S['body']))
        story.append(Paragraph(f"<b>Constellation answer:</b> {answer}", S['body']))
        story.append(Spacer(1, 4))

    # ═══════════════════════════════════════════
    # Section 6: Technical Advantages
    # ═══════════════════════════════════════════
    story.append(Paragraph("6. Technical Advantages", S['section_title']))
    story.append(HorizontalRule(CONTENT_W, GOLD, 1))
    story.append(Spacer(1, 8))

    tech_sections = [
        ("6.1  Performance",
         "Constellation\u2019s Rust backend performs file operations, link scanning, task extraction, and database "
         "queries at native speed. Heavy operations \u2014 vault-wide task scanning, dataview queries, link graph "
         "building \u2014 execute in the Rust process and return structured results to the frontend.",
         "Obsidian\u2019s plugin system runs everything in the Electron renderer process (JavaScript). Community "
         "plugins like Dataview perform full vault scans in JS, competing for the same thread as the editor."),
        ("6.2  Binary Size and Resource Usage",
         "Tauri v2 uses the system\u2019s native webview rather than bundling Chromium. The result is a significantly "
         "smaller binary and lower memory footprint compared to Obsidian\u2019s Electron-based architecture.",
         None),
        ("6.3  Security Model",
         "Tauri\u2019s Rust backend provides a natural security boundary. File system access is controlled through "
         "explicit Tauri commands \u2014 the frontend cannot access arbitrary files. Path traversal prevention is "
         "enforced at the Rust layer (canonicalization checks on all file operations).",
         None),
        ("6.4  Data Sovereignty",
         "All data lives on the user\u2019s file system in standard formats: Markdown files with YAML frontmatter, "
         "JSON database files, JSON configuration files, and standard image/PDF attachments.",
         "No telemetry. No cloud dependency. No account required."),
    ]

    for title, p1, p2 in tech_sections:
        story.append(Paragraph(title, S['subsection_title']))
        story.append(Paragraph(p1, S['body']))
        if p2:
            story.append(Paragraph(p2, S['body']))

    # ═══════════════════════════════════════════
    # Section 7: Validation Criteria
    # ═══════════════════════════════════════════
    story.append(Paragraph("7. Development Validation Criteria", S['section_title']))
    story.append(HorizontalRule(CONTENT_W, GOLD, 1))
    story.append(Spacer(1, 8))

    story.append(Paragraph(
        "This section defines how we measure whether Constellation fulfills its purpose. Each criterion "
        "maps to a testable capability.",
        S['body']
    ))

    validation_tables = [
        ("7.1  Core Promise: \u201cRead and Write Obsidian Vaults\u201d", [
            ["Open a vault created in Obsidian", "All notes visible, frontmatter parsed, links resolved"],
            ["Edit a note and save", "File on disk updates, Obsidian can open and read the changes"],
            ["Create a note with frontmatter", "Valid YAML frontmatter, Obsidian-compatible"],
            ["Resolve [[wikilinks]]", "Same resolution behavior as Obsidian"],
            ["Render callouts, highlights, math, mermaid", "Visual parity with Obsidian\u2019s renderer"],
        ]),
        ("7.2  Multi-Vault Promise: \u201cA Vault of Vaults\u201d", [
            ["Register 3+ vaults", "All appear in file explorer with distinct colors"],
            ["Search across vaults", "Results from all vaults, labeled by source"],
            ["Graph across vaults", "Nodes from all vaults, cross-vault edges visible"],
            ["Backlinks across vaults", "Note in Vault A shows backlinks from Vault B"],
            ["Tasks across vaults", "Global Tasks view aggregates all vaults"],
        ]),
        ("7.3  Plugin Replacement Promise: \u201cNo Plugins Needed\u201d", [
            ["Dataview query in note", "TABLE, LIST, TASK queries render results"],
            ["Task checkbox toggle", "Toggle in sidebar updates file on disk"],
            ["Calendar dot indicators", "Days with notes/tasks show visual indicators"],
            ["Table formula evaluation", "=SUM(A1:A5) calculates correctly"],
            ["Template insertion", "Variables replaced with current date, time, title"],
            ["Import from Notion export", "Hex IDs removed, links converted to wikilinks"],
        ]),
        ("7.4  User Experience Promise: \u201cWorks for Everyone\u201d", [
            ["Switch to Arabic", "Full UI in Arabic, RTL layout, mirrored sidebar"],
            ["Create note with Arabic properties", "Date/list/checkbox keys detected correctly"],
            ["Open app on new machine with universe copy", "All settings, bookmarks, workspaces restored"],
            ["Lock app, enter PIN", "Notes inaccessible until correct PIN entered"],
        ]),
    ]

    for title, rows in validation_tables:
        story.append(Paragraph(title, S['subsection_title']))
        story.append(Spacer(1, 4))
        t = build_table(
            ["Test", "Expected Result"],
            rows,
            [CONTENT_W * 0.40, CONTENT_W * 0.60],
            S
        )
        story.append(t)
        story.append(Spacer(1, 10))

    # ═══════════════════════════════════════════
    # Section 8: Competitive Positioning
    # ═══════════════════════════════════════════
    story.append(Paragraph("8. Competitive Positioning", S['section_title']))
    story.append(HorizontalRule(CONTENT_W, GOLD, 1))
    story.append(Spacer(1, 8))

    story.append(Paragraph("Against Obsidian", S['subsection_title']))
    story.append(Paragraph(
        "Constellation is not anti-Obsidian. It is built <i>on</i> Obsidian\u2019s file format and <i>for</i> "
        "Obsidian\u2019s user base. The pitch is simple:",
        S['body']
    ))
    story.append(Spacer(1, 4))
    story.append(GoldAccentBox(
        "\u201cKeep your Markdown files exactly where they are. Keep using Obsidian if you want. But when you "
        "need multiple vaults in one window, built-in databases, cross-vault search, AI assistance, and zero "
        "plugin management \u2014 open Constellation.\u201d",
        CONTENT_W,
        S['quote']
    ))
    story.append(Spacer(1, 8))
    story.append(Paragraph(
        "Constellation does not require migration. It reads the same files. Users can switch between "
        "Obsidian and Constellation freely.",
        S['body']
    ))
    story.append(Spacer(1, 8))

    story.append(Paragraph("Against Notion, Logseq, Roam", S['subsection_title']))
    story.append(Spacer(1, 4))

    t8 = build_table(
        ["Dimension", "Constellation", "Notion", "Logseq", "Roam"],
        [
            ["Data ownership", "Local files", "Cloud-hosted", "Local files", "Cloud-hosted"],
            ["Offline capability", "Full", "Limited", "Full", "None"],
            ["File format", "Standard Markdown", "Proprietary", "Markdown/EDN", "Proprietary"],
            ["Multi-vault", "Native", "N/A (workspaces)", "Single graph", "Single graph"],
            ["Database views", "Built-in Bases", "Native databases", "Queries (limited)", "Queries"],
            ["AI integration", "Built-in (4 providers)", "Built-in (1 provider)", "Plugin", "Plugin"],
            ["Pricing", "Free / Open Source", "Freemium + subscription", "Free", "Subscription"],
            ["RTL / Arabic", "15 languages, 4 RTL", "Limited", "Limited", "Limited"],
        ],
        [CONTENT_W * 0.18, CONTENT_W * 0.22, CONTENT_W * 0.22, CONTENT_W * 0.18, CONTENT_W * 0.20],
        S
    )
    story.append(t8)

    # ═══════════════════════════════════════════
    # Section 9: Roadmap
    # ═══════════════════════════════════════════
    story.append(Paragraph("9. Roadmap Implications", S['section_title']))
    story.append(HorizontalRule(CONTENT_W, GOLD, 1))
    story.append(Spacer(1, 8))

    story.append(Paragraph(
        "Based on this concept paper, the following development priorities align with Constellation\u2019s positioning:",
        S['body']
    ))

    story.append(Paragraph("High Priority (Reinforces Core Differentiators)", S['subsection_title']))
    high = [
        "<b>Polish multi-vault experience</b> \u2014 cross-vault move/copy, vault-scoped settings",
        "<b>Bases performance at scale</b> \u2014 handle 10,000+ note databases efficiently",
        "<b>AI skill expansion</b> \u2014 custom skill builder, context-aware vault Q&amp;A",
        "<b>Mobile companion</b> \u2014 read-only vault browser for iOS/Android",
    ]
    for i, item in enumerate(high, 1):
        story.append(Paragraph(f"{i}.&nbsp;&nbsp;{item}", S['bullet']))

    story.append(Paragraph("Medium Priority (Competitive Parity)", S['subsection_title']))
    med = [
        "<b>Canvas / whiteboard</b> \u2014 infinite canvas with embedded notes",
        "<b>PDF annotation</b> \u2014 highlight and annotate PDFs within vaults",
        "<b>Obsidian URI compatibility</b> \u2014 handle obsidian:// protocol links",
        "<b>Publish / static site export</b> \u2014 generate websites from vault content",
    ]
    for i, item in enumerate(med, 5):
        story.append(Paragraph(f"{i}.&nbsp;&nbsp;{item}", S['bullet']))

    story.append(Paragraph("Lower Priority (Nice to Have)", S['subsection_title']))
    low = [
        "<b>Audio recording and transcription</b>",
        "<b>Plugin API</b> \u2014 allow third-party extensions (carefully scoped)",
        "<b>Collaborative editing</b> \u2014 real-time multi-user editing via CRDT",
    ]
    for i, item in enumerate(low, 9):
        story.append(Paragraph(f"{i}.&nbsp;&nbsp;{item}", S['bullet']))

    # ═══════════════════════════════════════════
    # Section 10: Conclusion
    # ═══════════════════════════════════════════
    story.append(Paragraph("10. Conclusion", S['section_title']))
    story.append(HorizontalRule(CONTENT_W, GOLD, 1))
    story.append(Spacer(1, 8))

    story.append(Paragraph(
        "Constellation exists because knowledge management should not require systems integration. A "
        "note-taking application should ship with the tools its users need \u2014 databases, tasks, calendars, "
        "templates, importers, AI, and multi-vault support \u2014 tested together, updated together, and usable "
        "out of the box.",
        S['body']
    ))
    story.append(Paragraph(
        "For the Obsidian user who has built a workflow on community plugins and feels the friction of managing "
        "that stack, Constellation offers a unified alternative that reads the same files, requires zero "
        "configuration, and adds capabilities that Obsidian\u2019s architecture cannot provide \u2014 true multi-vault "
        "workspaces, cross-vault everything, and portable universe-based configuration.",
        S['body']
    ))

    story.append(Spacer(1, 16))
    story.append(HorizontalRule(CONTENT_W, GOLD, 2))
    story.append(Spacer(1, 12))

    story.append(GoldAccentBox(
        "The files are yours. The format is Markdown. The door is open in both directions.",
        CONTENT_W,
        ParagraphStyle(
            'closing_quote', fontName='Helvetica-Bold', fontSize=12, leading=18,
            textColor=DARK_BLUE, alignment=TA_LEFT
        )
    ))

    story.append(Spacer(1, 20))
    story.append(Paragraph(
        "<i>Constellation is open source under the MIT license.</i>",
        S['footer_text']
    ))
    story.append(Paragraph(
        "<i>Repository: github.com/eisaShamsi/Constellation</i>",
        S['footer_text']
    ))

    # ─── Build ───
    doc.build(story)
    print(f"PDF generated: {output_path}")


if __name__ == "__main__":
    build_pdf()
