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
export function buildNote({ title, summary, parsed, libraryContext, heroImageLocalPath, heroImageInfo, lang = 'en', isContradictionTarget = false }) {
	const { markdown, links, properties, typedLinkHints } = htmlToMarkdown(parsed.text);

	const maturity = pickMaturity();
	const stage = pickStage(maturity);
	const tags = deriveTags(summary, parsed);
	const extract = (summary?.extract ?? '').trim();

	// Curate which infobox properties we promote into the YAML frontmatter.
	// Keeps the frontmatter useful for Dataview queries and .base files
	// without dumping 40+ raw fields.
	const curatedProps = curateProperties(properties);

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
		library: libraryContext.libraryId,
		cUniverse: libraryContext.cUniverseName,
		folder: libraryContext.folderPath,
		// Curated properties from Wikipedia infobox
		...curatedProps,
		source: lang === 'ar' ? 'ويكيبيديا' : 'Wikipedia',
		source_url: `https://${lang}.wikipedia.org/wiki/${encodeURIComponent(title.replace(/ /g, '_'))}`,
		license: 'CC BY-SA 4.0',
		attribution: lang === 'ar'
			? `المحتوى مشتقّ من مقالة ويكيبيديا "${title}" بإسهام كتّابها. يُرجع إلى source_url للاطلاع على التاريخ والمراجعات.`
			: `Content derived from the Wikipedia article "${title}" by its contributors. See source_url for history and revisions.`,
		created: now,
	});

	const parts = [fm, '', `# ${title}`];

	// Optional TL;DR from the REST summary
	if (extract) {
		const label = lang === 'ar' ? 'ملخّص' : 'TL;DR';
		parts.push('', `> [!abstract] ${label}`, `> ${extract.replace(/\n/g, ' ')}`);
	}

	// Warning callout for notes that appear in a curated contradiction pair
	if (isContradictionTarget) {
		const heading = lang === 'ar' ? 'وجهة نظر متنازعة' : 'Disputed perspective';
		const body = lang === 'ar'
			? 'هذه الملاحظة جزء من جدل تاريخي. انظر الروابط المُعلَّمة بـ `contradicts` للاطلاع على المواقف المقابلة.'
			: 'This note participates in a historic debate. See `contradicts` links for the opposing position.';
		parts.push('', `> [!warning] ${heading}`, `> ${body}`);
	}

	// Example callout listing notable works / known-for items (if present in infobox)
	const exampleItems = curatedProps.notable_works ?? curatedProps.notable_ideas ?? curatedProps.known_for;
	if (Array.isArray(exampleItems) && exampleItems.length >= 2) {
		const heading = lang === 'ar' ? 'أمثلة بارزة' : 'Notable examples';
		parts.push('', `> [!example] ${heading}`);
		for (const item of exampleItems.slice(0, 5)) {
			parts.push(`> - ${item}`);
		}
	}

	// Info callout summarising key infobox facts (era/school/field/nationality)
	const factKeys = ['era', 'school', 'field', 'nationality', 'occupation', 'born', 'died'];
	const facts = factKeys.filter(k => curatedProps[k]).map(k => `> - **${displayKey(k, lang)}**: ${formatFact(curatedProps[k])}`);
	if (facts.length >= 2) {
		const heading = lang === 'ar' ? 'حقائق رئيسية' : 'Key facts';
		parts.push('', `> [!info] ${heading}`, ...facts);
	}

	// Optional hero image
	if (heroImageLocalPath && heroImageInfo) {
		const artistClean = cleanCreditField(heroImageInfo.artist);
		const credit = artistClean
			? `${artistClean} — ${heroImageInfo.license}`
			: heroImageInfo.license;
		parts.push('', `![${title}](${heroImageLocalPath})`);
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
	return { filename, content: parts.join('\n') + '\n', linksOut: links, typedLinkHints };
}

/**
 * Promote a whitelist of infobox fields to the YAML frontmatter.
 * Rename some Wikipedia-ish keys to Constellation-friendly names.
 */
