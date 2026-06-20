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

export type CalendarSystem = 'gregorian' | 'hijri' | 'solar-hijri' | 'hebrew' | 'indian' | 'buddhist' | 'chinese' | 'korean';

/** Our system id → the Temporal/Intl calendar id (only for the Temporal-backed ones).
 *  §A.4 — indian (Saka) + buddhist are solar (fixed 12-month structure), so the generic
 *  Temporal branch handles them with no per-system grid code. (Chinese/Korean are §B — the
 *  polyfill throws on their leap months, so they get a separate Intl-only branch.) */
const TEMPORAL_CAL: Partial<Record<CalendarSystem, string>> = {
	'solar-hijri': 'persian',
	hebrew: 'hebrew',
	indian: 'indian',
	buddhist: 'buddhist',
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
	if (systems.some((s) => !!TEMPORAL_CAL[s])) tasks.push(getTemporal()); // any Temporal-backed system
	await Promise.all(tasks);
	// chinese/korean need NO engine — they're driven entirely by the host Intl (the Temporal polyfill
	// THROWS on their leap-month codes; ICU's Intl handles them). §B.
}

// ─── §B — LUNISOLAR (chinese, korean/dangi) via host Intl ONLY ──────────────
// The Temporal polyfill throws on chinese/dangi leap months, but ICU's Intl renders them perfectly
// (闰二月 / 윤2월). We drive the grid by ISO day-walking + Intl.formatToParts. The display "month" is
// an ORDINAL (1..12 or 1..13 in a leap year), so a leap month is its own page; navigation never skips
// or duplicates it. Node-verified against the 2023 leap-2 year. All Intl-call walks are memoised per
// (calendar, relatedYear) and run only on month-nav (off the keystroke hot path — Perf Rule 3).
const LUNISOLAR_CAL: Partial<Record<CalendarSystem, string>> = { chinese: 'chinese', korean: 'dangi' };
// Month names are rendered in each calendar's OWN script regardless of UI language — a Chinese month
// IS 五月 (not the English "Fifth Month", and not the Arabic placeholder "M04" that ICU emits when a
// UI locale lacks the names). This is also what visibly distinguishes Chinese (五月) from Korean (5월),
// since the two share IDENTICAL lunar dates (Korea uses the Chinese calendar). Boss-directed 2026-06-20.
const LUNISOLAR_NAME_LOCALE: Partial<Record<CalendarSystem, string>> = { chinese: 'zh', korean: 'ko' };

