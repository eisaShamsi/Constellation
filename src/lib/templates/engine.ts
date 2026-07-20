// ─── Constellation Template Engine ───
// Processes template variables with sync and async support.
// Sync: {{date}}, {{date:FORMAT}}, {{time}}, {{title}}, {{folder}}, {{library}}, {{cursor}}
// Async: {{clipboard}}, {{frontmatter.KEY}}, {{file.createdAt}}, {{file.modifiedAt}},
//        {{yesterday}}, {{tomorrow}}, {{date+N}}, {{date-N}}, {{prompt:Q}}, {{suggester:opts}}

export interface TemplateContext {
	title: string;
	folder: string;
	library: string;
	filePath?: string;
	frontmatter?: Record<string, string>;
}

export interface TemplateCallbacks {
	promptUser?: (question: string, defaultValue?: string) => Promise<string | null>;
	suggestOptions?: (options: string[]) => Promise<string | null>;
	getClipboard?: () => Promise<string>;
	getFileMetadata?: (filePath: string) => Promise<{ created: number; modified: number } | null>;
}

/** Cursor placeholder — replaced last so we can return its position */
const CURSOR_MARKER = '{{cursor}}';

export interface TemplateResult {
	content: string;
	cursorOffset: number | null;
}

// ─── Date Helpers ───

function addDays(d: Date, n: number): Date {
	const result = new Date(d);
	result.setDate(result.getDate() + n);
	return result;
}

/**
 * Format a date with a subset of common format tokens.
 * Supports: YYYY, YY, MM, DD, HH, mm, ss, ddd, dddd, MMM, MMMM
 */
function formatDate(d: Date, fmt: string): string {
	const year = d.getFullYear();
	const month = d.getMonth();
	const day = d.getDate();
	const hours = d.getHours();
	const mins = d.getMinutes();
	const secs = d.getSeconds();
	const dow = d.getDay();

	const dayNames = ['Sunday', 'Monday', 'Tuesday', 'Wednesday', 'Thursday', 'Friday', 'Saturday'];
	const dayNamesShort = ['Sun', 'Mon', 'Tue', 'Wed', 'Thu', 'Fri', 'Sat'];
	const monthNames = ['January', 'February', 'March', 'April', 'May', 'June',
		'July', 'August', 'September', 'October', 'November', 'December'];
	const monthNamesShort = ['Jan', 'Feb', 'Mar', 'Apr', 'May', 'Jun',
		'Jul', 'Aug', 'Sep', 'Oct', 'Nov', 'Dec'];

	// ONE PASS over the format string, longest tokens first.
	//
	// This was a chain of `.replace('YYYY', …).replace('YY', …)…` — two defects at once. Each
	// `.replace` with a STRING pattern replaces only the FIRST occurrence, so a format repeating a
	// token ("YYYY … YYYY", a header + footer date) left the second one unexpanded; and that
	// leftover `YYYY` was then eaten by the NEXT rule's `YY`, producing "26YY". Serial passes also
	// mean an earlier substitution's OUTPUT is re-scanned by later rules.
	//
	// A single alternation, ordered longest-first, expands each token exactly once from the
	// original string and can never re-read its own output.
	const map: Record<string, string> = {
		YYYY: String(year),
		YY: String(year).slice(-2),
		MMMM: monthNames[month],
		MMM: monthNamesShort[month],
		MM: String(month + 1).padStart(2, '0'),
		DD: String(day).padStart(2, '0'),
		dddd: dayNames[dow],
		ddd: dayNamesShort[dow],
		HH: String(hours).padStart(2, '0'),
		mm: String(mins).padStart(2, '0'),
		ss: String(secs).padStart(2, '0'),
	};
	// Longest-first so YYYY wins over YY, MMMM over MMM over MM, dddd over ddd.
	return fmt.replace(/YYYY|MMMM|dddd|MMM|ddd|YY|MM|DD|HH|mm|ss/g, (tok) => map[tok] ?? tok);
}

