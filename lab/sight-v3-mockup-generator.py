"""
Constellation Sight v3 — celestial-hemisphere mock-up generator.

Generates 4 layout options, each as a separate SVG in docs/:
  Option A — Polar grid          (refinement of v0; cleaner spread)
  Option B — Stratum rings       (concentric Living-Link bands)
  Option C — Hemispheric pair    (two domes: core / periphery)
  Option D — Suwaidi-faithful    (explicit homage; rectangular IAU-style borders)

Color scheme matches Suwaidi star chart (cream parchment background,
near-black stars, blue ink for labels, red ink for date-range rim,
cyan + gold reference rings, soft gray Milky Way).
"""

from __future__ import annotations
import math
import random
from pathlib import Path

# ─── Canvas ──────────────────────────────────────────────────────────
W, H = 1600, 1600
CX, CY = W / 2, H / 2
DOME_R = 660
RIM_INNER = 670
RIM_MID = 692
RIM_OUTER = 720

# ─── Suwaidi-reference palette (light theme) ────────────────────────
BG = "#faf6e8"             # warm cream parchment
INK = "#1a1a1a"            # near-black for stars + body text
INK_SOFT = "#3a3a3a"       # softer near-black for connector lines
RULE_FAINT = "#b8a98a"     # faded sand for grid + borders
GOLD = "#c9a227"           # ecliptic ring, score, current-year highlight
CYAN = "#2b8fa8"           # equator/reference ring
RED_INK = "#a83232"        # date-range rim text (like Suwaidi outer rim)
BLUE_INK = "#2a4a8c"       # constellation labels + title (Suwaidi ink color)
MILKY = "#e6dec0"          # Milky Way cloud (slightly darker than BG)

# Star magnitudes — solid black with alpha modulation
STAR_FILL = INK

# ─── Universe shape (matches Eisa's actual data 2026-05-07) ──────────
N_NOTES = 7636
N_COMMUNITIES = 20
HEALTH_SCORE = 91
N_EDGES = 217108
YEARS = [2019, 2020, 2021, 2022, 2023, 2024, 2025, 2026]
MONTHS = ["Jan", "Feb", "Mar", "Apr", "May", "Jun",
          "Jul", "Aug", "Sep", "Oct", "Nov", "Dec"]

# ─── Helpers ─────────────────────────────────────────────────────────

def polar(r: float, theta_rad: float, cx: float = CX, cy: float = CY) -> tuple[float, float]:
    """Polar to Cartesian. theta=0 is at TOP (12 o'clock), CW positive."""
    return cx + r * math.sin(theta_rad), cy - r * math.cos(theta_rad)


# ─── Shared drawing primitives ───────────────────────────────────────

def magnitude_size(centrality: float) -> float:
    """6 visual magnitudes, log-ish distributed."""
    if centrality > 0.85: return 6.0
    if centrality > 0.65: return 4.0
    if centrality > 0.40: return 2.5
    if centrality > 0.20: return 1.5
    if centrality > 0.08: return 0.9
    return 0.5


def magnitude_alpha(centrality: float) -> float:
    """Faint stars are slightly translucent; bright stars solid."""
    return min(0.55 + centrality * 0.45, 1.0)


def gen_milky_way(orient_deg: float = 35.0) -> str:
    """Soft cloud band approximating TF-IDF content-similarity density."""
    return f'''
    <defs>
      <radialGradient id="mw1" cx="50%" cy="50%" r="50%">
        <stop offset="0%" stop-color="{MILKY}" stop-opacity="0.95" />
        <stop offset="60%" stop-color="{MILKY}" stop-opacity="0.55" />
        <stop offset="100%" stop-color="{MILKY}" stop-opacity="0" />
      </radialGradient>
      <radialGradient id="mw2" cx="50%" cy="50%" r="50%">
        <stop offset="0%" stop-color="{MILKY}" stop-opacity="0.7" />
        <stop offset="100%" stop-color="{MILKY}" stop-opacity="0" />
      </radialGradient>
    </defs>
    <ellipse cx="{CX - 100}" cy="{CY - 80}" rx="320" ry="180"
             fill="url(#mw1)" transform="rotate({orient_deg} {CX - 100} {CY - 80})" />
    <ellipse cx="{CX + 180}" cy="{CY + 60}" rx="240" ry="140"
             fill="url(#mw2)" transform="rotate({-orient_deg} {CX + 180} {CY + 60})" />
    '''


def gen_year_track(years: list[int] = YEARS, current: int = 2026) -> list[str]:
    """Outer rim: year wedges (red ink, like Suwaidi date-range markers)."""
    parts = [f'<circle cx="{CX}" cy="{CY}" r="{RIM_OUTER}" fill="none" '
             f'stroke="{INK}" stroke-opacity="0.55" stroke-width="0.8" />',
             f'<circle cx="{CX}" cy="{CY}" r="{RIM_MID}" fill="none" '
             f'stroke="{INK}" stroke-opacity="0.4" stroke-width="0.6" />']
    n = len(years)
    arc = 2 * math.pi / n
    for i, year in enumerate(years):
        theta_start = i * arc
        theta_mid = theta_start + arc / 2
        x0, y0 = polar(RIM_MID, theta_start)
        x1, y1 = polar(RIM_OUTER, theta_start)
        parts.append(f'<line x1="{x0:.1f}" y1="{y0:.1f}" x2="{x1:.1f}" y2="{y1:.1f}" '
                     f'stroke="{INK}" stroke-opacity="0.45" stroke-width="0.6" />')
        is_current = (year == current)
        font_size = 14 if is_current else 12
        weight = 600 if is_current else 400
        x_text, y_text = polar((RIM_MID + RIM_OUTER) / 2, theta_mid)
        parts.append(f'<text x="{x_text:.1f}" y="{y_text:.1f}" '
                     f'font-size="{font_size}" fill="{RED_INK}" font-weight="{weight}" '
                     f'text-anchor="middle" dominant-baseline="middle" '
                     f'font-family="serif">{year}</text>')
    return parts


def gen_month_track(current_idx: int = 4) -> list[str]:
    """Inner rim: 12-month wedges (black ink, near-pole text)."""
    parts = [f'<circle cx="{CX}" cy="{CY}" r="{RIM_INNER}" fill="none" '
             f'stroke="{INK}" stroke-opacity="0.3" stroke-width="0.5" />']
    arc = 2 * math.pi / 12
    for i, month in enumerate(MONTHS):
        theta_start = i * arc
        theta_mid = theta_start + arc / 2
        x0, y0 = polar(RIM_INNER, theta_start)
        x1, y1 = polar(RIM_MID, theta_start)
        parts.append(f'<line x1="{x0:.1f}" y1="{y0:.1f}" x2="{x1:.1f}" y2="{y1:.1f}" '
                     f'stroke="{INK}" stroke-opacity="0.25" stroke-width="0.4" />')
        is_current = (i == current_idx)
        font_size = 11 if is_current else 9
        weight = 600 if is_current else 400
        fill = GOLD if is_current else INK
        x_text, y_text = polar((RIM_INNER + RIM_MID) / 2, theta_mid)
        parts.append(f'<text x="{x_text:.1f}" y="{y_text:.1f}" '
                     f'font-size="{font_size}" fill="{fill}" font-weight="{weight}" '
                     f'fill-opacity="0.85" text-anchor="middle" dominant-baseline="middle" '
                     f'font-family="serif">{month}</text>')
    return parts


