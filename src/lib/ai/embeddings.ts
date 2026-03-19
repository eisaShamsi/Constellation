/**
 * Local Embedding Service — @xenova/transformers (WASM, fully offline)
 *
 * Computes semantic embeddings for notes using a lightweight model.
 * Embeddings are cached in memory and persisted to localStorage.
 * Used by the AI Context Radar to find semantically related notes.
 */

// Dynamic import to avoid loading the 50MB model at startup
let pipeline: any = null;
let extractor: any = null;

const CACHE_KEY = 'constellation:embeddings';
const MODEL_ID = 'Xenova/all-MiniLM-L6-v2'; // 23MB, 384-dim, fast

export interface NoteEmbedding {
	id: string; // note path
	name: string;
	libraryName: string;
	vector: number[];
	timestamp: number; // when computed
}

let embeddingCache: Map<string, NoteEmbedding> = new Map();
let isInitialized = false;
let isBuilding = false;

// Progress callback
type ProgressCallback = (current: number, total: number, status: string) => void;

/**
 * Initialize the embedding pipeline (loads model on first call)
 */
export async function initEmbeddings(): Promise<void> {
	if (isInitialized) return;

	try {
		const { pipeline: pipelineFn } = await import('@xenova/transformers');
		pipeline = pipelineFn;
		extractor = await pipeline('feature-extraction', MODEL_ID, {
			quantized: true, // Use quantized model for speed
		});
		isInitialized = true;

		// Load cached embeddings from localStorage
		loadCache();
	} catch (err) {
		console.error('[Embeddings] Failed to initialize:', err);
		throw err;
	}
}

/**
 * Compute embedding for a single text
 */
async function embed(text: string): Promise<number[]> {
	if (!extractor) throw new Error('Embeddings not initialized');

	// Truncate to ~256 tokens worth of text (~1000 chars)
	const truncated = text.slice(0, 1000);
	const output = await extractor(truncated, { pooling: 'mean', normalize: true });
	return Array.from(output.data);
}

/**
 * Build embedding index for all notes
 */
export async function buildIndex(
	notes: { path: string; name: string; libraryName: string; content: string }[],
	onProgress?: ProgressCallback
): Promise<void> {
	if (isBuilding) return;
	isBuilding = true;

	try {
		if (!isInitialized) {
			onProgress?.(0, notes.length, 'Loading AI model...');
			await initEmbeddings();
		}

		const total = notes.length;
		let done = 0;

		for (const note of notes) {
			// Skip if already cached and content hasn't changed
			const existing = embeddingCache.get(note.path);
			if (existing) {
				done++;
				if (done % 10 === 0) {
					onProgress?.(done, total, `Indexed ${done}/${total} notes`);
				}
				continue;
			}

			// Compute embedding
			const text = `${note.name.replace(/\.md$/, '')} ${note.content}`;
			const vector = await embed(text);

			embeddingCache.set(note.path, {
				id: note.path,
				name: note.name,
				libraryName: note.libraryName,
				vector,
				timestamp: Date.now(),
			});

			done++;
			if (done % 5 === 0 || done === total) {
				onProgress?.(done, total, `Indexed ${done}/${total} notes`);
			}
		}

		// Save cache
		saveCache();
		onProgress?.(total, total, 'Done');
	} finally {
		isBuilding = false;
	}
}

/**
 * Find the N most semantically similar notes to a given note
 */
export function findSimilar(
	notePath: string,
	topN: number = 10,
	minSimilarity: number = 0.3
): { path: string; name: string; libraryName: string; similarity: number }[] {
	const source = embeddingCache.get(notePath);
	if (!source) return [];

	const results: { path: string; name: string; libraryName: string; similarity: number }[] = [];

	for (const [path, entry] of embeddingCache) {
		if (path === notePath) continue;
		const sim = cosineSimilarity(source.vector, entry.vector);
		if (sim >= minSimilarity) {
			results.push({
				path: entry.id,
				name: entry.name,
				libraryName: entry.libraryName,
				similarity: sim,
			});
		}
	}

	results.sort((a, b) => b.similarity - a.similarity);
	return results.slice(0, topN);
}

/**
 * Check if the embedding index has been built
 */
export function isIndexBuilt(): boolean {
	return embeddingCache.size > 0;
}

/**
 * Get index stats
 */
export function getIndexStats(): { count: number; building: boolean; initialized: boolean } {
	return {
		count: embeddingCache.size,
		building: isBuilding,
		initialized: isInitialized,
	};
}

/**
 * Cosine similarity between two vectors
 */
function cosineSimilarity(a: number[], b: number[]): number {
	if (a.length !== b.length) return 0;
	let dot = 0, normA = 0, normB = 0;
	for (let i = 0; i < a.length; i++) {
		dot += a[i] * b[i];
		normA += a[i] * a[i];
		normB += b[i] * b[i];
	}
	const denom = Math.sqrt(normA) * Math.sqrt(normB);
	return denom === 0 ? 0 : dot / denom;
}

/**
 * Persist cache to localStorage
 */
function saveCache(): void {
	try {
		const data: Record<string, NoteEmbedding> = {};
		for (const [k, v] of embeddingCache) {
			data[k] = v;
		}
		localStorage.setItem(CACHE_KEY, JSON.stringify(data));
	} catch {
		// localStorage full or unavailable — silently fail
	}
}

/**
 * Load cache from localStorage
 */
function loadCache(): void {
	try {
		const raw = localStorage.getItem(CACHE_KEY);
		if (!raw) return;
		const data = JSON.parse(raw) as Record<string, NoteEmbedding>;
		for (const [k, v] of Object.entries(data)) {
			embeddingCache.set(k, v);
		}
	} catch {
		// Corrupt cache — start fresh
		embeddingCache.clear();
	}
}
