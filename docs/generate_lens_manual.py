"""
Generate Constellation Lens User Manual PDF with visual elements.
Includes drawn diagrams: nodes, edges, communities, gaps, health gauge.
"""

from reportlab.lib.pagesizes import A4
from reportlab.platypus import (
    SimpleDocTemplate, Paragraph, Spacer, Table, TableStyle,
    HRFlowable, Flowable, KeepTogether
)
from reportlab.lib.styles import getSampleStyleSheet, ParagraphStyle
from reportlab.lib.units import mm, cm
from reportlab.lib.colors import HexColor, Color, white, black
from reportlab.lib.enums import TA_LEFT, TA_CENTER
from reportlab.graphics.shapes import Drawing, Circle, Line, Ellipse, String, Rect
from reportlab.graphics import renderPDF
import math

# ─── Colors ───
PURPLE = HexColor('#7c3aed')
GREEN = HexColor('#16a34a')
AMBER = HexColor('#f59e0b')
RED = HexColor('#ef4444')
BLUE = HexColor('#60a5fa')
TEAL = HexColor('#34d399')
PINK = HexColor('#f472b6')
ORANGE = HexColor('#fb923c')
GRAY = HexColor('#94a3b8')
LIGHT_BG = HexColor('#f9f8ff')
BORDER = HexColor('#e5e7eb')

# ─── Custom Flowable for drawings ───
class LensDrawing(Flowable):
    def __init__(self, width, height, draw_func):
        Flowable.__init__(self)
        self.width = width
        self.height = height
        self.draw_func = draw_func

    def draw(self):
        self.draw_func(self.canv, self.width, self.height)


def draw_node_sizes(canv, w, h):
    """Draw large vs small nodes to explain centrality."""
    # Large node (bridge)
    canv.setFillColor(PURPLE)
    canv.circle(60, h - 30, 18, fill=1, stroke=0)
    canv.setFillColor(white)
    canv.setFont('Helvetica-Bold', 7)
    canv.drawCentredString(60, h - 33, 'BRIDGE')

    # Medium node
    canv.setFillColor(TEAL)
    canv.circle(140, h - 30, 10, fill=1, stroke=0)

    # Small node
    canv.setFillColor(BLUE)
    canv.circle(200, h - 30, 5, fill=1, stroke=0)

    # Tiny node
    canv.setFillColor(GRAY)
    canv.circle(240, h - 30, 3, fill=1, stroke=0)

    # Labels
    canv.setFillColor(black)
    canv.setFont('Helvetica', 8)
    canv.drawCentredString(60, h - 55, 'High centrality')
    canv.drawCentredString(140, h - 50, 'Medium')
    canv.drawCentredString(200, h - 45, 'Low')
    canv.drawCentredString(240, h - 43, 'Minimal')

    # Arrow
    canv.setStrokeColor(GRAY)
    canv.setLineWidth(0.5)
    canv.line(30, h - 60, 260, h - 60)
    canv.drawString(270, h - 63, '← Bridging importance →')


