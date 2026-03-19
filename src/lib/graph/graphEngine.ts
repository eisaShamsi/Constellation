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

export type LayoutMode = 'organic' | 'hierarchical' | 'temporal';

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
	layoutMode: LayoutMode;
	showSemanticLinks: boolean;
	semanticThreshold: number; // 0-1, default 0.5
	multiSphere: boolean; // each library gets its own sphere
}

export interface EngineCallbacks {
	onNodeClick: (path: string, libraryName: string) => void;
	onNodeHover: (name: string | null) => void;
	onStatsReady: (nodeCount: number, edgeCount: number, mocCount: number) => void;
	onContextMenu?: (node: { id: string; name: string; path: string; libraryName: string }, x: number, y: number) => void;
	onFocusChange?: (focused: boolean, nodeName?: string) => void;
	onHiddenCountChange?: (count: number) => void;
	onTiltChange?: (tilted: boolean) => void;
	onEdgeHover?: (info: { sourceName: string; targetName: string; linkType?: string; semantic?: boolean; similarity?: number } | null) => void;
}

interface EngineNode {
	id: string;
	x: number;
	y: number;
	z: number; // 3D depth coordinate
	r: number;
	color: number; // hex int for Pixi
	colorHex: string; // original hex string
	name: string;
	path: string;
	libraryName: string;
	linkCount: number;
	outgoingCount: number;
	isRTL: boolean;
	createdAt: number; // epoch ms (0 if unknown)
}

interface EngineLink {
	sourceIdx: number;
	targetIdx: number;
	semantic?: boolean; // true = AI-detected, false/undefined = explicit wikilink
	similarity?: number; // 0-1 confidence for semantic links
	linkType?: string; // relationship label: supports, contradicts, elaborates, questions, custom
}

// ─── Constants ────────────────────────────────────────────────────────

const DEFAULT_NODE_COLOR = 0xa78bfa;
const HIGHLIGHT_EDGE_COLOR = 0xf97316;
const DIM_ALPHA = 0.12;
const MOC_RING_COLOR = 0xf59e0b;
const RTL_REGEX = /[\u0600-\u06FF\u0750-\u077F\u08A0-\u08FF\uFB50-\uFDFF\uFE70-\uFEFF\u0590-\u05FF]/;
const ARABIC_REGEX = /[\u0600-\u06FF\u0750-\u077F\u08A0-\u08FF\uFB50-\uFDFF\uFE70-\uFEFF]/;
const HEBREW_REGEX = /[\u0590-\u05FF]/;
const CJK_REGEX = /[\u4E00-\u9FFF\u3040-\u309F\u30A0-\u30FF\uAC00-\uD7AF]/;

// Font stacks for different scripts
const FONT_LATIN = '"Inter", "Segoe UI", system-ui, -apple-system, sans-serif';
const FONT_ARABIC = '"Noto Naskh Arabic", "Noto Sans Arabic", "Segoe UI", "Tahoma", "Arial", sans-serif';
const FONT_HEBREW = '"Noto Sans Hebrew", "Segoe UI", "Arial", sans-serif';
const FONT_CJK = '"Noto Sans CJK SC", "Microsoft YaHei", "PingFang SC", sans-serif';

function getFontForText(text: string): string {
	if (ARABIC_REGEX.test(text)) return FONT_ARABIC;
	if (HEBREW_REGEX.test(text)) return FONT_HEBREW;
	if (CJK_REGEX.test(text)) return FONT_CJK;
	return FONT_LATIN;
}

const CELL_SIZE = 50;

// ─── Engine ───────────────────────────────────────────────────────────

export class GraphEngine {
	// Pixi objects
	private app: Application | null = null;
	private linkGfx: Graphics = new Graphics();
	private gizmoGfx: Graphics = new Graphics(); // 3D axis guide
	private gizmoLabels: Text[] = []; // X, Y, Z labels on axes
	private nodeContainer: Container = new Container();
	private nodeGfx: Graphics[] = [];
	private labelPool: Map<number, Text> = new Map();

	// Data (plain arrays — Law 1)
	private nodes: EngineNode[] = [];
	private links: EngineLink[] = []; // explicit links
	private semanticLinks: EngineLink[] = []; // AI-detected links (Phase 2)
	private clusterAssignments: Map<number, number> = new Map(); // nodeIdx → clusterId
	private clusterColors: Map<number, number> = new Map(); // clusterId → color (hex int)
	private showClusters: boolean = false;
	private neighborMap: Map<number, Set<number>> = new Map();

	// Render-only state (Law 1 — never leaves this class)
	private hoveredIdx: number = -1;
	private activeNodeIdx: number = -1;
	private searchQuery: string = '';
	private searchMatchSet: Set<number> = new Set();

	// Focus mode
	private focusNodeIdx: number = -1;
	private focusDepth: number = 2;
	private focusDirection: 'all' | 'incoming' | 'outgoing' = 'all';
	private focusSet: Set<number> = new Set();

	// Local graph mode (Space bar toggle)
	private localGraphMode: boolean = false;

	// Pin & Hide
	private pinnedNodeIds: Set<string> = new Set();
	private hiddenNodeIds: Set<string> = new Set();
	private hiddenIndices: Set<number> = new Set();

	// Layout transition
	private transitionFrom: { x: number; y: number }[] = [];
	private transitionTo: { x: number; y: number }[] = [];
	private transitionProgress: number = -1; // -1 = no transition
	private transitionFrames: number = 30;

	// Directed neighbor maps (for directional filter)
	private outgoingMap: Map<number, Set<number>> = new Map();
	private incomingMap: Map<number, Set<number>> = new Map();

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

	// 3D Camera
	private camRotX: number = 0; // pitch (degrees)
	private camRotY: number = 0; // yaw (degrees)
	private camRotZ: number = 0; // roll (degrees)
	private camPosX: number = 0; // camera position in graph space
	private camPosY: number = 0;
	private camPosZ: number = 0; // negative = into the screen (flying forward)
	private camDistance: number = 1200; // perspective distance
	private isRotating: boolean = false;
	private rotStartX: number = 0;
	private rotStartY: number = 0;
	private rotBaseX: number = 0;
	private rotBaseY: number = 0;
	private rotBaseZ: number = 0;

	// 4D: auto-rotation (slow idle spin when not interacting)
	private autoRotateSpeed: number = 0.05; // degrees per frame
	private autoRotateEnabled: boolean = true;
	private lastInteractionTime: number = 0;
	private autoRotateDelay: number = 3000; // ms of inactivity before auto-rotate starts

	// Camera gravity: pull camera back toward center
	private cameraGravityStrength: number = 0.02; // 0 = off, higher = stronger pull

