/**
 * Main entry point. Orchestrates the Trial Universe build.
 *
 *   node generator/index.mjs --stage poc      ~20 notes, 1 library, fast smoke test
 *   node generator/index.mjs --stage pilot    ~200 notes, 2 libraries, for user review
 *   node generator/index.mjs --stage full     ~5000 notes, all 12 libraries
 *
 * Build phases (per stage):
 *  1. Resolve topic seeds → {title, summary, parsed} for each seed.
 *  2. Expand: follow 1-2 hops of in-category links to reach the target count.
 *  3. Build each note to disk (with hero image + frontmatter + body).
 *  4. Typed-link second pass: convert {{LINK:Target}} → [[Target|type]].
 *  5. Emit universe.json, libraries.json, library.json, .base files.
 *  6. Report: note count per library, link-type distribution, orphan count.
 */

import { readFile, writeFile, mkdir } from 'node:fs/promises';
import { join, dirname } from 'node:path';
import { fileURLToPath } from 'node:url';
import { parseArgs } from 'node:util';

const __dirname = dirname(fileURLToPath(import.meta.url));
import { fetchSummary, fetchParsed, fetchImageInfo, downloadImage } from './fetch-wikipedia.mjs';
import { buildNote } from './build-note.mjs';
import { typeLinksInFile } from './link-typer.mjs';

const { values: args } = parseArgs({
	options: {
		stage:  { type: 'string', default: 'poc' },
		output: { type: 'string', default: join(__dirname, '..', 'output') },
	},
});

const STAGE_LIMITS = {
	poc:    { librariesMax: 1,  notesPerLibrary: 20,  spreadAcrossCUniverses: false },
	pilot:  { librariesMax: 4,  notesPerLibrary: 25,  spreadAcrossCUniverses: true  },
	full:   { librariesMax: 16, notesPerLibrary: 550, spreadAcrossCUniverses: true, expand: true  },
};

const LIMITS = STAGE_LIMITS[args.stage];
if (!LIMITS) { console.error(`Unknown stage: ${args.stage}`); process.exit(1); }

const configDir = join(__dirname, '..', 'config');
const topology = JSON.parse(await readFile(join(configDir, 'topology.json'), 'utf8'));
// Merge the Arab cUniverse (kept in its own file so the main topology stays readable)
try {
	const arabCU = JSON.parse(await readFile(join(configDir, 'topology-arab.json'), 'utf8'));
	topology.cUniverses.push(arabCU);
} catch { /* optional */ }

const outRoot = join(args.output, topology.universe.name);
await mkdir(outRoot, { recursive: true });

console.log(`\n=== Trial Universe build [stage=${args.stage}] ===`);
console.log(`Output: ${outRoot}`);
console.log(`Max libraries: ${LIMITS.librariesMax}, notes per library: ${LIMITS.notesPerLibrary}\n`);

const allNotes = []; // { title, libraryId, folderName, filename, filepath, linksOut, typedLinkHints, properties }
const contradictionPairs = [];
const libraryFolderIndexes = []; // names of the folder-index / seed notes
const skipLog = [];

// Every seed title across every library in the build — so expansion in library A
// never steals a topic that is an explicit seed in library B.
const GLOBAL_SEED_SET = new Set(
	topology.cUniverses.flatMap(cu => cu.libraries.flatMap(lib => lib.folders.flatMap(f => f.seeds)))
);

// Build a flat list of (cUniverse, library) pairs in the order to visit
const libraryOrder = [];
if (LIMITS.spreadAcrossCUniverses) {
	// Round-robin across cUniverses so the pilot reaches multiple domains
	let idx = 0;
	let added = true;
	while (added && libraryOrder.length < LIMITS.librariesMax) {
		added = false;
		for (const cu of topology.cUniverses) {
			if (idx < cu.libraries.length && libraryOrder.length < LIMITS.librariesMax) {
				libraryOrder.push({ cu, lib: cu.libraries[idx] });
				added = true;
			}
		}
		idx++;
	}
} else {
	for (const cu of topology.cUniverses) {
		for (const lib of cu.libraries) {
			if (libraryOrder.length >= LIMITS.librariesMax) break;
			libraryOrder.push({ cu, lib });
		}
		if (libraryOrder.length >= LIMITS.librariesMax) break;
	}
}

for (const { cu, lib } of libraryOrder) await buildLibrary(cu, lib);

// Typed link second pass
console.log('\n--- Pass 2: typed-link resolution ---');
const universeTitles = new Set(allNotes.map(n => n.title));
const titleToLibrary = new Map(allNotes.map(n => [n.title, n.libraryId]));

const folderIndex = new Map(); // libraryId → Map(folderName → Set<titles>)
for (const n of allNotes) {
	if (!folderIndex.has(n.libraryId)) folderIndex.set(n.libraryId, new Map());
	const lmap = folderIndex.get(n.libraryId);
	if (!lmap.has(n.folderName)) lmap.set(n.folderName, new Set());
	lmap.get(n.folderName).add(n.title);
}

