import { marked, type TokenizerAndRendererExtension, type Tokens } from 'marked';
import hljs from 'highlight.js';
import DOMPurify from 'dompurify';

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
		const target = token.target as string;
		let display = token.display as string;

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
		// Dataview queries
		if (lang === 'dataview') {
			return `<div class="dataview-query" data-dataview="${encodeURIComponent(code)}"><pre class="dataview-source"><code>${escapeHtml(code)}</code></pre></div>`;
		}
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

/** Detect if text is predominantly RTL (Arabic, Hebrew, etc.) */
export function detectDir(text: string): 'rtl' | 'ltr' {
	const clean = text.replace(/^---[\s\S]*?---\n?/, '')
		.replace(/[#*_`\[\]()!>|~\-=+\d\s\\\/:.;,?!@$%^&{}"'<>]/g, '');
	const sample = clean.slice(0, 200);
	if (!sample) return 'ltr';
	const rtlChars = (sample.match(/[\u0600-\u06FF\u0750-\u077F\u08A0-\u08FF\uFB50-\uFDFF\uFE70-\uFEFF\u0590-\u05FF]/g) || []).length;
	return rtlChars > sample.length * 0.3 ? 'rtl' : 'ltr';
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
		ADD_ATTR: ['data-wikilink', 'data-embed', 'data-library', 'data-fragment', 'data-link-type', 'data-math', 'data-mermaid', 'data-dataview', 'data-path', 'data-highlight-term', 'class'],
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

	// Render Dataview queries
	const dataviewEls = container.querySelectorAll('.dataview-query:not(.dataview-rendered)');
	if (dataviewEls.length > 0) {
		const { executeDataviewQuery } = await import('$lib/dataview/store');
		// Get library paths from the global store
		const { libraries } = await import('$lib/libraries/store');
		const { get } = await import('svelte/store');
		const libraryList = get(libraries);
		const libraryPaths: [string, string][] = libraryList.map((v: any) => [v.name, v.path]);

		for (const el of dataviewEls) {
			const queryText = decodeURIComponent((el as HTMLElement).getAttribute('data-dataview') || '');
			if (!queryText) continue;
			el.classList.add('dataview-rendered');

			// Show loading state
			el.innerHTML = '<div class="dv-inline-loading">Loading query...</div>';

			try {
				const result = await executeDataviewQuery(queryText, libraryPaths);
				if (result.error) {
					el.innerHTML = `<div class="dv-inline-error">${DOMPurify.sanitize(result.error)}</div>`;
					continue;
				}

				let html = '';
				if (result.query_type === 'table') {
					html += '<div class="dv-inline-table-wrap"><table class="dv-inline-table">';
					html += '<thead><tr><th>File</th>';
					for (const col of result.columns) {
						html += `<th>${DOMPurify.sanitize(col)}</th>`;
					}
					html += '</tr></thead><tbody>';
					for (const row of result.rows) {
						const name = row.file_name.replace(/\.md$/, '');
						html += `<tr><td><a class="dv-inline-link" data-path="${DOMPurify.sanitize(row.file_path)}" data-library="${DOMPurify.sanitize(row.library_name)}">${DOMPurify.sanitize(name)}</a></td>`;
						for (const col of result.columns) {
							const val = row.properties[col] || '';
							html += `<td>${DOMPurify.sanitize(val)}</td>`;
						}
						html += '</tr>';
					}
					html += '</tbody></table></div>';
				} else if (result.query_type === 'list') {
					html += '<ul class="dv-inline-list">';
					for (const row of result.rows) {
						const name = row.file_name.replace(/\.md$/, '');
						html += `<li><a class="dv-inline-link" data-path="${DOMPurify.sanitize(row.file_path)}" data-library="${DOMPurify.sanitize(row.library_name)}">${DOMPurify.sanitize(name)}</a></li>`;
					}
					html += '</ul>';
				} else {
					html = '<div class="dv-inline-empty">No results</div>';
				}
				html += `<div class="dv-inline-footer">${result.rows.length} results &middot; ${result.query_time_ms}ms</div>`;
				el.innerHTML = html;
			} catch (e: any) {
				console.error('[Dataview] Query execution error:', e);
				el.innerHTML = `<div class="dv-inline-error">${DOMPurify.sanitize(e?.message || 'Query failed')}</div>`;
			}
		}
	}

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

/** Collect all tags from markdown text */
export function extractTags(md: string): string[] {
	const tags: string[] = [];
	// Inline tags: #tag (not in code blocks or URLs)
	const regex = /(?:^|\s)#([a-zA-Z\u0600-\u06FF][\w\u0600-\u06FF/\-]*)/g;
	let match;
	while ((match = regex.exec(md)) !== null) {
		if (!tags.includes(match[1])) tags.push(match[1]);
	}
	return tags;
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
};

/** Convert a KeyboardEvent into a normalized shortcut string like "Ctrl+Shift+N". */
export function eventToShortcut(e: KeyboardEvent): string {
	const parts: string[] = [];
	if (e.ctrlKey || e.metaKey) parts.push('Ctrl');
	if (e.shiftKey) parts.push('Shift');
	if (e.altKey) parts.push('Alt');

	// Use e.code (physical key) for letter keys when a modifier is held.
	// This ensures shortcuts work on non-Latin keyboard layouts (Arabic, Hebrew, etc.)
	// where e.key returns the locale character instead of the Latin letter.
	let key = e.key;
	if (e.code && e.code.startsWith('Key') && (e.ctrlKey || e.metaKey || e.altKey)) {
		key = e.code.slice(3); // 'KeyP' → 'P'
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

/** Format a shortcut string for display (e.g., "ArrowLeft" → "←"). */
export function formatShortcut(s: string): string {
	if (!s) return '';
	return s
		.replace('ArrowLeft', '←')
		.replace('ArrowRight', '→')
		.replace('ArrowUp', '↑')
		.replace('ArrowDown', '↓');
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
