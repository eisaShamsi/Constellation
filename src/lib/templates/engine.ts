// ─── Constellation Template Engine ───
// Processes template variables: {{date}}, {{title}}, {{time}}, {{folder}}, {{library}}, {{date:FORMAT}}, {{cursor}}

export interface TemplateContext {
	title: string;
	folder: string;
	library: string;
}

/** Cursor placeholder — replaced last so we can return its position */
const CURSOR_MARKER = '{{cursor}}';

export interface TemplateResult {
	content: string;
	cursorOffset: number | null; // byte offset of {{cursor}} position, or null
}

/**
 * Process template variables in content.
 * Supported variables:
 *   {{date}}         → current date in YYYY-MM-DD
 *   {{date:FORMAT}}  → current date in custom format (subset of strftime)
 *   {{time}}         → current time in HH:MM
 *   {{title}}        → note title (file name without .md)
 *   {{folder}}       → parent folder name
 *   {{library}}        → library name
 *   {{cursor}}       → removed, cursor position returned
 */
export function processTemplate(raw: string, ctx: TemplateContext): TemplateResult {
	const now = new Date();

	let content = raw;

	// {{date:FORMAT}} — custom date format
	content = content.replace(/\{\{date:([^}]+)\}\}/g, (_match, fmt: string) => {
		return formatDate(now, fmt);
	});

	// {{date}} — YYYY-MM-DD
	content = content.replace(/\{\{date\}\}/gi, formatDate(now, 'YYYY-MM-DD'));

	// {{time}} — HH:MM
	content = content.replace(/\{\{time\}\}/gi,
		`${String(now.getHours()).padStart(2, '0')}:${String(now.getMinutes()).padStart(2, '0')}`
	);

	// {{title}}
	content = content.replace(/\{\{title\}\}/gi, ctx.title);

	// {{folder}}
	content = content.replace(/\{\{folder\}\}/gi, ctx.folder);

	// {{library}}
	content = content.replace(/\{\{(?:vault|library)\}\}/gi, ctx.library);

	// {{cursor}} — find position then remove
	const cursorIdx = content.indexOf(CURSOR_MARKER);
	const cursorOffset = cursorIdx >= 0 ? cursorIdx : null;
	content = content.replace(/\{\{cursor\}\}/gi, '');

	return { content, cursorOffset };
}

/**
 * Format a date with a subset of common format tokens.
 * Supports: YYYY, YY, MM, DD, HH, mm, ss, ddd, dddd, MMM, MMMM
 */
function formatDate(d: Date, fmt: string): string {
	const year = d.getFullYear();
	const month = d.getMonth(); // 0-indexed
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

	return fmt
		.replace('YYYY', String(year))
		.replace('YY', String(year).slice(-2))
		.replace('MMMM', monthNames[month])
		.replace('MMM', monthNamesShort[month])
		.replace('MM', String(month + 1).padStart(2, '0'))
		.replace('DD', String(day).padStart(2, '0'))
		.replace('dddd', dayNames[dow])
		.replace('ddd', dayNamesShort[dow])
		.replace('HH', String(hours).padStart(2, '0'))
		.replace('mm', String(mins).padStart(2, '0'))
		.replace('ss', String(secs).padStart(2, '0'));
}

/**
 * Extract only the body from a template file (strip frontmatter).
 */
export function extractTemplateBody(content: string): string {
	if (!content.startsWith('---')) return content;
	const endIdx = content.indexOf('---', 3);
	if (endIdx < 0) return content;
	return content.slice(endIdx + 3).trimStart();
}
