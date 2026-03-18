/**
 * GraphMind — Web Worker for D3 force simulation.
 * Runs layout computation off the main thread.
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
let tickCount = 0;
const MAX_TICKS = 300;

function sendPositions(settled: boolean) {
	// Pack positions as flat Float64Array: [x0, y0, x1, y1, ...]
	const positions = new Float64Array(nodes.length * 2);
	for (let i = 0; i < nodes.length; i++) {
		positions[i * 2] = nodes[i].x ?? 0;
		positions[i * 2 + 1] = nodes[i].y ?? 0;
	}
	(self as unknown as Worker).postMessage(
		{ type: 'positions', positions, settled },
		// Transfer the buffer for zero-copy
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

	// Create fresh node objects
	nodes = inNodes.map((n) => ({
		id: n.id,
		x: n.x ?? (Math.random() - 0.5) * 1000,
		y: n.y ?? (Math.random() - 0.5) * 1000,
	}));

	const nodeMap = new Map(nodes.map((n) => [n.id, n]));

	// Resolve edges to node references
	const edges: WEdge[] = inEdges
		.filter((e) => nodeMap.has(e.source) && nodeMap.has(e.target))
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
		.force('collide', forceCollide<WNode>(8))
		.alphaDecay(0.02)
		.velocityDecay(0.4)
		.on('tick', () => {
			tickCount++;
			// Send position updates every 3 ticks for performance
			if (tickCount % 3 === 0 || tickCount <= 5) {
				const settled = tickCount >= MAX_TICKS || (simulation?.alpha() ?? 0) < 0.001;
				sendPositions(settled);
			}
		})
		.on('end', () => {
			sendPositions(true);
		});
}

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
				simulation.alpha(0.5).restart();
				tickCount = 0;
			}
			break;

		case 'pinNode':
			if (nodes.length > 0) {
				const node = nodes.find((n) => n.id === msg.id);
				if (node) {
					node.fx = msg.x;
					node.fy = msg.y;
					simulation?.alpha(0.3).restart();
				}
			}
			break;

		case 'unpinNode':
			if (nodes.length > 0) {
				const node = nodes.find((n) => n.id === msg.id);
				if (node) {
					node.fx = null;
					node.fy = null;
					simulation?.alpha(0.3).restart();
				}
			}
			break;

		case 'restart':
			simulation?.alpha(0.5).restart();
			tickCount = 0;
			break;

		case 'stop':
			simulation?.stop();
			break;
	}
};
