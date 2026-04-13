<script lang="ts">
	/**
	 * ConstellationSight2 — Ground-up rebuild with Living Link System.
	 *
	 * The stethoscope for your knowledge system: shows the circulatory health
	 * of your thinking — where blood flows strongly, where vessels narrow,
	 * where the heart beats fast, and where tissue is dying.
	 *
	 * Architecture:
	 *   Gravity-well radial layout + Canvas rendering
	 *   Living Link enrichment (type colors, weight thickness, confidence styles)
	 *   Search (6 scopes, chips, history, category badges)
	 *   Insight Panel (health, links, lifecycle, formulation, communities)
	 */
	import { onMount, onDestroy } from 'svelte';
	import { invoke } from '@tauri-apps/api/core';
	import SightPanel from './SightPanel.svelte';
	import * as d3 from 'd3';
	import { t, dir, getSearchOps } from '$lib/i18n';
	import { detectDir } from '$lib/utils';
	import { readSearchHistory, addSearchHistory } from '$lib/libraries/searchHistory';
	import {
		stripInvisibleChars, canonicalizeSearchQuery, hasAdvancedSyntaxMultilingual,
		universalSearch, constellationSearch, parseSearchQuery,
		embedText, appSettings, type UniversalSearchResponse,
	} from '$lib/libraries/store';
	import { get } from 'svelte/store';
	import type { SkyNode, SkyLink } from '$lib/libraries/store';
	import type { ClusterInfo, StructuralGap, UniverseHealth, CommunityProfile } from '$lib/graph/clusterEngine';

	// ─── Types ────────────────────────────────────────────────
	interface SimNode extends d3.SimulationNodeDatum {
		id: string;
		name: string;
		path: string;
		libraryName: string;
		centrality: number;
		libraryColor: string;
		r: number;
		maturity?: string;
		linkCount?: number;
		stratum?: number;
	}

	interface SimLink extends d3.SimulationLinkDatum<SimNode> {
		linkType?: string;
		weight?: number;
		confidence?: string;
		annotation?: string;
		traversalCount?: number;
		status?: string;
	}

	interface SearchMatch {
		node: SimNode;
		matchType: string;
		matchCategories: string[];
	}

	// ─── Link Type Colors (Living Link System) ────────────────
	const LINK_TYPE_COLORS: Record<string, string> = {
		supports: '#4A9EFF', contradicts: '#FF4A4A', causes: '#FF8C42',
		exemplifies: '#4AFF88', generalizes: '#C084FC', 'derives-from': '#FACC15',
		'part-of': '#94A3B8', associative: '#A78BFA', relates: '#94a3b8',
	};

	const MATURITY_COLORS: Record<string, string> = {
		seed: '#d1d5db', sapling: '#86efac', evergreen: '#16a34a',
		canonical: '#f59e0b', wilting: '#ef4444',
	};

	const CONFIDENCE_STYLE: Record<string, { dash: number[]; widthMul: number }> = {
		hypothesis: { dash: [4, 3], widthMul: 0.7 },
		evidence: { dash: [], widthMul: 1.0 },
		established: { dash: [], widthMul: 1.5 },
		contested: { dash: [2, 2], widthMul: 1.2 },
	};

	const CAT_COLORS: Record<string, string> = {
		T: '#3b82f6', C: '#16a34a', '#': '#f472b6', P: '#f59e0b', S: '#7c3aed',
	};

	// ─── Props ────────────────────────────────────────────────
	let {
		nodes = [] as SkyNode[],
		links = [] as SkyLink[],
		centrality = new Map<string, number>(),
		communityAssignments = new Map<string, number>(),
		communityColors = new Map<number, string>(),
		gaps = [] as StructuralGap[],
		health = null as UniverseHealth | null,
		bridges = [] as { id: string; name: string; centrality: number }[],
		communities = [] as ClusterInfo[],
		communityProfiles = [] as CommunityProfile[],
		contradictions = [] as [string, string][],
		libraryColorMap = {} as Record<string, string>,
		searchMatchIds = null as Set<string> | null,
		onNoteClick,
		onClose,
	}: {
		nodes: SkyNode[];
		links: SkyLink[];
		centrality?: Map<string, number>;
		communityAssignments?: Map<string, number>;
		communityColors?: Map<number, string>;
		gaps?: StructuralGap[];
		health?: UniverseHealth | null;
		bridges?: { id: string; name: string; centrality: number }[];
		communities?: ClusterInfo[];
		communityProfiles?: CommunityProfile[];
		contradictions?: [string, string][];
		libraryColorMap?: Record<string, string>;
		searchMatchIds?: Set<string> | null;
		onNoteClick?: (path: string, name: string, highlightTerm?: string) => void;
		onClose?: () => void;
	} = $props();

	// ─── State ────────────────────────────────────────────────
	let canvasEl: HTMLCanvasElement;
	let ctx: CanvasRenderingContext2D | null = null;
	let width = $state(0);
	let height = $state(0);
	let destroyed = false;
	let animFrame = 0;

	// Simulation
	let simNodes: SimNode[] = [];
	let simLinks: SimLink[] = [];
	let simulation: d3.Simulation<SimNode, SimLink> | null = null;

	// Interaction
	let panX = 0, panY = 0, zoom = 1;
	let isPanning = false;
	let panStartX = 0, panStartY = 0;
	let hoveredNode: SimNode | null = null;
	let hoveredLink: SimLink | null = null;

	// Living Link enrichment map: "source::target" → link data
	let linkEnrichment = new Map<string, { weight: number; confidence: string; annotation: string; traversalCount: number; status: string }>();

	// Search
	let searchVisible = $state(false);
	let searchQuery = $state('');
	let searchScope = $state<'all' | 'title' | 'content' | 'tag' | 'property' | 'semantic'>('all');
	let searchResults = $state<SearchMatch[]>([]);
	let searchIdx = $state(0);
	let showChips = $state(false);
	let showHistory = $state(false);
	let historyItems = $state<{ query: string; timestamp: number }[]>([]);
	let searchMatchSet = $state<Set<string>>(new Set());

	// Settings — persisted across remounts via module-level storage
	const _defaults = { showLegend: true, linkStrokeMul: 1.0, linkOpacity: 0.6, arrowSize: 8 };
	const _saved = (globalThis as any).__sight2Settings ?? { ..._defaults };
	let showLegend = $state(_saved.showLegend);
	let settingsVisible = $state(false);
	let linkStrokeMul = $state(_saved.linkStrokeMul);
	let linkOpacity = $state(_saved.linkOpacity);
	let arrowSize = $state(_saved.arrowSize);

	// Save settings on change so they survive remounts
	function persistSettings() {
		(globalThis as any).__sight2Settings = { showLegend, linkStrokeMul, linkOpacity, arrowSize };
	}

	// Search match categories map: nodeId → categories[]
	let searchMatchCats = $state<Map<string, string[]>>(new Map());

	// Neighborhood highlight: click a node to see its connections
	let selectedNode: SimNode | null = null;
	let neighborIds = new Set<string>();

	// Search target node IDs (from [[X]] in structured queries)
	let searchTargetIds = new Set<string>();

	// SightPanel
	let panelVisible = $state(false);

	const isRTL = $derived($dir === 'rtl');

	// Syntax chips (reactive to locale)
	const syntaxChips = $derived.by(() => {
		const _locale = $t('searchHub.linksTo');
		const ops = getSearchOps();
		return [
			{ label: 'linksTo', syntax: (ops?.linksTo ?? 'links to') + ' [[' },
			{ label: 'linksFrom', syntax: (ops?.linksFrom ?? 'links from') + ' [[' },
			{ label: 'mutual', syntax: (ops?.mutual ?? 'mutual') + ' [[' },
			{ label: 'mentions', syntax: (ops?.mentions ?? 'mentions') + ' [[' },
			{ label: 'orphans', syntax: ops?.orphans ?? 'orphans' },
			{ label: 'linksAll', syntax: (ops?.linksAll ?? 'links all') + ' [[' },
			{ label: 'supports', syntax: (ops?.supports ?? 'supports') + ' [[' },
			{ label: 'contradicts', syntax: (ops?.contradicts ?? 'contradicts') + ' [[' },
			{ label: 'causes', syntax: (ops?.causes ?? 'causes') + ' [[' },
			{ label: 'exemplifies', syntax: (ops?.exemplifies ?? 'exemplifies') + ' [[' },
			{ label: 'generalizes', syntax: (ops?.generalizes ?? 'generalizes') + ' [[' },
			{ label: 'derivesFrom', syntax: (ops?.derivesFrom ?? 'derives from') + ' [[' },
			{ label: 'partOf', syntax: (ops?.partOf ?? 'part of') + ' [[' },
			{ label: 'tag', syntax: '#' },
			{ label: 'property', syntax: 'key=value' },
			{ label: 'scope', syntax: (ops?.scope ?? 'in') + ':' },
		];
	});

	// ─── Build Simulation Data ────────────────────────────────
	function buildSimData() {
		const nodeMap = new Map<string, SimNode>();
		simNodes = nodes.map(n => {
			const c = centrality.get(n.id) ?? 0;
			const libColor = libraryColorMap[n.libraryName] ?? '#94a3b8';
			const sn: SimNode = {
				id: n.id,
				name: n.name,
				path: n.path,
				libraryName: n.libraryName,
				centrality: c,
				libraryColor: libColor,
				r: Math.max(3, 3 + c * 18),
				maturity: (n as any).maturity,
				linkCount: n.linkCount,
				stratum: (n as any).stratum,
				x: (Math.random() - 0.5) * width * 0.8,
				y: (Math.random() - 0.5) * height * 0.8,
			};
			nodeMap.set(n.id, sn);
			return sn;
		});

		simLinks = [];
		for (const l of links) {
			const src = nodeMap.get(l.source);
			const tgt = nodeMap.get(l.target);
			if (src && tgt && src !== tgt) {
				const key = `${src.name.toLowerCase()}::${tgt.name.toLowerCase()}`;
				const enriched = linkEnrichment.get(key);
				simLinks.push({
					source: src,
					target: tgt,
					linkType: l.linkType || enriched?.status === 'active' ? (l.linkType || 'relates') : undefined,
					weight: enriched?.weight ?? 1.0,
					confidence: enriched?.confidence ?? 'hypothesis',
					annotation: enriched?.annotation ?? '',
					traversalCount: enriched?.traversalCount ?? 0,
					status: enriched?.status ?? 'active',
				});
			}
		}
	}

	// ─── Load Living Link Enrichment from note_links ──────────
	async function loadLinkEnrichment() {
		try {
			const stats: any = await invoke('constellation_link_stats');
			if (stats?.sample_links) {
				for (const link of stats.sample_links) {
					const key = `${link.source?.toLowerCase()}::${link.target?.toLowerCase()}`;
					linkEnrichment.set(key, {
						weight: link.weight ?? 1.0,
						confidence: link.confidence ?? 'hypothesis',
						annotation: link.annotation ?? '',
						traversalCount: link.traversal_count ?? 0,
						status: 'active',
					});
				}
			}
		} catch {}
	}

	// ─── Gravity-Well Layout ──────────────────────────────────
	// Position = centrality (distance from center) × library (angular sector).
	// No community detection. Libraries are the user's own organization.
	function computeGravityWellLayout() {
		if (simNodes.length === 0) return;

		const maxR = Math.min(width, height) * 0.45;

		// ── 1. Assign rings by centrality percentile ──
		const sorted = [...simNodes].sort((a, b) => b.centrality - a.centrality);
		const n = sorted.length;
		const ringRadii = [0, maxR * 0.27, maxR * 0.56, maxR * 0.88];

		const nodeRings = new Map<string, number>();
		sorted.forEach((node, i) => {
			const pct = i / n;
			nodeRings.set(node.id, pct < 0.05 ? 0 : pct < 0.15 ? 1 : pct < 0.35 ? 2 : 3);
		});

		// ── 2. Library sectors (user's own organization) ──
		const libraryNames = [...new Set(simNodes.map(n => n.libraryName))].sort();
		const numLibs = Math.max(libraryNames.length, 1);
		const sectorWidth = (Math.PI * 2) / numLibs;
		const libIndex = new Map<string, number>();
		libraryNames.forEach((name, i) => libIndex.set(name, i));

		// ── 3. Group by (ring, library) ──
		const groups = new Map<string, SimNode[]>();
		for (const node of simNodes) {
			const ring = nodeRings.get(node.id) ?? 3;
			const lib = libIndex.get(node.libraryName) ?? 0;
			const key = `${ring}:${lib}`;
			if (!groups.has(key)) groups.set(key, []);
			groups.get(key)!.push(node);
		}

		// ── 4. Position each node ──
		for (const [key, members] of groups) {
			const [ringStr, libStr] = key.split(':');
			const ring = parseInt(ringStr);
			const lib = parseInt(libStr);
			const baseRadius = ringRadii[ring];
			const baseAngle = (lib / numLibs) * Math.PI * 2;

			members.forEach((node, i) => {
				const angleOffset = (i / Math.max(members.length, 1)) * sectorWidth * 0.8;
				const angle = baseAngle + sectorWidth * 0.1 + angleOffset;
				const jitter = (Math.random() - 0.5) * baseRadius * 0.12;
				const radius = ring === 0
					? maxR * 0.02 + (i / Math.max(members.length, 1)) * maxR * 0.06
					: baseRadius + jitter;
				node.x = radius * Math.cos(angle);
				node.y = radius * Math.sin(angle);
			});
		}

		// ── 5. Collision avoidance for small datasets ──
		if (simNodes.length <= 1500) {
			const boundaryForce = () => {
				const force = (alpha: number) => {
					for (const node of simNodes) {
						const x = node.x ?? 0, y = node.y ?? 0;
						const dist = Math.sqrt(x * x + y * y);
						const limit = maxR - (node.r ?? 3);
						if (dist > limit) {
							const scale = limit / dist;
							node.x = x * scale;
							node.y = y * scale;
							node.vx = (node.vx ?? 0) * 0.1;
							node.vy = (node.vy ?? 0) * 0.1;
						}
					}
				};
				force.initialize = () => {};
				return force;
			};

			simulation = d3.forceSimulation(simNodes)
				.force('collide', d3.forceCollide<SimNode>().radius(d => d.r + 1.5).strength(0.5))
				.force('boundary', boundaryForce())
				.alphaDecay(0.15)
				.stop();
			for (let i = 0; i < 15; i++) simulation.tick();
		}

		requestDraw();
	}

	// ─── Search ───────────────────────────────────────────────
	async function executeSearch() {
		if (!searchQuery.trim()) { searchResults = []; searchIdx = 0; searchMatchSet = new Set(); searchMatchCats = new Map(); requestDraw(); return; }
		addSearchHistory(searchQuery);
		historyItems = readSearchHistory();

		const cleanQ = stripInvisibleChars(searchQuery);
		const ops = getSearchOps();
		const canonicalized = canonicalizeSearchQuery(cleanQ, ops);
		const isAdvanced = hasAdvancedSyntaxMultilingual(canonicalized, ops);

		const nodeMap = new Map(simNodes.map(n => [n.id, n]));
		const matches: SearchMatch[] = [];
		searchTargetIds = new Set<string>();

		if (isAdvanced) {
			// ── Structured query: links to/from, orphans, mutual, cognitive types ──
			try {
				const req = parseSearchQuery(canonicalized);
				const results = await constellationSearch(req);
				for (const r of results) {
					const id = r.name.toLowerCase();
					const node = nodeMap.get(id);
					if (!node) continue;
					const CAT_MAP: Record<string, string> = { wikilink: 'W', title: 'T', content: 'C', tag: '#', property: 'P', semantic: 'S', hybrid: 'C' };
					const cat = CAT_MAP[r.match_type] ?? r.match_type.charAt(0).toUpperCase();
					matches.push({ node, matchType: cat, matchCategories: [cat] });
				}

				// Also add the TARGET nodes (from [[X]]) to the match set
				const targetRe = /\[\[([^\]]+)\]\]/g;
				let tm;
				while ((tm = targetRe.exec(canonicalized)) !== null) {
					const targetId = tm[1].toLowerCase();
					searchTargetIds.add(targetId);
					const targetNode = nodeMap.get(targetId);
					if (targetNode && !matches.find(m => m.node.id === targetId)) {
						matches.push({ node: targetNode, matchType: 'target', matchCategories: ['T'] });
					}
					}
			} catch {}
		} else {
			// ── Free text: universal search across all categories ──
			const q = searchQuery.toLowerCase();
			const titleMatchIds = new Set<string>();
			const contentMatchIds = new Set<string>();
			const tagMatchIds = new Set<string>();
			const propertyMatchIds = new Set<string>();
			const semanticMatchIds = new Set<string>();

			// Local title matches
			if (searchScope === 'all' || searchScope === 'title') {
				for (const n of simNodes) {
					if (n.name.toLowerCase().includes(q)) titleMatchIds.add(n.id);
				}
			}

			// Backend search
			if (searchScope !== 'title') {
				try {
					let qEmbed: number[] | null = null;
					if ((searchScope === 'all' || searchScope === 'semantic') && get(appSettings).enabledFeatures?.semanticSearch) {
						try { qEmbed = await embedText(canonicalized); } catch {}
					}
					const resp: any = await universalSearch(canonicalized, qEmbed, 200);
					if (searchScope === 'all' || searchScope === 'content') {
						for (const r of resp?.contents ?? []) contentMatchIds.add(r.name.toLowerCase());
					}
					if (searchScope === 'all' || searchScope === 'tag') {
						for (const r of resp?.tags ?? []) tagMatchIds.add(r.name.toLowerCase());
					}
					if (searchScope === 'all' || searchScope === 'property') {
						for (const r of resp?.properties ?? []) propertyMatchIds.add(r.name.toLowerCase());
					}
					if (searchScope === 'all' || searchScope === 'semantic') {
						for (const r of resp?.semantic ?? []) semanticMatchIds.add(r.name.toLowerCase());
					}
					if (searchScope === 'all') {
						for (const r of resp?.titles ?? []) titleMatchIds.add(r.name.toLowerCase());
					}
				} catch {}
			}

			// Classify
			const allIds = new Set([...titleMatchIds, ...contentMatchIds, ...tagMatchIds, ...propertyMatchIds, ...semanticMatchIds]);
			for (const id of allIds) {
				const node = nodeMap.get(id);
				if (!node) continue;
				const cats: string[] = [];
				if (titleMatchIds.has(id)) cats.push('T');
				if (contentMatchIds.has(id)) cats.push('C');
				if (tagMatchIds.has(id)) cats.push('#');
				if (propertyMatchIds.has(id)) cats.push('P');
				if (semanticMatchIds.has(id)) cats.push('S');
				matches.push({ node, matchType: cats.join('·') || 'match', matchCategories: cats });
			}
		}

		matches.sort((a, b) => (b.matchCategories?.length ?? 0) - (a.matchCategories?.length ?? 0));
		searchResults = matches;
		searchIdx = 0;
		searchMatchSet = new Set(matches.map(m => m.node.id));
		const catMap = new Map<string, string[]>();
		for (const m of matches) catMap.set(m.node.id, m.matchCategories);
		searchMatchCats = catMap;

		if (searchResults.length > 0) centerOnSearchResult();
		requestDraw();
	}

	function centerOnSearchResult() {
		const match = searchResults[searchIdx];
		if (!match) return;
		const nx = match.node.x ?? 0, ny = match.node.y ?? 0;
		zoom = 3;
		panX = -nx * zoom;
		panY = -ny * zoom;
		requestDraw();
	}

	function nextSearchResult() {
		if (searchResults.length === 0) return;
		searchIdx = (searchIdx + 1) % searchResults.length;
		centerOnSearchResult();
	}

	function prevSearchResult() {
		if (searchResults.length === 0) return;
		searchIdx = (searchIdx - 1 + searchResults.length) % searchResults.length;
		centerOnSearchResult();
	}

	function resetSearch() {
		searchQuery = '';
		searchResults = [];
		searchIdx = 0;
		searchMatchSet = new Set();
		searchMatchCats = new Map();
		searchTargetIds = new Set();
		requestDraw();
	}

	function closeSearch() {
		resetSearch();
		showChips = false;
		showHistory = false;
		searchVisible = false;
	}

	function insertChipSyntax(syntax: string) {
		searchQuery = searchQuery ? searchQuery + ' ' + syntax : syntax;
		showChips = false;
	}

	function selectHistory(item: { query: string }) {
		searchQuery = item.query;
		showHistory = false;
		executeSearch();
	}

	// ─── Canvas Rendering ─────────────────────────────────────
	function requestDraw() {
		if (animFrame) return;
		animFrame = requestAnimationFrame(() => {
			animFrame = 0;
			draw();
		});
	}

	function draw() {
		if (!ctx || destroyed) return;
		const w = width, h = height;
		ctx.clearRect(0, 0, w, h);

		ctx.save();
		ctx.translate(w / 2 + panX, h / 2 + panY);
		ctx.scale(zoom, zoom);

		drawRadialGuides();
		drawLinks();
		drawNodes();
		drawSearchBadges();
		if (hoveredNode) drawHoverLabel(hoveredNode);
		if (hoveredLink) drawLinkAnnotation(hoveredLink);

		ctx.restore();
	}

	function drawRadialGuides() {
		const minDim = Math.min(width, height);
		const ringRadii = [minDim * 0.12, minDim * 0.25, minDim * 0.45];
		const guideColor = 'rgba(148, 163, 184, 0.08)';

		ctx!.strokeStyle = guideColor;
		ctx!.lineWidth = 0.5 / zoom;
		for (const r of ringRadii) {
			ctx!.beginPath();
			ctx!.arc(0, 0, r, 0, Math.PI * 2);
			ctx!.stroke();
		}

		ctx!.beginPath();
		ctx!.arc(0, 0, 2 / zoom, 0, Math.PI * 2);
		ctx!.fillStyle = 'rgba(148, 163, 184, 0.2)';
		ctx!.fill();
	}

	function drawLinks() {
		const hw = width / 2 / zoom, hh = height / 2 / zoom;
		const vpLeft = -panX / zoom - hw - 50, vpRight = -panX / zoom + hw + 50;
		const vpTop = -panY / zoom - hh - 50, vpBottom = -panY / zoom + hh + 50;
		const hasSearch = searchMatchSet.size > 0;

		for (const link of simLinks) {
			const src = link.source as SimNode;
			const tgt = link.target as SimNode;
			const sx = src.x ?? 0, sy = src.y ?? 0;
			const tx = tgt.x ?? 0, ty = tgt.y ?? 0;

			if ((sx < vpLeft && tx < vpLeft) || (sx > vpRight && tx > vpRight) ||
				(sy < vpTop && ty < vpTop) || (sy > vpBottom && ty > vpBottom)) continue;

			// Classify link relevance to search/neighborhood
			const srcMatch = hasSearch && searchMatchSet.has(src.id);
			const tgtMatch = hasSearch && searchMatchSet.has(tgt.id);
			const searchHighlight = srcMatch && tgtMatch;        // BOTH endpoints are matches → bold
			const searchPartial = srcMatch || tgtMatch;           // one endpoint is a match → visible
			const searchDim = hasSearch && !searchPartial;        // neither → dim
			const neighborDim = selectedNode != null && src.id !== selectedNode.id && tgt.id !== selectedNode.id;

			const typed = link.linkType && link.linkType !== 'relates' && LINK_TYPE_COLORS[link.linkType];

			// Search-highlighted links: colored by direction relative to target
			// Green = inward (result → target), Red = outward (target → result)
			let color: string;
			if (searchHighlight && searchTargetIds.size > 0) {
				const srcIsTarget = searchTargetIds.has(src.id);
				const tgtIsTarget = searchTargetIds.has(tgt.id);
				color = tgtIsTarget ? '#16a34a' : srcIsTarget ? '#ef4444' : '#f59e0b'; // green inward, red outward, amber other
			} else {
				color = typed ? LINK_TYPE_COLORS[link.linkType!] : '#94a3b8';
			}

			const w = link.weight ?? 1.0;
			const baseWidth = (typed ? Math.max(0.8, Math.min(4, w * 0.5)) : 0.7) * linkStrokeMul
				* (searchHighlight ? 3.0 : 1.0);  // 3× thicker for search-matched links
			const conf = CONFIDENCE_STYLE[link.confidence ?? 'hypothesis'] ?? CONFIDENCE_STYLE.hypothesis;

			const isDormant = link.status === 'dormant';
			const baseAlpha = searchHighlight ? 1.0 : searchDim ? 0.06 : neighborDim ? 0.06 : isDormant ? 0.27 : linkOpacity;
			const alphaHex = Math.round(baseAlpha * 255).toString(16).padStart(2, '0');

			// All lines solid — no dashes
			ctx!.beginPath();
			ctx!.moveTo(sx, sy);
			ctx!.lineTo(tx, ty);
			ctx!.strokeStyle = color + alphaHex;
			ctx!.lineWidth = (baseWidth * conf.widthMul) / zoom;
			ctx!.stroke();

			// Direction arrows along the link line (ALL links, not just typed)
			if (baseAlpha > 0.1) {
				const dx = tx - sx, dy = ty - sy;
				const len = Math.sqrt(dx * dx + dy * dy);
				if (len > 20) {
					const ux = dx / len, uy = dy / len;
					const as = arrowSize / zoom;
					// Place arrows at 30%, 55%, 80% of the link length
					const positions = len > 80 ? [0.3, 0.55, 0.8] : len > 40 ? [0.35, 0.65] : [0.5];
					for (const t of positions) {
						const ax = sx + dx * t, ay = sy + dy * t;
						ctx!.beginPath();
						ctx!.moveTo(ax + ux * as, ay + uy * as);
						ctx!.lineTo(ax - uy * as * 0.5 - ux * as * 0.3, ay + ux * as * 0.5 - uy * as * 0.3);
						ctx!.lineTo(ax + uy * as * 0.5 - ux * as * 0.3, ay - ux * as * 0.5 - uy * as * 0.3);
						ctx!.closePath();
						ctx!.fillStyle = color + alphaHex;
						ctx!.fill();
					}
				}
			}
		}
	}

	function drawNodes() {
		const hw = width / 2 / zoom, hh = height / 2 / zoom;
		const vpLeft = -panX / zoom - hw - 20, vpRight = -panX / zoom + hw + 20;
		const vpTop = -panY / zoom - hh - 20, vpBottom = -panY / zoom + hh + 20;
		const hasSearch = searchMatchSet.size > 0;
		const currentMatch = searchResults[searchIdx]?.node;

		for (const n of simNodes) {
			const x = n.x ?? 0, y = n.y ?? 0;
			if (x < vpLeft || x > vpRight || y < vpTop || y > vpBottom) continue;

			const isMatch = hasSearch && searchMatchSet.has(n.id);
			const isCurrent = currentMatch?.id === n.id;
			const isNeighbor = selectedNode && (n === selectedNode || neighborIds.has(n.id));
			const maturityAlpha: Record<string, number> = { seed: 0.5, sapling: 0.7, evergreen: 0.9, canonical: 1.0, wilting: 0.4 };
			// Dim logic: search dims non-matches, neighborhood dims non-neighbors
			const baseAlpha = maturityAlpha[n.maturity ?? 'seed'] ?? 0.6;
			const alpha = hasSearch ? (isMatch ? 1.0 : 0.15)
				: selectedNode ? (isNeighbor ? 1.0 : 0.12)
				: baseAlpha;

			// Bridge emphasis
			if (n.centrality > 0.4 && alpha > 0.3) {
				ctx!.beginPath();
				ctx!.arc(x, y, n.r + 4 / zoom, 0, Math.PI * 2);
				ctx!.fillStyle = n.libraryColor + '33';
				ctx!.fill();
			}

			// Search match highlight ring
			if (isMatch) {
				ctx!.beginPath();
				ctx!.arc(x, y, n.r + (isCurrent ? 6 : 3) / zoom, 0, Math.PI * 2);
				ctx!.strokeStyle = isCurrent ? '#f59e0b' : '#3b82f6';
				ctx!.lineWidth = (isCurrent ? 3 : 2) / zoom;
				ctx!.stroke();
			}

			// Selected node highlight
			if (n === selectedNode) {
				ctx!.beginPath();
				ctx!.arc(x, y, n.r + 5 / zoom, 0, Math.PI * 2);
				ctx!.strokeStyle = '#f59e0b';
				ctx!.lineWidth = 2.5 / zoom;
				ctx!.stroke();
			}

			// Node circle — colored by library
			ctx!.globalAlpha = alpha;
			ctx!.beginPath();
			ctx!.arc(x, y, n.r, 0, Math.PI * 2);
			ctx!.fillStyle = n.libraryColor;
			ctx!.fill();
			ctx!.globalAlpha = 1.0;

			// Maturity border
			if (n.maturity && MATURITY_COLORS[n.maturity] && (!hasSearch || isMatch)) {
				ctx!.strokeStyle = MATURITY_COLORS[n.maturity];
				ctx!.lineWidth = 1.5 / zoom;
				ctx!.stroke();
			}

			// Bridge ring
			if (n.centrality > 0.5 && (!hasSearch || isMatch)) {
				ctx!.strokeStyle = '#7c3aed';
				ctx!.lineWidth = 2 / zoom;
				ctx!.stroke();
			}

			// Hover highlight
			if (hoveredNode === n) {
				ctx!.strokeStyle = '#1e1b4b';
				ctx!.lineWidth = 2.5 / zoom;
				ctx!.stroke();
			}
		}
	}

	function drawSearchBadges() {
		if (searchMatchCats.size === 0) return;
		const hw = width / 2 / zoom, hh = height / 2 / zoom;
		const vpLeft = -panX / zoom - hw - 40, vpRight = -panX / zoom + hw + 40;
		const vpTop = -panY / zoom - hh - 40, vpBottom = -panY / zoom + hh + 40;
		const currentMatch = searchResults[searchIdx]?.node;

		for (const n of simNodes) {
			const cats = searchMatchCats.get(n.id);
			if (!cats || cats.length === 0) continue;
			const x = n.x ?? 0, y = n.y ?? 0;
			if (x < vpLeft || x > vpRight || y < vpTop || y > vpBottom) continue;

			const isCurrent = currentMatch?.id === n.id;

			// Draw category badges below the node
			const badgeY = y + n.r + 6 / zoom;
			const badgeH = 16 / zoom;
			const badgeW = 16 / zoom;
			const gap = 3 / zoom;
			const totalW = cats.length * (badgeW + gap) - gap;
			let bx = x - totalW / 2;

			for (const cat of cats) {
				const col = CAT_COLORS[cat] ?? '#94a3b8';
				ctx!.fillStyle = col;
				ctx!.beginPath();
				ctx!.roundRect(bx, badgeY, badgeW, badgeH, 3 / zoom);
				ctx!.fill();
				// Badge letter
				ctx!.fillStyle = '#ffffff';
				ctx!.font = `bold ${10 / zoom}px system-ui, sans-serif`;
				ctx!.textAlign = 'center';
				ctx!.textBaseline = 'middle';
				ctx!.fillText(cat, bx + badgeW / 2, badgeY + badgeH / 2);
				bx += badgeW + gap;
			}

			// Pointer arrow for current match — large amber triangle above the node
			if (isCurrent) {
				const arrW = 20 / zoom;
				const arrH = 18 / zoom;
				const tipY = y - n.r - 5 / zoom;
				const topY = tipY - arrH;
				// Draw twice: shadow first, then fill — ensures visibility
				ctx!.beginPath();
				ctx!.moveTo(x, tipY);
				ctx!.lineTo(x - arrW / 2, topY);
				ctx!.lineTo(x + arrW / 2, topY);
				ctx!.closePath();
				ctx!.fillStyle = '#f59e0b';
				ctx!.fill();
				ctx!.strokeStyle = '#ffffff';
				ctx!.lineWidth = 2 / zoom;
				ctx!.stroke();
				// Small dot at tip for precision
				ctx!.beginPath();
				ctx!.arc(x, tipY, 2 / zoom, 0, Math.PI * 2);
				ctx!.fillStyle = '#f59e0b';
				ctx!.fill();
			}
		}
	}

	function drawHoverLabel(n: SimNode) {
		const x = n.x ?? 0, y = n.y ?? 0;
		const label = n.name;
		ctx!.font = `${12 / zoom}px system-ui, sans-serif`;
		const metrics = ctx!.measureText(label);
		const lw = metrics.width + 10 / zoom;
		const lh = 18 / zoom;
		const lx = x - lw / 2;
		const ly = y - n.r - lh - 6 / zoom;

		ctx!.fillStyle = 'rgba(30,30,40,0.9)';
		ctx!.beginPath();
		ctx!.roundRect(lx, ly, lw, lh, 3 / zoom);
		ctx!.fill();
		ctx!.fillStyle = '#ffffff';
		ctx!.textAlign = 'center';
		ctx!.textBaseline = 'middle';
		ctx!.fillText(label, x, ly + lh / 2);
	}

	function drawLinkAnnotation(link: SimLink) {
		if (!link.annotation) return;
		const src = link.source as SimNode;
		const tgt = link.target as SimNode;
		const mx = ((src.x ?? 0) + (tgt.x ?? 0)) / 2;
		const my = ((src.y ?? 0) + (tgt.y ?? 0)) / 2;

		const text = `${link.linkType ?? 'relates'}: ${link.annotation}`;
		ctx!.font = `${10 / zoom}px system-ui, sans-serif`;
		const tw = Math.min(ctx!.measureText(text).width + 10 / zoom, 200 / zoom);
		const th = 16 / zoom;

		ctx!.fillStyle = 'rgba(30,30,40,0.85)';
		ctx!.beginPath();
		ctx!.roundRect(mx - tw / 2, my - th - 4 / zoom, tw, th, 3 / zoom);
		ctx!.fill();
		ctx!.fillStyle = '#ffffff';
		ctx!.textAlign = 'center';
		ctx!.textBaseline = 'middle';
		ctx!.fillText(text.length > 40 ? text.slice(0, 40) + '...' : text, mx, my - th / 2 - 4 / zoom);
	}

	// ─── Interaction ──────────────────────────────────────────
	function onMouseDown(e: MouseEvent) {
		isPanning = true;
		panStartX = e.clientX - panX;
		panStartY = e.clientY - panY;
	}

	function onMouseMove(e: MouseEvent) {
		if (isPanning) {
			panX = e.clientX - panStartX;
			panY = e.clientY - panStartY;
			requestDraw();
			return;
		}
		const rect = canvasEl.getBoundingClientRect();
		const mx = (e.clientX - rect.left - rect.width / 2 - panX) / zoom;
		const my = (e.clientY - rect.top - rect.height / 2 - panY) / zoom;

		hoveredNode = null;
		hoveredLink = null;
		for (const n of simNodes) {
			const dx = (n.x ?? 0) - mx, dy = (n.y ?? 0) - my;
			if (dx * dx + dy * dy < (n.r + 4) * (n.r + 4)) {
				hoveredNode = n;
				break;
			}
		}

		if (!hoveredNode) {
			for (const link of simLinks) {
				if (!link.annotation) continue;
				const src = link.source as SimNode;
				const tgt = link.target as SimNode;
				const sx = src.x ?? 0, sy = src.y ?? 0;
				const tx = tgt.x ?? 0, ty = tgt.y ?? 0;
				const dx = tx - sx, dy = ty - sy;
				const len2 = dx * dx + dy * dy;
				if (len2 === 0) continue;
				const t = Math.max(0, Math.min(1, ((mx - sx) * dx + (my - sy) * dy) / len2));
				const px = sx + t * dx, py = sy + t * dy;
				const dist = Math.sqrt((mx - px) * (mx - px) + (my - py) * (my - py));
				if (dist < 6 / zoom) {
					hoveredLink = link;
					break;
				}
			}
		}

		canvasEl.style.cursor = hoveredNode ? 'pointer' : hoveredLink ? 'help' : 'grab';
		requestDraw();
	}

	function onMouseUp() { isPanning = false; }

	function onWheel(e: WheelEvent) {
		e.preventDefault();
		zoom = Math.max(0.1, Math.min(5, zoom + (e.deltaY > 0 ? -0.08 : 0.08)));
		requestDraw();
	}

	function onClick(e: MouseEvent) {
		if (hoveredNode) {
			if (e.detail === 2 && onNoteClick) {
				// Double-click: open the note
				const hl = searchQuery.trim() || undefined;
				onNoteClick(hoveredNode.path, hoveredNode.name, hl);
			} else {
				// Single-click: neighborhood highlight — show connected nodes
				selectedNode = hoveredNode;
				neighborIds = new Set<string>();
				for (const link of simLinks) {
					const src = link.source as SimNode;
					const tgt = link.target as SimNode;
					if (src.id === hoveredNode.id) neighborIds.add(tgt.id);
					if (tgt.id === hoveredNode.id) neighborIds.add(src.id);
				}
				requestDraw();
			}
		} else {
			// Click empty space: clear neighborhood highlight
			if (selectedNode) {
				selectedNode = null;
				neighborIds = new Set();
				requestDraw();
			}
		}
	}

	function fitToScreen() {
		if (simNodes.length === 0) return;
		let minX = Infinity, maxX = -Infinity, minY = Infinity, maxY = -Infinity;
		for (const n of simNodes) {
			minX = Math.min(minX, (n.x ?? 0) - n.r);
			maxX = Math.max(maxX, (n.x ?? 0) + n.r);
			minY = Math.min(minY, (n.y ?? 0) - n.r);
			maxY = Math.max(maxY, (n.y ?? 0) + n.r);
		}
		const rangeX = maxX - minX || 1;
		const rangeY = maxY - minY || 1;
		zoom = Math.min(width / rangeX, height / rangeY) * 0.85;
		zoom = Math.max(0.1, Math.min(3, zoom));
		panX = 0; panY = 0;
		requestDraw();
	}

	// ─── Lifecycle ────────────────────────────────────────────
	onMount(async () => {
		const rect = canvasEl.parentElement?.getBoundingClientRect();
		width = rect?.width ?? 800;
		height = rect?.height ?? 600;
		canvasEl.width = width * devicePixelRatio;
		canvasEl.height = height * devicePixelRatio;
		ctx = canvasEl.getContext('2d');
		if (ctx) ctx.scale(devicePixelRatio, devicePixelRatio);

		buildSimData();
		computeGravityWellLayout();
		fitToScreen();

		loadLinkEnrichment().then(() => {
			for (const link of simLinks) {
				const src = link.source as SimNode;
				const tgt = link.target as SimNode;
				const key = `${src.name.toLowerCase()}::${tgt.name.toLowerCase()}`;
				const enriched = linkEnrichment.get(key);
				if (enriched) {
					link.weight = enriched.weight;
					link.confidence = enriched.confidence;
					link.annotation = enriched.annotation;
					link.traversalCount = enriched.traversalCount;
					link.status = enriched.status;
				}
			}
			requestDraw();
		});
	});

	onDestroy(() => {
		destroyed = true;
		if (animFrame) cancelAnimationFrame(animFrame);
		simulation?.stop();
	});