def draw_community_colors(canv, w, h):
    """Draw colored node groups to explain communities."""
    # Community 1 (purple cluster)
    canv.setFillColor(HexColor('#a78bfa'))
    canv.setStrokeColor(HexColor('#a78bfa'))
    canv.setLineWidth(0.3)
    for x, y, r in [(50, h-25, 8), (70, h-20, 6), (55, h-40, 5), (75, h-38, 7), (40, h-35, 4)]:
        canv.circle(x, y, r, fill=1, stroke=0)
    # Lines between them
    canv.setStrokeColor(HexColor('#c4b5fd'))
    canv.setLineWidth(0.5)
    canv.line(50, h-25, 70, h-20)
    canv.line(50, h-25, 55, h-40)
    canv.line(70, h-20, 75, h-38)
    canv.line(55, h-40, 75, h-38)

    # Community 2 (teal cluster)
    canv.setFillColor(TEAL)
    for x, y, r in [(180, h-22, 7), (200, h-28, 9), (195, h-42, 5), (215, h-35, 6)]:
        canv.circle(x, y, r, fill=1, stroke=0)
    canv.setStrokeColor(HexColor('#86efac'))
    canv.line(180, h-22, 200, h-28)
    canv.line(200, h-28, 195, h-42)
    canv.line(200, h-28, 215, h-35)

    # Community 3 (orange cluster)
    canv.setFillColor(ORANGE)
    for x, y, r in [(320, h-25, 6), (340, h-30, 8), (330, h-42, 5)]:
        canv.circle(x, y, r, fill=1, stroke=0)
    canv.setStrokeColor(HexColor('#fdba74'))
    canv.line(320, h-25, 340, h-30)
    canv.line(340, h-30, 330, h-42)

    # Labels
    canv.setFillColor(black)
    canv.setFont('Helvetica', 8)
    canv.drawCentredString(60, h-55, 'Community A')
    canv.drawCentredString(195, h-55, 'Community B')
    canv.drawCentredString(330, h-55, 'Community C')

    # Ellipse boundaries
    canv.setStrokeColor(HexColor('#a78bfa'))
    canv.setLineWidth(0.8)
    canv.setDash(3, 2)
    canv.ellipse(25, h-52, 95, h-10)
    canv.setStrokeColor(TEAL)
    canv.ellipse(165, h-52, 230, h-10)
    canv.setStrokeColor(ORANGE)
    canv.ellipse(305, h-52, 355, h-10)
    canv.setDash()


def draw_structural_gap(canv, w, h):
    """Draw a structural gap (red dashed line) between two communities."""
    # Community 1
    canv.setFillColor(PURPLE)
    for x, y, r in [(60, h-25, 8), (80, h-20, 6), (55, h-38, 5), (75, h-35, 7)]:
        canv.circle(x, y, r, fill=1, stroke=0)
    canv.setStrokeColor(HexColor('#c4b5fd'))
    canv.setLineWidth(0.5)
    canv.line(60, h-25, 80, h-20)
    canv.line(60, h-25, 55, h-38)
    canv.line(80, h-20, 75, h-35)

    # Gap (red dashed line)
    canv.setStrokeColor(RED)
    canv.setLineWidth(1.5)
    canv.setDash(6, 4)
    canv.line(110, h-30, 220, h-30)
    canv.setDash()

    # Community 2
    canv.setFillColor(TEAL)
    for x, y, r in [(250, h-22, 7), (270, h-28, 9), (260, h-40, 5), (280, h-36, 6)]:
        canv.circle(x, y, r, fill=1, stroke=0)
    canv.setStrokeColor(HexColor('#86efac'))
    canv.setLineWidth(0.5)
    canv.line(250, h-22, 270, h-28)
    canv.line(270, h-28, 260, h-40)
    canv.line(270, h-28, 280, h-36)

    # Labels
    canv.setFillColor(black)
    canv.setFont('Helvetica', 8)
    canv.drawCentredString(67, h-52, 'Islamic Heritage')
    canv.drawCentredString(265, h-52, 'Systems Engineering')
    canv.setFillColor(RED)
    canv.setFont('Helvetica-Bold', 8)
    canv.drawCentredString(165, h-18, 'STRUCTURAL GAP')
    canv.setFont('Helvetica', 7)
    canv.setFillColor(HexColor('#666666'))
    canv.drawCentredString(165, h-42, 'No connections — blind spot')


def draw_health_gauge(canv, w, h):
    """Draw the Universe Health gauge."""
    cx, cy = 80, h - 45
    r = 35

    # Background arc
    canv.setStrokeColor(BORDER)
    canv.setLineWidth(6)
    canv.arc(cx-r, cy-r, cx+r, cy+r, 0, 180)

    # Score arc (green for 87)
    canv.setStrokeColor(GREEN)
    canv.setLineWidth(6)
    canv.arc(cx-r, cy-r, cx+r, cy+r, 0, 180 * 0.87)

    # Score text
    canv.setFillColor(GREEN)
    canv.setFont('Helvetica-Bold', 20)
    canv.drawCentredString(cx, cy - 5, '87')
    canv.setFont('Helvetica', 7)
    canv.setFillColor(GRAY)
    canv.drawCentredString(cx, cy - 18, 'Universe Health')

    # Metric boxes
    metrics = [
        ('0.72', 'Modularity', 170),
        ('10%', 'Dominance', 240),
        ('5.50', 'Entropy', 310),
        ('2.7', 'Links/Note', 380),
    ]
    for val, label, x in metrics:
        canv.setFillColor(HexColor('#f5f3ff'))
        canv.roundRect(x - 28, h - 60, 56, 35, 4, fill=1, stroke=0)
        canv.setFillColor(black)
        canv.setFont('Helvetica-Bold', 11)
        canv.drawCentredString(x, h - 38, val)
        canv.setFillColor(GRAY)
        canv.setFont('Helvetica', 6)
        canv.drawCentredString(x, h - 50, label.upper())


