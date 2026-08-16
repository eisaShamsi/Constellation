import { marked, type TokenizerAndRendererExtension, type Tokens } from 'marked';
import hljs from 'highlight.js';
import DOMPurify from 'dompurify';
import { stripLinkTypePrefix } from '$lib/libraries/linkTypeRegistry';

// ─── WikiLink extension for marked ───
const wikilinkExtension: TokenizerAndRendererExtension = {
	name: 'wikilink',
	level: 'inline',
	start(src: string) {
		return src.indexOf('[[');
	},
	tokenizer(src: string) {
		// Embed: ![[target]]
		const embedMatch = src.match(/^!\[\[([^\]|]+?)(?:\|([^\]]+?))?\]\]/);
		if (embedMatch) {
			return {
				type: 'wikilink',
				raw: embedMatch[0],
				target: embedMatch[1].trim(),
				display: (embedMatch[2] || embedMatch[1]).trim(),
				embed: true
			};
		}
		const match = src.match(/^\[\[([^\]|]+?)(?:\|([^\]]+?))?\]\]/);
		if (match) {
			return {
				type: 'wikilink',
				raw: match[0],
				target: match[1].trim(),
				display: (match[2] || match[1]).trim(),
				embed: false
			};
		}
		return undefined;
	},
	renderer(token: any) {
		let target = token.target as string;
		let display = token.display as string;
		// MIG-067 — predicate-first [[type::target]]: strip a known type:: prefix
		// so the link resolves to (and, with no explicit alias, displays) the target.
		const _strippedTarget = stripLinkTypePrefix(target);
		if (_strippedTarget !== target) {
			if (display === target) display = _strippedTarget;
			target = _strippedTarget;
		}

		// Parse fragment (#heading or #^block-id)
		let fragment = '';
		let baseTarget = target;
		const hashIdx = target.indexOf('#');
		if (hashIdx >= 0) {
			fragment = target.slice(hashIdx + 1);
			baseTarget = target.slice(0, hashIdx);
			// If no explicit alias, show note > heading
			if (target === display) {
				const notePart = baseTarget || 'this note';
				display = fragment.startsWith('^') ? notePart : `${notePart} > ${fragment}`;
			}
		}

		// Parse typed link: [[note|type:related-to]]
		let linkType = '';
		const typeMatch = display.match(/^type:(.+)$/i);
		if (typeMatch) {
			linkType = typeMatch[1].trim();
			display = baseTarget || target; // Show note name instead of type spec
		}

		// For library:note syntax, show just the note name if no explicit alias
		const isCrossLibrary = baseTarget.includes(':') && target === token.target && target === (token.display as string);
		if (isCrossLibrary && !fragment) {
			display = baseTarget.split(':').pop()!.trim();
		}
		const fragmentAttr = fragment ? ` data-fragment="${encodeURIComponent(fragment)}"` : '';
		const typeAttr = linkType ? ` data-link-type="${encodeURIComponent(linkType)}"` : '';

		if (token.embed) {
			// Check if it's an image
			const ext = baseTarget.split('.').pop()?.toLowerCase() ?? '';
			const imgExts = ['png', 'jpg', 'jpeg', 'gif', 'svg', 'webp', 'bmp', 'avif'];
			if (imgExts.includes(ext)) {
				// Parse size from display: ![[img.png|640]] or ![[img.png|640x480]]
				const sizeMatch = display.match(/^(\d+)(?:x(\d+))?$/);
				if (sizeMatch && target !== display) {
					const w = sizeMatch[1];
					const h = sizeMatch[2];
					return `<img class="embed-image" data-embed="${encodeURIComponent(target)}" src="" alt="${escapeHtml(target)}" width="${w}" ${h ? `height="${h}"` : ''} />`;
				}
				return `<img class="embed-image" data-embed="${encodeURIComponent(target)}" src="" alt="${escapeHtml(target)}" />`;
			}
			return `<div class="embed-note" data-embed="${encodeURIComponent(target)}"${fragmentAttr}><span class="embed-icon">📄</span> ${escapeHtml(display)}</div>`;
		}
		const crossClass = isCrossLibrary ? ' cross-library' : '';
		const typeClass = linkType ? ` link-type-${linkType.replace(/[^a-z0-9-]/gi, '')}` : '';
		return `<a class="wikilink${crossClass}${typeClass}" data-wikilink="${encodeURIComponent(baseTarget || target)}"${fragmentAttr}${typeAttr} href="#" ${linkType ? `title="type: ${escapeHtml(linkType)}"` : ''}>${escapeHtml(display)}</a>`;
	}
};

// ─── Highlight extension (==text==) ───
const highlightExtension: TokenizerAndRendererExtension = {
	name: 'highlight',
	level: 'inline',
	start(src: string) {
		return src.indexOf('==');
	},
	tokenizer(src: string) {
		const match = src.match(/^==([^=]+)==/);
		if (match) {
			return { type: 'highlight', raw: match[0], text: match[1] };
		}
		return undefined;
	},
	renderer(token: any) {
		return `<mark>${escapeHtml(token.text)}</mark>`;
	}
};

// ─── Comment extension (%%hidden%%) ───
const commentExtension: TokenizerAndRendererExtension = {
	name: 'comment',
	level: 'inline',
	start(src: string) {
		return src.indexOf('%%');
	},
	tokenizer(src: string) {
		const match = src.match(/^%%([\s\S]+?)%%/);
		if (match) {
			return { type: 'comment', raw: match[0], text: match[1] };
		}
		return undefined;
	},
	renderer(_token: any) {
		return ''; // Hidden in reading view
	}
};

