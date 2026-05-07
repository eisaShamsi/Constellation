/**
 * Sight v3 — calendar rim geometry helpers (MIG-019 §2C).
 *
 * Pure TypeScript: no Pixi dependency. Returns geometric primitives
 * that `SightV3.svelte` renders via Pixi Graphics + Text.
 *
 * Calendar systems supported in §2C:
 *   - 'gregorian' (default; Intl.DateTimeFormat with no calendar override)
 *   - 'hijri' (Intl.DateTimeFormat with calendar: 'islamic-umalqura' which
 *     is the Saudi Umm al-Qura observed calendar — most accurate match for
 *     the Hijri dates Eisa's universe is likely to use; fallback to
 *     'islamic-civil' if the locale doesn't support Umm al-Qura).
 *   - 'solar-hijri' / 'hebrew' — placeholder; UI shows the system in the
 *     Settings checkbox but rim renders Gregorian month names until
 *     PJ-014 backfill (per Concept Paper §11 Q3 follow-up).
 *
 * Multiple enabled systems render as concentric rings — innermost is the
 * first in the user's enabled list.
 */

export type CalendarSystem = 'gregorian' | 'hijri' | 'solar-hijri' | 'hebrew';

export interface RimViewport {
    cx: number;
    cy: number;
    /** Inner edge of the rim (just outside the dome). */
    innerRadius: number;
    /** Outer edge of the rim. */
    outerRadius: number;
}

export interface MonthSegment {
    calendar: CalendarSystem;
    /** Ring index (0 = innermost). */
    ringIndex: number;
    /** 0-11. Gregorian: 0 = January. Hijri: 0 = Muharram. */
    monthIndex: number;
    /** Radians, 0 = top (12 o'clock), increasing clockwise (so the
     *  user reads months Jan → Dec around the dome clockwise from top). */
    startAngle: number;
    endAngle: number;
    midAngle: number;
    /** Label text in the user's locale. */
    label: string;
    /** Screen position for the label centroid (within the ring). */
    labelX: number;
    labelY: number;
    /** Inner/outer radius for this segment's arc (used by drawArc). */
    rIn: number;
    rOut: number;
}

const TWO_PI = Math.PI * 2;
const RING_THICKNESS = 22; // pixels per concentric ring

/**
 * Returns 12 month-name strings for the given calendar in the user's
 * UI locale. For Hijri, uses Umm al-Qura when supported; otherwise
 * falls back to Islamic civil. Returns Gregorian month names for
 * not-yet-localized systems with a placeholder prefix so the user
 * sees they're stand-ins.
 */
export function monthLabels(calendar: CalendarSystem, locale: string): string[] {
    if (calendar === 'gregorian') {
        return gregorianMonthLabels(locale);
    }
    if (calendar === 'hijri') {
        return hijriMonthLabels(locale);
    }
    // Solar-Hijri / Hebrew are placeholders for now — render Gregorian
    // month names with a marker so the user knows they're stand-ins
    // until PJ-014 ships full localization.
    return gregorianMonthLabels(locale).map((m) => `${m}*`);
}

function gregorianMonthLabels(locale: string): string[] {
    const fmt = new Intl.DateTimeFormat(locale, { month: 'long' });
    const out: string[] = [];
    for (let i = 0; i < 12; i++) {
        // Use a date guaranteed to be in month i (15th avoids edge cases).
        out.push(fmt.format(new Date(2024, i, 15)));
    }
    return out;
}

function hijriMonthLabels(locale: string): string[] {
    // Try Umm al-Qura first (Saudi observed calendar); fall back to Civil.
    const candidates = ['islamic-umalqura', 'islamic-civil'];
    for (const cal of candidates) {
        try {
            const fmt = new Intl.DateTimeFormat(`${locale}-u-ca-${cal}`, { month: 'long' });
            const out: string[] = [];
            // Walk through 12 successive Hijri months. We can't directly
            // construct a Hijri date in JS, but stepping by ~29.5 days
            // from a known anchor sweeps through them.
            const anchor = new Date(2024, 0, 15); // mid-January 2024
            const ms_per_day = 86_400_000;
            for (let i = 0; i < 12; i++) {
                const date = new Date(anchor.getTime() + i * 30 * ms_per_day);
                const formatted = fmt.format(date);
                if (!out.includes(formatted)) {
                    out.push(formatted);
                }
                if (out.length >= 12) break;
            }
            // If we got 12 distinct names, we're good. Otherwise fall through.
            if (out.length >= 12) {
                return out.slice(0, 12);
            }
        } catch {
            // try next candidate
        }
    }
    // Last resort — hardcoded Hijri month names in Arabic / English.
    return [
        'Muḥarram', 'Ṣafar', "Rabīʿ al-awwal", "Rabīʿ ath-thānī",
        "Jumādā al-ūlā", "Jumādā ath-thāniyah", 'Rajab', "Shaʿbān",
        'Ramaḍān', 'Shawwāl', "Dhū al-qaʿdah", 'Dhū al-ḥijjah',
    ];
}

