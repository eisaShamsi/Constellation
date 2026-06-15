/**
 * Convert parsed Wikipedia HTML into Constellation-flavoured markdown.
 *
 * Responsibilities:
 *  - Strip MediaWiki infoboxes, references, navboxes, edit links, styles.
 *  - Keep headings (demoted one level so note title stays H1).
 *  - Convert <a href="/wiki/TARGET"> to [[TARGET]] placeholders.
 *  - Preserve tables as GFM tables.
 *  - Extract a list of linked Wikipedia titles for later typed-link assignment.
 *  - Keep inline cite markers as [*] so readers know claims are referenced.
 */

import * as cheerio from 'cheerio';

/**
 * @param {string} html
 * @returns {{ markdown: string, links: string[], properties: Record<string,string|string[]>, infobox: Record<string,string[]> }}
 */
export function htmlToMarkdown(html) {
	const $ = cheerio.load(html, { decodeEntities: true });

	// Harvest infobox BEFORE stripping — it's our main source of structured properties.
	const { properties, infobox } = extractInfoboxProperties($);

	// Remove elements we never want to carry over
	$('.mw-editsection, .reference, .reflist, .navbox, .navbox-inner, .navbox-group, .vertical-navbox, .metadata, .hatnote, .sistersitebox, .infobox, .infobox-data, .thumb, .shortdescription, .box-More_citations_needed, .sidebar, .side-box, .plainlist, .nowraplinks, .mw-collapsible, .collapsible, table.ambox, table.wikitable.sidebar, style, script, sup.reference, sup.noprint, .noprint, .mw-selflink').remove();
	// Any table that is itself a sidebar/navbox variant (sometimes just raw <table class="sidebar">)
	$('table').each((_, el) => {
		const c = $(el).attr('class') ?? '';
		if (/\b(sidebar|navbox|infobox|vertical-navbox|metadata|plainlist|navigation|toc)\b/i.test(c)) $(el).remove();
		// Tiny 1-column "Part of a series on" tables
		const rows = $(el).find('tr').length;
		const cols = $(el).find('tr').first().children('th, td').length;
		if (rows >= 4 && cols === 1) $(el).remove();
	});
	$('.mw-empty-elt').remove();
	$('#References, #External_links, #See_also, #Further_reading, #Notes, #Bibliography').each((_, el) => {
		const h = $(el).closest('h1, h2, h3, h4, h5, h6');
		if (h.length) {
			let sib = h[0].next;
			h.remove();
			while (sib && !/^h[1-6]$/i.test(sib.name)) {
				const next = sib.next;
				$(sib).remove();
				sib = next;
			}
		}
	});

	// Collect inline typed-link hints from the body (before the <a> replacement erases markers)
	const typedLinkHints = collectTypedLinkHints($);

	const links = new Set();

	// Rewrite anchors to placeholders
	$('a').each((_, el) => {
		const $el = $(el);
		const href = $el.attr('href') ?? '';
		const m = href.match(/^\/wiki\/([^#?]+)/);
		if (m) {
			const target = decodeURIComponent(m[1]).replace(/_/g, ' ');
			// Skip non-article namespaces
			if (/^(File|Image|Category|Template|Help|Portal|Special|Talk|User|Wikipedia):/.test(target)) {
				$el.replaceWith($el.text());
				return;
			}
			links.add(target);
			const text = $el.text();
			$el.replaceWith(text === target ? `{{LINK:${target}}}` : `{{LINK:${target}|${text}}}`);
		} else {
			$el.replaceWith($el.text());
		}
	});

	// Walk the DOM and emit markdown
	const out = [];
	$('body').contents().each((_, node) => out.push(render($, node, 0)));
	const mdRaw = out.join('');
	const markdown = cleanup(mdRaw);

	return { markdown, links: [...links], properties, infobox, typedLinkHints };
}

/**
 * Extract structured properties from the Wikipedia infobox.
 * We walk the `<table class="infobox">` rows and harvest:
 *   - Key/value pairs keyed by a normalized field name
 *   - Lists of linked articles (preserved as titles for typed links)
 *
 * Common fields this picks up:
 *   Born, Died, Nationality, Field, Fields, Institutions, Alma mater,
 *   Known for, Notable works, Notable students, Influences, Influenced,
 *   Predecessor, Successor, Era, School, Tradition, Region, Main interests,
 *   Occupation, Awards, Discovered by, Formulated by, Developed by, Part of,
 *   Genre, Director, Author, Language, Country, Publisher, Published.
 */
function extractInfoboxProperties($) {
	const props = {};
	const infobox = {};
	$('table.infobox').each((_, tbl) => {
		// Remove things that would pollute our text extraction
		const $tbl = $(tbl);
		$tbl.find('style, script, .mw-empty-elt, sup.reference, .reference').remove();
		$tbl.find('tr').each((_, tr) => {
			const $th = $(tr).find('th').first();
			const $td = $(tr).find('td').first();
			if (!$th.length || !$td.length) return;
			const key = normalizeKey($th.text());
			if (!key) return;
			const linkTitles = [];
			$td.find('a').each((_, a) => {
				const href = $(a).attr('href') ?? '';
				const m = href.match(/^\/wiki\/([^#?]+)/);
				if (m) {
					const t = decodeURIComponent(m[1]).replace(/_/g, ' ');
					if (!/^(File|Image|Category|Template|Help|Portal|Special|Talk|User|Wikipedia):/.test(t)) {
						linkTitles.push(t);
					}
				}
			});
			// When a cell has multiple <li> or <br>-separated items, capture each piece
			const pieces = [];
			if ($td.find('li').length > 0) {
				$td.find('li').each((_, li) => {
					const t = cleanInfoboxValue($(li).text());
					if (t) pieces.push(t);
				});
			} else {
				const html = $td.html() ?? '';
				// split on <br> so "(date)(age)place" stays readable
				for (const raw of html.split(/<br\s*\/?>/i)) {
					const $$ = cheerio.load(`<x>${raw}</x>`);
					const t = cleanInfoboxValue($$('x').text());
					if (t) pieces.push(t);
				}
			}
			const text = pieces.join(' · ');
			if (!text) return;
			// For array-worthy fields, keep the piece list; for scalar fields keep the joined text
			const isListField = /^(school|institutions|alma_mater|era|main_interests|notable_ideas|notable_works|known_for|influences|influenced|doctoral_advisor|doctoral_students|part_of|discovered_by|formulated_by|developed_by|founded_by|founder|predecessor|successor|genre|awards|occupation|writers|authors|composer|director|cast|language|country)$/.test(key);
			if (isListField && pieces.length > 1) {
				// Harvest links into array values; fall back to text pieces
				const arr = linkTitles.length ? linkTitles : pieces;
				props[key] = [...new Set([...(props[key] ?? []), ...arr])].slice(0, 10);
				if (!(key in infobox)) infobox[key] = [];
				infobox[key].push(text);
				return;
			}
			if (!(key in infobox)) infobox[key] = [];
			infobox[key].push(text);
			// Flatten simple scalar values; keep arrays for link-rich fields
			if (linkTitles.length >= 1 && /(influence|predecessor|successor|student|doctoral|advisor|school|field|institution|notable|known|main_interest|part_of|formulated_by|discovered_by|developed_by|founder|founded_by|director|author|writer|cast)/i.test(key)) {
				props[key] = [...new Set([...(props[key] ?? []), ...linkTitles])];
			} else if (!props[key]) {
				props[key] = text;
			}
		});
	});
	return { properties: props, infobox };
}

function normalizeKey(s) {
	return s
		.toLowerCase()
		.replace(/[^\w\u0600-\u06FF\u0370-\u03FF ]+/g, '')
		.trim()
		.replace(/\s+/g, '_');
}

function cleanInfoboxValue(s) {
	return s
		.replace(/\.mw-[\w-]+[\s\S]*?\{[^}]*\}/g, '')   // stray CSS blocks
		.replace(/\.mw-parser-output[^\n]*/g, '')       // stray selectors
		.replace(/@media[^{]*\{[^}]*\}/g, '')
		.replace(/\[\d+\]/g, '')                        // citation markers
		.replace(/[\u200e\u200f]/g, '')                 // bidi control chars
		.replace(/\((?:aged\s*\d+|\d{4}-\d{2}-\d{2})\)/g, m => ` ${m} `)  // space around parens
		.replace(/\s+/g, ' ')
		.trim()
		.slice(0, 200);
}

/**
 * Scan the <body> for phrases that surround links — used by the typed-link
 * second pass to boost non-supports types. Returns a map target → type-vote.
 */
function collectTypedLinkHints($) {
	const hints = new Map(); // target → { causes, derives, contradicts, generalizes, exemplifies }
	const PATTERNS = [
		{ re: /(led to|caused|gave rise to|resulted in|brought about|triggered|produced)\s+(?:the\s+)?/i, type: 'causes' },
		{ re: /(based on|building on|derived from|influenced by|inspired by|following)/i, type: 'derives-from' },
		{ re: /(contrary to|opposed|rejected by|disputed by|refuted by|against)/i, type: 'contradicts' },
		{ re: /(such as|for example|for instance|including)/i, type: 'exemplifies' },
	];
	$('a').each((_, a) => {
		const href = $(a).attr('href') ?? '';
		const m = href.match(/^\/wiki\/([^#?]+)/);
		if (!m) return;
		const target = decodeURIComponent(m[1]).replace(/_/g, ' ');
		if (/^(File|Image|Category|Template|Help|Portal|Special|Talk|User|Wikipedia):/.test(target)) return;
		const parent = $(a).parent();
		const before = parent.text().split($(a).text())[0]?.slice(-120) ?? '';
		for (const p of PATTERNS) {
			if (p.re.test(before)) {
				if (!hints.has(target)) hints.set(target, {});
				hints.get(target)[p.type] = (hints.get(target)[p.type] ?? 0) + 1;
			}
		}
	});
	// Convert Map → plain object for JSON-serialisation / passing
	const out = {};
	for (const [k, v] of hints.entries()) out[k] = v;
	return out;
}

function render($, node, depth) {
	if (node.type === 'text') return node.data;
	if (node.type !== 'tag') return '';
	const $el = $(node);
	const tag = node.name;
	const inner = () => $el.contents().map((_, c) => render($, c, depth)).get().join('');

	switch (tag) {
		case 'p':      return `\n\n${inner().trim()}\n\n`;
		case 'br':     return '\n';
		case 'h1':     return `\n\n## ${inner().trim()}\n\n`;
		case 'h2':     return `\n\n## ${inner().trim()}\n\n`;
		case 'h3':     return `\n\n### ${inner().trim()}\n\n`;
		case 'h4':     return `\n\n#### ${inner().trim()}\n\n`;
		case 'h5':     return `\n\n##### ${inner().trim()}\n\n`;
		case 'h6':     return `\n\n###### ${inner().trim()}\n\n`;
		case 'strong': case 'b': return `**${inner()}**`;
		case 'em': case 'i':     return `*${inner()}*`;
		case 'code':   return `\`${inner()}\``;
		case 'pre':    return `\n\n\`\`\`\n${$el.text()}\n\`\`\`\n\n`;
		case 'blockquote': return `\n\n> ${inner().trim().replace(/\n/g, '\n> ')}\n\n`;
		case 'ul':     return renderList($, $el, depth, '- ');
		case 'ol':     return renderList($, $el, depth, '1. ');
		case 'li':     return inner();
		case 'table':  return renderTable($, $el);
		case 'figure': return ''; // images handled separately via hero
		case 'img':    return '';
		case 'dl': case 'dd': case 'dt': case 'span': case 'div': case 'section': return inner();
		default:       return inner();
	}
}

function renderList($, $ul, depth, bullet) {
	const items = $ul.children('li').map((_, li) => {
		const $li = $(li);
		const text = $li.contents().map((_, c) => render($, c, depth + 1)).get().join('').trim();
		return `${' '.repeat(depth * 2)}${bullet}${text}`;
	}).get();
	return `\n\n${items.join('\n')}\n\n`;
}

// Defensive backstop: no legitimate Wikipedia table approaches this many rows.
// With the two fixes below the explosion can't occur (output is linear in the
// real table size); this is a belt-and-suspenders guard against any other
// pathological input.
const MAX_TABLE_ROWS = 2000;

function renderTable($, $table) {
	const rows = [];
	// FIX (Bug A): `.find('tr')` is a DESCENDANT selector — for a table nested
	// N deep (Wikipedia {{clade}} phylogeny trees nest <table> 10-15 levels),
	// the OUTER table grabbed every <tr> in the entire subtree. Combined with
	// the per-cell re-render below, the same rows were emitted combinatorially
	// (Spirochete.md exploded to 122.9 MB: `| --- | --- |` × 1,078,846). Restrict
	// to rows whose NEAREST <table> ancestor IS this table → each table renders
	// its own rows exactly once.
	const self = $table.get(0);
	$table.find('tr').filter((_, tr) => $(tr).closest('table').get(0) === self).each((_, tr) => {
		if (rows.length >= MAX_TABLE_ROWS) return false; // defensive hard stop
		const cells = $(tr).children('th, td').map((_, c) => {
			const $c = $(c);
			const t = $c.contents().map((_, child) => renderCell($, child)).get().join('').trim().replace(/\n+/g, ' ');
			return t || ' ';
		}).get();
		if (cells.length) rows.push(cells);
	});
	if (rows.length < 2) return '';
	const header = rows[0];
	const sep = header.map(() => '---');
	const body = rows.slice(1);
	const lines = [
		`| ${header.join(' | ')} |`,
		`| ${sep.join(' | ')} |`,
		...body.map(r => `| ${r.join(' | ')} |`),
	];
	return `\n\n${lines.join('\n')}\n\n`;
}

// Render a single table-cell child. FIX (Bug B): a nested <table> inside a cell
// (the clade-tree pattern) is flattened to its plain text — GFM has no nested
// tables, and re-rendering nested tables as grids is what fed the combinatorial
// blow-up. Everything else renders normally.
function renderCell($, child) {
	if (child.type === 'tag' && child.name === 'table') {
		return $(child).text().replace(/\s+/g, ' ').trim();
	}
	return render($, child, 0);
}

function cleanup(md) {
	return md
		.replace(/\[\s*edit\s*\]/g, '')
		.replace(/\[\s*\d+\s*\]/g, '') // citation markers
		.replace(/\n{3,}/g, '\n\n')
		.replace(/[ \t]+\n/g, '\n')
		.trim();
}
