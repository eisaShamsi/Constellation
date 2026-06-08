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

// Tauri's default CSP forbids `unsafe-eval`. PIXI v8 normally generates its
// WebGL shader functions at runtime via `new Function(...)` which triggers
// the CSP violation and throws
//   "Current environment does not allow unsafe-eval, please use
//    pixi.js/unsafe-eval module to enable support."
// The `pixi.js/unsafe-eval` subpath installs a pre-compiled function
// generator that doesn't use eval, and MUST be imported BEFORE any PIXI
// class is constructed. Side-effect import only — nothing to name.
// See docs/LESSONS-LEARNED.md LL-017 for the full diagnostic story.
import 'pixi.js/unsafe-eval';
import { Application, Graphics, Container, Text, TextStyle } from 'pixi.js';
// MIG-072 — palette types/defaults only (a PURE module: no Svelte/store imports, so this honours
// the "ZERO Svelte imports" law above). The resolved palette is pushed in via setPalette().
import { type SkyPalette, DEFAULT_SKY_PALETTE } from './skyPalette';

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
}

export interface EngineCallbacks {
	onNodeClick: (path: string, libraryName: string) => void;
	onNodeHover: (node: { name: string; path: string; libraryName: string } | null) => void;
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
	stratum: number; // 1–8, Knowledge Strata (CE Phase 2)
	maturity: string; // seed|sapling|evergreen|canonical|wilting (CE Phase 3)
	originType: string; // received|discovered|mixed|none (CE Phase 5)
}

interface EngineLink {
	sourceIdx: number;
	targetIdx: number;
	semantic?: boolean; // true = AI-detected, false/undefined = explicit wikilink
	similarity?: number; // 0-1 confidence for semantic links
	linkType?: string; // relationship label: supports, contradicts, elaborates, questions, custom
}

// ─── Constants ────────────────────────────────────────────────────────

// MIG-072 §3 — the hovered-edge highlight colour moved into the palette (this.palette.edgeHighlight).

