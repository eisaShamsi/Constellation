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

	onMount(() => {
		if (nodes.length === 0) return;
		renderGraph();
	});

	$effect(() => {
		if (nodes.length > 0 && containerEl) {
			renderGraph();
		}
	});

	function renderGraph() {
		if (!containerEl) return;
		containerEl.innerHTML = '';

		const width = containerEl.clientWidth;
		const height = containerEl.clientHeight;

		const svg = d3.select(containerEl)
			.append('svg')
			.attr('width', width)
			.attr('height', height)
			.attr('viewBox', [0, 0, width, height]);

		// Zoom
		const g = svg.append('g');
		svg.call(d3.zoom<SVGSVGElement, unknown>()
			.extent([[0, 0], [width, height]])
			.scaleExtent([0.1, 8])
			.on('zoom', (event) => {
				g.attr('transform', event.transform);
			}) as any);

		// Build simulation
		const nodeData = nodes.map(n => ({ ...n }));
		const linkData = links.map(l => ({ ...l }));

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

		node.append('circle')
			.attr('r', (d: any) => Math.max(4, Math.min(12, 3 + d.linkCount * 1.5)))
			.attr('fill', (d: any) => d.id === activeNodeId ? '#7c3aed' : '#6b7280')
			.attr('stroke', '#fff')
			.attr('stroke-width', 1.5);

		node.append('text')
			.text((d: any) => d.name)
			.attr('x', 0)
			.attr('y', (d: any) => Math.max(4, Math.min(12, 3 + d.linkCount * 1.5)) + 12)
			.attr('text-anchor', 'middle')
			.attr('font-size', '9px')
			.attr('fill', '#5c5c66')
			.attr('pointer-events', 'none');

		// Hover effects
		node.on('mouseover', function(event, d: any) {
			d3.select(this).select('circle').attr('fill', '#7c3aed');
			// Highlight connected links
			link.attr('stroke', (l: any) =>
				l.source.id === d.id || l.target.id === d.id ? '#7c3aed' : '#d0d0d6'
			).attr('stroke-width', (l: any) =>
				l.source.id === d.id || l.target.id === d.id ? 2 : 1
			);
		}).on('mouseout', function(event, d: any) {
			d3.select(this).select('circle').attr('fill', d.id === activeNodeId ? '#7c3aed' : '#6b7280');
			link.attr('stroke', '#d0d0d6').attr('stroke-width', 1);
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
		background: #fafafa;
		position: relative;
	}
	.graph-empty {
		position: absolute; inset: 0;
		display: flex; align-items: center; justify-content: center;
		color: #b0b0b8; font-size: 0.85rem;
	}
</style>
