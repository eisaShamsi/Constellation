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
 * @returns {{ markdown: string, links: string[] }}
 */
export function htmlToMarkdown(html) {
	const $ = cheerio.load(html, { decodeEntities: true });

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

	return { markdown, links: [...links] };
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

function renderTable($, $table) {
	const rows = [];
	$table.find('tr').each((_, tr) => {
		const cells = $(tr).children('th, td').map((_, c) => {
			const $c = $(c);
			const t = $c.contents().map((_, child) => render($, child, 0)).get().join('').trim().replace(/\n+/g, ' ');
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

function cleanup(md) {
	return md
		.replace(/\[\s*edit\s*\]/g, '')
		.replace(/\[\s*\d+\s*\]/g, '') // citation markers
		.replace(/\n{3,}/g, '\n\n')
		.replace(/[ \t]+\n/g, '\n')
		.trim();
}
