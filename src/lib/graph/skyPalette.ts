/**
 * MIG-072 — Sky View palette (Option B: "follow the SV colouring mechanism").
 *
 * The Sky View renderers (PIXI `graphEngine.ts` + Canvas-2D `LocalSkyView.svelte`) are *told* their
 * colours — the engine never reads CSS variables (Perf Rule 3: zero `getComputedStyle` in `draw()`).
 * This module is the SINGLE place that resolves every Sky View colour/intensity, from:
 *   (a) a per-Universe CSS var (`--skyview-*`) the Style Setter writes into `appSettings.styleOverride`
 *       / the live draft, falling back to TODAY'S EXACT hardcoded value → unset = pixel-identical; and
 *   (b) the typed-link colours the caller passes in (built from `linkTypeColor()` — the user-editable
 *       registry, the single source — read in the Svelte layer so this module stays pure/DOM-free).
 *
 * `resolveSkyPalette()` is PURE (no DOM, no Svelte, no store imports) so the plain-TS engine can import
 * its type + default without pulling reactive deps into the render core. The Svelte layer wraps it in a
 * `$derived` and pushes the result via `setPalette()`.
 *
 * WIRING AUDIT NOTE: these vars are JS-consumed (a canvas cannot read CSS vars), so they are wired here
 * via `SKY_PALETTE_COLOR_VARS` / `SKY_PALETTE_ALPHA_VARS` rather than as `var(--X)` in CSS. The audit
 * recognises a `--skyview-*` control as wired when its name appears in one of those tables.
 */

/** One resolved palette — all colours are PIXI ints (0xRRGGBB); alphas are 0..1. */
export interface SkyPalette {
	// Nodes
	nodeDefault: number;        // base fill when not coloured by library/folder
	ringActive: number;         // the open note's ring
	ringPinned: number;         // pinned ring
	ringOrphan: number;         // orphan (0-link) pulsing ring
	// Knowledge layers
	glowReceived: number;       // provenance: received
	glowDiscovered: number;     // provenance: discovered
	mocRing: number;            // 5+ outgoing links
	maturitySeed: number;
	maturitySapling: number;
	maturityEvergreen: number;
	maturityCanonical: number;
	maturityWilting: number;
	// Links
	edgeNormal: number;         // untyped edge
	edgeHighlight: number;      // hovered untyped edge
	arrowOut: number;           // hover mid-arrow, outgoing
	arrowIn: number;            // hover mid-arrow, incoming
	semantic: number;           // AI dashed link
	cluster: number;            // cluster boundary ellipse
	trail: number;              // trail path overlay
	typedLinks: Record<string, number>; // from the Link Types registry (single source), passed by caller
	// Labels & overlays
	label: number;              // node name text
	gizmoX: number;
	gizmoY: number;
	gizmoZ: number;
	gizmoCentre: number;        // gizmo centre dot
	badgeTitle: number;
	badgeContent: number;
	badgeTag: number;
	badgeProperty: number;
	badgeWikilink: number;
	badgeSemantic: number;
	badgeStructured: number;
	// In-scope "intensity" alphas (owner scope choice A)
	edgeNormalAlpha: number;    // untyped edge opacity (mode-dependent default)
	semanticAlphaMul: number;   // semantic opacity = similarity * this
	glowOriginAlpha: number;    // provenance glow opacity
	stratumGlowAlphaUnit: number; // stratum glow opacity per level above 3
	dimAlpha: number;           // dimmed (out-of-focus) opacity
	// Per-ring frame stroke (MIG-072 §2 — Eisa): EACH node ring has its own width multiplier + solid/dotted.
	// Keyed by ring id: active · pinned · orphan · sapling · evergreen · canonical · wilting · moc.
	ringFrames: Record<string, { width: number; style: string }>;
	// Ring-stack spacing (MIG-072 §2 mock-up): first-ring gap from the fill + gap between stacked rings.
	ringBase: number;
	ringGap: number;
}

type ModeDefault = { dark: number; light: number; var: string };

/**
 * Colour fields: today's EXACT constants (dark/light pair) + the Style-Setter var name.
 * When the var is unset the mode default applies (dark/light each keep their original look); a set
 * value applies to BOTH modes (one var per element — owner scope choice 2).
 */
