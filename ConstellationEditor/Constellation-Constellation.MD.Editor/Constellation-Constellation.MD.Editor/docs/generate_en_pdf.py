#!/usr/bin/env python3
"""Generate the English Constellation Concept Paper PDF (v2.0)."""

from reportlab.lib.pagesizes import A4
from reportlab.lib.styles import ParagraphStyle, getSampleStyleSheet
from reportlab.lib.units import inch, mm
from reportlab.lib.colors import HexColor, white, black
from reportlab.lib.enums import TA_CENTER, TA_LEFT, TA_JUSTIFY
from reportlab.platypus import (
    SimpleDocTemplate, Paragraph, Spacer, Table, TableStyle,
    PageBreak, KeepTogether, HRFlowable
)
from reportlab.pdfgen import canvas
from reportlab.lib import colors
from reportlab.pdfbase import pdfmetrics
from reportlab.pdfbase.ttfonts import TTFont
import os

# Colors
DARK_BLUE = HexColor('#1a2332')
GOLD = HexColor('#d4a574')
LIGHT_GOLD = HexColor('#f5e6d3')
MED_BLUE = HexColor('#2c3e50')
LIGHT_BLUE = HexColor('#eef2f7')
ACCENT = HexColor('#c9956b')
TEXT_COLOR = HexColor('#2c3e50')
LIGHT_TEXT = HexColor('#5a6a7a')

# Register fonts
pdfmetrics.registerFont(TTFont('Arial', 'C:/Windows/Fonts/arial.ttf'))
pdfmetrics.registerFont(TTFont('Arial-Bold', 'C:/Windows/Fonts/arialbd.ttf'))
pdfmetrics.registerFont(TTFont('Arial-Italic', 'C:/Windows/Fonts/ariali.ttf'))
pdfmetrics.registerFont(TTFont('Arial-BoldItalic', 'C:/Windows/Fonts/arialbi.ttf'))
pdfmetrics.registerFont(TTFont('Tahoma', 'C:/Windows/Fonts/tahoma.ttf'))

W, H = A4

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
            if i > 0:  # Skip cover page
                self.setFont('Arial', 8)
                self.setFillColor(LIGHT_TEXT)
                self.drawCentredString(W / 2, 20 * mm, f"Constellation — Concept Paper  |  Page {i} of {num_pages - 1}")
                # Gold line at top
                self.setStrokeColor(GOLD)
                self.setLineWidth(0.5)
                self.line(30 * mm, H - 15 * mm, W - 30 * mm, H - 15 * mm)
            canvas.Canvas.showPage(self)
        canvas.Canvas.save(self)


def create_styles():
    styles = {}

    styles['title'] = ParagraphStyle(
        'Title', fontName='Arial-Bold', fontSize=36, textColor=DARK_BLUE,
        alignment=TA_CENTER, spaceAfter=12, leading=44
    )
    styles['subtitle'] = ParagraphStyle(
        'Subtitle', fontName='Arial', fontSize=16, textColor=GOLD,
        alignment=TA_CENTER, spaceAfter=6, leading=22
    )
    styles['h1'] = ParagraphStyle(
        'H1', fontName='Arial-Bold', fontSize=20, textColor=DARK_BLUE,
        spaceBefore=28, spaceAfter=14, leading=26,
        borderWidth=0, borderPadding=0
    )
    styles['h2'] = ParagraphStyle(
        'H2', fontName='Arial-Bold', fontSize=15, textColor=MED_BLUE,
        spaceBefore=20, spaceAfter=10, leading=20
    )
    styles['h3'] = ParagraphStyle(
        'H3', fontName='Arial-Bold', fontSize=12, textColor=ACCENT,
        spaceBefore=14, spaceAfter=8, leading=16
    )
    styles['body'] = ParagraphStyle(
        'Body', fontName='Arial', fontSize=10.5, textColor=TEXT_COLOR,
        alignment=TA_JUSTIFY, spaceAfter=8, leading=16,
        firstLineIndent=0
    )
    styles['body_bold'] = ParagraphStyle(
        'BodyBold', fontName='Arial-Bold', fontSize=10.5, textColor=TEXT_COLOR,
        alignment=TA_JUSTIFY, spaceAfter=8, leading=16
    )
    styles['quote'] = ParagraphStyle(
        'Quote', fontName='Arial-Italic', fontSize=11, textColor=ACCENT,
        alignment=TA_CENTER, spaceBefore=12, spaceAfter=12, leading=17,
        leftIndent=40, rightIndent=40
    )
    styles['toc_item'] = ParagraphStyle(
        'TOC', fontName='Arial', fontSize=11, textColor=MED_BLUE,
        spaceBefore=4, spaceAfter=4, leading=16, leftIndent=20
    )
    styles['toc_title'] = ParagraphStyle(
        'TOCTitle', fontName='Arial-Bold', fontSize=16, textColor=DARK_BLUE,
        spaceBefore=10, spaceAfter=20, leading=22
    )
    styles['bullet'] = ParagraphStyle(
        'Bullet', fontName='Arial', fontSize=10.5, textColor=TEXT_COLOR,
        spaceAfter=4, leading=16, leftIndent=24, bulletIndent=12
    )
    styles['code'] = ParagraphStyle(
        'Code', fontName='Courier', fontSize=9, textColor=MED_BLUE,
        spaceAfter=4, leading=14, leftIndent=30, backColor=LIGHT_BLUE
    )
    styles['th'] = ParagraphStyle(
        'TableHeader', fontName='Arial-Bold', fontSize=9.5, textColor=white,
        leading=14
    )
    return styles


