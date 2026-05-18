/**
 * MIG-026 Phase κ.2 — user-defined .js plugin loader (Path A:
 * Tauri asset:// + native dynamic import, no eval).
 *
 * Why this exists: κ.1 ships a declarative JSON loader covering 4
 * shapes (sectoral / rings / horizontal-bands / gradient). κ.2
 * extends that with arbitrary-code plugins for users who need:
 *   - custom remap logic (e.g. per-note frontmatter-driven sector
 *     assignment that the declarative loader doesn't yet support)
 *   - shapes outside the v1 declarative set (grid / ladder /
 *     relational / cyclic-flow / binary-flow)
 *
 * Why .js, not .ts: Constellation's CSP forbids `unsafe-eval`
 * (orientation §3.4 + LL-019). Native dynamic import() of an asset://
 * URL satisfies CSP; runtime TypeScript transpilation does not.
 * Users author .ts and compile to .js on their side via `tsc` (the
 * Obsidian-plugin pattern). Plan §12.2 originally said .ts; this is
 * the Path A redefinition Eisa picked 2026-05-18 after the CSP
 * surprise surfaced.
 *
 * Security model: Obsidian-trust per Architect §3.H. First-detection
 * banner asks the user to enable each plugin file by filename. Once
 * enabled, the plugin runs with full webview privileges. The
 * `appSettings.sight.enabledTraditionPlugins: string[]` setting
 * persists the consent list across sessions.
 *
 * Lifecycle:
 *   1. Sight mount calls loadPluginRegistry().
 *   2. IPC returns absolute paths to every .js file in
 *      <Universe>/.constellation/traditions/.
 *   3. For each path: if filename is in enabledTraditionPlugins,
 *      try `await import(convertFileSrc(path))`; otherwise mark as
 *      pending-consent.
 *   4. Loader returns { loaded, pending, failed } so SightV6 can
 *      register loaded plugins + surface pending/failed in the UI.
 *
 * Per Working Agreement #4: per-plugin try/catch + on-screen error
 * surface (no console-only diagnostics since release binaries don't
 * expose devtools — see feedback_devtools_dev_only.md).
 */
import { invoke } from '@tauri-apps/api/core';
import { convertFileSrc } from '@tauri-apps/api/core';
import type { UserTraditionModule } from './userDefinedLoader';
import type {
	LayoutCacheRow,
	TraditionLayout,
	TraditionShape,
} from '../types';

interface UserPluginFileDto {
	filename: string;
	absPath: string;
}

/** Plugin discovered but not yet user-approved. Surfaces in the
 *  SightV6 consent banner. */
export interface PendingPlugin {
	filename: string;
	absPath: string;
}

/** Plugin that failed to load. Either the dynamic import threw, the
 *  module had no valid default export, or validation rejected its
 *  shape. The user-visible UI surfaces the error so the plugin
 *  author can fix it. */
export interface FailedPlugin {
	filename: string;
	absPath: string;
	error: string;
}

/** Result of one plugin-registry load pass. SightV6 calls this on
 *  mount + whenever the consent list changes. */
export interface PluginLoadResult {
	loaded: UserTraditionModule[];
	pending: PendingPlugin[];
	failed: FailedPlugin[];
}

const ALLOWED_SHAPES: ReadonlyArray<TraditionShape> = [
	'sectoral',
	'rings',
	'grid',
	'ladder',
	'relational',
	'cyclic-flow',
	'binary-flow',
	'gradient',
	'horizontal-bands',
];

function isString(v: unknown): v is string {
	return typeof v === 'string';
}
function isObject(v: unknown): v is Record<string, unknown> {
	return typeof v === 'object' && v !== null && !Array.isArray(v);
}
function isFunction(v: unknown): v is (...args: unknown[]) => unknown {
	return typeof v === 'function';
}

const ID_PATTERN = /^user-[a-z0-9][a-z0-9-]{2,40}$/;

/** Duck-type a plugin's default export into a UserTraditionModule.
 *  Rejects with a human-readable error if the shape is wrong.
 *  Successful conversion guarantees the renderer can consume the
 *  module without runtime crashes (the per-shape spec callbacks
 *  return their declared types). */