// §B.2 — PHONETIC month names (Boss-directed 2026-06-20): the native month's PRONUNCIATION written in
// the UI's script — the Hijri "Muharram" pattern generalised. The OS cannot transliterate (verified:
// zh-Latn still yields 五月; no Intl transliterator), so these are AUTHORED tables. Latin = standard
// romanization (Chinese Pinyin / Korean Revised Romanization — 6월→yuwol, 10월→siwol verified against
// sources). Arabic ('arab') is PENDING Boss verification → omitted for now, so it falls back to Latin
// until the verified table is added. Months are numbered (12 each) + a leap prefix, so the tables are tiny.
type PhoneticScript = 'latn' | 'arab';
const LUNAR_PHONETIC: Record<'chinese' | 'korean', Partial<Record<PhoneticScript, string[]>>> = {
	chinese: {
		latn: ['Yīyuè', 'Èryuè', 'Sānyuè', 'Sìyuè', 'Wǔyuè', 'Liùyuè', 'Qīyuè', 'Bāyuè', 'Jiǔyuè', 'Shíyuè', 'Shíyīyuè', 'Shí\'èryuè'],
		// Boss-verified 2026-06-20 (months 2,3,6,7,8,9 = accepted drafts; Boss corrected 1,4,5,10,11,12).
		arab: ['إي-يوي', 'أر-يوي', 'سان-يوي', 'سُه-يوي', 'وُو-يوي', 'ليو-يوي', 'تشي-يوي', 'با-يوي', 'جيو-يوي', 'شِر-يوي', 'شِر-إي-يوي', 'شِر-أر-يوي'],
	},
	korean: {
		latn: ['Irwol', 'Iwol', 'Samwol', 'Sawol', 'Owol', 'Yuwol', 'Chirwol', 'Parwol', 'Guwol', 'Siwol', 'Sibirwol', 'Sibiwol'],
		arab: ['إر-وُل', 'آي-وُل', 'سام-وُل', 'سا-وُل', 'أوه-وُل', 'يو-وُل', 'تشير-وُل', 'بار-وُل', 'گو-وُل', 'سي-وُل', 'سي-بِر-وُل', 'سي-بي-وُل'], // Boss-verified 2026-06-20 (#4 damma fix)
	},
};
const LUNAR_LEAP_PREFIX: Record<'chinese' | 'korean', Partial<Record<PhoneticScript, string>>> = {
	chinese: { latn: 'Rùn ', arab: 'رون ' },   // 闰 — leap-month marker (arab: my draft, pending Boss ok)
	korean:  { latn: 'Yun ', arab: 'يون ' },   // 윤 (arab: my draft, pending Boss ok)
};
let _monthNameStyle: 'native' | 'phonetic' = 'native';
export function setMonthNameStyle(s: string | undefined): void { _monthNameStyle = s === 'phonetic' ? 'phonetic' : 'native'; }
/** A lunisolar month's display name: native script (五월/5월) OR — if the user picked 'phonetic' —
 *  the romanized/transliterated form in the UI's script (Wǔyuè / Owol; Arabic falls back to Latin). */
function lunarMonthName(system: CalendarSystem, iso: string, locale: string): string {
	const cal = LUNISOLAR_CAL[system]!;
	if (_monthNameStyle === 'phonetic' && (system === 'chinese' || system === 'korean')) {
		const nv = lunarNav(iso, cal);
		const script: PhoneticScript = locale.startsWith('ar') ? 'arab' : 'latn';
		const tbl = LUNAR_PHONETIC[system];
		const base = tbl[script]?.[nv.monthNum - 1] ?? tbl.latn?.[nv.monthNum - 1];
		if (base) return (nv.isLeap ? (LUNAR_LEAP_PREFIX[system][script] ?? LUNAR_LEAP_PREFIX[system].latn ?? '') : '') + base;
	}
	return new Intl.DateTimeFormat(LUNISOLAR_NAME_LOCALE[system], { timeZone: 'UTC', calendar: cal, month: 'long' }).format(new Date(iso));
}
const DANGI_OFFSET = 2333; // Korean Dangi (단기/檀紀) era = Gregorian-aligned year + 2333.
// The user's per-lunisolar-calendar YEAR-display preference (Boss-directed 2026-06-20 — "give users the
// option in Calendar Settings"). Set by CalendarPanel from appSettings via setLunarYearStyles (same
// module-prefs pattern as applyCalendarPrefs). Chinese: sexagenary-gregorian|sexagenary|gregorian.
// Korean: dangi|dangi-gregorian|gregorian|sexagenary.
export type LunarYearStyles = { chinese: string; korean: string };
const LUNAR_YEAR_DEFAULT: LunarYearStyles = { chinese: 'sexagenary-gregorian', korean: 'dangi' };
let _lunarYearStyles: LunarYearStyles = { ...LUNAR_YEAR_DEFAULT };
export function setLunarYearStyles(s: Partial<LunarYearStyles> | undefined): void {
	_lunarYearStyles = { chinese: s?.chinese || LUNAR_YEAR_DEFAULT.chinese, korean: s?.korean || LUNAR_YEAR_DEFAULT.korean };
}
/** Sexagenary cycle name (干支) in the calendar's own script — "丙午" (zh/chinese) / "병오" (ko/dangi). */
function sexagenaryName(iso: string, cal: string, nameLocale: string): string {
	// 'yearName' is a valid Intl part for chinese/dangi but absent from the TS lib's part-type registry.
	for (const p of new Intl.DateTimeFormat(nameLocale, { timeZone: 'UTC', calendar: cal, year: 'numeric' }).formatToParts(new Date(iso))) if ((p.type as string) === 'yearName') return p.value;
	return '';
}
/** The year/era label for a lunisolar header, honouring the user's per-calendar preference.
 *  The era words (단기 / 年 / 년) stay in their own script — they ARE the calendar's identity. */
