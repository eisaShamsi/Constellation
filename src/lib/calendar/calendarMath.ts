/**
 * MIG-081 — unified calendar math for the Calendar (4 systems).
 *
 * One job: given a "display" year+month in a chosen calendar SYSTEM, produce a 6×7
 * month grid whose cells are keyed back to **Gregorian ISO `YYYY-MM-DD`** (the only
 * thing the rest of Constellation stores — daily-note filenames, the note/task dot
 * lookups, openDailyNote). Display labels (month name, day number) are localised.
 *
 * Per-system grid math:
 *   - gregorian   → native Date (no dependency).
 *   - hijri       → Eisa's vendored `hijri.js` (astronomical engine; the "same one").
 *   - solar-hijri → Temporal calendar `persian`  (lazy polyfill).
 *   - hebrew      → Temporal calendar `hebrew`    (lazy polyfill).
 *
 * Engines (hijri.js, Temporal) are lazy-imported — they never touch the boot/editor
 * path (Perf Rules 3/6). Call `ensureCalendarEngines(systems)` before rendering.
 */

export type CalendarSystem = 'gregorian' | 'hijri' | 'solar-hijri' | 'hebrew';

/** Our system id → the Temporal/Intl calendar id (only for the Temporal-backed ones). */
const TEMPORAL_CAL: Partial<Record<CalendarSystem, string>> = {
	'solar-hijri': 'persian',
	hebrew: 'hebrew',
};

// ─── lazy engine loaders ───────────────────────────────────────────────
let _Temporal: any = null;
let _Hijri: any = null;

async function getTemporal(): Promise<any> {
	if (_Temporal) return _Temporal;
	const native = (globalThis as any)?.Temporal;
	if (native) { _Temporal = native; return _Temporal; }
	const mod = await import('@js-temporal/polyfill');
	_Temporal = (mod as any).Temporal;
	return _Temporal;
}

async function getHijri(): Promise<any> {
	if (_Hijri) return _Hijri;
	const mod = await import('./hijri');
	_Hijri = (mod as any).default;
	return _Hijri;
}

/** Preload the engine(s) needed. Hijri is ALWAYS loaded — the rich grid uses its
 *  getMoonPhase (astronomical → universal, shown for every system) + Hijri events. */
export async function ensureCalendarEngines(systems: CalendarSystem[]): Promise<void> {
	const tasks: Promise<any>[] = [getHijri()];
	if (systems.some((s) => s === 'solar-hijri' || s === 'hebrew')) tasks.push(getTemporal());
	await Promise.all(tasks);
}

// ─── helpers ───────────────────────────────────────────────────────────
const pad = (n: number) => String(n).padStart(2, '0');
const isoOf = (y: number, m: number, d: number) => `${y}-${pad(m)}-${pad(d)}`;
const todayISO = () => {
	const n = new Date();
	return isoOf(n.getFullYear(), n.getMonth() + 1, n.getDate());
};

/** JS day-of-week for a Gregorian ISO date, 0=Sun..6=Sat. */
function dowOfISO(iso: string): number {
	const [y, m, d] = iso.split('-').map(Number);
	return new Date(y, m - 1, d).getDay();
}

/** Localised number (honours the locale's numbering system, e.g. Arabic-Indic). */
export function localeNum(n: number, locale: string): string {
	try { return n.toLocaleString(locale); } catch { return String(n); }
}

export interface GridCell {
	/** Gregorian ISO YYYY-MM-DD — the storage key for this cell. */
	iso: string;
	/** Day number in the DISPLAY calendar, localised. */
	dayLabel: string;
	/** Raw day number in the display calendar (for logic). */
	dayNum: number;
	inCurrentMonth: boolean;
	isToday: boolean;
}

export interface MonthGrid {
	cells: GridCell[];        // 42 cells (6×7)
	monthLabel: string;       // localised "Month Year" in the display calendar
	displayYear: number;
	displayMonth: number;     // 1-based in the display calendar
	weekdayLabels: string[];  // 7 localised narrow weekday names, week-start applied
}

