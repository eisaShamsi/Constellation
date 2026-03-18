/**
 * GraphMind — Layer 2: Pixi.js Imperative Graph Engine
 *
 * THREE LAWS:
 * 1. hoveredIdx is a plain number inside this class. Never $state. Never sent to worker.
 * 2. Worker accepts only DRAG_END and SETTINGS_CHANGE restarts.
 * 3. this.config is a plain JS object. Never reactive.
 *
 * ZERO Svelte imports. This file must never import from 'svelte'.
 */

import { Application, Graphics, Container, Text, TextStyle } from 'pixi.js';

// ─── Types ────────────────────────────────────────────────────────────

export interface EngineConfig {
	nodeSize: number;
	labelVisibility: 'hover' | 'always' | 'none';
	labelFontSize: number;
	linkThickness: number;
	repelForce: number;
	linkForce: number;
	linkDistance: number;
	showOrphans: boolean;
	colorByLibrary: boolean;
}

export interface EngineCallbacks {
	onNodeClick: (path: string, libraryName: string) => void;
	onNodeHover: (name: string | null) => void;
	onStatsReady: (nodeCount: number, edgeCount: number, mocCount: number) => void;
}

interface EngineNode {
	id: string;
	x: number;
	y: number;
	r: number;
	color: number; // hex int for Pixi
	colorHex: string; // original hex string
	name: string;
	path: string;
	libraryName: string;
	linkCount: number;
	outgoingCount: number;
	isRTL: boolean;
}

interface EngineLink {
	sourceIdx: number;
	targetIdx: number;
}

// ─── Constants ────────────────────────────────────────────────────────

const DEFAULT_NODE_COLOR = 0xa78bfa;
const HIGHLIGHT_EDGE_COLOR = 0xf97316;
const DIM_ALPHA = 0.12;
const MOC_RING_COLOR = 0xf59e0b;
const RTL_REGEX = /[\u0600-\u06FF\u0750-\u077F\u08A0-\u08FF\uFB50-\uFDFF\uFE70-\uFEFF\u0590-\u05FF]/;

const CELL_SIZE = 50;

// ─── Engine ───────────────────────────────────────────────────────────

export class GraphEngine {
	// Pixi objects
	private app: Application | null = null;
	private linkGfx: Graphics = new Graphics();
	private nodeContainer: Container = new Container();
	private nodeGfx: Graphics[] = [];
	private labelPool: Map<number, Text> = new Map();

	// Data (plain arrays — Law 1)
	private nodes: EngineNode[] = [];
	private links: EngineLink[] = [];
	private neighborMap: Map<number, Set<number>> = new Map();

	// Render-only state (Law 1 — never leaves this class)
	private hoveredIdx: number = -1;
	private activeNodeIdx: number = -1;
	private searchQuery: string = '';
	private searchMatchSet: Set<number> = new Set();

	// View transform
	private viewX: number = 0;
	private viewY: number = 0;
	private viewScale: number = 1;

	// Interaction
	private draggedNodeIdx: number = -1;
	private isPanning: boolean = false;
	private panStartX: number = 0;
	private panStartY: number = 0;
	private panViewX: number = 0;
	private panViewY: number = 0;
	private isDragging: boolean = false;
	private pointerDownTime: number = 0;

	// Worker
	private worker: Worker | null = null;
	private layoutSettled: boolean = false;
	private didInitialFit: boolean = false;

	// Theme
	private isDark: boolean = false;
	private themeObserver: MutationObserver | null = null;

	// Resize
	private resizeObserver: ResizeObserver | null = null;

	// Redraw flag
	private needsRedraw: boolean = true;

	// Config (Law 3 — plain object)
	public config: EngineConfig;

	// Callbacks
	private callbacks: EngineCallbacks;

	// Container
	private container: HTMLDivElement;

	// Spatial hash for hit testing
	private spatialGrid: Map<string, number[]> = new Map();

	constructor(container: HTMLDivElement, config: EngineConfig, callbacks: EngineCallbacks) {
		this.container = container;
		this.config = { ...config };
		this.callbacks = callbacks;
		this.isDark = document.body.classList.contains('theme-dark');
	}