for (const n of allNotes) {
	const siblings = folderIndex.get(n.libraryId).get(n.folderName) ?? new Set();
	await typeLinksInFile({
		notePath: n.filepath,
		noteTitle: n.title,
		folderName: n.folderName,
		universeTitles,
		titleToLibrary,
		libraryId: n.libraryId,
		folderSiblings: siblings,
		contradictionPairs,
		libraryFolderIndexes,
		typedLinkHints: n.typedLinkHints ?? {},
		properties: n.properties ?? {},
	});
}

// Universe metadata
await writeFile(join(outRoot, 'universe.json'), JSON.stringify({
	name: topology.universe.name,
	tagline: topology.universe.tagline,
	created: new Date().toISOString(),
	generator: 'Constellation Trial Universe Generator',
	source: 'Wikipedia / Wikimedia Commons / Wikidata',
	license: 'CC BY-SA 4.0 (content), CC0 (structured data)',
}, null, 2));

// Distribution report
const typeCounts = {};
for (const n of allNotes) {
	const body = await readFile(n.filepath, 'utf8');
	for (const m of body.matchAll(/\[\[[^\]]+?\|([a-z-]+)\]\]/g)) {
		typeCounts[m[1]] = (typeCounts[m[1]] ?? 0) + 1;
	}
}

console.log('\n=== Build complete ===');
console.log(`Notes: ${allNotes.length}   Skipped seeds: ${skipLog.length}`);
if (skipLog.length) {
	await writeFile(join(outRoot, 'skip-log.json'), JSON.stringify(skipLog, null, 2));
	console.log(`(skip details → ${join(outRoot, 'skip-log.json')})`);
}
console.log('Link type distribution:');
const total = Object.values(typeCounts).reduce((a, b) => a + b, 0) || 1;
for (const [t, c] of Object.entries(typeCounts).sort((a, b) => b[1] - a[1])) {
	console.log(`  ${t.padEnd(14)} ${String(c).padStart(4)}  (${((c * 100) / total).toFixed(1)}%)`);
}

// ─── helpers ───

async function buildLibrary(cu, lib) {
	console.log(`\n--- Library: ${lib.name} (${cu.name}) ---`);
	for (const pair of lib.contradictionSeeds ?? []) contradictionPairs.push(pair);

	const libRoot = join(outRoot, 'child-universes', cu.name, 'libraries', lib.name);
	await mkdir(join(libRoot, 'attachments', 'img'), { recursive: true });

	await writeFile(join(libRoot, 'library.json'), JSON.stringify({
		id: lib.id, name: lib.name, cUniverse: cu.id, color: cu.color, createdAt: new Date().toISOString(),
	}, null, 2));

	// The per-library quota is the user-facing target; both seed and expansion
	// phases draw from it.
	const quota = lib.targetNoteCount
		? Math.min(lib.targetNoteCount, LIMITS.notesPerLibrary)
		: LIMITS.notesPerLibrary;
	let libraryNoteCount = 0;
	const libTitleSet = new Set();
	const folderBySeed = new Map(); // seed title → { folder, folderDir }

	// ── Phase 1: explicit seeds ─────────────────────────────────────────────
	for (const folder of lib.folders) {
		if (libraryNoteCount >= quota) break;
		const folderDir = join(libRoot, folder.name);
		await mkdir(folderDir, { recursive: true });
		if (folder.seeds[0]) libraryFolderIndexes.push(folder.seeds[0]);
		const folderQuota = Math.ceil(quota / lib.folders.length);
		let folderCount = 0;

		for (const seed of folder.seeds) {
			if (folderCount >= folderQuota) break;
			if (libraryNoteCount >= quota) break;
			try {
				const note = await buildSeed(seed, cu, lib, folder, libRoot, folderDir);
				if (note) {
					allNotes.push(note);
					libTitleSet.add(note.title);
					folderBySeed.set(note.title, { folder, folderDir });
					folderCount++;
					libraryNoteCount++;
					process.stdout.write('.');
				} else {
					skipLog.push({ seed, library: lib.id, reason: 'disambiguation or 404' });
					process.stdout.write('x');
				}
			} catch (e) {
				skipLog.push({ seed, library: lib.id, reason: e.message });
				console.warn(`\n  ! ${seed}: ${e.message}`);
			}
		}
	}

	// ── Phase 2: expansion ──────────────────────────────────────────────────
	// Walk outbound links of the seed notes. For each new target not already
	// processed anywhere, fetch it and attach to the folder of the seed that
	// first referenced it. Other libraries' seeds are skipped so each topic
	// lives in exactly one place.
	if (LIMITS.expand && libraryNoteCount < quota) {
		process.stdout.write(` [expanding...]`);
		const relatedDir = join(libRoot, cu.wikipediaLang === 'ar' ? 'مواضيع ذات صلة' : 'Related Topics');
		await mkdir(relatedDir, { recursive: true });
		const relatedFolder = { name: cu.wikipediaLang === 'ar' ? 'مواضيع ذات صلة' : 'Related Topics', seeds: [] };

		// Build the expansion queue: titles → first discovering seed
		const queue = new Map();
		for (const n of allNotes) {
			if (n.libraryId !== lib.id) continue;
			for (const outTitle of n.linksOut) {
				if (libTitleSet.has(outTitle)) continue;        // already in this library
				if (GLOBAL_SEED_SET.has(outTitle)) continue;    // owned by another library
				if (queue.has(outTitle)) continue;
				queue.set(outTitle, n.title);
			}
		}

		// Process queue until quota hit, rate-limited by fetcher
		let expandCount = 0;
		for (const [title, sourceTitle] of queue) {
			if (libraryNoteCount >= quota) break;
			const parent = folderBySeed.get(sourceTitle);
			const { folder, folderDir } = parent ?? { folder: relatedFolder, folderDir: relatedDir };
			try {
				const note = await buildSeed(title, cu, lib, folder, libRoot, folderDir);
				if (note) {
					allNotes.push(note);
					libTitleSet.add(note.title);
					libraryNoteCount++;
					expandCount++;
					if (expandCount % 10 === 0) process.stdout.write('+');
				}
			} catch { /* silent: expansion is best-effort */ }
		}
		process.stdout.write(` (+${expandCount} via expansion)`);
	}

	console.log(`\n  ${libraryNoteCount} notes in ${lib.name}`);
}