function lunarYearLabel(system: CalendarSystem, iso: string, relatedYear: number, locale: string): string {
	const style = (system === 'korean' ? _lunarYearStyles.korean : _lunarYearStyles.chinese) || LUNAR_YEAR_DEFAULT[system as 'chinese' | 'korean'];
	const greg = localeNum(relatedYear, locale);
	const sexa = () => { const s = sexagenaryName(iso, LUNISOLAR_CAL[system]!, LUNISOLAR_NAME_LOCALE[system]!); return s ? s + (system === 'korean' ? '년' : '年') : greg; };
	const dangi = `단기 ${localeNum(relatedYear + DANGI_OFFSET, locale)}`;
	switch (style) {
		case 'gregorian': return greg;
		case 'sexagenary': return sexa();
		case 'dangi': return dangi;
		case 'dangi-gregorian': return `${dangi} (${greg})`;
		default: return `${sexa()} ${greg}`; // 'sexagenary-gregorian' (chinese default)
	}
}

/** en-locale numeric parts → stable tokens for navigation ("2"/"2bis", numeric relatedYear/day). */
function lunarNav(iso: string, cal: string): { relatedYear: number; monthNum: number; isLeap: boolean; day: number } {
	const o: Record<string, string> = {};
	// timeZone:'UTC' — `new Date(isoDateOnly)` is UTC midnight; without this the formatter would use the
	// system TZ and report the PREVIOUS day west of UTC, throwing off the day-1 detection the walk relies on.
	for (const p of new Intl.DateTimeFormat('en-u-ca-' + cal, { timeZone: 'UTC', year: 'numeric', month: 'numeric', day: 'numeric' }).formatToParts(new Date(iso))) o[p.type] = p.value;
	const m = o.month ?? '1';
	return { relatedYear: parseInt(o.relatedYear ?? '0', 10), monthNum: parseInt(m, 10), isLeap: m.includes('bis'), day: parseInt(o.day ?? '1', 10) };
}
const _lunarYearCache = new Map<string, string[]>(); // key `${cal}:${relatedYear}` → [day-1 ISO of each ordinal]
/** The day-1 ISO of every month in `relatedYear` (12 or 13 entries), memoised. */
function lunarMonths(relatedYear: number, cal: string): string[] {
	const key = `${cal}:${relatedYear}`;
	const hit = _lunarYearCache.get(key);
	if (hit) return hit;
	// New year = the first Gregorian date whose relatedYear === target (relatedYear flips at month-1 day-1).
	let iso = `${relatedYear}-01-01`;
	for (let i = 0; i < 75 && lunarNav(iso, cal).relatedYear !== relatedYear; i++) iso = isoShift(iso, 1);
	const months: string[] = [];
	while (lunarNav(iso, cal).relatedYear === relatedYear) {
		months.push(iso);
		// Next month's day-1: a lunar month is 29–30 days, so jump 28 then walk to the day-1 reset.
		let probe = isoShift(iso, 28);
		while (lunarNav(probe, cal).day !== 1) probe = isoShift(probe, 1);
		iso = probe;
		if (months.length > 14) break; // safety
	}
	_lunarYearCache.set(key, months);
	return months;
}
/** {relatedYear, ordinal(1-based)} of the lunar month containing `iso`. */
function lunarOrdinalOf(iso: string, cal: string): { relatedYear: number; ordinal: number } {
	const relatedYear = lunarNav(iso, cal).relatedYear;
	const ms = lunarMonths(relatedYear, cal);
	let ordinal = ms.length;
	for (let i = 0; i < ms.length; i++) { if (iso < (ms[i + 1] ?? '9999-99-99')) { ordinal = i + 1; break; } }
	return { relatedYear, ordinal };
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

/** Localised number (honours the locale's numbering system, e.g. Arabic-Indic).
 *  useGrouping:false — calendar numbers (year, day, week) must NOT get a thousands
 *  separator (a Hijri year is "1448", never "1,448"). Days/weeks are <1000 anyway. */
export function localeNum(n: number, locale: string): string {
	try { return n.toLocaleString(locale, { useGrouping: false }); } catch { return String(n); }
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
	if (system === 'chinese' || system === 'korean') {
		const cal = LUNISOLAR_CAL[system]!;
		const { relatedYear, ordinal } = lunarOrdinalOf(todayISO(), cal); // year=relatedYear, month=ordinal
		return { year: relatedYear, month: ordinal, day: lunarNav(todayISO(), cal).day };
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
		// §A.4b — month number beside the name (Eisa: include Gregorian too, for consistency).
		const monthName = new Intl.DateTimeFormat(locale, { month: 'long' }).format(new Date(displayYear, displayMonth - 1, 1));
		monthLabel = `${monthName} (${localeNum(displayMonth, locale)}) ${localeNum(displayYear, locale)}`;
		isoForDay = (d) => isoOf(displayYear, displayMonth, d);
	} else if (system === 'hijri') {
		const H = _Hijri;
		daysInThisMonth = H.daysInMonth(displayYear, displayMonth);
		const g1 = H.hijriToGregorian(displayYear, displayMonth, 1);
		firstISO = isoOf(g1.year, g1.month, g1.day);
		const names = (locale.startsWith('ar') ? H.MONTH_NAMES : H.MONTH_NAMES_EN) as string[];
		// §A.4b — month number beside the name (helps with unfamiliar cultural months).
		monthLabel = `${names?.[displayMonth - 1] ?? `M${displayMonth}`} (${localeNum(displayMonth, locale)}) ${localeNum(displayYear, locale)}`;
		isoForDay = (d) => { const g = H.hijriToGregorian(displayYear, displayMonth, d); return isoOf(g.year, g.month, g.day); };
	} else if (system === 'chinese' || system === 'korean') {
		// §B — lunisolar: displayMonth is an ORDINAL (1..12/13). Drive the grid by ISO from Intl.
		const cal = LUNISOLAR_CAL[system]!;
		const months = lunarMonths(displayYear, cal);
		const ord = Math.min(Math.max(1, displayMonth), months.length);
		firstISO = months[ord - 1];
		const nextISO = (ord < months.length) ? months[ord] : lunarMonths(displayYear + 1, cal)[0];
		daysInThisMonth = Math.round((Date.parse(nextISO) - Date.parse(firstISO)) / 86400000);
		const nv = lunarNav(firstISO, cal);
		// Localised name carries the leap marker (闰二月 / 윤2월); (N) is the month number (a leap month
		// shares its sibling's number); relatedYear is the Gregorian-ish year (no Latin era suffix).
		// Name: native script (五月) OR the user's phonetic choice (Wǔyuè); number + year stay in the UI locale.
		const monthName = lunarMonthName(system, firstISO, locale);
		monthLabel = `${monthName} (${localeNum(nv.monthNum, locale)}) · ${lunarYearLabel(system, firstISO, nv.relatedYear, locale)}`;
		isoForDay = (d) => isoShift(firstISO, d - 1);
	} else {
		const T = _Temporal;
		const cal = TEMPORAL_CAL[system]!;
		const first = T.PlainDate.from({ year: displayYear, month: displayMonth, day: 1, calendar: cal });
		daysInThisMonth = first.daysInMonth;
		const g1 = first.withCalendar('iso8601');
		firstISO = isoOf(g1.year, g1.month, g1.day);
		// §A.4 + §A.4b — build "MonthName (N) Year" ourselves: `{month:'long'}` gives the bare name
		// (no ERA — Intl appended "AP"/"Śaka"/"BE" which the rich header's suffix would DOUBLE), plus
		// the month NUMBER (helps with unfamiliar cultural months) + the year.
		const monthName = new Intl.DateTimeFormat(locale, { calendar: cal, month: 'long' }).format(new Date(firstISO));
		monthLabel = `${monthName} (${localeNum(displayMonth, locale)}) ${localeNum(displayYear, locale)}`;
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
	if (system === 'chinese' || system === 'korean') return localeNum(lunarNav(iso, LUNISOLAR_CAL[system]!).day, locale);
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
	if (system === 'chinese' || system === 'korean') {
		// §B — lunisolar: `month` is an ORDINAL; the year has 12 OR 13 months (a leap year).
		const cal = LUNISOLAR_CAL[system]!;
		const total = lunarMonths(year, cal).length;
		let o = month + dir, y = year;
		if (o > total) { y++; o = 1; } else if (o < 1) { y--; o = lunarMonths(y, cal).length; }
		return { year: y, month: o };
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
	if (system === 'chinese' || system === 'korean') {
		// §B — lunisolar: relatedYear + month number + day. (Leap-month disambiguation for the
		// frontmatter field is a §C concern — the display path uses displayDayNum, not this.)
		const nv = lunarNav(iso, LUNISOLAR_CAL[system]!);
		return { year: nv.relatedYear, month: nv.monthNum, day: nv.day };
	}
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

/** "June (6) 2026" (or a range) from the grid's first/last Gregorian ISO cells.
 *  §A.4b — month number beside each name, in the secondary subtitle too. */
function gregRange(firstISO: string, lastISO: string, locale: string): string {
	const f = new Date(firstISO), l = new Date(lastISO);
	const mf = new Intl.DateTimeFormat(locale, { month: 'long' });
	const mn = (dt: Date) => `${mf.format(dt)} (${localeNum(dt.getMonth() + 1, locale)})`;
	const myr = (dt: Date) => `${mn(dt)} ${localeNum(dt.getFullYear(), locale)}`;
	if (f.getFullYear() === l.getFullYear() && f.getMonth() === l.getMonth()) return myr(l);
	if (f.getFullYear() === l.getFullYear()) return `${mn(f)} – ${myr(l)}`;
	return `${myr(f)} – ${myr(l)}`;
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
	const mn = (h: { month: number }) => `${names?.[h.month - 1] ?? `M${h.month}`} (${localeNum(h.month, locale)})`;
	const myr = (h: { year: number; month: number }) => `${mn(h)} ${localeNum(h.year, locale)}`;
	if (a.year === b.year && a.month === b.month) return `${myr(a)} ${suffix}`;
	if (a.year === b.year) return `${mn(a)} – ${myr(b)} ${suffix}`;
	return `${myr(a)} – ${myr(b)} ${suffix}`;
}

/** §B — the lunisolar (chinese/korean) month range for a SECONDARY subtitle, "MonthName (N) Year"
 *  with the leap marker carried by the localised name (闰二月 / 윤2월). Uses lunarNav on the boundary
 *  ISOs directly (no ordinal math needed — the boundary date's own month is what we label). */
function lunarRange(system: CalendarSystem, firstISO: string, lastISO: string, locale: string): string {
	const cal = LUNISOLAR_CAL[system]!;
	const fnv = lunarNav(firstISO, cal), lnv = lunarNav(lastISO, cal);
	const name = (iso: string) => lunarMonthName(system, iso, locale);
	const mn = (iso: string, nv: typeof fnv) => `${name(iso)} (${localeNum(nv.monthNum, locale)})`;
	const myr = (iso: string, nv: typeof fnv) => `${mn(iso, nv)} ${lunarYearLabel(system, iso, nv.relatedYear, locale)}`;
	const same = fnv.relatedYear === lnv.relatedYear && fnv.monthNum === lnv.monthNum && fnv.isLeap === lnv.isLeap;
	if (same) return myr(lastISO, lnv);
	if (fnv.relatedYear === lnv.relatedYear) return `${mn(firstISO, fnv)} – ${myr(lastISO, lnv)}`;
	return `${myr(firstISO, fnv)} – ${myr(lastISO, lnv)}`;
}

/** The month-year range of [firstISO, lastISO] expressed in `system` — used for the SECONDARY-
 *  calendar subtitle. Gregorian/Hijri reuse their dedicated formatters; Temporal systems via Intl. */
function systemRange(system: CalendarSystem, firstISO: string, lastISO: string, locale: string): string {
	if (system === 'gregorian') return gregRange(firstISO, lastISO, locale);
	if (system === 'hijri') return hijriRange(firstISO, lastISO, locale);
	if (system === 'chinese' || system === 'korean') return lunarRange(system, firstISO, lastISO, locale);
	const cal = TEMPORAL_CAL[system];
	if (!cal || !_Temporal) return '';
	try {
		const a = _Temporal.PlainDate.from(firstISO).withCalendar(cal);
		const b = _Temporal.PlainDate.from(lastISO).withCalendar(cal);
		const df = new Date(firstISO), dl = new Date(lastISO);
		const mf = new Intl.DateTimeFormat(locale, { calendar: cal, month: 'long' }); // bare name (no era)
		const mn = (dt: Date, mon: number) => `${mf.format(dt)} (${localeNum(mon, locale)})`;
		const myr = (dt: Date, mon: number, yr: number) => `${mn(dt, mon)} ${localeNum(yr, locale)}`;
		if (a.year === b.year && a.month === b.month) return myr(dl, b.month, b.year);
		if (a.year === b.year) return `${mn(df, a.month)} – ${myr(dl, b.month, b.year)}`;
		return `${myr(df, a.month, a.year)} – ${myr(dl, b.month, b.year)}`;
	} catch { return ''; }
}

/** Build the RICH grid (dual dates, moon phase, week numbers, Hijri events/sacred). */
export function buildRichMonthGrid(
	system: CalendarSystem,
	displayYear: number,
	displayMonth: number,
	locale: string,
	weekStart: 0 | 1 = 0,
	secondary: CalendarSystem | 'none' = 'none',
): RichMonthGrid {
	const base = buildMonthGrid(system, displayYear, displayMonth, locale, weekStart);
	const H = _Hijri;
	const isAr = locale.startsWith('ar');

	const cells: RichCell[] = base.cells.map((c) => {
		const [gy, gm, gd] = c.iso.split('-').map(Number);
		const moon = H ? H.getMoonPhase(gy, gm, gd) : null;
		// Per-cell second date = the user's chosen SECONDARY system ("none" → single calendar,
		// no second date under the day). Skipped when it would just repeat the primary.
		let subLabel = '';
		if (secondary !== 'none' && secondary !== system) {
			try { subLabel = displayDayNum(secondary, c.iso, locale); } catch { subLabel = ''; }
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
	else if (system === 'indian') suffix = 'SE';   // §A.4 — Saka Era
	else if (system === 'buddhist') suffix = 'BE';  // §A.4 — Buddhist Era
	const isSacred = system === 'hijri' && !!H && H.isSacredMonth(displayMonth);

	const firstISO = base.cells[0]?.iso ?? '';
	const lastISO = base.cells[base.cells.length - 1]?.iso ?? '';
	// The subtitle = the SECONDARY calendar's month range. 'none' → no subtitle (single calendar);
	// skipped if it would just repeat the primary. Governed by the same setting as the per-cell 2nd date.
	const subtitleRange = (secondary !== 'none' && secondary !== system && firstISO && lastISO)
		? systemRange(secondary, firstISO, lastISO, locale)
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