	async init(): Promise<void> {
		const width = this.container.clientWidth || 800;
		const height = this.container.clientHeight || 600;

		this.app = new Application();
		await this.app.init({
			width,
			height,
			antialias: true,
			backgroundAlpha: 0,
			resolution: window.devicePixelRatio || 1,
			autoDensity: true,
			preference: 'webgl' as any,
		});

		this.container.appendChild(this.app.canvas as HTMLCanvasElement);
		(this.app.canvas as HTMLCanvasElement).style.display = 'block';
		(this.app.canvas as HTMLCanvasElement).style.width = '100%';
		(this.app.canvas as HTMLCanvasElement).style.height = '100%';
		(this.app.canvas as HTMLCanvasElement).style.cursor = 'grab';

		// Stage setup
		this.app.stage.addChild(this.linkGfx);
		this.app.stage.addChild(this.nodeContainer);

		// Event listeners on the canvas
		const canvas = this.app.canvas as HTMLCanvasElement;
		canvas.addEventListener('pointermove', this.onPointerMove);
		canvas.addEventListener('pointerdown', this.onPointerDown);
		canvas.addEventListener('pointerup', this.onPointerUp);
		canvas.addEventListener('pointerleave', this.onPointerLeave);
		canvas.addEventListener('wheel', this.onWheel, { passive: false });
		canvas.addEventListener('dblclick', this.onDoubleClick);

		// Theme observer
		this.themeObserver = new MutationObserver(() => {
			this.isDark = document.body.classList.contains('theme-dark');
			this.needsRedraw = true;
		});
		this.themeObserver.observe(document.body, { attributes: true, attributeFilter: ['class'] });

		// Resize observer
		this.resizeObserver = new ResizeObserver(() => {
			this.resize();
		});
		this.resizeObserver.observe(this.container);

		// Render loop
		this.app.ticker.add(this.draw);
	}

	// ─── Public API ────────────────────────────────────────────────

	setData(
		rawNodes: { id: string; name: string; path: string; libraryName: string; linkCount: number; outgoingCount: number }[],
		rawLinks: { source: string; target: string }[],
		colorMap: Record<string, string>
	): void {
		// Kill previous worker
		if (this.worker) {
			this.worker.terminate();
			this.worker = null;
		}

		// Filter orphans if needed
		const linkedIds = new Set<string>();
		for (const l of rawLinks) {
			linkedIds.add(l.source);
			linkedIds.add(l.target);
		}

		const filteredNodes = this.config.showOrphans
			? rawNodes
			: rawNodes.filter((n) => linkedIds.has(n.id));

		const nodeIdMap = new Map<string, number>();

		// Build nodes
		const sizeMul = this.config.nodeSize / 4;
		this.nodes = filteredNodes.map((n, i) => {
			nodeIdMap.set(n.id, i);
			const hexStr = this.config.colorByLibrary ? (colorMap[n.libraryName] || '#a78bfa') : '#a78bfa';
			return {
				id: n.id,
				x: (Math.random() - 0.5) * 800,
				y: (Math.random() - 0.5) * 800,
				r: Math.max(2, (2 + Math.sqrt(n.linkCount) * 1.5) * (n.outgoingCount >= 5 ? 1.6 : 1) * sizeMul),
				color: hexToInt(hexStr),
				colorHex: hexStr,
				name: n.name.replace(/\.md$/, ''),
				path: n.path,
				libraryName: n.libraryName,
				linkCount: n.linkCount,
				outgoingCount: n.outgoingCount,
				isRTL: RTL_REGEX.test(n.name),
			};
		});

		// Build links
		this.links = [];
		this.neighborMap = new Map();
		for (let i = 0; i < this.nodes.length; i++) {
			this.neighborMap.set(i, new Set());
		}

		for (const l of rawLinks) {
			const si = nodeIdMap.get(l.source);
			const ti = nodeIdMap.get(l.target);
			if (si !== undefined && ti !== undefined && si !== ti) {
				this.links.push({ sourceIdx: si, targetIdx: ti });
				this.neighborMap.get(si)!.add(ti);
				this.neighborMap.get(ti)!.add(si);
			}
		}

		// Create Pixi graphics for nodes
		this.nodeContainer.removeChildren();
		this.nodeGfx = [];
		for (let i = 0; i < this.nodes.length; i++) {
			const gfx = new Graphics();
			this.nodeContainer.addChild(gfx);
			this.nodeGfx.push(gfx);
		}

		// Clear labels
		this.labelPool.forEach((t) => t.destroy());
		this.labelPool.clear();

		// Stats
		const mocCount = this.nodes.filter((n) => n.outgoingCount >= 5).length;
		this.callbacks.onStatsReady(this.nodes.length, this.links.length, mocCount);

		// Reset view
		this.viewX = 0;
		this.viewY = 0;
		this.viewScale = 1;
		this.hoveredIdx = -1;
		this.layoutSettled = false;
		this.didInitialFit = false;
		this.needsRedraw = true;

		// Start worker
		this.startWorker();
	}