/**
 * Compute month-arc segments for one or more concentric rings (one per
 * enabled calendar system). Returns a flat list across all rings.
 *
 * Months span 30° each. Index 0 starts at the top (12 o'clock) and
 * proceeds clockwise (matching how a calendar reads).
 */
export function monthArcSegments(
    viewport: RimViewport,
    enabledCalendars: CalendarSystem[],
    locale: string,
): MonthSegment[] {
    const segments: MonthSegment[] = [];
    const segArc = TWO_PI / 12;

    for (let r = 0; r < enabledCalendars.length; r++) {
        const calendar = enabledCalendars[r];
        const rIn = viewport.innerRadius + r * RING_THICKNESS;
        const rOut = rIn + RING_THICKNESS;
        const labelR = (rIn + rOut) / 2;
        const labels = monthLabels(calendar, locale);

        for (let m = 0; m < 12; m++) {
            // Convert "0 = top, clockwise" to standard math angle (0 = right, ccw):
            // standardAngle = -π/2 + clockwiseAngle
            const startClockwise = m * segArc;
            const endClockwise = (m + 1) * segArc;
            const midClockwise = startClockwise + segArc / 2;

            const startAngle = -Math.PI / 2 + startClockwise;
            const endAngle = -Math.PI / 2 + endClockwise;
            const midAngle = -Math.PI / 2 + midClockwise;

            segments.push({
                calendar,
                ringIndex: r,
                monthIndex: m,
                startAngle,
                endAngle,
                midAngle,
                label: labels[m] ?? `M${m}`,
                labelX: viewport.cx + labelR * Math.cos(midAngle),
                labelY: viewport.cy + labelR * Math.sin(midAngle),
                rIn,
                rOut,
            });
        }
    }
    return segments;
}

/**
 * Hit-test: given screen coords (px, py), find which month segment
 * (if any) the cursor is over.
 *
 * Returns null if the cursor is outside any rim ring.
 */
export function pickMonth(
    viewport: RimViewport,
    px: number,
    py: number,
    enabledCalendars: CalendarSystem[],
): { calendar: CalendarSystem; ringIndex: number; monthIndex: number } | null {
    const dx = px - viewport.cx;
    const dy = py - viewport.cy;
    const r = Math.hypot(dx, dy);

    // Determine which ring (if any) the cursor is within.
    const ringIndex = Math.floor((r - viewport.innerRadius) / RING_THICKNESS);
    if (ringIndex < 0 || ringIndex >= enabledCalendars.length) return null;

    // Convert math angle (0 = right, ccw) to clockwise-from-top.
    const mathAngle = Math.atan2(dy, dx); // -π..π, 0 = right
    let clockwise = mathAngle + Math.PI / 2; // 0 = top, ccw still
    // Normalize to [0, 2π) clockwise: invert direction
    if (clockwise < 0) clockwise += TWO_PI;
    // mathAngle increases counter-clockwise; we want clockwise, so:
    clockwise = TWO_PI - clockwise;
    if (clockwise >= TWO_PI) clockwise -= TWO_PI;

    const segArc = TWO_PI / 12;
    let monthIndex = Math.floor(clockwise / segArc);
    if (monthIndex < 0) monthIndex = 0;
    if (monthIndex > 11) monthIndex = 11;

    return {
        calendar: enabledCalendars[ringIndex],
        ringIndex,
        monthIndex,
    };
}

/** Get the Gregorian-equivalent (year, month) range for a calendar+month
 *  filter. For Gregorian: trivial. For Hijri: returns the wider range
 *  that any Gregorian date in the matching Hijri month could fall in.
 *  §2C ships Gregorian-only filtering; Hijri filter logic lands when
 *  PJ-014 backfills the Hijri date math. */
export function gregorianMonthFromSegment(
    seg: { calendar: CalendarSystem; monthIndex: number },
): number | null {
    if (seg.calendar === 'gregorian') return seg.monthIndex;
    // Hijri / Solar-Hijri / Hebrew month → Gregorian-equivalent filter
    // is non-trivial; defer to PJ-014. Until then, hovering a non-Gregorian
    // month does NOT trigger a star filter.
    return null;
}
