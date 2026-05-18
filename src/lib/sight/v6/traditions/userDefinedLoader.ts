/**
 * MIG-026 Phase κ.1 — declarative JSON loader for user-defined
 * traditions.
 *
 * Workflow:
 * 1. Sight mount calls `loadUserTraditions()` which invokes the Rust
 *    IPC `sight_v6_read_user_traditions` to read every `.json` file
 *    under `<Universe>/.constellation/traditions/` (excluding the
 *    `schema/` subfolder).
 * 2. Each file is parsed + validated against the v1 schema (hand-
 *    rolled validator below — keeps dep surface small + lets us
 *    emit Constellation-specific warnings).
 * 3. Valid specs are converted to `UserTraditionModule` (structurally
 *    identical to TraditionModule but with `id: string` instead of
 *    the closed `TraditionId` union).
 * 4. The result is handed to `registerUserTraditions()` in
 *    traditions/index.ts, which merges them into a side-map that
 *    the chip + anchor renderer consult alongside the curated
 *    REGISTRY.
 *
 * Validation results:
 * - Successful files become UserTraditionModule entries.
 * - Failed files emit a console.warn naming the file + the specific
 *   schema violation, then are skipped. Other files still load.
 * - Schema-version mismatch (e.g. `schema_version: 99`) → warning +
 *   skipped (Plan §12.1 Stage 3 verification).
 *
 * Per-note frontmatter integration (e.g. user-defined-tradition-
 * specific YAML field that decides sector/ring placement) is a
 * follow-up Pending Job — for now, the declarative shapes use the
 * same default + hash-bucket fallbacks the curated baselines use
 * until the Rust LayoutCacheRow extension lands.
 *
 * The TS plugin loader (Phase κ.2) lives in a separate file and is
 * out of scope here.
 *
 * Reference:
 *   docs/traditions/schema/tradition.v1.schema.json
 *   docs/traditions/schema/EXAMPLE.json
 *   lab/reports/MIG-026-SIGHT-REGISTER-EXPANSION-PLAN.md §12.1
 */
import { invoke } from '@tauri-apps/api/core';
import type {
	LayoutCacheRow,
	TraditionLayout,
	TraditionModule,
	TraditionShape,
	SectorSpec,
	RingSpec,
	HorizontalBandsSpec,
	GradientSpec,
} from '../types';

// ════════════════════════════════════════════════════════════════════
// Types
// ════════════════════════════════════════════════════════════════════

/** One file as returned by the Rust IPC. */
interface UserTraditionFileDto {
	filename: string;
	content: string;
}

/** Subset of TraditionShape currently supported by the declarative
 *  loader (v1 schema). Other shapes need TS plugin loader (κ.2). */
type DeclarativeShape = 'sectoral' | 'rings' | 'horizontal-bands' | 'gradient';

/** Raw shape of a user-tradition spec after JSON.parse, before
 *  validation. Loose by design — validation tightens to known fields. */
interface RawUserSpec {
	schema_version?: unknown;
	id?: unknown;
	name?: unknown;
	shape?: unknown;
	family?: unknown;
	tooltip?: unknown;
	scope?: unknown;
	sectorDividers?: unknown;
	rings?: unknown;
	horizontalBands?: unknown;
	gradient?: unknown;
	citation?: unknown;
}

/** A successfully-loaded user-defined tradition. Mirrors TraditionModule
 *  except `id` is `string` (with `user-` prefix) instead of the closed
 *  TraditionId union. Carries the human-facing metadata (name, tooltip,
 *  scope, family, citation) inline so the chip + manifest modal can
 *  read it without an extra lookup. */
export interface UserTraditionModule {
	id: string;
	name: string;
	shape: TraditionShape;
	family: string;
	tooltip: string;
	scope: string;
	citation: string;
	remapStarPosition: (
		row: LayoutCacheRow,
		defaultPos: { x: number; y: number },
		layout: TraditionLayout,
	) => { x: number; y: number };
	sectorDividers?: (layout: TraditionLayout) => SectorSpec[];
	ringBoundaries?: (layout: TraditionLayout) => RingSpec[];
	horizontalBandsSpec?: (layout: TraditionLayout) => HorizontalBandsSpec;
	gradientSpec?: (layout: TraditionLayout) => GradientSpec;
}

// ════════════════════════════════════════════════════════════════════
// Validation
// ════════════════════════════════════════════════════════════════════

/** Validation result for one file. Either a valid module or an error
 *  with the file's name + a human-readable message. */