// MIG-072 — the typed-link colours moved to the Style Setter's single source: they are read from the
// Link Types registry (`linkTypeColor()`) in the Svelte layer and pushed in via `this.palette.typedLinks`
// (see setPalette / skyPalette.ts). The old hardcoded `TYPED_LINK_COLORS` map was a stale duplicate that
// ignored the user's edits — removed.
// MIG-072 §3 — this.palette.dimAlpha moved into the palette (this.palette.dimAlpha), user-adjustable.
// MIG-072 §2 — MOC_RING_COLOR + MATURITY_COLORS moved into the palette (this.palette.mocRing /
// this.maturityColor()), so the Style Setter controls them. Removed.
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
	private colorMap: Record<string, string> = {};
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
	private searchMatchTypes: Map<number, Set<string>> = new Map(); // index → set of match_types

	// Search link highlights (colored lines between matched nodes)
	private searchLinkHighlights: Map<string, { color: number; bidir: boolean }> = new Map();

	// Highlight filter (from sidebar selection)
	private highlightSet: Set<number> = new Set();
	private highlightColor: number = 0x7c3aed;

	// MIG-072 — the resolved Sky View palette (every colour/intensity). Built in the Svelte layer from
	// the --skyview-* vars + the Link Types registry, pushed via setPalette(). The engine NEVER reads
	// CSS (Perf Rule 3); it just uses these values. Defaults = today's exact look until setPalette runs.
	private palette: SkyPalette = DEFAULT_SKY_PALETTE;
	private _lastLabelKey = ''; // MIG-072 §4 — detect label-style changes to rebuild pooled labels

	// CE Phase 8: Trail path overlay
	private trailNodeIndices: number[] = [];

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
	// DEFAULT_PITCH is the resting camera tilt. With pitch=0 the scene is
	// viewed head-on and auto-rotation around Y-axis appears flat (nodes
	// just shift left/right in 2D). A small negative pitch tilts the
	// "floor" of the universe away from the viewer so Y-rotation becomes
	// a visible orbital motion — the universe rotates around an imaginary
	// vertical axis through its center, like a spinning globe viewed from
	// slightly above.
	private static readonly DEFAULT_PITCH = -18; // degrees — tune to taste
	private camRotX: number = GraphEngine.DEFAULT_PITCH; // pitch (degrees)
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
	private autoRotateBaseSpeed: number = 0.1; // degrees per frame — full speed (~6°/sec @ 60fps)
	private autoRotateSlowSpeed: number = 0.01; // degrees per frame — when mouse is over canvas
	private autoRotateCurrentSpeed: number = 0.1; // current interpolated speed
	private autoRotateEnabled: boolean = true;
	private lastInteractionTime: number = 0;
	private autoRotateDelay: number = 3000; // ms of inactivity before auto-rotate starts
	private mouseOverCanvas: boolean = false;
	private lastMouseMoveTime: number = 0;
	private mouseIdleDelay: number = 2000; // ms after mouse stops moving to resume speed

	// Camera gravity: pull camera back toward center
	private cameraGravityStrength: number = 0.02; // 0 = off, higher = stronger pull

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
		rawNodes: { id: string; name: string; path: string; libraryName: string; linkCount: number; outgoingCount: number; createdAt?: number; stratum?: number; maturity?: string; originType?: string }[],
		rawLinks: { source: string; target: string; linkType?: string }[],
		colorMap: Record<string, string>
	): void {
		// Store color map for sphere borders
		this.colorMap = colorMap;

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
		// PJ-10 (post-MIG-061): count-aware damping. cache_boot_snapshot_sky
		// now returns ALL cUniverse nodes (e.g. 8 751 on Eisa Universe), and
		// §K finally delivers real `stratum` to the strata-sizing branch — so
		// foundational nodes render large and crowd the federated view.
		// Shrink node radius as the rendered set grows past ~1500. Single-
		// universe graphs (≤1500 nodes; Eisa Cognitive Knowledge ≈ 987) are
		// UNCHANGED — countDamp = 1. Floor 0.10; exponent 1.2 (PJ-10 r3, ~0.12x at 8751 nodes).
		const countDamp = filteredNodes.length <= 1500
			? 1
			: Math.max(0.10, Math.pow(1500 / filteredNodes.length, 1.2));
		this.nodes = filteredNodes.map((n, i) => {
			nodeIdMap.set(n.id, i);
			const hexStr = this.config.colorByLibrary ? (colorMap[n.libraryName] || '#a78bfa') : '#a78bfa';
			const stratum = (n as any).stratum ?? 2;
			// CE Phase 2: Earned complexity — strata sizing activates at 20+ nodes
			const useStrata = rawNodes.length >= 20 && stratum > 0;
			const baseR = useStrata
				? (2 + (stratum - 1) * 2.5)                          // stratum-based radius
				: (2 + Math.sqrt(n.linkCount) * 1.5);                // fallback: link-count
			return {
				id: n.id,
				x: (Math.random() - 0.5) * 800,
				y: (Math.random() - 0.5) * 800,
				z: (Math.random() - 0.5) * 400, // 3D depth spread
				r: Math.max(2, baseR * (n.outgoingCount >= 5 ? 1.6 : 1) * sizeMul * countDamp),
				color: hexToInt(hexStr),
				colorHex: hexStr,
				name: n.name.replace(/\.md$/, ''),
				path: n.path,
				libraryName: n.libraryName,
				linkCount: n.linkCount,
				outgoingCount: n.outgoingCount,
				isRTL: RTL_REGEX.test(n.name),
				createdAt: n.createdAt ?? 0,
				stratum,
				maturity: (n as any).maturity ?? 'seed',
				originType: (n as any).originType ?? 'none',
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

	/**
	 * MIG-072 — receive the resolved Sky View palette (mirrors setData: the engine is *told* its
	 * colours, never fetches them). Called once after init and again whenever the user's --skyview-*
	 * overrides, the live draft, the Link Types registry, or dark/light change. Pure assignment +
	 * redraw — no allocation, no CSS read.
	 */
	setPalette(palette: SkyPalette): void {
		// MIG-072 §4 — pooled Text keeps its style, so a label font/size/weight/colour change needs a
		// rebuild. Only rebuilds on an actual label-style change (not per frame / per unrelated edit).
		const labelKey = palette.label + '|' + palette.labelFamily + '|' + palette.labelSize + '|' + palette.labelWeight;
		this.palette = palette;
		if (labelKey !== this._lastLabelKey) {
			this._lastLabelKey = labelKey;
			this.labelPool.forEach((t) => t.destroy());
			this.labelPool.clear();
		}
		this.needsRedraw = true;
	}

	/** MIG-072 — maturity ring colour from the palette (no per-frame allocation; called only for
	 *  non-seed nodes, but handles every state). */
	private maturityColor(m: string): number {
		switch (m) {
			case 'sapling':   return this.palette.maturitySapling;
			case 'evergreen': return this.palette.maturityEvergreen;
			case 'canonical': return this.palette.maturityCanonical;
			case 'wilting':   return this.palette.maturityWilting;
			default:          return this.palette.maturitySeed;
		}
	}

	/** MIG-072 §2 — stroke a node ring honouring the palette's width multiplier (lighter/thicker) and
	 *  solid/dotted style. PIXI v8 Graphics has no native line-dash, so "dotted" is drawn as short arc
	 *  segments with gaps (same technique as the dashed semantic links). */
	private strokeRing(gfx: Graphics, cx: number, cy: number, r: number, baseWidth: number, color: number, alpha: number, frameId: string): void {
		const frame = this.palette.ringFrames[frameId];
		const width = baseWidth * (frame?.width ?? 1);
		if (frame?.style === 'dotted') {
			// Clean round dots evenly spaced around the ring. (Earlier we stroked short arc segments,
			// which read as spiky asterisks on small nodes.) Dot radius tracks the frame width; all dots
			// go into ONE fill() — cheaper than the per-segment strokes AND visually clean.
			const dotR = Math.max(0.6, width * 0.7);
			const circ = 2 * Math.PI * r;
			const count = Math.max(6, Math.min(28, Math.round(circ / (dotR * 5))));
			for (let i = 0; i < count; i++) {
				const a = (i / count) * 2 * Math.PI;
				gfx.circle(cx + r * Math.cos(a), cy + r * Math.sin(a), dotR);
			}
			gfx.fill({ color, alpha });
		} else {
			gfx.circle(cx, cy, r);
			gfx.stroke({ width, color, alpha });
		}
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
			// PJ-10: same count-aware damping as the build path.
			const countDamp = this.nodes.length <= 1500
				? 1
				: Math.max(0.10, Math.pow(1500 / this.nodes.length, 1.2));
			for (const n of this.nodes) {
				n.r = Math.max(2, (2 + Math.sqrt(n.linkCount) * 1.5) * (n.outgoingCount >= 5 ? 1.6 : 1) * sizeMul * countDamp);
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
					// Stronger center pull (0.1 → 0.4) so disconnected components
				// — typically driven by how few links cross between libraries —
				// are drawn into one unified constellation. Knowledge formulation
				// is ONE universe regardless of language or library.
				centerForce: 0.4,
				},
			});
		}

		this.needsRedraw = true;
	}

	setActiveNode(nodeId: string): void {
		this.activeNodeIdx = this.nodes.findIndex((n) => n.id === nodeId);
		this.needsRedraw = true;
	}

	/** Highlight all nodes matching a filter path or array of paths (for cUniverse = multiple library paths) */
	setHighlightFilter(filterPath: string | string[] | null, color: number = 0x7c3aed): void {
		this.highlightSet.clear();
		this.highlightColor = color;
		if (filterPath) {
			const paths = Array.isArray(filterPath) ? filterPath : [filterPath];
			const norms = paths.map(p => p.replace(/\\/g, '/').toLowerCase());
			for (let i = 0; i < this.nodes.length; i++) {
				const nodePath = this.nodes[i].path.replace(/\\/g, '/').toLowerCase();
				for (const norm of norms) {
					if (nodePath.startsWith(norm + '/') || nodePath === norm) {
						this.highlightSet.add(i);
						break;
					}
				}
			}
		}
		this.needsRedraw = true;
	}

	setSearch(query: string): void {
		this.searchQuery = query.toLowerCase();
		this.searchMatchSet.clear();
		this.searchMatchTypes.clear();
		this.searchLinkHighlights.clear();
		this.clearSearchBadges();
		if (this.searchQuery) {
			for (let i = 0; i < this.nodes.length; i++) {
				if (this.nodes[i].name.toLowerCase().includes(this.searchQuery)) {
					this.searchMatchSet.add(i);
					this.searchMatchTypes.set(i, new Set(['title']));
				}
			}
		}
		this.needsRedraw = true;
	}

	/** Add extended search matches from IPC hybrid search (content/semantic hits). */
	setSearchExtended(matchedIds: Set<string>, matchTypes?: Map<string, string>): void {
		for (let i = 0; i < this.nodes.length; i++) {
			if (matchedIds.has(this.nodes[i].id)) {
				this.searchMatchSet.add(i);
				if (matchTypes) {
					const mt = matchTypes.get(this.nodes[i].id);
					if (mt) {
						const existing = this.searchMatchTypes.get(i);
						if (existing) { existing.add(mt); } else { this.searchMatchTypes.set(i, new Set([mt])); }
					}
				}
			}
		}
		// Stop force simulation so badges stay positioned
		if (this.searchMatchSet.size > 0) {
			this.worker?.postMessage({ type: 'stop' });
		}
		this.needsRedraw = true;
	}

	/** MIG-072 §3 — search-badge colour from the palette (was the hardcoded BADGE_COLORS map). */
	private badgeColor(mt: string): number {
		switch (mt) {
			case 'title':    return this.palette.badgeTitle;
			case 'content':  return this.palette.badgeContent;
			case 'tag':      return this.palette.badgeTag;
			case 'property': return this.palette.badgeProperty;
			case 'wikilink': return this.palette.badgeWikilink;
			case 'semantic': return this.palette.badgeSemantic;
			default:         return this.palette.badgeStructured;
		}
	}
	private readonly BADGE_CHARS: Record<string, string> = {
		title: 'T', content: 'C', tag: '#', property: 'P', wikilink: 'W', semantic: 'S', structured: '?',
	};

	/** Badge overlay: one entry per node, with multiple badge+label pairs + name text. */
	private badgeOverlays: { idx: number; arrowGfx: Graphics; nameLabel: Text; badges: { gfx: Graphics; label: Text; color: number }[] }[] = [];

	/** Create badge objects (once). Positions updated in drawSearchBadges(). */
	renderSearchBadges(): void {
		this.clearSearchBadges();
		if (!this.app || this.searchMatchSet.size === 0) return;
		const dark = this.isDark;

		for (const idx of this.searchMatchSet) {
			const n = this.nodes[idx];
			if (!n) continue;
			const types = this.searchMatchTypes.get(idx) || new Set(['content']);

			const arrowGfx = new Graphics();
			this.app.stage.addChild(arrowGfx);

			// Name label
			const nameLabel = new Text({
				text: n.name,
				style: new TextStyle({ fontSize: 11, fill: this.palette.label, fontFamily: this.palette.labelFamily || 'system-ui', fontWeight: '500' }), // MIG-072 §4 — match label colour/font
			});
			nameLabel.anchor.set(0, 0.5);
			this.app.stage.addChild(nameLabel);

			const badges: { gfx: Graphics; label: Text; color: number }[] = [];
			for (const mt of types) {
				const color = this.badgeColor(mt);
				const ch = this.BADGE_CHARS[mt] ?? '?';
				const gfx = new Graphics();
				this.app.stage.addChild(gfx);
				const label = new Text({
					text: ch,
					style: new TextStyle({ fontSize: 10, fontWeight: 'bold', fill: '#ffffff', fontFamily: 'system-ui' }),
				});
				label.anchor.set(0.5, 0.5);
				this.app.stage.addChild(label);
				badges.push({ gfx, label, color });
			}

			this.badgeOverlays.push({ idx, arrowGfx, nameLabel, badges });
		}
		this.needsRedraw = true;
	}

	/** Reposition badges + name labels in the draw loop — follows nodes through pan/zoom/motion. */
	private drawSearchBadges(w: number, h: number): void {
		if (this.badgeOverlays.length === 0) return;
		const is3D = this.isRotated();
		const badgeSize = 14;
		const gap = 1;

		for (const b of this.badgeOverlays) {
			const n = this.nodes[b.idx];
			if (!n) {
				b.arrowGfx.visible = false;
				b.nameLabel.visible = false;
				for (const bg of b.badges) { bg.gfx.visible = false; bg.label.visible = false; }
				continue;
			}

			let sx: number, sy: number;
			if (is3D) {
				const p = this.project3D(n.x, n.y, n.z ?? 0, w, h);
				sx = p.sx; sy = p.sy;
			} else {
				sx = n.x * this.viewScale + w / 2 + this.viewX;
				sy = n.y * this.viewScale + h / 2 + this.viewY;
			}

			const r = Math.max((n.r ?? 4) * this.viewScale, 3);
			const count = b.badges.length;
			const stackHeight = count * (badgeSize + gap) - gap;

			// Badges stacked vertically above the node
			const topY = sy - r - stackHeight - 12;

			for (let i = 0; i < count; i++) {
				const bg = b.badges[i];
				const bx = sx - badgeSize / 2;
				const by = topY + i * (badgeSize + gap);
				bg.gfx.clear();
				bg.gfx.roundRect(bx, by, badgeSize, badgeSize, 3);
				bg.gfx.fill({ color: bg.color, alpha: 0.95 });
				bg.gfx.visible = true;
				bg.label.position.set(bx + badgeSize / 2, by + badgeSize / 2);
				bg.label.visible = true;
			}

			// Name label: to the right of the badge stack (or left in RTL)
			const isRTL = /[\u0600-\u06FF\u0750-\u077F\u08A0-\u08FF\uFB50-\uFDFF\uFE70-\uFEFF]/.test(n.name);
			const nameMidY = topY + stackHeight / 2;
			if (isRTL) {
				b.nameLabel.anchor.set(1, 0.5);
				b.nameLabel.position.set(sx - badgeSize / 2 - 4, nameMidY);
			} else {
				b.nameLabel.anchor.set(0, 0.5);
				b.nameLabel.position.set(sx + badgeSize / 2 + 4, nameMidY);
			}
			b.nameLabel.visible = true;

			// Black arrow from bottom of badge stack down to node
			const arrowStartY = topY + stackHeight;
			b.arrowGfx.clear();
			b.arrowGfx.moveTo(sx, arrowStartY + 2);
			b.arrowGfx.lineTo(sx, sy - r);
			b.arrowGfx.stroke({ width: 1.5, color: 0x000000, alpha: 0.6 });
			// Arrowhead
			const aLen = 5;
			b.arrowGfx.moveTo(sx, sy - r);
			b.arrowGfx.lineTo(sx - aLen * 0.5, sy - r - aLen);
			b.arrowGfx.moveTo(sx, sy - r);
			b.arrowGfx.lineTo(sx + aLen * 0.5, sy - r - aLen);
			b.arrowGfx.stroke({ width: 1.5, color: 0x000000, alpha: 0.6 });
			b.arrowGfx.visible = true;
		}
	}

	clearSearchBadges(): void {
		for (const b of this.badgeOverlays) {
			b.arrowGfx.destroy();
			b.nameLabel.destroy();
			for (const bg of b.badges) { bg.gfx.destroy(); bg.label.destroy(); }
		}
		this.badgeOverlays = [];
	}

	/** Set search matches with multiple types per node (from universalSearch categories). */
	setSearchExtendedMulti(matchedIds: Set<string>, typeMap: Map<string, Set<string>>): void {
		this.searchMatchSet.clear();
		// Keep title matches from setSearch()
		const existingTitles = new Map<number, Set<string>>();
		for (const [idx, types] of this.searchMatchTypes) {
			if (types.has('title')) existingTitles.set(idx, types);
		}
		this.searchMatchTypes.clear();
		// Restore title matches
		for (const [idx, types] of existingTitles) {
			this.searchMatchSet.add(idx);
			this.searchMatchTypes.set(idx, new Set(types));
		}
		// Add universal search matches
		for (let i = 0; i < this.nodes.length; i++) {
			const id = this.nodes[i].id;
			if (matchedIds.has(id)) {
				this.searchMatchSet.add(i);
				const types = typeMap.get(id);
				if (types) {
					const existing = this.searchMatchTypes.get(i);
					if (existing) {
						for (const t of types) existing.add(t);
					} else {
						this.searchMatchTypes.set(i, new Set(types));
					}
				}
			}
		}
		if (this.searchMatchSet.size > 0) {
			this.worker?.postMessage({ type: 'stop' });
		}
		this.needsRedraw = true;
	}

	/** Set highlighted links between matched nodes and a target, with directional colors. */
	setSearchLinkHighlights(targetId: string, matchedIds: Set<string>, direction: 'to' | 'from' | 'all' | 'mutual'): void {
		this.searchLinkHighlights.clear();
		const GREEN = 0x16a34a; // incoming
		const RED = 0xef4444;   // outgoing
		const PURPLE = 0x7c3aed; // bidirectional
		const targetIdx = this.nodes.findIndex(n => n.id === targetId);
		if (targetIdx < 0) return;

		const bidir = direction === 'all' || direction === 'mutual';

		for (let li = 0; li < this.links.length; li++) {
			const link = this.links[li];
			const srcIdx = link.sourceIdx;
			const tgtIdx = link.targetIdx;

			if (direction === 'to' || direction === 'all' || direction === 'mutual') {
				// Incoming: source links TO target — green
				if (tgtIdx === targetIdx && matchedIds.has(this.nodes[srcIdx]?.id)) {
					this.searchLinkHighlights.set(`${srcIdx}-${tgtIdx}`, { color: bidir ? PURPLE : GREEN, bidir });
				}
			}
			if (direction === 'from' || direction === 'all' || direction === 'mutual') {
				// Outgoing: target links TO source — red
				if (srcIdx === targetIdx && matchedIds.has(this.nodes[tgtIdx]?.id)) {
					const key = `${srcIdx}-${tgtIdx}`;
					const existing = this.searchLinkHighlights.get(key);
					if (existing) {
						// Already has incoming — make bidirectional
						this.searchLinkHighlights.set(key, { color: PURPLE, bidir: true });
					} else {
						this.searchLinkHighlights.set(key, { color: bidir ? PURPLE : RED, bidir });
					}
				}
			}
		}
		this.needsRedraw = true;
	}

	clearSearchLinkHighlights(): void {
		this.searchLinkHighlights.clear();
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

	/** CE Phase 8: Set trail path for overlay rendering. Paths are note file paths in order. */
	setTrailPath(notePaths: string[]): void {
		this.trailNodeIndices = [];
		for (const p of notePaths) {
			const norm = p.replace(/\\/g, '/').toLowerCase();
			const idx = this.nodes.findIndex(n => n.path.replace(/\\/g, '/').toLowerCase() === norm);
			if (idx >= 0) this.trailNodeIndices.push(idx);
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
		// 0.5× = zoom out 2× so the user sees the full constellation with
		// breathing room around the edges. Without this the fit was pegged
		// right against the viewport bounds, obscuring outer nodes during
		// 3D rotation (the sphere's corners sweep outside the screen).
		const scale = Math.min((w - pad * 2) / gw, (h - pad * 2) / gh, 3) * 0.5;

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

	/**
	 * Recolor every node using a resolver function. Used by GraphMindView
	 * when the user flips the legend's color-by mode (library → folder →
	 * stratum → maturity). Doesn't touch positions or links — just swaps
	 * the cached hex/int color per node and triggers a redraw.
	 *
	 * Cheap compared to setData: O(nodes), no worker rebuild, no layout
	 * reset. Called on every mode switch.
	 */
	setNodeColors(resolver: (node: {
		id: string;
		name: string;
		path: string;
		libraryName: string;
		linkCount: number;
		outgoingCount: number;
		stratum?: number;
		maturity?: string;
		originType?: string;
	}) => string): void {
		for (const n of this.nodes) {
			const hex = resolver(n as any);
			n.colorHex = hex;
			n.color = hexToInt(hex);
		}
		this.needsRedraw = true;
	}

	/** Move camera in 3D space (for WASD controls) */
	moveCamera(dx: number, dy: number, dz: number): void {
		this.camPosX += dx;
		this.camPosY += dy;
		this.camPosZ += dz;
		this.needsRedraw = true;
	}

	resetTilt(): void {
		// Animate back to the default orbital tilt (not flat zero — pitch=0
		// would hide the Y-axis rotation and make the universe look like a
		// flat fan). See DEFAULT_PITCH for the resting pose rationale.
		const startX = this.camRotX;
		const startY = this.camRotY;
		const startZ = this.camRotZ;
		const startPosX = this.camPosX;
		const startPosY = this.camPosY;
		const startPosZ = this.camPosZ;
		const targetX = GraphEngine.DEFAULT_PITCH;
		const frames = 20;
		let frame = 0;

		const animate = () => {
			frame++;
			const t = frame / frames;
			const ease = t * t * (3 - 2 * t); // smoothstep
			this.camRotX = startX + (targetX - startX) * ease;
			this.camRotY = startY * (1 - ease);
			this.camRotZ = startZ * (1 - ease);
			this.camPosX = startPosX * (1 - ease);
			this.camPosY = startPosY * (1 - ease);
			this.camPosZ = startPosZ * (1 - ease);
			this.needsRedraw = true;
			if (frame < frames) {
				requestAnimationFrame(animate);
			} else {
				this.camRotX = targetX;
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
		// Treat the default orbital pitch as "at rest" so the axis-gizmo
		// and reset button only appear once the user has moved the camera
		// beyond that baseline.
		return Math.abs(this.camRotX - GraphEngine.DEFAULT_PITCH) > 1
			|| Math.abs(this.camRotY) > 1
			|| Math.abs(this.camRotZ) > 1;
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
		// ⚡ Stop the render ticker IMMEDIATELY — prevents BatcherPipe from accessing
		// geometry on objects we're about to orphan, which caused the freeze + null errors.
		this.app?.ticker?.stop();

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

		// ⚡ PERF: Order matters critically here.
		// 1. Clear nodeContainer FIRST — this orphans all node Graphics objects from
		//    their parent WITHOUT Pixi traversing their render groups (O(1) per node).
		//    If we called stage.removeChildren() first while nodeContainer still had
		//    children, Pixi would recursively clean up render groups for every node → O(N) freeze.
		// 2. Then clear stage — now nodeContainer is empty so this is cheap (O(1)).
		// 3. Drop individual t.destroy() calls — Pixi's destroy() pushes to internal render
		//    pipeline arrays and throws if called while still in the hierarchy.
		//    Textures are freed by app.destroy({ texture: true }); objects are GC'd by JS.
		this.nodeContainer.removeChildren();
		this.nodeGfx = [];
		this.app?.stage.removeChildren();
		this.labelPool.clear();
		this.gizmoLabels = [];

		this.app?.destroy(true, { children: false, texture: true });
		this.app = null;
	}

	// ─── Worker (Layer 3) ──────────────────────────────────────────

	private startWorker(): void {
		// new Worker() only throws synchronously for invalid constructor args;
		// module-load failures surface asynchronously via onerror. Both paths
		// fall back to a deterministic circular layout so the user still sees
		// structure even if the physics worker can't start.
		try {
			this.worker = new Worker(new URL('./forceWorker.ts', import.meta.url), { type: 'module' });
		} catch {
			this.applyCircularLayout();
			return;
		}

		this.worker.onerror = () => {
			this.applyCircularLayout();
		};

		this.worker.onmessage = (e: MessageEvent) => {
			if (e.data.type === 'positions') {
				// Accept 3D positions from worker: [x0, y0, z0, x1, y1, z1, ...]
				const pos = e.data.positions as Float64Array;
				const stride = pos.length / this.nodes.length >= 2.5 ? 3 : 2;
				for (let i = 0; i < this.nodes.length && i * stride + (stride - 1) < pos.length; i++) {
					this.nodes[i].x = pos[i * stride];
					this.nodes[i].y = pos[i * stride + 1];
					if (stride === 3) {
						this.nodes[i].z = pos[i * stride + 2];
					}
				}
				this.needsRedraw = true;

				if (e.data.settled) {
					// Fit on every settled event, not just the first. The old
					// `didInitialFit` gate meant the first fit could land
					// before the clusters had fully merged — and subsequent
					// settle-events re-positioned everything without ever
					// re-fitting, leaving the view zoomed in. User pan/zoom
					// doesn't re-run the worker, so re-fitting on `settled`
					// only affects layout-driven updates, not interaction.
					if (!this.didInitialFit) {
						this.didInitialFit = true;
						this.layoutSettled = true;
					}
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
				// Stronger center pull (0.1 → 0.4) so disconnected components
				// — typically driven by how few links cross between libraries —
				// are drawn into one unified constellation. Knowledge formulation
				// is ONE universe regardless of language or library.
				centerForce: 0.4,
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

		// Track mouse presence for auto-rotation slowdown
		this.mouseOverCanvas = true;
		this.lastMouseMoveTime = performance.now();

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
			this.callbacks.onNodeHover(idx >= 0 ? { name: this.nodes[idx].name, path: this.nodes[idx].path, libraryName: this.nodes[idx].libraryName } : null);
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
		this.mouseOverCanvas = false;
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

		// 4D: Auto-rotate when idle (slow ambient spin like a living organism).
		// Paused while the user is hovering a node — hover = inquiry state, the
		// rotation should stop so the revealed ego-network holds still.
		if (this.autoRotateEnabled
			&& this.hoveredIdx < 0
			&& !this.isDragging
			&& !this.isRotating
			&& !this.isPanning
		) {
			// Determine target speed
			const mouseActive = this.mouseOverCanvas && (now - this.lastMouseMoveTime < this.mouseIdleDelay);
			const targetSpeed = mouseActive ? this.autoRotateSlowSpeed : this.autoRotateBaseSpeed;
			// Smooth interpolation toward target speed (ease in/out)
			const lerpRate = mouseActive ? 0.15 : 0.03; // slow down fast, speed up gradually
			this.autoRotateCurrentSpeed += (targetSpeed - this.autoRotateCurrentSpeed) * lerpRate;

			if (now - this.lastInteractionTime > this.autoRotateDelay) {
				this.camRotY += this.autoRotateCurrentSpeed;
				if (this.camRotY > 360) this.camRotY -= 360;
				if (this.camRotY < -360) this.camRotY += 360;
				this.needsRedraw = true;
			}
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
		const is3D = this.isRotated();

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
		const normalEdgeColor = this.palette.edgeNormal;       // MIG-072 §3
		const normalEdgeAlpha = this.palette.edgeNormalAlpha;  // MIG-072 §3

		// ─── Cluster boundaries (Phase 2, drawn first so links render on top) ────
		let clusterPositions: Map<number, { xs: number[]; ys: number[] }> | null = null;
		if (this.showClusters && this.clusterAssignments.size > 0 && hovered < 0) {
			// Group node positions by cluster
			clusterPositions = new Map();
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
				const color = this.clusterColors.get(cid) ?? this.palette.cluster;
				this.linkGfx.ellipse(cx, cy, rx, ry);
				this.linkGfx.fill({ color, alpha: 0.06 });
				this.linkGfx.stroke({ width: 1, color, alpha: 0.15 });
			}
		}

		// ⚡ PERF (Sky View "nervous system" design): on a 7k-node / 217k-edge
		// universe, iterating + projecting every edge every frame collapses the
		// ticker to 0.3 fps and blocks the auto-rotate. Default SV should be
		// *edgeless* — the user sees constellations drifting, then hovers a
		// node to reveal its ego-network. The loop runs only when:
		//   - a node is hovered (show its neighbor edges)
		//   - a search is active (match highlights)
		//   - focus or local-graph mode is on (scoped view)
		//   - explicit search link highlights exist
		// Cluster boundaries above this gate always render because they're
		// drawn once per cluster, not once per edge.
		const needsEdgeDraw = hovered >= 0
			|| hasSearch
			|| visibleSet !== null
			|| this.searchLinkHighlights.size > 0;

		if (needsEdgeDraw) { for (const link of this.links) {
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

			// Resolve typed link color (CE Phase 1)
			const typedColor = link.linkType ? (this.palette.typedLinks[link.linkType] ?? null) : null;

			if (isNeighborEdge) {
				const edgeColor = typedColor ?? this.palette.edgeHighlight;
				this.linkGfx.moveTo(sx, sy);
				this.linkGfx.lineTo(tx, ty);
				this.linkGfx.stroke({ width: this.config.linkThickness * 2, color: edgeColor, alpha: 0.9 });

				// Direction arrowhead at the target end — users asked to see
				// where each edge points when hovering (source → target).
				const angle = Math.atan2(ty - sy, tx - sx);
				const aLen = 9;
				const tgtR = Math.max((this.nodes[link.targetIdx].r ?? 4) * this.viewScale, 3);
				// Anchor arrowhead just outside the target node's radius so it
				// points *to* the node rather than overlapping it.
				const ax = tx - tgtR * Math.cos(angle);
				const ay = ty - tgtR * Math.sin(angle);
				this.linkGfx.moveTo(ax, ay);
				this.linkGfx.lineTo(ax - aLen * Math.cos(angle - 0.38), ay - aLen * Math.sin(angle - 0.38));
				this.linkGfx.moveTo(ax, ay);
				this.linkGfx.lineTo(ax - aLen * Math.cos(angle + 0.38), ay - aLen * Math.sin(angle + 0.38));
				this.linkGfx.stroke({ width: this.config.linkThickness * 2, color: edgeColor, alpha: 0.9 });

				// Mid-link direction arrow, color-coded from the hovered node's
				// perspective: green = outgoing (hovered is source), red =
				// incoming (hovered is target). Lets the user read each
				// connection's flow at a glance while hovering.
				const isOutgoing = link.sourceIdx === hovered;
				const midColor = isOutgoing ? this.palette.arrowOut : this.palette.arrowIn; // green / red
				const mx = (sx + tx) / 2;
				const my = (sy + ty) / 2;
				const midLen = 8;
				this.linkGfx.moveTo(mx, my);
				this.linkGfx.lineTo(mx - midLen * Math.cos(angle - 0.38), my - midLen * Math.sin(angle - 0.38));
				this.linkGfx.moveTo(mx, my);
				this.linkGfx.lineTo(mx - midLen * Math.cos(angle + 0.38), my - midLen * Math.sin(angle + 0.38));
				this.linkGfx.stroke({ width: this.config.linkThickness * 2, color: midColor, alpha: 1.0 });

				// For contradicts: draw reverse arrow too (bidirectional tension)
				if (link.linkType === 'contradicts') {
					this.linkGfx.moveTo(tx, ty);
					this.linkGfx.lineTo(sx, sy);
					this.linkGfx.stroke({ width: this.config.linkThickness * 2, color: edgeColor, alpha: 0.6 });
					// Mirror arrowhead at the source end for the contradicts case.
					const rAngle = angle + Math.PI;
					const srcR = Math.max((this.nodes[link.sourceIdx].r ?? 4) * this.viewScale, 3);
					const rax = sx - srcR * Math.cos(rAngle);
					const ray = sy - srcR * Math.sin(rAngle);
					this.linkGfx.moveTo(rax, ray);
					this.linkGfx.lineTo(rax - aLen * Math.cos(rAngle - 0.38), ray - aLen * Math.sin(rAngle - 0.38));
					this.linkGfx.moveTo(rax, ray);
					this.linkGfx.lineTo(rax - aLen * Math.cos(rAngle + 0.38), ray - aLen * Math.sin(rAngle + 0.38));
					this.linkGfx.stroke({ width: this.config.linkThickness * 2, color: edgeColor, alpha: 0.6 });
				}

				// Render edge label if linkType exists
				if (link.linkType) {
					const mx = (sx + tx) / 2, my = (sy + ty) / 2;
					if (!this.labelPool.has(-link.sourceIdx - 1000)) {
						const label = new Text({
							text: link.linkType,
							style: { fontSize: 9, fill: edgeColor, fontFamily: 'system-ui' },
						});
						label.anchor.set(0.5, 0.5);
						this.app!.stage.addChild(label);
						this.labelPool.set(-link.sourceIdx - 1000, label);
					}
					const label = this.labelPool.get(-link.sourceIdx - 1000);
					if (label) { label.x = mx; label.y = my - 8; label.visible = true; }
				}
			} else {
				// Normal state: typed links show their color at low opacity; untyped use normalEdgeColor
				const edgeColor = typedColor ?? normalEdgeColor;
				const edgeAlpha = typedColor ? (normalEdgeAlpha * 2.5) : normalEdgeAlpha; // typed links slightly more visible
				const edgeWidth = link.linkType === 'causes'
					? this.config.linkThickness * 0.8   // causes = thicker
					: this.config.linkThickness * 0.5;
				this.linkGfx.moveTo(sx, sy);
				this.linkGfx.lineTo(tx, ty);
				this.linkGfx.stroke({ width: edgeWidth, color: edgeColor, alpha: edgeAlpha });
			}

				// Search link highlight: thick colored line with directional arrowheads
				const hlKey = `${link.sourceIdx}-${link.targetIdx}`;
				const hlData = this.searchLinkHighlights.get(hlKey);
				if (hlData) {
					const { color: hlColor, bidir } = hlData;
					this.linkGfx.moveTo(sx, sy);
					this.linkGfx.lineTo(tx, ty);
					this.linkGfx.stroke({ width: 3, color: hlColor, alpha: 0.9 });
					// Arrowhead at target end
					const angle = Math.atan2(ty - sy, tx - sx);
					const aLen = 10;
					this.linkGfx.moveTo(tx, ty);
					this.linkGfx.lineTo(tx - aLen * Math.cos(angle - 0.35), ty - aLen * Math.sin(angle - 0.35));
					this.linkGfx.moveTo(tx, ty);
					this.linkGfx.lineTo(tx - aLen * Math.cos(angle + 0.35), ty - aLen * Math.sin(angle + 0.35));
					this.linkGfx.stroke({ width: 3, color: hlColor, alpha: 0.9 });
					// Arrowhead at source end (for bidir or "links from")
					if (bidir) {
						const rAngle = angle + Math.PI;
						this.linkGfx.moveTo(sx, sy);
						this.linkGfx.lineTo(sx - aLen * Math.cos(rAngle - 0.35), sy - aLen * Math.sin(rAngle - 0.35));
						this.linkGfx.moveTo(sx, sy);
						this.linkGfx.lineTo(sx - aLen * Math.cos(rAngle + 0.35), sy - aLen * Math.sin(rAngle + 0.35));
						this.linkGfx.stroke({ width: 3, color: hlColor, alpha: 0.9 });
					}
				}
			} } // closes `for` loop and `if (needsEdgeDraw)` gate

			// ─── Semantic Links (dashed, Phase 2) ────
		if (this.config.showSemanticLinks && this.semanticLinks.length > 0 && hovered < 0 && !hasSearch) {
			const semanticColor = this.palette.semantic; // indigo (MIG-072 §3)
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
				const alpha = (sl.similarity ?? 0.5) * this.palette.semanticAlphaMul;
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
			if (!inVisibleSet) alpha = this.palette.dimAlpha;
			else if (hovered >= 0 && !isHovered && !isNeighbor) alpha = this.palette.dimAlpha;
			else if (hasSearch && !this.searchMatchSet.has(i) && hovered < 0) alpha = this.palette.dimAlpha;

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
			// MIG-072 — base fill is the library/folder colour; when a node falls back to the app default
			// purple, the Style Setter's "Node default" (--skyview-node-default) takes over.
			gfx.fill({ color: n.color === 0xa78bfa ? this.palette.nodeDefault : n.color, alpha: alpha * luminosity });

			// MIG-072 §2 (mock-up-approved): glows are diffuse halos drawn FIRST (behind), then the
			// applicable rings STACK into evenly-spaced bands just below — so 2-3 co-occurring rings stay
			// individually legible. dense = PJ-10 r3 damping for big graphs.
			const dense = this.nodes.length > 1500 ? 0.5 : 1;

			// CE Phase 2: Stratum glow halo — complementary color for max contrast
			if (n.stratum >= 4 && this.nodes.length >= 20) {
				gfx.circle(sx, sy, r + 5 * (this.nodes.length > 1500 ? 0.5 : 1)); // PJ-10 r3: halve frame in dense mode
				gfx.fill({ color: complementaryColor(n.color), alpha: (n.stratum - 3) * this.palette.stratumGlowAlphaUnit * alpha });
			}

			// CE Phase 5: Provenance origin glow — blue (received) / amber (discovered)
			if (n.originType === 'received' || n.originType === 'discovered') {
				const oColor = n.originType === 'received' ? this.palette.glowReceived : this.palette.glowDiscovered;
				gfx.circle(sx, sy, r + 6 * (this.nodes.length > 1500 ? 0.5 : 1)); // PJ-10 r3
				gfx.fill({ color: oColor, alpha: this.palette.glowOriginAlpha * alpha });
			}

			// Collect the applicable rings inner-to-outer, then draw them stacked with even gaps.
			const ringStack: { color: number; baseWidth: number; alpha: number; frameId: string }[] = [];
			if (n.maturity && n.maturity !== 'seed') {
				ringStack.push({ color: this.maturityColor(n.maturity), baseWidth: 1.5 * dense, alpha: (n.maturity === 'wilting' ? 0.3 : 0.7) * alpha, frameId: n.maturity });
			}
			if (n.outgoingCount >= 5) ringStack.push({ color: this.palette.mocRing, baseWidth: 1.5 * dense, alpha, frameId: 'moc' });
			if (isOrphan && alpha > this.palette.dimAlpha) {
				const pulse = 0.3 + 0.2 * Math.sin(Date.now() / 600 + i);
				ringStack.push({ color: this.palette.ringOrphan, baseWidth: 1 * dense, alpha: pulse, frameId: 'orphan' });
			}
			if (isPinned) ringStack.push({ color: this.palette.ringPinned, baseWidth: 2 * dense, alpha, frameId: 'pinned' });
			if (isActive) ringStack.push({ color: this.palette.ringActive, baseWidth: 2 * dense, alpha: 0.8, frameId: 'active' });
			if (this.highlightSet.has(i)) ringStack.push({ color: this.highlightColor, baseWidth: 2.5 * dense, alpha: 0.9, frameId: 'selection' });

			const ringGap = this.palette.ringGap * dense; // user-set separation between stacked rings
			let ringR = r + this.palette.ringBase * dense; // user-set first-ring gap (from the fill)
			for (const ring of ringStack) {
				this.strokeRing(gfx, sx, sy, ringR, ring.baseWidth, ring.color, ring.alpha, ring.frameId);
				ringR += ringGap;
			}
		}

		// CE Phase 8: Trail path overlay — thick colored line connecting trail notes in order
		if (this.trailNodeIndices.length >= 2 && this.app) {
			const TRAIL_COLOR = this.palette.trail; // MIG-072 §3
			const w = this.app.screen.width;
			const h = this.app.screen.height;
			for (let i = 0; i < this.trailNodeIndices.length - 1; i++) {
				const a = this.nodes[this.trailNodeIndices[i]];
				const b = this.nodes[this.trailNodeIndices[i + 1]];
				if (!a || !b) continue;
				const pa = this.project3D(a.x + this.viewX, a.y + this.viewY, a.z, w, h);
				const pb = this.project3D(b.x + this.viewX, b.y + this.viewY, b.z, w, h);
				this.linkGfx.moveTo(pa.sx, pa.sy);
				this.linkGfx.lineTo(pb.sx, pb.sy);
				this.linkGfx.stroke({ width: 3, color: TRAIL_COLOR, alpha: 0.8 });
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

		// ─── Search Badges ────
		this.drawSearchBadges(w, h);
	};

	/** Draw 3D axis gizmo in bottom-left corner (only when rotated) */
	/** Draw translucent sphere borders around each library in multi-sphere mode */
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
			{ ux: 1, uy: 0, uz: 0, color: this.palette.gizmoX }, // X (MIG-072 §4)
			{ ux: 0, uy: 1, uz: 0, color: this.palette.gizmoY }, // Y
			{ ux: 0, uy: 0, uz: 1, color: this.palette.gizmoZ }, // Z
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
			lbl.style.fill = axis.color; // MIG-072 §4 — match the palette axis colour
			lbl.x = ex + dx;
			lbl.y = ey + dy;
			lbl.visible = true;
		}

		// Draw center dot
		this.gizmoGfx.circle(cx, cy, 3);
		this.gizmoGfx.fill({ color: this.palette.gizmoCentre, alpha: 0.5 }); // MIG-072 §4
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

			// When hovering, also label every neighbor so the user can read the
			// names of the connected notes without chasing each edge one-by-one.
			if (!showAll && !isHovered && !isActive && !isNeighbor) continue;

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
				// MIG-072 §4 — label font from the palette: family (override or script-aware), size (Setter
				// wins over the ⚙ panel when set), weight, colour. Pooled labels are recreated on a label-
				// style change via setPalette() so edits apply.
				const fontFamily = this.palette.labelFamily || getFontForText(n.name);
				const style = new TextStyle({
					fontSize: this.palette.labelSize > 0 ? this.palette.labelSize : this.config.labelFontSize,
					fontFamily,
					fontWeight: String(this.palette.labelWeight) as any,
					fill: this.palette.label,
					align: n.isRTL ? 'right' : 'left',
					// @ts-ignore — direction is a valid PixiJS TextStyle property but not in the TS types
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
				label.alpha = this.palette.dimAlpha;
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

/** Compute the complementary color (180° hue rotation) for maximum contrast. */
function complementaryColor(hex: number): number {
	let r = ((hex >> 16) & 0xFF) / 255;
	let g = ((hex >> 8) & 0xFF) / 255;
	let b = (hex & 0xFF) / 255;
	const max = Math.max(r, g, b), min = Math.min(r, g, b);
	let h = 0, s = 0;
	const l = (max + min) / 2;
	if (max !== min) {
		const d = max - min;
		s = l > 0.5 ? d / (2 - max - min) : d / (max + min);
		if (max === r) h = ((g - b) / d + (g < b ? 6 : 0)) / 6;
		else if (max === g) h = ((b - r) / d + 2) / 6;
		else h = ((r - g) / d + 4) / 6;
	}
	h = (h + 0.5) % 1; // rotate 180°
	// HSL to RGB
	const hue2rgb = (p: number, q: number, t: number) => {
		if (t < 0) t += 1; if (t > 1) t -= 1;
		if (t < 1/6) return p + (q - p) * 6 * t;
		if (t < 1/2) return q;
		if (t < 2/3) return p + (q - p) * (2/3 - t) * 6;
		return p;
	};
	if (s === 0) { r = g = b = l; } else {
		const q = l < 0.5 ? l * (1 + s) : l + s - l * s;
		const p = 2 * l - q;
		r = hue2rgb(p, q, h + 1/3);
		g = hue2rgb(p, q, h);
		b = hue2rgb(p, q, h - 1/3);
	}
	return ((Math.round(r * 255) << 16) | (Math.round(g * 255) << 8) | Math.round(b * 255));
}