def gen_health_card_corners() -> list[str]:
    """Score roundel at top + four metrics at four corners (cream theme)."""
    parts = []
    parts.append(f'<circle cx="{CX}" cy="90" r="58" fill="{BG}" '
                 f'stroke="{GOLD}" stroke-width="2.5" />')
    parts.append(f'<text x="{CX}" y="86" font-size="38" fill="{GOLD}" '
                 f'text-anchor="middle" dominant-baseline="middle" font-weight="600" '
                 f'font-family="serif">{HEALTH_SCORE}</text>')
    parts.append(f'<text x="{CX}" y="114" font-size="11" fill="{INK}" '
                 f'fill-opacity="0.6" text-anchor="middle" font-family="serif">/ 100</text>')
    parts.append(f'<text x="{CX}" y="138" font-size="9" fill="{INK}" '
                 f'fill-opacity="0.55" text-anchor="middle" letter-spacing="2.0" '
                 f'font-family="serif">UNIVERSE HEALTH</text>')

    metrics = [
        ("MODULARITY", "0.63", "CAUTION", "#c9831f", 80, 200),
        ("DOMINANCE",  "18%",  "HEALTHY", "#3a8a4a", W - 80, 200),
        ("ENTROPY",    "3.62", "HEALTHY", "#3a8a4a", 80, H - 200),
        ("CONNECTIVITY", "28.43", "HEALTHY", "#3a8a4a", W - 80, H - 200),
    ]
    for label, value, status, status_color, x, y in metrics:
        anchor = "start" if x < CX else "end"
        parts.append(f'<text x="{x:.1f}" y="{y:.1f}" font-size="10" '
                     f'fill="{INK}" fill-opacity="0.55" letter-spacing="1.5" '
                     f'text-anchor="{anchor}" font-family="serif">{label}</text>')
        parts.append(f'<text x="{x:.1f}" y="{y + 24:.1f}" font-size="22" '
                     f'fill="{INK}" fill-opacity="0.92" font-weight="500" '
                     f'text-anchor="{anchor}" font-family="serif">{value}</text>')
        parts.append(f'<text x="{x:.1f}" y="{y + 44:.1f}" font-size="9" '
                     f'fill="{status_color}" fill-opacity="0.95" '
                     f'letter-spacing="1.5" font-weight="600" '
                     f'text-anchor="{anchor}" font-family="serif">{status}</text>')
    return parts


def gen_corner_labels(option_label: str) -> list[str]:
    """Top-left: title + author. Top-right: counts."""
    parts = []
    parts.append(f'<text x="80" y="80" font-size="22" fill="{BLUE_INK}" '
                 f'font-weight="600" font-family="serif">Constellation Sight</text>')
    parts.append(f'<text x="80" y="105" font-size="11" fill="{BLUE_INK}" '
                 f'fill-opacity="0.7" font-style="italic" font-family="serif">'
                 f'@uconstellation.world · v3 mock-up · {option_label}</text>')
    parts.append(f'<text x="{W - 80}" y="80" font-size="11" fill="{INK}" '
                 f'fill-opacity="0.7" text-anchor="end" letter-spacing="0.5" '
                 f'font-family="serif">'
                 f'{N_NOTES:,} notes · {N_EDGES:,} edges · {N_COMMUNITIES} communities</text>')
    return parts


# ─── OPTION A — Polar grid (refined) ─────────────────────────────────

def gen_stars_polar() -> list[dict]:
    rng = random.Random(42)
    raw_sizes = [N_NOTES * (0.85 ** i) * (0.5 + 0.5 * rng.random())
                 for i in range(N_COMMUNITIES)]
    raw_sizes[0] = max(int(raw_sizes[0]), 1500)
    total = sum(raw_sizes)
    scale = N_NOTES / total
    sizes = [max(5, int(s * scale)) for s in raw_sizes]
    wedge = 2 * math.pi / N_COMMUNITIES
    stars = []
    for c_idx in range(N_COMMUNITIES):
        wedge_start = c_idx * wedge
        for _ in range(sizes[c_idx]):
            u = rng.random()
            centrality = u ** 2.5
            radius = (1 - centrality) ** 0.6 * (DOME_R - 30) + 8
            t = rng.random()
            angle = wedge_start + (0.06 + 0.88 * t) * wedge
            stars.append({
                "x": CX + radius * math.sin(angle),
                "y": CY - radius * math.cos(angle),
                "size": magnitude_size(centrality),
                "alpha": magnitude_alpha(centrality),
                "community": c_idx,
                "centrality": centrality,
            })
    return stars


