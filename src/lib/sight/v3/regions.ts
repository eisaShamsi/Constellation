/**
 * Sight v3 — Region (library) wedge sizing for the Regions mode.
 *
 * MIG-019 §2G.2. Per `docs/SIGHT-V3-VISUAL-SPEC.md` §1.1:
 *   • Wedges = libraries with at least one note (empty libraries
 *     compress out)
 *   • Order  = by note count, largest first
 *   • Label  = library name
 *
 * The Rust IPC `constellation_sight_v3_layout` returns one `LayoutPoint`
 * per note carrying `note_path`, but NOT a library identifier. To assign
 * each note to a library we walk the `library_paths` array (which the
 * frontend already passes to the IPC) and prefix-match. This is done
 * ONCE at fetch time, then cached per layout result — never in the
 * render loop.
 */

/** Shape of `LayoutPoint` from the Rust IPC. Mirrors the field set in
 *  `src-tauri/src/sight_layout.rs::LayoutPoint`. */
export interface LayoutPoint {
    note_path: string;
    embed_x: number;
    embed_y: number;
    community_id: number;
    centrality_norm: number;
}

/** A single library wedge on the rim. */
export interface RegionWedge {
    /** Full library path (e.g., `"E:/.../Research"`). Stable id. */
    libraryPath: string;
    /** Display name from the libraries store. */
    libraryName: string;
    /** Number of notes assigned to this library. */
    noteCount: number;
    /** Arc start (radians, theta=0 at top, CW+). */
    arcStartRad: number;
    /** Arc end (radians). */
    arcEndRad: number;
    /** Mid-arc — convenience for label placement. */
    arcMidRad: number;
    /** Fraction of the full circle this wedge occupies. */
    arcFrac: number;
}

/** Result bundle: the wedges in canonical order, plus a per-note
 *  azimuth lookup so the render loop can position stars in O(1). */
export interface RegionLayout {
    wedges: ReadonlyArray<RegionWedge>;
    /** note_path → wedge it belongs to. Used by the renderer to compute
     *  each star's azimuth as `arcMidRad + jitter` (jitter from the
     *  star's existing `embed_*` so positions are deterministic). */
    pathToWedge: ReadonlyMap<string, RegionWedge>;
    /** Total notes assigned. May be < layoutPoints.length if some notes
     *  fail to match any library (shouldn't happen, but guard anyway). */
    assignedCount: number;
    /** Notes whose path didn't prefix-match any library. Render fallback:
     *  spread evenly in a thin "Other" wedge at the rim's tail. */
    orphanPaths: ReadonlyArray<string>;
}

/**
 * Build the region wedges for the Regions mode.
 *
 * @param layoutPoints  All notes from the layout IPC.
 * @param libraryPaths  Tuples of `[libraryPath, libraryName]` — same
 *                      array the frontend passed INTO the layout IPC.
 *                      Order doesn't matter; we sort by note count below.
 *
 * @returns Wedges in canonical order (largest first) + lookup map.
 */
