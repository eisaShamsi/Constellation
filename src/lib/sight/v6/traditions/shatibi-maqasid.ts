/**
 * MIG-026 Phase ε.2 — Shāṭibī maqāṣid al-sharīʿa (3 tiers × 5 essentials
 * = 15-cell grid).
 *
 * Per Concept Paper §4.1 (Arabic / Islamic beyond uṣūl):
 *   Geometry         3 concentric ring tiers × 5 angular sectors = 15
 *                    grid cells. Composed by setting BOTH
 *                    ringBoundaries + sectorDividers callbacks; the
 *                    anchor dispatch fires both renderers and the
 *                    visual grid emerges as their union.
 *                    Tiers (radial, inner → outer):
 *                      ḍarūriyyāt   (necessities,  0-33%)
 *                      ḥājiyyāt     (needs,       33-67%)
 *                      taḥsīniyyāt  (improvements, 67-100%)
 *                    Essentials (angular, 5 sectors rotated π/4 CW
 *                    from cardinal to avoid stratum-label collision):
 *                      dīn  (religion) · nafs (life) · ʿaql (intellect)
 *                      · nasl (progeny) · māl (property)
 *   Cultural framing Sunni Islamic legal theory; Abū Isḥāq al-Shāṭibī
 *                    (d. 790/1388) systematized the maqāṣid al-sharīʿa
 *                    framework: the law has telic purposes (maqāṣid),
 *                    organized as 5 universal essentials × 3 tiers of
 *                    necessity. The cellular grid visualizes any note
 *                    at the intersection of "which essential?" and
 *                    "what tier of necessity?"
 *   Citation         al-Shāṭibī, *al-Muwāfaqāt fī Uṣūl al-Sharīʿa*
 *                    (Reconciliation of Sharīʿa Foundations, ~1380),
 *                    vol. 2 §§ 8-15; Auda, *Maqasid al-Shariah as
 *                    Philosophy of Islamic Law* (2008) ch. 1-2.
 *
 * Star cell assignment (Plan §7.2): from frontmatter
 * `maqasid_essential` + `maqasid_tier` fields; default ḍarūriyyāt-dīn
 * (cell 0,0) if absent. §ε.2 ships with HASH-DISTRIBUTED fallback
 * across all 15 cells because LayoutCacheRow doesn't yet carry these
 * fields and a single-cell default would crowd one of 15 cells with
 * all stars (less informative than uniform spread). Per-note opt-in
 * ships as a §ε.2-fix-N follow-up.
 *
 * Concept Paper: docs/Constellation-Sight-Concept-Paper-v4.0.md §4.1
 * Plan:          lab/reports/MIG-026-SIGHT-REGISTER-EXPANSION-PLAN.md §7.2
 */
import type {
	LayoutCacheRow,
	TraditionLayout,
	TraditionModule,
	SectorSpec,
	RingSpec,
} from '../types';

type MaqasidTier = 0 | 1 | 2;       // ḍarūriyyāt / ḥājiyyāt / taḥsīniyyāt
type MaqasidEssential = 0 | 1 | 2 | 3 | 4; // dīn / nafs / ʿaql / nasl / māl

const TIER_BOUNDARIES_FRAC = [0.33, 0.67];

const TIER_LABELS: Record<MaqasidTier, string> = {
	0: 'ḍarūriyyāt',
	1: 'ḥājiyyāt',
	2: 'taḥsīniyyāt',
};

const ESSENTIAL_LABELS: Record<MaqasidEssential, string> = {
	0: 'dīn',
	1: 'nafs',
	2: 'ʿaql',
	3: 'nasl',
	4: 'māl',
};

/** Sector start angles in canvas math convention. 5 sectors of 2π/5
 *  (72°) each, rotated π/4 (45°) CW from cardinal vertical axis. The
 *  rotation choice mirrors Longino (4-sector also uses π/4); for the
 *  5-sector pattern, π/4 places dividers at -45°, 27°, 99°, 171°, 243°
 *  (none at cardinal axes), and sector 0 contains the +x axis (3
 *  o'clock) — ring labels drawn by drawRingBoundaries fall cleanly
 *  inside sector 0 with no divider crossing them. Sector 4 contains
 *  the +y axis (12 o'clock) but no divider crosses it, so the stratum
 *  labels (FOUNDATION/etc.) remain clear. */
