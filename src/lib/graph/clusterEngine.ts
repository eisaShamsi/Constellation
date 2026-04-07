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

/* ------------------------------------------------------------------ */
/*  Constellation Lens — Structural Gaps, Entropy, Universe Health     */
/* ------------------------------------------------------------------ */

export interface StructuralGap {
	community1: number;
	community2: number;
	community1Name: string;
	community2Name: string;
	interLinkCount: number;
	/** Nodes in either community that could bridge the gap. */
	potentialBridges: string[];
}

export interface UniverseHealth {
	modularity: number;   // 0-1, from Louvain
	dominance: number;    // % of nodes in largest community (lower = better)
	entropy: number;      // Shannon entropy of community sizes (higher = more diverse)
	connectivity: number; // edges / nodes ratio
	score: number;        // composite 0-100
}

/**
 * Detect structural gaps between communities — pairs with high internal
 * density but low/zero inter-community connectivity.
 *
 * Based on Ronald Burt's structural holes theory (1992).
 */
export function computeStructuralGaps(
	clusters: ClusterInfo[],
	links: { source: string; target: string }[],
	assignments: Map<string, number>,
): StructuralGap[] {
	if (clusters.length < 2) return [];

	// Count inter-community links for each community pair
	const pairKey = (a: number, b: number) => a < b ? `${a}:${b}` : `${b}:${a}`;
	const interLinks = new Map<string, number>();
	const borderNodes = new Map<string, Set<string>>(); // pairKey → set of node IDs near the border

	for (const link of links) {
		const ca = assignments.get(link.source);
		const cb = assignments.get(link.target);
		if (ca === undefined || cb === undefined || ca === cb) continue;
		const key = pairKey(ca, cb);
		interLinks.set(key, (interLinks.get(key) ?? 0) + 1);
		if (!borderNodes.has(key)) borderNodes.set(key, new Set());
		borderNodes.get(key)!.add(link.source);
		borderNodes.get(key)!.add(link.target);
	}

	// Find community pairs with zero or very low inter-connectivity
	const gaps: StructuralGap[] = [];
	for (let i = 0; i < clusters.length; i++) {
		for (let j = i + 1; j < clusters.length; j++) {
			const ci = clusters[i];
			const cj = clusters[j];
			// Only consider communities with 3+ members each
			if (ci.memberIds.length < 3 || cj.memberIds.length < 3) continue;

			const key = pairKey(ci.id, cj.id);
			const count = interLinks.get(key) ?? 0;
			// Expected links = (size_i × size_j) / total_nodes (rough baseline)
			const expected = (ci.memberIds.length * cj.memberIds.length) / 100;

			// Gap if actual links are much fewer than expected
			if (count < Math.max(1, expected * 0.3)) {
				const bridges = borderNodes.has(key) ? [...borderNodes.get(key)!].slice(0, 5) : [];
				gaps.push({
					community1: ci.id,
					community2: cj.id,
					community1Name: ci.suggestedName,
					community2Name: cj.suggestedName,
					interLinkCount: count,
					potentialBridges: bridges,
				});
			}
		}
	}

	// Sort by lowest inter-link count (most disconnected first)
	gaps.sort((a, b) => a.interLinkCount - b.interLinkCount);
	return gaps.slice(0, 10); // Top 10 gaps
}

/**
 * Shannon entropy of community size distribution.
 * Higher entropy = more diverse/balanced community structure.
 */
export function computeEntropy(clusters: ClusterInfo[], totalNodes: number): number {
	if (clusters.length === 0 || totalNodes === 0) return 0;

	let entropy = 0;
	for (const c of clusters) {
		const p = c.memberIds.length / totalNodes;
		if (p > 0) entropy -= p * Math.log2(p);
	}
	return entropy;
}

/**
 * Composite universe health metric (0-100).
 *
 * Components:
 * - Modularity (0-1): how distinct the communities are
 * - Dominance: % of nodes in largest community (lower = better)
 * - Entropy: Shannon entropy of community distribution (higher = better)
 * - Connectivity: edge/node ratio (higher = better)
 *
 * Formula per concept paper Section 3.4.
 */
export function computeUniverseHealth(
	modularity: number,
	clusters: ClusterInfo[],
	totalNodes: number,
	totalEdges: number,
	gapCount: number,
): UniverseHealth {
	// Dominance: fraction in largest community
	const largestSize = clusters.reduce((max, c) => Math.max(max, c.memberIds.length), 0);
	const dominance = totalNodes > 0 ? largestSize / totalNodes : 0;

	// Entropy
	const entropy = computeEntropy(clusters, totalNodes);
	const maxEntropy = clusters.length > 1 ? Math.log2(clusters.length) : 1;
	const normEntropy = maxEntropy > 0 ? Math.min(entropy / maxEntropy, 1) : 0;

	// Connectivity
	const connectivity = totalNodes > 0 ? totalEdges / totalNodes : 0;
	const normConnectivity = Math.min(connectivity / 4, 1); // 4 edges/node = fully connected

	// Normalize modularity (0.3-0.6 is healthy per concept paper)
	const normModularity = Math.min(modularity / 0.6, 1);

	// Gap penalty
	const possiblePairs = clusters.length * (clusters.length - 1) / 2;
	const gapPenalty = possiblePairs > 0 ? Math.min(gapCount / possiblePairs, 1) : 0;

	// Composite: weighted sum
	const score = Math.round(
		25 * normModularity +
		25 * (1 - dominance) +
		25 * normEntropy +
		15 * normConnectivity +
		10 * (1 - gapPenalty)
	);

	return {
		modularity,
		dominance,
		entropy,
		connectivity,
		score: Math.max(0, Math.min(100, score)),
	};
}

