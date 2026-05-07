/**
 * Sight v3 — Rim-axis mode definitions (the 6 toggleable views).
 *
 * MIG-019 §2G.2. Per `docs/SIGHT-V3-VISUAL-SPEC.md` §1, the chart is a
 * multi-lens diagnostic instrument. The same Universe is read from six
 * different cognitive angles by switching the rim axis. Radius (centrality)
 * and color (community) stay invariant; only azimuth changes.
 *
 * Each mode declares: how its wedges are formed, what label to show on
 * each wedge, and whether it's currently backed by data. Modes with
 * `ready: false` render as dimmed buttons in the toggle bar with a
 * "Available later" tooltip — the implementation lights up when the
 * corresponding Concept Paper P2/P3/P4 features ship.
 */

export type SightMode = 'regions' | 'linkTypes' | 'time' | 'confidence' | 'stages' | 'acts';

/** Wedge labels are user-facing — they go through `$t()` at render time.
 *  Here we list the i18n KEYS, not the strings themselves. */
export interface ModeMeta {
    /** Internal id, also stored in `appSettings.sight.lastMode`. */
    id: SightMode;
    /** Single-letter label for the toggle button (R/L/T/C/S/A). */
    letter: string;
    /** i18n key for the full mode name (e.g., "sight.v3.mode.regions"). */
    nameKey: string;
    /** i18n key for the one-line description shown below the toggle bar. */
    captionKey: string;
    /** Backed by current data? If false, render as dimmed "Available later". */
    ready: boolean;
    /** Wedge count: fixed (number) or 'variable' (computed from data). */
    wedges: number | 'variable';
    /** For non-variable modes: the wedge labels (i18n keys). */
    wedgeLabels: ReadonlyArray<string> | null;
    /** What data field defines a note's azimuth in this mode. Used in
     *  the visual-spec doc — not consumed at runtime, but kept here so
     *  the contract is single-sourced. */
    azimuthRule: string;
}

/** All 6 modes in canonical toggle order: ready first (R · L · T),
 *  then "available later" (C · S · A). */
export const MODES: ReadonlyArray<ModeMeta> = Object.freeze([
    {
        id: 'regions',
        letter: 'R',
        nameKey: 'sight.v3.mode.regions.name',
        captionKey: 'sight.v3.mode.regions.caption',
        ready: true,
        wedges: 'variable',
        wedgeLabels: null,
        azimuthRule: 'library_path of the note',
    },
    {
        id: 'linkTypes',
        letter: 'L',
        nameKey: 'sight.v3.mode.linkTypes.name',
        captionKey: 'sight.v3.mode.linkTypes.caption',
        ready: true,
        wedges: 7,
        wedgeLabels: Object.freeze([
            'sight.v3.linkType.supports',
            'sight.v3.linkType.contradicts',
            'sight.v3.linkType.causes',
            'sight.v3.linkType.exemplifies',
            'sight.v3.linkType.generalizes',
            'sight.v3.linkType.derivesFrom',
            'sight.v3.linkType.partOf',
        ]),
        azimuthRule: 'dominant outgoing link type',
    },
    {
        id: 'time',
        letter: 'T',
        nameKey: 'sight.v3.mode.time.name',
        captionKey: 'sight.v3.mode.time.caption',
        ready: true,
        wedges: 'variable',
        wedgeLabels: null,
        azimuthRule: 'created_at year (with month sub-divisions on the most recent year)',
    },
    {
        id: 'confidence',
        letter: 'C',
        nameKey: 'sight.v3.mode.confidence.name',
        captionKey: 'sight.v3.mode.confidence.caption',
        ready: false,
        wedges: 4,
        wedgeLabels: Object.freeze([
            'sight.v3.confidence.hypothesis',
            'sight.v3.confidence.evidence',
            'sight.v3.confidence.established',
            'sight.v3.confidence.contested',
        ]),
        azimuthRule: 'confidence of strongest outgoing link',
    },
    {
        id: 'stages',
        letter: 'S',
        nameKey: 'sight.v3.mode.stages.name',
        captionKey: 'sight.v3.mode.stages.caption',
        ready: false,
        wedges: 6,
        wedgeLabels: Object.freeze([
            'sight.v3.stage.spark',
            'sight.v3.stage.birth',
            'sight.v3.stage.growth',
            'sight.v3.stage.maturity',
            'sight.v3.stage.dormancy',
            'sight.v3.stage.archival',
        ]),
        azimuthRule: 'dominant lifecycle stage of links',
    },
    {
        id: 'acts',
        letter: 'A',
        nameKey: 'sight.v3.mode.acts.name',
        captionKey: 'sight.v3.mode.acts.caption',
        ready: false,
        wedges: 5,
        wedgeLabels: Object.freeze([
            'sight.v3.act.observation',
            'sight.v3.act.connection',
            'sight.v3.act.tension',
            'sight.v3.act.synthesis',
            'sight.v3.act.conviction',
        ]),
        azimuthRule: 'which Act produced the note',
    },
]);