const SKY_COLOR_FIELDS: Record<string, ModeDefault> = {
	nodeDefault:      { dark: 0xa78bfa, light: 0xa78bfa, var: '--skyview-node-default' },
	ringActive:       { dark: 0xffffff, light: 0x333333, var: '--skyview-ring-active' },
	ringPinned:       { dark: 0x06b6d4, light: 0x06b6d4, var: '--skyview-ring-pinned' },
	ringOrphan:       { dark: 0x64748b, light: 0x94a3b8, var: '--skyview-ring-orphan' },
	glowReceived:     { dark: 0x4a9eff, light: 0x4a9eff, var: '--skyview-glow-received' },
	glowDiscovered:   { dark: 0xffb347, light: 0xffb347, var: '--skyview-glow-discovered' },
	mocRing:          { dark: 0xf59e0b, light: 0xf59e0b, var: '--skyview-moc-ring' },
	maturitySeed:     { dark: 0x999999, light: 0x999999, var: '--skyview-maturity-seed' },
	maturitySapling:  { dark: 0x4ade80, light: 0x4ade80, var: '--skyview-maturity-sapling' },
	maturityEvergreen:{ dark: 0x16a34a, light: 0x16a34a, var: '--skyview-maturity-evergreen' },
	maturityCanonical:{ dark: 0xf59e0b, light: 0xf59e0b, var: '--skyview-maturity-canonical' },
	maturityWilting:  { dark: 0x16a34a, light: 0x16a34a, var: '--skyview-maturity-wilting' },
	edgeNormal:       { dark: 0x475569, light: 0xbcccdc, var: '--skyview-edge-normal' },
	edgeHighlight:    { dark: 0xf97316, light: 0xf97316, var: '--skyview-edge-highlight' },
	arrowOut:         { dark: 0x22c55e, light: 0x22c55e, var: '--skyview-arrow-out' },
	arrowIn:          { dark: 0xef4444, light: 0xef4444, var: '--skyview-arrow-in' },
	semantic:         { dark: 0x818cf8, light: 0x6366f1, var: '--skyview-semantic' },
	cluster:          { dark: 0x7c3aed, light: 0x7c3aed, var: '--skyview-cluster' },
	trail:            { dark: 0xff6b6b, light: 0xff6b6b, var: '--skyview-trail' },
	label:            { dark: 0xe2e8f0, light: 0x1e293b, var: '--skyview-label' },
	gizmoX:           { dark: 0xef4444, light: 0xef4444, var: '--skyview-gizmo-x' },
	gizmoY:           { dark: 0x22c55e, light: 0x22c55e, var: '--skyview-gizmo-y' },
	gizmoZ:           { dark: 0x3b82f6, light: 0x3b82f6, var: '--skyview-gizmo-z' },
	gizmoCentre:      { dark: 0xffffff, light: 0x333333, var: '--skyview-gizmo-centre' },
	badgeTitle:       { dark: 0x3b82f6, light: 0x3b82f6, var: '--skyview-badge-title' },
	badgeContent:     { dark: 0x16a34a, light: 0x16a34a, var: '--skyview-badge-content' },
	badgeTag:         { dark: 0xf472b6, light: 0xf472b6, var: '--skyview-badge-tag' },
	badgeProperty:    { dark: 0xf59e0b, light: 0xf59e0b, var: '--skyview-badge-property' },
	badgeWikilink:    { dark: 0x60a5fa, light: 0x60a5fa, var: '--skyview-badge-wikilink' },
	badgeSemantic:    { dark: 0x7c3aed, light: 0x7c3aed, var: '--skyview-badge-semantic' },
	badgeStructured:  { dark: 0x94a3b8, light: 0x94a3b8, var: '--skyview-badge-structured' },
};

/** Alpha fields (0..1) + var name. edgeNormalAlpha is mode-dependent (handled in defaults). */
const SKY_ALPHA_FIELDS: Record<string, { def: number; var: string }> = {
	semanticAlphaMul:     { def: 0.6,  var: '--skyview-semantic-alpha' },
	glowOriginAlpha:      { def: 0.06, var: '--skyview-glow-alpha' },
	stratumGlowAlphaUnit: { def: 0.08, var: '--skyview-stratum-alpha' },
	dimAlpha:             { def: 0.12, var: '--skyview-dim-alpha' },
};
const EDGE_NORMAL_ALPHA_VAR = '--skyview-edge-normal-alpha';

/** Per-ring frame: each ring id → its width + style var names (literal strings = the audit's consumers). */
const FRAME_VARS: Record<string, { width: string; style: string }> = {
	active:    { width: '--skyview-frame-active-width',    style: '--skyview-frame-active-style' },
	pinned:    { width: '--skyview-frame-pinned-width',    style: '--skyview-frame-pinned-style' },
	orphan:    { width: '--skyview-frame-orphan-width',    style: '--skyview-frame-orphan-style' },
	sapling:   { width: '--skyview-frame-sapling-width',   style: '--skyview-frame-sapling-style' },
	evergreen: { width: '--skyview-frame-evergreen-width', style: '--skyview-frame-evergreen-style' },
	canonical: { width: '--skyview-frame-canonical-width', style: '--skyview-frame-canonical-style' },
	wilting:   { width: '--skyview-frame-wilting-width',   style: '--skyview-frame-wilting-style' },
	moc:       { width: '--skyview-frame-moc-width',       style: '--skyview-frame-moc-style' },
};