/** {year,month,day} of today in the given system (1-based month). */
export async function todayInSystem(system: CalendarSystem): Promise<{ year: number; month: number; day: number }> {
	const n = new Date();
	if (system === 'gregorian') return { year: n.getFullYear(), month: n.getMonth() + 1, day: n.getDate() };
	if (system === 'hijri') {
		const H = await getHijri();
		const h = H.gregorianToHijri(n.getFullYear(), n.getMonth() + 1, n.getDate());
		return { year: h.year, month: h.month, day: h.day };
	}
	const T = await getTemporal();
	const p = T.PlainDate.from(todayISO()).withCalendar(TEMPORAL_CAL[system]!);
	return { year: p.year, month: p.month, day: p.day };
}

/** Localised narrow weekday names (Sun..Sat order), then rotated by weekStart (0=Sun,1=Mon). */
function weekdayNames(locale: string, weekStart: number): string[] {
	const fmt = new Intl.DateTimeFormat(locale, { weekday: 'narrow' });
	// 2024-07-07 is a Sunday.
	const base: string[] = [];
	for (let i = 0; i < 7; i++) base.push(fmt.format(new Date(2024, 6, 7 + i)));
	return base.slice(weekStart).concat(base.slice(0, weekStart));
}

/**
 * Build the 6×7 grid for (displayYear, displayMonth) in `system`.
 * weekStart: 0=Sunday, 1=Monday. locale drives labels + numerals.
 * Engines must already be loaded (call ensureCalendarEngines first).
 */
export function buildMonthGrid(
	system: CalendarSystem,
	displayYear: number,
	displayMonth: number,
	locale: string,
	weekStart = 0,
): MonthGrid {
	const tISO = todayISO();
	const weekdayLabels = weekdayNames(locale, weekStart);

	// Resolve, per system: the ISO date of day-1 of (displayYear,displayMonth),
	// the number of days in that month, the month label, and a (day → ISO) mapper.
	let firstISO: string;
	let daysInThisMonth: number;
	let monthLabel: string;
	let isoForDay: (dayNum: number) => string;

	if (system === 'gregorian') {
		firstISO = isoOf(displayYear, displayMonth, 1);
		daysInThisMonth = new Date(displayYear, displayMonth, 0).getDate();
		monthLabel = new Intl.DateTimeFormat(locale, { month: 'long', year: 'numeric' })
			.format(new Date(displayYear, displayMonth - 1, 1));
		isoForDay = (d) => isoOf(displayYear, displayMonth, d);
	} else if (system === 'hijri') {
		const H = _Hijri;
		daysInThisMonth = H.daysInMonth(displayYear, displayMonth);
		const g1 = H.hijriToGregorian(displayYear, displayMonth, 1);
		firstISO = isoOf(g1.year, g1.month, g1.day);
		const names = (locale.startsWith('ar') ? H.MONTH_NAMES : H.MONTH_NAMES_EN) as string[];
		monthLabel = `${names?.[displayMonth - 1] ?? `M${displayMonth}`} ${localeNum(displayYear, locale)}`;
		isoForDay = (d) => { const g = H.hijriToGregorian(displayYear, displayMonth, d); return isoOf(g.year, g.month, g.day); };
	} else {
		const T = _Temporal;
		const cal = TEMPORAL_CAL[system]!;
		const first = T.PlainDate.from({ year: displayYear, month: displayMonth, day: 1, calendar: cal });
		daysInThisMonth = first.daysInMonth;
		const g1 = first.withCalendar('iso8601');
		firstISO = isoOf(g1.year, g1.month, g1.day);
		monthLabel = new Intl.DateTimeFormat(locale, { calendar: cal, month: 'long', year: 'numeric' })
			.format(new Date(firstISO));
		isoForDay = (d) => { const g = first.with({ day: d }).withCalendar('iso8601'); return isoOf(g.year, g.month, g.day); };
	}

	// Leading blanks: how many cells before day-1, given the week-start.
	const firstDow = dowOfISO(firstISO);            // 0=Sun..6=Sat
	const lead = (firstDow - weekStart + 7) % 7;

	// Current-month cells first (we know their display day numbers directly).
	const monthCells: GridCell[] = [];
	for (let d = 1; d <= daysInThisMonth; d++) {
		const iso = isoForDay(d);
		monthCells.push({ iso, dayNum: d, dayLabel: localeNum(d, locale), inCurrentMonth: true, isToday: iso === tISO });
	}
	// Leading: walk back from firstISO by `lead` Gregorian days.
	const lead_cells: GridCell[] = [];
	for (let i = lead; i >= 1; i--) {
		const iso = isoShift(firstISO, -i);
		lead_cells.push({ iso, dayNum: 0, dayLabel: displayDayNum(system, iso, locale), inCurrentMonth: false, isToday: iso === tISO });
	}
	// Trailing: fill to 42.
	const lastISO = monthCells[monthCells.length - 1].iso;
	const trail: GridCell[] = [];
	const need = 42 - lead_cells.length - monthCells.length;
	for (let i = 1; i <= need; i++) {
		const iso = isoShift(lastISO, i);
		trail.push({ iso, dayNum: 0, dayLabel: displayDayNum(system, iso, locale), inCurrentMonth: false, isToday: iso === tISO });
	}

	return {
		cells: [...lead_cells, ...monthCells, ...trail],
		monthLabel,
		displayYear,
		displayMonth,
		weekdayLabels,
	};
}