def draw_legend(canv, w, h):
    """Draw the complete legend with all visual elements."""
    y = h - 15
    items = [
        ('large_circle', 'Large node', 'Bridge note — connects different knowledge areas'),
        ('small_circle', 'Small node', 'Peripheral — lives within a single topic'),
        ('colors', 'Node color', 'Each color = auto-detected topic cluster'),
        ('solid_line', 'Solid line', 'Wikilink between two notes'),
        ('dashed_line', 'Red dashed', 'Structural gap — blind spot between areas'),
        ('ellipse', 'Colored region', 'Community boundary — dense topic cluster'),
    ]
    for icon_type, title, desc in items:
        # Icon
        if icon_type == 'large_circle':
            canv.setFillColor(PURPLE)
            canv.circle(20, y, 8, fill=1, stroke=0)
        elif icon_type == 'small_circle':
            canv.setFillColor(BLUE)
            canv.circle(20, y, 4, fill=1, stroke=0)
        elif icon_type == 'colors':
            canv.setFillColor(HexColor('#a78bfa'))
            canv.circle(12, y, 4, fill=1, stroke=0)
            canv.setFillColor(TEAL)
            canv.circle(20, y, 4, fill=1, stroke=0)
            canv.setFillColor(ORANGE)
            canv.circle(28, y, 4, fill=1, stroke=0)
        elif icon_type == 'solid_line':
            canv.setStrokeColor(GRAY)
            canv.setLineWidth(1.5)
            canv.line(8, y, 32, y)
        elif icon_type == 'dashed_line':
            canv.setStrokeColor(RED)
            canv.setLineWidth(1.5)
            canv.setDash(4, 3)
            canv.line(8, y, 32, y)
            canv.setDash()
        elif icon_type == 'ellipse':
            canv.setStrokeColor(PURPLE)
            canv.setLineWidth(1)
            canv.setFillColor(HexColor('#f5f3ff'))
            canv.ellipse(6, y - 6, 34, y + 6, fill=1, stroke=1)

        # Text
        canv.setFillColor(black)
        canv.setFont('Helvetica-Bold', 9)
        canv.drawString(45, y + 2, title)
        canv.setFillColor(HexColor('#666666'))
        canv.setFont('Helvetica', 8)
        canv.drawString(45, y - 10, desc)

        y -= 28


# ─── Build PDF ───
doc = SimpleDocTemplate(
    'docs/Constellation_Lens_User_Manual.pdf',
    pagesize=A4,
    leftMargin=20*mm, rightMargin=20*mm,
    topMargin=15*mm, bottomMargin=15*mm,
)

styles = getSampleStyleSheet()
styles.add(ParagraphStyle('MainTitle', parent=styles['Title'], fontSize=24, textColor=PURPLE, spaceAfter=4))
styles.add(ParagraphStyle('Subtitle', parent=styles['Normal'], fontSize=11, textColor=GRAY, alignment=TA_CENTER, spaceAfter=16))
styles.add(ParagraphStyle('H2', parent=styles['Heading2'], fontSize=15, textColor=HexColor('#333'), spaceBefore=20, spaceAfter=8))
styles.add(ParagraphStyle('H3', parent=styles['Heading3'], fontSize=12, textColor=HexColor('#555'), spaceBefore=14, spaceAfter=6))
styles.add(ParagraphStyle('Body', parent=styles['Normal'], fontSize=10, leading=14, spaceAfter=6))
styles.add(ParagraphStyle('BodyBold', parent=styles['Normal'], fontSize=10, leading=14, spaceAfter=6, fontName='Helvetica-Bold'))
styles.add(ParagraphStyle('Quote', parent=styles['Normal'], fontSize=9, leading=13, leftIndent=12, textColor=HexColor('#555'), spaceAfter=8, backColor=LIGHT_BG))
styles.add(ParagraphStyle('Tip', parent=styles['Normal'], fontSize=9, leading=13, leftIndent=12, textColor=HexColor('#4338ca'), spaceAfter=8))
styles.add(ParagraphStyle('Footer', parent=styles['Normal'], fontSize=8, textColor=GRAY, alignment=TA_CENTER))