function validateExport(filename: string, raw: unknown): UserTraditionModule | string {
	if (!isObject(raw)) {
		return 'default export must be an object';
	}
	const m = raw;
	if (!isString(m['id']) || !ID_PATTERN.test(m['id'])) {
		return `id must match ${ID_PATTERN} (got ${JSON.stringify(m['id'])})`;
	}
	if (!isString(m['name']) || m['name'].length === 0) {
		return 'name is required (non-empty string)';
	}
	if (!isString(m['shape']) || !ALLOWED_SHAPES.includes(m['shape'] as TraditionShape)) {
		return `shape must be one of ${ALLOWED_SHAPES.join(', ')}`;
	}
	if (!isFunction(m['remapStarPosition'])) {
		return 'remapStarPosition function is required';
	}
	// Build a UserTraditionModule. Optional spec callbacks pass
	// through; if absent the anchor renderer falls back to the
	// shape's no-spec behavior (typically no chrome strokes).
	const mod: UserTraditionModule = {
		id: m['id'],
		name: m['name'],
		shape: m['shape'] as TraditionShape,
		family: isString(m['family']) ? m['family'] : 'user-defined',
		tooltip: isString(m['tooltip']) ? m['tooltip'] : m['name'],
		scope: isString(m['scope']) ? m['scope'] : '',
		citation: isString(m['citation']) ? m['citation'] : '',
		remapStarPosition: (row: LayoutCacheRow, defaultPos, layout: TraditionLayout) => {
			try {
				const r = (m['remapStarPosition'] as (
					row: LayoutCacheRow,
					defaultPos: { x: number; y: number },
					layout: TraditionLayout,
				) => unknown)(row, defaultPos, layout);
				if (
					isObject(r) &&
					typeof r['x'] === 'number' &&
					typeof r['y'] === 'number' &&
					Number.isFinite(r['x']) &&
					Number.isFinite(r['y'])
				) {
					return { x: r['x'], y: r['y'] };
				}
				return defaultPos;
			} catch (err) {
				console.warn(
					`[plugin loader] ${filename}: remapStarPosition threw — using default position`,
					err,
				);
				return defaultPos;
			}
		},
	};
	if (isFunction(m['sectorDividers'])) {
		mod.sectorDividers = wrapSpecCallback(filename, 'sectorDividers', m['sectorDividers']);
	}
	if (isFunction(m['ringBoundaries'])) {
		mod.ringBoundaries = wrapSpecCallback(filename, 'ringBoundaries', m['ringBoundaries']);
	}
	if (isFunction(m['horizontalBandsSpec'])) {
		mod.horizontalBandsSpec = wrapSpecCallback(
			filename,
			'horizontalBandsSpec',
			m['horizontalBandsSpec'],
		);
	}
	if (isFunction(m['gradientSpec'])) {
		mod.gradientSpec = wrapSpecCallback(filename, 'gradientSpec', m['gradientSpec']);
	}
	return mod;
}

/** Wrap an arbitrary user-supplied spec callback so that a runtime
 *  throw inside the user's code degrades gracefully (renderer paints
 *  without the chrome instead of crashing the whole dome). */
function wrapSpecCallback<T>(
	filename: string,
	specName: string,
	fn: unknown,
): (layout: TraditionLayout) => T {
	return (layout: TraditionLayout) => {
		try {
			return (fn as (l: TraditionLayout) => T)(layout);
		} catch (err) {
			console.warn(`[plugin loader] ${filename}: ${specName} threw — chrome skipped`, err);
			return null as unknown as T;
		}
	};
}

/**
 * Read all plugin file paths via IPC, then for each:
 *   - If filename is in `enabledFilenames`, attempt dynamic import
 *     of `convertFileSrc(absPath)` and validate the default export.
 *   - Otherwise, mark as pending-consent.
 *
 * Returns { loaded, pending, failed } so the caller (SightV6) can
 * register loaded plugins, surface a consent banner for pending, and
 * surface error UI for failures.
 */
export async function loadPluginRegistry(
	enabledFilenames: string[],
): Promise<PluginLoadResult> {
	let files: UserPluginFileDto[];
	try {
		files = await invoke<UserPluginFileDto[]>('sight_v6_read_user_plugins');
	} catch (err) {
		console.warn('[plugin loader] IPC failed; no plugins loaded:', err);
		return { loaded: [], pending: [], failed: [] };
	}
	const enabledSet = new Set(enabledFilenames);
	const loaded: UserTraditionModule[] = [];
	const pending: PendingPlugin[] = [];
	const failed: FailedPlugin[] = [];
	for (const file of files) {
		if (!enabledSet.has(file.filename)) {
			pending.push({ filename: file.filename, absPath: file.absPath });
			continue;
		}
		const assetUrl = convertFileSrc(file.absPath);
		try {
			// Vite + esbuild parse static `import()` calls; this dynamic
			// arg form bypasses that, deferring resolution to native
			// runtime ESM. Tauri's asset protocol serves the file with
			// the right MIME type; the webview parses it as a module.
			const mod = (await import(/* @vite-ignore */ assetUrl)) as { default?: unknown };
			if (!mod || mod.default === undefined) {
				failed.push({
					filename: file.filename,
					absPath: file.absPath,
					error: 'no default export — plugin file must `export default { ... }`',
				});
				continue;
			}
			const valid = validateExport(file.filename, mod.default);
			if (typeof valid === 'string') {
				failed.push({
					filename: file.filename,
					absPath: file.absPath,
					error: valid,
				});
				continue;
			}
			loaded.push(valid);
		} catch (err) {
			const msg = err instanceof Error ? err.message : String(err);
			failed.push({
				filename: file.filename,
				absPath: file.absPath,
				error: `import failed: ${msg}`,
			});
		}
	}
	return { loaded, pending, failed };
}