function curateProperties(raw) {
	const out = {};
	const MAP = {
		born: 'born',
		birth_date: 'born',
		died: 'died',
		death_date: 'died',
		nationality: 'nationality',
		citizenship: 'nationality',
		ethnicity: 'ethnicity',
		field: 'field',
		fields: 'field',
		main_interests: 'main_interests',
		school: 'school',
		school_tradition: 'school',
		tradition: 'school',
		era: 'era',
		region: 'region',
		institutions: 'institutions',
		alma_mater: 'alma_mater',
		education: 'alma_mater',
		doctoral_advisor: 'doctoral_advisor',
		doctoral_students: 'doctoral_students',
		influences: 'influenced_by',
		influenced_by: 'influenced_by',
		influenced: 'influenced',
		notable_works: 'notable_works',
		notable_ideas: 'notable_ideas',
		known_for: 'known_for',
		awards: 'awards',
		occupation: 'occupation',
		genre: 'genre',
		language: 'language',
		country: 'country',
		country_of_origin: 'country',
		publisher: 'publisher',
		publication_date: 'published',
		published: 'published',
		written: 'published',
		author: 'author',
		authors: 'author',
		director: 'director',
		writer: 'writer',
		writers: 'writer',
		composer: 'composer',
		discovered_by: 'discovered_by',
		formulated_by: 'formulated_by',
		developed_by: 'developed_by',
		founded_by: 'founded_by',
		founder: 'founder',
		predecessor: 'predecessor',
		successor: 'successor',
		part_of: 'part_of',
	};
	for (const [rawKey, val] of Object.entries(raw)) {
		const mapped = MAP[rawKey];
		if (!mapped) continue;
		if (out[mapped]) continue; // first hit wins (avoid duplicates from merged infoboxes)
		if (Array.isArray(val)) {
			out[mapped] = val.slice(0, 8);
		} else if (typeof val === 'string' && val.length > 0) {
			out[mapped] = val.length > 160 ? val.slice(0, 160) + '…' : val;
		}
	}
	return out;
}

function sanitizeFilename(title) {
	return title
		.replace(/[\/\\:*?"<>|]/g, '-')
		.replace(/\s+/g, ' ')
		.trim();
}

function deriveTags(summary, parsed) {
	const tags = new Set();
	// Skip maintenance / bookkeeping categories (English + Arabic) — these
	// describe Wikipedia administrivia, not the article's topic.
	const NOISE = new RegExp(
		// English
		'(stub|article|unreferenced|commons|wikidata|webarchive|hidden|wikipedia|wikisource|wiktionary|pages (with|using|containing)|CS1|Use dmy|Use mdy|Use british|Use american|Articles (needing|with|containing|lacking|to be|that|having|missing|requiring)|short description|all articles|vague|unclear|peer[- ]reviewed|ISBN|OCLC|DOI|protected|NPOV|cleanup|copy[- ]edit|dates|references|citations|Coordinates|webarchive|from [A-Z][a-z]+ \\d{4}|since \\d{4})|'
		// Arabic
		+ '(قالب|أرشيف|صفحات|مقالات|مصادر|بحاجة|بوابة|أخطاء|الاستشهاد|تحتاج|تحوي|تستعمل|ترجمة|تحقق|ترقية|ملحة|تعديل|تصنيف-|قيد-الإنشاء|قصاصات|ملاحظة-صيانة|محتوى|بيانات-ويكي|ويكي-بيانات|محمية|تحتاج-مصدر|محايدة|غامضة)',
		'i'
	);
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

function displayKey(k, lang) {
	const EN = {
		era: 'Era', school: 'School', field: 'Field', nationality: 'Nationality',
		occupation: 'Occupation', born: 'Born', died: 'Died',
	};
	const AR = {
		era: 'الحقبة', school: 'المدرسة', field: 'المجال', nationality: 'الجنسية',
		occupation: 'المهنة', born: 'المولد', died: 'الوفاة',
	};
	return (lang === 'ar' ? AR : EN)[k] ?? k;
}

function formatFact(v) {
	if (Array.isArray(v)) return v.slice(0, 3).join(', ');
	return String(v);
}

function cleanCreditField(s) {
	if (!s) return '';
	return decodeEntities(s)
		.replace(/\[\d+\]/g, '')          // strip citation markers
		.replace(/\n+/g, ' ')
		.replace(/\s{2,}/g, ' ')
		.trim()
		.slice(0, 200);
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
		.replace(/[^\p{L}\p{N} -]+/gu, '')
		.trim()
		.replace(/\s+/g, '-')
		// toLowerCase only affects Latin-script; Arabic/CJK pass through unchanged
		.toLowerCase();
}