story = []

# Title page
story.append(Spacer(1, 40*mm))
story.append(Paragraph('Constellation Lens', styles['MainTitle']))
story.append(Paragraph('User Manual', styles['Subtitle']))
story.append(Spacer(1, 10*mm))
story.append(Paragraph('A Network Analysis Engine for Knowledge Discovery', styles['Subtitle']))
story.append(Spacer(1, 5*mm))
story.append(Paragraph('Cognitive Engine — Layer 3', styles['Subtitle']))
story.append(Spacer(1, 20*mm))
story.append(Paragraph('Version 1.0 — April 2026', styles['Footer']))
story.append(Paragraph('uConstellation.world', styles['Footer']))

# Page 2: What is it?
story.append(Spacer(1, 30*mm))
story.append(Paragraph('What Is the Constellation Lens?', styles['H2']))
story.append(Paragraph(
    'Imagine looking at a city from above at night. Some buildings have many roads connecting them — they are hubs. '
    'Some neighborhoods are dense with activity, while others sit isolated. And between some neighborhoods, there are '
    'no roads at all — blind spots that could benefit from a bridge.', styles['Body']))
story.append(Paragraph(
    'The Constellation Lens does this for your knowledge. It takes your notes and the links between them and analyzes '
    'their structure using algorithms from network science. It answers: <b>"What patterns and gaps exist in my thinking?"</b>', styles['Body']))
story.append(Paragraph(
    'It does not tell you what to think. It shows you the shape of what you already know — where your knowledge is '
    'deep, where it is shallow, and where two areas could connect but do not yet.', styles['Body']))

# Visual Legend
story.append(Paragraph('Visual Legend', styles['H2']))
story.append(Paragraph('Every element in the Lens graph has meaning:', styles['Body']))
story.append(LensDrawing(450, 180, draw_legend))

# Node sizes
story.append(Paragraph('Node Size = Bridging Importance', styles['H3']))
story.append(Paragraph(
    'The Lens resizes every node based on its <b>betweenness centrality</b> — a measure of how many shortest paths '
    'between other notes pass through it. A note with only 3 links can be a critical bridge if it connects two large '
    'topic clusters that would otherwise be disconnected.', styles['Body']))
story.append(LensDrawing(450, 65, draw_node_sizes))

# Community colors
story.append(Paragraph('Node Color = Topic Community', styles['H3']))
story.append(Paragraph(
    'The Lens uses the <b>Louvain algorithm</b> to automatically detect topic clusters (communities) from your link '
    'patterns — not from folders or tags. Each community gets a distinct color. Notes of the same color are densely '
    'connected to each other.', styles['Body']))
story.append(LensDrawing(400, 65, draw_community_colors))

# Structural gaps
story.append(Paragraph('Structural Gaps = Blind Spots', styles['H3']))
story.append(Paragraph(
    'The most distinctive insight: the Lens identifies pairs of communities with <b>high internal density but low '
    'inter-community connections</b>. These are blind spots — areas where two relevant knowledge domains exist but '
    'lack bridges between them. Based on Ronald Burt\'s structural holes theory (1992).', styles['Body']))
story.append(LensDrawing(400, 65, draw_structural_gap))

# Universe Health
story.append(Paragraph('Universe Health', styles['H2']))
story.append(Paragraph(
    'A composite score (0-100) measuring the cognitive diversity of your knowledge base:', styles['Body']))