type ValidationResult =
	| { ok: true; module: UserTraditionModule }
	| { ok: false; filename: string; error: string };

const ID_PATTERN = /^user-[a-z0-9][a-z0-9-]{2,40}$/;
const DECLARATIVE_SHAPES: DeclarativeShape[] = [
	'sectoral',
	'rings',
	'horizontal-bands',
	'gradient',
];

function isString(v: unknown): v is string {
	return typeof v === 'string';
}
function isFiniteNumber(v: unknown): v is number {
	return typeof v === 'number' && Number.isFinite(v);
}
function isObject(v: unknown): v is Record<string, unknown> {
	return typeof v === 'object' && v !== null && !Array.isArray(v);
}

/** FNV-1a 32-bit hash → normalized [0, 1). Same hash family used by
 *  the curated tradition modules so user-defined traditions get
 *  deterministic placement consistent with the rest of the system. */
function pathHash01(path: string, seed = 0x811c9dc5): number {
	let h = seed >>> 0;
	for (let i = 0; i < path.length; i++) {
		h ^= path.charCodeAt(i);
		h = Math.imul(h, 0x01000193);
	}
	return ((h >>> 0) & 0xffff) / 0xffff;
}

/** Validate one parsed spec + return a UserTraditionModule or an
 *  error message. Mirrors the JSON Schema's required-field +
 *  shape-specific-spec rules. */
function validateAndConstruct(filename: string, raw: RawUserSpec): ValidationResult {
	if (raw.schema_version !== 1) {
		return {
			ok: false,
			filename,
			error: `schema_version must be 1 (got ${JSON.stringify(raw.schema_version)}). This Constellation build only understands v1 user traditions.`,
		};
	}
	if (!isString(raw.id)) {
		return { ok: false, filename, error: 'id is required and must be a string' };
	}
	if (!ID_PATTERN.test(raw.id)) {
		return {
			ok: false,
			filename,
			error: `id "${raw.id}" must match ${ID_PATTERN} (start with "user-", 3–41 chars after prefix, letters/digits/hyphens only)`,
		};
	}
	const id = raw.id;
	if (!isString(raw.name) || raw.name.length === 0 || raw.name.length > 60) {
		return { ok: false, filename, error: 'name is required, 1–60 chars' };
	}
	const name = raw.name;
	if (!isString(raw.shape) || !DECLARATIVE_SHAPES.includes(raw.shape as DeclarativeShape)) {
		return {
			ok: false,
			filename,
			error: `shape "${raw.shape}" not supported by the declarative loader. v1 supports: ${DECLARATIVE_SHAPES.join(', ')}.`,
		};
	}
	const shape = raw.shape as DeclarativeShape;
	const family = isString(raw.family) ? raw.family : 'user-defined';
	const tooltip = isString(raw.tooltip) ? raw.tooltip : name;
	const scope = isString(raw.scope) ? raw.scope : '';
	const citation = isString(raw.citation) ? raw.citation : '';

	// Shape-specific spec validation + remap construction.
	switch (shape) {
		case 'sectoral':
			return constructSectoral(filename, id, name, family, tooltip, scope, citation, raw);
		case 'rings':
			return constructRings(filename, id, name, family, tooltip, scope, citation, raw);
		case 'horizontal-bands':
			return constructHorizontalBands(filename, id, name, family, tooltip, scope, citation, raw);
		case 'gradient':
			return constructGradient(filename, id, name, family, tooltip, scope, citation, raw);
	}
}

