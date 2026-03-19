/**
 * GraphMind — Layer 3: Web Worker for TRUE 3D force simulation.
 *
 * d3-force is 2D only, so we add Z-axis physics manually:
 * - Each node has x, y, z, vx, vy, vz
 * - d3-force handles x, y forces (links, charge, center)
 * - We manually compute Z repulsion + Z centering on each tick
 * - Positions streamed as Float64Array [x0, y0, z0, x1, y1, z1, ...]
 *
 * STRICT STOP GATE (Law 2):
 * Only dragEnd and updateSettings may restart the simulation.
 */

import {
	forceSimulation,
	forceLink,
	forceManyBody,
	forceCenter,
	forceCollide,
	type SimulationNodeDatum,
	type SimulationLinkDatum,
} from 'd3-force';

interface WNode extends SimulationNodeDatum {
	id: string;
	z: number;
	vz: number;
	fx?: number | null;
	fy?: number | null;
	fz?: number | null;
	linkCount: number;
}

interface WEdge extends SimulationLinkDatum<WNode> {
	source: string | WNode;
	target: string | WNode;
}

let simulation: ReturnType<typeof forceSimulation<WNode>> | null = null;
let nodes: WNode[] = [];
let edges: WEdge[] = [];
let nodeById: Map<string, WNode> = new Map();
let tickCount = 0;
const MAX_TICKS = 300;

function sendPositions(settled: boolean) {
	// Pack as [x0, y0, z0, x1, y1, z1, ...] — 3 values per node
	const positions = new Float64Array(nodes.length * 3);
	for (let i = 0; i < nodes.length; i++) {
		positions[i * 3] = nodes[i].x ?? 0;
		positions[i * 3 + 1] = nodes[i].y ?? 0;
		positions[i * 3 + 2] = nodes[i].z ?? 0;
	}
	(self as unknown as Worker).postMessage(
		{ type: 'positions', positions, settled },
		[positions.buffer] as any
	);
}

/**
 * Custom Z-axis physics applied every tick.
 * Since d3-force only handles x,y, we manually:
 * 1. Apply Z repulsion between nodes (simple N-body on Z axis)
 * 2. Apply Z centering force (pull toward z=0)
 * 3. Apply Z link attraction (linked nodes attract in Z)
 * 4. Apply velocity damping
 */
function applyZForces(repelStrength: number, centerStrength: number, linkStrength: number) {
	const N = nodes.length;
	const velocityDecay = 0.5;

	// Z centering
	for (let i = 0; i < N; i++) {
		const n = nodes[i];
		if (n.fz != null) { n.z = n.fz; n.vz = 0; continue; } // pinned
		n.vz += -n.z * centerStrength * 0.1; // pull toward z=0
	}

	// Z repulsion (approximate — use random sampling for large N)
	const sampleSize = Math.min(N, 50); // limit comparisons for performance
	for (let i = 0; i < N; i++) {
		if (nodes[i].fz != null) continue;
		let fz = 0;
		for (let s = 0; s < sampleSize; s++) {
			const j = (sampleSize === N) ? s : Math.floor(Math.random() * N);
			if (i === j) continue;
			const dx = (nodes[i].x ?? 0) - (nodes[j].x ?? 0);
			const dy = (nodes[i].y ?? 0) - (nodes[j].y ?? 0);
			const dz = nodes[i].z - nodes[j].z;
			const dist2 = dx * dx + dy * dy + dz * dz + 1;
			// Repel in Z proportional to inverse square, scaled by repelStrength
			fz += (dz / Math.sqrt(dist2)) * repelStrength / dist2 * 500;
		}
		nodes[i].vz += fz;
	}

	// Z link attraction (linked nodes should be somewhat close in Z too)
	for (const edge of edges) {
		const src = edge.source as WNode;
		const tgt = edge.target as WNode;
		if (!src || !tgt) continue;
		const dz = tgt.z - src.z;
		const force = dz * linkStrength * 0.02;
		if (src.fz == null) src.vz += force;
		if (tgt.fz == null) tgt.vz -= force;
	}

	// Apply velocity + decay
	for (let i = 0; i < N; i++) {
		if (nodes[i].fz != null) continue;
		nodes[i].vz *= velocityDecay;
		nodes[i].z += nodes[i].vz;
	}
}