/* ------------------------------------------------------------------ */
/*  Features 2,4,5,7: Stratum weighting, provenance, maturity, bridges */
/* ------------------------------------------------------------------ */

export interface CommunityProfile {
	id: number;
	name: string;
	color: string;
	memberCount: number;
	/** Maturity breakdown: { seed: 5, sapling: 3, evergreen: 2, ... } */
	maturityBreakdown: Record<string, number>;
	/** Provenance breakdown: { received: 4, discovered: 6, mixed: 1, ... } */
	provenanceBreakdown: Record<string, number>;
	/** % of notes that are wilting */
	wiltingPercent: number;
	/** Average stratum of community members */
	avgStratum: number;
}

/**
 * Build rich profiles for each community — maturity, provenance, stratum.
 * Uses CE Layer 1 data already on StarNode objects.
 */
export function buildCommunityProfiles(
	clusters: ClusterInfo[],
	assignments: Map<string, number>,
	nodes: { id: string; stratum?: number; maturity?: string; originType?: string }[],
): CommunityProfile[] {
	const nodeMap = new Map(nodes.map(n => [n.id, n]));

	return clusters.map(c => {
		const maturityBreakdown: Record<string, number> = {};
		const provenanceBreakdown: Record<string, number> = {};
		let stratumSum = 0;
		let stratumCount = 0;
		let wiltingCount = 0;

		for (const id of c.memberIds) {
			const node = nodeMap.get(id);
			if (!node) continue;

			// Maturity
			const m = node.maturity ?? 'seed';
			maturityBreakdown[m] = (maturityBreakdown[m] ?? 0) + 1;
			if (m === 'wilting') wiltingCount++;

			// Provenance
			const p = node.originType ?? 'none';
			provenanceBreakdown[p] = (provenanceBreakdown[p] ?? 0) + 1;

			// Stratum
			const s = node.stratum ?? 1;
			stratumSum += s;
			stratumCount++;
		}

		return {
			id: c.id,
			name: c.suggestedName,
			color: c.color,
			memberCount: c.memberIds.length,
			maturityBreakdown,
			provenanceBreakdown,
			wiltingPercent: c.memberIds.length > 0 ? (wiltingCount / c.memberIds.length) * 100 : 0,
			avgStratum: stratumCount > 0 ? stratumSum / stratumCount : 1,
		};
	});
}

/**
 * Feature 2: Apply stratum weighting to centrality scores.
 * A bridge between high-stratum notes is more important than between low-stratum.
 * Multiply centrality by average stratum of the node's neighbors.
 */
export function stratumWeightedCentrality(
	centrality: Map<string, number>,
	links: { source: string; target: string }[],
	nodes: { id: string; stratum?: number }[],
): Map<string, number> {
	const nodeStratum = new Map(nodes.map(n => [n.id, n.stratum ?? 1]));
	const neighborStrata = new Map<string, number[]>();

	for (const l of links) {
		if (!neighborStrata.has(l.source)) neighborStrata.set(l.source, []);
		if (!neighborStrata.has(l.target)) neighborStrata.set(l.target, []);
		neighborStrata.get(l.source)!.push(nodeStratum.get(l.target) ?? 1);
		neighborStrata.get(l.target)!.push(nodeStratum.get(l.source) ?? 1);
	}

	const weighted = new Map<string, number>();
	let maxVal = 0;

	for (const [id, c] of centrality) {
		const ownStratum = nodeStratum.get(id) ?? 1;
		const neighbors = neighborStrata.get(id) ?? [];
		const avgNeighborStratum = neighbors.length > 0
			? neighbors.reduce((s, v) => s + v, 0) / neighbors.length
			: 1;
		// Weight: centrality × sqrt(own_stratum × avg_neighbor_stratum)
		const w = c * Math.sqrt(ownStratum * avgNeighborStratum);
		weighted.set(id, w);
		if (w > maxVal) maxVal = w;
	}

	// Re-normalize to 0-1
	if (maxVal > 0) {
		for (const [id, w] of weighted) {
			weighted.set(id, w / maxVal);
		}
	}
	return weighted;
}

/**
 * Feature 7: Suggest bridge notes for each structural gap.
 * For each gap, find notes that share tags with notes in BOTH communities.
 */
export function suggestBridges(
	gaps: StructuralGap[],
	clusters: ClusterInfo[],
	nodes: { id: string; name: string }[],
	links: { source: string; target: string }[],
): StructuralGap[] {
	const clusterMap = new Map(clusters.map(c => [c.id, new Set(c.memberIds)]));
	// Build neighbor map
	const neighbors = new Map<string, Set<string>>();
	for (const l of links) {
		if (!neighbors.has(l.source)) neighbors.set(l.source, new Set());
		if (!neighbors.has(l.target)) neighbors.set(l.target, new Set());
		neighbors.get(l.source)!.add(l.target);
		neighbors.get(l.target)!.add(l.source);
	}

	return gaps.map(gap => {
		const c1Members = clusterMap.get(gap.community1) ?? new Set();
		const c2Members = clusterMap.get(gap.community2) ?? new Set();
		const bridges: string[] = [];

		// Find nodes in c1 that have neighbors in c2 (or vice versa)
		for (const id of c1Members) {
			const ns = neighbors.get(id);
			if (ns) {
				for (const n of ns) {
					if (c2Members.has(n)) { bridges.push(id); break; }
				}
			}
		}
		for (const id of c2Members) {
			const ns = neighbors.get(id);
			if (ns) {
				for (const n of ns) {
					if (c1Members.has(n)) { bridges.push(id); break; }
				}
			}
		}

		return { ...gap, potentialBridges: [...new Set(bridges)].slice(0, 5) };
	});
}