	updateConfig(partial: Partial<EngineConfig>): void {
		const physicsKeys = ['repelForce', 'linkForce', 'linkDistance'];
		let physicsChanged = false;

		for (const [key, val] of Object.entries(partial)) {
			if (physicsKeys.includes(key) && (this.config as any)[key] !== val) {
				physicsChanged = true;
			}
			(this.config as any)[key] = val;
		}

		// Update node sizes if nodeSize changed
		if ('nodeSize' in partial) {
			const sizeMul = this.config.nodeSize / 4;
			for (const n of this.nodes) {
				n.r = Math.max(2, (2 + Math.sqrt(n.linkCount) * 1.5) * (n.outgoingCount >= 5 ? 1.6 : 1) * sizeMul);
			}
		}

		// Send SETTINGS_CHANGE to worker (Law 2 — allowed restart)
		if (physicsChanged && this.worker) {
			this.worker.postMessage({
				type: 'updateSettings',
				settings: {
					repelForce: this.config.repelForce,
					linkForce: this.config.linkForce,
					linkDistance: this.config.linkDistance,
					centerForce: 0.1,
				},
			});
		}

		this.needsRedraw = true;
	}

	setActiveNode(nodeId: string): void {
		this.activeNodeIdx = this.nodes.findIndex((n) => n.id === nodeId);
		this.needsRedraw = true;
	}

	setSearch(query: string): void {
		this.searchQuery = query.toLowerCase();
		this.searchMatchSet.clear();
		if (this.searchQuery) {
			for (let i = 0; i < this.nodes.length; i++) {
				if (this.nodes[i].name.toLowerCase().includes(this.searchQuery)) {
					this.searchMatchSet.add(i);
				}
			}
		}
		this.needsRedraw = true;
	}

	fitToScreen(): void {
		if (this.nodes.length === 0 || !this.app) return;
		const w = this.container.clientWidth;
		const h = this.container.clientHeight;
		if (w === 0 || h === 0) return;

		let minX = Infinity, maxX = -Infinity, minY = Infinity, maxY = -Infinity;
		for (const n of this.nodes) {
			if (n.x < minX) minX = n.x;
			if (n.x > maxX) maxX = n.x;
			if (n.y < minY) minY = n.y;
			if (n.y > maxY) maxY = n.y;
		}

		const gw = maxX - minX || 1;
		const gh = maxY - minY || 1;
		const pad = 60;
		const scale = Math.min((w - pad * 2) / gw, (h - pad * 2) / gh, 3);

		// Animate toward target
		const targetScale = scale;
		const targetX = -(minX + maxX) / 2 * scale;
		const targetY = -(minY + maxY) / 2 * scale;

		// Simple animation over ~15 frames
		const frames = 15;
		const startScale = this.viewScale;
		const startX = this.viewX;
		const startY = this.viewY;
		let frame = 0;

		const animate = () => {
			frame++;
			const t = Math.min(1, frame / frames);
			const ease = t * (2 - t); // easeOut
			this.viewScale = startScale + (targetScale - startScale) * ease;
			this.viewX = startX + (targetX - startX) * ease;
			this.viewY = startY + (targetY - startY) * ease;
			this.needsRedraw = true;
			if (frame < frames) requestAnimationFrame(animate);
		};
		requestAnimationFrame(animate);
	}