// ─── Inline math ($formula$) ───
const inlineMathExtension: TokenizerAndRendererExtension = {
	name: 'inlineMath',
	level: 'inline',
	start(src: string) {
		return src.indexOf('$');
	},
	tokenizer(src: string) {
		// Don't match $$ (block math)
		const match = src.match(/^\$([^\$\n]+?)\$/);
		if (match && !src.startsWith('$$')) {
			return { type: 'inlineMath', raw: match[0], formula: match[1] };
		}
		return undefined;
	},
	renderer(token: any) {
		return `<span class="math-inline" data-math="${encodeURIComponent(token.formula)}">${escapeHtml(token.formula)}</span>`;
	}
};

// ─── Callout renderer (overrides blockquote) ───
const calloutTypes: Record<string, { icon: string; color: string }> = {
	note: { icon: '📝', color: '#448aff' },
	abstract: { icon: '📋', color: '#00b0ff' },
	summary: { icon: '📋', color: '#00b0ff' },
	tldr: { icon: '📋', color: '#00b0ff' },
	info: { icon: 'ℹ️', color: '#00b0ff' },
	todo: { icon: '☑️', color: '#00b0ff' },
	tip: { icon: '💡', color: '#00bfa5' },
	hint: { icon: '💡', color: '#00bfa5' },
	important: { icon: '💡', color: '#00bfa5' },
	success: { icon: '✅', color: '#00c853' },
	check: { icon: '✅', color: '#00c853' },
	done: { icon: '✅', color: '#00c853' },
	question: { icon: '❓', color: '#64dd17' },
	help: { icon: '❓', color: '#64dd17' },
	faq: { icon: '❓', color: '#64dd17' },
	warning: { icon: '⚠️', color: '#ff9100' },
	caution: { icon: '⚠️', color: '#ff9100' },
	attention: { icon: '⚠️', color: '#ff9100' },
	failure: { icon: '❌', color: '#ff5252' },
	fail: { icon: '❌', color: '#ff5252' },
	missing: { icon: '❌', color: '#ff5252' },
	danger: { icon: '⚡', color: '#ff1744' },
	error: { icon: '⚡', color: '#ff1744' },
	bug: { icon: '🐛', color: '#f50057' },
	example: { icon: '📌', color: '#7c4dff' },
	quote: { icon: '💬', color: '#9e9e9e' },
	cite: { icon: '💬', color: '#9e9e9e' },
};

function escapeHtml(str: string): string {
	return str.replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;');
}

// ─── Footnote storage ───
const footnoteDefinitions = new Map<string, string>();
const footnoteReferences: string[] = [];

const footnoteRefExtension: TokenizerAndRendererExtension = {
	name: 'footnoteRef',
	level: 'inline',
	start(src: string) {
		return src.indexOf('[^');
	},
	tokenizer(src: string) {
		const match = src.match(/^\[\^([^\]]+)\]/);
		if (match) {
			return { type: 'footnoteRef', raw: match[0], ref: match[1] };
		}
		return undefined;
	},
	renderer(token: any) {
		const idx = footnoteReferences.indexOf(token.ref);
		const num = idx >= 0 ? idx + 1 : footnoteReferences.push(token.ref);
		const safeRef = escapeHtml(token.ref);
		return `<sup class="footnote-ref"><a href="#fn-${safeRef}" id="fnref-${safeRef}">${num}</a></sup>`;
	}
};

// Custom renderer to handle callouts in blockquotes and syntax highlighting
const renderer = {
	blockquote(this: any, token: Tokens.Blockquote): string {
		const rawText = token.raw || '';
		// Check for callout syntax: > [!type] or > [!type]+ or > [!type]-
		const calloutMatch = rawText.match(/>\s*\[!(\w+)\]([+-])?\s*(.*)/);
		if (calloutMatch) {
			const type = calloutMatch[1].toLowerCase();
			const foldable = calloutMatch[2];
			const title = calloutMatch[3]?.trim() || type.charAt(0).toUpperCase() + type.slice(1);
			const info = calloutTypes[type] || calloutTypes.note!;

			// Get the body content (everything after the first line)
			const lines = rawText.split('\n').slice(1);
			const bodyContent = lines.map(l => l.replace(/^>\s?/, '')).join('\n');
			const bodyHtml = bodyContent ? marked.parse(bodyContent) as string : '';

			const foldableClass = foldable ? ' callout-foldable' : '';
			const collapsedClass = foldable === '-' ? ' callout-collapsed' : '';

			return `<div class="callout callout-${type}${foldableClass}${collapsedClass}" style="--callout-color: ${info.color}">
				<div class="callout-title">${foldable ? '<span class="callout-fold">▶</span>' : ''}<span class="callout-icon">${info.icon}</span><span class="callout-title-text">${escapeHtml(title)}</span></div>
				<div class="callout-content">${bodyHtml}</div>
			</div>`;
		}
		// Default blockquote
		const body = this.parser.parse(token.tokens);
		return `<blockquote>${body}</blockquote>`;
	},
	code(token: Tokens.Code): string {
		const lang = token.lang || '';
		const code = token.text;
		// Mermaid diagrams
		if (lang === 'mermaid') {
			return `<div class="mermaid-container" data-mermaid="${encodeURIComponent(code)}"><pre class="mermaid">${escapeHtml(code)}</pre></div>`;
		}
		// Block math
		if (lang === 'math') {
			return `<div class="math-block" data-math="${encodeURIComponent(code)}">${escapeHtml(code)}</div>`;
		}
		// Syntax highlighting
		if (lang && hljs.getLanguage(lang)) {
			const highlighted = hljs.highlight(code, { language: lang }).value;
			return `<pre><code class="hljs language-${lang}">${highlighted}</code></pre>`;
		}
		// Auto-detect
		if (code.length > 20) {
			try {
				const result = hljs.highlightAuto(code);
				if (result.relevance > 5) {
					return `<pre><code class="hljs">${result.value}</code></pre>`;
				}
			} catch { /* fallback */ }
		}
		return `<pre><code>${escapeHtml(code)}</code></pre>`;
	}
};