/** Quick lookup by id. */
export const MODE_BY_ID: ReadonlyMap<SightMode, ModeMeta> = Object.freeze(
    new Map(MODES.map((m) => [m.id, m])),
);

/** The default mode for a fresh Universe (spec §1.3). */
export const DEFAULT_MODE: SightMode = 'regions';

/** Resolve a possibly-stored-but-now-unready mode to the default.
 *  E.g., if a user had "confidence" set on an old build but the
 *  Confidence Pack hasn't shipped here, fall back to Regions. */
export function resolveMode(stored: string | undefined | null): SightMode {
    if (!stored) return DEFAULT_MODE;
    const meta = MODE_BY_ID.get(stored as SightMode);
    if (!meta || !meta.ready) return DEFAULT_MODE;
    return meta.id;
}

/** Keyboard shortcut → mode (spec §5.3). Case-insensitive. */
export const KEYS_TO_MODE: ReadonlyMap<string, SightMode> = Object.freeze(
    new Map<string, SightMode>([
        ['r', 'regions'],
        ['l', 'linkTypes'],
        ['t', 'time'],
        ['c', 'confidence'],
        ['s', 'stages'],
        ['a', 'acts'],
    ]),
);

// ─── §2G.3d: per-mode (X, Y, Z) position dispatch ────────────────────
//
// Each mode picks its own X (azimuth), Y (radius), Z (magnitude). Color
// stays invariant across all modes (Louvain community). The dispatch
// keeps SightV3.svelte's render loop oblivious to the cognitive
// variables — it just calls `positionForMode(mode, ctx)` per note and
// gets back screen-ready (azimuth, radius, magnitude, alpha).
//
// Approved by Eisa 2026-05-07. Spec §1, §2 in SIGHT-V3-VISUAL-SPEC.md.

import { magnitudeSize, magnitudeAlpha } from './polar';
import type { RegionLayout, RegionWedge } from './regions';
import { azimuthInWedge } from './regions';

/** Bag of per-note inputs the position dispatch needs. Built once per
 *  fetch / mode switch by SightV3.svelte; passed by reference to the
 *  per-mode position functions (no copying on the hot path). */
export interface ModeContext {
    /** Note path — the layout point's primary key. */
    notePath: string;
    /** Pre-computed centrality rank percentile [0, 1]. 0 = most
     *  central → goes to chart center. 1 = least central → goes to rim. */
    centralityRank: number;
    /** Total link count (in + out). Used as Z for Regions. */
    linkCount: number;
    /** Outgoing link count. Used as Z for Link Types. */
    outgoingCount: number;
    /** Creation timestamp (epoch ms). May be null if missing. */
    createdAt: number | null;
    /** Last-edit timestamp (epoch ms). May be null. */
    modifiedAt: number | null;
    /** Region wedge layout (for Regions mode). */
    regionLayout: RegionLayout | null;
    /** Per-note embed angle from the original MDS layout. Used to
     *  spread stars within their wedge deterministically. */
    embedAngleRad: number;
    /** Dome geometry for radial/azimuthal calculations. */
    domeR: number;
    /** Edge padding (small inset so the most-central star isn't on
     *  the pole exactly). */
    innerInset: number;
    /** Outer cap (small inset from rim so least-central stars don't
     *  kiss the rim divider). */
    outerCap: number;
    /** Universe-wide stats — needed for relative metrics like time
     *  recency, link diversity, etc. Built once per fetch. */
    stats: ModeStats;
}

/** Universe-wide pre-computed stats so each per-note call is O(1). */
export interface ModeStats {
    /** Min / max created_at across all notes (epoch ms). For T mode. */
    minCreatedAt: number;
    maxCreatedAt: number;
    /** Min / max modified_at. For T mode recency. */
    minModifiedAt: number;
    maxModifiedAt: number;
    /** Year wedges for T mode — computed once, cached. */
    timeYearWedges: ReadonlyArray<TimeYearWedge>;
    /** Path → year wedge index lookup (T mode). */
    pathToTimeWedge: ReadonlyMap<string, TimeYearWedge>;
}