story.append(LensDrawing(430, 70, draw_health_gauge))
story.append(Spacer(1, 4*mm))

health_data = [
    ['Component', 'What It Measures', 'Healthy Range'],
    ['Modularity', 'How distinct your topic clusters are', '0.3 – 0.6'],
    ['Dominance', '% of notes in the largest community', 'Below 35%'],
    ['Entropy', 'How evenly knowledge is distributed', 'Above 2.0 bits'],
    ['Links/Note', 'Average connections per note', 'Above 1.0'],
]
t = Table(health_data, colWidths=[80, 250, 90])
t.setStyle(TableStyle([
    ('BACKGROUND', (0, 0), (-1, 0), HexColor('#f5f3ff')),
    ('FONTNAME', (0, 0), (-1, 0), 'Helvetica-Bold'),
    ('FONTSIZE', (0, 0), (-1, -1), 9),
    ('GRID', (0, 0), (-1, -1), 0.5, BORDER),
    ('VALIGN', (0, 0), (-1, -1), 'TOP'),
    ('TOPPADDING', (0, 0), (-1, -1), 5),
    ('BOTTOMPADDING', (0, 0), (-1, -1), 5),
]))
story.append(t)

# How to use
story.append(Paragraph('How to Use', styles['H2']))
steps = [
    '1. Click the <b>Lens button</b> in the left dock bar (magnifier with plus sign)',
    '2. Wait a few seconds while the analysis runs',
    '3. The Lens view appears: graph on the left, analytics panel on the right',
    '4. Explore: check the health score, review the top bridges, examine communities',
    '5. Look for blind spots — each is a research opportunity',
    '6. Click any bridge note to open it in the editor',
    '7. Close: click × or press Escape',
]
for s in steps:
    story.append(Paragraph(s, styles['Body']))

# Tips
story.append(Paragraph('Tips', styles['H2']))
tips = [
    '<b>Start with the health score.</b> If it is low, check dominance and entropy to understand why.',
    '<b>Check the blind spots.</b> Each gap between communities is a potential research question.',
    '<b>Watch the bridges.</b> Your top bridge notes are the structural backbone — develop them further.',
    '<b>Use layer peeling.</b> Hide dominant MOC notes to reveal the organic structure underneath.',
    '<b>The Lens works better with more links.</b> The more wikilinks you create, the richer the analysis.',
]
for tip in tips:
    story.append(Paragraph(tip, styles['Tip']))

# Advanced
story.append(Paragraph('Advanced Features', styles['H2']))
story.append(Paragraph('<b>Layer Peeling:</b> Temporarily hide the top 1-20 most central notes. '
    'This reveals the secondary structure beneath dominant MOC and index notes.', styles['Body']))
story.append(Paragraph('<b>Tag Edges:</b> Toggle on to reveal implicit connections between notes '
    'sharing tags but no wikilinks. Often reveals connections you have been making unconsciously.', styles['Body']))

# Technical
story.append(Paragraph('How It Works', styles['H2']))
story.append(Paragraph(
    '<b>Betweenness Centrality</b> (Brandes\' algorithm, 2001): For each note, counts how many '
    'shortest paths between all other note pairs pass through it. Computed in Rust. O(VE) complexity.', styles['Body']))
story.append(Paragraph(
    '<b>Community Detection</b> (Louvain algorithm): Groups notes into clusters by maximizing '
    'modularity. Computed in JavaScript.', styles['Body']))
story.append(Paragraph(
    '<b>Structural Gap Detection</b> (Burt\'s structural holes, 1992): Identifies community pairs '
    'with high internal density but low inter-community connections.', styles['Body']))
story.append(Paragraph(
    'All computation runs <b>locally on your machine</b>. No data leaves your device.', styles['Body']))

# Footer
story.append(Spacer(1, 20*mm))
story.append(HRFlowable(width='100%', thickness=0.5, color=BORDER))
story.append(Paragraph('Constellation Lens User Manual v1.0 — April 2026 — uConstellation.world', styles['Footer']))

doc.build(story)
print('PDF generated: docs/Constellation_Lens_User_Manual.pdf')