/** Shift a Gregorian ISO date by `delta` days. */
function isoShift(iso: string, delta: number): string {
	const [y, m, d] = iso.split('-').map(Number);
	const dt = new Date(y, m - 1, d + delta);
	return isoOf(dt.getFullYear(), dt.getMonth() + 1, dt.getDate());
}

/** The display-calendar day-of-month number for a Gregorian ISO date (for greyed neighbour cells). */
function displayDayNum(system: CalendarSystem, iso: string, locale: string): string {
	const [y, m, d] = iso.split('-').map(Number);
	if (system === 'gregorian') return localeNum(d, locale);
	if (system === 'hijri') { const h = _Hijri.gregorianToHijri(y, m, d); return localeNum(h.day, locale); }
	const p = _Temporal.PlainDate.from(iso).withCalendar(TEMPORAL_CAL[system]!);
	return localeNum(p.day, locale);
}

/** Step one month forward/back in the display calendar. Returns {year,month}. */
export function stepMonth(system: CalendarSystem, year: number, month: number, dir: 1 | -1): { year: number; month: number } {
	if (system === 'gregorian' || system === 'hijri') {
		// 12-month years for both Gregorian and Hijri.
		let m = month + dir, y = year;
		if (m > 12) { m = 1; y++; } else if (m < 1) { m = 12; y--; }
		return { year: y, month: m };
	}
	// Persian/Hebrew: month count varies (Hebrew leap years have 13) — use Temporal.
	const T = _Temporal;
	const cur = T.PlainYearMonth.from({ year, month, calendar: TEMPORAL_CAL[system]! });
	const next = cur.add({ months: dir });
	return { year: next.year, month: next.month };
}

/**
 * Cultural-date string for a Gregorian ISO date, for the secondary-date label AND the
 * non-authoritative frontmatter field. Returns e.g. "1447-12-03" (numeric) — the field
 * value — or a localised long form for display.
 */
export async function culturalDateParts(
	system: CalendarSystem,
	iso: string,
): Promise<{ year: number; month: number; day: number }> {
	const [y, m, d] = iso.split('-').map(Number);
	if (system === 'gregorian') return { year: y, month: m, day: d };
	if (system === 'hijri') { const H = await getHijri(); const h = H.gregorianToHijri(y, m, d); return { year: h.year, month: h.month, day: h.day }; }
	const T = await getTemporal();
	const p = T.PlainDate.from(iso).withCalendar(TEMPORAL_CAL[system]!);
	return { year: p.year, month: p.month, day: p.day };
}

// ─── MIG-081 §C.2f — Hijri engine prefs (corrections + calculation mode) ──────
// Source of truth is appSettings (synced with the universe), NOT the engine's own
// per-device localStorage. We PUSH appSettings into the singleton engine on load /
// on change, overriding whatever it read from localStorage. Both corrections and
// mode flow through the engine's unified gregorianToHijri/hijriToGregorian/daysInMonth
// (verified: _engine() switches mode; hijriToJDN applies _getCumulativeCorrection),
// which are exactly the functions buildMonthGrid uses → the grid reflects them.

export type CalculationMode = 'astronomical' | 'tabular';

/** Push the universe's calendar prefs into the (loaded) Hijri engine. Always sets mode
 *  explicitly (so a stale localStorage value can't linger) and replaces the full
 *  corrections set. Idempotent; safe to call on every render-prefs change.
 *  NOTE: appSettings is the single synced source of truth. The engine's own
 *  setCorrection/clearCorrections still mirror into localStorage['hijri-corrections']
 *  as a side effect — that mirror is DISCARDED (always overwritten from appSettings on
 *  every load), never read as authoritative once this runs. Don't mistake it for a 2nd source. */