/** A single year wedge for T mode. */
export interface TimeYearWedge {
    year: number;
    arcStartRad: number;
    arcEndRad: number;
    arcMidRad: number;
    noteCount: number;
}

/** Output of position dispatch — what SightV3.svelte renders. */
export interface ModePosition {
    /** Angle in radians, theta=0 at top, CW+ (matches polar.ts). */
    azimuth: number;
    /** Distance from chart center in pixels. */
    radius: number;
    /** Star size in pixels. */
    magnitude: number;
    /** Alpha [0, 1]. */
    alpha: number;
}

/** Dispatch — pick the (X, Y, Z) algorithm for the active mode. */
export function positionForMode(mode: SightMode, ctx: ModeContext): ModePosition {
    switch (mode) {
        case 'regions': return positionForRegions(ctx);
        case 'linkTypes': return positionForLinkTypes(ctx);
        case 'time': return positionForTime(ctx);
        // C / S / A "available later" — fall back to Regions until their
        // data layers ship (Concept Paper §6.3 P2/P3/P4). When wired,
        // each gets its own positionForX function below.
        case 'confidence':
        case 'stages':
        case 'acts':
        default:
            return positionForRegions(ctx);
    }
}

// ─── R · Regions ─────────────────────────────────────────────────────
// X = library wedge azimuth · Y = centrality rank · Z = total degree
function positionForRegions(ctx: ModeContext): ModePosition {
    const wedge = ctx.regionLayout?.pathToWedge.get(ctx.notePath) ?? null;
    const azimuth = wedge ? azimuthInWedge(wedge, ctx.embedAngleRad) : 0;
    const radius = ctx.innerInset + ctx.centralityRank * (ctx.outerCap - ctx.innerInset);
    const z = magnitudeFromCount(ctx.linkCount);
    return { azimuth, radius, magnitude: z.size, alpha: z.alpha };
}

// ─── L · Link Types ──────────────────────────────────────────────────
// X = dominant outgoing link type · Y = link-type diversity · Z = total
//    outgoing links. The `dominantLinkType` and `typeDiversity` fields
//    live on a per-note stats map populated when MIG-008 link types
//    are surfaced into ModeContext. Until then, falls back to Regions.
function positionForLinkTypes(ctx: ModeContext): ModePosition {
    // TODO §2G.4+ — wire dominant link type + type diversity. Until
    // those land, route through Regions for visual continuity (the
    // toggle UI will dim L until the data is there).
    const r = positionForRegions(ctx);
    const z = magnitudeFromCount(ctx.outgoingCount);
    return { ...r, magnitude: z.size, alpha: z.alpha };
}

// ─── T · Time ────────────────────────────────────────────────────────
// X = creation date wedge · Y = recency (last edit) · Z = age
function positionForTime(ctx: ModeContext): ModePosition {
    const wedge = ctx.stats.pathToTimeWedge.get(ctx.notePath);
    let azimuth = 0;
    if (wedge) {
        // Spread within the year wedge by createdAt offset within the year.
        const span = wedge.arcEndRad - wedge.arcStartRad;
        const wrapped = ((ctx.embedAngleRad % (2 * Math.PI)) + 2 * Math.PI) % (2 * Math.PI);
        const t = wrapped / (2 * Math.PI);
        azimuth = wedge.arcStartRad + (0.04 + 0.92 * t) * span;
    }
    // Y = recency: center = recently modified, rim = dormant.
    let recencyNorm = 1.0;  // default: rim
    if (ctx.modifiedAt != null && ctx.stats.maxModifiedAt > ctx.stats.minModifiedAt) {
        const span = ctx.stats.maxModifiedAt - ctx.stats.minModifiedAt;
        // 1 = most recent → near center; 0 = oldest → near rim.
        const recent = (ctx.modifiedAt - ctx.stats.minModifiedAt) / span;
        recencyNorm = 1 - recent;  // older → larger radius
    }
    const radius = ctx.innerInset + recencyNorm * (ctx.outerCap - ctx.innerInset);
    // Z = age: older notes brighter (like ancient stars). Norm via createdAt.
    let ageNorm = 0.5;
    if (ctx.createdAt != null && ctx.stats.maxCreatedAt > ctx.stats.minCreatedAt) {
        const span = ctx.stats.maxCreatedAt - ctx.stats.minCreatedAt;
        ageNorm = 1 - (ctx.createdAt - ctx.stats.minCreatedAt) / span;  // 1 = oldest
    }
    const z = {
        size: magnitudeSize(ageNorm),
        alpha: magnitudeAlpha(ageNorm),
    };
    return { azimuth, radius, magnitude: z.size, alpha: z.alpha };
}