// ─── Block math ($$...$$) handling via walkTokens ───
function processBlockMath(token: any) {
	if (token.type === 'paragraph' && token.raw) {
		const mathMatch = token.raw.match(/^\$\$([\s\S]+?)\$\$/);
		if (mathMatch) {
			token.type = 'html' as any;
			token.raw = mathMatch[0];
			(token as any).text = `<div class="math-block" data-math="${encodeURIComponent(mathMatch[1])}">${escapeHtml(mathMatch[1])}</div>`;
			token.tokens = [];
		}
	}
	// Collect footnote definitions
	if (token.type === 'paragraph' && token.raw) {
		const fnMatch = token.raw.match(/^\[\^([^\]]+)\]:\s*([\s\S]+)/);
		if (fnMatch) {
			footnoteDefinitions.set(fnMatch[1], fnMatch[2].trim());
			token.type = 'html' as any;
			(token as any).text = '';
			token.tokens = [];
		}
	}
}

marked.use({
	extensions: [wikilinkExtension, highlightExtension, commentExtension, inlineMathExtension, footnoteRefExtension],
	renderer: renderer as any,
	walkTokens: processBlockMath
});
marked.setOptions({ breaks: true, gfm: true });

/** Detect if text is predominantly RTL (Arabic, Hebrew, etc.)
 *  DELIBERATELY blind to the §B4 RLM/LRM direction marks (U+200E/U+200F are in no counted
 *  range): the marks are PER-LINE overrides — they must never shift the NOTE-level base this
 *  function feeds (H1→H2/H3). Per-line rendering honors them via bidiPlugin.detectLineDir. */
