/**
 * taskDates.ts — natural-language due-date resolution for task lines.
 *
 * MIG-080 §C.2 (Boss 2026-06-21): a user can write a task's due date in plain
 * words — "today", "tomorrow", "yesterday", "next week", "next month", a weekday
 * ("Monday", "next Friday"), or "in N days / in N weeks" — WITH or WITHOUT the
 * 📅 marker. On line commit the NotePane editor PINS it to a real fixed date,
 * converting the words in place to `📅 YYYY-MM-DD` (a visible, undoable editor
 * edit — never a save-path content rewrite, per the content-integrity class).
 *
 * Pure + deterministic: every function takes `now` so it is unit-testable and
 * never reads the clock implicitly.
 */

const WEEKDAYS: Record<string, number> = {
	sunday: 0, monday: 1, tuesday: 2, wednesday: 3, thursday: 4, friday: 5, saturday: 6,
};

function ymd(d: Date): string {
	const y = d.getFullYear();
	const m = String(d.getMonth() + 1).padStart(2, '0');
	const day = String(d.getDate()).padStart(2, '0');
	return `${y}-${m}-${day}`;
}

function addDays(now: Date, n: number): Date {
	const d = new Date(now.getFullYear(), now.getMonth(), now.getDate());
	d.setDate(d.getDate() + n);
	return d;
}

function addMonths(now: Date, n: number): Date {
	const d = new Date(now.getFullYear(), now.getMonth(), now.getDate());
	d.setMonth(d.getMonth() + n);
	return d;
}

/** Next occurrence of `weekday` strictly after today; `nextWeek` adds 7 more. */
function nextWeekday(now: Date, weekday: number, nextWeek: boolean): Date {
	const today = new Date(now.getFullYear(), now.getMonth(), now.getDate());
	let delta = (weekday - today.getDay() + 7) % 7;
	if (delta === 0) delta = 7;       // "Monday" on a Monday → the coming Monday
	if (nextWeek) delta += 7;          // "next Monday" → the Monday of next week
	return addDays(today, delta);
}

/**
 * The ordered keyword patterns. Longer / more-specific phrases come FIRST so
 * e.g. "next week" wins over "week" and "next friday" over "friday".
 * Each returns the resolved Date (relative to `now`), or null to skip.
 */
type Matcher = { re: RegExp; resolve: (m: RegExpExecArray, now: Date) => Date | null };

const MATCHERS: Matcher[] = [
	{ re: /\btoday\b/i,            resolve: (_m, now) => addDays(now, 0) },
	{ re: /\btomorrow\b/i,        resolve: (_m, now) => addDays(now, 1) },
	{ re: /\byesterday\b/i,       resolve: (_m, now) => addDays(now, -1) },
	{ re: /\bnext\s+week\b/i,     resolve: (_m, now) => addDays(now, 7) },
	{ re: /\bnext\s+month\b/i,    resolve: (_m, now) => addMonths(now, 1) },
	// "in 3 days" / "in 2 weeks"
	{ re: /\bin\s+(\d{1,3})\s+days?\b/i,  resolve: (m, now) => addDays(now, parseInt(m[1], 10)) },
	{ re: /\bin\s+(\d{1,3})\s+weeks?\b/i, resolve: (m, now) => addDays(now, parseInt(m[1], 10) * 7) },
	// "next Friday" / "Friday" (weekday names)
	{
		re: /\b(next\s+)?(sunday|monday|tuesday|wednesday|thursday|friday|saturday)\b/i,
		resolve: (m, now) => nextWeekday(now, WEEKDAYS[m[2].toLowerCase()], !!m[1]),
	},
];

export const TASK_RE = /^\s*[-*+]\s+\[[ xX/\-]\]\s/;   // "- [ ] " / "- [x] " / "* [/] " etc.

