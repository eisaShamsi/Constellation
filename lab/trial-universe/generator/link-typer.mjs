/**
 * Second-pass link typing. After all notes are built, walk each note's body
 * and convert `{{LINK:Target}}` placeholders into typed Constellation wikilinks
 * (`[[Target|cognitive-type]]`). Heuristics are structural — no AI involved.
 */

import { readFile, writeFile } from 'node:fs/promises';

const CAUSAL_HINTS = /(led to|caused|gave rise to|resulted in|brought about|triggered|produced)/i;
const DERIVE_HINTS = /(based on|building on|derived from|influenced by|inspired by|following|after)/i;

/**
 * @param {object} args
 * @param {string} args.notePath
 * @param {string} args.noteTitle
 * @param {string} args.folderName
 * @param {Set<string>} args.universeTitles
 * @param {Map<string, string>} args.titleToLibrary
 * @param {string} args.libraryId
 * @param {Set<string>} args.folderSiblings
 * @param {Array<[string,string]>} args.contradictionPairs
 * @param {Array<string>} args.libraryFolderIndexes
 * @param {Record<string, Record<string, number>>} [args.typedLinkHints]
 *        target → { causes, derives-from, contradicts, exemplifies } vote counts
 *        (captured from surrounding text during HTML parsing).
 * @param {object} [args.properties]
 *        Curated frontmatter for this note. Used to extract infobox-derived
 *        typed-link hints (e.g. `influenced_by` → derives-from).
 */
export async function typeLinksInFile(args) {
	const {
		notePath, noteTitle, folderName, universeTitles, titleToLibrary,
		libraryId, folderSiblings, contradictionPairs, libraryFolderIndexes,
		typedLinkHints = {}, properties = {},
	} = args;

	const raw = await readFile(notePath, 'utf8');
	const contradictSet = new Set(
		contradictionPairs
			.flatMap(([a, b]) => a === noteTitle ? [b] : b === noteTitle ? [a] : [])
	);

	// Infobox-derived typed-link signals override in-body hints — these are the
	// highest-confidence signals we have.
	const infoboxTypes = buildInfoboxTypeMap(properties);

	const out = raw.replace(/\{\{LINK:([^}|]+)(?:\|([^}]+))?\}\}/g, (match, rawTarget, displayText) => {
		const target = rawTarget.trim();
		if (!universeTitles.has(target)) {
			return displayText ?? target;
		}

		const type = chooseType({
			source: noteTitle, target, folderName, folderSiblings,
			libraryId, targetLibrary: titleToLibrary.get(target),
			contradictSet, libraryFolderIndexes,
			surroundingText: surrounding(raw, match),
			inBodyHint: typedLinkHints[target],
			infoboxHint: infoboxTypes[target],
		});

		const display = displayText && displayText !== target ? displayText : null;
		return display ? `[[${target}|${display}|${type}]]` : `[[${target}|${type}]]`;
	});

	if (out !== raw) await writeFile(notePath, out);
}

/**
 * Map target title → hinted cognitive type based on which infobox field it came from.
 *
 *   influenced_by / predecessor / doctoral_advisor → derives-from
 *   influenced / successor / doctoral_students     → causes  (reverse derivation)
 *   school / era / field / part_of                 → part-of
 *   notable_works / notable_ideas / known_for      → exemplifies
 *   discovered_by / formulated_by / founded_by     → derives-from
 */
function buildInfoboxTypeMap(props) {
	const m = {};
	const mark = (targets, type) => {
		for (const t of targets ?? []) if (typeof t === 'string') m[t] = type;
	};
	mark(props.influenced_by, 'derives-from');
	mark(props.predecessor, 'derives-from');
	mark(props.doctoral_advisor, 'derives-from');
	mark(props.discovered_by, 'derives-from');
	mark(props.formulated_by, 'derives-from');
	mark(props.developed_by, 'derives-from');
	mark(props.founded_by, 'derives-from');
	mark(props.founder, 'derives-from');
	mark(props.influenced, 'causes');
	mark(props.successor, 'causes');
	mark(props.doctoral_students, 'causes');
	mark(props.school, 'part-of');
	mark(props.era, 'part-of');
	mark(props.field, 'part-of');
	mark(props.part_of, 'part-of');
	mark(props.notable_works, 'exemplifies');
	mark(props.notable_ideas, 'exemplifies');
	mark(props.known_for, 'exemplifies');
	mark(props.main_interests, 'part-of');
	return m;
}

function surrounding(raw, match) {
	const i = raw.indexOf(match);
	return raw.slice(Math.max(0, i - 160), Math.min(raw.length, i + match.length + 160));
}

function chooseType({
	source, target, folderName, folderSiblings, libraryId,
	targetLibrary, contradictSet, libraryFolderIndexes, surroundingText,
	inBodyHint, infoboxHint,
}) {
	// 1. Curated contradictions always win
	if (contradictSet.has(target)) return 'contradicts';

	// 2. Infobox-derived typed links (highest confidence after curated)
	if (infoboxHint) return infoboxHint;

	// 3. In-body typed-link hints (from surrounding-text phrases)
	if (inBodyHint) {
		const best = Object.entries(inBodyHint).sort((a, b) => b[1] - a[1])[0];
		if (best) return best[0];
	}

	// 4. Part-of: link into the enclosing folder's index note
	if (libraryFolderIndexes.includes(target)) return 'part-of';

	// 5. Causal language in surrounding text (fallback)
	if (CAUSAL_HINTS.test(surroundingText)) return 'causes';

	// 6. Derivation language
	if (DERIVE_HINTS.test(surroundingText)) return 'derives-from';

	// 7. Exemplifies: target is a broad category (heuristic: target contains folder word)
	if (folderName && target.toLowerCase().includes(folderName.toLowerCase().split(' ')[0])) {
		return 'exemplifies';
	}

	// 8. Generalizes: reverse — source title contains target (source is broader)
	if (source.toLowerCase().includes(target.toLowerCase()) && source !== target) {
		return 'generalizes';
	}

	// 9. Supports: same-folder siblings
	if (folderSiblings.has(target)) return 'supports';

	// 10. Default: across libraries → derives-from; same library → supports
	return targetLibrary && targetLibrary !== libraryId ? 'derives-from' : 'supports';
}