export function detectDir(text: string): 'rtl' | 'ltr' {
	const clean = text.replace(/^---[\s\S]*?---\n?/, '')
		.replace(/[#*_`\[\]()!>|~\-=+\d\s\\\/:.;,?!@$%^&{}"'<>]/g, '');
	const sample = clean.slice(0, 200);
	if (!sample) return 'ltr';
	const rtlChars = (sample.match(/[\u0600-\u06FF\u0750-\u077F\u08A0-\u08FF\uFB50-\uFDFF\uFE70-\uFEFF\u0590-\u05FF]/g) || []).length;
	return rtlChars > sample.length * 0.3 ? 'rtl' : 'ltr';
}

/**
 * Detect the dominant script of a text and map it to the locale whose typed-link
 * labels best fit it (MIG-067 \u00A7E.2). This lets a note's link labels read in the
 * note's OWN language, independent of the UI language \u2014 an English note shows
 * `supports`, an Arabic note shows `\u064A\u062F\u0639\u0645`, even when the app is set to the other.
 *
 * Coarse by design: script detection, not full language ID. Latin \u2192 en (the
 * English type names \u2014 German/French/etc. can't be told apart by script alone).
 * The Arabic-script trio is split by language-specific letters: Urdu-only letters
 * (\u0679 \u0688 \u0691 \u06D2 \u06BA \u06C1) \u2192 ur, Persian letters
 * (\u06AF \u0686 \u067E \u0698) \u2192 fa, otherwise \u2192 ar.
 */
export function dominantLocale(text: string): string {
	const clean = (text || '').replace(/^---[\s\S]*?---\n?/, '').slice(0, 4000);
	let arabic = 0, hebrew = 0, cyrillic = 0, devanagari = 0, hangul = 0, kana = 0, han = 0, latin = 0;
	let fa = false, ur = false;
	for (const ch of clean) {
		const c = ch.codePointAt(0) ?? 0;
		if ((c >= 0x0600 && c <= 0x06FF) || (c >= 0x0750 && c <= 0x077F) || (c >= 0x08A0 && c <= 0x08FF) || (c >= 0xFB50 && c <= 0xFDFF) || (c >= 0xFE70 && c <= 0xFEFF)) {
			arabic++;
			if (c === 0x0679 || c === 0x0688 || c === 0x0691 || c === 0x06D2 || c === 0x06BA || c === 0x06C1) ur = true;
			else if (c === 0x06AF || c === 0x0686 || c === 0x067E || c === 0x0698) fa = true;
		} else if (c >= 0x0590 && c <= 0x05FF) hebrew++;
		else if (c >= 0x0400 && c <= 0x04FF) cyrillic++;
		else if (c >= 0x0900 && c <= 0x097F) devanagari++;
		else if (c >= 0xAC00 && c <= 0xD7AF) hangul++;
		else if ((c >= 0x3040 && c <= 0x309F) || (c >= 0x30A0 && c <= 0x30FF)) kana++;
		else if ((c >= 0x4E00 && c <= 0x9FFF) || (c >= 0x3400 && c <= 0x4DBF)) han++;
		else if ((c >= 0x41 && c <= 0x5A) || (c >= 0x61 && c <= 0x7A) || (c >= 0xC0 && c <= 0x024F)) latin++;
	}
	const max = Math.max(arabic, hebrew, cyrillic, devanagari, hangul, kana, han, latin);
	if (max === 0) return 'en';
	if (max === arabic) return ur ? 'ur' : fa ? 'fa' : 'ar';
	if (max === hebrew) return 'he';
	if (max === cyrillic) return 'ru';
	if (max === devanagari) return 'hi';
	if (max === hangul) return 'ko';
	if (max === kana) return 'ja';
	if (max === han) return 'zh';
	return 'en';
}

/** Convert digits to Arabic-Indic numerals */
const HINDI_DIGITS = ['٠', '١', '٢', '٣', '٤', '٥', '٦', '٧', '٨', '٩'];

export function formatNumerals(value: string | number, style: 'arabic' | 'hindi' = 'arabic'): string {
	const str = String(value);
	if (style !== 'hindi') return str;
	return str.replace(/[0-9]/g, (d) => HINDI_DIGITS[parseInt(d)]);
}

/** Format a date string according to user preferences */
const MONTH_NAMES: Record<string, string[]> = {
	en: ['January','February','March','April','May','June','July','August','September','October','November','December'],
	ar: ['يناير','فبراير','مارس','أبريل','مايو','يونيو','يوليو','أغسطس','سبتمبر','أكتوبر','نوفمبر','ديسمبر'],
	fa: ['ژانویه','فوریه','مارس','آوریل','مه','ژوئن','ژوئیه','اوت','سپتامبر','اکتبر','نوامبر','دسامبر'],
	he: ['ינואר','פברואר','מרץ','אפריל','מאי','יוני','יולי','אוגוסט','ספטמבר','אוקטובר','נובמבר','דצמבר'],
	de: ['Januar','Februar','März','April','Mai','Juni','Juli','August','September','Oktober','November','Dezember'],
	fr: ['janvier','février','mars','avril','mai','juin','juillet','août','septembre','octobre','novembre','décembre'],
	es: ['enero','febrero','marzo','abril','mayo','junio','julio','agosto','septiembre','octubre','noviembre','diciembre'],
	pt: ['janeiro','fevereiro','março','abril','maio','junho','julho','agosto','setembro','outubro','novembro','dezembro'],
	tr: ['Ocak','Şubat','Mart','Nisan','Mayıs','Haziran','Temmuz','Ağustos','Eylül','Ekim','Kasım','Aralık'],
	ru: ['январь','февраль','март','апрель','май','июнь','июль','август','сентябрь','октябрь','ноябрь','декабрь'],
	ja: ['1月','2月','3月','4月','5月','6月','7月','8月','9月','10月','11月','12月'],
	ko: ['1월','2월','3월','4월','5월','6월','7월','8월','9월','10월','11월','12월'],
	zh: ['一月','二月','三月','四月','五月','六月','七月','八月','九月','十月','十一月','十二月'],
	hi: ['जनवरी','फरवरी','मार्च','अप्रैल','मई','जून','जुलाई','अगस्त','सितम्बर','अक्टूबर','नवम्बर','दिसम्बर'],
	ur: ['جنوری','فروری','مارچ','اپریل','مئی','جون','جولائی','اگست','ستمبر','اکتوبر','نومبر','دسمبر'],
};

export function formatDate(
	value: string,
	format: string = 'DD/MM/YYYY',
	locale: string = 'en',
	numeralStyle: 'arabic' | 'hindi' = 'arabic'
): string {
	if (!value) return '';
	try {
		const d = new Date(value + 'T00:00:00');
		if (isNaN(d.getTime())) return value;
		const day = d.getDate();
		const month = d.getMonth();
		const year = d.getFullYear();
		const pad = (n: number) => String(n).padStart(2, '0');
		const months = MONTH_NAMES[locale] || MONTH_NAMES.en;
		let result = '';
		switch (format) {
			case 'DD/MM/YYYY': result = `${pad(day)}/${pad(month + 1)}/${year}`; break;
			case 'MM/DD/YYYY': result = `${pad(month + 1)}/${pad(day)}/${year}`; break;
			case 'YYYY-MM-DD': result = `${year}-${pad(month + 1)}-${pad(day)}`; break;
			case 'YYYY/MM/DD': result = `${year}/${pad(month + 1)}/${pad(day)}`; break;
			case 'D MMMM YYYY': result = `${day} ${months[month]} ${year}`; break;
			case 'MMMM D, YYYY': result = `${months[month]} ${day}, ${year}`; break;
			default: result = `${pad(day)}/${pad(month + 1)}/${year}`;
		}
		return formatNumerals(result, numeralStyle);
	} catch { return value; }
}

/** Simple hash for cache key */
function quickHash(s: string): string {
	let h = 0;
	for (let i = 0; i < s.length; i++) {
		h = ((h << 5) - h + s.charCodeAt(i)) | 0;
	}
	return h.toString(36);
}

/** LRU cache for rendered markdown — avoids re-parsing unchanged content */
const _renderCache = new Map<string, string>();
const _RENDER_CACHE_MAX = 50;

/** Render markdown to HTML with all extended syntax. */
export function renderMarkdown(md: string): string {
	const key = `${quickHash(md)}_${md.length}`;
	const cached = _renderCache.get(key);
	if (cached !== undefined) return cached;

	// Reset footnote state
	footnoteDefinitions.clear();
	footnoteReferences.length = 0;

	let html = marked.parse(md) as string;

	// Append footnote definitions at bottom
	if (footnoteReferences.length > 0) {
		let fnHtml = '<div class="footnotes"><hr /><ol>';
		for (const ref of footnoteReferences) {
			const def = footnoteDefinitions.get(ref) || '';
			fnHtml += `<li id="fn-${escapeHtml(ref)}">${escapeHtml(def)} <a href="#fnref-${escapeHtml(ref)}" class="footnote-backref">\u21a9</a></li>`;
		}
		fnHtml += '</ol></div>';
		html += fnHtml;
	}

	const result = DOMPurify.sanitize(html, {
		ADD_TAGS: ['math', 'semantics', 'mrow', 'mi', 'mo', 'mn', 'msup', 'msub', 'mfrac', 'munder', 'mover', 'annotation'],
		ADD_ATTR: ['data-wikilink', 'data-embed', 'data-library', 'data-fragment', 'data-link-type', 'data-math', 'data-mermaid', 'data-path', 'data-highlight-term', 'class'],
		ALLOW_DATA_ATTR: true,
	});

	// Evict oldest entries when cache is full
	if (_renderCache.size >= _RENDER_CACHE_MAX) {
		const first = _renderCache.keys().next().value;
		if (first !== undefined) _renderCache.delete(first);
	}
	_renderCache.set(key, result);

	return result;
}

/** Post-process rendered HTML in the DOM: render math, mermaid, etc. */
export async function postProcessRenderedContent(container: HTMLElement) {
	// Render KaTeX math
	try {
		const katex = await import('katex');
		container.querySelectorAll('.math-inline').forEach(el => {
			const formula = decodeURIComponent(el.getAttribute('data-math') || '');
			try {
				el.innerHTML = DOMPurify.sanitize(katex.default.renderToString(formula, { throwOnError: false, displayMode: false }));
				el.classList.add('math-rendered');
			} catch { /* keep raw */ }
		});
		container.querySelectorAll('.math-block').forEach(el => {
			const formula = decodeURIComponent(el.getAttribute('data-math') || '');
			try {
				el.innerHTML = DOMPurify.sanitize(katex.default.renderToString(formula, { throwOnError: false, displayMode: true }));
				el.classList.add('math-rendered');
			} catch { /* keep raw */ }
		});
	} catch { /* katex not available */ }

	// Render Mermaid diagrams
	try {
		const mermaidModule = await import('mermaid');
		const mermaid = mermaidModule.default;
		mermaid.initialize({ startOnLoad: false, theme: 'default', securityLevel: 'strict' });
		const mermaidEls = container.querySelectorAll('.mermaid-container');
		for (let i = 0; i < mermaidEls.length; i++) {
			const el = mermaidEls[i] as HTMLElement;
			const code = decodeURIComponent(el.getAttribute('data-mermaid') || '');
			try {
				const { svg } = await mermaid.render(`mermaid-${Date.now()}-${i}`, code);
				el.innerHTML = DOMPurify.sanitize(svg, { USE_PROFILES: { svg: true } });
				el.classList.add('mermaid-rendered');
			} catch { /* keep raw */ }
		}
	} catch { /* mermaid not available */ }

	// Handle callout fold toggles (use event delegation to avoid duplicate listeners)
	if (!container.dataset.calloutDelegated) {
		container.dataset.calloutDelegated = 'true';
		container.addEventListener('click', (e) => {
			const titleEl = (e.target as HTMLElement).closest('.callout-foldable .callout-title');
			if (titleEl) {
				const callout = titleEl.closest('.callout');
				if (callout) callout.classList.toggle('callout-collapsed');
			}
		});
	}
}

/**
 * §141 — canonical normalisation for path keys used by every reactive
 * `Map<filePath, V>` in the app (`stageMap`, `maturityMap`,
 * `writeAheadBuffer`, `recentWrites`, the localStorage backups, etc.).
 * Forward-slash + lowercase. Defined ONCE so the semantic contract is in
 * one place — if a future filesystem change demands a different rule
 * (e.g. case-sensitive volumes, NFC normalisation) the entire app picks
 * it up from a single edit.
 */
export function normalizePathKey(p: string): string {
	return p.replace(/\\/g, '/').toLowerCase();
}

/**
 * MIG-100 — subscribe to a Svelte store but IGNORE the synchronous first fire
 * (which reports the CURRENT value at subscribe time, not a mutation). For
 * watchers that must react only to a real change AFTER the subscription —
 * e.g. "on the first user tab mutation" — the initial fire is a false
 * positive. One helper so the subtle idiom can't drift across copies.
 */
export function subscribeSkipInitial<T>(
	store: { subscribe: (fn: (v: T) => void) => () => void },
	fn: (v: T) => void
): () => void {
	let initial = true;
	return store.subscribe((v) => {
		if (initial) {
			initial = false;
			return;
		}
		fn(v);
	});
}

/**
 * §137 — migrate every entry in a path-keyed Map from `oldPath` to `newPath`.
 *
 * Rule 8 (Write-Time Derivation): when a path mutates — file rename, folder
 * rename — every reactive Map keyed by that path must follow it in the same
 * transaction. Without this, derived UI surfaces (file-tree stage emoji,
 * tab-strip maturity dot, alias index, etc.) silently fall out of sync the
 * moment the user renames anything; the symptom is "the icon disappeared
 * after I renamed it" and the cause is a stale Map keyed on the old path.
 *
 * Handles three cases atomically:
 *   - Direct file rename: `/lib/foo.md` → `/lib/foo v2.md`
 *   - Folder rename: every key under `/lib/folder/` rekeyed under the new prefix
 *   - No-op (oldKey === newKey, e.g. canonical-file rename where only the
 *     frontmatter title changed and the disk path stayed the same): returns
 *     `null` so the caller can skip the store update entirely.
 *
 * Returns `null` when no entry was migrated (no allocation; caller's `$state`
 * stays referentially equal so Svelte doesn't fire spurious reactivity).
 * Returns a fresh `Map<string, V>` when at least one entry migrated.
 *
 * Path normalisation: backslash → forward-slash + lowercase, matching the
 * canonical key shape used by `stageMap` / `maturityMap` / etc. in
 * `+layout.svelte`. Callers that key on a different shape must pre-normalise.
 */
export function migratePathKeyedMap<V>(
	map: Map<string, V>,
	oldPath: string,
	newPath: string,
): Map<string, V> | null {
	const oldKey = normalizePathKey(oldPath);
	const newKey = normalizePathKey(newPath);
	if (oldKey === newKey) return null;
	if (map.size === 0) return null;

	const prefix = oldKey + '/';
	let mutated = false;
	const next = new Map<string, V>();
	for (const [key, val] of map) {
		if (key === oldKey) {
			next.set(newKey, val);
			mutated = true;
		} else if (key.startsWith(prefix)) {
			next.set(newKey + key.substring(oldKey.length), val);
			mutated = true;
		} else {
			next.set(key, val);
		}
	}
	return mutated ? next : null;
}

/**
 * §139 — same migration as `migratePathKeyedMap`, but mutates the map
 * in place instead of returning a fresh Map. Use this when the consumer
 * map is a `SvelteMap` (or any container whose mutations are inherently
 * reactive) — no reassignment needed.
 *
 * Returns `true` if at least one entry moved, `false` otherwise. Consumer
 * can use the return value to decide whether to fire any post-migration
 * side-effects (rare).
 */
export function migratePathKeyedMapInPlace<V>(
	map: Map<string, V>,
	oldPath: string,
	newPath: string,
): boolean {
	const oldKey = normalizePathKey(oldPath);
	const newKey = normalizePathKey(newPath);
	if (oldKey === newKey) return false;
	if (map.size === 0) return false;

	const prefix = oldKey + '/';
	const moves: Array<[string, string, V]> = [];
	for (const [key, val] of map) {
		if (key === oldKey) {
			moves.push([key, newKey, val]);
		} else if (key.startsWith(prefix)) {
			moves.push([key, newKey + key.substring(oldKey.length), val]);
		}
	}
	if (moves.length === 0) return false;
	for (const [oldK, newK, val] of moves) {
		map.delete(oldK);
		map.set(newK, val);
	}
	return true;
}

/** Collect all wikilink targets from markdown text */
export function extractWikilinks(md: string): string[] {
	const links: string[] = [];
	const regex = /\[\[([^\]|]+?)(?:\|[^\]]+?)?\]\]/g;
	let match;
	while ((match = regex.exec(md)) !== null) {
		links.push(match[1].trim());
	}
	return links;
}

