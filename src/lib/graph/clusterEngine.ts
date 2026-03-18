/**
 * GraphMind — Phase 2: Louvain Community Detection
 *
 * Implements the Louvain algorithm for community detection in graphs.
 * Groups notes into clusters based on link density.
 * Linear time complexity, runs entirely in-browser.
 */

export interface ClusterResult {
	/** Map from node id → cluster id (0-based) */
	assignments: Map<string, number>;
	/** Cluster metadata: id → member count and suggested name */
	clusters: ClusterInfo[];
	/** Modularity score (0-1, higher = better partition) */
	modularity: number;
}

export interface ClusterInfo {
	id: number;
	memberIds: string[];
	memberNames: string[];
	suggestedName: string; // Auto-generated from most common words in titles
	color: string;
}

const CLUSTER_COLORS = [
	'#a78bfa', '#34d399', '#60a5fa', '#f472b6', '#fbbf24',
	'#f87171', '#2dd4bf', '#818cf8', '#fb923c', '#a3e635',
	'#e879f9', '#38bdf8', '#facc15', '#4ade80', '#f43f5e',
	'#94a3b8', '#c084fc', '#22d3ee', '#fb7185', '#84cc16',
];

/**
 * Louvain community detection algorithm.
 * Returns cluster assignments for each node.
 */
export function detectClusters(
	nodes: { id: string; name: string }[],
	links: { source: string; target: string }[],
	minClusterSize: number = 3
): ClusterResult {
	const n = nodes.length;
	if (n === 0) return { assignments: new Map(), clusters: [], modularity: 0 };

	const idToIdx = new Map<string, number>();
	nodes.forEach((node, i) => idToIdx.set(node.id, i));

	// Build adjacency list with weights
	const adj: Map<number, Map<number, number>> = new Map();
	for (let i = 0; i < n; i++) adj.set(i, new Map());

	let totalWeight = 0;
	for (const link of links) {
		const si = idToIdx.get(link.source);
		const ti = idToIdx.get(link.target);
		if (si === undefined || ti === undefined || si === ti) continue;

		const w = 1;
		adj.get(si)!.set(ti, (adj.get(si)!.get(ti) ?? 0) + w);
		adj.get(ti)!.set(si, (adj.get(ti)!.get(si) ?? 0) + w);
		totalWeight += w;
	}

	if (totalWeight === 0) return { assignments: new Map(), clusters: [], modularity: 0 };

	const m = totalWeight; // total edge weight (each edge counted once)

	// Node degrees (sum of edge weights)
	const degree: number[] = new Array(n).fill(0);
	for (let i = 0; i < n; i++) {
		for (const w of adj.get(i)!.values()) {
			degree[i] += w;
		}
	}

	// Initial: each node in its own community
	const community: number[] = Array.from({ length: n }, (_, i) => i);

	// Community totals
	const communitySum: number[] = [...degree]; // sum of degrees in community
	const communityIn: number[] = new Array(n).fill(0); // sum of internal edges * 2

	// Louvain Phase 1: local moving
	let improved = true;
	let iterations = 0;
	const MAX_ITERATIONS = 20;

	while (improved && iterations < MAX_ITERATIONS) {
		improved = false;
		iterations++;

		for (let i = 0; i < n; i++) {
			const ci = community[i];
			const ki = degree[i];

			// Compute weights to neighboring communities
			const neighborComms: Map<number, number> = new Map();
			for (const [j, w] of adj.get(i)!) {
				const cj = community[j];
				neighborComms.set(cj, (neighborComms.get(cj) ?? 0) + w);
			}

			// Weight to own community
			const kiIn = neighborComms.get(ci) ?? 0;

			// Remove i from its community
			communitySum[ci] -= ki;
			communityIn[ci] -= 2 * kiIn;

			// Find best community to move to
			let bestComm = ci;
			let bestGain = 0;

			for (const [cj, wj] of neighborComms) {
				// Modularity gain of moving i to cj
				const gain = wj - (communitySum[cj] * ki) / (2 * m);
				if (gain > bestGain) {
					bestGain = gain;
					bestComm = cj;
				}
			}

			// Move i to best community
			community[i] = bestComm;
			const wToBest = neighborComms.get(bestComm) ?? 0;
			communitySum[bestComm] += ki;
			communityIn[bestComm] += 2 * wToBest;

			if (bestComm !== ci) improved = true;
		}
	}

	// Renumber communities contiguously
	const uniqueComms = [...new Set(community)].sort((a, b) => a - b);
	const commRemap = new Map<number, number>();
	uniqueComms.forEach((c, i) => commRemap.set(c, i));

	const assignments = new Map<string, number>();
	for (let i = 0; i < n; i++) {
		assignments.set(nodes[i].id, commRemap.get(community[i])!);
	}

	// Build cluster info
	const clusterMembers: Map<number, { ids: string[]; names: string[] }> = new Map();
	for (let i = 0; i < n; i++) {
		const cid = commRemap.get(community[i])!;
		if (!clusterMembers.has(cid)) clusterMembers.set(cid, { ids: [], names: [] });
		clusterMembers.get(cid)!.ids.push(nodes[i].id);
		clusterMembers.get(cid)!.names.push(nodes[i].name);
	}

	const clusters: ClusterInfo[] = [];
	for (const [cid, members] of clusterMembers) {
		if (members.ids.length < minClusterSize) continue;
		clusters.push({
			id: cid,
			memberIds: members.ids,
			memberNames: members.names,
			suggestedName: suggestClusterName(members.names),
			color: CLUSTER_COLORS[clusters.length % CLUSTER_COLORS.length],
		});
	}

	// Compute modularity
	let Q = 0;
	for (let i = 0; i < n; i++) {
		for (const [j, w] of adj.get(i)!) {
			if (community[i] === community[j]) {
				Q += w - (degree[i] * degree[j]) / (2 * m);
			}
		}
	}
	Q /= (2 * m);

	return { assignments, clusters, modularity: Q };
}