export function buildRegionLayout(
    layoutPoints: ReadonlyArray<LayoutPoint>,
    libraryPaths: ReadonlyArray<[string, string]>,
): RegionLayout {
    if (layoutPoints.length === 0 || libraryPaths.length === 0) {
        return {
            wedges: [],
            pathToWedge: new Map(),
            assignedCount: 0,
            orphanPaths: [],
        };
    }

    // Step 1: count notes per library by prefix match.
    // Sort library paths longest-first so nested libraries resolve to
    // the most-specific match (e.g., "/Research/2024" beats "/Research").
    const libsByLength = libraryPaths.slice().sort(
        (a, b) => b[0].length - a[0].length,
    );

    const counts = new Map<string, number>();
    const namesByPath = new Map<string, string>();
    for (const [p, name] of libsByLength) {
        counts.set(p, 0);
        namesByPath.set(p, name);
    }

    // Per-note assignment: which library path each note belongs to.
    // Build first (so we can compute counts), then we'll re-walk to
    // populate the pathToWedge lookup once wedges are sized.
    const assignments = new Map<string, string>();
    const orphans: string[] = [];

    for (const pt of layoutPoints) {
        let matched: string | null = null;
        for (const [libPath] of libsByLength) {
            if (pathStartsWithLibrary(pt.note_path, libPath)) {
                matched = libPath;
                break;
            }
        }
        if (matched != null) {
            assignments.set(pt.note_path, matched);
            counts.set(matched, (counts.get(matched) ?? 0) + 1);
        } else {
            orphans.push(pt.note_path);
        }
    }

    // Step 2: build wedge list, drop libraries with zero notes
    // (compress empty wedges per spec §1.2 "empty wedges compress out").
    const wedgesUnsorted: { path: string; name: string; count: number }[] = [];
    for (const [p, name] of libsByLength) {
        const c = counts.get(p) ?? 0;
        if (c > 0) wedgesUnsorted.push({ path: p, name, count: c });
    }

    // If we have orphans, append a synthetic "Other" wedge so they're
    // still placed somewhere. Tiny by design (typically 0 in practice).
    if (orphans.length > 0) {
        wedgesUnsorted.push({
            path: '__sight_v3_orphan__',
            name: 'Other',
            count: orphans.length,
        });
    }

    if (wedgesUnsorted.length === 0) {
        return {
            wedges: [],
            pathToWedge: new Map(),
            assignedCount: 0,
            orphanPaths: orphans,
        };
    }

    // Step 3: sort by count desc (spec §1.1: "by note count, largest first").
    wedgesUnsorted.sort((a, b) => b.count - a.count);

    // Step 4: compute angular extents.
    const totalCount = wedgesUnsorted.reduce((s, w) => s + w.count, 0);
    const TAU = Math.PI * 2;
    const wedges: RegionWedge[] = [];
    let acc = 0;
    for (const w of wedgesUnsorted) {
        const frac = w.count / totalCount;
        const arcSize = TAU * frac;
        const wedge: RegionWedge = {
            libraryPath: w.path,
            libraryName: w.name,
            noteCount: w.count,
            arcStartRad: acc,
            arcEndRad: acc + arcSize,
            arcMidRad: acc + arcSize / 2,
            arcFrac: frac,
        };
        wedges.push(wedge);
        acc += arcSize;
    }

    // Step 5: build the path→wedge lookup map.
    const wedgeByLibPath = new Map<string, RegionWedge>();
    for (const w of wedges) wedgeByLibPath.set(w.libraryPath, w);
    const pathToWedge = new Map<string, RegionWedge>();
    assignments.forEach((libPath, notePath) => {
        const w = wedgeByLibPath.get(libPath);
        if (w) pathToWedge.set(notePath, w);
    });
    // Orphans → "Other" wedge if present
    const otherWedge = wedgeByLibPath.get('__sight_v3_orphan__');
    if (otherWedge) {
        for (const op of orphans) pathToWedge.set(op, otherWedge);
    }

    return {
        wedges,
        pathToWedge,
        assignedCount: assignments.size + orphans.length,
        orphanPaths: orphans,
    };
}

/**
 * True iff `notePath` starts with `libraryPath` (and has either a
 * separator after, or equals it). Tolerates both `/` and `\`.
 *
 * Examples:
 *   pathStartsWithLibrary("E:/Lib/Note.md", "E:/Lib")          -> true
 *   pathStartsWithLibrary("E:/Lib2/Note.md", "E:/Lib")          -> false  (different lib)
 *   pathStartsWithLibrary("E:\\Lib\\sub\\Note.md", "E:/Lib")    -> true   (sep tolerated)
 */
export function pathStartsWithLibrary(notePath: string, libraryPath: string): boolean {
    if (libraryPath.length === 0) return false;
    const np = normalizePathSeparators(notePath);
    const lp = normalizePathSeparators(libraryPath);
    if (np.length < lp.length) return false;
    if (!np.startsWith(lp)) return false;
    if (np.length === lp.length) return true;
    const next = np.charCodeAt(lp.length);
    return next === 0x2f /* '/' */ || next === 0x5c /* '\\' */;
}

function normalizePathSeparators(p: string): string {
    return p.replace(/\\/g, '/');
}

/**
 * Compute a star's azimuth within a Region wedge.
 *
 * The wedge spans `[arcStartRad, arcEndRad]`. Inside it, the note's
 * position is determined by a deterministic projection of its existing
 * `embed_*` coordinates so:
 *   • Notes from the same community within a wedge cluster together.
 *   • The mapping is stable across renders (no random jitter per frame).
 *   • A 4 % padding from the wedge edges keeps stars off the rim divider.
 *
 * @param embedAngleRad  The note's angle in the original unit-disk
 *                       embedding (atan2 of embed_x, embed_y).
 *                       Caller computes once per layout result.
 */
export function azimuthInWedge(
    wedge: RegionWedge,
    embedAngleRad: number,
): number {
    // Map embed angle [0, TAU) to wedge fraction [0, 1].
    const TAU = Math.PI * 2;
    const wrapped = ((embedAngleRad % TAU) + TAU) % TAU;
    const t = wrapped / TAU;
    const padding = 0.04;
    const span = wedge.arcEndRad - wedge.arcStartRad;
    return wedge.arcStartRad + (padding + (1 - 2 * padding) * t) * span;
}
