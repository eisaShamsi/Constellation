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

/** Preload only the engine(s) the chosen systems need. Resolves when ready. */
export async function ensureCalendarEngines(systems: CalendarSystem[]): Promise<void> {
	const tasks: Promise<any>[] = [];
	if (systems.includes('hijri')) tasks.push(getHijri());
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

/** The frontmatter field name for a system (gregorian writes none). */
export function frontmatterKey(system: CalendarSystem): string | null {
	switch (system) {
		case 'hijri': return 'hijri';
		case 'solar-hijri': return 'jalali';
		case 'hebrew': return 'hebrew';
		default: return null;
	}
}