</script>

<div class="sight2-root" dir={isRTL ? 'rtl' : 'ltr'}>
	<!-- Header -->
	<div class="sight2-header">
		<svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M2 12s3-7 10-7 10 7 10 7-3 7-10 7-10-7-10-7Z"/><circle cx="12" cy="12" r="3"/></svg>
		<span class="sight2-title">{$t('lens.title') || 'Constellation Sight'}</span>
		<span class="sight2-stats">{simNodes.length} {$t('lens.nodes') || 'nodes'} · {simLinks.length} {$t('lens.links') || 'links'}</span>
		<div class="sight2-toolbar">
			<!-- Search toggle -->
			<button class="sight2-btn" class:active={searchVisible} onclick={() => { searchVisible = !searchVisible; if (!searchVisible) closeSearch(); }} title={$t('layout.search') || 'Search'}>
				<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="11" cy="11" r="8"/><path d="m21 21-4.3-4.3"/></svg>
			</button>
			<!-- Fit to screen -->
			<button class="sight2-btn" onclick={fitToScreen} title={$t('lens.fitToScreen') || 'Fit to screen'}>
				<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M8 3H5a2 2 0 0 0-2 2v3"/><path d="M21 8V5a2 2 0 0 0-2-2h-3"/><path d="M3 16v3a2 2 0 0 0 2 2h3"/><path d="M16 21h3a2 2 0 0 0 2-2v-3"/></svg>
			</button>
			<!-- Panel toggle -->
			<button class="sight2-btn" class:active={panelVisible} onclick={() => panelVisible = !panelVisible} title={$t('sightPanel.overview') || 'Analytics'}>
				<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><rect x="3" y="3" width="7" height="7" rx="1"/><rect x="14" y="3" width="7" height="4" rx="1"/><rect x="14" y="10" width="7" height="11" rx="1"/><rect x="3" y="13" width="7" height="8" rx="1"/></svg>
			</button>
			<!-- Settings toggle -->
			<button class="sight2-btn" class:active={settingsVisible} onclick={() => settingsVisible = !settingsVisible} title={$t('ribbon.settings') || 'Settings'}>
				<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="12" cy="12" r="3"/><path d="M12.22 2h-.44a2 2 0 0 0-2 2v.18a2 2 0 0 1-1 1.73l-.43.25a2 2 0 0 1-2 0l-.15-.08a2 2 0 0 0-2.73.73l-.22.38a2 2 0 0 0 .73 2.73l.15.1a2 2 0 0 1 1 1.72v.51a2 2 0 0 1-1 1.74l-.15.09a2 2 0 0 0-.73 2.73l.22.38a2 2 0 0 0 2.73.73l.15-.08a2 2 0 0 1 2 0l.43.25a2 2 0 0 1 1 1.73V20a2 2 0 0 0 2 2h.44a2 2 0 0 0 2-2v-.18a2 2 0 0 1 1-1.73l.43-.25a2 2 0 0 1 2 0l.15.08a2 2 0 0 0 2.73-.73l.22-.39a2 2 0 0 0-.73-2.73l-.15-.08a2 2 0 0 1-1-1.74v-.5a2 2 0 0 1 1-1.74l.15-.09a2 2 0 0 0 .73-2.73l-.22-.38a2 2 0 0 0-2.73-.73l-.15.08a2 2 0 0 1-2 0l-.43-.25a2 2 0 0 1-1-1.73V4a2 2 0 0 0-2-2z"/></svg>
			</button>
		</div>
		<button class="sight2-close" onclick={() => onClose?.()}>×</button>
	</div>

	<!-- Body -->
	<div class="sight2-body">
		<div class="sight2-canvas-wrap">
			<!-- Search bar -->
			{#if searchVisible}
				<div class="sight2-search">
					<div class="sight2-search-scope">
						<button class:active={searchScope === 'all'} onclick={() => searchScope = 'all'}>{$t("lens.scopeAll") || "All"}</button>
						<button class:active={searchScope === 'title'} onclick={() => searchScope = 'title'}>{$t("searchHub.titles") || "Title"}</button>
						<button class:active={searchScope === 'content'} onclick={() => searchScope = 'content'}>{$t("searchHub.contents") || "Content"}</button>
						<button class:active={searchScope === 'tag'} onclick={() => searchScope = 'tag'}>{$t("searchHub.tags") || "Tags"}</button>
						<button class:active={searchScope === 'property'} onclick={() => searchScope = 'property'}>{$t("searchHub.properties") || "Props"}</button>
						<button class:active={searchScope === 'semantic'} onclick={() => searchScope = 'semantic'}>{$t("searchHub.semantic") || "Semantic"}</button>
					</div>
					<div class="sight2-search-input">
						<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="11" cy="11" r="8"/><path d="m21 21-4.3-4.3"/></svg>
						<input type="text" dir="auto"
							placeholder={$t('lens.searchAll') || 'Search... (Enter)'}
							bind:value={searchQuery}
							onfocus={() => { if (!searchQuery) { historyItems = readSearchHistory(); showHistory = true; } }}
							onblur={() => setTimeout(() => { showHistory = false; }, 200)}
							oninput={() => { showHistory = false; }}
							onkeydown={(e) => {
								if (e.key === 'Enter') { e.preventDefault(); searchResults.length > 0 ? (e.shiftKey ? prevSearchResult() : nextSearchResult()) : executeSearch(); }
								if (e.key === 'Escape') { closeSearch(); e.stopPropagation(); }
							}} />
						<button class="sight2-search-clear" onclick={resetSearch} title={$t('common.clear') || 'Clear'}>×</button>
						<button class="sight2-chips-btn" class:active={showChips} onclick={() => showChips = !showChips} title={$t('searchHub.syntaxHelpers') || 'Syntax'}>
							<svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="12" cy="12" r="1"/><circle cx="19" cy="12" r="1"/><circle cx="5" cy="12" r="1"/></svg>
						</button>
						<!-- Search history dropdown -->
						{#if showHistory && historyItems.length > 0 && !searchQuery}
							<div class="sight2-dropdown">
								{#each historyItems.slice(0, 8) as item}
									<button class="sight2-dropdown-item" onclick={() => selectHistory(item)} dir="auto">{item.query}</button>
								{/each}
							</div>
						{/if}
						<!-- Syntax chips dropdown -->
						{#if showChips}
							<div class="sight2-dropdown sight2-chips-grid">
								{#each syntaxChips as chip}
									<button class="sight2-chip" onclick={() => insertChipSyntax(chip.syntax)}>{$t(`searchHub.${chip.label}`)}</button>
								{/each}
							</div>
						{/if}
					</div>
					<!-- Results navigation -->
					{#if searchResults.length > 0}
						{@const currentMatch = searchResults[searchIdx]}
						{#each (currentMatch?.matchCategories ?? []) as cat}
							<span class="sight2-cat" style="background:{CAT_COLORS[cat] ?? '#94a3b8'}">{cat}</span>
						{/each}
						<span class="sight2-search-count">{searchIdx + 1}/{searchResults.length}</span>
						<button class="sight2-search-nav" onclick={prevSearchResult}>
							<svg width="10" height="10" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><polyline points="15 18 9 12 15 6"/></svg>
						</button>
						<button class="sight2-search-nav" onclick={nextSearchResult}>
							<svg width="10" height="10" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><polyline points="9 6 15 12 9 18"/></svg>
						</button>
					{:else if searchQuery}
						<span class="sight2-search-none">0</span>
					{/if}
				</div>
			{/if}

			<!-- Settings panel -->
			{#if settingsVisible}
				<div class="sight2-settings">
					<div class="sight2-settings-title">{$t("lens.display") || "Display"}</div>
					<label class="sight2-settings-row">
						<span>{$t("lens.legend") || "Legend"}</span>
						<button class:active={showLegend} onclick={() => { showLegend = !showLegend; persistSettings(); }}>{showLegend ? 'On' : 'Off'}</button>
					</label>
					<div class="sight2-settings-title" style="margin-top:4px">{$t('searchHub.linksTo') || 'Links'}</div>
					<label class="sight2-settings-slider">
						<span>{$t("lens.settingStroke") || "Stroke"}: {linkStrokeMul.toFixed(1)}×</span>
						<input type="range" min="0.5" max="4" step="0.25" bind:value={linkStrokeMul} oninput={() => { requestDraw(); persistSettings(); }} />
					</label>
					<label class="sight2-settings-slider">
						<span>{$t("lens.settingOpacity") || "Opacity"}: {Math.round(linkOpacity * 100)}%</span>
						<input type="range" min="0.1" max="1" step="0.05" bind:value={linkOpacity} oninput={() => { requestDraw(); persistSettings(); }} />
					</label>
					<label class="sight2-settings-slider">
						<span>{$t("lens.settingArrows") || "Arrows"}: {arrowSize}px</span>
						<input type="range" min="2" max="16" step="1" bind:value={arrowSize} oninput={() => { requestDraw(); persistSettings(); }} />
					</label>
				</div>
			{/if}

			<!-- Canvas -->
			<canvas bind:this={canvasEl} style="width:{width}px;height:{height}px"
				onmousedown={onMouseDown} onmousemove={onMouseMove} onmouseup={onMouseUp}
				onmouseleave={onMouseUp} onwheel={onWheel} onclick={onClick}>
			</canvas>

			<!-- Legend -->
			{#if showLegend}
				<div class="sight2-legend">
					<div class="sight2-legend-title">{$t('lens.legend') || 'Legend'}</div>
					<!-- Nodes -->
					<div class="sight2-legend-row">
						<span class="sight2-lg-dot sight2-lg-big"></span>
						<span>{$t("lens.largeNode") || "Large"} — {$t("lens.bridgeDesc") || "bridge / high centrality"}</span>
					</div>
					<div class="sight2-legend-row">
						<span class="sight2-lg-dot sight2-lg-small"></span>
						<span>{$t("lens.smallNode") || "Small"} — {$t("lens.peripheralDesc") || "peripheral"}</span>
					</div>
					<div class="sight2-legend-row">
						<span class="sight2-lg-dot" style="background:#a78bfa"></span>
						<span class="sight2-lg-dot" style="background:#34d399"></span>
						<span class="sight2-lg-dot" style="background:#60a5fa"></span>
						<span>{$t("lens.colorLibrary") || "Color = library"}</span>
					</div>
					<div class="sight2-legend-row">
						<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="#f59e0b" stroke-width="2"><circle cx="12" cy="12" r="9"/></svg>
						<span>{$t("lens.clickConnections") || "Click = show connections"}</span>
					</div>
					<div class="sight2-legend-divider"></div>
					<!-- Link types -->
					<div class="sight2-legend-title">{$t('searchHub.linksTo') || 'Link Types'}</div>
					<div class="sight2-legend-row">
						<svg width="20" height="4"><line x1="0" y1="2" x2="16" y2="2" stroke="#4A9EFF" stroke-width="2"/><polygon points="16,0 20,2 16,4" fill="#4A9EFF"/></svg>
						<span>{$t("lens.linkSupports") || "supports"}</span>
					</div>
					<div class="sight2-legend-row">
						<svg width="20" height="4"><line x1="0" y1="2" x2="16" y2="2" stroke="#FF4A4A" stroke-width="2"/><polygon points="16,0 20,2 16,4" fill="#FF4A4A"/></svg>
						<span>{$t("lens.linkContradicts") || "contradicts"}</span>
					</div>
					<div class="sight2-legend-row">
						<svg width="20" height="4"><line x1="0" y1="2" x2="16" y2="2" stroke="#FF8C42" stroke-width="2"/><polygon points="16,0 20,2 16,4" fill="#FF8C42"/></svg>
						<span>{$t("lens.linkCauses") || "causes"}</span>
					</div>
					<div class="sight2-legend-row">
						<svg width="20" height="4"><line x1="0" y1="2" x2="16" y2="2" stroke="#4AFF88" stroke-width="2"/><polygon points="16,0 20,2 16,4" fill="#4AFF88"/></svg>
						<span>{$t("lens.linkExemplifies") || "exemplifies"}</span>
					</div>
					<div class="sight2-legend-row">
						<svg width="20" height="4"><line x1="0" y1="2" x2="16" y2="2" stroke="#C084FC" stroke-width="2"/><polygon points="16,0 20,2 16,4" fill="#C084FC"/></svg>
						<span>{$t("lens.linkGeneralizes") || "generalizes"}</span>
					</div>
					<div class="sight2-legend-row">
						<svg width="20" height="4"><line x1="0" y1="2" x2="16" y2="2" stroke="#FACC15" stroke-width="2"/><polygon points="16,0 20,2 16,4" fill="#FACC15"/></svg>
						<span>{$t("lens.linkDerivesFrom") || "derives-from"}</span>
					</div>
					<div class="sight2-legend-divider"></div>
					<!-- Confidence = thickness -->
					<div class="sight2-legend-row">
						<svg width="20" height="4"><line x1="0" y1="2" x2="20" y2="2" stroke="#94a3b8" stroke-width="1"/></svg>
						<span>{$t("lens.thinHypothesis") || "thin = hypothesis"}</span>
					</div>
					<div class="sight2-legend-row">
						<svg width="20" height="4"><line x1="0" y1="2" x2="20" y2="2" stroke="#4A9EFF" stroke-width="3"/></svg>
						<span>{$t("lens.thickEstablished") || "thick = established"}</span>
					</div>
					<div class="sight2-legend-row">
						<svg width="20" height="6"><polygon points="0,3 6,0 6,6" fill="#4A9EFF"/><polygon points="8,3 14,0 14,6" fill="#4A9EFF"/></svg>
						<span>{$t("lens.arrowsDirection") || "arrows = direction"}</span>
					</div>
				</div>
			{/if}
		</div>
		<!-- SightPanel sidebar -->
		{#if panelVisible}
			{@const orphans = simNodes.filter(n => (n.linkCount ?? 0) === 0).length}
			{@const libMap = new Map<string, { name: string; color: string; count: number }>()}
			{@const _build = simNodes.forEach(n => {
				const existing = libMap.get(n.libraryName);
				if (existing) existing.count++;
				else libMap.set(n.libraryName, { name: n.libraryName, color: n.libraryColor, count: 1 });
			})}
			<SightPanel
				nodeCount={simNodes.length}
				linkCount={simLinks.length}
				orphanCount={orphans}
				{bridges}
				libraryBreakdown={[...libMap.values()].sort((a, b) => b.count - a.count)}
				onNoteClick={(name) => {
					// Find node by name and trigger neighborhood highlight
					const node = simNodes.find(n => n.name === name || n.id === name.toLowerCase());
					if (node) {
						selectedNode = node;
						neighborIds = new Set<string>();
						for (const link of simLinks) {
							const src = link.source as SimNode;
							const tgt = link.target as SimNode;
							if (src.id === node.id) neighborIds.add(tgt.id);
							if (tgt.id === node.id) neighborIds.add(src.id);
						}
						// Center on node
						panX = -(node.x ?? 0) * zoom;
						panY = -(node.y ?? 0) * zoom;
						requestDraw();
					}
				}}
			/>
		{/if}
	</div>
</div>

<style>
	.sight2-root {
		display: flex;
		flex-direction: column;
		width: 100%;
		height: 100%;
		background: var(--background-primary, #fafafa);
		overflow: hidden;
	}
	.sight2-header {
		display: flex;
		align-items: center;
		gap: 10px;
		padding: 8px 16px;
		border-bottom: 1px solid var(--background-modifier-border, #e5e7eb);
		flex-shrink: 0;
		background: var(--background-primary, #fff);
	}
	.sight2-header svg { color: var(--text-muted, #64748b); }
	.sight2-title { font-size: 14px; font-weight: 700; color: var(--text-normal, #1a1a1a); }
	.sight2-stats { font-size: 11px; color: var(--text-muted, #64748b); }
	.sight2-toolbar { display: flex; gap: 2px; margin-inline-start: auto; }
	.sight2-btn {
		width: 28px; height: 28px; border: none; border-radius: 4px;
		background: none; color: var(--text-muted, #64748b); cursor: pointer;
		display: flex; align-items: center; justify-content: center;
	}
	.sight2-btn:hover { background: var(--background-modifier-hover, #f1f5f9); color: var(--text-normal, #1a1a1a); }
	.sight2-btn.active { background: var(--interactive-accent, #7c3aed); color: white; }
	.sight2-close {
		border: none; background: none; cursor: pointer;
		font-size: 18px; color: var(--text-muted, #64748b); padding: 0 4px;
	}
	.sight2-close:hover { color: var(--text-normal, #1a1a1a); }

	/* Body */
	.sight2-body { flex: 1; display: flex; overflow: hidden; }
	.sight2-canvas-wrap {
		flex: 1;
		position: relative;
		overflow: hidden;
	}
	.sight2-canvas-wrap canvas {
		display: block;
		cursor: grab;
	}
	.sight2-canvas-wrap canvas:active { cursor: grabbing; }

	/* ─── Search bar ─── */
	.sight2-search {
		position: absolute; top: 8px; inset-inline-start: 8px; z-index: 10;
		display: flex; align-items: center; gap: 6px;
		background: var(--background-primary, rgba(255,255,255,0.97));
		border: 1px solid var(--background-modifier-border, #e5e7eb);
		border-radius: 8px; padding: 6px 10px;
		box-shadow: 0 2px 8px rgba(0,0,0,0.08);
	}
	.sight2-search-scope { display: flex; gap: 1px; flex-shrink: 0; }
	.sight2-search-scope button {
		padding: 2px 6px; font-size: 9px;
		border: 1px solid var(--background-modifier-border, #e5e7eb);
		border-radius: 3px; background: none;
		color: var(--text-muted, #64748b); cursor: pointer; font-family: inherit;
	}
	.sight2-search-scope button:hover { background: var(--background-modifier-hover, #f1f5f9); }
	.sight2-search-scope button.active { background: var(--interactive-accent, #7c3aed); color: white; border-color: var(--interactive-accent, #7c3aed); }
	.sight2-search-input {
		position: relative; display: flex; align-items: center; gap: 4px;
		flex: 1; min-width: 200px; max-width: 400px;
	}
	.sight2-search-input svg { color: var(--text-muted, #64748b); flex-shrink: 0; }
	.sight2-search-input input {
		border: none; outline: none; background: none; font-size: 12px;
		font-family: inherit; color: var(--text-normal, #1a1a1a); flex: 1; min-width: 0;
	}
	.sight2-search-clear {
		border: none; background: none; color: var(--text-muted, #64748b);
		cursor: pointer; font-size: 14px; padding: 0 2px;
	}
	.sight2-chips-btn {
		border: none; background: none; cursor: pointer;
		color: var(--text-muted, #64748b); padding: 2px; border-radius: 3px;
	}
	.sight2-chips-btn:hover, .sight2-chips-btn.active {
		color: var(--interactive-accent, #7c3aed);
		background: rgba(124,58,237,0.1);
	}
	.sight2-cat {
		font-size: 9px; color: white; padding: 1px 5px;
		border-radius: 4px; white-space: nowrap;
	}
	.sight2-search-count { font-size: 10px; color: var(--text-muted, #64748b); white-space: nowrap; }
	.sight2-search-none { font-size: 10px; color: #ef4444; white-space: nowrap; }
	.sight2-search-nav {
		border: none; background: none; color: var(--text-muted, #64748b);
		cursor: pointer; padding: 0 2px; display: flex; align-items: center;
	}
	.sight2-search-nav:hover { color: var(--text-normal, #1a1a1a); }

	/* Dropdowns */
	.sight2-dropdown {
		position: absolute; top: 100%; inset-inline-start: 0; z-index: 100;
		background: var(--background-primary, rgba(255,255,255,0.98));
		border: 1px solid var(--background-modifier-border, #e5e7eb);
		border-radius: 8px; box-shadow: 0 4px 16px rgba(0,0,0,0.12);
		min-width: 200px; max-height: 250px; overflow-y: auto; margin-top: 4px;
	}
	.sight2-dropdown-item {
		display: block; width: 100%; text-align: start;
		padding: 6px 12px; border: none; background: none; cursor: pointer;
		font-size: 11px; color: var(--text-normal, #1a1a1a);
	}
	.sight2-dropdown-item:hover { background: var(--background-modifier-hover, #f1f5f9); }
	.sight2-chips-grid {
		display: flex; flex-wrap: wrap; gap: 6px; padding: 10px; max-width: 450px;
	}
	.sight2-chip {
		padding: 4px 10px; border-radius: 6px;
		border: 1.5px solid var(--background-modifier-border, #d1d5db);
		background: var(--background-secondary, #f9fafb);
		color: var(--text-normal, #374151);
		font-size: 11px; font-weight: 500; cursor: pointer; white-space: nowrap;
		transition: all 0.12s;
	}
	.sight2-chip:hover {
		border-color: var(--interactive-accent, #7c3aed);
		color: var(--interactive-accent, #7c3aed);
		background: rgba(124,58,237,0.06);
	}

	/* ─── Settings panel ─── */
	.sight2-settings {
		position: absolute; top: 8px; inset-inline-end: 8px; z-index: 10;
		background: var(--background-primary, rgba(255,255,255,0.97));
		border: 1px solid var(--background-modifier-border, #e5e7eb);
		border-radius: 8px; padding: 12px;
		box-shadow: 0 2px 8px rgba(0,0,0,0.08);
		width: 200px; display: flex; flex-direction: column; gap: 8px;
	}
	.sight2-settings-title { font-size: 12px; font-weight: 700; color: var(--text-normal, #1a1a1a); }
	.sight2-settings-row {
		display: flex; align-items: center; justify-content: space-between;
		gap: 8px;
	}
	.sight2-settings-row span { font-size: 10px; color: var(--text-muted, #64748b); }
	.sight2-settings-row button {
		padding: 2px 10px; border-radius: 4px; font-size: 10px; cursor: pointer;
		border: 1px solid var(--background-modifier-border, #e5e7eb);
		background: none; color: var(--text-muted, #64748b); font-family: inherit;
	}
	.sight2-settings-row button.active { background: var(--interactive-accent, #7c3aed); color: white; border-color: var(--interactive-accent, #7c3aed); }
	.sight2-settings-slider {
		display: flex; flex-direction: column; gap: 2px;
	}
	.sight2-settings-slider span { font-size: 10px; color: var(--text-muted, #64748b); }
	.sight2-settings-slider input[type="range"] { width: 100%; height: 14px; cursor: pointer; }

	/* ─── Legend ─── */
	.sight2-legend {
		position: absolute; bottom: 16px; inset-inline-start: 16px; z-index: 5;
		background: var(--background-primary, rgba(255,255,255,0.95));
		border: 1px solid var(--background-modifier-border, #e5e7eb);
		border-radius: 8px; padding: 10px 14px; font-size: 10px;
		box-shadow: 0 2px 8px rgba(0,0,0,0.08);
		display: flex; flex-direction: column; gap: 4px; max-width: 220px;
	}
	.sight2-legend-title { font-size: 11px; font-weight: 700; color: var(--text-normal, #1a1a1a); margin-bottom: 2px; }
	.sight2-legend-row { display: flex; align-items: center; gap: 5px; color: var(--text-muted, #64748b); font-size: 10px; }
	.sight2-legend-divider { height: 1px; background: var(--background-modifier-border, #e5e7eb); margin: 2px 0; }
	.sight2-lg-dot {
		width: 8px; height: 8px; border-radius: 50%; background: #7c3aed;
		flex-shrink: 0; display: inline-block;
	}
	.sight2-lg-big { width: 14px; height: 14px; }
	.sight2-lg-small { width: 5px; height: 5px; }
</style>