function initSimulation(
	inNodes: { id: string; x?: number; y?: number; z?: number; linkCount?: number }[],
	inEdges: { source: string; target: string }[],
	settings: { repelForce: number; linkForce: number; linkDistance: number; centerForce: number }
) {
	if (simulation) {
		simulation.stop();
		simulation = null;
	}

	tickCount = 0;

	// Initialize with random 3D positions
	nodes = inNodes.map((n) => ({
		id: n.id,
		x: n.x ?? (Math.random() - 0.5) * 1000,
		y: n.y ?? (Math.random() - 0.5) * 1000,
		z: n.z ?? (Math.random() - 0.5) * 1000, // TRUE 3D — random Z spread
		vz: 0,
		linkCount: n.linkCount ?? 0,
	}));

	nodeById = new Map(nodes.map((n) => [n.id, n]));

	edges = inEdges
		.filter((e) => nodeById.has(e.source) && nodeById.has(e.target))
		.map((e) => ({ source: e.source, target: e.target }));

	simulation = forceSimulation<WNode>(nodes)
		.force(
			'link',
			forceLink<WNode, WEdge>(edges)
				.id((d) => d.id)
				.strength(settings.linkForce)
				.distance(settings.linkDistance)
		)
		.force('charge', forceManyBody<WNode>().strength(-settings.repelForce).theta(1.2))
		.force('center', forceCenter(0, 0).strength(settings.centerForce))
		.force('collide', forceCollide<WNode>(6))
		.alphaDecay(0.03)
		.alphaMin(0.005)
		.velocityDecay(0.5)
		.on('tick', () => {
			tickCount++;

			// Apply Z-axis physics on every tick (d3 only does x,y)
			applyZForces(settings.repelForce, settings.centerForce, settings.linkForce);

			if (tickCount % 3 === 0 || tickCount <= 5) {
				const settled = tickCount >= MAX_TICKS || (simulation?.alpha() ?? 0) < 0.005;
				sendPositions(settled);
				if (settled && simulation) {
					simulation.stop();
				}
			}
		})
		.on('end', () => {
			sendPositions(true);
		});
}

// ─── Message Handler with STRICT STOP GATE ────────────────────────────

self.onmessage = (e: MessageEvent) => {
	const msg = e.data;

	switch (msg.type) {
		case 'init':
			initSimulation(msg.nodes, msg.edges, msg.settings);
			(self as unknown as Worker).postMessage({ type: 'ready' });
			break;

		case 'updateSettings':
			if (simulation) {
				const s = msg.settings;
				(simulation.force('charge') as any)?.strength(-s.repelForce);
				(simulation.force('link') as any)?.strength(s.linkForce).distance(s.linkDistance);
				(simulation.force('center') as any)?.strength(s.centerForce);
				simulation.alpha(0.3).restart();
				tickCount = 0;
			}
			break;

		case 'dragEnd':
			if (simulation) {
				const node = nodeById.get(msg.id);
				if (node) {
					node.x = msg.x;
					node.y = msg.y;
					node.fx = msg.x;
					node.fy = msg.y;
					simulation.alpha(0.3).restart();
					tickCount = 0;
				}
			}
			break;

		case 'pinNode':
			if (nodeById.size > 0) {
				const node = nodeById.get(msg.id);
				if (node) {
					node.fx = msg.x;
					node.fy = msg.y;
				}
			}
			break;

		case 'unpinNode':
			if (nodeById.size > 0) {
				const node = nodeById.get(msg.id);
				if (node) {
					node.fx = null;
					node.fy = null;
				}
			}
			break;

		case 'stop':
			simulation?.stop();
			break;

		default:
			break;
	}
};
