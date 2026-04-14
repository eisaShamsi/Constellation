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
 * @param {string} args.notePath                — absolute path of the note file
 * @param {string} args.noteTitle
 * @param {string} args.folderName              — containing folder name
 * @param {Set<string>} args.universeTitles     — every note title in the Universe
 * @param {Map<string, string>} args.titleToLibrary  — target title → libraryId
 * @param {string} args.libraryId
 * @param {Set<string>} args.folderSiblings     — titles in the same folder
 * @param {Array<[string,string]>} args.contradictionPairs
 * @param {Array<string>} args.libraryFolderIndexes — names of folder-index notes
 */
export async function typeLinksInFile(args) {
	const {
		notePath, noteTitle, folderName, universeTitles, titleToLibrary,
		libraryId, folderSiblings, contradictionPairs, libraryFolderIndexes,
	} = args;

	const raw = await readFile(notePath, 'utf8');
	const contradictSet = new Set(
		contradictionPairs
			.flatMap(([a, b]) => a === noteTitle ? [b] : b === noteTitle ? [a] : [])
	);

	const out = raw.replace(/\{\{LINK:([^}|]+)(?:\|([^}]+))?\}\}/g, (match, rawTarget, displayText) => {
		const target = rawTarget.trim();
		if (!universeTitles.has(target)) {
			// dangling link — keep the display text only
			return displayText ?? target;
		}

		const type = chooseType({
			source: noteTitle, target, folderName, folderSiblings,
			libraryId, targetLibrary: titleToLibrary.get(target),
			contradictSet, libraryFolderIndexes,
			surroundingText: surrounding(raw, match),
		});

		const display = displayText && displayText !== target ? displayText : null;
		return display ? `[[${target}|${display}|${type}]]` : `[[${target}|${type}]]`;
	});

	if (out !== raw) await writeFile(notePath, out);
}

function surrounding(raw, match) {
	const i = raw.indexOf(match);
	return raw.slice(Math.max(0, i - 160), Math.min(raw.length, i + match.length + 160));
}

function chooseType({
	source, target, folderName, folderSiblings, libraryId,
	targetLibrary, contradictSet, libraryFolderIndexes, surroundingText,
}) {
	// 1. Curated contradictions always win
	if (contradictSet.has(target)) return 'contradicts';

	// 2. Part-of: link into the enclosing folder's index note, or from a
	//    specific instance to a broader category in the same library.
	if (libraryFolderIndexes.includes(target)) return 'part-of';

	// 3. Causal language in surrounding text
	if (CAUSAL_HINTS.test(surroundingText)) return 'causes';

	// 4. Derivation language
	if (DERIVE_HINTS.test(surroundingText)) return 'derives-from';

	// 5. Exemplifies: target is a broad category and source is a specific instance
	//    (heuristic: target's name appears in source's folder name or tags)
	if (folderName && target.toLowerCase().includes(folderName.toLowerCase().split(' ')[0])) {
		return 'exemplifies';
	}

	// 6. Generalizes: reverse — source title contains target (source is broader)
	if (source.toLowerCase().includes(target.toLowerCase()) && source !== target) {
		return 'generalizes';
	}

	// 7. Supports: same-folder siblings
	if (folderSiblings.has(target)) return 'supports';

	// 8. Default across libraries → derives-from; same library → supports
	return targetLibrary && targetLibrary !== libraryId ? 'derives-from' : 'supports';
}