	resize(): void {
		if (!this.app || !this.container) return;
		const w = this.container.clientWidth;
		const h = this.container.clientHeight;
		if (w > 0 && h > 0) {
			this.app.renderer.resize(w, h);
			this.needsRedraw = true;
		}
	}

	destroy(): void {
		if (this.worker) {
			this.worker.postMessage({ type: 'stop' });
			this.worker.terminate();
			this.worker = null;
		}

		const canvas = this.app?.canvas as HTMLCanvasElement | undefined;
		if (canvas) {
			canvas.removeEventListener('pointermove', this.onPointerMove);
			canvas.removeEventListener('pointerdown', this.onPointerDown);
			canvas.removeEventListener('pointerup', this.onPointerUp);
			canvas.removeEventListener('pointerleave', this.onPointerLeave);
			canvas.removeEventListener('wheel', this.onWheel);
			canvas.removeEventListener('dblclick', this.onDoubleClick);
		}

		this.themeObserver?.disconnect();
		this.resizeObserver?.disconnect();

		this.labelPool.forEach((t) => t.destroy());
		this.labelPool.clear();

		this.app?.destroy(true, { children: true, texture: true });
		this.app = null;
	}

	// ─── Worker (Layer 3) ──────────────────────────────────────────

	private startWorker(): void {
		try {
			this.worker = new Worker(new URL('./forceWorker.ts', import.meta.url), { type: 'module' });
		} catch {
			// Fallback: circular layout
			this.applyCircularLayout();
			return;
		}

		this.worker.onmessage = (e: MessageEvent) => {
			if (e.data.type === 'positions') {
				const pos = e.data.positions as Float64Array;
				for (let i = 0; i < this.nodes.length && i * 2 + 1 < pos.length; i++) {
					this.nodes[i].x = pos[i * 2];
					this.nodes[i].y = pos[i * 2 + 1];
				}
				this.needsRedraw = true;

				if (e.data.settled && !this.didInitialFit) {
					this.didInitialFit = true;
					this.layoutSettled = true;
					this.fitToScreen();
				}
			}
		};

		const workerNodes = this.nodes.map((n) => ({ id: n.id, x: n.x, y: n.y }));
		const workerEdges = this.links.map((l) => ({
			source: this.nodes[l.sourceIdx].id,
			target: this.nodes[l.targetIdx].id,
		}));

		this.worker.postMessage({
			type: 'init',
			nodes: workerNodes,
			edges: workerEdges,
			settings: {
				repelForce: this.config.repelForce,
				linkForce: this.config.linkForce,
				linkDistance: this.config.linkDistance,
				centerForce: 0.1,
			},
		});
	}

	private applyCircularLayout(): void {
		const n = this.nodes.length;
		const radius = Math.max(200, n * 3);
		for (let i = 0; i < n; i++) {
			const angle = (2 * Math.PI * i) / n;
			this.nodes[i].x = Math.cos(angle) * radius;
			this.nodes[i].y = Math.sin(angle) * radius;
		}
		this.needsRedraw = true;
		setTimeout(() => this.fitToScreen(), 100);
	}

	// ─── Hit Testing ───────────────────────────────────────────────

	private buildSpatialGrid(): void {
		this.spatialGrid.clear();
		for (let i = 0; i < this.nodes.length; i++) {
			const cx = Math.floor(this.nodes[i].x / CELL_SIZE);
			const cy = Math.floor(this.nodes[i].y / CELL_SIZE);
			const key = `${cx},${cy}`;
			if (!this.spatialGrid.has(key)) this.spatialGrid.set(key, []);
			this.spatialGrid.get(key)!.push(i);
		}
	}