def gold_hr():
    return HRFlowable(width="100%", thickness=1.5, color=GOLD, spaceBefore=6, spaceAfter=6)


def make_table(headers, rows, col_widths=None):
    """Create a styled table."""
    data = [headers] + rows
    if not col_widths:
        available = W - 60 * mm
        col_widths = [available / len(headers)] * len(headers)

    t = Table(data, colWidths=col_widths, repeatRows=1)
    style_cmds = [
        ('BACKGROUND', (0, 0), (-1, 0), DARK_BLUE),
        ('TEXTCOLOR', (0, 0), (-1, 0), GOLD),
        ('FONTNAME', (0, 0), (-1, 0), 'Arial-Bold'),
        ('FONTSIZE', (0, 0), (-1, 0), 9.5),
        ('FONTNAME', (0, 1), (-1, -1), 'Arial'),
        ('FONTSIZE', (0, 1), (-1, -1), 9),
        ('TEXTCOLOR', (0, 1), (-1, -1), TEXT_COLOR),
        ('ALIGN', (0, 0), (-1, -1), 'LEFT'),
        ('VALIGN', (0, 0), (-1, -1), 'TOP'),
        ('TOPPADDING', (0, 0), (-1, -1), 6),
        ('BOTTOMPADDING', (0, 0), (-1, -1), 6),
        ('LEFTPADDING', (0, 0), (-1, -1), 8),
        ('RIGHTPADDING', (0, 0), (-1, -1), 8),
        ('GRID', (0, 0), (-1, -1), 0.5, HexColor('#dde3ea')),
        ('LINEBELOW', (0, 0), (-1, 0), 1.5, GOLD),
    ]
    # Alternating row colors
    for i in range(1, len(data)):
        if i % 2 == 0:
            style_cmds.append(('BACKGROUND', (0, i), (-1, i), LIGHT_BLUE))
        else:
            style_cmds.append(('BACKGROUND', (0, i), (-1, i), white))

    t.setStyle(TableStyle(style_cmds))
    return t


def p(style, text):
    return Paragraph(text, style)