	// Multi-sphere: each library orbits as its own sphere
	private librarySpheres: Map<string, {
		centerX: number; centerY: number; centerZ: number; // sphere center in world space
		radius: number; // sphere radius
		localRotAngle: number; // local self-rotation (degrees)
		localRotSpeed: number; // degrees per frame
		orbitAngle: number; // position on global orbit (degrees)
		nodeIndices: number[]; // indices into this.nodes
	}> = new Map();
	private globalOrbitSpeed: number = 0.02; // degrees per frame
	private globalOrbitRadius: number = 0; // computed from library count

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
		this.app.stage.addChild(this.gizmoGfx);

		// Create gizmo axis labels (X, Y, Z)
		const gizmoLabelStyle = (color: string) => new TextStyle({
			fontSize: 13, fontWeight: 'bold', fontFamily: 'monospace', fill: color,
		});
		const axisLabels = [
			{ text: 'X', color: '#ef4444' },
			{ text: 'Y', color: '#22c55e' },
			{ text: 'Z', color: '#3b82f6' },
		];
		for (const al of axisLabels) {
			const t = new Text({ text: al.text, style: gizmoLabelStyle(al.color) });
			t.anchor.set(0.5, 0.5);
			t.visible = false;
			this.app.stage.addChild(t);
			this.gizmoLabels.push(t);
		}

