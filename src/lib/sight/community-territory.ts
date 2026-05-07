/**
 * Sight v3 — community territory polygon computation.
 *
 * Given a set of (x, y) points labeled by community id, compute a polygon
 * boundary per community. §1E uses convex hull (simple, robust, sub-ms);
 * a future MIG can swap to alpha-shape for tighter territories where
 * communities have non-convex shapes.
 *
 * Convex hull algorithm: Andrew's monotone chain. O(n log n). Deterministic
 * given a stable sort order. Handles degenerate cases (1 point, 2 points,
 * collinear points) by returning the bounding shape directly.
 *
 * See: docs/Constellation-Sight-v3-Concept-Paper-v1.1.md §2 (territory row).
 */

export interface Point2D {
    x: number;
    y: number;
}

/**
 * Computes the convex hull of a point set. Returns the hull vertices in
 * counter-clockwise order, ready to draw as a closed polygon.
 *
 * For n < 3 points: returns the points themselves (Pixi will draw a dot
 * or a line segment; the territory is conceptually still drawn).
 */
export function convexHull(points: Point2D[]): Point2D[] {
    const n = points.length;
    if (n <= 2) return points.slice();

    // Sort lexicographically by (x, y).
    const sorted = points.slice().sort((a, b) => (a.x === b.x ? a.y - b.y : a.x - b.x));

    // Cross-product of vectors OA and OB (positive = counter-clockwise).
    const cross = (o: Point2D, a: Point2D, b: Point2D): number =>
        (a.x - o.x) * (b.y - o.y) - (a.y - o.y) * (b.x - o.x);

    // Lower hull
    const lower: Point2D[] = [];
    for (const p of sorted) {
        while (lower.length >= 2 && cross(lower[lower.length - 2], lower[lower.length - 1], p) <= 0) {
            lower.pop();
        }
        lower.push(p);
    }

    // Upper hull
    const upper: Point2D[] = [];
    for (let i = sorted.length - 1; i >= 0; i--) {
        const p = sorted[i];
        while (upper.length >= 2 && cross(upper[upper.length - 2], upper[upper.length - 1], p) <= 0) {
            upper.pop();
        }
        upper.push(p);
    }

    // Concatenate lower + upper, dropping the last point of each (it's
    // the first point of the other half).
    lower.pop();
    upper.pop();
    return lower.concat(upper);
}

/**
 * Bucket points by community id and compute a hull polygon for each.
 * Communities with fewer than 3 points still get an entry — the
 * caller decides whether to render as a polygon, line, or dot.
 */
export function communityTerritories(
    points: Array<Point2D & { communityId: number }>,
): Map<number, Point2D[]> {
    const buckets = new Map<number, Point2D[]>();
    for (const p of points) {
        const list = buckets.get(p.communityId);
        if (list) {
            list.push({ x: p.x, y: p.y });
        } else {
            buckets.set(p.communityId, [{ x: p.x, y: p.y }]);
        }
    }
    const territories = new Map<number, Point2D[]>();
    for (const [id, pts] of buckets) {
        territories.set(id, convexHull(pts));
    }
    return territories;
}