export interface DateOption {
	/** what's inserted on accept, e.g. "📅 2026-06-22" */
	label: string;
	/** the keyword this resolves, shown as the suggestion detail, e.g. "tomorrow" */
	detail: string;
	/** the resolved date, YYYY-MM-DD */
	date: string;
}

export interface DateCompletions {
	/** char offset (within `before`) where the trigger text starts (the replace-from) */
	from: number;
	options: DateOption[];
}

const ymdOpt = (key: string, d: Date): DateOption => ({ label: `\u{1F4C5} ${ymd(d)}`, detail: key, date: ymd(d) });

/** The fixed keyword menu offered after the `@` trigger, each resolved against `now`. */
function keywordMenu(now: Date): { key: string; opt: DateOption }[] {
	const out: { key: string; opt: DateOption }[] = [
		{ key: 'today', opt: ymdOpt('today', addDays(now, 0)) },
		{ key: 'tomorrow', opt: ymdOpt('tomorrow', addDays(now, 1)) },
		{ key: 'yesterday', opt: ymdOpt('yesterday', addDays(now, -1)) },
		{ key: 'next week', opt: ymdOpt('next week', addDays(now, 7)) },
		{ key: 'next month', opt: ymdOpt('next month', addMonths(now, 1)) },
	];
	for (const [name, idx] of Object.entries(WEEKDAYS)) {
		out.push({ key: name, opt: ymdOpt(name, nextWeekday(now, idx, false)) });
		out.push({ key: `next ${name}`, opt: ymdOpt(`next ${name}`, nextWeekday(now, idx, true)) });
	}
	return out;
}

/**
 * MIG-080 §C.2 (Boss 2026-06-21, research-backed): the AUTOSUGGEST source for
 * task due dates. Given the text BEFORE the cursor (on a task line — the caller
 * gates that), return `📅 YYYY-MM-DD` suggestions the user ACCEPTS (never a silent
 * rewrite). Two proven gates (Obsidian nldates `@` + Obsidian Tasks task-line
 * auto-suggest):
 *   (A) `@` trigger — "@", "@to", "@next w", "@in 3 days" → the keyword menu,
 *       filtered by the partial (explicit intent; the recommended path).
 *   (B) bare keyword fallback — a COMPLETE date phrase at the cursor ("tomorrow",
 *       "next week", "in 3 days") with no `@` → a single resolved suggestion (the
 *       "forgot the @" safety net). Non-destructive: ignored if the user keeps typing.
 * Returns null when nothing matches.
 */
export function taskDateCompletions(before: string, now: Date): DateCompletions | null {
	// (A) `@` trigger: @ + letters/digits/spaces up to the cursor.
	const at = before.match(/@\s*([\p{L}\d ]*)$/u);
	if (at) {
		const partial = at[1].toLowerCase().trim();
		const from = before.length - at[0].length;
		const options: DateOption[] = [];
		// dynamic "in N days/weeks"
		const inM = partial.match(/^in\s+(\d{1,3})\s+(days?|weeks?)$/);
		if (inM) {
			const n = parseInt(inM[1], 10);
			options.push(ymdOpt(partial, inM[2].startsWith('week') ? addDays(now, n * 7) : addDays(now, n)));
		}
		for (const { key, opt } of keywordMenu(now)) {
			if (partial === '' || key.startsWith(partial)) options.push(opt);
		}
		return options.length ? { from, options: options.slice(0, 8) } : null;
	}

	// (B) bare complete phrase at the cursor (no `@`).
	let best: { idx: number; date: Date; key: string } | null = null;
	for (const { re, resolve } of MATCHERS) {
		const m = before.match(new RegExp(re.source + '$', re.flags.replace('g', '')));
		if (m) {
			const d = resolve(m as RegExpExecArray, now);
			if (d) {
				const idx = before.length - m[0].length;
				if (!best || idx > best.idx) best = { idx, date: d, key: m[0].trim() };
			}
		}
	}
	if (best) return { from: best.idx, options: [ymdOpt(best.key, best.date)] };
	return null;
}