		// Event listeners on the canvas
		const canvas = this.app.canvas as HTMLCanvasElement;
		canvas.addEventListener('pointermove', this.onPointerMove);
		canvas.addEventListener('pointerdown', this.onPointerDown);
		canvas.addEventListener('pointerup', this.onPointerUp);
		canvas.addEventListener('pointerleave', this.onPointerLeave);
		canvas.addEventListener('wheel', this.onWheel, { passive: false });
		canvas.addEventListener('dblclick', this.onDoubleClick);
		canvas.addEventListener('contextmenu', this.onContextMenu);
		canvas.addEventListener('auxclick', (e) => e.preventDefault()); // prevent middle-click paste

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
		rawNodes: { id: string; name: string; path: string; libraryName: string; linkCount: number; outgoingCount: number; createdAt?: number }[],
		rawLinks: { source: string; target: string; linkType?: string }[],
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
				z: (Math.random() - 0.5) * 400, // 3D depth spread
				r: Math.max(2, (2 + Math.sqrt(n.linkCount) * 1.5) * (n.outgoingCount >= 5 ? 1.6 : 1) * sizeMul),
				color: hexToInt(hexStr),
				colorHex: hexStr,
				name: n.name.replace(/\.md$/, ''),
				path: n.path,
				libraryName: n.libraryName,
				linkCount: n.linkCount,
				outgoingCount: n.outgoingCount,
				isRTL: RTL_REGEX.test(n.name),
				createdAt: n.createdAt ?? 0,
			};
		});

		// Build links
		this.links = [];
		this.neighborMap = new Map();
		this.outgoingMap = new Map();
		this.incomingMap = new Map();
		for (let i = 0; i < this.nodes.length; i++) {
			this.neighborMap.set(i, new Set());
			this.outgoingMap.set(i, new Set());
			this.incomingMap.set(i, new Set());
		}

		for (const l of rawLinks) {
			const si = nodeIdMap.get(l.source);
			const ti = nodeIdMap.get(l.target);
			if (si !== undefined && ti !== undefined && si !== ti) {
				this.links.push({ sourceIdx: si, targetIdx: ti, linkType: l.linkType });
				this.neighborMap.get(si)!.add(ti);
				this.neighborMap.get(ti)!.add(si);
				this.outgoingMap.get(si)!.add(ti);
				this.incomingMap.get(ti)!.add(si);
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

	/** Inject AI-detected semantic links (Phase 2) */
	setSemanticLinks(links: { source: string; target: string; similarity: number }[]): void {
		const nodeIdMap = new Map<string, number>();
		this.nodes.forEach((n, i) => nodeIdMap.set(n.id, i));

		this.semanticLinks = [];
		for (const l of links) {
			const si = nodeIdMap.get(l.source);
			const ti = nodeIdMap.get(l.target);
			if (si !== undefined && ti !== undefined && si !== ti) {
				this.semanticLinks.push({
					sourceIdx: si,
					targetIdx: ti,
					semantic: true,
					similarity: l.similarity,
				});
			}
		}
		this.needsRedraw = true;
	}

	/** Set cluster assignments for community visualization (Phase 2) */
	setClusters(assignments: Map<string, number>, clusterColors: Map<number, string>): void {
		this.clusterAssignments.clear();
		this.clusterColors.clear();

		const nodeIdMap = new Map<string, number>();
		this.nodes.forEach((n, i) => nodeIdMap.set(n.id, i));

		for (const [id, cid] of assignments) {
			const idx = nodeIdMap.get(id);
			if (idx !== undefined) this.clusterAssignments.set(idx, cid);
		}
		for (const [cid, hex] of clusterColors) {
			this.clusterColors.set(cid, hexToInt(hex));
		}
		this.showClusters = true;
		this.needsRedraw = true;
	}

	clearClusters(): void {
		this.clusterAssignments.clear();
		this.clusterColors.clear();
		this.showClusters = false;
		this.needsRedraw = true;
	}

	/**
	 * Project nodes into a 3D sphere (Crystal Ball).
	 * - Single mode: all nodes fill one sphere volume
	 * - Multi-sphere mode: each library gets its own sphere, arranged in orbit
	 */
	private projectOntoSphere(): void {
		if (this.nodes.length === 0) return;

		if (this.config.multiSphere) {
			this.projectMultiSphere();
		} else {
			this.projectSingleSphere(this.nodes.map((_, i) => i), 0, 0, 0);
		}

		this.needsRedraw = true;
	}

	/** Fill a subset of nodes into a sphere volume centered at (cx, cy, cz) */
	private projectSingleSphere(indices: number[], cx: number, cy: number, cz: number): void {
		const N = indices.length;
		if (N === 0) return;

		const outerR = Math.max(200, Math.sqrt(N) * 28);
		const now = Date.now();
		const phi = (1 + Math.sqrt(5)) / 2;

		// Rank by seniority: high links + old = closer to core
		const scored = indices.map(idx => {
			const n = this.nodes[idx];
			const linkScore = Math.sqrt(n.linkCount) * 3;
			const ageDays = n.createdAt > 0 ? (now - n.createdAt) / 86400000 : 0;
			const ageScore = Math.min(10, Math.sqrt(ageDays / 30) * 3);
			return { idx, score: linkScore + ageScore };
		}).sort((a, b) => b.score - a.score);

		for (let rank = 0; rank < N; rank++) {
			const { idx } = scored[rank];
			const t = (rank + 0.5) / N;
			const r = outerR * t;
			const theta = Math.acos(1 - 2 * t);
			const lonAngle = 2 * Math.PI * rank / phi;

			this.nodes[idx].x = cx + r * Math.sin(theta) * Math.cos(lonAngle);
			this.nodes[idx].y = cy + r * Math.cos(theta);
			this.nodes[idx].z = cz + r * Math.sin(theta) * Math.sin(lonAngle);
		}
	}

	/** Multi-sphere: each library gets its own sphere, arranged in a circular orbit */
	private projectMultiSphere(): void {
		// Group nodes by library
		const libGroups = new Map<string, number[]>();
		for (let i = 0; i < this.nodes.length; i++) {
			const lib = this.nodes[i].libraryName;
			if (!libGroups.has(lib)) libGroups.set(lib, []);
			libGroups.get(lib)!.push(i);
		}

		const libs = Array.from(libGroups.keys());
		const libCount = libs.length;
		if (libCount === 0) return;

		// Orbit radius: enough to separate spheres without overlap
		const maxSphereR = Math.max(200, Math.sqrt(this.nodes.length / libCount) * 28);
		this.globalOrbitRadius = libCount === 1 ? 0 : maxSphereR * 2.5;

		// Clear and rebuild sphere data
		this.librarySpheres.clear();

		for (let li = 0; li < libCount; li++) {
			const libName = libs[li];
			const nodeIndices = libGroups.get(libName)!;

			// Position each library on a circular orbit (in the XZ plane)
			const orbitAngle = (360 / libCount) * li;
			const rad = orbitAngle * Math.PI / 180;
			const centerX = this.globalOrbitRadius * Math.cos(rad);
			const centerY = 0;
			const centerZ = this.globalOrbitRadius * Math.sin(rad);

			// Compute individual sphere radius
			const sphereR = Math.max(200, Math.sqrt(nodeIndices.length) * 28);

			this.librarySpheres.set(libName, {
				centerX, centerY, centerZ,
				radius: sphereR,
				localRotAngle: li * 60, // offset start rotations
				localRotSpeed: 0.03 + li * 0.01, // slightly different speeds
				orbitAngle,
				nodeIndices,
			});

			// Project this library's nodes into its own sphere
			this.projectSingleSphere(nodeIndices, centerX, centerY, centerZ);
		}
	}

	/** Apply multi-sphere rotations in the draw loop */
	private updateMultiSphereRotations(): void {
		if (!this.config.multiSphere || this.librarySpheres.size === 0) return;

		// Global orbit: rotate all sphere centers around the origin
		for (const [, sphere] of this.librarySpheres) {
			sphere.orbitAngle += this.globalOrbitSpeed;
			if (sphere.orbitAngle > 360) sphere.orbitAngle -= 360;

			const rad = sphere.orbitAngle * Math.PI / 180;
			const newCX = this.globalOrbitRadius * Math.cos(rad);
			const newCZ = this.globalOrbitRadius * Math.sin(rad);

			// Move all nodes relative to new sphere center
			const dx = newCX - sphere.centerX;
			const dz = newCZ - sphere.centerZ;

			for (const idx of sphere.nodeIndices) {
				this.nodes[idx].x += dx;
				this.nodes[idx].z += dz;
			}

			sphere.centerX = newCX;
			sphere.centerZ = newCZ;

			// Local rotation: rotate each node around its sphere center (Y axis)
			sphere.localRotAngle += sphere.localRotSpeed;
			const localRad = sphere.localRotSpeed * Math.PI / 180;
			const cosR = Math.cos(localRad);
			const sinR = Math.sin(localRad);

			for (const idx of sphere.nodeIndices) {
				const n = this.nodes[idx];
				const lx = n.x - sphere.centerX;
				const lz = n.z - sphere.centerZ;
				n.x = sphere.centerX + lx * cosR - lz * sinR;
				n.z = sphere.centerZ + lx * sinR + lz * cosR;
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

	// ─── Focus Mode ────────────────────────────────────────────

	setFocusNode(nodeId: string | null): void {
		if (!nodeId) {
			this.focusNodeIdx = -1;
			this.focusSet.clear();
			this.callbacks.onFocusChange?.(false);
		} else {
			this.focusNodeIdx = this.nodes.findIndex((n) => n.id === nodeId);
			if (this.focusNodeIdx >= 0) {
				this.rebuildFocusSet();
				this.callbacks.onFocusChange?.(true, this.nodes[this.focusNodeIdx].name);
			}
		}
		this.needsRedraw = true;
	}

	setFocusDepth(depth: number): void {
		this.focusDepth = depth;
		if (this.focusNodeIdx >= 0) {
			this.rebuildFocusSet();
			this.needsRedraw = true;
		}
	}

	setFocusDirection(dir: 'all' | 'incoming' | 'outgoing'): void {
		this.focusDirection = dir;
		if (this.focusNodeIdx >= 0) {
			this.rebuildFocusSet();
			this.needsRedraw = true;
		}
	}

	getFocusDirection(): 'all' | 'incoming' | 'outgoing' {
		return this.focusDirection;
	}

	private rebuildFocusSet(): void {
		this.focusSet = this.getNeighborsAtDepth(this.focusNodeIdx, this.focusDepth);
	}

	// ─── Local Graph Mode ──────────────────────────────────────

	toggleLocalGraph(): void {
		this.localGraphMode = !this.localGraphMode;
		this.needsRedraw = true;
	}

	getLocalGraphMode(): boolean {
		return this.localGraphMode;
	}

	// ─── Pin & Hide ────────────────────────────────────────────

	pinNode(nodeId: string): void {
		if (this.pinnedNodeIds.has(nodeId)) {
			this.pinnedNodeIds.delete(nodeId);
			this.worker?.postMessage({ type: 'unpinNode', id: nodeId });
		} else {
			this.pinnedNodeIds.add(nodeId);
			const node = this.nodes.find((n) => n.id === nodeId);
			if (node) {
				this.worker?.postMessage({ type: 'pinNode', id: nodeId, x: node.x, y: node.y });
			}
		}
		this.needsRedraw = true;
	}

	isNodePinned(nodeId: string): boolean {
		return this.pinnedNodeIds.has(nodeId);
	}

	hideNode(nodeId: string): void {
		this.hiddenNodeIds.add(nodeId);
		this.rebuildHiddenIndices();
		this.callbacks.onHiddenCountChange?.(this.hiddenNodeIds.size);
		this.needsRedraw = true;
	}

	showAllHidden(): void {
		this.hiddenNodeIds.clear();
		this.hiddenIndices.clear();
		this.callbacks.onHiddenCountChange?.(0);
		this.needsRedraw = true;
	}

	private rebuildHiddenIndices(): void {
		this.hiddenIndices.clear();
		for (let i = 0; i < this.nodes.length; i++) {
			if (this.hiddenNodeIds.has(this.nodes[i].id)) {
				this.hiddenIndices.add(i);
			}
		}
	}

	// ─── 3D Camera ─────────────────────────────────────────────

	/**
	 * Project a 2D graph point (gx, gy, 0) through 3D rotation + perspective.
	 * Returns screen (sx, sy, depth) where depth is used for size scaling.
	 */
	private project3D(gx: number, gy: number, gz: number, w: number, h: number): { sx: number; sy: number; depth: number } {
		// Convert degrees to radians
		const rx = this.camRotX * Math.PI / 180;
		const ry = this.camRotY * Math.PI / 180;
		const rz = this.camRotZ * Math.PI / 180;

		// Position relative to camera
		let x = (gx - this.camPosX) * this.viewScale + this.viewX;
		let y = (gy - this.camPosY) * this.viewScale + this.viewY;
		let z = (gz - this.camPosZ) * this.viewScale;

		// Rotate around Z axis (roll)
		if (rz !== 0) {
			const cosZ = Math.cos(rz), sinZ = Math.sin(rz);
			const x2 = x * cosZ - y * sinZ;
			const y2 = x * sinZ + y * cosZ;
			x = x2; y = y2;
		}

		// Rotate around X axis (pitch)
		if (rx !== 0) {
			const cosX = Math.cos(rx), sinX = Math.sin(rx);
			const y2 = y * cosX - z * sinX;
			const z2 = y * sinX + z * cosX;
			y = y2; z = z2;
		}

		// Rotate around Y axis (yaw)
		if (ry !== 0) {
			const cosY = Math.cos(ry), sinY = Math.sin(ry);
			const x2 = x * cosY + z * sinY;
			const z2 = -x * sinY + z * cosY;
			x = x2; z = z2;
		}

		// Perspective projection
		const d = this.camDistance;
		const scale = d / (d + z);

		return {
			sx: x * scale + w / 2,
			sy: y * scale + h / 2,
			depth: scale, // >1 = closer, <1 = farther
		};
	}

	/** Move camera in 3D space (for WASD controls) */
	moveCamera(dx: number, dy: number, dz: number): void {
		this.camPosX += dx;
		this.camPosY += dy;
		this.camPosZ += dz;
		this.needsRedraw = true;
	}

	resetTilt(): void {
		// Animate back to flat
		const startX = this.camRotX;
		const startY = this.camRotY;
		const startZ = this.camRotZ;
		const startPosX = this.camPosX;
		const startPosY = this.camPosY;
		const startPosZ = this.camPosZ;
		const frames = 20;
		let frame = 0;

		const animate = () => {
			frame++;
			const t = frame / frames;
			const ease = t * t * (3 - 2 * t); // smoothstep
			this.camRotX = startX * (1 - ease);
			this.camRotY = startY * (1 - ease);
			this.camRotZ = startZ * (1 - ease);
			this.camPosX = startPosX * (1 - ease);
			this.camPosY = startPosY * (1 - ease);
			this.camPosZ = startPosZ * (1 - ease);
			this.needsRedraw = true;
			if (frame < frames) {
				requestAnimationFrame(animate);
			} else {
				this.camRotX = 0;
				this.camRotY = 0;
				this.camRotZ = 0;
				this.camPosX = 0;
				this.camPosY = 0;
				this.camPosZ = 0;
				this.callbacks.onTiltChange?.(false);
			}
		};
		requestAnimationFrame(animate);
	}

	isRotated(): boolean {
		return Math.abs(this.camRotX) > 1 || Math.abs(this.camRotY) > 1 || Math.abs(this.camRotZ) > 1;
	}

	// ─── Layout Modes ──────────────────────────────────────────

	setLayoutMode(mode: LayoutMode): void {
		this.config.layoutMode = mode;

		if (mode === 'organic') {
			// Re-run force simulation — worker handles organic layout
			this.startWorker();
			return;
		}

		// Compute target positions
		const targets = mode === 'hierarchical'
			? this.computeHierarchicalLayout()
			: this.computeTemporalLayout();

		// Stop the force worker — we're using computed positions
		if (this.worker) {
			this.worker.postMessage({ type: 'stop' });
		}

		// Animate transition from current to target
		this.transitionFrom = this.nodes.map(n => ({ x: n.x, y: n.y }));
		this.transitionTo = targets;
		this.transitionProgress = 0;
		this.needsRedraw = true;
	}

	getLayoutMode(): LayoutMode {
		return this.config.layoutMode;
	}

	cycleLayoutMode(): LayoutMode {
		const modes: LayoutMode[] = ['organic', 'hierarchical', 'temporal'];
		const idx = modes.indexOf(this.config.layoutMode);
		const next = modes[(idx + 1) % modes.length];
		this.setLayoutMode(next);
		return next;
	}

	private computeHierarchicalLayout(): { x: number; y: number }[] {
		const n = this.nodes.length;
		if (n === 0) return [];

		// Find root nodes: MOCs (outgoing >= 5) or nodes with most outgoing links
		const roots: number[] = [];
		for (let i = 0; i < n; i++) {
			if (this.nodes[i].outgoingCount >= 5) roots.push(i);
		}
		// If no MOCs, pick top 3 by link count
		if (roots.length === 0) {
			const sorted = [...Array(n).keys()].sort((a, b) => this.nodes[b].linkCount - this.nodes[a].linkCount);
			roots.push(...sorted.slice(0, Math.min(3, n)));
		}

		// BFS from roots to assign levels
		const level = new Array(n).fill(-1);
		const queue: number[] = [];
		for (const r of roots) {
			if (level[r] === -1) {
				level[r] = 0;
				queue.push(r);
			}
		}
		while (queue.length > 0) {
			const cur = queue.shift()!;
			const neighbors = this.neighborMap.get(cur);
			if (!neighbors) continue;
			for (const nb of neighbors) {
				if (level[nb] === -1) {
					level[nb] = level[cur] + 1;
					queue.push(nb);
				}
			}
		}
		// Assign orphans to max level + 1
		const maxLevel = Math.max(...level.filter(l => l >= 0), 0);
		for (let i = 0; i < n; i++) {
			if (level[i] === -1) level[i] = maxLevel + 1;
		}

		// Group by level
		const levels: number[][] = [];
		for (let i = 0; i < n; i++) {
			const lv = level[i];
			while (levels.length <= lv) levels.push([]);
			levels[lv].push(i);
		}

		// Assign positions: y = level * spacing, x = spread within level
		const ySpacing = 120;
		const totalHeight = levels.length * ySpacing;
		const positions: { x: number; y: number }[] = new Array(n);

		for (let lv = 0; lv < levels.length; lv++) {
			const row = levels[lv];
			const xSpacing = Math.max(40, 800 / (row.length + 1));
			const rowWidth = (row.length - 1) * xSpacing;
			for (let j = 0; j < row.length; j++) {
				positions[row[j]] = {
					x: -rowWidth / 2 + j * xSpacing,
					y: -totalHeight / 2 + lv * ySpacing,
				};
			}
		}

		return positions;
	}

	private computeTemporalLayout(): { x: number; y: number }[] {
		const n = this.nodes.length;
		if (n === 0) return [];

		// Sort nodes by createdAt
		const indices = [...Array(n).keys()];
		const hasDate = this.nodes.some(nd => nd.createdAt > 0);

		if (!hasDate) {
			// Fallback: spread linearly by index (alphabetical order)
			indices.sort((a, b) => this.nodes[a].name.localeCompare(this.nodes[b].name));
		} else {
			indices.sort((a, b) => (this.nodes[a].createdAt || 0) - (this.nodes[b].createdAt || 0));
		}

		// Spread on x-axis, scatter y to avoid overlap
		const xSpacing = Math.max(20, 2000 / (n + 1));
		const totalWidth = (n - 1) * xSpacing;
		const positions: { x: number; y: number }[] = new Array(n);

		for (let j = 0; j < indices.length; j++) {
			const idx = indices[j];
			positions[idx] = {
				x: -totalWidth / 2 + j * xSpacing,
				y: (Math.random() - 0.5) * 200 + (this.nodes[idx].linkCount * 10), // slight y scatter by link count
			};
		}

		return positions;
	}

	// ─── BFS Traversal ─────────────────────────────────────────

	private getNeighborsAtDepth(startIdx: number, maxDepth: number): Set<number> {
		const visited = new Set<number>();
		const queue: [number, number][] = [[startIdx, 0]];
		visited.add(startIdx);

		// Pick the right neighbor map based on focus direction
		const getNeighbors = (idx: number): Set<number> | undefined => {
			if (this.focusDirection === 'outgoing') return this.outgoingMap.get(idx);
			if (this.focusDirection === 'incoming') return this.incomingMap.get(idx);
			return this.neighborMap.get(idx); // 'all'
		};

		while (queue.length > 0) {
			const [idx, depth] = queue.shift()!;
			if (depth >= maxDepth) continue;

			const neighbors = getNeighbors(idx);
			if (!neighbors) continue;

			for (const nIdx of neighbors) {
				if (!visited.has(nIdx)) {
					visited.add(nIdx);
					queue.push([nIdx, depth + 1]);
				}
			}
		}

		return visited;
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
			canvas.removeEventListener('contextmenu', this.onContextMenu);
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
				// Accept 3D positions from worker: [x0, y0, z0, x1, y1, z1, ...]
				const pos = e.data.positions as Float64Array;
				const stride = pos.length / this.nodes.length >= 2.5 ? 3 : 2; // detect 2D vs 3D
				for (let i = 0; i < this.nodes.length && i * stride + (stride - 1) < pos.length; i++) {
					this.nodes[i].x = pos[i * stride];
					this.nodes[i].y = pos[i * stride + 1];
					if (stride === 3) {
						this.nodes[i].z = pos[i * stride + 2];
					}
				}
				this.needsRedraw = true;

				if (e.data.settled && !this.didInitialFit) {
					this.didInitialFit = true;
					this.layoutSettled = true;
					this.fitToScreen();
				}
			}
		};

		const workerNodes = this.nodes.map((n) => ({ id: n.id, x: n.x, y: n.y, z: n.z, linkCount: n.linkCount }));
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

	/** Hit test in 3D mode: test against projected screen positions */
	private hitTest3D(screenX: number, screenY: number): number {
		const w = this.container.clientWidth;
		const h = this.container.clientHeight;
		const hitRadius = 15; // pixels on screen
		let closest = -1;
		let closestDist = hitRadius * hitRadius;

		for (let i = 0; i < this.nodes.length; i++) {
			if (this.hiddenIndices.has(i)) continue;
			const n = this.nodes[i];
			const p = this.project3D(n.x, n.y, n.z, w, h);
			if (p.depth <= 0) continue; // behind camera
			const dx = screenX - p.sx;
			const dy = screenY - p.sy;
			const d2 = dx * dx + dy * dy;
			if (d2 < closestDist) {
				closestDist = d2;
				closest = i;
			}
		}
		return closest;
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

		if (this.isRotating) {
			const dx = e.clientX - this.rotStartX;
			const dy = e.clientY - this.rotStartY;
			// Left/right drag = yaw (Y rotation), up/down = pitch (X rotation)
			this.camRotY = this.rotBaseY + dx * 0.4;
			this.camRotX = this.rotBaseX - dy * 0.4;
			// Ctrl held during rotation = also add roll (Z rotation)
			if (e.ctrlKey || e.metaKey) {
				this.camRotZ = this.rotBaseZ + dx * 0.2;
			}
			this.needsRedraw = true;
			const tilted = this.isRotated();
			this.callbacks.onTiltChange?.(tilted);
			return;
		}

		if (this.isPanning) {
			this.viewX = this.panViewX + (e.clientX - this.panStartX);
			this.viewY = this.panViewY + (e.clientY - this.panStartY);
			this.needsRedraw = true;
			return;
		}

		if (this.draggedNodeIdx >= 0) {
			this.isDragging = true;
			this.lastInteractionTime = performance.now();
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
		const screenX = e.clientX - rect.left;
		const screenY = e.clientY - rect.top;
		let idx: number;

		if (this.isRotated()) {
			// 3D mode: hit test against projected screen positions
			idx = this.hitTest3D(screenX, screenY);
		} else {
			// 2D mode: standard spatial grid
			const { wx, wy } = this.screenToWorld(screenX, screenY);
			this.buildSpatialGrid();
			idx = this.hitTest(wx, wy);
		}

		if (idx !== this.hoveredIdx) {
			this.hoveredIdx = idx; // Law 1: plain variable, never $state
			canvas.style.cursor = idx >= 0 ? 'pointer' : 'grab';
			this.callbacks.onNodeHover(idx >= 0 ? this.nodes[idx].name : null);
			// Send edge relationship info for hovered node's connections
			if (idx >= 0 && this.callbacks.onEdgeHover) {
				const typedEdge = this.links.find(l =>
					(l.sourceIdx === idx || l.targetIdx === idx) && l.linkType
				);
				if (typedEdge) {
					const src = this.nodes[typedEdge.sourceIdx];
					const tgt = this.nodes[typedEdge.targetIdx];
					this.callbacks.onEdgeHover({ sourceName: src.name, targetName: tgt.name, linkType: typedEdge.linkType });
				} else {
					this.callbacks.onEdgeHover(null);
				}
			}
			this.needsRedraw = true;
			// NOTICE: Worker is NEVER notified of hover. Law 2 enforced.
		}
	};

	private onPointerDown = (e: PointerEvent): void => {
		// Middle mouse button or Shift+left click = start 3D rotation
		if (e.button === 1 || (e.button === 0 && e.shiftKey)) {
			e.preventDefault();
			this.isRotating = true;
			this.lastInteractionTime = performance.now();
			this.rotStartX = e.clientX;
			this.rotStartY = e.clientY;
			this.rotBaseX = this.camRotX;
			this.rotBaseY = this.camRotY;
			this.rotBaseZ = this.camRotZ;
			const canvas = this.app?.canvas as HTMLCanvasElement;
			if (canvas) canvas.style.cursor = 'move';
			return;
		}

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
			this.lastInteractionTime = performance.now();
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

		if (this.isRotating) {
			this.isRotating = false;
			if (canvas) canvas.style.cursor = this.hoveredIdx >= 0 ? 'pointer' : 'grab';
			return;
		}

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
		this.isRotating = false;
		this.lastInteractionTime = performance.now();
		this.draggedNodeIdx = -1;
		if (this.hoveredIdx !== -1) {
			this.hoveredIdx = -1;
			this.callbacks.onNodeHover(null);
			this.needsRedraw = true;
		}
	};

	private onWheel = (e: WheelEvent): void => {
		e.preventDefault();

		if (this.isRotated()) {
			if (e.ctrlKey || e.metaKey) {
				// Ctrl+Scroll in 3D = zoom (change perspective / viewScale)
				const zoomFactor = e.deltaY > 0 ? 0.9 : 1.1;
				this.viewScale *= zoomFactor;
				this.viewScale = Math.max(0.05, Math.min(15, this.viewScale));
			} else {
				// Regular scroll in 3D = fly forward/backward
				const speed = 20;
				const ry = this.camRotY * Math.PI / 180;
				const rx = this.camRotX * Math.PI / 180;
				const dir = e.deltaY > 0 ? 1 : -1;
				this.camPosX += Math.sin(ry) * dir * speed;
				this.camPosY -= Math.sin(rx) * dir * speed;
				this.camPosZ += Math.cos(ry) * Math.cos(rx) * dir * speed;
			}
			this.needsRedraw = true;
			return;
		}

		// 2D mode: standard zoom
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

	private onContextMenu = (e: MouseEvent): void => {
		e.preventDefault();
		if (this.hoveredIdx < 0) return;
		const node = this.nodes[this.hoveredIdx];
		this.callbacks.onContextMenu?.(
			{ id: node.id, name: node.name, path: node.path, libraryName: node.libraryName },
			e.clientX,
			e.clientY
		);
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
		const now = performance.now();

		// 4D: Auto-rotate when idle (slow ambient spin like a living organism)
		if (this.autoRotateEnabled && !this.isDragging && !this.isRotating && !this.isPanning) {
			if (now - this.lastInteractionTime > this.autoRotateDelay) {
				this.camRotY += this.autoRotateSpeed;
				if (this.camRotY > 360) this.camRotY -= 360;
				if (this.camRotY < -360) this.camRotY += 360;
				this.needsRedraw = true;
			}
		}

		// Multi-sphere: rotate each library sphere locally + orbit globally
		if (this.config.multiSphere && !this.isDragging && !this.isRotating) {
			this.updateMultiSphereRotations();
		}

		// Gravity: gently pull camera position back toward sphere center (0,0,0)
		// Only when not actively dragging/rotating
		if (!this.isDragging && !this.isRotating && !this.isPanning) {
			const grav = this.cameraGravityStrength;
			const threshold = 0.5; // stop jitter below this
			if (Math.abs(this.camPosX) > threshold) { this.camPosX *= (1 - grav); this.needsRedraw = true; }
			if (Math.abs(this.camPosY) > threshold) { this.camPosY *= (1 - grav); this.needsRedraw = true; }
			if (Math.abs(this.camPosZ) > threshold) { this.camPosZ *= (1 - grav); this.needsRedraw = true; }
		}

		// Handle layout transition animation
		if (this.transitionProgress >= 0 && this.transitionProgress < this.transitionFrames) {
			this.transitionProgress++;
			const t = this.transitionProgress / this.transitionFrames;
			const ease = t * t * (3 - 2 * t); // smoothstep
			for (let i = 0; i < this.nodes.length && i < this.transitionFrom.length; i++) {
				this.nodes[i].x = this.transitionFrom[i].x + (this.transitionTo[i].x - this.transitionFrom[i].x) * ease;
				this.nodes[i].y = this.transitionFrom[i].y + (this.transitionTo[i].y - this.transitionFrom[i].y) * ease;
			}
			this.needsRedraw = true;
			if (this.transitionProgress >= this.transitionFrames) {
				this.transitionProgress = -1; // done
				this.fitToScreen();
			}
		}

		if (!this.needsRedraw || !this.app) return;
		this.needsRedraw = false;

		const w = this.container.clientWidth;
		const h = this.container.clientHeight;
		const hovered = this.hoveredIdx;
		const neighbors = hovered >= 0 ? this.neighborMap.get(hovered) : null;
		const hasSearch = this.searchQuery.length > 0;
		const dark = this.isDark;

		// Build visible set based on focus/local/hidden state
		const hasFocus = this.focusNodeIdx >= 0;
		const hasLocal = this.localGraphMode && this.activeNodeIdx >= 0;
		let visibleSet: Set<number> | null = null;

		if (hasFocus) {
			visibleSet = this.focusSet;
		} else if (hasLocal) {
			visibleSet = this.getNeighborsAtDepth(this.activeNodeIdx, 2);
		}

		// ─── Links ────
		this.linkGfx.clear();
		const normalEdgeColor = dark ? 0x475569 : 0xbcccdc;
		const normalEdgeAlpha = dark ? 0.25 : 0.15;
		const is3D = this.isRotated();

		// ─── Cluster boundaries (Phase 2, drawn first so links render on top) ────
		if (this.showClusters && this.clusterAssignments.size > 0 && hovered < 0) {
			// Group node positions by cluster
			const clusterPositions: Map<number, { xs: number[]; ys: number[] }> = new Map();
			for (const [idx, cid] of this.clusterAssignments) {
				if (this.hiddenIndices.has(idx)) continue;
				const n = this.nodes[idx];
				let sx: number, sy: number;
				if (is3D) {
					const p = this.project3D(n.x, n.y, n.z, w, h);
					sx = p.sx; sy = p.sy;
				} else {
					sx = n.x * this.viewScale + w / 2 + this.viewX;
					sy = n.y * this.viewScale + h / 2 + this.viewY;
				}
				if (!clusterPositions.has(cid)) clusterPositions.set(cid, { xs: [], ys: [] });
				clusterPositions.get(cid)!.xs.push(sx);
				clusterPositions.get(cid)!.ys.push(sy);
			}

			// Draw translucent ellipses
			for (const [cid, pos] of clusterPositions) {
				if (pos.xs.length < 3) continue;
				const cx = pos.xs.reduce((a, b) => a + b, 0) / pos.xs.length;
				const cy = pos.ys.reduce((a, b) => a + b, 0) / pos.ys.length;
				let maxDx = 0, maxDy = 0;
				for (let i = 0; i < pos.xs.length; i++) {
					maxDx = Math.max(maxDx, Math.abs(pos.xs[i] - cx));
					maxDy = Math.max(maxDy, Math.abs(pos.ys[i] - cy));
				}
				const rx = maxDx + 30, ry = maxDy + 30;
				const color = this.clusterColors.get(cid) ?? 0x7c3aed;
				this.linkGfx.ellipse(cx, cy, rx, ry);
				this.linkGfx.fill({ color, alpha: 0.06 });
				this.linkGfx.stroke({ width: 1, color, alpha: 0.15 });
			}
		}

		for (const link of this.links) {
			// Skip edges involving hidden nodes
			if (this.hiddenIndices.has(link.sourceIdx) || this.hiddenIndices.has(link.targetIdx)) continue;

			const src = this.nodes[link.sourceIdx];
			const tgt = this.nodes[link.targetIdx];

			let sx: number, sy: number, tx: number, ty: number;
			if (is3D) {
				const sp = this.project3D(src.x, src.y, src.z, w, h);
				const tp = this.project3D(tgt.x, tgt.y, tgt.z, w, h);
				sx = sp.sx; sy = sp.sy; tx = tp.sx; ty = tp.sy;
			} else {
				sx = src.x * this.viewScale + w / 2 + this.viewX;
				sy = src.y * this.viewScale + h / 2 + this.viewY;
				tx = tgt.x * this.viewScale + w / 2 + this.viewX;
				ty = tgt.y * this.viewScale + h / 2 + this.viewY;
			}

			// Focus/local mode: skip edges where both ends are outside visible set
			if (visibleSet && !visibleSet.has(link.sourceIdx) && !visibleSet.has(link.targetIdx)) continue;

			const isNeighborEdge = hovered >= 0 && (link.sourceIdx === hovered || link.targetIdx === hovered);

			if (hovered >= 0 && !isNeighborEdge) continue;

			if (hasSearch) {
				const srcMatch = this.searchMatchSet.has(link.sourceIdx);
				const tgtMatch = this.searchMatchSet.has(link.targetIdx);
				if (!srcMatch && !tgtMatch) continue;
			}

			if (isNeighborEdge) {
				this.linkGfx.moveTo(sx, sy);
				this.linkGfx.lineTo(tx, ty);
				this.linkGfx.stroke({ width: this.config.linkThickness * 2, color: HIGHLIGHT_EDGE_COLOR, alpha: 0.9 });

				// Render edge label if linkType exists
				if (link.linkType) {
					const mx = (sx + tx) / 2, my = (sy + ty) / 2;
					const key = `edge_${link.sourceIdx}_${link.targetIdx}`;
					if (!this.labelPool.has(-link.sourceIdx - 1000)) {
						const label = new Text({
							text: link.linkType,
							style: { fontSize: 9, fill: HIGHLIGHT_EDGE_COLOR, fontFamily: 'system-ui' },
						});
						label.anchor.set(0.5, 0.5);
						this.app!.stage.addChild(label);
						this.labelPool.set(-link.sourceIdx - 1000, label);
					}
					const label = this.labelPool.get(-link.sourceIdx - 1000);
					if (label) { label.x = mx; label.y = my - 8; label.visible = true; }
				}
			} else {
				this.linkGfx.moveTo(sx, sy);
				this.linkGfx.lineTo(tx, ty);
				this.linkGfx.stroke({ width: this.config.linkThickness * 0.5, color: normalEdgeColor, alpha: normalEdgeAlpha });
			}
		}

		// ─── Semantic Links (dashed, Phase 2) ────
		if (this.config.showSemanticLinks && this.semanticLinks.length > 0 && hovered < 0 && !hasSearch) {
			const semanticColor = dark ? 0x818cf8 : 0x6366f1; // indigo
			for (const sl of this.semanticLinks) {
				if (this.hiddenIndices.has(sl.sourceIdx) || this.hiddenIndices.has(sl.targetIdx)) continue;
				if (visibleSet && !visibleSet.has(sl.sourceIdx) && !visibleSet.has(sl.targetIdx)) continue;

				const src = this.nodes[sl.sourceIdx];
				const tgt = this.nodes[sl.targetIdx];
				let sx: number, sy: number, tx: number, ty: number;
				if (is3D) {
					const sp = this.project3D(src.x, src.y, src.z, w, h);
					const tp = this.project3D(tgt.x, tgt.y, tgt.z, w, h);
					sx = sp.sx; sy = sp.sy; tx = tp.sx; ty = tp.sy;
				} else {
					sx = src.x * this.viewScale + w / 2 + this.viewX;
					sy = src.y * this.viewScale + h / 2 + this.viewY;
					tx = tgt.x * this.viewScale + w / 2 + this.viewX;
					ty = tgt.y * this.viewScale + h / 2 + this.viewY;
				}

				// Draw dashed line
				const alpha = (sl.similarity ?? 0.5) * 0.6;
				const dx = tx - sx, dy = ty - sy;
				const len = Math.sqrt(dx * dx + dy * dy);
				if (len < 1) continue;
				const dashLen = 6, gapLen = 4;
				const ux = dx / len, uy = dy / len;
				let d = 0;
				while (d < len) {
					const segEnd = Math.min(d + dashLen, len);
					this.linkGfx.moveTo(sx + ux * d, sy + uy * d);
					this.linkGfx.lineTo(sx + ux * segEnd, sy + uy * segEnd);
					this.linkGfx.stroke({ width: this.config.linkThickness * 0.4, color: semanticColor, alpha });
					d = segEnd + gapLen;
				}
			}
		}

		// ─── Nodes ────
		for (let i = 0; i < this.nodes.length; i++) {
			const n = this.nodes[i];
			const gfx = this.nodeGfx[i];
			if (!gfx) continue;

			// Hidden nodes: completely invisible
			if (this.hiddenIndices.has(i)) {
				gfx.clear();
				continue;
			}

			let sx: number, sy: number, depthScale = 1;
			if (is3D) {
				const p = this.project3D(n.x, n.y, n.z, w, h);
				sx = p.sx; sy = p.sy; depthScale = p.depth;
			} else {
				sx = n.x * this.viewScale + w / 2 + this.viewX;
				sy = n.y * this.viewScale + h / 2 + this.viewY;
			}

			const isHovered = i === hovered;
			const isActive = i === this.activeNodeIdx;
			const isNeighbor = neighbors?.has(i) ?? false;
			const inVisibleSet = visibleSet ? visibleSet.has(i) : true;
			const isPinned = this.pinnedNodeIds.has(n.id);

			// Determine alpha
			let alpha = 1.0;
			if (!inVisibleSet) alpha = DIM_ALPHA;
			else if (hovered >= 0 && !isHovered && !isNeighbor) alpha = DIM_ALPHA;
			else if (hasSearch && !this.searchMatchSet.has(i) && hovered < 0) alpha = DIM_ALPHA;

			// Luminosity by recency: recently modified notes are brighter
			let luminosity = 1.0;
			if (n.createdAt > 0 && alpha === 1.0) {
				const now = Date.now();
				const age = now - n.createdAt;
				const WEEK = 7 * 24 * 60 * 60 * 1000;
				luminosity = Math.max(0.4, 1.0 - (age / (52 * WEEK))); // Fade over 1 year
			}

			const r = n.r * (isHovered ? 1.4 : isActive ? 1.3 : 1) * depthScale;
			const isOrphan = n.linkCount === 0;

			gfx.clear();
			gfx.circle(sx, sy, r);
			gfx.fill({ color: n.color, alpha: alpha * luminosity });

			// Orphan pulsing ring (dim, animated)
			if (isOrphan && alpha > DIM_ALPHA) {
				const pulse = 0.3 + 0.2 * Math.sin(Date.now() / 600 + i);
				gfx.circle(sx, sy, r + 3);
				gfx.stroke({ width: 1, color: dark ? 0x64748b : 0x94a3b8, alpha: pulse });
			}

			// Active ring
			if (isActive) {
				gfx.circle(sx, sy, r + 2);
				gfx.stroke({ width: 2, color: dark ? 0xffffff : 0x333333, alpha: 0.8 });
			}

			// Pinned indicator (cyan ring)
			if (isPinned) {
				gfx.circle(sx, sy, r + 2.5);
				gfx.stroke({ width: 2, color: 0x06b6d4, alpha: alpha });
			}

			// MOC gold ring
			if (n.outgoingCount >= 5) {
				gfx.circle(sx, sy, r + 1.5);
				gfx.stroke({ width: 1.5, color: MOC_RING_COLOR, alpha: alpha });
			}
		}

		// Orphan pulse requires continuous redraw
		if (this.config.showOrphans && this.nodes.some(n => n.linkCount === 0)) {
			this.needsRedraw = true;
		}

		// ─── 3D Axis Gizmo ────
		this.drawAxisGizmo(w, h, dark);

		// ─── Labels ────
		this.updateLabels(w, h, hovered, neighbors, dark);
	};

	/** Draw 3D axis gizmo in bottom-left corner (only when rotated) */
	private drawAxisGizmo(w: number, h: number, dark: boolean): void {
		this.gizmoGfx.clear();
		const rotated = this.isRotated();

		// Hide labels when not rotated
		for (const lbl of this.gizmoLabels) lbl.visible = false;
		if (!rotated) return;

		const cx = 60;
		const cy = h - 120;
		const axisLen = 45;

		const rx = this.camRotX * Math.PI / 180;
		const ry = this.camRotY * Math.PI / 180;
		const rz = this.camRotZ * Math.PI / 180;

		const axes = [
			{ ux: 1, uy: 0, uz: 0, color: 0xef4444 }, // X red
			{ ux: 0, uy: 1, uz: 0, color: 0x22c55e }, // Y green
			{ ux: 0, uy: 0, uz: 1, color: 0x3b82f6 }, // Z blue
		];

		for (let i = 0; i < axes.length; i++) {
			const axis = axes[i];
			let x = axis.ux, y = axis.uy, z = axis.uz;

			// Apply same rotation as camera (Z → X → Y)
			if (rz !== 0) {
				const c = Math.cos(rz), s = Math.sin(rz);
				const x2 = x * c - y * s, y2 = x * s + y * c;
				x = x2; y = y2;
			}
			if (rx !== 0) {
				const c = Math.cos(rx), s = Math.sin(rx);
				const y2 = y * c - z * s, z2 = y * s + z * c;
				y = y2; z = z2;
			}
			if (ry !== 0) {
				const c = Math.cos(ry), s = Math.sin(ry);
				const x2 = x * c + z * s, z2 = -x * s + z * c;
				x = x2; z = z2;
			}

			const ex = cx + x * axisLen;
			const ey = cy + y * axisLen;

			// Draw axis line
			this.gizmoGfx.moveTo(cx, cy);
			this.gizmoGfx.lineTo(ex, ey);
			this.gizmoGfx.stroke({ width: 2.5, color: axis.color, alpha: 0.9 });

			// Draw axis endpoint dot
			this.gizmoGfx.circle(ex, ey, 4);
			this.gizmoGfx.fill({ color: axis.color, alpha: 0.9 });

			// Position text label at end of axis (offset slightly outward)
			const labelOffset = 14;
			const dx = x !== 0 ? Math.sign(x) * labelOffset : 0;
			const dy = y !== 0 ? Math.sign(y) * labelOffset : 0;
			const lbl = this.gizmoLabels[i];
			lbl.x = ex + dx;
			lbl.y = ey + dy;
			lbl.visible = true;
		}

		// Draw center dot
		this.gizmoGfx.circle(cx, cy, 3);
		this.gizmoGfx.fill({ color: dark ? 0xffffff : 0x333333, alpha: 0.5 });
	}

	private updateLabels(w: number, h: number, hovered: number, neighbors: Set<number> | null | undefined, dark: boolean): void {
		if (!this.app) return;

		// Clean up old labels
		this.labelPool.forEach((text, idx) => {
			text.visible = false;
		});

		if (this.config.labelVisibility === 'none') return;

		const showAll = this.config.labelVisibility === 'always';
		const is3D = this.isRotated();

		for (let i = 0; i < this.nodes.length; i++) {
			const n = this.nodes[i];
			const isHovered = i === hovered;
			const isActive = i === this.activeNodeIdx;
			const isNeighbor = neighbors?.has(i) ?? false;

			if (!showAll && !isHovered && !isActive) continue;

			let sx: number, sy: number;
			if (is3D) {
				const p = this.project3D(n.x, n.y, n.z, w, h);
				sx = p.sx; sy = p.sy;
			} else {
				sx = n.x * this.viewScale + w / 2 + this.viewX;
				sy = n.y * this.viewScale + h / 2 + this.viewY;
			}

			let label = this.labelPool.get(i);
			if (!label) {
				const fontFamily = getFontForText(n.name);
				const style = new TextStyle({
					fontSize: this.config.labelFontSize,
					fontFamily,
					fill: dark ? '#e2e8f0' : '#1e293b',
					align: n.isRTL ? 'right' : 'left',
					direction: n.isRTL ? 'rtl' : 'ltr',
				});
				label = new Text({ text: n.name, style });
				label.anchor.set(n.isRTL ? 1 : 0, 0.5);
				this.app!.stage.addChild(label);
				this.labelPool.set(i, label);
			}

			label.visible = true;
			const labelR = n.r * (isHovered ? 1.4 : isActive ? 1.3 : 1);
			const offsetX = n.isRTL ? -labelR - 6 : labelR + 6;
			label.position.set(sx + offsetX, sy);

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