/** Get all note names from a library tree (for autocomplete) */
export function collectNoteNames(entries: any[]): { name: string; path: string }[] {
	const notes: { name: string; path: string }[] = [];
	function walk(entries: any[]) {
		for (const entry of entries) {
			if (!entry.is_dir && entry.name.endsWith('.md')) {
				notes.push({ name: entry.name.replace('.md', ''), path: entry.path });
			}
			if (entry.children) walk(entry.children);
		}
	}
	walk(entries);
	return notes;
}

// ─── Keyboard Shortcuts ───

/** Default keyboard shortcuts for all commands. Command ID → shortcut string. */
export const DEFAULT_SHORTCUTS: Record<string, string> = {
	'command-palette': 'Ctrl+P',
	'new-note': 'Ctrl+N',
	'quick-capture': 'Ctrl+Shift+N',
	'new-base': 'Ctrl+Shift+B',
	'quick-switch': 'Ctrl+O',
	'search': 'Ctrl+Shift+F',
	'toggle-edit': 'Ctrl+E',
	'insert-template': 'Ctrl+T',
	'toggle-bold': 'Ctrl+B',
	'toggle-italic': 'Ctrl+I',
	'close-note': 'Ctrl+W',
	'toggle-left': 'Ctrl+\\',
	'settings': 'Ctrl+,',
	'add-property': 'Ctrl+;',
	'second-screen': 'Ctrl+Shift+2',
	'nav-back': 'Alt+ArrowLeft',
	'nav-forward': 'Alt+ArrowRight',
	'insert-link': 'Ctrl+K',
	'duplicate-line': 'Ctrl+Shift+D',
	'toggle-comment': 'Ctrl+/',
	'select-next': 'Ctrl+D',
	// PJ-294 — a new EMPTY tab. Until now this lived only on the "+" button beside the tab
	// strip: no command, no shortcut, no palette entry, so the only way to open one was to
	// find and click that button. `Ctrl+Shift+T` is free in this table and reads the way it
	// does everywhere else.
	'new-tab': 'Ctrl+Shift+T',
};