async function buildSeed(title, cu, lib, folder, libRoot, folderDir) {
	const lang = cu.wikipediaLang ?? 'en';
	const summary = await fetchSummary(title, lang);
	if (!summary || summary.type === 'disambiguation') return null;
	const parsed = await fetchParsed(title, lang);
	if (!parsed) return null;

	let heroImageLocalPath = null;
	let heroImageInfo = null;
	const thumbTitle = summary.thumbnail?.source ? extractFileTitle(summary.thumbnail.source) : null;
	if (thumbTitle) {
		heroImageInfo = await fetchImageInfo(thumbTitle);
		if (heroImageInfo) {
			const absPath = await downloadImage(heroImageInfo, join(libRoot, 'attachments', 'img'));
			if (absPath) {
				heroImageLocalPath = `../attachments/img/${absPath.split(/[\/\\]/).pop()}`;
			}
		}
	}

	const isContradictionTarget = (lib.contradictionSeeds ?? []).some(p => p[0] === title || p[1] === title);
	const built = buildNote({
		title,
		summary,
		parsed,
		libraryContext: { libraryId: lib.id, folderPath: folder.name, cUniverseName: cu.name },
		heroImageLocalPath,
		heroImageInfo,
		lang,
		isContradictionTarget,
	});

	const filepath = join(folderDir, built.filename);
	await writeFile(filepath, built.content);

	// Re-parse to pull properties from the body for the link-typer.
	// (We pass them through from buildNote rather than re-parsing; simpler.)
	return {
		title,
		libraryId: lib.id,
		folderName: folder.name,
		filename: built.filename,
		filepath,
		linksOut: built.linksOut,
		typedLinkHints: built.typedLinkHints,
		properties: extractPropertiesFromFrontmatter(built.content),
	};
}

function extractPropertiesFromFrontmatter(content) {
	const m = content.match(/^---\n([\s\S]*?)\n---/);
	if (!m) return {};
	const body = m[1];
	// Simple YAML-ish parse of array fields we care about for link typing
	const props = {};
	const ARRAY_KEYS = [
		'influenced_by', 'influenced', 'predecessor', 'successor', 'doctoral_advisor',
		'doctoral_students', 'notable_works', 'notable_ideas', 'known_for',
		'main_interests', 'school', 'era', 'field', 'part_of', 'discovered_by',
		'formulated_by', 'developed_by', 'founded_by', 'founder', 'institutions', 'alma_mater',
	];
	for (const key of ARRAY_KEYS) {
		const re = new RegExp(`^${key}:\\s*\\n((?:\\s{2,}-\\s.+\\n)+)`, 'm');
		const mm = body.match(re);
		if (mm) {
			props[key] = mm[1]
				.split('\n')
				.map(l => l.match(/-\s+(?:"([^"]+)"|(\S.*))$/))
				.filter(Boolean)
				.map(mm2 => (mm2[1] ?? mm2[2]).trim());
			continue;
		}
		// scalar form
		const scalar = body.match(new RegExp(`^${key}:\\s*(?:"([^"]+)"|(.+))$`, 'm'));
		if (scalar) props[key] = scalar[1] ?? scalar[2];
	}
	return props;
}

function extractFileTitle(thumbUrl) {
	const m = thumbUrl.match(/\/([^\/]+?)(?:\/\d+px-[^\/]+)?$/);
	return m ? decodeURIComponent(m[1]) : null;
}
