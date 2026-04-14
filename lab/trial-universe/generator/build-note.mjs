/**
 * Build a single Constellation note from Wikipedia material.
 *
 * Inputs
 *  - `title`            — article title
 *  - `summary`          — REST summary object (for extract/thumb/coordinates)
 *  - `parsed`           — Action API parse result (for full HTML + links + images)
 *  - `libraryContext`   — { libraryId, folderPath, libraryTitles: Set<string> }
 *  - `noteIdFn`         — function(title) => canonical file basename used across the Universe
 *
 * Output
 *  - `{ filename, content, linksOut }` where `linksOut` is the list of target
 *    Wikipedia titles that appeared in the body (deduplicated).
 *
 * `content` is a full markdown file with YAML frontmatter. Wikilinks are
 * preserved as `{{LINK:Target}}` placeholders — the link-typer pass will
 * resolve them to `[[Note Title|cognitive-type]]` once the full Universe topology
 * is known.
 */

import { htmlToMarkdown } from './html-to-md.mjs';

const now = new Date().toISOString();

function frontmatter(obj) {
	const lines = ['---'];
	for (const [k, v] of Object.entries(obj)) {
		if (v === undefined || v === null) continue;
		if (Array.isArray(v)) {
			if (v.length === 0) continue;
			lines.push(`${k}:`);
			for (const item of v) lines.push(`  - ${yamlScalar(item)}`);
		} else if (typeof v === 'object') {
			lines.push(`${k}:`);
			for (const [kk, vv] of Object.entries(v)) {
				if (vv === undefined || vv === null) continue;
				lines.push(`  ${kk}: ${yamlScalar(vv)}`);
			}
		} else {
			lines.push(`${k}: ${yamlScalar(v)}`);
		}
	}
	lines.push('---');
	return lines.join('\n');
}

function yamlScalar(v) {
	if (typeof v !== 'string') return String(v);
	if (/[:\-#&*!|>'"%@`]/.test(v) || /^\s|\s$/.test(v) || v.includes('\n')) {
		return JSON.stringify(v);
	}
	return v;
}

function pickMaturity() {
	const r = Math.random();
	if (r < 0.01) return 'canonical';
	if (r < 0.11) return 'evergreen';
	if (r < 0.51) return 'sapling';
	return 'seed';
}

function pickStage(maturity) {
	if (maturity === 'canonical') return 'maturity';
	if (maturity === 'evergreen') return 'growth';
	if (maturity === 'sapling') return 'birth';
	return 'spark';
}

/**
 * @param {object} args
 * @param {string} args.title
 * @param {object} args.summary
 * @param {object} args.parsed
 * @param {{ libraryId: string, folderPath: string }} args.libraryContext
 * @param {string[]} [args.heroImageLocalPath]
 * @param {object} [args.heroImageInfo]
 */
export function buildNote({ title, summary, parsed, libraryContext, heroImageLocalPath, heroImageInfo }) {
	const { markdown, links } = htmlToMarkdown(parsed.text);

	const maturity = pickMaturity();
	const stage = pickStage(maturity);
	const tags = deriveTags(summary, parsed);
	const extract = (summary?.extract ?? '').trim();

	const canonicalNormalized = decodeEntities(summary?.titles?.canonical?.replace(/_/g, ' ') ?? '');
	const displayNormalized = decodeEntities(summary?.titles?.display?.replace(/<[^>]+>/g, '') ?? '');
	const aliasCandidates = new Set();
	if (canonicalNormalized && canonicalNormalized !== title) aliasCandidates.add(canonicalNormalized);
	if (displayNormalized && displayNormalized !== title) aliasCandidates.add(displayNormalized);

	const fm = frontmatter({
		title,
		aliases: [...aliasCandidates],
		tags,
		maturity,
		stage,
		source: 'Wikipedia',
		source_url: `https://en.wikipedia.org/wiki/${encodeURIComponent(title.replace(/ /g, '_'))}`,
		license: 'CC BY-SA 4.0',
		attribution: `Content derived from the Wikipedia article "${title}" by its contributors. See source_url for history and revisions.`,
		created: now,
		library: libraryContext.libraryId,
	});

	const parts = [fm, '', `# ${title}`];

	// Optional TL;DR from the REST summary
	if (extract) parts.push('', `> [!abstract] TL;DR`, `> ${extract.replace(/\n/g, ' ')}`);

	// Optional hero image
	if (heroImageLocalPath && heroImageInfo) {
		const alt = heroImageInfo.description?.slice(0, 140) || title;
		const credit = heroImageInfo.artist
			? `${heroImageInfo.artist} — ${heroImageInfo.license}`
			: heroImageInfo.license;
		parts.push('', `![${alt}](${heroImageLocalPath})`);
		parts.push('', `*${credit}*`);
	}

	parts.push('', markdown);

	// Footer with license + source
	parts.push(
		'',
		'---',
		'',
		`*Source: [Wikipedia](${`https://en.wikipedia.org/wiki/${encodeURIComponent(title.replace(/ /g, '_'))}`}) · License: CC BY-SA 4.0*`,
	);

	const filename = sanitizeFilename(title) + '.md';
	return { filename, content: parts.join('\n') + '\n', linksOut: links };
}

function sanitizeFilename(title) {
	return title
		.replace(/[\/\\:*?"<>|]/g, '-')
		.replace(/\s+/g, ' ')
		.trim();
}

function deriveTags(summary, parsed) {
	const tags = new Set();
	// Skip maintenance / bookkeeping categories that are not descriptive
	const NOISE = /(stub|article|unreferenced|commons|wikidata|webarchive|hidden|wikipedia|wikisource|wiktionary|pages (with|using|containing)|CS1|Use dmy|Use mdy|Use british|Use american|Articles (needing|with|containing|lacking|to be|that|having|missing|requiring)|short description|all articles|vague|unclear|peer[- ]reviewed|ISBN|OCLC|DOI|protected|NPOV|cleanup|copy[- ]edit|dates|references|citations|Coordinates|webarchive|from [A-Z][a-z]+ \d{4}|since \d{4})/i;
	for (const c of parsed.categories ?? []) {
		const name = (c['*'] ?? c.category ?? '').replace(/_/g, ' ');
		if (!name) continue;
		if (NOISE.test(name)) continue;
		if (name.length > 40) continue;
		tags.add(tagify(name));
		if (tags.size >= 6) break;
	}
	if (summary?.description && summary.description.length < 40) tags.add(tagify(summary.description));
	return [...tags].slice(0, 6);
}

function decodeEntities(s) {
	return s
		.replace(/&amp;/g, '&')
		.replace(/&lt;/g, '<')
		.replace(/&gt;/g, '>')
		.replace(/&quot;/g, '"')
		.replace(/&#039;/g, "'")
		.replace(/&#x27;/g, "'")
		.replace(/&nbsp;/g, ' ');
}

function tagify(s) {
	return s
		.replace(/[^\w\u0600-\u06FF\u0370-\u03FF -]+/g, '')
		.trim()
		.replace(/\s+/g, '-')
		.toLowerCase();
}
