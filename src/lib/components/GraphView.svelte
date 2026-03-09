<script lang="ts">
	import { onMount, onDestroy } from 'svelte';
	import * as d3 from 'd3';

	interface GraphNode extends d3.SimulationNodeDatum {
		id: string;
		name: string;
		path: string;
		vaultName: string;
		group?: string;
		linkCount: number;
	}

	interface GraphLink {
		source: string;
		target: string;
	}

	let {
		nodes = [] as GraphNode[],
		links = [] as GraphLink[],
		onNodeClick,
		activeNodeId = '',
		ar = false,
	}: {
		nodes: GraphNode[];
		links: GraphLink[];
		onNodeClick: (path: string, vaultName: string) => void;
		activeNodeId?: string;
		ar?: boolean;
	} = $props();

	let containerEl: HTMLDivElement;
	let simulation: d3.Simulation<any, any> | null = null;

	// Store references for external updates (center on active node)
	let svgRef: d3.Selection<SVGSVGElement, unknown, null, undefined> | null = null;
	let gRef: d3.Selection<SVGGElement, unknown, null, undefined> | null = null;
	let zoomBehavior: d3.ZoomBehavior<SVGSVGElement, unknown> | null = null;
	let nodeDataRef: any[] = [];
	let nodeGroupRef: d3.Selection<SVGGElement, any, SVGGElement, unknown> | null = null;
	let linkRef: d3.Selection<SVGLineElement, any, SVGGElement, unknown> | null = null;
	let prevActiveNodeId = '';

	onMount(() => {
		if (nodes.length === 0) return;
		renderGraph();
	});

	$effect(() => {
		if (nodes.length > 0 && containerEl) {
			renderGraph();
		}
	});

	// React to activeNodeId changes — center and highlight
	$effect(() => {
		if (activeNodeId && activeNodeId !== prevActiveNodeId && svgRef && gRef && zoomBehavior && nodeDataRef.length > 0) {
			prevActiveNodeId = activeNodeId;
			centerOnNode(activeNodeId);
		}
	});

	function centerOnNode(nodeId: string) {
		if (!svgRef || !gRef || !zoomBehavior) return;

		const targetNode = nodeDataRef.find((n: any) => n.id === nodeId);
		if (!targetNode || targetNode.x == null || targetNode.y == null) return;

		const width = containerEl.clientWidth;
		const height = containerEl.clientHeight;
		const scale = 1.5;
		const x = width / 2 - targetNode.x * scale;
		const y = height / 2 - targetNode.y * scale;

		svgRef.transition()
			.duration(750)
			.call(
				zoomBehavior.transform as any,
				d3.zoomIdentity.translate(x, y).scale(scale)
			);

		// Update node visuals — highlight active, dim others
		updateActiveHighlight(nodeId);
	}

	function updateActiveHighlight(nodeId: string) {
		if (!nodeGroupRef || !linkRef) return;

		// Highlight circles
		nodeGroupRef.select('circle')
			.transition().duration(300)
			.attr('fill', (d: any) => d.id === nodeId ? '#7c3aed' : '#6b7280')
			.attr('stroke', (d: any) => d.id === nodeId ? '#a78bfa' : '#fff')
			.attr('stroke-width', (d: any) => d.id === nodeId ? 3 : 1.5);

		// Highlight connected links
		linkRef
			.transition().duration(300)
			.attr('stroke', (l: any) =>
				l.source.id === nodeId || l.target.id === nodeId ? '#7c3aed' : '#d0d0d6'
			)
			.attr('stroke-width', (l: any) =>
				l.source.id === nodeId || l.target.id === nodeId ? 2 : 1
			)
			.attr('stroke-opacity', (l: any) =>
				l.source.id === nodeId || l.target.id === nodeId ? 1 : 0.4
			);

		// Highlight connected node circles too
		const connectedIds = new Set<string>();
		linkRef.each((l: any) => {
			if (l.source.id === nodeId) connectedIds.add(l.target.id);
			if (l.target.id === nodeId) connectedIds.add(l.source.id);
		});

		nodeGroupRef.select('circle')
			.transition().duration(300)
			.attr('opacity', (d: any) =>
				d.id === nodeId || connectedIds.has(d.id) ? 1 : 0.3
			);

		// Show label on active node persistently
		nodeGroupRef.select('text.node-label')
			.transition().duration(300)
			.attr('opacity', (d: any) =>
				d.id === nodeId || connectedIds.has(d.id) ? 1 : 0
			);
	}

	function renderGraph() {
		if (!containerEl) return;
		containerEl.innerHTML = '';

		const width = containerEl.clientWidth;
		const height = containerEl.clientHeight;

		// Tooltip
		const tooltip = d3.select(containerEl)
			.append('div')
			.attr('class', 'graph-tooltip')
			.style('opacity', 0);

		const svg = d3.select(containerEl)
			.append('svg')
			.attr('width', width)
			.attr('height', height)
			.attr('viewBox', [0, 0, width, height]);
		svgRef = svg;

		// Zoom
		const g = svg.append('g');
		gRef = g;

		zoomBehavior = d3.zoom<SVGSVGElement, unknown>()
			.extent([[0, 0], [width, height]])
			.scaleExtent([0.1, 8])
			.on('zoom', (event) => {
				g.attr('transform', event.transform);
			});
		svg.call(zoomBehavior as any);

		// Build simulation
		const nodeData = nodes.map(n => ({ ...n }));
		const linkData = links.map(l => ({ ...l }));
		nodeDataRef = nodeData;

		simulation = d3.forceSimulation(nodeData)
			.force('link', d3.forceLink(linkData).id((d: any) => d.id).distance(80))
			.force('charge', d3.forceManyBody().strength(-200))
			.force('center', d3.forceCenter(width / 2, height / 2))
			.force('collision', d3.forceCollide().radius(20));

		// Draw links
		const link = g.append('g')
			.selectAll('line')
			.data(linkData)
			.join('line')
			.attr('stroke', '#d0d0d6')
			.attr('stroke-width', 1)
			.attr('stroke-opacity', 0.6);
		linkRef = link as any;

		// Draw nodes
		const node = g.append('g')
			.selectAll('g')
			.data(nodeData)
			.join('g')
			.attr('cursor', 'pointer')
			.call(d3.drag<SVGGElement, any>()
				.on('start', (event, d: any) => {
					if (!event.active) simulation?.alphaTarget(0.3).restart();
					d.fx = d.x;
					d.fy = d.y;
				})
				.on('drag', (event, d: any) => {
					d.fx = event.x;
					d.fy = event.y;
				})
				.on('end', (event, d: any) => {
					if (!event.active) simulation?.alphaTarget(0);
					d.fx = null;
					d.fy = null;
				}) as any);
		nodeGroupRef = node as any;

		node.append('circle')
			.attr('r', (d: any) => Math.max(4, Math.min(12, 3 + d.linkCount * 1.5)))
			.attr('fill', (d: any) => d.id === activeNodeId ? '#7c3aed' : '#6b7280')
			.attr('stroke', (d: any) => d.id === activeNodeId ? '#a78bfa' : '#fff')
			.attr('stroke-width', (d: any) => d.id === activeNodeId ? 3 : 1.5);

		// Hidden labels (shown on hover or for active/connected nodes)
		node.append('text')
			.attr('class', 'node-label')
			.text((d: any) => d.name)
			.attr('x', 0)
			.attr('y', (d: any) => Math.max(4, Math.min(12, 3 + d.linkCount * 1.5)) + 12)
			.attr('text-anchor', 'middle')
			.attr('font-size', '9px')
			.attr('fill', 'var(--text-secondary, #5c5c66)')
			.attr('pointer-events', 'none')
			.attr('opacity', 0);

		// Hover: show tooltip + highlight connections
		node.on('mouseover', function(event, d: any) {
			// Show tooltip
			tooltip
				.html(d.name)
				.style('opacity', 1)
				.style('left', (event.offsetX + 12) + 'px')
				.style('top', (event.offsetY - 8) + 'px');

			// Highlight this node
			d3.select(this).select('circle')
				.transition().duration(150)
				.attr('fill', '#7c3aed')
				.attr('stroke', '#a78bfa')
				.attr('stroke-width', 3);

			// Show this node's label
			d3.select(this).select('text.node-label')
				.transition().duration(150)
				.attr('opacity', 1);

			// Find connected node ids
			const connectedIds = new Set<string>();
			link.each((l: any) => {
				if (l.source.id === d.id) connectedIds.add(l.target.id);
				if (l.target.id === d.id) connectedIds.add(l.source.id);
			});

			// Highlight connected links
			link
				.transition().duration(150)
				.attr('stroke', (l: any) =>
					l.source.id === d.id || l.target.id === d.id ? '#7c3aed' : '#d0d0d6'
				)
				.attr('stroke-width', (l: any) =>
					l.source.id === d.id || l.target.id === d.id ? 2 : 1
				)
				.attr('stroke-opacity', (l: any) =>
					l.source.id === d.id || l.target.id === d.id ? 1 : 0.15
				);

			// Dim unrelated nodes, highlight connected
			node.select('circle')
				.transition().duration(150)
				.attr('opacity', (n: any) =>
					n.id === d.id || connectedIds.has(n.id) ? 1 : 0.15
				);

			// Show connected labels
			node.select('text.node-label')
				.transition().duration(150)
				.attr('opacity', (n: any) =>
					n.id === d.id || connectedIds.has(n.id) ? 1 : 0
				);

		}).on('mousemove', function(event) {
			tooltip
				.style('left', (event.offsetX + 12) + 'px')
				.style('top', (event.offsetY - 8) + 'px');

		}).on('mouseout', function(event, d: any) {
			tooltip.style('opacity', 0);

			// Restore node styles
			d3.select(this).select('circle')
				.transition().duration(200)
				.attr('fill', d.id === activeNodeId ? '#7c3aed' : '#6b7280')
				.attr('stroke', d.id === activeNodeId ? '#a78bfa' : '#fff')
				.attr('stroke-width', d.id === activeNodeId ? 3 : 1.5);

			// Restore all
			node.select('circle')
				.transition().duration(200)
				.attr('opacity', 1);

			link
				.transition().duration(200)
				.attr('stroke', '#d0d0d6')
				.attr('stroke-width', 1)
				.attr('stroke-opacity', 0.6);

			// Hide all labels
			node.select('text.node-label')
				.transition().duration(200)
				.attr('opacity', 0);

		}).on('click', (event: any, d: any) => {
			onNodeClick(d.path, d.vaultName);
		});

		simulation.on('tick', () => {
			link
				.attr('x1', (d: any) => d.source.x)
				.attr('y1', (d: any) => d.source.y)
				.attr('x2', (d: any) => d.target.x)
				.attr('y2', (d: any) => d.target.y);

			node.attr('transform', (d: any) => `translate(${d.x},${d.y})`);
		});

		// If there's an initial active node, center on it once the simulation stabilizes
		if (activeNodeId) {
			prevActiveNodeId = activeNodeId;
			simulation.on('end', () => {
				centerOnNode(activeNodeId);
			});
			// Also center after a delay in case simulation takes long
			setTimeout(() => centerOnNode(activeNodeId), 1500);
		}
	}

	onDestroy(() => {
		simulation?.stop();
	});
</script>

<div class="graph-container" bind:this={containerEl}>
	{#if nodes.length === 0}
		<div class="graph-empty">{ar ? 'لا توجد ملاحظات لعرضها' : 'No notes to display'}</div>
	{/if}
</div>

<style>
	.graph-container {
		width: 100%; height: 100%;
		background: var(--bg, #fafafa);
		position: relative;
		overflow: hidden;
	}
	.graph-empty {
		position: absolute; inset: 0;
		display: flex; align-items: center; justify-content: center;
		color: var(--text-faint, #b0b0b8); font-size: 0.85rem;
	}
	.graph-container :global(.graph-tooltip) {
		position: absolute;
		pointer-events: none;
		background: var(--bg, #fff);
		border: 1px solid var(--border, #e0e0e4);
		border-radius: 6px;
		padding: 4px 10px;
		font-size: 0.8rem;
		font-weight: 500;
		color: var(--text, #1f2328);
		box-shadow: 0 2px 8px rgba(0,0,0,0.12);
		white-space: nowrap;
		z-index: 10;
		transition: opacity 0.15s ease;
	}
</style>