	private hitTest(wx: number, wy: number): number {
		const hitR = 12 / this.viewScale;
		const cx = Math.floor(wx / CELL_SIZE);
		const cy = Math.floor(wy / CELL_SIZE);

		for (let dx = -1; dx <= 1; dx++) {
			for (let dy = -1; dy <= 1; dy++) {
				const cell = this.spatialGrid.get(`${cx + dx},${cy + dy}`);
				if (!cell) continue;
				for (const idx of cell) {
					const ddx = wx - this.nodes[idx].x;
					const ddy = wy - this.nodes[idx].y;
					if (ddx * ddx + ddy * ddy < hitR * hitR) return idx;
				}
			}
		}
		return -1;
	}

	private screenToWorld(sx: number, sy: number): { wx: number; wy: number } {
		const w = this.container.clientWidth;
		const h = this.container.clientHeight;
		return {
			wx: (sx - w / 2 - this.viewX) / this.viewScale,
			wy: (sy - h / 2 - this.viewY) / this.viewScale,
		};
	}

	// ─── Interaction (Law 1 — hover NEVER leaves this class) ──────

	private onPointerMove = (e: PointerEvent): void => {
		const canvas = this.app?.canvas as HTMLCanvasElement;
		if (!canvas) return;

		if (this.isPanning) {
			this.viewX = this.panViewX + (e.clientX - this.panStartX);
			this.viewY = this.panViewY + (e.clientY - this.panStartY);
			this.needsRedraw = true;
			return;
		}

		if (this.draggedNodeIdx >= 0) {
			this.isDragging = true;
			const rect = canvas.getBoundingClientRect();
			const { wx, wy } = this.screenToWorld(e.clientX - rect.left, e.clientY - rect.top);
			this.nodes[this.draggedNodeIdx].x = wx;
			this.nodes[this.draggedNodeIdx].y = wy;

			// Pin in worker (NO restart — Law 2)
			this.worker?.postMessage({
				type: 'pinNode',
				id: this.nodes[this.draggedNodeIdx].id,
				x: wx,
				y: wy,
			});

			this.needsRedraw = true;
			return;
		}

		// Hover detection
		const rect = canvas.getBoundingClientRect();
		const { wx, wy } = this.screenToWorld(e.clientX - rect.left, e.clientY - rect.top);
		this.buildSpatialGrid();
		const idx = this.hitTest(wx, wy);

		if (idx !== this.hoveredIdx) {
			this.hoveredIdx = idx; // Law 1: plain variable, never $state
			canvas.style.cursor = idx >= 0 ? 'pointer' : 'grab';
			this.callbacks.onNodeHover(idx >= 0 ? this.nodes[idx].name : null);
			this.needsRedraw = true;
			// NOTICE: Worker is NEVER notified of hover. Law 2 enforced.
		}
	};

	private onPointerDown = (e: PointerEvent): void => {
		if (e.button !== 0) return;
		this.pointerDownTime = Date.now();

		if (this.hoveredIdx >= 0) {
			// Start dragging a node
			this.draggedNodeIdx = this.hoveredIdx;
			this.isDragging = false;
			const canvas = this.app?.canvas as HTMLCanvasElement;
			if (canvas) canvas.style.cursor = 'grabbing';
		} else {
			// Start panning
			this.isPanning = true;
			this.panStartX = e.clientX;
			this.panStartY = e.clientY;
			this.panViewX = this.viewX;
			this.panViewY = this.viewY;
			const canvas = this.app?.canvas as HTMLCanvasElement;
			if (canvas) canvas.style.cursor = 'grabbing';
		}
	};

	private onPointerUp = (e: PointerEvent): void => {
		const canvas = this.app?.canvas as HTMLCanvasElement;

		if (this.draggedNodeIdx >= 0) {
			if (this.isDragging) {
				// Drag ended — send DRAG_END (Law 2 — allowed restart)
				const node = this.nodes[this.draggedNodeIdx];
				this.worker?.postMessage({
					type: 'dragEnd',
					id: node.id,
					x: node.x,
					y: node.y,
				});
			} else {
				// Was a click, not a drag
				const elapsed = Date.now() - this.pointerDownTime;
				if (elapsed < 300) {
					const node = this.nodes[this.draggedNodeIdx];
					this.callbacks.onNodeClick(node.path, node.libraryName);
				}
				// Unpin
				this.worker?.postMessage({
					type: 'unpinNode',
					id: this.nodes[this.draggedNodeIdx].id,
				});
			}
			this.draggedNodeIdx = -1;
			this.isDragging = false;
		}

		if (this.isPanning) {
			this.isPanning = false;
		}

		if (canvas) canvas.style.cursor = this.hoveredIdx >= 0 ? 'pointer' : 'grab';
	};

