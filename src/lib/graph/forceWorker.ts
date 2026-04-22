/**
 * GraphMind — Layer 3: Web Worker for 3D force simulation.
 *
 * Strategy: d3-force runs the standard 2D simulation (x, y).
 * After each tick, we compute Z coordinates by treating each node's
 * 2D position as input to a deterministic 3D mapping:
 *   - Z = f(x, y, nodeIndex) using a hash-like function
 *   - Z magnitude matches X,Y spread so the shape is spherical
 *   - Connected nodes get similar Z offsets (via iterative smoothing)
 *
 * This avoids fighting d3's forces and guarantees equal spread in all axes.
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
	forceX,
	forceY,
	type SimulationNodeDatum,
	type SimulationLinkDatum,
} from 'd3-force';

interface WNode extends SimulationNodeDatum {
	id: string;
	z: number;
	baseZ: number; // deterministic Z seed based on index
	fx?: number | null;
	fy?: number | null;
	linkCount: number;
}

interface WEdge extends SimulationLinkDatum<WNode> {
	source: string | WNode;
	target: string | WNode;
}

let simulation: ReturnType<typeof forceSimulation<WNode>> | null = null;
let nodes: WNode[] = [];
let resolvedEdges: { src: number; tgt: number }[] = [];
let nodeById: Map<string, WNode> = new Map();
let tickCount = 0;
const MAX_TICKS = 300;

// Deterministic hash for Z seeding (consistent per node)
function hashZ(index: number, total: number): number {
	// Golden ratio based distribution — gives even spread in [-1, 1]
	const phi = (1 + Math.sqrt(5)) / 2;
	const h = ((index * phi) % 1) * 2 - 1; // [-1, 1]
	return h;
}

function sendPositions(settled: boolean) {
	// Compute Z values: match the X,Y spread so shape is round from every angle
	let maxXY = 0;
	for (const n of nodes) {
		const ax = Math.abs(n.x ?? 0);
		const ay = Math.abs(n.y ?? 0);
		if (ax > maxXY) maxXY = ax;
		if (ay > maxXY) maxXY = ay;
	}
	if (maxXY < 1) maxXY = 1;

	// Set Z from deterministic seed, scaled to match X,Y spread. Scale up
	// (1.4×) so the sphere reads as genuinely 3D when rotated — at 1:1
	// with X/Y, the Z spread was too flat and the constellation collapsed
	// to a strip on yaw rotation. 1.4× gives visible depth without making
	// the graph "egg-shaped" in Z.
	for (let i = 0; i < nodes.length; i++) {
		nodes[i].z = nodes[i].baseZ * maxXY * 1.4;
	}

	// Edge-based Z smoothing used to run 3 passes at 0.15 mixing rate. On
	// a 217k-edge universe that flattens Z almost completely — connected
	// nodes drag each other onto the same depth plane. Drop to a single
	// very light pass so connected nodes sit on _similar_ depths without
	// collapsing the whole field. This is what restores the 3D spherical
	// impression when the view rotates.
	for (const e of resolvedEdges) {
		const src = nodes[e.src];
		const tgt = nodes[e.tgt];
		if (!src || !tgt) continue;
		const avg = (src.z + tgt.z) * 0.5;
		src.z = src.z * 0.97 + avg * 0.03;
		tgt.z = tgt.z * 0.97 + avg * 0.03;
	}

	// Pack as [x0, y0, z0, x1, y1, z1, ...]
	const positions = new Float64Array(nodes.length * 3);
	for (let i = 0; i < nodes.length; i++) {
		positions[i * 3] = nodes[i].x ?? 0;
		positions[i * 3 + 1] = nodes[i].y ?? 0;
		positions[i * 3 + 2] = nodes[i].z;
	}
	(self as unknown as Worker).postMessage(
		{ type: 'positions', positions, settled },
		[positions.buffer] as any
	);
}

function initSimulation(
	inNodes: { id: string; x?: number; y?: number; linkCount?: number }[],
	inEdges: { source: string; target: string }[],
	settings: { repelForce: number; linkForce: number; linkDistance: number; centerForce: number }
) {
	if (simulation) {
		simulation.stop();
		simulation = null;
	}

	tickCount = 0;
	const N = inNodes.length;

	nodes = inNodes.map((n, i) => ({
		id: n.id,
		x: n.x ?? (Math.random() - 0.5) * 1000,
		y: n.y ?? (Math.random() - 0.5) * 1000,
		z: 0,
		baseZ: hashZ(i, N), // deterministic Z seed in [-1, 1]
		linkCount: n.linkCount ?? 0,
	}));

	nodeById = new Map(nodes.map((n) => [n.id, n]));
	const nodeIdxMap = new Map(nodes.map((n, i) => [n.id, i]));

	const edges: WEdge[] = inEdges
		.filter((e) => nodeById.has(e.source) && nodeById.has(e.target))
		.map((e) => ({ source: e.source, target: e.target }));

	// Pre-resolve edge indices for Z smoothing
	resolvedEdges = inEdges
		.filter((e) => nodeIdxMap.has(e.source) && nodeIdxMap.has(e.target))
		.map((e) => ({ src: nodeIdxMap.get(e.source)!, tgt: nodeIdxMap.get(e.target)! }));

	simulation = forceSimulation<WNode>(nodes)
		.force(
			'link',
			forceLink<WNode, WEdge>(edges)
				.id((d) => d.id)
				.strength(settings.linkForce)
				.distance(settings.linkDistance)
		)
		// distanceMax caps repulsion range. Without it every node pushes
		// every other node apart regardless of distance — which is exactly
		// what keeps two disconnected library-clusters floating apart.
		// Capping at 400 world-units means far clusters stop repelling each
		// other and the center pull wins, merging them into one
		// constellation without having to crank center force so hard it
		// collapses the whole graph to a blob.
		.force('charge', forceManyBody<WNode>().strength(-settings.repelForce).distanceMax(400).theta(1.2))
		.force('center', forceCenter(0, 0).strength(settings.centerForce))
		// Per-axis pull toward origin. With bounded repulsion above, a
		// modest strength is enough to merge the clusters.
		.force('x', forceX<WNode>(0).strength(0.15))
		.force('y', forceY<WNode>(0).strength(0.15))
		.force('collide', forceCollide<WNode>(6))
		.alphaDecay(0.03)
		.alphaMin(0.005)
		.velocityDecay(0.5)
		.on('tick', () => {
			tickCount++;
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