/**
 * A replacer that yields `s` VERBATIM.
 *
 * `String.replace(pattern, replacementString)` treats `$&`, `$'`, "$`" and `$1` as special —
 * so passing user text (a note title, clipboard contents, a prompt answer) as the replacement
 * string lets those sequences inject the matched text instead of themselves. A note titled
 * "Cost $& benefit" expanded to garbage. A FUNCTION replacer is never interpreted, so every
 * user-supplied value below goes through this.
 */
const verbatim = (s: string) => () => s;

// ─── Sync engine REMOVED (MIG-TPL §1, 2026-07-19) ───
//
// `processTemplate` had ZERO callers (verified repo-wide) while still being exported and
// imported — and it silently lacked clipboard / file.* / prompt / suggester, leaving those
// tokens verbatim in the output. A second engine that quietly does less than the real one is
// a trap for the next caller, not backward compatibility. `processTemplateAsync` is the engine.

// ─── Async Engine (full feature set) ───

/**
 * Process template variables asynchronously.
 * Supports all sync variables plus: {{clipboard}}, {{frontmatter.KEY}},
 * {{file.createdAt}}, {{file.modifiedAt}}, {{prompt:Q}}, {{suggester:opts}}
 */
export async function processTemplateAsync(
	raw: string,
	ctx: TemplateContext,
	callbacks: TemplateCallbacks = {}
): Promise<TemplateResult> {
	const now = new Date();
	let content = raw;

	// 1. {{yesterday:FORMAT}} / {{yesterday}}
	content = content.replace(/\{\{yesterday(?::([^}]+))?\}\}/gi, (_m, fmt?: string) =>
		formatDate(addDays(now, -1), fmt || 'YYYY-MM-DD'));

	// 2. {{tomorrow:FORMAT}} / {{tomorrow}}
	content = content.replace(/\{\{tomorrow(?::([^}]+))?\}\}/gi, (_m, fmt?: string) =>
		formatDate(addDays(now, 1), fmt || 'YYYY-MM-DD'));

	// 3. {{date+N:FORMAT}} / {{date-N:FORMAT}}
	content = content.replace(/\{\{date([+-]\d+)(?::([^}]+))?\}\}/g, (_m, offset: string, fmt?: string) =>
		formatDate(addDays(now, parseInt(offset, 10)), fmt || 'YYYY-MM-DD'));

	// 4. {{date:FORMAT}}
	content = content.replace(/\{\{date:([^}]+)\}\}/g, (_m, fmt: string) => formatDate(now, fmt));

	// 5. {{date}}
	content = content.replace(/\{\{date\}\}/gi, formatDate(now, 'YYYY-MM-DD'));

	// 6. {{time}}
	content = content.replace(/\{\{time\}\}/gi,
		`${String(now.getHours()).padStart(2, '0')}:${String(now.getMinutes()).padStart(2, '0')}`);

	// 7. {{title}}
	content = content.replace(/\{\{title\}\}/gi, verbatim(ctx.title));

	// 8. {{folder}}
	content = content.replace(/\{\{folder\}\}/gi, verbatim(ctx.folder));

	// 9. {{library}}
	content = content.replace(/\{\{(?:vault|library)\}\}/gi, verbatim(ctx.library));

	// 10. {{clipboard}} — async
	if (/\{\{clipboard\}\}/i.test(content) && callbacks.getClipboard) {
		try {
			const clip = await callbacks.getClipboard();
			content = content.replace(/\{\{clipboard\}\}/gi, verbatim(clip));
		} catch {
			content = content.replace(/\{\{clipboard\}\}/gi, '');
		}
	}

	// 11. {{frontmatter.KEY}}
	if (ctx.frontmatter) {
		content = content.replace(/\{\{frontmatter\.([^}]+)\}\}/g, (_m, key: string) =>
			ctx.frontmatter?.[key.trim()] ?? '');
	}

	// 12. {{file.createdAt:FORMAT}} / {{file.createdAt}} / {{file.modifiedAt:FORMAT}} / {{file.modifiedAt}}
	if (/\{\{file\.(createdAt|modifiedAt)/i.test(content) && callbacks.getFileMetadata && ctx.filePath) {
		try {
			const meta = await callbacks.getFileMetadata(ctx.filePath);
			if (meta) {
				content = content.replace(/\{\{file\.(createdAt|modifiedAt)(?::([^}]+))?\}\}/gi,
					(_m, field: string, fmt?: string) => {
						const ts = field.toLowerCase() === 'createdat' ? meta.created : meta.modified;
						const d = new Date(ts * 1000);
						return formatDate(d, fmt || 'YYYY-MM-DD');
					});
			}
		} catch {
			content = content.replace(/\{\{file\.(createdAt|modifiedAt)(?::([^}]+))?\}\}/gi, '');
		}
	}

	// 13. {{prompt:Question}} / {{prompt:Question|default}} — sequential.
	//
	// THE ANSWER IS DATA, NEVER SYNTAX. The scan resumes AFTER the inserted answer (`searchFrom`)
	// instead of restarting at 0, so a reply containing `{{prompt:…}}` is left verbatim rather
	// than re-prompted. The old loop re-`exec`ed the whole string including what it had just
	// inserted — a second unexpected dialog at best, unbounded in the pathological case.
	if (callbacks.promptUser) {
		const promptRegex = /\{\{prompt:([^}]+)\}\}/i;
		let searchFrom = 0;
		let match = promptRegex.exec(content.slice(searchFrom));
		while (match) {
			const at = searchFrom + match.index;
			const parts = match[1].split('|');
			const question = parts[0].trim();
			const defaultVal = parts[1]?.trim();
			const answer = await callbacks.promptUser(question, defaultVal);
			const text = answer ?? '';
			content = content.slice(0, at) + text + content.slice(at + match[0].length);
			searchFrom = at + text.length;                       // resume PAST the answer
			match = promptRegex.exec(content.slice(searchFrom));
		}
	}

	// 14. {{suggester:opt1,opt2,...}} — sequential. Same inertness rule as prompts: the chosen
	// option is data, so the scan resumes past it rather than re-reading it as syntax.
	if (callbacks.suggestOptions) {
		const suggestRegex = /\{\{suggester:([^}]+)\}\}/i;
		let searchFrom = 0;
		let match = suggestRegex.exec(content.slice(searchFrom));
		while (match) {
			const at = searchFrom + match.index;
			const options = match[1].split(',').map(s => s.trim()).filter(Boolean);
			const chosen = await callbacks.suggestOptions(options);
			const text = chosen ?? '';
			content = content.slice(0, at) + text + content.slice(at + match[0].length);
			searchFrom = at + text.length;
			match = suggestRegex.exec(content.slice(searchFrom));
		}
	}

	// 15. {{cursor}} — last
	const cursorIdx = content.indexOf(CURSOR_MARKER);
	const cursorOffset = cursorIdx >= 0 ? cursorIdx : null;
	content = content.replace(/\{\{cursor\}\}/gi, '');

	return { content, cursorOffset };
}

// ─── Utility ───

/**
 * Extract only the body from a template file (strip frontmatter).
 *
 * The closing fence must be a `---` ON ITS OWN LINE. This used `indexOf('---', 3)`, which matched
 * a `---` ANYWHERE — including inside a frontmatter VALUE (an em-dash-heavy title, a URL like
 * `https://x/a---b`). The block was then cut early and its remainder leaked into the inserted body.
 *
 * Requires an opening fence on its own first line too, so a body that merely starts with a
 * horizontal rule is not mistaken for frontmatter. CRLF tolerated.
 */
export function extractTemplateBody(content: string): string {
	const m = /^---[ \t]*\r?\n([\s\S]*?)\r?\n---[ \t]*(?:\r?\n|$)/.exec(content);
	if (!m) return content;
	return content.slice(m[0].length).trimStart();
}