/** True when a keyboard event targets an editable surface (an <input>, <textarea>,
 *  a contentEditable element, or the CodeMirror editor). Keydown handlers that
 *  preventDefault() bare keys must early-return on this so a (possibly user-rebound)
 *  shortcut never swallows a keystroke meant for the field. MIG-089 Language-First audit. */
export function isEditableTarget(e: Event): boolean {
	const t = e.target as HTMLElement | null;
	if (!t) return false;
	const tag = t.tagName;
	return tag === 'INPUT' || tag === 'TEXTAREA' || t.isContentEditable === true || !!t.closest?.('.cm-editor');
}

/** Convert a KeyboardEvent into a normalized shortcut string like "Ctrl+Shift+N". */
export function eventToShortcut(e: KeyboardEvent): string {
	const parts: string[] = [];
	if (e.ctrlKey || e.metaKey) parts.push('Ctrl');
	if (e.shiftKey) parts.push('Shift');
	if (e.altKey) parts.push('Alt');

	// Use e.code (physical key) for letter/digit keys when a modifier is held.
	// This ensures shortcuts work on non-Latin keyboard layouts (Arabic, Hebrew, etc.)
	// where e.key returns the locale character instead of the Latin letter — and,
	// for digits, that Shift+2 arrives as '2', not the layout's '@' (without this,
	// a 'Ctrl+Shift+2' binding can never match on any layout).
	let key = e.key;
	if (e.code && e.code.startsWith('Key') && (e.ctrlKey || e.metaKey || e.altKey)) {
		key = e.code.slice(3); // 'KeyP' → 'P'
	} else if (e.code && e.code.startsWith('Digit') && (e.ctrlKey || e.metaKey || e.altKey)) {
		key = e.code.slice(5); // 'Digit2' → '2'
	}
	if (key === ' ') key = 'Space';
	if (/^[a-zA-Z]$/.test(key)) key = key.toUpperCase();

	// Don't add modifier keys themselves
	if (['Control', 'Shift', 'Alt', 'Meta'].includes(key)) return '';

	parts.push(key);
	return parts.join('+');
}