	private onPointerLeave = (): void => {
		this.isPanning = false;
		this.draggedNodeIdx = -1;
		if (this.hoveredIdx !== -1) {
			this.hoveredIdx = -1;
			this.callbacks.onNodeHover(null);
			this.needsRedraw = true;
		}
	};

	private onWheel = (e: WheelEvent): void => {
		e.preventDefault();
		const zoomFactor = e.deltaY > 0 ? 0.9 : 1.1;
		const canvas = this.app?.canvas as HTMLCanvasElement;
		if (!canvas) return;

		const rect = canvas.getBoundingClientRect();
		const mx = e.clientX - rect.left;
		const my = e.clientY - rect.top;
		const w = this.container.clientWidth;
		const h = this.container.clientHeight;
		const wx = mx - w / 2 - this.viewX;
		const wy = my - h / 2 - this.viewY;

		this.viewScale *= zoomFactor;
		this.viewScale = Math.max(0.05, Math.min(15, this.viewScale));
		this.viewX -= wx * (zoomFactor - 1);
		this.viewY -= wy * (zoomFactor - 1);
		this.needsRedraw = true;
	};

	private onDoubleClick = (e: MouseEvent): void => {
		if (this.hoveredIdx < 0) return;
		const node = this.nodes[this.hoveredIdx];
		// Animate camera to center on this node
		const w = this.container.clientWidth;
		const h = this.container.clientHeight;
		const targetScale = Math.max(this.viewScale, 1.5);
		const targetX = -node.x * targetScale;
		const targetY = -node.y * targetScale;

		const frames = 20;
		const startScale = this.viewScale;
		const startX = this.viewX;
		const startY = this.viewY;
		let frame = 0;

		const animate = () => {
			frame++;
			const t = Math.min(1, frame / frames);
			const ease = t * (2 - t);
			this.viewScale = startScale + (targetScale - startScale) * ease;
			this.viewX = startX + (targetX - startX) * ease;
			this.viewY = startY + (targetY - startY) * ease;
			this.needsRedraw = true;
			if (frame < frames) requestAnimationFrame(animate);
		};
		requestAnimationFrame(animate);
	};

	// ─── Render Loop (Pixi Ticker) ────────────────────────────────