export async function applyCalendarPrefs(
	corrections: Record<string, number>,
	mode: CalculationMode,
): Promise<void> {
	const H = await getHijri();
	H.clearCorrections();
	for (const [key, off] of Object.entries(corrections ?? {})) {
		const [y, m] = key.split('-').map(Number);
		if (Number.isFinite(y) && Number.isFinite(m) && off) H.setCorrection(y, m, off);
	}
	H.setMode(mode);
}

/** The 12 Hijri month names for the locale (Muharram … Dhul-Hijjah). Engine must be
 *  loaded (call ensureCalendarEngines first). Used by the Calendar Settings month picker. */
export function hijriMonthNames(locale: string): string[] {
	const H = _Hijri;
	if (!H) return Array.from({ length: 12 }, (_, i) => `M${i + 1}`);
	const arr = (locale.startsWith('ar') ? H.MONTH_NAMES : H.MONTH_NAMES_EN) as string[] | undefined;
	return arr && arr.length === 12 ? arr.slice() : Array.from({ length: 12 }, (_, i) => `M${i + 1}`);
}

/** The frontmatter field name for a system (gregorian writes none). */
export function frontmatterKey(system: CalendarSystem): string | null {
	switch (system) {
		case 'hijri': return 'hijri';
		case 'solar-hijri': return 'jalali';
		case 'hebrew': return 'hebrew';
		default: return null;
	}
}

// ═══════════════════════════════════════════════════════════════════════
// MIG-081 §C.2 — RICH grid (ports Eisa's app: dual dates + moon phase + events
// + week number + AH/sacred). Additive over buildMonthGrid so the plain §C path
// is untouched. Engines must be loaded (ensureCalendarEngines — always loads Hijri).
// ═══════════════════════════════════════════════════════════════════════

export interface RichCell extends GridCell {
	subLabel: string;       // cross-reference date: Gregorian day (non-Greg primary) | Hijri day (Greg primary)
	weekNumber: number;     // ISO-8601 week of this cell's Gregorian date
	moonSymbol: string;     // ●◗◑◕○◔◐◖ from the engine's astronomical phase (universal)
	moonName: string;
	eventType?: 'holiday' | 'observance' | 'special'; // Hijri only
	eventName?: string;     // Hijri only
}

export interface RichMonthGrid {
	cells: RichCell[];
	monthLabel: string;     // primary-calendar month + year
	suffix: string;         // 'AH' | 'SH' | 'AM' | '' (era marker)
	subtitleRange: string;  // the CROSS-REFERENCE range in the OTHER calendar: Gregorian range when
	                        // the primary is non-Gregorian, Hijri range when the primary IS Gregorian.
	isSacred: boolean;      // Hijri sacred month → gold pill
	displayYear: number;
	displayMonth: number;
	weekdayLabels: string[];
}

/** ISO-8601 week number for a Gregorian ISO date. */
function isoWeek(iso: string): number {
	const [y, m, d] = iso.split('-').map(Number);
	const dt = new Date(Date.UTC(y, m - 1, d));
	const day = (dt.getUTCDay() + 6) % 7;            // Mon=0..Sun=6
	dt.setUTCDate(dt.getUTCDate() - day + 3);         // Thursday of this ISO week
	const firstThu = new Date(Date.UTC(dt.getUTCFullYear(), 0, 4));
	const ftDay = (firstThu.getUTCDay() + 6) % 7;
	firstThu.setUTCDate(firstThu.getUTCDate() - ftDay + 3);
	return 1 + Math.round((dt.getTime() - firstThu.getTime()) / (7 * 86400000));
}

/** "June – July 2026" (or "June 2026") from the grid's first/last Gregorian ISO cells. */
function gregRange(firstISO: string, lastISO: string, locale: string): string {
	const f = new Date(firstISO), l = new Date(lastISO);
	const mf = new Intl.DateTimeFormat(locale, { month: 'long' });
	const my = new Intl.DateTimeFormat(locale, { month: 'long', year: 'numeric' });
	if (f.getFullYear() === l.getFullYear() && f.getMonth() === l.getMonth()) return my.format(l);
	if (f.getFullYear() === l.getFullYear()) return `${mf.format(f)} – ${my.format(l)}`;
	return `${my.format(f)} – ${my.format(l)}`;
}