/** Normalize a stored shortcut string for consistent comparison. */
export function normalizeShortcut(s: string): string {
	if (!s) return '';
	return s.split('+').map(part => {
		if (/^[a-zA-Z]$/.test(part)) return part.toUpperCase();
		if (part === '←') return 'ArrowLeft';
		if (part === '→') return 'ArrowRight';
		return part;
	}).join('+');
}

/** Resolve a command's shortcut: custom override if set, else default. */
export function getResolvedShortcut(commandId: string, customShortcuts: Record<string, string>): string {
	if (commandId in customShortcuts) return customShortcuts[commandId];
	return DEFAULT_SHORTCUTS[commandId] ?? '';
}

/**
 * Split a shortcut into its modifiers and its key — WITHOUT splitting on the delimiter blindly.
 *
 * `+` is both the separator and a key a user can press (the numpad one, and unshifted `+` on
 * German and Nordic layouts). `'+'.split('+')` gives `['', '']`, so a naive parse counts two
 * "parts" and concludes the combination has a modifier — which let a completely bare `+` past the
 * bare-key refusal, and rendered as an EMPTY row on the macOS display. Consuming known modifier
 * prefixes from the front instead leaves the remainder as the key, whatever character it is.
 */
export function parseShortcut(combo: string): { mods: string[]; key: string } {
	const KNOWN = ['Ctrl', 'Shift', 'Alt'];
	const mods: string[] = [];
	let rest = combo;
	for (;;) {
		const m = KNOWN.find((k) => rest.startsWith(k + '+'));
		if (!m) break;
		mods.push(m);
		rest = rest.slice(m.length + 1);
	}
	return { mods, key: rest };
}

/** True on macOS. Takes the platform string so it stays pure and testable. */
export function isMacPlatform(ua = typeof navigator !== 'undefined' ? navigator.userAgent : ''): boolean {
	return /Mac|iPhone|iPad|iPod/.test(ua);
}

/**
 * Format a shortcut string for display (e.g., "ArrowLeft" → "←").
 *
 * **Cross-Platform by Design.** The STORED form is already platform-neutral and must stay that
 * way: `eventToShortcut` maps both `ctrlKey` AND `metaKey` to the single token `Ctrl`, so a
 * binding saved on Windows with Ctrl is the same string a Mac user's ⌘ produces, and neither
 * needs migrating when they sync settings between machines. Only the DISPLAY differs — a Mac
 * user must see ⌘⇧T, not "Ctrl+Shift+T", for a key they press with Command.
 */
export function formatShortcut(s: string, mac = isMacPlatform()): string {
	if (!s) return '';
	const { mods, key } = parseShortcut(s);
	const shownKey =
		{ ArrowLeft: '←', ArrowRight: '→', ArrowUp: '↑', ArrowDown: '↓' }[key] ?? key;
	if (!mac) return [...mods, shownKey].join('+');
	// Mac convention: symbols, no separators — ⌘⇧T rather than "Cmd+Shift+T".
	const sym: Record<string, string> = { Ctrl: '⌘', Shift: '⇧', Alt: '⌥' };
	return mods.map((m) => sym[m] ?? m).join('') + shownKey;
}

/**
 * Combinations the global dispatcher answers ITSELF, before it ever reaches the command table —
 * so no command can be bound to them. (PJ-294)
 *
 * **This table is the dispatcher's, not a copy of it.** `handleGlobalKeydown` compares against
 * these same entries rather than re-testing the keys inline, because a private list that merely
 * *matches* the dispatcher is a list that stops matching the first time someone adds a handler.
 * The gate caught exactly that: `Ctrl+.` opens the emoji picker and returns before the command
 * loop, but it is not in `DEFAULT_SHORTCUTS`, so a conflict check that consulted only the command
 * ids reported it FREE — the binding saved, displayed as live, survived restarts, and could never
 * once fire.
 */
export const RESERVED_SHORTCUTS: Record<string, string> = {
	'Ctrl+.': 'emoji-icon-picker',
	Escape: 'close-overlay',
	// ── The EDITOR's keys. ────────────────────────────────────────────────────────────────────
	// The same hazard one layer down, and the one that would have hurt most. A command bound to
	// one of these wins: the global dispatcher is capture-phase and calls `preventDefault`, and
	// CodeMirror's `runHandlers` stops on an already-defaulted event — so the editor's own binding
	// never runs. Giving away Ctrl+Z means the user presses undo in a note, nothing happens, no
	// error appears, and the edit they wanted to take back is what the debounced save writes to
	// disk. Ctrl+F would likewise kill find-in-note, and Ctrl+X would leave the clipboard stale.
	//
	// These are the ones NotePane installs (`defaultKeymap`, `historyKeymap`, `searchKeymap`) that
	// no shipped default already claims. Ctrl+B/I/K/D and Ctrl+/ are deliberately app-owned and so
	// are absent here — the pinned test asserting no default lands on a reserved combination is
	// what keeps that distinction honest.
	'Ctrl+Z': 'editor-undo',
	'Ctrl+Y': 'editor-redo',
	'Ctrl+Shift+Z': 'editor-redo',
	'Ctrl+X': 'editor-cut',
	'Ctrl+C': 'editor-copy',
	'Ctrl+V': 'editor-paste',
	'Ctrl+A': 'editor-select-all',
	'Ctrl+F': 'editor-find',
	// Constellation's OWN editor keymaps (PJ-106 §B1/§B2/§B3 — the RTL-aware motion and selection
	// keys, mounted in NotePane, FocusPane and the conflict-merge view). Reserving the CodeMirror
	// stock keys above and stopping there was the eighth gate finding on this feature, and the
	// point where hand-listing stopped being defensible: every round added a source I had not
	// thought to consult. The list is now DERIVED — `tests/pj-294` scans every keymap declared in
	// `src/lib/editor/` and fails if any modified binding is missing from this table — so a keymap
	// added tomorrow cannot quietly become a combination the Hotkeys screen hands out.
	'Ctrl+ArrowUp': 'editor-paragraph-up',
	'Ctrl+ArrowDown': 'editor-paragraph-down',
	'Ctrl+L': 'editor-select-line',
	'Alt+L': 'editor-select-line',
	'Ctrl+Shift+L': 'editor-select-paragraph',
	'Ctrl+Shift+S': 'editor-select-sentence',
};

