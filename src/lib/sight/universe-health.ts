/**
 * Sight v3 — universe-health metrics + threshold badges (MIG-019 §2D).
 *
 * Thin wrapper over `clusterEngine.ts::computeUniverseHealth` that adds
 * threshold-based status classification (healthy / caution / imbalanced)
 * per Concept Paper v1.1 §3.4.
 *
 * Metrics:
 *   - Modularity (M):    healthy [0.3, 0.6]
 *   - Dominance (D):     healthy < 0.35  (fraction in largest community)
 *   - Entropy (E):       healthy > 2.0 bits
 *   - Connectivity (C):  healthy >= 1.0 (edges/nodes ratio — proxy for
 *                                       average path length per the
 *                                       existing clusterEngine impl)
 *
 * The metrics themselves come from clusterEngine.ts; this module
 * doesn't reimplement them. Pure-TS, no Pixi dep.
 */
import {
    computeUniverseHealth as computeUniverseHealthCore,
    computeStructuralGaps,
    type ClusterInfo,
    type UniverseHealth,
} from '$lib/graph/clusterEngine';

export type HealthStatus = 'healthy' | 'caution' | 'imbalanced';

export interface MetricBadge {
    /** Numeric value */
    value: number;
    /** Status badge */
    status: HealthStatus;
    /** Display string with appropriate precision */
    display: string;
}

export interface HealthReport {
    /** Composite score, 0-100. */
    score: number;
    /** Per-metric breakdown. */
    modularity: MetricBadge;
    dominance: MetricBadge;
    entropy: MetricBadge;
    connectivity: MetricBadge;
    /** Total notes considered. */
    totalNotes: number;
    /** Total edges considered. */
    totalEdges: number;
    /** Communities found. */
    communityCount: number;
}

/**
 * Per Concept Paper v1.1 §3.4:
 *   Modularity: 0.3-0.6 healthy. < 0.3 = poor structure (caution).
 *                                > 0.6 = over-fragmented (caution).
 */
function modularityStatus(m: number): HealthStatus {
    if (m >= 0.3 && m <= 0.6) return 'healthy';
    if (m >= 0.2 && m < 0.3) return 'caution';
    if (m > 0.6 && m <= 0.7) return 'caution';
    return 'imbalanced';
}

/** Lower = better. < 0.35 healthy; 0.35-0.5 caution; > 0.5 imbalanced. */
function dominanceStatus(d: number): HealthStatus {
    if (d < 0.35) return 'healthy';
    if (d < 0.5) return 'caution';
    return 'imbalanced';
}

/** Higher = better. > 2.0 healthy; 1.0-2.0 caution; < 1.0 imbalanced. */
function entropyStatus(e: number): HealthStatus {
    if (e > 2.0) return 'healthy';
    if (e >= 1.0) return 'caution';
    return 'imbalanced';
}

/**
 * Connectivity is `totalEdges / totalNodes` per clusterEngine. The
 * Concept Paper §3.4 frames "average path length < 4 hops = healthy"
 * — a different metric. As a proxy, edges/nodes ≥ 1 means there's at
 * least one edge per node on average, which usually keeps avg path
 * length below 4 on small-world graphs.
 */
function connectivityStatus(c: number): HealthStatus {
    if (c >= 1.0) return 'healthy';
    if (c >= 0.5) return 'caution';
    return 'imbalanced';
}

/**
 * Compute the full health report. Fed by SightV3.svelte's
 * buildIndices() output.
 */
export function computeHealthReport(
    modularity: number,
    clusters: ClusterInfo[],
    edges: Array<{ source: string; target: string }>,
    assignments: Map<string, number>,
    totalNotes: number,
): HealthReport {
    const totalEdges = edges.length;
    const gaps = computeStructuralGaps(clusters, edges, assignments);
    const core: UniverseHealth = computeUniverseHealthCore(
        modularity,
        clusters,
        totalNotes,
        totalEdges,
        gaps.length,
    );

    return {
        score: core.score,
        modularity: {
            value: core.modularity,
            status: modularityStatus(core.modularity),
            display: core.modularity.toFixed(2),
        },
        dominance: {
            value: core.dominance,
            status: dominanceStatus(core.dominance),
            display: `${(core.dominance * 100).toFixed(0)}%`,
        },
        entropy: {
            value: core.entropy,
            status: entropyStatus(core.entropy),
            display: core.entropy.toFixed(2),
        },
        connectivity: {
            value: core.connectivity,
            status: connectivityStatus(core.connectivity),
            display: core.connectivity.toFixed(2),
        },
        totalNotes,
        totalEdges,
        communityCount: clusters.length,
    };
}

/** Empty / placeholder report for edge cases (no notes, no communities). */
export function emptyHealthReport(): HealthReport {
    const empty: MetricBadge = { value: 0, status: 'imbalanced', display: '–' };
    return {
        score: 0,
        modularity: empty,
        dominance: empty,
        entropy: empty,
        connectivity: empty,
        totalNotes: 0,
        totalEdges: 0,
        communityCount: 0,
    };
}