def build_pdf():
    output = os.path.join(os.path.dirname(__file__), "Constellation — Concept Paper.pdf")

    doc = SimpleDocTemplate(
        output, pagesize=A4,
        leftMargin=30 * mm, rightMargin=30 * mm,
        topMargin=25 * mm, bottomMargin=25 * mm
    )

    S = create_styles()
    story = []
    avail = W - 60 * mm  # available width

    # ===== COVER PAGE =====
    story.append(Spacer(1, 80))

    # Decorative line
    story.append(HRFlowable(width="40%", thickness=2, color=GOLD, spaceBefore=0, spaceAfter=20))
    story.append(p(S['title'], 'CONSTELLATION'))
    story.append(Spacer(1, 8))
    story.append(p(S['subtitle'], 'Concept Paper'))
    story.append(Spacer(1, 4))
    story.append(p(S['subtitle'], 'Version 2.0 — March 2026'))
    story.append(Spacer(1, 20))
    story.append(HRFlowable(width="40%", thickness=2, color=GOLD, spaceBefore=0, spaceAfter=40))

    tagline_style = ParagraphStyle(
        'Tagline', fontName='Arial-Italic', fontSize=13, textColor=GOLD,
        alignment=TA_CENTER, leading=20
    )
    story.append(p(tagline_style, 'A Universe for Your Knowledge'))
    story.append(Spacer(1, 30))

    desc_style = ParagraphStyle(
        'CoverDesc', fontName='Arial', fontSize=11, textColor=LIGHT_TEXT,
        alignment=TA_CENTER, leading=17
    )
    story.append(p(desc_style,
        'Multi-library knowledge management. Built-in databases, tasks, calendars, '
        'AI assistance, and templates. Non-destructive. No plugins required.'
    ))
    story.append(Spacer(1, 60))

    footer_style = ParagraphStyle(
        'CoverFooter', fontName='Arial', fontSize=9, textColor=LIGHT_TEXT,
        alignment=TA_CENTER, leading=14
    )
    # Developer line with Arabic name using Tahoma
    dev_style = ParagraphStyle(
        'CoverDev', fontName='Arial', fontSize=10, textColor=LIGHT_TEXT,
        alignment=TA_CENTER, leading=16
    )
    story.append(p(dev_style,
        'Developed by Eisa ALSHAMSI — '
        '<font face="Tahoma">\u0637\u0648\u0631 \u0628\u0648\u0627\u0633\u0637\u0629: \u0639\u064a\u0633\u0649 \u0627\u0644\u0634\u0627\u0645\u0633\u064a</font>'
    ))
    story.append(Spacer(1, 8))
    story.append(p(footer_style, 'Tauri v2 + SvelteKit + Svelte 5 + Rust'))
    story.append(p(footer_style, 'Open Source — MIT License'))

    story.append(PageBreak())

    # ===== TABLE OF CONTENTS =====
    story.append(p(S['toc_title'], 'Table of Contents'))
    story.append(gold_hr())
    toc_items = [
        ('1', 'What Is Constellation?'),
        ('2', 'The Problem Constellation Solves'),
        ('3', 'Core Architecture: The Universe Model'),
        ('4', 'What Constellation Offers'),
        ('5', 'Who Is Constellation For?'),
        ('6', 'Technical Advantages'),
        ('7', 'Development Validation Criteria'),
        ('8', 'Competitive Landscape'),
        ('9', 'Roadmap'),
        ('10', 'Conclusion'),
    ]
    for num, title in toc_items:
        story.append(p(S['toc_item'], f'<b>{num}.</b>  {title}'))

    story.append(PageBreak())

    # ===== SECTION 1 =====
    story.append(p(S['h1'], '1. What Is Constellation?'))
    story.append(gold_hr())
    story.append(p(S['body'],
        'Constellation is a desktop knowledge management platform built for people who think in connected '
        'notes. It stores everything as standard Markdown files on your local file system — no cloud accounts, '
        'no vendor lock-in, no subscription required.'
    ))
    story.append(p(S['body'],
        'Constellation introduces the <b>Universe</b> — a portable, self-contained workspace that unifies '
        'multiple libraries of Markdown files, structured databases, AI assistance, task management, and '
        'calendar views into a single coherent experience. Where other tools give you a single notebook, '
        'Constellation gives you an interconnected system.'
    ))
    story.append(p(S['body_bold'],
        'Technical foundation: Tauri v2 (Rust backend) + SvelteKit + Svelte 5. Native performance, '
        'small binary size, full offline operation, no Electron overhead.'
    ))

    # ===== SECTION 2 =====
    story.append(p(S['h1'], '2. The Problem Constellation Solves'))
    story.append(gold_hr())
    story.append(p(S['body'],
        'Knowledge management today is fragmented. Users face a common set of problems regardless of '
        'which tool they use:'
    ))

    t2_headers = [
        Paragraph('Problem', S['th']),
        Paragraph('What Users Do Today', S['th']),
        Paragraph('The Cost', S['th'])
    ]
    t2_rows = [
        [p(S['body'], 'Notes scattered across tools'), p(S['body'], 'Manual copy-paste between apps'), p(S['body'], 'Lost connections, duplicated effort')],
        [p(S['body'], 'One notebook/library at a time'), p(S['body'], 'Close one project to open another'), p(S['body'], 'Context-switching, no cross-project search or linking')],
        [p(S['body'], 'Missing task management'), p(S['body'], 'Separate task app (Todoist, Things, etc.)'), p(S['body'], 'Tasks disconnected from the notes that created them')],
        [p(S['body'], 'No database views'), p(S['body'], 'Export to spreadsheets or use separate tools'), p(S['body'], 'Data lives outside the knowledge system')],
        [p(S['body'], 'Rigid table editing'), p(S['body'], 'Edit tables in a spreadsheet, paste back'), p(S['body'], 'Workflow interruption, no formulas in notes')],
        [p(S['body'], 'Calendar disconnected'), p(S['body'], 'Separate calendar app'), p(S['body'], 'Daily notes and tasks not visible in one place')],
        [p(S['body'], 'Template systems'), p(S['body'], 'Manual copy-paste or tool-specific syntax'), p(S['body'], 'Inconsistency, wasted setup time')],
        [p(S['body'], 'AI as an afterthought'), p(S['body'], 'Separate AI tool, copy context manually'), p(S['body'], 'No integration with your actual notes')],
        [p(S['body'], 'Importing from other tools'), p(S['body'], 'Manual conversion or format-specific scripts'), p(S['body'], 'Friction prevents migration, data stays trapped')],
    ]
    story.append(make_table(t2_headers, t2_rows, [avail * 0.25, avail * 0.35, avail * 0.40]))
    story.append(Spacer(1, 8))

    story.append(p(S['body'],
        'Some tools solve a few of these. None solve all of them. Users end up assembling a patchwork of '
        'apps, plugins, and workarounds — becoming systems integrators instead of knowledge workers.'
    ))
    story.append(p(S['body_bold'],
        'Constellation eliminates the integration tax. Every capability listed above is built in, tested '
        'together, and ships as a unified experience.'
    ))

    # Non-Destructive section
    story.append(p(S['h2'], 'Non-Destructive by Design'))
    story.append(p(S['body'],
        'Constellation is built on a foundational principle: <b>your files are never modified without your '
        'explicit action.</b> It reads your existing Markdown folders exactly as they are — it does '
        'not inject metadata, rewrite frontmatter, alter folder structures, or create hidden configuration '
        'files inside your libraries. Your Markdown files remain pure, portable, and fully compatible with '
        'any text editor or tool that reads standard Markdown.'
    ))
    story.append(p(S['body'],
        'This means adopting Constellation carries <b>zero risk</b>. Point it at your existing folders of '
        'notes, explore every feature, and if you ever decide to use a different tool — nothing has changed. '
        'There is no migration, no conversion, and no cleanup required. Constellation is a window into your '
        'knowledge, not a lock on it.'
    ))

    # ===== SECTION 3 =====
    story.append(p(S['h1'], '3. Core Architecture: The Universe Model'))
    story.append(gold_hr())
    story.append(p(S['body'],
        'Constellation\'s defining architectural concept is the <b>Universe</b> — a portable directory '
        'that owns all user configuration and workspace state, separate from your notes.'
    ))

    story.append(p(S['code'], 'MyUniverse/'))
    story.append(p(S['code'], '  universe.json          # Identity and metadata'))
    story.append(p(S['code'], '  libraries.json         # Registered library paths'))
    story.append(p(S['code'], '  settings.json          # All preferences'))
    story.append(p(S['code'], '  bookmarks.json         # Saved bookmarks'))
    story.append(p(S['code'], '  workspaces.json        # Tab layouts'))
    story.append(p(S['code'], '  property-types.json    # Custom property mappings'))
    story.append(p(S['code'], '  bases/                 # Workspace-level databases'))
    story.append(Spacer(1, 6))

    story.append(p(S['h3'], 'Why This Matters'))
    bullets = [
        '<b>Portability.</b> Copy the universe directory to another machine and everything follows — settings, bookmarks, workspaces, database definitions. The libraries themselves are just folders of Markdown files that live wherever you want.',
        '<b>Multi-library by design.</b> A universe can register any number of libraries. Search, Sky View, task scanning, backlinks, and databases all operate across library boundaries natively.',
        '<b>Hierarchy.</b> Universes can reference child universes, inheriting their libraries. A team lead\'s universe can include a shared team universe plus a personal universe — with circular reference prevention built in.',
        '<b>No lock-in.</b> The universe is JSON files in a folder. The libraries are Markdown files in folders. Walk away at any time — your notes are standard files that any tool can read.',
    ]
    for text in bullets:
        story.append(p(S['bullet'], f'&#8226;  {text}'))

    # ===== SECTION 4 =====
    story.append(p(S['h1'], '4. What Constellation Offers'))
    story.append(gold_hr())

    story.append(p(S['h2'], '4.1 Capabilities That Set Constellation Apart'))

    t4_headers = [
        Paragraph('Capability', S['th']),
        Paragraph('Details', S['th'])
    ]
    t4_rows = [
        [p(S['body'], '<b>True multi-library workspace</b>'), p(S['body'], 'Open, search, link, and graph across multiple libraries simultaneously in one window.')],
        [p(S['body'], '<b>Universe portability</b>'), p(S['body'], 'All configuration travels in a single portable directory. Move machines and your entire workspace follows.')],
        [p(S['body'], '<b>Child universes</b>'), p(S['body'], 'Compose workspaces hierarchically — a team library feeds into your personal universe automatically.')],
        [p(S['body'], '<b>Cross-library backlinks</b>'), p(S['body'], 'See which notes in <i>any</i> library link to the current note — not limited to a single library.')],
        [p(S['body'], '<b>Cross-library graph</b>'), p(S['body'], 'One knowledge graph showing connections across all your libraries.')],
        [p(S['body'], '<b>Unified task scanning</b>'), p(S['body'], 'Global Tasks view aggregates tasks from every library with filtering by library, priority, due date, and text search.')],
        [p(S['body'], '<b>Built-in Bases (databases)</b>'), p(S['body'], 'Non-destructive database views with table/card/list modes, filtering, sorting, inline editing — no external tools needed.')],
        [p(S['body'], '<b>Table formulas</b>'), p(S['body'], '=SUM(), =AVG(), =COUNT(), =MIN(), =MAX() with cell references and ranges, evaluated in-place inside your Markdown tables.')],
        [p(S['body'], '<b>Multi-provider AI</b>'), p(S['body'], 'OpenAI, Anthropic, Google Gemini, and Ollama (local) from one interface, with 8 pre-built skills — directly integrated with your notes.')],
        [p(S['body'], '<b>Second screen</b>'), p(S['body'], 'A fully independent secondary window that extends your workspace across two screens — edit, browse, view graphs, or manage tasks side by side with no limitations. Not just a reference pane; a complete second workspace.')],
        [p(S['body'], '<b>15 languages at launch</b>'), p(S['body'], 'English, Arabic, German, Spanish, French, Hebrew, Hindi, Japanese, Korean, Portuguese, Russian, Turkish, Urdu, Chinese, Farsi — all with full RTL support.')],
        [p(S['body'], '<b>Security layer</b>'), p(S['body'], 'Library encryption at rest, idle lock with PIN, API key storage in OS keyring.')],
        [p(S['body'], '<b>Non-destructive library access</b>'), p(S['body'], 'Never modifies library files without explicit user action. Zero-risk adoption — try Constellation and switch tools freely with no trace left behind.')],
    ]
    story.append(make_table(t4_headers, t4_rows, [avail * 0.30, avail * 0.70]))

    story.append(p(S['h2'], '4.2 Everything Built In'))
    story.append(p(S['body'],
        'Features that other tools require plugins, extensions, or external apps to achieve ship built into Constellation:'
    ))
    t42_headers = [
        Paragraph('Feature', S['th']),
        Paragraph('Traditional Answer', S['th']),
        Paragraph('Constellation (Built-In)', S['th'])
    ]
    t42_rows = [
        [p(S['body'], 'Structured queries'), p(S['body'], 'Plugin-based apps / external scripts'), p(S['body'], 'Native Lens query parser (TABLE, LIST, TASK, CALENDAR queries)')],
        [p(S['body'], 'Task management'), p(S['body'], 'Separate task apps or plugins'), p(S['body'], 'Library-wide scanning, toggle, due dates, priority, tags')],
        [p(S['body'], 'Calendar sidebar'), p(S['body'], 'Separate calendar plugins'), p(S['body'], 'Month view with note/task dots, daily note creation')],
        [p(S['body'], 'Advanced tables'), p(S['body'], 'Basic Markdown tables or spreadsheets'), p(S['body'], 'Row/column operations, sorting, move, formulas')],
        [p(S['body'], 'Templates'), p(S['body'], 'Manual copy-paste or plugin syntax'), p(S['body'], 'Template variables (date, time, title, folder, library, cursor)')],
        [p(S['body'], 'Note importing'), p(S['body'], 'Manual conversion scripts'), p(S['body'], '7 formats: Markdown folders, Notion, Bear, Evernote, HTML, CSV, Plain Text')],
        [p(S['body'], 'Backlinks panel'), p(S['body'], 'Basic or plugin-dependent'), p(S['body'], 'Enhanced with cross-library support and unlinked mentions')],
        [p(S['body'], 'Sky View'), p(S['body'], 'Single-library only in most tools'), p(S['body'], 'Cross-library nodes, force controls, grouping')],
        [p(S['body'], 'Tag browser'), p(S['body'], 'Basic implementations'), p(S['body'], 'Tag frequency analysis, library-wide aggregation')],
    ]
    story.append(make_table(t42_headers, t42_rows, [avail * 0.22, avail * 0.33, avail * 0.45]))

    story.append(p(S['h2'], '4.3 Import From Anywhere'))
    story.append(p(S['body'],
        'Constellation\'s built-in importer supports migration from:'
    ))
    t43_headers = [
        Paragraph('Source', S['th']),
        Paragraph('What Gets Imported', S['th'])
    ]
    t43_rows = [
        [p(S['body'], '<b>Markdown folders</b>'), p(S['body'], 'Direct library registration — no conversion needed')],
        [p(S['body'], '<b>Notion exports</b>'), p(S['body'], 'Cleans hex IDs, converts internal links to wikilinks')],
        [p(S['body'], '<b>Bear notes</b>'), p(S['body'], 'Converts Bear\'s format to standard Markdown')],
        [p(S['body'], '<b>Evernote (.enex)</b>'), p(S['body'], 'Converts ENML to Markdown, preserves tags and dates as frontmatter')],
        [p(S['body'], '<b>HTML files</b>'), p(S['body'], 'Converts to clean Markdown')],
        [p(S['body'], '<b>CSV files</b>'), p(S['body'], 'Each row becomes a note with frontmatter properties')],
        [p(S['body'], '<b>Plain text files</b>'), p(S['body'], 'Direct import with Markdown extension')],
    ]
    story.append(make_table(t43_headers, t43_rows, [avail * 0.30, avail * 0.70]))
    story.append(Spacer(1, 6))
    story.append(p(S['body'],
        'Your existing notes from any tool become first-class citizens in Constellation without losing '
        'structure or metadata.'
    ))

    story.append(p(S['h2'], '4.4 What Constellation Does Not Do (Yet)'))
    story.append(p(S['body'],
        'Transparency matters. These are capabilities not currently in Constellation:'
    ))
    t44_headers = [
        Paragraph('Feature', S['th']),
        Paragraph('Status', S['th'])
    ]
    t44_rows = [
        [p(S['body'], 'Mobile apps (iOS/Android)'), p(S['body'], 'Not yet — desktop only (Windows, macOS, Linux)')],
        [p(S['body'], 'Cloud sync'), p(S['body'], 'Not built-in — use Git, Syncthing, or any file sync solution')],
        [p(S['body'], 'Infinite canvas / whiteboard'), p(S['body'], 'Not yet')],
        [p(S['body'], 'PDF annotation'), p(S['body'], 'Not yet')],
        [p(S['body'], 'Audio recording'), p(S['body'], 'Not yet')],
        [p(S['body'], 'Third-party plugin API'), p(S['body'], 'Not yet')],
    ]
    story.append(make_table(t44_headers, t44_rows, [avail * 0.45, avail * 0.55]))

    # ===== SECTION 5 =====
    story.append(p(S['h1'], '5. Who Is Constellation For?'))
    story.append(gold_hr())

    personas = [
        ('5.1 The Multi-Project Professional',
         'Consultant, researcher, or knowledge worker who maintains separate note collections for different clients, projects, or life domains.',
         'Must close one project to open another. Cannot search across collections. Cannot see connections between a client\'s project notes and research notes in a separate folder.',
         'Register all libraries in one universe. Search, graph, task scan, and link across all of them simultaneously.'),
        ('5.2 The Tool-Fatigued Power User',
         'Power user running multiple apps and extensions who spends significant time managing updates, resolving conflicts, and debugging breakage.',
         'Every tool update is a risk. Task management, databases, templates, calendar, and AI are all separate systems maintained by different teams on different schedules.',
         'All of these are built-in, tested together, and updated as one unit. Zero extension management.'),
        ('5.3 The Arabic/RTL Knowledge Worker',
         'User who works primarily in Arabic, Hebrew, Farsi, or Urdu and needs a note-taking system that treats RTL as a first-class concern.',
         'RTL support is inconsistent in most tools. Editors assume LTR. Date keys and list keys don\'t recognize Arabic equivalents. UI elements break in mirrored layouts.',
         '15 languages including 4 RTL languages. Arabic property key detection (date, list, checkbox keys recognized in Arabic). Full UI mirroring. RTL-aware tables, forms, editors, and calendar.'),
        ('5.4 The Team Lead or Organization Builder',
         'Manager or team lead who wants to share a knowledge base with team members while maintaining personal notes separately.',
         'No concept of workspace composition in most tools. Shared note collections require manual setup per person.',
         'Create a team universe with shared libraries. Each team member adds the team universe as a child of their personal universe. Team libraries appear automatically alongside personal libraries.'),
        ('5.5 The AI-Augmented Researcher',
         'Researcher or student who wants AI assistance integrated directly into their note-taking workflow — summarization, Q&A, writing assistance, translation.',
         'Must use a separate AI tool, manually copy context, and paste results back. Or install competing AI extensions with inconsistent interfaces and separate API key management.',
         'One AI settings panel. Four provider options (including local Ollama for privacy). Eight pre-built skills. API keys stored in the OS keyring, not in plaintext config files.'),
        ('5.6 The Security-Conscious User',
         'Professional handling sensitive notes (legal, medical, financial, personal) who needs encryption and access control.',
         'Most note apps offer no built-in encryption, no idle lock, and store API keys in plaintext configuration files.',
         'Library encryption at rest, idle lock with PIN, API key storage in OS keyring.'),
        ('5.7 The Migrating User',
         'Someone moving away from Notion, Evernote, Bear, or another tool who wants to own their data locally without losing years of accumulated notes.',
         'Migration is painful. Export formats are inconsistent. Internal links break. Metadata gets lost. Many users stay locked in because switching costs are too high.',
         'Built-in importer handles 7 formats. Notion hex IDs are cleaned, links are converted to wikilinks, Evernote ENML becomes Markdown with frontmatter. One-click migration, zero data loss.'),
    ]

    for title, profile, pain, answer in personas:
        story.append(p(S['h2'], title))
        story.append(p(S['body'], f'<b>Profile:</b> {profile}'))
        story.append(p(S['body'], f'<b>Pain today:</b> {pain}'))
        story.append(p(S['body'], f'<b>Constellation answer:</b> {answer}'))

    # ===== SECTION 6 =====
    story.append(p(S['h1'], '6. Technical Advantages'))
    story.append(gold_hr())

    tech_items = [
        ('6.1 Performance',
         'Constellation\'s Rust backend performs file operations, link scanning, task extraction, and database '
         'queries at native speed. Heavy operations — library-wide task scanning, structured queries, link graph '
         'building — execute in the Rust process and return structured results to the frontend. The editor '
         'never competes with background processing for resources.'),
        ('6.2 Binary Size and Resource Usage',
         'Tauri v2 uses the system\'s native webview rather than bundling Chromium. The result is a significantly '
         'smaller binary and lower memory footprint compared to Electron-based alternatives.'),
        ('6.3 Security Model',
         'Tauri\'s Rust backend provides a natural security boundary. File system access is controlled through '
         'explicit Tauri commands — the frontend cannot access arbitrary files. Path traversal prevention is '
         'enforced at the Rust layer (canonicalization checks on all file operations).'),
        ('6.4 Data Sovereignty',
         'All data lives on the user\'s file system in standard formats: Markdown files with YAML frontmatter, '
         'JSON .base files for databases, JSON files in the universe directory for configuration, and standard '
         'image/PDF files in library folders for attachments. No telemetry. No cloud dependency. No account required.'),
    ]
    for title, body in tech_items:
        story.append(p(S['h2'], title))
        story.append(p(S['body'], body))

    # ===== SECTION 7 =====
    story.append(p(S['h1'], '7. Development Validation Criteria'))
    story.append(gold_hr())
    story.append(p(S['body'], 'This section defines how we measure whether Constellation fulfills its purpose. Each criterion maps to a testable capability.'))

    story.append(p(S['h2'], '7.1 Core Promise: "Your Notes, Your Way"'))
    v1_headers = [Paragraph('Test', S['th']), Paragraph('Expected Result', S['th'])]
    v1_rows = [
        [p(S['body'], 'Open any folder of Markdown files'), p(S['body'], 'All notes visible, frontmatter parsed, links resolved')],
        [p(S['body'], 'Edit a note and save'), p(S['body'], 'File on disk updates, readable by any Markdown tool')],
        [p(S['body'], 'Create a note with frontmatter'), p(S['body'], 'Valid YAML frontmatter, standard format')],
        [p(S['body'], 'Resolve [[wikilinks]]'), p(S['body'], 'Correct resolution across files and folders')],
        [p(S['body'], 'Render callouts, highlights, math, mermaid'), p(S['body'], 'Rich rendering of extended Markdown syntax')],
    ]
    story.append(make_table(v1_headers, v1_rows, [avail * 0.40, avail * 0.60]))

    story.append(p(S['h2'], '7.2 Multi-Library Promise: "A Universe of Libraries"'))
    v2_headers = [Paragraph('Test', S['th']), Paragraph('Expected Result', S['th'])]
    v2_rows = [
        [p(S['body'], 'Register 3+ libraries'), p(S['body'], 'All appear in file explorer with distinct colors')],
        [p(S['body'], 'Search across libraries'), p(S['body'], 'Results from all libraries, labeled by source')],
        [p(S['body'], 'Graph across libraries'), p(S['body'], 'Nodes from all libraries, cross-library edges visible')],
        [p(S['body'], 'Backlinks across libraries'), p(S['body'], 'Note in Library A shows backlinks from Library B')],
        [p(S['body'], 'Tasks across libraries'), p(S['body'], 'Global Tasks view aggregates all libraries')],
    ]
    story.append(make_table(v2_headers, v2_rows, [avail * 0.40, avail * 0.60]))

    story.append(p(S['h2'], '7.3 All-In-One Promise: "Everything Built In"'))
    v3_headers = [Paragraph('Test', S['th']), Paragraph('Expected Result', S['th'])]
    v3_rows = [
        [p(S['body'], 'Structured query in note'), p(S['body'], 'TABLE, LIST, TASK queries render results')],
        [p(S['body'], 'Task checkbox toggle'), p(S['body'], 'Toggle in sidebar updates file on disk')],
        [p(S['body'], 'Calendar dot indicators'), p(S['body'], 'Days with notes/tasks show visual indicators')],
        [p(S['body'], 'Table formula evaluation'), p(S['body'], '=SUM(A1:A5) calculates correctly')],
        [p(S['body'], 'Template insertion'), p(S['body'], 'Variables replaced with current date, time, title')],
        [p(S['body'], 'Import from Notion export'), p(S['body'], 'Hex IDs removed, links converted to wikilinks')],
    ]
    story.append(make_table(v3_headers, v3_rows, [avail * 0.40, avail * 0.60]))

    story.append(p(S['h2'], '7.4 User Experience Promise: "Works for Everyone"'))
    v4_headers = [Paragraph('Test', S['th']), Paragraph('Expected Result', S['th'])]
    v4_rows = [
        [p(S['body'], 'Switch to Arabic'), p(S['body'], 'Full UI in Arabic, RTL layout, mirrored sidebar')],
        [p(S['body'], 'Create note with Arabic properties'), p(S['body'], 'Date/list/checkbox keys detected correctly')],
        [p(S['body'], 'Open app on new machine with universe copy'), p(S['body'], 'All settings, bookmarks, workspaces restored')],
        [p(S['body'], 'Lock app, enter PIN'), p(S['body'], 'Notes inaccessible until correct PIN entered')],
    ]
    story.append(make_table(v4_headers, v4_rows, [avail * 0.40, avail * 0.60]))

    # ===== SECTION 8 =====
    story.append(p(S['h1'], '8. Competitive Landscape'))
    story.append(gold_hr())

    story.append(p(S['body'],
        'Constellation occupies a unique position in the knowledge management space: '
        '<b>local-first, multi-library, all-in-one, and multilingual.</b>'
    ))

    t8_headers = [
        Paragraph('Dimension', S['th']),
        Paragraph('Constellation', S['th']),
        Paragraph('Obsidian', S['th']),
        Paragraph('Notion', S['th']),
        Paragraph('Logseq', S['th']),
        Paragraph('Roam', S['th']),
        Paragraph('Bear', S['th']),
    ]
    col_w = avail / 7
    t8_rows = [
        [p(S['body'], 'Data ownership'), p(S['body'], 'Local files'), p(S['body'], 'Local files'), p(S['body'], 'Cloud-hosted'), p(S['body'], 'Local files'), p(S['body'], 'Cloud-hosted'), p(S['body'], 'iCloud')],
        [p(S['body'], 'Offline capability'), p(S['body'], 'Full'), p(S['body'], 'Full'), p(S['body'], 'Limited'), p(S['body'], 'Full'), p(S['body'], 'None'), p(S['body'], 'Full')],
        [p(S['body'], 'File format'), p(S['body'], 'Standard Markdown'), p(S['body'], 'Standard Markdown'), p(S['body'], 'Proprietary'), p(S['body'], 'Markdown/EDN'), p(S['body'], 'Proprietary'), p(S['body'], 'Proprietary')],
        [p(S['body'], 'Multi-library'), p(S['body'], 'Native (Universe)'), p(S['body'], 'One vault per window'), p(S['body'], 'N/A (workspaces)'), p(S['body'], 'Single graph'), p(S['body'], 'Single graph'), p(S['body'], 'N/A')],
        [p(S['body'], 'Cross-library search'), p(S['body'], 'Yes'), p(S['body'], 'No'), p(S['body'], 'N/A'), p(S['body'], 'No'), p(S['body'], 'No'), p(S['body'], 'No')],
        [p(S['body'], 'Cross-library graph'), p(S['body'], 'Yes'), p(S['body'], 'No'), p(S['body'], 'N/A'), p(S['body'], 'No'), p(S['body'], 'No'), p(S['body'], 'No')],
        [p(S['body'], 'Database views'), p(S['body'], 'Built-in (Bases)'), p(S['body'], 'Plugin required'), p(S['body'], 'Native'), p(S['body'], 'Queries (limited)'), p(S['body'], 'Queries'), p(S['body'], 'No')],
        [p(S['body'], 'Task management'), p(S['body'], 'Built-in'), p(S['body'], 'Plugin required'), p(S['body'], 'Basic'), p(S['body'], 'Plugin required'), p(S['body'], 'Basic'), p(S['body'], 'No')],
        [p(S['body'], 'AI integration'), p(S['body'], 'Built-in (4 providers)'), p(S['body'], 'Plugin required'), p(S['body'], 'Built-in (1 provider)'), p(S['body'], 'Plugin required'), p(S['body'], 'Plugin required'), p(S['body'], 'No')],
        [p(S['body'], 'Table formulas'), p(S['body'], 'Built-in'), p(S['body'], 'Plugin required'), p(S['body'], 'Limited'), p(S['body'], 'No'), p(S['body'], 'No'), p(S['body'], 'No')],
        [p(S['body'], 'Import sources'), p(S['body'], '7 formats built-in'), p(S['body'], 'Plugin required'), p(S['body'], 'Built-in (limited)'), p(S['body'], 'Limited'), p(S['body'], 'Limited'), p(S['body'], 'Limited')],
        [p(S['body'], 'RTL / Arabic'), p(S['body'], '15 languages, 4 RTL'), p(S['body'], 'Community effort'), p(S['body'], 'Limited'), p(S['body'], 'Limited'), p(S['body'], 'Limited'), p(S['body'], 'No')],
        [p(S['body'], 'Pricing'), p(S['body'], 'Free / Open Source'), p(S['body'], 'Freemium'), p(S['body'], 'Freemium + subscription'), p(S['body'], 'Free'), p(S['body'], 'Subscription'), p(S['body'], 'Subscription')],
        [p(S['body'], 'Architecture'), p(S['body'], 'Tauri (Rust + native webview)'), p(S['body'], 'Electron'), p(S['body'], 'Web app'), p(S['body'], 'Electron'), p(S['body'], 'Web app'), p(S['body'], 'Native (Apple)')],
    ]
    story.append(make_table(t8_headers, t8_rows, [col_w] * 7))

    story.append(p(S['h3'], 'Constellation\'s Position'))
    story.append(p(S['body'],
        'Constellation does not compete by being "a better version of X." It competes by being <b>a complete '
        'knowledge management platform</b> that eliminates the need to assemble a stack of tools. Users coming '
        'from any tool — or from no tool at all — can start with Constellation and have everything they need '
        'from day one.'
    ))
    story.append(p(S['body'],
        'For users of existing Markdown-based tools, the transition is seamless: point Constellation at your '
        'existing folders and everything works. For users of proprietary tools, the built-in importer handles '
        'the conversion.'
    ))

    # ===== SECTION 9 =====
    story.append(p(S['h1'], '9. Roadmap'))
    story.append(gold_hr())
    story.append(p(S['body'],
        'Based on this concept paper, the following development priorities align with Constellation\'s positioning:'
    ))

    story.append(p(S['h2'], 'High Priority (Reinforces Core Differentiators)'))
    for i, item in enumerate([
        'Polish multi-library experience — cross-library move/copy, library-scoped settings',
        'Bases performance at scale — handle 10,000+ note databases efficiently',
        'AI skill expansion — custom skill builder, context-aware library Q&amp;A',
        'Mobile companion — read-only library browser for iOS/Android',
    ], 1):
        story.append(p(S['bullet'], f'{i}.  {item}'))

    story.append(p(S['h2'], 'Medium Priority (Broadens Platform)'))
    for i, item in enumerate([
        'Canvas / whiteboard — infinite canvas with embedded notes',
        'PDF annotation — highlight and annotate PDFs within libraries',
        'Publish / static site export — generate websites from library content',
        'Constellation URI protocol — deep linking into specific notes and views',
    ], 5):
        story.append(p(S['bullet'], f'{i}.  {item}'))

    story.append(p(S['h2'], 'Lower Priority (Future Vision)'))
    for i, item in enumerate([
        'Audio recording and transcription',
        'Plugin API — allow third-party extensions (carefully scoped)',
        'Collaborative editing — real-time multi-user editing via CRDT',
    ], 9):
        story.append(p(S['bullet'], f'{i}.  {item}'))

    # ===== SECTION 10 =====
    story.append(p(S['h1'], '10. Conclusion'))
    story.append(gold_hr())
    story.append(p(S['body'],
        'Constellation exists because knowledge management should not require systems integration. A note-taking '
        'platform should ship with the tools its users need — databases, tasks, calendars, templates, importers, '
        'AI, and multi-library support — tested together, updated together, and usable out of the box.'
    ))
    story.append(p(S['body'],
        'For the knowledge worker who has built a workflow across multiple tools and feels the friction of '
        'managing that stack, Constellation offers a unified alternative that works with standard Markdown files, '
        'requires zero configuration, and provides capabilities no single existing tool offers — true multi-library '
        'workspaces, cross-library everything, and portable universe-based configuration.'
    ))
    story.append(p(S['body'],
        'The files are yours. The format is Markdown. The door is always open.'
    ))
    story.append(Spacer(1, 20))
    story.append(HRFlowable(width="30%", thickness=2, color=GOLD, spaceBefore=10, spaceAfter=10))

    oss_style = ParagraphStyle(
        'OSS', fontName='Arial-Italic', fontSize=10, textColor=LIGHT_TEXT,
        alignment=TA_CENTER, leading=15
    )
    story.append(p(oss_style, 'Constellation is open source under the MIT license.'))
    story.append(p(oss_style, 'Developed by Eisa ALSHAMSI'))
    story.append(p(oss_style, 'github.com/eisaAlshamsi/Constellation'))

    # ===== LEGAL NOTICE =====
    story.append(PageBreak())
    story.append(p(S['h1'], 'Legal Notice'))
    story.append(gold_hr())

    story.append(p(S['h2'], 'Trademark Acknowledgments'))
    disclaimer_style = ParagraphStyle(
        'Disclaimer', fontName='Arial', fontSize=9.5, textColor=LIGHT_TEXT,
        alignment=TA_JUSTIFY, spaceAfter=8, leading=15
    )
    story.append(Paragraph(
        'All product names, logos, and brands mentioned in this document are the property of their respective '
        'owners. "Obsidian" is a trademark of Dynalist Inc. "Notion" is a trademark of Notion Labs, Inc. '
        '"Bear" is a trademark of Shiny Frog Ltd. "Evernote" is a trademark of Bending Spoons S.p.A. '
        '"Logseq" is a trademark of Logseq, Inc. "Roam" is a trademark of Roam Research, Inc.',
        disclaimer_style
    ))
    story.append(Paragraph(
        'Constellation is an independent project and is not affiliated with, endorsed by, or sponsored by '
        'any of the companies mentioned above. All references to third-party products in this document are '
        'for purposes of factual comparison and interoperability description only, under nominative fair use.',
        disclaimer_style
    ))

    story.append(p(S['h2'], 'Intellectual Property Statement'))
    story.append(Paragraph(
        'Constellation is original software developed independently. It does not contain, incorporate, or '
        'derive from any third-party application source code. Constellation reads and writes standard Markdown '
        'files with YAML frontmatter \u2014 open, non-proprietary formats. Wikilink syntax ([[link]]) originates '
        'from wiki software and is not proprietary to any vendor. File-level interoperability with various '
        'Markdown-based tools is achieved through standard file system operations on open formats, not through '
        'reverse engineering or use of proprietary APIs.',
        disclaimer_style
    ))

    story.append(p(S['h2'], 'Open Source Compliance'))
    story.append(Paragraph(
        'Constellation is licensed under the MIT License. All third-party dependencies are used in compliance '
        'with their respective open source licenses. A full dependency audit is maintained in the project repository.',
        disclaimer_style
    ))

    # Build with custom canvas for page numbers
    doc.build(story, canvasmaker=NumberedCanvas)
    print(f"PDF generated: {output}")
    return output


if __name__ == '__main__':
    build_pdf()