/** Why a combination may not be bound, or `null` when it may. */
export type ShortcutRefusal = 'bare-key' | 'reserved';

/**
 * May this combination be bound to a command? (PJ-294) — ONE function, so the screen's refusal and
 * its explanation cannot disagree about the reason.
 *
 * Two refusals, both about not taking keys away from the user:
 *   · **A bare key.** The dispatcher early-returns for editable targets, so a bare `A` binding
 *     would not eat your typing — but it WOULD fire on every stray press anywhere else, which is
 *     not something a rebinding screen should let someone do to themselves by accident. Function
 *     keys are exempt: they have no typing meaning.
 *   · **A reserved combination** (above): Escape, which the dispatcher documents as "always closes
 *     overlays (not remappable)" and whose loss would strand a user inside a full-page surface —
 *     including the Settings screen they rebound it from — and anything else the dispatcher
 *     handles before the command table.
 */
export function shortcutRefusal(
	combo: string,
	/** Additional reservations the CALLER knows about — in practice the editor's installed keymaps
	 *  (`$lib/editor/reservedKeys`), passed in rather than imported so `utils` stays free of
	 *  CodeMirror. */
	extraReserved: Record<string, string> = {},
): ShortcutRefusal | null {
	if (!combo) return 'bare-key';
	if (combo in RESERVED_SHORTCUTS || combo in extraReserved) return 'reserved';
	// Parsed, never `split('+')` — see `parseShortcut`: a bare `+` splits into two empty parts and
	// so counted as "modified", walking straight through the refusal below.
	const { mods, key } = parseShortcut(combo);
	// Escape in ANY form, not merely the bare key. Modifying it does not make it a different
	// key to the user — it is still the one they reach for to get out — and on Windows the OS
	// takes Ctrl+Escape for the Start menu, so the binding would be dead on arrival anyway.
	if (key === 'Escape') return 'reserved';
	const isFunctionKey = /^F([1-9]|1[0-9]|2[0-4])$/.test(key);
	return mods.length > 0 || isFunctionKey ? null : 'bare-key';
}

/**
 * Which OTHER command already answers to this combination? (PJ-294)
 *
 * Returns that command's id, or `null` when the combination is free. Resolution goes through
 * `getResolvedShortcut` so a command the user has already re-bound is compared on its CURRENT
 * binding, not the default it no longer uses — otherwise re-binding A to B's default and then B
 * to something else would report a conflict that no longer exists.
 */
export function findShortcutConflict(
	commandId: string,
	combo: string,
	customShortcuts: Record<string, string>,
	commandIds: string[],
): string | null {
	const target = normalizeShortcut(combo);
	if (!target) return null;
	// The caller's list is what is REGISTERED RIGHT NOW, and that is not the whole set: some
	// commands are conditional. `second-screen` (Ctrl+Shift+2) is only registered when a second
	// display is detected, so on an ordinary single-monitor machine it is absent from the list the
	// Hotkeys screen can pass — and its combination read as FREE. Binding something else to it
	// saved happily; the day a monitor was attached, the two collided and the dispatcher's
	// first-match-wins left the second-screen shortcut dead, with both rows still displaying it.
	//
	// So the union is taken HERE rather than asked of each caller. A caller holding a partial list
	// is the normal case, not a mistake to be remembered around.
	const ids = new Set([...commandIds, ...Object.keys(DEFAULT_SHORTCUTS), ...Object.keys(customShortcuts)]);
	for (const id of ids) {
		if (id === commandId) continue;
		if (normalizeShortcut(getResolvedShortcut(id, customShortcuts)) === target) return id;
	}
	return null;
}

/** Slugify a human-readable name for a download filename. */
function slugifyFilename(name: string): string {
	return (name || 'export').replace(/\s+/g, '-').toLowerCase();
}

/** Trigger a browser download of a JSON payload. Pass either a string or any serializable value. */
export function downloadJSON(filename: string, data: unknown): void {
	const json = typeof data === 'string' ? data : JSON.stringify(data, null, 2);
	const blob = new Blob([json], { type: 'application/json' });
	const url = URL.createObjectURL(blob);
	const a = document.createElement('a');
	a.href = url;
	a.download = filename.endsWith('.json') ? filename : `${slugifyFilename(filename)}.json`;
	a.click();
	URL.revokeObjectURL(url);
}

/** Open a native file picker for a .json file; resolves with its text, or null if cancelled. */
export function pickJSONFile(): Promise<string | null> {
	return new Promise((resolve) => {
		const input = document.createElement('input');
		input.type = 'file';
		input.accept = 'application/json,.json';
		input.onchange = async () => {
			const file = input.files?.[0];
			if (!file) return resolve(null);
			try { resolve(await file.text()); } catch { resolve(null); }
		};
		input.click();
	});
}
