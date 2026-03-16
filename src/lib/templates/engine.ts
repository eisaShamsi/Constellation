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

// ─── Sync Engine (backward compat) ───

/**
 * Process template variables synchronously (basic variables only).
 * Use processTemplateAsync for full feature set.
 */
export function processTemplate(raw: string, ctx: TemplateContext): TemplateResult {
	const now = new Date();
	let content = raw;

	// {{yesterday:FORMAT}} / {{yesterday}}
	content = content.replace(/\{\{yesterday(?::([^}]+))?\}\}/gi, (_m, fmt?: string) =>
		formatDate(addDays(now, -1), fmt || 'YYYY-MM-DD'));

	// {{tomorrow:FORMAT}} / {{tomorrow}}
	content = content.replace(/\{\{tomorrow(?::([^}]+))?\}\}/gi, (_m, fmt?: string) =>
		formatDate(addDays(now, 1), fmt || 'YYYY-MM-DD'));

	// {{date+N:FORMAT}} / {{date-N:FORMAT}} / {{date+N}} / {{date-N}}
	content = content.replace(/\{\{date([+-]\d+)(?::([^}]+))?\}\}/g, (_m, offset: string, fmt?: string) =>
		formatDate(addDays(now, parseInt(offset, 10)), fmt || 'YYYY-MM-DD'));

	// {{date:FORMAT}}
	content = content.replace(/\{\{date:([^}]+)\}\}/g, (_m, fmt: string) => formatDate(now, fmt));

	// {{date}}
	content = content.replace(/\{\{date\}\}/gi, formatDate(now, 'YYYY-MM-DD'));

	// {{time}}
	content = content.replace(/\{\{time\}\}/gi,
		`${String(now.getHours()).padStart(2, '0')}:${String(now.getMinutes()).padStart(2, '0')}`);

	// {{title}}
	content = content.replace(/\{\{title\}\}/gi, ctx.title);

	// {{folder}}
	content = content.replace(/\{\{folder\}\}/gi, ctx.folder);

	// {{library}}
	content = content.replace(/\{\{(?:vault|library)\}\}/gi, ctx.library);

	// {{frontmatter.KEY}} — sync access from context
	if (ctx.frontmatter) {
		content = content.replace(/\{\{frontmatter\.([^}]+)\}\}/g, (_m, key: string) =>
			ctx.frontmatter?.[key.trim()] ?? '');
	}

	// {{cursor}}
	const cursorIdx = content.indexOf(CURSOR_MARKER);
	const cursorOffset = cursorIdx >= 0 ? cursorIdx : null;
	content = content.replace(/\{\{cursor\}\}/gi, '');

	return { content, cursorOffset };
}

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
	content = content.replace(/\{\{title\}\}/gi, ctx.title);

	// 8. {{folder}}
	content = content.replace(/\{\{folder\}\}/gi, ctx.folder);

	// 9. {{library}}
	content = content.replace(/\{\{(?:vault|library)\}\}/gi, ctx.library);

	// 10. {{clipboard}} — async
	if (/\{\{clipboard\}\}/i.test(content) && callbacks.getClipboard) {
		try {
			const clip = await callbacks.getClipboard();
			content = content.replace(/\{\{clipboard\}\}/gi, clip);
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

	// 13. {{prompt:Question}} / {{prompt:Question|default}} — sequential
	if (callbacks.promptUser) {
		const promptRegex = /\{\{prompt:([^}]+)\}\}/i;
		let match = promptRegex.exec(content);
		while (match) {
			const parts = match[1].split('|');
			const question = parts[0].trim();
			const defaultVal = parts[1]?.trim();
			const answer = await callbacks.promptUser(question, defaultVal);
			content = content.slice(0, match.index) + (answer ?? '') + content.slice(match.index + match[0].length);
			match = promptRegex.exec(content);
		}
	}

	// 14. {{suggester:opt1,opt2,...}} — sequential
	if (callbacks.suggestOptions) {
		const suggestRegex = /\{\{suggester:([^}]+)\}\}/i;
		let match = suggestRegex.exec(content);
		while (match) {
			const options = match[1].split(',').map(s => s.trim()).filter(Boolean);
			const chosen = await callbacks.suggestOptions(options);
			content = content.slice(0, match.index) + (chosen ?? '') + content.slice(match.index + match[0].length);
			match = suggestRegex.exec(content);
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
 */
export function extractTemplateBody(content: string): string {
	if (!content.startsWith('---')) return content;
	const endIdx = content.indexOf('---', 3);
	if (endIdx < 0) return content;
	return content.slice(endIdx + 3).trimStart();
}
