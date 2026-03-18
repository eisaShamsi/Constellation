/**
 * GraphMind — Phase 2: Semantic Embedding Engine
 *
 * Uses @xenova/transformers (WASM, fully offline) to compute
 * sentence embeddings for notes, then finds semantic links
 * via cosine similarity.
 *
 * Model: Xenova/all-MiniLM-L6-v2 (~23MB, multilingual-friendly)
 * All computation is local — nothing leaves the user's machine.
 */

import { pipeline, type Pipeline } from '@xenova/transformers';

export interface SemanticLink {
	source: string; // node id
	target: string; // node id
	similarity: number; // 0-1
}

export interface EmbeddingResult {
	id: string;
	embedding: Float32Array;
}

export type EmbeddingProgress = {
	stage: 'loading-model' | 'embedding' | 'computing-links' | 'done';
	current: number;
	total: number;
};

let embedder: Pipeline | null = null;
let modelLoading = false;

/** Load the sentence embedding model (cached after first download) */
async function getEmbedder(): Promise<Pipeline> {
	if (embedder) return embedder;
	if (modelLoading) {
		// Wait for concurrent load
		while (modelLoading) {
			await new Promise((r) => setTimeout(r, 100));
		}
		return embedder!;
	}

	modelLoading = true;
	try {
		embedder = await pipeline('feature-extraction', 'Xenova/all-MiniLM-L6-v2', {
			quantized: true, // Use quantized model for speed (~23MB vs ~90MB)
		});
		return embedder;
	} finally {
		modelLoading = false;
	}
}

/** Extract a summary from note content for embedding (title + first ~300 chars) */
function extractSummary(name: string, content: string): string {
	// Strip frontmatter
	let text = content;
	if (text.startsWith('---')) {
		const endIdx = text.indexOf('---', 3);
		if (endIdx > 0) text = text.substring(endIdx + 3);
	}
	// Strip markdown syntax
	text = text
		.replace(/!\[.*?\]\(.*?\)/g, '') // images
		.replace(/\[([^\]]*)\]\(.*?\)/g, '$1') // links
		.replace(/#{1,6}\s/g, '') // headings
		.replace(/[*_~`>]/g, '') // formatting
		.replace(/\[\[([^\]|]+)(?:\|[^\]]+)?\]\]/g, '$1') // wikilinks
		.trim();

	// Combine name + first ~300 chars of content
	const summary = `${name}. ${text.substring(0, 300)}`;
	return summary;
}

/** Compute cosine similarity between two vectors */
function cosineSimilarity(a: Float32Array, b: Float32Array): number {
	let dot = 0, normA = 0, normB = 0;
	for (let i = 0; i < a.length; i++) {
		dot += a[i] * b[i];
		normA += a[i] * a[i];
		normB += b[i] * b[i];
	}
	return dot / (Math.sqrt(normA) * Math.sqrt(normB));
}

/**
 * Compute embeddings for a batch of notes.
 * Returns embeddings keyed by note id.
 */
export async function computeEmbeddings(
	notes: { id: string; name: string; content: string }[],
	onProgress?: (p: EmbeddingProgress) => void
): Promise<EmbeddingResult[]> {
	onProgress?.({ stage: 'loading-model', current: 0, total: notes.length });

	const model = await getEmbedder();

	onProgress?.({ stage: 'embedding', current: 0, total: notes.length });

	const results: EmbeddingResult[] = [];
	const BATCH_SIZE = 8;

	for (let i = 0; i < notes.length; i += BATCH_SIZE) {
		const batch = notes.slice(i, i + BATCH_SIZE);
		const texts = batch.map((n) => extractSummary(n.name, n.content));

		const output = await model(texts, { pooling: 'mean', normalize: true });

		// Extract embeddings from output tensor
		for (let j = 0; j < batch.length; j++) {
			const embedding = new Float32Array(output.data.slice(
				j * output.dims[1],
				(j + 1) * output.dims[1]
			));
			results.push({ id: batch[j].id, embedding });
		}

		onProgress?.({ stage: 'embedding', current: Math.min(i + BATCH_SIZE, notes.length), total: notes.length });

		// Yield to main thread periodically
		if (i % (BATCH_SIZE * 4) === 0 && i > 0) {
			await new Promise((r) => setTimeout(r, 0));
		}
	}

	return results;
}

/**
 * Find semantic links from precomputed embeddings.
 * Returns pairs above the similarity threshold, excluding explicit links.
 */
export function findSemanticLinks(
	embeddings: EmbeddingResult[],
	threshold: number = 0.5,
	explicitLinkSet?: Set<string>, // "sourceId->targetId" keys to exclude
	maxLinks: number = 500,
	onProgress?: (p: EmbeddingProgress) => void
): SemanticLink[] {
	const links: SemanticLink[] = [];
	const total = (embeddings.length * (embeddings.length - 1)) / 2;
	let computed = 0;

	onProgress?.({ stage: 'computing-links', current: 0, total });

	for (let i = 0; i < embeddings.length; i++) {
		for (let j = i + 1; j < embeddings.length; j++) {
			const sim = cosineSimilarity(embeddings[i].embedding, embeddings[j].embedding);

			if (sim >= threshold) {
				const fwd = `${embeddings[i].id}->${embeddings[j].id}`;
				const rev = `${embeddings[j].id}->${embeddings[i].id}`;

				// Skip if explicit link already exists
				if (explicitLinkSet && (explicitLinkSet.has(fwd) || explicitLinkSet.has(rev))) {
					continue;
				}

				links.push({
					source: embeddings[i].id,
					target: embeddings[j].id,
					similarity: sim,
				});
			}

			computed++;
			if (computed % 10000 === 0) {
				onProgress?.({ stage: 'computing-links', current: computed, total });
			}
		}
	}

	// Sort by similarity descending, take top N
	links.sort((a, b) => b.similarity - a.similarity);

	onProgress?.({ stage: 'done', current: total, total });

	return links.slice(0, maxLinks);
}

/**
 * Full pipeline: embed notes and find semantic links.
 */
export async function computeSemanticLinks(
	notes: { id: string; name: string; content: string }[],
	explicitLinks: { source: string; target: string }[],
	threshold: number = 0.5,
	maxLinks: number = 500,
	onProgress?: (p: EmbeddingProgress) => void
): Promise<SemanticLink[]> {
	if (notes.length < 2) return [];

	// Build explicit link set for exclusion
	const explicitSet = new Set<string>();
	for (const l of explicitLinks) {
		explicitSet.add(`${l.source}->${l.target}`);
		explicitSet.add(`${l.target}->${l.source}`);
	}

	const embeddings = await computeEmbeddings(notes, onProgress);
	return findSemanticLinks(embeddings, threshold, explicitSet, maxLinks, onProgress);
}

/** Check if the model is already cached/loaded */
export function isModelLoaded(): boolean {
	return embedder !== null;
}
