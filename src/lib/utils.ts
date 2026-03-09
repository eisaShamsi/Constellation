import { marked, type TokenizerAndRendererExtension } from 'marked';

// ─── WikiLink extension for marked ───
const wikilinkExtension: TokenizerAndRendererExtension = {
	name: 'wikilink',
	level: 'inline',
	start(src: string) {
		return src.indexOf('[[');
	},
	tokenizer(src: string) {
		const match = src.match(/^\[\[([^\]|]+?)(?:\|([^\]]+?))?\]\]/);
		if (match) {
			return {
				type: 'wikilink',
				raw: match[0],
				target: match[1].trim(),
				display: (match[2] || match[1]).trim()
			};
		}
		return undefined;
	},
	renderer(token: any) {
		const target = token.target as string;
		const display = token.display as string;
		return `<a class="wikilink" data-wikilink="${encodeURIComponent(target)}" href="javascript:void(0)">${display}</a>`;
	}
};

marked.use({ extensions: [wikilinkExtension] });
marked.setOptions({ breaks: true, gfm: true });

/** Detect if text is predominantly RTL (Arabic, Hebrew, etc.) */
export function detectDir(text: string): 'rtl' | 'ltr' {
	const clean = text.replace(/^---[\s\S]*?---\n?/, '')
		.replace(/[#*_`\[\]()!>|~\-=+\d\s\\\/:.;,?!@$%^&{}"'<>]/g, '');
	const sample = clean.slice(0, 200);
	if (!sample) return 'ltr';
	const rtlChars = (sample.match(/[\u0600-\u06FF\u0750-\u077F\u08A0-\u08FF\uFB50-\uFDFF\uFE70-\uFEFF\u0590-\u05FF]/g) || []).length;
	return rtlChars > sample.length * 0.3 ? 'rtl' : 'ltr';
}

/** Render markdown to HTML. */
export function renderMarkdown(md: string): string {
	return marked.parse(md) as string;
}