/**
 * Auto-generate a cluster name from member note titles.
 * Uses word frequency analysis (stop words removed).
 */
function suggestClusterName(names: string[]): string {
	const stopWords = new Set([
		// English
		'the', 'a', 'an', 'and', 'or', 'but', 'in', 'on', 'at', 'to', 'for',
		'of', 'with', 'by', 'from', 'is', 'was', 'are', 'were', 'be', 'been',
		'have', 'has', 'had', 'do', 'does', 'did', 'will', 'would', 'could',
		'should', 'may', 'might', 'can', 'shall', 'not', 'no', 'this', 'that',
		'it', 'its', 'my', 'our', 'your', 'his', 'her', 'their', 'about', 'as',
		'into', 'through', 'during', 'before', 'after', 'above', 'below', 'between',
		// Arabic common
		'من', 'في', 'على', 'إلى', 'عن', 'مع', 'هذا', 'هذه', 'ذلك', 'تلك',
		'التي', 'الذي', 'التي', 'هو', 'هي', 'هم', 'كان', 'كانت', 'ما', 'لا',
		'أو', 'و', 'ثم', 'لكن', 'بل', 'حتى', 'إن', 'أن', 'قد', 'لم', 'لن',
		// File extensions
		'md', 'txt', 'note',
	]);

	const wordFreq: Map<string, number> = new Map();

	for (const name of names) {
		const clean = name.replace(/\.md$/, '').replace(/[_\-]/g, ' ');
		const words = clean.split(/\s+/).filter(w => w.length > 1);
		for (const word of words) {
			const lower = word.toLowerCase();
			if (stopWords.has(lower) || /^\d+$/.test(lower)) continue;
			wordFreq.set(lower, (wordFreq.get(lower) ?? 0) + 1);
		}
	}

	// Sort by frequency, take top 2-3 words
	const sorted = [...wordFreq.entries()].sort((a, b) => b[1] - a[1]);
	const topWords = sorted.slice(0, 3).map(([w]) => w);

	if (topWords.length === 0) return `Cluster ${names.length}`;

	// Capitalize first letter
	return topWords.map(w => w.charAt(0).toUpperCase() + w.slice(1)).join(' / ');
}
