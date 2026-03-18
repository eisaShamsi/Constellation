/**
 * GraphMind — Layer 3: Web Worker for D3 force simulation.
 *
 * STRICT STOP GATE (Law 2):
 * Only two events may restart the simulation:
 *   1. dragEnd  — user finished dragging a node
 *   2. updateSettings — user changed physics settings
 *
 * Hover events NEVER reach this worker. This is enforced by design
 * in graphEngine.ts (Layer 2), which never sends hover-related messages.
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
	fx?: number | null;
	fy?: number | null;
}

interface WEdge extends SimulationLinkDatum<WNode> {
	source: string | WNode;
	target: string | WNode;
}

let simulation: ReturnType<typeof forceSimulation<WNode>> | null = null;
let nodes: WNode[] = [];
let nodeById: Map<string, WNode> = new Map();
let tickCount = 0;
const MAX_TICKS = 300;

function sendPositions(settled: boolean) {
	const positions = new Float64Array(nodes.length * 2);
	for (let i = 0; i < nodes.length; i++) {
		positions[i * 2] = nodes[i].x ?? 0;
		positions[i * 2 + 1] = nodes[i].y ?? 0;
	}
	(self as unknown as Worker).postMessage(
		{ type: 'positions', positions, settled },
		[positions.buffer] as any
	);
}

function initSimulation(
	inNodes: { id: string; x?: number; y?: number }[],
	inEdges: { source: string; target: string }[],
	settings: { repelForce: number; linkForce: number; linkDistance: number; centerForce: number }
) {
	if (simulation) {
		simulation.stop();
		simulation = null;
	}

	tickCount = 0;

	nodes = inNodes.map((n) => ({
		id: n.id,
		x: n.x ?? (Math.random() - 0.5) * 1000,
		y: n.y ?? (Math.random() - 0.5) * 1000,
	}));

	nodeById = new Map(nodes.map((n) => [n.id, n]));

	const edges: WEdge[] = inEdges
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
		// ── INIT: start fresh simulation ──
		case 'init':
			initSimulation(msg.nodes, msg.edges, msg.settings);
			(self as unknown as Worker).postMessage({ type: 'ready' });
			break;

		// ── SETTINGS_CHANGE: allowed restart (Law 2) ──
		case 'updateSettings':
			if (simulation) {
				const s = msg.settings;
				(simulation.force('charge') as any)?.strength(-s.repelForce);
				(simulation.force('link') as any)?.strength(s.linkForce).distance(s.linkDistance);
				(simulation.force('center') as any)?.strength(s.centerForce);
				simulation.alpha(0.3).restart(); // Allowed: SETTINGS_CHANGE
				tickCount = 0;
			}
			break;

		// ── DRAG_END: allowed restart (Law 2) ──
		case 'dragEnd':
			if (simulation) {
				const node = nodeById.get(msg.id);
				if (node) {
					node.x = msg.x;
					node.y = msg.y;
					node.fx = msg.x;
					node.fy = msg.y;
					simulation.alpha(0.3).restart(); // Allowed: DRAG_END
					tickCount = 0;
				}
			}
			break;

		// ── pinNode: position only, NO restart ──
		case 'pinNode':
			if (nodeById.size > 0) {
				const node = nodeById.get(msg.id);
				if (node) {
					node.fx = msg.x;
					node.fy = msg.y;
					// NO restart — just record position during active drag
				}
			}
			break;

		// ── unpinNode: release, NO restart ──
		case 'unpinNode':
			if (nodeById.size > 0) {
				const node = nodeById.get(msg.id);
				if (node) {
					node.fx = null;
					node.fy = null;
					// NO restart — node stays where it was
				}
			}
			break;

		// ── stop: terminate simulation ──
		case 'stop':
			simulation?.stop();
			break;

		// ── All other message types are silently ignored ──
		// This is the defensive gate. Hover, refresh, render events
		// can NEVER reach here because graphEngine.ts never sends them.
		default:
			break;
	}
};