/** Audit tables — every Style-Setter `--skyview-*` control must appear here (its JS consumer). */
export const SKY_PALETTE_COLOR_VARS: string[] = Object.values(SKY_COLOR_FIELDS).map((f) => f.var);
export const SKY_PALETTE_ALPHA_VARS: string[] = [
	...Object.values(SKY_ALPHA_FIELDS).map((f) => f.var),
	EDGE_NORMAL_ALPHA_VAR,
];

export function hexColorToInt(v: string | undefined): number | null {
	if (!v) return null;
	let s = v.trim().replace('#', '');
	if (s.length === 3) s = s[0] + s[0] + s[1] + s[1] + s[2] + s[2];
	if (s.length !== 6) return null;
	const n = parseInt(s, 16);
	return Number.isNaN(n) ? null : n;
}

function parseAlpha(v: string | undefined): number | null {
	if (v === undefined || v === '') return null;
	const n = parseFloat(v);
	return Number.isNaN(n) ? null : Math.max(0, Math.min(1, n));
}

/** All-defaults palette for a mode (no user overrides). typedLinks default empty (caller fills it). */
export function makeDefaultSkyPalette(isDark: boolean): SkyPalette {
	const out: Record<string, unknown> = {};
	for (const [k, d] of Object.entries(SKY_COLOR_FIELDS)) out[k] = isDark ? d.dark : d.light;
	out.edgeNormalAlpha = isDark ? 0.25 : 0.15;
	for (const [k, a] of Object.entries(SKY_ALPHA_FIELDS)) out[k] = a.def;
	const frames: Record<string, { width: number; style: string }> = {};
	for (const id of Object.keys(FRAME_VARS)) frames[id] = { width: 1.5, style: 'solid' };
	out.ringFrames = frames;
	out.ringBase = 1.5;
	out.ringGap = 2.6;
	out.typedLinks = {};
	return out as unknown as SkyPalette;
}

/** Static fallback the engine holds before the Svelte layer calls setPalette(). */
export const DEFAULT_SKY_PALETTE: SkyPalette = makeDefaultSkyPalette(false);

/**
 * Resolve the full palette from the merged Style-Setter vars (styleOverride + live draft).
 * `vars` keys are CSS var names (e.g. '--skyview-ring-active') → hex/number string values.
 * `typedLinks` is the registry-derived id→int map, built in the Svelte layer.
 */
export function resolveSkyPalette(
	vars: Record<string, string>,
	isDark: boolean,
	typedLinks: Record<string, number>
): SkyPalette {
	const p = makeDefaultSkyPalette(isDark) as unknown as Record<string, unknown>;
	for (const [k, d] of Object.entries(SKY_COLOR_FIELDS)) {
		const v = hexColorToInt(vars[d.var]);
		if (v !== null) p[k] = v;
	}
	for (const [k, a] of Object.entries(SKY_ALPHA_FIELDS)) {
		const v = parseAlpha(vars[a.var]);
		if (v !== null) p[k] = v;
	}
	const ena = parseAlpha(vars[EDGE_NORMAL_ALPHA_VAR]);
	if (ena !== null) p.edgeNormalAlpha = ena;
	// Node frame stroke — width multiplier + solid/dotted (consumers of these vars live here).
	const frames: Record<string, { width: number; style: string }> = {};
	for (const [id, vs] of Object.entries(FRAME_VARS)) {
		const w = parseFloat(vars[vs.width]);
		const s = vars[vs.style];
		frames[id] = {
			width: (!Number.isNaN(w) && w > 0) ? Math.min(5, Math.max(0.1, w)) : 1.5,
			style: s === 'dotted' ? 'dotted' : 'solid',
		};
	}
	p.ringFrames = frames;
	const rb = parseFloat(vars['--skyview-ring-base']);
	if (!Number.isNaN(rb) && rb >= 0) p.ringBase = Math.min(10, rb);
	const rg = parseFloat(vars['--skyview-ring-gap']);
	if (!Number.isNaN(rg) && rg > 0) p.ringGap = Math.min(12, rg);
	p.typedLinks = typedLinks;
	return p as unknown as SkyPalette;
}