// ─── Helpers ─────────────────────────────────────────────────────────

/** Log-bucket a non-negative count to a 6-magnitude size + alpha.
 *  log(1 + count) typically falls in [0, 7]; we normalize to [0, 1]
 *  with a soft cap at log(64) ≈ 4.16 so the brightest stars are
 *  notes with 60+ links (rare). */
function magnitudeFromCount(count: number): { size: number; alpha: number } {
    const log = Math.log(1 + Math.max(0, count));
    const norm = Math.min(1, log / 4.16);
    return {
        size: magnitudeSize(norm),
        alpha: magnitudeAlpha(norm),
    };
}

/** Build the universe-wide stats bag once per fetch. Caller passes
 *  the SkyNode array (which has createdAt) and the layout points.
 *  Returns empty/sentinel values when data is missing — the per-mode
 *  functions are tolerant of those (they fall back to default radii). */
export function buildModeStats(
    nodesByPath: ReadonlyMap<string, { createdAt?: number; modifiedAt?: number }>,
): ModeStats {
    let minCreatedAt = Number.POSITIVE_INFINITY;
    let maxCreatedAt = Number.NEGATIVE_INFINITY;
    let minModifiedAt = Number.POSITIVE_INFINITY;
    let maxModifiedAt = Number.NEGATIVE_INFINITY;
    nodesByPath.forEach((n) => {
        if (n.createdAt != null) {
            if (n.createdAt < minCreatedAt) minCreatedAt = n.createdAt;
            if (n.createdAt > maxCreatedAt) maxCreatedAt = n.createdAt;
        }
        if (n.modifiedAt != null) {
            if (n.modifiedAt < minModifiedAt) minModifiedAt = n.modifiedAt;
            if (n.modifiedAt > maxModifiedAt) maxModifiedAt = n.modifiedAt;
        }
    });
    if (!isFinite(minCreatedAt)) { minCreatedAt = 0; maxCreatedAt = 1; }
    if (!isFinite(minModifiedAt)) { minModifiedAt = 0; maxModifiedAt = 1; }

    // Build year wedges sized by note count per year (compress empty).
    const yearCounts = new Map<number, number>();
    nodesByPath.forEach((n) => {
        if (n.createdAt == null) return;
        const y = new Date(n.createdAt).getUTCFullYear();
        yearCounts.set(y, (yearCounts.get(y) ?? 0) + 1);
    });
    const years = [...yearCounts.entries()].sort((a, b) => a[0] - b[0]);
    const totalCount = years.reduce((s, [, c]) => s + c, 0) || 1;
    const TAU = Math.PI * 2;
    const timeYearWedges: TimeYearWedge[] = [];
    let acc = 0;
    for (const [year, count] of years) {
        const arcSize = TAU * (count / totalCount);
        timeYearWedges.push({
            year,
            arcStartRad: acc,
            arcEndRad: acc + arcSize,
            arcMidRad: acc + arcSize / 2,
            noteCount: count,
        });
        acc += arcSize;
    }
    // Path→wedge lookup
    const pathToTimeWedge = new Map<string, TimeYearWedge>();
    nodesByPath.forEach((n, path) => {
        if (n.createdAt == null) return;
        const y = new Date(n.createdAt).getUTCFullYear();
        const w = timeYearWedges.find((tw) => tw.year === y);
        if (w) pathToTimeWedge.set(path, w);
    });

    return {
        minCreatedAt,
        maxCreatedAt,
        minModifiedAt,
        maxModifiedAt,
        timeYearWedges,
        pathToTimeWedge,
    };
}

/** Empty stats — used before data loads. */
export function emptyModeStats(): ModeStats {
    return {
        minCreatedAt: 0,
        maxCreatedAt: 1,
        minModifiedAt: 0,
        maxModifiedAt: 1,
        timeYearWedges: [],
        pathToTimeWedge: new Map(),
    };
}
