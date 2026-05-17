/**
 * MIG-026 Phase ε.3 — Ibn Khaldūn ʿumrān (binary-flow, 2 horizontal
 * bands with cyclic flow).
 *
 * Per Concept Paper §4.1 (Arabic / Islamic beyond uṣūl):
 *   Geometry         2 horizontal bands (dome equator divider):
 *                      ḥaḍarī (top half)    — sedentary / urban
 *                                              civilization
 *                      badawī (bottom half) — nomadic / rural
 *                                              bedouin
 *                    Cyclic flow arrows (up on the left, down on
 *                    the right) convey the bidirectional generational
 *                    cycle: nomadic groups establish urban
 *                    civilization, which decays back into nomadic
 *                    conditions after roughly four generations of
 *                    ʿaṣabiyya (group-solidarity) attenuation.
 *   Cultural framing Islamic philosophical historiography; ʿAbd
 *                    al-Raḥmān Ibn Khaldūn (1332-1406) introduced
 *                    ʿilm al-ʿumrān (the science of civilization) in
 *                    the *Muqaddimah* as a cyclical theory of socio-
 *                    political dynamics. The geometry encodes the
 *                    badawī ↔ ḥaḍarī dyad as the primary explanatory
 *                    axis.
 *   Citation         Ibn Khaldūn, *Muqaddimah* (1377), book I §§ 1-5
 *                    (ʿumrān fundamentals); Mahdi, *Ibn Khaldûn's
 *                    Philosophy of History* (1957) ch. 4-5; Lacoste,
 *                    *Ibn Khaldun: The Birth of History* (1984).
 *
 * Star band assignment (Plan §7.3): from a frontmatter `umran_kind`
 * field (values: 'badawi' | 'hadari'); default 'badawi' (nomadic,
 * the foundational state in Ibn Khaldūn's framework) if absent.
 * §ε.3 ships with HASH-BUCKET fallback (50/50 split) so both bands
 * populate visibly — a single-default assignment would leave one
 * band empty and obscure the binary structure. Per-note opt-in
 * ships as §ε.3-fix-N follow-up.
 *
 * Concept Paper: docs/Constellation-Sight-Concept-Paper-v4.0.md §4.1
 * Plan:          lab/reports/MIG-026-SIGHT-REGISTER-EXPANSION-PLAN.md §7.3
 */
import type {
	LayoutCacheRow,
	TraditionLayout,
	TraditionModule,
	BinaryFlowSpec,
} from '../types';

type UmranKind = 'hadari' | 'badawi'; // cellA top / cellB bottom

function pathHash01(path: string): number {
	let h = 0x811c9dc5;
	for (let i = 0; i < path.length; i++) {
		h ^= path.charCodeAt(i);
		h = Math.imul(h, 0x01000193);
	}
	return ((h >>> 0) & 0xffff) / 0xffff;
}

function pathHash01Alt(path: string): number {
	let h = 0xcafebabe;
	for (let i = 0; i < path.length; i++) {
		h ^= path.charCodeAt(i);
		h = Math.imul(h, 0x01000193);
	}
	return (((h >>> 16) & 0xffff)) / 0xffff;
}

/** Determine a note's ʿumrān state.
 *
 *  Per Plan §7.3: read from frontmatter `umran_kind` field; default
 *  to a hash-bucketed 50/50 split if absent.
 *
 *  §ε.3 ship: LayoutCacheRow doesn't yet carry `umranKind`, so this
 *  hash-buckets the notePath into 'hadari' or 'badawi'. Both bands
 *  populate visibly so the binary-flow structure reads clearly.
 *  Per-note opt-in ships in a follow-up.
 */
function umranKindOf(row: LayoutCacheRow): UmranKind {
	return pathHash01(row.notePath) < 0.5 ? 'hadari' : 'badawi';
}

export const ibnKhaldunUmran: TraditionModule = {
	id: 'ibn-khaldun-umran',
	name: 'Ibn Khaldūn ʿumrān',
	shape: 'binary-flow',

	remapStarPosition: (row: LayoutCacheRow, _defaultPos, layout: TraditionLayout) => {
		const kind = umranKindOf(row);
		// Top band: centerY - r to centerY (ḥaḍarī)
		// Bottom band: centerY to centerY + r (badawī)
		// Each band's vertical extent: r. Inset 10% so stars don't
		// kiss the divider or the rim.
		const bandTop = kind === 'hadari'
			? layout.centerY - layout.radius
			: layout.centerY;
		const bandHeight = layout.radius;
		const jitterY = pathHash01(row.notePath);
		const y = bandTop + bandHeight * (0.10 + jitterY * 0.80);

		// Horizontal: jittered across the dome chord at this y, clipped
		// to the circle bound with 10% inset.
		const dy = y - layout.centerY;
		const halfWidthAtY = Math.sqrt(
			Math.max(0, layout.radius * layout.radius - dy * dy),
		);
		const safeHalfWidth = halfWidthAtY * 0.90;
		const jitterX = pathHash01Alt(row.notePath);
		const x = layout.centerX + (jitterX - 0.5) * 2 * safeHalfWidth;

		return { x, y };
	},

	binaryFlowSpec: (_layout: TraditionLayout): BinaryFlowSpec => {
		return {
			cellA: { label: 'ḥaḍarī · sedentary' },
			cellB: { label: 'badawī · nomadic' },
			flowDirection: 'cyclic',
		};
	},
};