def gen_constellation_lines(stars: list[dict]) -> list[tuple[dict, dict]]:
    rng = random.Random(7)
    by_c: dict[int, list[dict]] = {}
    for s in stars:
        by_c.setdefault(s["community"], []).append(s)
    lines = []
    for members in by_c.values():
        if len(members) < 3: continue
        bright = sorted(members, key=lambda s: -s["centrality"])[:10]
        for i in range(min(len(bright) - 1, 6)):
            lines.append((bright[i], bright[i + 1]))
        for _ in range(min(3, len(bright) // 3)):
            a, b = rng.choice(bright), rng.choice(bright)
            if a is not b: lines.append((a, b))
    return lines


def build_option_a() -> str:
    """Polar grid. Stars positioned by centrality-radius + community-angle."""
    stars = gen_stars_polar()
    lines = gen_constellation_lines(stars)

    out = [
        f'<?xml version="1.0" encoding="UTF-8"?>',
        f'<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {W} {H}" width="{W}" height="{H}">',
        f'<rect width="{W}" height="{H}" fill="{BG}" />',
    ]
    out.append(gen_milky_way())

    # Polar grid: 4 declination rings
    for ring in [0.25, 0.5, 0.75, 1.0]:
        r = ring * DOME_R
        op = 0.18 if ring < 1.0 else 0.40
        out.append(f'<circle cx="{CX}" cy="{CY}" r="{r:.1f}" fill="none" '
                   f'stroke="{RULE_FAINT}" stroke-opacity="{op}" stroke-width="0.5" />')

    # Community wedge spokes (very faint)
    for c_idx in range(N_COMMUNITIES):
        theta = c_idx * 2 * math.pi / N_COMMUNITIES
        x0, y0 = polar(50, theta)
        x1, y1 = polar(DOME_R - 5, theta)
        out.append(f'<line x1="{x0:.1f}" y1="{y0:.1f}" x2="{x1:.1f}" y2="{y1:.1f}" '
                   f'stroke="{RULE_FAINT}" stroke-opacity="0.18" stroke-width="0.4" />')

    # Two reference rings: gold (ecliptic-equivalent) + cyan (equator-equivalent)
    out.append(f'<circle cx="{CX}" cy="{CY}" r="{0.4 * DOME_R:.1f}" fill="none" '
               f'stroke="{GOLD}" stroke-opacity="0.55" stroke-width="0.8" stroke-dasharray="4 3" />')
    out.append(f'<circle cx="{CX}" cy="{CY}" r="{0.7 * DOME_R:.1f}" fill="none" '
               f'stroke="{CYAN}" stroke-opacity="0.50" stroke-width="0.8" stroke-dasharray="2 4" />')

    # Constellation lines
    out.append('<g stroke-linecap="round">')
    for a, b in lines:
        out.append(f'<line x1="{a["x"]:.1f}" y1="{a["y"]:.1f}" x2="{b["x"]:.1f}" y2="{b["y"]:.1f}" '
                   f'stroke="{INK_SOFT}" stroke-opacity="0.30" stroke-width="0.5" />')
    out.append('</g>')

    # Stars
    out.append('<g>')
    for s in stars:
        out.append(f'<circle cx="{s["x"]:.1f}" cy="{s["y"]:.1f}" r="{s["size"]:.2f}" '
                   f'fill="{STAR_FILL}" fill-opacity="{s["alpha"]:.2f}" />')
    out.append('</g>')

    out.extend(gen_year_track())
    out.extend(gen_month_track())
    out.append(f'<circle cx="{CX}" cy="{CY}" r="{DOME_R}" fill="none" '
               f'stroke="{INK}" stroke-opacity="0.3" stroke-width="0.7" />')
    out.extend(gen_health_card_corners())
    out.extend(gen_corner_labels("Option A — Polar grid"))
    out.append('</svg>')
    return "\n".join(out)


# ─── OPTION B — Stratum concentric rings ─────────────────────────────

def build_option_b() -> str:
    """Concentric Living-Link stratum rings. Each stage gets a band:
    spark (innermost) → birth → growth → maturity → dormancy → archival.
    Stars positioned in their stratum ring; community is angular."""
    rng = random.Random(43)
    out = [
        f'<?xml version="1.0" encoding="UTF-8"?>',
        f'<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {W} {H}" width="{W}" height="{H}">',
        f'<rect width="{W}" height="{H}" fill="{BG}" />',
    ]
    out.append(gen_milky_way())

    # 6 stratum rings — stars distribute by stage
    strata = ["Spark", "Birth", "Growth", "Maturity", "Dormancy", "Archival"]
    ring_outer = [0.18, 0.32, 0.50, 0.68, 0.84, 1.00]  # outer radius of each band
    ring_colors = [GOLD, GOLD, CYAN, CYAN, RULE_FAINT, RULE_FAINT]  # subtle band tint

    # Soft band fills
    for i, (label, r_out) in enumerate(zip(strata, ring_outer)):
        r_in = ring_outer[i - 1] if i > 0 else 0
        if i > 0:
            # subtle band fill
            out.append(f'<path d="M {CX - r_out * DOME_R:.1f} {CY} '
                       f'A {r_out * DOME_R:.1f} {r_out * DOME_R:.1f} 0 1 0 {CX + r_out * DOME_R:.1f} {CY} '
                       f'A {r_out * DOME_R:.1f} {r_out * DOME_R:.1f} 0 1 0 {CX - r_out * DOME_R:.1f} {CY} Z '
                       f'M {CX - r_in * DOME_R:.1f} {CY} '
                       f'A {r_in * DOME_R:.1f} {r_in * DOME_R:.1f} 0 1 1 {CX + r_in * DOME_R:.1f} {CY} '
                       f'A {r_in * DOME_R:.1f} {r_in * DOME_R:.1f} 0 1 1 {CX - r_in * DOME_R:.1f} {CY} Z" '
                       f'fill="{ring_colors[i]}" fill-opacity="0.04" fill-rule="evenodd" />')

    # Ring boundaries
    for r_frac in ring_outer:
        out.append(f'<circle cx="{CX}" cy="{CY}" r="{r_frac * DOME_R:.1f}" fill="none" '
                   f'stroke="{INK}" stroke-opacity="0.18" stroke-width="0.6" stroke-dasharray="6 4" />')

    # Stage labels at the rim of each ring (12 o'clock position)
    for i, (label, r_out) in enumerate(zip(strata, ring_outer)):
        r_label = (ring_outer[i - 1] if i > 0 else 0) * DOME_R + (r_out - (ring_outer[i - 1] if i > 0 else 0)) * DOME_R / 2
        x, y = polar(r_label, math.radians(-2))  # slightly off 12 o'clock
        out.append(f'<text x="{x:.1f}" y="{y:.1f}" font-size="10" fill="{INK}" '
                   f'fill-opacity="0.55" letter-spacing="1.5" font-style="italic" '
                   f'text-anchor="start" dominant-baseline="middle" font-family="serif">{label}</text>')

    # Wedge spokes
    for c_idx in range(N_COMMUNITIES):
        theta = c_idx * 2 * math.pi / N_COMMUNITIES
        x0, y0 = polar(50, theta)
        x1, y1 = polar(DOME_R - 5, theta)
        out.append(f'<line x1="{x0:.1f}" y1="{y0:.1f}" x2="{x1:.1f}" y2="{y1:.1f}" '
                   f'stroke="{RULE_FAINT}" stroke-opacity="0.15" stroke-width="0.4" />')

    # Stars: assigned to a stratum + community
    raw_sizes = [N_NOTES * (0.85 ** i) * (0.5 + 0.5 * rng.random())
                 for i in range(N_COMMUNITIES)]
    raw_sizes[0] = max(int(raw_sizes[0]), 1500)
    total = sum(raw_sizes)
    scale = N_NOTES / total
    sizes = [max(5, int(s * scale)) for s in raw_sizes]
    wedge = 2 * math.pi / N_COMMUNITIES
    # Stratum distribution — power law biased toward early stages
    stratum_weights = [0.05, 0.10, 0.30, 0.30, 0.20, 0.05]

    stars = []
    for c_idx in range(N_COMMUNITIES):
        wedge_start = c_idx * wedge
        for _ in range(sizes[c_idx]):
            u = rng.random()
            cum = 0
            stratum_idx = 0
            for i, w in enumerate(stratum_weights):
                cum += w
                if u < cum:
                    stratum_idx = i
                    break
            r_in = (ring_outer[stratum_idx - 1] if stratum_idx > 0 else 0) * DOME_R
            r_out = ring_outer[stratum_idx] * DOME_R
            radius = r_in + (0.10 + 0.80 * rng.random()) * (r_out - r_in)
            t = rng.random()
            angle = wedge_start + (0.06 + 0.88 * t) * wedge
            centrality = (1 - stratum_idx / 5.0) * (0.3 + 0.7 * rng.random())
            stars.append({
                "x": CX + radius * math.sin(angle),
                "y": CY - radius * math.cos(angle),
                "size": magnitude_size(centrality),
                "alpha": magnitude_alpha(centrality),
                "community": c_idx,
                "centrality": centrality,
                "stratum": stratum_idx,
            })

    # Sparse intra-community lines (within same wedge, similar stratum)
    by_c: dict[int, list[dict]] = {}
    for s in stars:
        by_c.setdefault(s["community"], []).append(s)
    out.append('<g stroke-linecap="round">')
    for members in by_c.values():
        bright = sorted(members, key=lambda s: -s["centrality"])[:8]
        for i in range(min(len(bright) - 1, 5)):
            a, b = bright[i], bright[i + 1]
            out.append(f'<line x1="{a["x"]:.1f}" y1="{a["y"]:.1f}" x2="{b["x"]:.1f}" y2="{b["y"]:.1f}" '
                       f'stroke="{INK_SOFT}" stroke-opacity="0.28" stroke-width="0.5" />')
    out.append('</g>')

    # Stars
    out.append('<g>')
    for s in stars:
        out.append(f'<circle cx="{s["x"]:.1f}" cy="{s["y"]:.1f}" r="{s["size"]:.2f}" '
                   f'fill="{STAR_FILL}" fill-opacity="{s["alpha"]:.2f}" />')
    out.append('</g>')

    out.extend(gen_year_track())
    out.extend(gen_month_track())
    out.append(f'<circle cx="{CX}" cy="{CY}" r="{DOME_R}" fill="none" '
               f'stroke="{INK}" stroke-opacity="0.3" stroke-width="0.7" />')
    out.extend(gen_health_card_corners())
    out.extend(gen_corner_labels("Option B — Stratum rings (Living-Link bands)"))
    out.append('</svg>')
    return "\n".join(out)


# ─── OPTION C — Hemispheric pair ─────────────────────────────────────

def build_option_c() -> str:
    """Two domes side-by-side. Left = core (top centrality / mature stratum).
    Right = periphery (low centrality / fleeting). Each dome has its own
    polar layout + rim. Health card at very top spans the gap."""
    # Reduce dome radius so two fit
    half_w = W / 2
    sub_dome_r = 380
    sub_rim_inner = 388
    sub_rim_mid = 405
    sub_rim_outer = 425
    cx_l = half_w / 2 + 30
    cx_r = half_w + half_w / 2 - 30
    cy = H / 2 + 40
    rng = random.Random(44)

    out = [
        f'<?xml version="1.0" encoding="UTF-8"?>',
        f'<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {W} {H}" width="{W}" height="{H}">',
        f'<rect width="{W}" height="{H}" fill="{BG}" />',
    ]

    def draw_hemisphere(cx: float, cy: float, label: str, sub_label: str,
                        members_count: int, is_core: bool):
        # Milky Way
        out.append(f'''
          <ellipse cx="{cx - 50}" cy="{cy - 30}" rx="170" ry="95"
                   fill="{MILKY}" fill-opacity="0.55" transform="rotate(28 {cx - 50} {cy - 30})" />
          <ellipse cx="{cx + 80}" cy="{cy + 50}" rx="120" ry="70"
                   fill="{MILKY}" fill-opacity="0.40" transform="rotate(-22 {cx + 80} {cy + 50})" />
        ''')
        # Polar grid
        for ring in [0.33, 0.66, 1.0]:
            r = ring * sub_dome_r
            op = 0.18 if ring < 1.0 else 0.40
            out.append(f'<circle cx="{cx}" cy="{cy}" r="{r:.1f}" fill="none" '
                       f'stroke="{RULE_FAINT}" stroke-opacity="{op}" stroke-width="0.5" />')
        # Wedges (10 per hemisphere, since 20 communities split between core/periphery)
        for c_idx in range(10):
            theta = c_idx * 2 * math.pi / 10
            x0, y0 = polar(30, theta, cx, cy)
            x1, y1 = polar(sub_dome_r - 5, theta, cx, cy)
            out.append(f'<line x1="{x0:.1f}" y1="{y0:.1f}" x2="{x1:.1f}" y2="{y1:.1f}" '
                       f'stroke="{RULE_FAINT}" stroke-opacity="0.15" stroke-width="0.4" />')
        # Reference rings
        out.append(f'<circle cx="{cx}" cy="{cy}" r="{0.5 * sub_dome_r:.1f}" fill="none" '
                   f'stroke="{GOLD if is_core else CYAN}" stroke-opacity="0.4" '
                   f'stroke-width="0.7" stroke-dasharray="3 3" />')

        # Stars
        wedge = 2 * math.pi / 10
        stars = []
        for c_idx in range(10):
            wedge_start = c_idx * wedge
            n = int(members_count * (0.92 ** c_idx) * (0.5 + 0.5 * rng.random()))
            n = max(8, n)
            for _ in range(n):
                u = rng.random()
                centrality = (u ** 2.5) if is_core else (u ** 1.5 * 0.6)
                radius = (1 - centrality) ** 0.6 * (sub_dome_r - 25) + 6
                t = rng.random()
                angle = wedge_start + (0.06 + 0.88 * t) * wedge
                stars.append({
                    "x": cx + radius * math.sin(angle),
                    "y": cy - radius * math.cos(angle),
                    "size": magnitude_size(centrality) * 0.85,
                    "alpha": magnitude_alpha(centrality),
                    "community": c_idx,
                    "centrality": centrality,
                })

        # Lines
        by_c: dict[int, list[dict]] = {}
        for s in stars:
            by_c.setdefault(s["community"], []).append(s)
        for members in by_c.values():
            bright = sorted(members, key=lambda s: -s["centrality"])[:6]
            for i in range(min(len(bright) - 1, 4)):
                a, b = bright[i], bright[i + 1]
                out.append(f'<line x1="{a["x"]:.1f}" y1="{a["y"]:.1f}" x2="{b["x"]:.1f}" y2="{b["y"]:.1f}" '
                           f'stroke="{INK_SOFT}" stroke-opacity="0.28" stroke-width="0.5" />')

        for s in stars:
            out.append(f'<circle cx="{s["x"]:.1f}" cy="{s["y"]:.1f}" r="{s["size"]:.2f}" '
                       f'fill="{STAR_FILL}" fill-opacity="{s["alpha"]:.2f}" />')

        # Rim — simplified single track (just months)
        out.append(f'<circle cx="{cx}" cy="{cy}" r="{sub_rim_inner}" fill="none" '
                   f'stroke="{INK}" stroke-opacity="0.3" stroke-width="0.5" />')
        out.append(f'<circle cx="{cx}" cy="{cy}" r="{sub_rim_outer}" fill="none" '
                   f'stroke="{INK}" stroke-opacity="0.5" stroke-width="0.7" />')
        for i, m in enumerate(MONTHS):
            theta_mid = (i + 0.5) * 2 * math.pi / 12
            x_text, y_text = polar((sub_rim_inner + sub_rim_outer) / 2, theta_mid, cx, cy)
            out.append(f'<text x="{x_text:.1f}" y="{y_text:.1f}" font-size="9" fill="{INK}" '
                       f'fill-opacity="0.7" text-anchor="middle" dominant-baseline="middle" '
                       f'font-family="serif">{m}</text>')

        # Hemisphere label below
        out.append(f'<text x="{cx:.1f}" y="{cy + sub_rim_outer + 35}" '
                   f'font-size="22" fill="{BLUE_INK}" font-weight="600" '
                   f'text-anchor="middle" font-family="serif">{label}</text>')
        out.append(f'<text x="{cx:.1f}" y="{cy + sub_rim_outer + 58}" '
                   f'font-size="11" fill="{INK}" fill-opacity="0.65" '
                   f'text-anchor="middle" font-style="italic" font-family="serif">{sub_label}</text>')

    draw_hemisphere(cx_l, cy, "Core", "high centrality · maturity / canonical strata", N_NOTES // 4, is_core=True)
    draw_hemisphere(cx_r, cy, "Periphery", "low centrality · spark / dormancy / archival strata", N_NOTES // 6, is_core=False)

    # Health card spans the top
    out.append(f'<circle cx="{CX}" cy="80" r="48" fill="{BG}" '
               f'stroke="{GOLD}" stroke-width="2" />')
    out.append(f'<text x="{CX}" y="78" font-size="32" fill="{GOLD}" '
               f'text-anchor="middle" dominant-baseline="middle" font-weight="600" '
               f'font-family="serif">{HEALTH_SCORE}</text>')
    out.append(f'<text x="{CX}" y="100" font-size="9" fill="{INK}" fill-opacity="0.6" '
               f'text-anchor="middle" font-family="serif">/ 100</text>')
    out.append(f'<text x="{CX}" y="138" font-size="9" fill="{INK}" fill-opacity="0.55" '
               f'text-anchor="middle" letter-spacing="2" font-family="serif">UNIVERSE HEALTH</text>')

    # Health metrics in a row below the score
    metrics_row = [
        ("MOD", "0.63", "#c9831f"),
        ("DOM", "18%",  "#3a8a4a"),
        ("ENT", "3.62", "#3a8a4a"),
        ("CON", "28.43","#3a8a4a"),
    ]
    for i, (k, v, col) in enumerate(metrics_row):
        x = CX - 240 + i * 160
        out.append(f'<text x="{x}" y="160" font-size="10" fill="{INK}" fill-opacity="0.55" '
                   f'letter-spacing="1.5" text-anchor="middle" font-family="serif">{k}</text>')
        out.append(f'<text x="{x}" y="180" font-size="18" fill="{INK}" font-weight="500" '
                   f'text-anchor="middle" font-family="serif">{v}</text>')
        out.append(f'<circle cx="{x}" cy="194" r="3" fill="{col}" />')

    out.extend(gen_corner_labels("Option C — Hemispheric pair (core / periphery)"))
    out.append('</svg>')
    return "\n".join(out)


# ─── OPTION D — Suwaidi-faithful ─────────────────────────────────────

def build_option_d() -> str:
    """Explicit homage to the Suwaidi reference — IAU-style rectangular
    constellation borders aligned to the polar grid, tighter rim with
    date-range markers, formal legend block."""
    rng = random.Random(45)
    out = [
        f'<?xml version="1.0" encoding="UTF-8"?>',
        f'<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {W} {H}" width="{W}" height="{H}">',
        f'<rect width="{W}" height="{H}" fill="{BG}" />',
    ]
    out.append(gen_milky_way(orient_deg=42))

    # Dense polar grid: 6 declination rings + 36 RA spokes (every 10°)
    for ring in [0.16, 0.33, 0.50, 0.66, 0.83, 1.0]:
        r = ring * DOME_R
        op = 0.20 if ring < 1.0 else 0.45
        out.append(f'<circle cx="{CX}" cy="{CY}" r="{r:.1f}" fill="none" '
                   f'stroke="{CYAN}" stroke-opacity="{op}" stroke-width="0.4" />')
    for i in range(36):
        theta = i * math.pi / 18
        x0, y0 = polar(0, theta)
        x1, y1 = polar(DOME_R, theta)
        out.append(f'<line x1="{x0:.1f}" y1="{y0:.1f}" x2="{x1:.1f}" y2="{y1:.1f}" '
                   f'stroke="{CYAN}" stroke-opacity="0.10" stroke-width="0.3" />')

    # Reference rings: equator (cyan, solid) + ecliptic (yellow, solid)
    out.append(f'<circle cx="{CX}" cy="{CY}" r="{0.66 * DOME_R:.1f}" fill="none" '
               f'stroke="{CYAN}" stroke-opacity="0.85" stroke-width="1.2" />')
    out.append(f'<circle cx="{CX + 60}" cy="{CY - 30}" r="{0.55 * DOME_R:.1f}" fill="none" '
               f'stroke="{GOLD}" stroke-opacity="0.85" stroke-width="1.2" />')

    # IAU-style rectangular community borders — built from polar grid arcs + radial spokes
    # 20 communities → arrange on polar grid: 4 declination bands × 5 angular sectors
    # Some communities span multiple cells, like real IAU regions
    rng2 = random.Random(101)
    band_radii = [0.1, 0.36, 0.62, 0.85, 1.0]
    # Generate ~20 IAU-like rectangular regions
    region_rects = []
    for band_idx in range(4):  # 4 bands
        n_in_band = 5 if band_idx < 2 else (4 if band_idx == 2 else 6)
        slice_arc = 2 * math.pi / n_in_band
        for s_idx in range(n_in_band):
            theta_start = s_idx * slice_arc
            theta_end = (s_idx + 1) * slice_arc
            r_in = band_radii[band_idx] * DOME_R
            r_out = band_radii[band_idx + 1] * DOME_R
            region_rects.append((r_in, r_out, theta_start, theta_end))

    for r_in, r_out, t_start, t_end in region_rects:
        # Build a polygonal rectangle in polar coords
        steps = max(2, int((t_end - t_start) / 0.1))
        path = []
        path.append(f"M {polar(r_in, t_start)[0]:.1f} {polar(r_in, t_start)[1]:.1f}")
        for i in range(steps + 1):
            t = t_start + (t_end - t_start) * i / steps
            x, y = polar(r_in, t)
            path.append(f"L {x:.1f} {y:.1f}")
        for i in range(steps + 1):
            t = t_end - (t_end - t_start) * i / steps
            x, y = polar(r_out, t)
            path.append(f"L {x:.1f} {y:.1f}")
        path.append("Z")
        out.append(f'<path d="{" ".join(path)}" fill="none" '
                   f'stroke="{INK}" stroke-opacity="0.18" stroke-width="0.6" stroke-dasharray="3 3" />')

    # Stars: assigned to rectangular regions
    stars = []
    for region_idx, (r_in, r_out, t_start, t_end) in enumerate(region_rects):
        n = int(N_NOTES / len(region_rects) * (0.4 + 1.6 * rng.random()))
        for _ in range(n):
            t = rng.random()
            radius = r_in + (0.1 + 0.8 * rng.random()) * (r_out - r_in)
            angle = t_start + (0.05 + 0.9 * rng.random()) * (t_end - t_start)
            centrality = rng.random() ** 2.5
            stars.append({
                "x": CX + radius * math.sin(angle),
                "y": CY - radius * math.cos(angle),
                "size": magnitude_size(centrality),
                "alpha": magnitude_alpha(centrality),
                "region": region_idx,
                "centrality": centrality,
            })

    # Constellation lines: chains within regions
    by_r: dict[int, list[dict]] = {}
    for s in stars:
        by_r.setdefault(s["region"], []).append(s)
    out.append('<g stroke-linecap="round">')
    for members in by_r.values():
        bright = sorted(members, key=lambda s: -s["centrality"])[:6]
        for i in range(min(len(bright) - 1, 4)):
            a, b = bright[i], bright[i + 1]
            out.append(f'<line x1="{a["x"]:.1f}" y1="{a["y"]:.1f}" x2="{b["x"]:.1f}" y2="{b["y"]:.1f}" '
                       f'stroke="{INK}" stroke-opacity="0.50" stroke-width="0.6" />')
    out.append('</g>')

    out.append('<g>')
    for s in stars:
        out.append(f'<circle cx="{s["x"]:.1f}" cy="{s["y"]:.1f}" r="{s["size"]:.2f}" '
                   f'fill="{STAR_FILL}" fill-opacity="{s["alpha"]:.2f}" />')
    out.append('</g>')

    # Rim with date-range markers (Suwaidi-style)
    # 36 segments of 10° each, labeled with date ranges
    out.append(f'<circle cx="{CX}" cy="{CY}" r="{RIM_OUTER}" fill="none" '
               f'stroke="{INK}" stroke-opacity="0.6" stroke-width="0.8" />')
    out.append(f'<circle cx="{CX}" cy="{CY}" r="{RIM_INNER}" fill="none" '
               f'stroke="{INK}" stroke-opacity="0.4" stroke-width="0.5" />')
    # Degree numbers on inner ring
    for i in range(36):
        theta = i * math.pi / 18
        x0, y0 = polar(RIM_INNER, theta)
        x1, y1 = polar(RIM_OUTER, theta)
        op = 0.6 if i % 3 == 0 else 0.3
        sw = 0.7 if i % 3 == 0 else 0.4
        out.append(f'<line x1="{x0:.1f}" y1="{y0:.1f}" x2="{x1:.1f}" y2="{y1:.1f}" '
                   f'stroke="{INK}" stroke-opacity="{op}" stroke-width="{sw}" />')
    # Year labels at every 30° (aligned to RA 0, 2h, 4h, ... 22h positions = 12 outer wedges)
    for i, year in enumerate(YEARS[-8:]):
        theta_mid = (i + 0.5) * 2 * math.pi / 8
        x_text, y_text = polar((RIM_INNER + RIM_OUTER) / 2 + 8, theta_mid)
        is_current = (year == 2026)
        out.append(f'<text x="{x_text:.1f}" y="{y_text:.1f}" font-size="13" '
                   f'fill="{RED_INK}" font-weight="{600 if is_current else 400}" '
                   f'text-anchor="middle" dominant-baseline="middle" font-family="serif">{year}</text>')
    # Inner: degree marks at every 30° (RA hour positions)
    for hr in range(12):
        theta_mid = hr * 2 * math.pi / 12
        x_text, y_text = polar(RIM_INNER - 14, theta_mid)
        out.append(f'<text x="{x_text:.1f}" y="{y_text:.1f}" font-size="9" '
                   f'fill="{RED_INK}" fill-opacity="0.7" '
                   f'text-anchor="middle" dominant-baseline="middle" font-family="serif">{hr * 2}h</text>')

    out.append(f'<circle cx="{CX}" cy="{CY}" r="{DOME_R}" fill="none" '
               f'stroke="{INK}" stroke-opacity="0.5" stroke-width="0.8" />')

    # Health metrics integrated into a Suwaidi-style legend block at top-right corner
    legend_x = W - 280
    legend_y = 200
    out.append(f'<text x="{legend_x}" y="{legend_y}" font-size="11" fill="{BLUE_INK}" '
               f'font-weight="600" letter-spacing="1.5" font-family="serif">UNIVERSE HEALTH</text>')
    out.append(f'<text x="{legend_x}" y="{legend_y + 28}" font-size="32" fill="{GOLD}" '
               f'font-weight="600" font-family="serif">{HEALTH_SCORE}<tspan font-size="14" fill="{INK}" fill-opacity="0.6"> / 100</tspan></text>')
    metrics = [
        ("Modularity", "0.63", "caution", "#c9831f"),
        ("Dominance",  "18%",  "healthy", "#3a8a4a"),
        ("Entropy",    "3.62", "healthy", "#3a8a4a"),
        ("Connectivity","28.43","healthy", "#3a8a4a"),
    ]
    for i, (k, v, status, col) in enumerate(metrics):
        y = legend_y + 78 + i * 22
        out.append(f'<text x="{legend_x}" y="{y}" font-size="11" fill="{INK}" '
                   f'fill-opacity="0.85" font-family="serif">{k}</text>')
        out.append(f'<text x="{legend_x + 130}" y="{y}" font-size="11" fill="{INK}" '
                   f'font-weight="500" text-anchor="end" font-family="serif">{v}</text>')
        out.append(f'<text x="{legend_x + 200}" y="{y}" font-size="9" fill="{col}" '
                   f'font-weight="500" letter-spacing="1" text-anchor="end" font-family="serif">{status.upper()}</text>')

    # Author + title (Suwaidi-style, top-left)
    out.append(f'<text x="80" y="80" font-size="22" fill="{BLUE_INK}" '
               f'font-weight="600" font-family="serif">Constellation Sight</text>')
    out.append(f'<text x="80" y="105" font-size="12" fill="{BLUE_INK}" '
               f'fill-opacity="0.85" font-style="italic" font-family="serif">Northern celestial section · @uconstellation.world</text>')
    out.append(f'<text x="80" y="125" font-size="10" fill="{INK}" fill-opacity="0.55" '
               f'font-family="serif">Option D — Suwaidi-faithful · v3 mock-up 2026-05-07</text>')

    out.append('</svg>')
    return "\n".join(out)


# ─── OPTION A v2 — Polar grid + 5-mode toggle UI ──────────────────────

def build_option_a_toggle() -> str:
    """Eisa's pick (2026-05-07): Option A polar grid as the base, plus the
    5-mode toggle UI (R · L · C · S · A) shown along the bottom.

    Default mode = Regions (rim wedges = libraries, sized by note count,
    empty wedges compressed out per Eisa's rule). The 5 thumbnails along
    the bottom are BOTH a preview AND the toggle controls — clicking a
    thumbnail switches the rim axis with a 600ms eased migration.

    Time is preserved as a slim right-edge year ladder (orthogonal,
    independent of the rim axis). Health card stays in the top-right
    corner. Per concept paper §6.3, only Regions and Link Types are
    backed by current data — Confidence / Stages / Acts ship as
    'available later' until their packs land."""

    W2, H2 = 1600, 1920
    CX2, CY2 = W2 / 2, 820
    DOME_R2 = 540
    RIM_INNER2 = DOME_R2 + 18
    RIM_OUTER2 = DOME_R2 + 70

    rng = random.Random(42)
    out = [
        '<?xml version="1.0" encoding="UTF-8"?>',
        f'<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {W2} {H2}" width="{W2}" height="{H2}">',
        f'<rect width="{W2}" height="{H2}" fill="{BG}" />',
    ]

    # Milky Way (custom defs to avoid clashing with shared helpers)
    out.append(f'''
      <defs>
        <radialGradient id="mw-toggle-l" cx="50%" cy="50%" r="50%">
          <stop offset="0%" stop-color="{MILKY}" stop-opacity="0.92" />
          <stop offset="60%" stop-color="{MILKY}" stop-opacity="0.50" />
          <stop offset="100%" stop-color="{MILKY}" stop-opacity="0" />
        </radialGradient>
        <radialGradient id="mw-toggle-r" cx="50%" cy="50%" r="50%">
          <stop offset="0%" stop-color="{MILKY}" stop-opacity="0.65" />
          <stop offset="100%" stop-color="{MILKY}" stop-opacity="0" />
        </radialGradient>
      </defs>
      <ellipse cx="{CX2 - 100}" cy="{CY2 - 80}" rx="290" ry="170"
               fill="url(#mw-toggle-l)" transform="rotate(35 {CX2 - 100} {CY2 - 80})" />
      <ellipse cx="{CX2 + 180}" cy="{CY2 + 60}" rx="230" ry="125"
               fill="url(#mw-toggle-r)" transform="rotate(-35 {CX2 + 180} {CY2 + 60})" />
    ''')

    # Polar declination rings
    for ring in [0.25, 0.5, 0.75, 1.0]:
        r = ring * DOME_R2
        op = 0.18 if ring < 1.0 else 0.40
        out.append(f'<circle cx="{CX2}" cy="{CY2}" r="{r:.1f}" fill="none" '
                   f'stroke="{RULE_FAINT}" stroke-opacity="{op}" stroke-width="0.5" />')

    # Reference rings: gold (ecliptic) + cyan (equator)
    out.append(f'<circle cx="{CX2}" cy="{CY2}" r="{0.4 * DOME_R2:.1f}" fill="none" '
               f'stroke="{GOLD}" stroke-opacity="0.55" stroke-width="0.8" stroke-dasharray="4 3" />')
    out.append(f'<circle cx="{CX2}" cy="{CY2}" r="{0.7 * DOME_R2:.1f}" fill="none" '
               f'stroke="{CYAN}" stroke-opacity="0.50" stroke-width="0.8" stroke-dasharray="2 4" />')

    # Demo libraries (Regions) with proportional wedges
    libraries = [
        ("Research", 0.32, 7),
        ("Reading",  0.25, 5),
        ("Daily",    0.18, 4),
        ("Projects", 0.15, 3),
        ("Drafts",   0.10, 1),
    ]
    total_w = sum(w for _, w, _ in libraries)
    lib_arcs = []
    acc = 0.0
    for name, w, n_c in libraries:
        arc_size = 2 * math.pi * w / total_w
        lib_arcs.append((name, acc, acc + arc_size, n_c))
        acc += arc_size

    # Region wedge spokes (faint, from interior to rim)
    for name, t_start, t_end, _ in lib_arcs:
        x0, y0 = polar(40, t_start, CX2, CY2)
        x1, y1 = polar(DOME_R2 - 5, t_start, CX2, CY2)
        out.append(f'<line x1="{x0:.1f}" y1="{y0:.1f}" x2="{x1:.1f}" y2="{y1:.1f}" '
                   f'stroke="{RULE_FAINT}" stroke-opacity="0.30" stroke-width="0.5" />')

    # Stars: distribute across libraries × communities
    stars = []
    community_counter = 0
    for name, t_start, t_end, n_c in lib_arcs:
        wedge_size = (t_end - t_start)
        sub_wedge = wedge_size / n_c
        notes_per_lib = int(N_NOTES * (t_end - t_start) / (2 * math.pi))
        sub_sizes = [int(notes_per_lib * (0.85 ** i) * (0.5 + 0.5 * rng.random()))
                     for i in range(n_c)]
        sub_total = sum(sub_sizes) or 1
        scale = notes_per_lib / sub_total
        sub_sizes = [max(3, int(s * scale)) for s in sub_sizes]
        for c_local in range(n_c):
            sub_start = t_start + c_local * sub_wedge
            for _ in range(sub_sizes[c_local]):
                u = rng.random()
                centrality = u ** 2.5
                radius = (1 - centrality) ** 0.6 * (DOME_R2 - 25) + 8
                t = rng.random()
                angle = sub_start + (0.04 + 0.92 * t) * sub_wedge
                stars.append({
                    "x": CX2 + radius * math.sin(angle),
                    "y": CY2 - radius * math.cos(angle),
                    "size": magnitude_size(centrality),
                    "alpha": magnitude_alpha(centrality),
                    "community": community_counter + c_local,
                    "centrality": centrality,
                })
        community_counter += n_c

    # Resting state: edges hidden by default. Per concept paper §4.1 —
    # links surface only when a node is hovered or clicked. Below we draw
    # ONE demo selected node with its outgoing edges to illustrate the
    # active state. Everything else stays clean.
    out.append('<g>')
    for s in stars:
        out.append(f'<circle cx="{s["x"]:.1f}" cy="{s["y"]:.1f}" r="{s["size"]:.2f}" '
                   f'fill="{STAR_FILL}" fill-opacity="{s["alpha"]:.2f}" />')
    out.append('</g>')

    # ── Demo: selected node + its edges (illustrates hover/click state) ──
    bright_sorted = sorted(stars, key=lambda s: -s["centrality"])
    demo_star = bright_sorted[3] if len(bright_sorted) > 3 else bright_sorted[0]
    demo_rng = random.Random(99)

    def dist_to_demo(s: dict) -> float:
        return math.hypot(s["x"] - demo_star["x"], s["y"] - demo_star["y"])

    candidates_near = [s for s in stars if 80 < dist_to_demo(s) < 250 and s is not demo_star]
    candidates_mid = [s for s in stars if 250 < dist_to_demo(s) < 500 and s is not demo_star]
    candidates_far = [s for s in stars if dist_to_demo(s) > 550 and s is not demo_star]
    related = []
    if candidates_near:
        related.extend(demo_rng.sample(candidates_near, min(3, len(candidates_near))))
    if candidates_mid:
        related.extend(demo_rng.sample(candidates_mid, min(3, len(candidates_mid))))
    if candidates_far:
        related.extend(demo_rng.sample(candidates_far, min(2, len(candidates_far))))

    out.append('<g stroke-linecap="round">')
    for r in related:
        out.append(f'<line x1="{demo_star["x"]:.1f}" y1="{demo_star["y"]:.1f}" '
                   f'x2="{r["x"]:.1f}" y2="{r["y"]:.1f}" '
                   f'stroke="{GOLD}" stroke-opacity="0.7" stroke-width="0.9" />')
    out.append('</g>')

    # Re-stroke the related stars in gold (they're highlighted on the linked side)
    for r in related:
        out.append(f'<circle cx="{r["x"]:.1f}" cy="{r["y"]:.1f}" r="{r["size"] + 1.5:.2f}" '
                   f'fill="none" stroke="{GOLD}" stroke-opacity="0.65" stroke-width="0.7" />')
    # Selection ring around the demo star
    out.append(f'<circle cx="{demo_star["x"]:.1f}" cy="{demo_star["y"]:.1f}" r="14" '
               f'fill="none" stroke="{GOLD}" stroke-opacity="0.95" stroke-width="1.6" />')
    out.append(f'<circle cx="{demo_star["x"]:.1f}" cy="{demo_star["y"]:.1f}" r="{demo_star["size"] + 0.5:.2f}" '
               f'fill="{GOLD}" fill-opacity="0.95" />')

    # Hint caption near the demo star (offset so it doesn't overlap)
    hint_x = demo_star["x"] + 28
    hint_y = demo_star["y"] - 18
    if demo_star["x"] > CX2:
        hint_x = demo_star["x"] - 28
    out.append(f'<line x1="{demo_star["x"] + 12:.1f}" y1="{demo_star["y"] - 8:.1f}" '
               f'x2="{hint_x - 4:.1f}" y2="{hint_y - 2:.1f}" '
               f'stroke="{INK}" stroke-opacity="0.5" stroke-width="0.5" />')
    text_anchor = "start" if demo_star["x"] <= CX2 else "end"
    out.append(f'<text x="{hint_x:.1f}" y="{hint_y:.1f}" font-size="11" fill="{INK}" '
               f'fill-opacity="0.8" font-style="italic" text-anchor="{text_anchor}" '
               f'font-family="serif">selected · {len(related)} links shown</text>')
    out.append(f'<text x="{hint_x:.1f}" y="{hint_y + 14:.1f}" font-size="9" fill="{INK}" '
               f'fill-opacity="0.55" text-anchor="{text_anchor}" '
               f'font-family="serif">edges appear on hover or click</text>')

    # Region rim: outer + inner circles, dividers, labels
    out.append(f'<circle cx="{CX2}" cy="{CY2}" r="{DOME_R2}" fill="none" '
               f'stroke="{INK}" stroke-opacity="0.4" stroke-width="0.7" />')
    out.append(f'<circle cx="{CX2}" cy="{CY2}" r="{RIM_INNER2}" fill="none" '
               f'stroke="{INK}" stroke-opacity="0.35" stroke-width="0.5" />')
    out.append(f'<circle cx="{CX2}" cy="{CY2}" r="{RIM_OUTER2}" fill="none" '
               f'stroke="{INK}" stroke-opacity="0.55" stroke-width="0.8" />')

    for name, t_start, t_end, _ in lib_arcs:
        # Wedge divider (radial, on rim only)
        x0, y0 = polar(RIM_INNER2 - 2, t_start, CX2, CY2)
        x1, y1 = polar(RIM_OUTER2 + 4, t_start, CX2, CY2)
        out.append(f'<line x1="{x0:.1f}" y1="{y0:.1f}" x2="{x1:.1f}" y2="{y1:.1f}" '
                   f'stroke="{INK}" stroke-opacity="0.55" stroke-width="0.7" />')

        # Tangent label (rotated to follow the rim arc)
        t_mid = (t_start + t_end) / 2
        label_r = (RIM_INNER2 + RIM_OUTER2) / 2 + 2
        x_text, y_text = polar(label_r, t_mid, CX2, CY2)
        rot_deg = math.degrees(t_mid)
        # Flip 180° on the left half so text reads right-side-up
        if math.pi / 2 < t_mid < 3 * math.pi / 2:
            rot_deg += 180
        out.append(f'<text x="{x_text:.1f}" y="{y_text:.1f}" font-size="15" '
                   f'fill="{BLUE_INK}" font-weight="600" letter-spacing="2.5" '
                   f'text-anchor="middle" dominant-baseline="middle" '
                   f'transform="rotate({rot_deg:.1f} {x_text:.1f} {y_text:.1f})" '
                   f'font-family="serif">{name.upper()}</text>')

        # Note count just outside the rim
        label_r2 = RIM_OUTER2 + 22
        x_text2, y_text2 = polar(label_r2, t_mid, CX2, CY2)
        n_notes = int(N_NOTES * (t_end - t_start) / (2 * math.pi))
        out.append(f'<text x="{x_text2:.1f}" y="{y_text2:.1f}" font-size="10" '
                   f'fill="{INK}" fill-opacity="0.6" letter-spacing="1" '
                   f'text-anchor="middle" dominant-baseline="middle" '
                   f'transform="rotate({rot_deg:.1f} {x_text2:.1f} {y_text2:.1f})" '
                   f'font-family="serif">{n_notes:,} notes</text>')

    # (Time-axis ladder removed — Time is now its own toggle mode T,
    #  no orthogonal calendar needed.)

    # Title (top-left)
    out.append(f'<text x="80" y="80" font-size="22" fill="{BLUE_INK}" '
               f'font-weight="600" font-family="serif">Constellation Sight</text>')
    out.append(f'<text x="80" y="105" font-size="11" fill="{BLUE_INK}" '
               f'fill-opacity="0.7" font-style="italic" font-family="serif">'
               f'@uconstellation.world · v3 mock-up · Option A + 5-mode toggle</text>')
    out.append(f'<text x="80" y="125" font-size="10" fill="{INK}" fill-opacity="0.55" '
               f'font-family="serif">{N_NOTES:,} notes · {N_EDGES:,} edges · {N_COMMUNITIES} communities</text>')

    # Active mode caption
    out.append(f'<text x="80" y="170" font-size="12" fill="{INK}" fill-opacity="0.65" '
               f'letter-spacing="2" font-family="serif">RIM AXIS</text>')
    out.append(f'<text x="80" y="200" font-size="22" fill="{GOLD}" font-weight="600" '
               f'letter-spacing="1.5" font-family="serif">REGIONS</text>')
    out.append(f'<text x="80" y="222" font-size="10" fill="{INK}" fill-opacity="0.6" '
               f'font-style="italic" font-family="serif">'
               f'A note\'s azimuth shows the library it lives in.</text>')

    # ── Universe Health: roundel top-center above dome, metrics row below
    #    the roundel touching the dome (Eisa 2026-05-07 directive).
    #    Dome top edge is at y = CY2 - DOME_R2 = 280.
    dome_top_y = CY2 - DOME_R2
    roundel_r = 50
    roundel_cy = 165
    # Caption above roundel
    out.append(f'<text x="{CX2:.1f}" y="{roundel_cy - roundel_r - 18}" font-size="11" '
               f'fill="{INK}" fill-opacity="0.6" letter-spacing="3" '
               f'text-anchor="middle" font-family="serif">UNIVERSE HEALTH</text>')
    # The roundel
    out.append(f'<circle cx="{CX2}" cy="{roundel_cy}" r="{roundel_r}" fill="{BG}" '
               f'stroke="{GOLD}" stroke-width="2.5" />')
    out.append(f'<text x="{CX2}" y="{roundel_cy - 4}" font-size="38" fill="{GOLD}" '
               f'text-anchor="middle" dominant-baseline="middle" font-weight="600" '
               f'font-family="serif">{HEALTH_SCORE}</text>')
    out.append(f'<text x="{CX2}" y="{roundel_cy + 22}" font-size="10" fill="{INK}" '
               f'fill-opacity="0.6" text-anchor="middle" font-family="serif">/ 100</text>')

    # Metrics flank the roundel — two on the left, two on the right,
    # all vertically centered on the roundel (Eisa 2026-05-07 directive).
    # Each metric is a small vertical stack: label / value / status pill.
    metrics_left = [
        ("MODULARITY", "0.63",  "caution", "#c9831f"),
        ("DOMINANCE",  "18%",   "healthy", "#3a8a4a"),
    ]
    metrics_right = [
        ("ENTROPY",     "3.62",  "healthy", "#3a8a4a"),
        ("CONNECTIVITY","28.43", "healthy", "#3a8a4a"),
    ]

    def _draw_metric(cx: float, cy: float, label: str, value: str, status: str, col: str) -> None:
        out.append(f'<text x="{cx:.1f}" y="{cy - 22:.1f}" font-size="10" '
                   f'fill="{INK}" fill-opacity="0.55" letter-spacing="2" '
                   f'text-anchor="middle" font-family="serif">{label}</text>')
        out.append(f'<text x="{cx:.1f}" y="{cy + 1:.1f}" font-size="22" '
                   f'fill="{INK}" fill-opacity="0.92" font-weight="500" '
                   f'text-anchor="middle" dominant-baseline="middle" font-family="serif">{value}</text>')
        pill_w = max(60, len(status) * 7 + 14)
        pill_y = cy + 17
        out.append(f'<rect x="{cx - pill_w / 2:.1f}" y="{pill_y:.1f}" '
                   f'width="{pill_w}" height="16" rx="8" '
                   f'fill="{col}" fill-opacity="0.12" '
                   f'stroke="{col}" stroke-opacity="0.65" stroke-width="0.7" />')
        out.append(f'<text x="{cx:.1f}" y="{pill_y + 11:.1f}" font-size="9" fill="{col}" '
                   f'font-weight="600" letter-spacing="1.6" '
                   f'text-anchor="middle" font-family="serif">{status.upper()}</text>')

    # Tightened cluster around the roundel (Eisa 2026-05-07 follow-up):
    # 160-px stride between blocks → metrics hug the roundel rather than
    # spreading to the canvas edges.
    left_xs = [480, 640]
    right_xs = [960, 1120]
    for (label, value, status, col), x in zip(metrics_left, left_xs):
        _draw_metric(x, roundel_cy, label, value, status, col)
    for (label, value, status, col), x in zip(metrics_right, right_xs):
        _draw_metric(x, roundel_cy, label, value, status, col)

    # ─── Toggle preview strip (bottom): 5 mini-charts as click targets ───
    out.append(f'<line x1="80" y1="1545" x2="{W2 - 80}" y2="1545" '
               f'stroke="{RULE_FAINT}" stroke-opacity="0.4" stroke-width="0.5" />')
    out.append(f'<text x="{W2 / 2}" y="1580" font-size="11" fill="{INK}" '
               f'fill-opacity="0.55" letter-spacing="3.5" '
               f'text-anchor="middle" font-family="serif">RIM AXIS · CLICK TO SWITCH MODE</text>')
    out.append(f'<text x="{W2 / 2}" y="1600" font-size="9" fill="{INK}" '
               f'fill-opacity="0.45" font-style="italic" letter-spacing="0.5" '
               f'text-anchor="middle" font-family="serif">'
               f'Stars migrate around the rim with a 600 ms ease — '
               f'their radius and color stay the same.</text>')

    modes = [
        ("R", "Regions",    5, True,  "active"),
        ("L", "Link Types", 7, True,  "ready"),
        ("T", "Time",       8, True,  "ready"),
        ("C", "Confidence", 4, False, "later"),
        ("S", "Stages",     6, False, "later"),
        ("A", "Acts",       5, False, "later"),
    ]
    strip_cy = 1750
    n_modes = len(modes)
    cell_w = (W2 - 120) / n_modes
    for i, (letter, name, n_wedges, is_ready, status) in enumerate(modes):
        cx = 60 + cell_w * (i + 0.5)
        is_active = (status == "active")
        thumb_r = 80 if is_active else 68
        thumb_rim = thumb_r + 10

        # Card background
        card_x = cx - cell_w / 2 + 14
        card_y = strip_cy - 130
        card_w = cell_w - 28
        card_h = 280
        border_color = GOLD if is_active else RULE_FAINT
        border_op = 0.85 if is_active else 0.35
        bw = 2.0 if is_active else 0.6
        out.append(f'<rect x="{card_x:.1f}" y="{card_y:.1f}" width="{card_w:.1f}" height="{card_h:.1f}" '
                   f'rx="10" fill="{BG}" stroke="{border_color}" stroke-opacity="{border_op}" stroke-width="{bw}" />')

        thumb_op = 1.0 if is_ready else 0.42

        # Inner faint rings
        for ring in [0.5, 1.0]:
            out.append(f'<circle cx="{cx:.1f}" cy="{strip_cy - 18}" r="{ring * thumb_r * 0.8:.1f}" '
                       f'fill="none" stroke="{RULE_FAINT}" stroke-opacity="{0.25 * thumb_op:.2f}" '
                       f'stroke-width="0.4" />')
        # Dome outline
        out.append(f'<circle cx="{cx:.1f}" cy="{strip_cy - 18}" r="{thumb_r}" fill="none" '
                   f'stroke="{INK}" stroke-opacity="{0.35 * thumb_op:.2f}" stroke-width="0.6" />')
        # Rim outline
        out.append(f'<circle cx="{cx:.1f}" cy="{strip_cy - 18}" r="{thumb_rim}" fill="none" '
                   f'stroke="{INK}" stroke-opacity="{0.55 * thumb_op:.2f}" stroke-width="0.7" />')

        # Wedge dividers per mode (n varies)
        arc = 2 * math.pi / n_wedges
        for w_i in range(n_wedges):
            t = w_i * arc
            x0, y0 = polar(thumb_r * 0.85, t, cx, strip_cy - 18)
            x1, y1 = polar(thumb_rim, t, cx, strip_cy - 18)
            out.append(f'<line x1="{x0:.1f}" y1="{y0:.1f}" x2="{x1:.1f}" y2="{y1:.1f}" '
                       f'stroke="{INK}" stroke-opacity="{0.45 * thumb_op:.2f}" stroke-width="0.5" />')

        # Sample stars (40-60 per thumbnail)
        thumb_rng = random.Random(100 + i)
        n_stars_thumb = 60 if is_ready else 40
        for s_i in range(n_stars_thumb):
            wedge_idx = s_i % n_wedges
            t_w = wedge_idx * arc + (0.1 + 0.8 * thumb_rng.random()) * arc
            u = thumb_rng.random()
            cent = u ** 2.5
            r_s = (1 - cent) ** 0.6 * (thumb_r - 8) + 4
            sx = cx + r_s * math.sin(t_w)
            sy = (strip_cy - 18) - r_s * math.cos(t_w)
            sz = 0.5 + cent * 1.5
            out.append(f'<circle cx="{sx:.1f}" cy="{sy:.1f}" r="{sz:.2f}" '
                       f'fill="{STAR_FILL}" fill-opacity="{0.7 * thumb_op:.2f}" />')

        # Status badge above the card
        badge_y = strip_cy - 145
        if is_active:
            out.append(f'<rect x="{cx - 38:.1f}" y="{badge_y}" width="76" height="20" '
                       f'rx="10" fill="{GOLD}" />')
            out.append(f'<text x="{cx:.1f}" y="{badge_y + 14}" font-size="10" '
                       f'fill="{BG}" font-weight="600" letter-spacing="2" '
                       f'text-anchor="middle" font-family="serif">ACTIVE</text>')
        elif is_ready:
            out.append(f'<rect x="{cx - 32:.1f}" y="{badge_y}" width="64" height="20" '
                       f'rx="10" fill="none" stroke="{INK}" stroke-opacity="0.7" stroke-width="0.8" />')
            out.append(f'<text x="{cx:.1f}" y="{badge_y + 14}" font-size="10" '
                       f'fill="{INK}" fill-opacity="0.85" font-weight="500" letter-spacing="2" '
                       f'text-anchor="middle" font-family="serif">READY</text>')
        else:
            out.append(f'<rect x="{cx - 56:.1f}" y="{badge_y}" width="112" height="20" '
                       f'rx="10" fill="none" stroke="{INK}" stroke-opacity="0.35" stroke-width="0.7" />')
            out.append(f'<text x="{cx:.1f}" y="{badge_y + 14}" font-size="9" '
                       f'fill="{INK}" fill-opacity="0.55" letter-spacing="1.8" '
                       f'text-anchor="middle" font-family="serif">AVAILABLE LATER</text>')

        # Mode letter (large) below dome
        letter_y = strip_cy + 100
        letter_color = GOLD if is_active else INK
        letter_op = 1.0 if is_ready else 0.45
        out.append(f'<text x="{cx:.1f}" y="{letter_y}" font-size="32" '
                   f'fill="{letter_color}" fill-opacity="{letter_op:.2f}" '
                   f'font-weight="600" text-anchor="middle" dominant-baseline="middle" '
                   f'font-family="serif">{letter}</text>')

        # Mode name (mid-size)
        out.append(f'<text x="{cx:.1f}" y="{letter_y + 24}" font-size="12" '
                   f'fill="{INK}" fill-opacity="{0.85 * thumb_op:.2f}" '
                   f'font-weight="500" letter-spacing="2" '
                   f'text-anchor="middle" font-family="serif">{name.upper()}</text>')

        # Wedge count caption
        out.append(f'<text x="{cx:.1f}" y="{letter_y + 42}" font-size="10" '
                   f'fill="{INK}" fill-opacity="{0.55 * thumb_op:.2f}" font-style="italic" '
                   f'text-anchor="middle" font-family="serif">{n_wedges} wedges</text>')

    out.append('</svg>')
    return "\n".join(out)


# ─── Main ────────────────────────────────────────────────────────────

def main() -> None:
    docs = Path(__file__).parent.parent / "docs"
    options = [
        ("A", build_option_a, "polar-grid"),
        ("B", build_option_b, "stratum-rings"),
        ("C", build_option_c, "hemispheric-pair"),
        ("D", build_option_d, "suwaidi-faithful"),
        ("A2", build_option_a_toggle, "toggle"),
    ]
    for letter, builder, slug in options:
        path = docs / f"Constellation-Sight-v3-mockup-{letter}-{slug}.svg"
        path.write_text(builder(), encoding="utf-8")
        print(f"Wrote {path.name}")


if __name__ == "__main__":
    main()