const SECTOR_ARC = (2 * Math.PI) / 5;
const SECTOR_ROTATION_OFFSET = Math.PI / 4;
const SECTOR_START_ANGLES: Record<MaqasidEssential, number> = {
	0: -Math.PI / 2 + SECTOR_ROTATION_OFFSET,                        // dīn   (NE)
	1: -Math.PI / 2 + SECTOR_ROTATION_OFFSET + SECTOR_ARC,           // nafs  (SE)
	2: -Math.PI / 2 + SECTOR_ROTATION_OFFSET + 2 * SECTOR_ARC,       // ʿaql  (SW)
	3: -Math.PI / 2 + SECTOR_ROTATION_OFFSET + 3 * SECTOR_ARC,       // nasl  (W-NW)
	4: -Math.PI / 2 + SECTOR_ROTATION_OFFSET + 4 * SECTOR_ARC,       // māl   (N, contains +y axis)
};

const ESSENTIAL_ORDER: MaqasidEssential[] = [0, 1, 2, 3, 4];

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

/** Determine a note's (tier, essential) cell.
 *
 *  Per Plan §7.2: read from frontmatter `maqasid_tier` + `maqasid_
 *  essential` fields; default to a hash-distributed cell if absent.
 *
 *  §ε.2 ship: LayoutCacheRow doesn't yet carry these fields, so this
 *  hash-buckets the notePath into one of 15 cells. Each note
 *  deterministically lands in the same cell on every paint. Per-note
 *  opt-in ships in a follow-up.
 */
function maqasidCellOf(row: LayoutCacheRow): { tier: MaqasidTier; essential: MaqasidEssential } {
	const cell = Math.floor(pathHash01(row.notePath) * 15);
	const safeCell = cell >= 15 ? 14 : (cell < 0 ? 0 : cell);
	const tier = (safeCell % 3) as MaqasidTier;
	const essential = (Math.floor(safeCell / 3)) as MaqasidEssential;
	return { tier, essential };
}

export const shatibiMaqasid: TraditionModule = {
	id: 'shatibi-maqasid',
	name: 'Shāṭibī maqāṣid',
	shape: 'grid',

	remapStarPosition: (row: LayoutCacheRow, _defaultPos, layout: TraditionLayout) => {
		const { tier, essential } = maqasidCellOf(row);
		// Radial: jittered within the tier's annulus
		const innerFrac = tier === 0 ? 0 : TIER_BOUNDARIES_FRAC[tier - 1];
		const outerFrac = tier === 2 ? 1.0 : TIER_BOUNDARIES_FRAC[tier];
		const safeInner = innerFrac + (outerFrac - innerFrac) * 0.10;
		const safeOuter = innerFrac + (outerFrac - innerFrac) * 0.90;
		const jitterR = pathHash01Alt(row.notePath);
		const radial = layout.radius * (safeInner + (safeOuter - safeInner) * jitterR);

		// Angular: jittered within the essential's sector wedge
		const startAngle = SECTOR_START_ANGLES[essential];
		const jitterA = pathHash01(row.notePath + ':a');
		const angle = startAngle + (0.1 + jitterA * 0.8) * SECTOR_ARC;

		return {
			x: layout.centerX + radial * Math.cos(angle),
			y: layout.centerY + radial * Math.sin(angle),
		};
	},

	ringBoundaries: (_layout: TraditionLayout): RingSpec[] => {
		// 2 boundary arcs at 33% and 67% + a 3rd RingSpec at 1.0 to
		// carry the outermost tier label.
		return [
			{ radiusFrac: TIER_BOUNDARIES_FRAC[0], label: TIER_LABELS[0] },
			{ radiusFrac: TIER_BOUNDARIES_FRAC[1], label: TIER_LABELS[1] },
			{ radiusFrac: 1.0, label: TIER_LABELS[2] },
		];
	},

	sectorDividers: (_layout: TraditionLayout): SectorSpec[] => {
		return ESSENTIAL_ORDER.map((ess): SectorSpec => {
			const start = SECTOR_START_ANGLES[ess];
			return {
				angleStart: start,
				angleEnd: start + SECTOR_ARC,
				label: ESSENTIAL_LABELS[ess],
			};
		});
	},
};