	private draw = (): void => {
		if (!this.needsRedraw || !this.app) return;
		this.needsRedraw = false;

		const w = this.container.clientWidth;
		const h = this.container.clientHeight;
		const hovered = this.hoveredIdx;
		const neighbors = hovered >= 0 ? this.neighborMap.get(hovered) : null;
		const hasSearch = this.searchQuery.length > 0;
		const dark = this.isDark;

		// ─── Links ────
		this.linkGfx.clear();
		const normalEdgeColor = dark ? 0x475569 : 0xbcccdc;
		const normalEdgeAlpha = dark ? 0.25 : 0.15;

		for (const link of this.links) {
			const src = this.nodes[link.sourceIdx];
			const tgt = this.nodes[link.targetIdx];

			const sx = src.x * this.viewScale + w / 2 + this.viewX;
			const sy = src.y * this.viewScale + h / 2 + this.viewY;
			const tx = tgt.x * this.viewScale + w / 2 + this.viewX;
			const ty = tgt.y * this.viewScale + h / 2 + this.viewY;

			// Determine if highlighted
			const isNeighborEdge = hovered >= 0 && (link.sourceIdx === hovered || link.targetIdx === hovered);

			if (hovered >= 0 && !isNeighborEdge) {
				// Hide non-neighbor edges when hovering
				continue;
			}

			if (hasSearch) {
				const srcMatch = this.searchMatchSet.has(link.sourceIdx);
				const tgtMatch = this.searchMatchSet.has(link.targetIdx);
				if (!srcMatch && !tgtMatch) continue;
			}

			if (isNeighborEdge) {
				this.linkGfx.moveTo(sx, sy);
				this.linkGfx.lineTo(tx, ty);
				this.linkGfx.stroke({ width: this.config.linkThickness * 2, color: HIGHLIGHT_EDGE_COLOR, alpha: 0.9 });
			} else {
				this.linkGfx.moveTo(sx, sy);
				this.linkGfx.lineTo(tx, ty);
				this.linkGfx.stroke({ width: this.config.linkThickness * 0.5, color: normalEdgeColor, alpha: normalEdgeAlpha });
			}
		}

		// ─── Nodes ────
		for (let i = 0; i < this.nodes.length; i++) {
			const n = this.nodes[i];
			const gfx = this.nodeGfx[i];
			if (!gfx) continue;

			const sx = n.x * this.viewScale + w / 2 + this.viewX;
			const sy = n.y * this.viewScale + h / 2 + this.viewY;

			const isHovered = i === hovered;
			const isActive = i === this.activeNodeIdx;
			const isNeighbor = neighbors?.has(i) ?? false;

			// Determine alpha
			let alpha = 1.0;
			if (hovered >= 0 && !isHovered && !isNeighbor) alpha = DIM_ALPHA;
			if (hasSearch && !this.searchMatchSet.has(i) && hovered < 0) alpha = DIM_ALPHA;

			const r = n.r * (isHovered ? 1.4 : isActive ? 1.3 : 1);

			gfx.clear();
			gfx.circle(sx, sy, r);
			gfx.fill({ color: n.color, alpha });

			// Active ring
			if (isActive) {
				gfx.circle(sx, sy, r + 2);
				gfx.stroke({ width: 2, color: dark ? 0xffffff : 0x333333, alpha: 0.8 });
			}

			// MOC gold ring
			if (n.outgoingCount >= 5) {
				gfx.circle(sx, sy, r + 1.5);
				gfx.stroke({ width: 1.5, color: MOC_RING_COLOR, alpha: alpha });
			}
		}

		// ─── Labels ────
		this.updateLabels(w, h, hovered, neighbors, dark);
	};

	private updateLabels(w: number, h: number, hovered: number, neighbors: Set<number> | null | undefined, dark: boolean): void {
		if (!this.app) return;

		// Clean up old labels
		this.labelPool.forEach((text, idx) => {
			text.visible = false;
		});

		if (this.config.labelVisibility === 'none') return;

		const showAll = this.config.labelVisibility === 'always';

		for (let i = 0; i < this.nodes.length; i++) {
			const n = this.nodes[i];
			const isHovered = i === hovered;
			const isActive = i === this.activeNodeIdx;
			const isNeighbor = neighbors?.has(i) ?? false;

			if (!showAll && !isHovered && !isActive) continue;

			const sx = n.x * this.viewScale + w / 2 + this.viewX;
			const sy = n.y * this.viewScale + h / 2 + this.viewY;

			let label = this.labelPool.get(i);
			if (!label) {
				const style = new TextStyle({
					fontSize: this.config.labelFontSize,
					fontFamily: 'system-ui, -apple-system, sans-serif',
					fill: dark ? '#e2e8f0' : '#1e293b',
					align: n.isRTL ? 'right' : 'left',
				});
				label = new Text({ text: n.name, style });
				label.anchor.set(n.isRTL ? 1 : 0, 0);
				this.app!.stage.addChild(label);
				this.labelPool.set(i, label);
			}

			label.visible = true;
			const offsetX = n.isRTL ? -n.r - 4 : n.r + 4;
			label.position.set(sx + offsetX, sy - this.config.labelFontSize / 2);

			// Dim if not relevant
			if (hovered >= 0 && !isHovered && !isNeighbor) {
				label.alpha = DIM_ALPHA;
			} else {
				label.alpha = 1;
			}
		}
	}
}

// ─── Helpers ──────────────────────────────────────────────────────────

function hexToInt(hex: string): number {
	return parseInt(hex.replace('#', ''), 16);
}