function constructSectoral(
	filename: string,
	id: string,
	name: string,
	family: string,
	tooltip: string,
	scope: string,
	citation: string,
	raw: RawUserSpec,
): ValidationResult {
	if (!Array.isArray(raw.sectorDividers) || raw.sectorDividers.length < 2 || raw.sectorDividers.length > 8) {
		return {
			ok: false,
			filename,
			error: 'sectoral shape requires sectorDividers array (2–8 entries)',
		};
	}
	const sectors: { angleStart: number; angleEnd: number; label: string }[] = [];
	for (let i = 0; i < raw.sectorDividers.length; i++) {
		const item = raw.sectorDividers[i];
		if (!isObject(item)) {
			return { ok: false, filename, error: `sectorDividers[${i}] must be an object` };
		}
		const aS = item['angleStartDeg'];
		const aE = item['angleEndDeg'];
		const lbl = item['label'];
		if (!isFiniteNumber(aS) || !isFiniteNumber(aE) || !isString(lbl) || lbl.length === 0) {
			return {
				ok: false,
				filename,
				error: `sectorDividers[${i}] needs angleStartDeg (number), angleEndDeg (number), label (non-empty string)`,
			};
		}
		sectors.push({
			angleStart: (aS * Math.PI) / 180,
			angleEnd: (aE * Math.PI) / 180,
			label: lbl,
		});
	}
	const sectorCount = sectors.length;
	return {
		ok: true,
		module: {
			id,
			name,
			shape: 'sectoral',
			family,
			tooltip,
			scope,
			citation,
			remapStarPosition: (row, defaultPos, layout) => {
				const bucket = Math.floor(pathHash01(row.notePath) * sectorCount);
				const idx = Math.min(sectorCount - 1, Math.max(0, bucket));
				const sec = sectors[idx];
				const dx = defaultPos.x - layout.centerX;
				const dy = defaultPos.y - layout.centerY;
				const radial = Math.hypot(dx, dy);
				const jitter = pathHash01(row.notePath, 0xcafebabe);
				const wedgeSpan = sec.angleEnd - sec.angleStart;
				const clamped = 0.03 + jitter * 0.94;
				const angle = sec.angleStart + clamped * wedgeSpan;
				return {
					x: layout.centerX + Math.cos(angle) * radial,
					y: layout.centerY + Math.sin(angle) * radial,
				};
			},
			sectorDividers: () =>
				sectors.map((s) => ({
					angleStart: s.angleStart,
					angleEnd: s.angleEnd,
					label: s.label,
				})),
		},
	};
}

function constructRings(
	filename: string,
	id: string,
	name: string,
	family: string,
	tooltip: string,
	scope: string,
	citation: string,
	raw: RawUserSpec,
): ValidationResult {
	if (!Array.isArray(raw.rings) || raw.rings.length < 2 || raw.rings.length > 8) {
		return { ok: false, filename, error: 'rings shape requires rings array (2–8 entries)' };
	}
	const rings: { innerFrac: number; outerFrac: number; label: string }[] = [];
	for (let i = 0; i < raw.rings.length; i++) {
		const item = raw.rings[i];
		if (!isObject(item)) {
			return { ok: false, filename, error: `rings[${i}] must be an object` };
		}
		const inner = item['innerFrac'];
		const outer = item['outerFrac'];
		const lbl = item['label'];
		if (
			!isFiniteNumber(inner) ||
			!isFiniteNumber(outer) ||
			!isString(lbl) ||
			lbl.length === 0 ||
			inner < 0 ||
			outer > 1 ||
			outer <= inner
		) {
			return {
				ok: false,
				filename,
				error: `rings[${i}] needs innerFrac < outerFrac in [0, 1] + label`,
			};
		}
		rings.push({ innerFrac: inner, outerFrac: outer, label: lbl });
	}
	const ringCount = rings.length;
	return {
		ok: true,
		module: {
			id,
			name,
			shape: 'rings',
			family,
			tooltip,
			scope,
			citation,
			remapStarPosition: (row, defaultPos, layout) => {
				const bucket = Math.floor(pathHash01(row.notePath) * ringCount);
				const idx = Math.min(ringCount - 1, Math.max(0, bucket));
				const ring = rings[idx];
				const dx = defaultPos.x - layout.centerX;
				const dy = defaultPos.y - layout.centerY;
				const baseAngle = Math.atan2(dy, dx);
				const jitter = pathHash01(row.notePath, 0xcafebabe);
				const targetRadius =
					layout.radius * (ring.innerFrac + (ring.outerFrac - ring.innerFrac) * jitter);
				return {
					x: layout.centerX + Math.cos(baseAngle) * targetRadius,
					y: layout.centerY + Math.sin(baseAngle) * targetRadius,
				};
			},
			ringBoundaries: () =>
				rings.map((r) => ({
					radiusFrac: r.outerFrac,
					label: r.label,
				})),
		},
	};
}