/** "Dhū al-Ḥijjah 1447 – Muḥarram 1448 AH" — the Hijri-month range spanning the grid's
 *  first/last Gregorian ISO cells (correction/mode-aware via the engine). Used as the
 *  cross-reference subtitle when the PRIMARY system is Gregorian (the mirror of gregRange). */
function hijriRange(firstISO: string, lastISO: string, locale: string): string {
	const H = _Hijri;
	if (!H) return '';
	const isAr = locale.startsWith('ar');
	const names = (isAr ? H.MONTH_NAMES : H.MONTH_NAMES_EN) as string[] | undefined;
	const suffix = isAr ? 'هـ' : 'AH';
	const [fy, fm, fd] = firstISO.split('-').map(Number);
	const [ly, lm, ld] = lastISO.split('-').map(Number);
	const a = H.gregorianToHijri(fy, fm, fd);
	const b = H.gregorianToHijri(ly, lm, ld);
	const nm = (h: { year: number; month: number }) => `${names?.[h.month - 1] ?? `M${h.month}`} ${localeNum(h.year, locale)}`;
	if (a.year === b.year && a.month === b.month) return `${nm(a)} ${suffix}`;
	if (a.year === b.year) return `${names?.[a.month - 1] ?? `M${a.month}`} – ${nm(b)} ${suffix}`;
	return `${nm(a)} – ${nm(b)} ${suffix}`;
}

/** Build the RICH grid (dual dates, moon phase, week numbers, Hijri events/sacred). */
export function buildRichMonthGrid(
	system: CalendarSystem,
	displayYear: number,
	displayMonth: number,
	locale: string,
	weekStart: 0 | 1 = 0,
): RichMonthGrid {
	const base = buildMonthGrid(system, displayYear, displayMonth, locale, weekStart);
	const H = _Hijri;
	const isAr = locale.startsWith('ar');

	const cells: RichCell[] = base.cells.map((c) => {
		const [gy, gm, gd] = c.iso.split('-').map(Number);
		const moon = H ? H.getMoonPhase(gy, gm, gd) : null;
		// Cross-reference sub-date: show Gregorian under a non-Greg primary; under a
		// Gregorian primary, show the Hijri day (so both systems are always visible).
		let subLabel: string;
		if (system === 'gregorian') {
			const h = H ? H.gregorianToHijri(gy, gm, gd) : null;
			subLabel = h ? localeNum(h.day, locale) : '';
		} else {
			subLabel = localeNum(gd, locale);
		}
		let eventType: RichCell['eventType'];
		let eventName: string | undefined;
		if (system === 'hijri' && H && c.inCurrentMonth) {
			const h = H.gregorianToHijri(gy, gm, gd);
			const ev = H.getEvent(h.month, h.day);
			if (ev) { eventType = ev.type; eventName = ev.name; }
		}
		return {
			...c,
			subLabel,
			weekNumber: isoWeek(c.iso),
			moonSymbol: moon?.symbol ?? '',
			moonName: moon?.name ?? '',
			eventType,
			eventName,
		};
	});

	let suffix = '';
	if (system === 'hijri') suffix = isAr ? 'هـ' : 'AH';
	else if (system === 'solar-hijri') suffix = locale.startsWith('fa') ? 'ه‍.ش' : 'SH';
	else if (system === 'hebrew') suffix = 'AM';
	const isSacred = system === 'hijri' && !!H && H.isSacredMonth(displayMonth);

	const firstISO = base.cells[0]?.iso ?? '';
	const lastISO = base.cells[base.cells.length - 1]?.iso ?? '';
	// The subtitle shows the OTHER calendar (cross-reference): Gregorian range for a non-Gregorian
	// primary, Hijri range for a Gregorian primary — so the pill (primary) is never just repeated.
	const subtitleRange = firstISO && lastISO
		? (system === 'gregorian' ? hijriRange(firstISO, lastISO, locale) : gregRange(firstISO, lastISO, locale))
		: '';

	return {
		cells,
		monthLabel: base.monthLabel,
		suffix,
		subtitleRange,
		isSacred,
		displayYear: base.displayYear,
		displayMonth: base.displayMonth,
		weekdayLabels: base.weekdayLabels,
	};
}