function constructHorizontalBands(
	filename: string,
	id: string,
	name: string,
	family: string,
	tooltip: string,
	scope: string,
	citation: string,
	raw: RawUserSpec,
): ValidationResult {
	if (
		!Array.isArray(raw.horizontalBands) ||
		raw.horizontalBands.length < 2 ||
		raw.horizontalBands.length > 6
	) {
		return {
			ok: false,
			filename,
			error: 'horizontal-bands shape requires horizontalBands array (2–6 entries)',
		};
	}
	const bands: { label: string }[] = [];
	for (let i = 0; i < raw.horizontalBands.length; i++) {
		const item = raw.horizontalBands[i];
		if (!isObject(item) || !isString(item['label']) || item['label'].length === 0) {
			return { ok: false, filename, error: `horizontalBands[${i}] needs label (non-empty string)` };
		}
		bands.push({ label: item['label'] });
	}
	const bandCount = bands.length;
	return {
		ok: true,
		module: {
			id,
			name,
			shape: 'horizontal-bands',
			family,
			tooltip,
			scope,
			citation,
			remapStarPosition: (row, _defaultPos, layout) => {
				const bucket = Math.floor(pathHash01(row.notePath) * bandCount);
				const zone = Math.min(bandCount - 1, Math.max(0, bucket));
				const bandHalf = layout.radius / bandCount;
				const zoneCenterY = layout.centerY - layout.radius + bandHalf * (zone * 2 + 1);
				const jitterY = pathHash01(row.notePath);
				const y = zoneCenterY + (jitterY - 0.5) * bandHalf * 1.6;
				const dy = y - layout.centerY;
				const halfWidthAtY = Math.sqrt(
					Math.max(0, layout.radius * layout.radius - dy * dy),
				);
				const safeHalfWidth = halfWidthAtY * 0.9;
				const jitterX = pathHash01(row.notePath, 0xcafebabe);
				const x = layout.centerX + (jitterX - 0.5) * 2 * safeHalfWidth;
				return { x, y };
			},
			horizontalBandsSpec: () => ({ bands: bands.map((b) => ({ label: b.label })) }),
		},
	};
}

function constructGradient(
	filename: string,
	id: string,
	name: string,
	family: string,
	tooltip: string,
	scope: string,
	citation: string,
	raw: RawUserSpec,
): ValidationResult {
	if (!isObject(raw.gradient)) {
		return { ok: false, filename, error: 'gradient shape requires gradient object' };
	}
	const g = raw.gradient;
	const centerOpacity = g['centerOpacity'];
	const edgeOpacity = g['edgeOpacity'];
	if (
		!isFiniteNumber(centerOpacity) ||
		!isFiniteNumber(edgeOpacity) ||
		centerOpacity < 0 ||
		centerOpacity > 1 ||
		edgeOpacity < 0 ||
		edgeOpacity > 1
	) {
		return {
			ok: false,
			filename,
			error: 'gradient.centerOpacity + gradient.edgeOpacity required in [0, 1]',
		};
	}
	const centerLabel = isString(g['centerLabel']) ? g['centerLabel'] : '';
	const edgeLabel = isString(g['edgeLabel']) ? g['edgeLabel'] : '';
	return {
		ok: true,
		module: {
			id,
			name,
			shape: 'gradient',
			family,
			tooltip,
			scope,
			citation,
			remapStarPosition: (_row, defaultPos, _layout) => defaultPos,
			gradientSpec: () => ({
				centerOpacity,
				edgeOpacity,
				centerLabel,
				edgeLabel,
			}),
		},
	};
}

// ════════════════════════════════════════════════════════════════════
// Public API
// ════════════════════════════════════════════════════════════════════

/** Load every user-defined tradition file from the active Universe.
 *  Returns successful UserTraditionModule entries; logs warnings for
 *  schema violations + version mismatches. The Sight mount calls this
 *  once at boot. */
export async function loadUserTraditions(): Promise<UserTraditionModule[]> {
	let files: UserTraditionFileDto[];
	try {
		files = await invoke<UserTraditionFileDto[]>('sight_v6_read_user_traditions');
	} catch (err) {
		console.warn('[user-tradition loader] IPC failed; no user traditions loaded:', err);
		return [];
	}
	const out: UserTraditionModule[] = [];
	const seenIds = new Set<string>();
	for (const file of files) {
		let raw: unknown;
		try {
			raw = JSON.parse(file.content);
		} catch (err) {
			console.warn(
				`[user-tradition loader] ${file.filename}: not valid JSON — skipped (${err})`,
			);
			continue;
		}
		if (!isObject(raw)) {
			console.warn(
				`[user-tradition loader] ${file.filename}: root value must be an object — skipped`,
			);
			continue;
		}
		const result = validateAndConstruct(file.filename, raw as RawUserSpec);
		if (!result.ok) {
			console.warn(
				`[user-tradition loader] ${result.filename}: ${result.error} — skipped`,
			);
			continue;
		}
		if (seenIds.has(result.module.id)) {
			console.warn(
				`[user-tradition loader] ${file.filename}: duplicate id ${result.module.id} (already registered from an earlier file) — skipped`,
			);
			continue;
		}
		seenIds.add(result.module.id);
		out.push(result.module);
	}
	return out;
}
