/**
 * Unified icon loader for the Emoji & Icon Library plug-in.
 *
 * Produces a single flat list of Icon entries across four sets:
 *  - Lucide        (~1,500 icons, 24×24 stroked, MIT)
 *  - Phosphor      (~1,500 icons regular weight, 256×256 filled, MIT)
 *  - Heroicons     (  ~300 icons 24×24 outline, MIT)
 *  - Feather       (  ~290 icons 24×24 stroked, MIT)
 *
 * Each entry carries a namespaced id ("lucide:heart", "phosphor:heart",
 * "hi:heart", "feather:heart") so shortcode autocomplete and app-icon
 * overrides can target a specific rendering.
 *
 * All icons are wrapped in a consistent <svg> template at load time so
 * the picker displays them uniformly. Lazy-loaded: build runs only on
 * first picker open / first `:` keystroke.
 */

export type IconSet = 'lucide' | 'phosphor' | 'heroicons' | 'feather';

export interface Icon {
	id: string;       // "lucide:heart" — namespaced
	set: IconSet;
	name: string;     // kebab-case name within the set
	svg: string;      // complete standalone <svg> string
}

let _cache: Icon[] | null = null;
let _loading: Promise<Icon[]> | null = null;

export async function loadAllIcons(): Promise<Icon[]> {
	if (_cache) return _cache;
	if (!_loading) {
		_loading = build().then(all => { _cache = all; return all; });
	}
	return _loading;
}

async function build(): Promise<Icon[]> {
	const results = await Promise.all([
		loadLucide().catch(() => [] as Icon[]),
		loadPhosphor().catch(() => [] as Icon[]),
		loadHeroicons().catch(() => [] as Icon[]),
		loadFeather().catch(() => [] as Icon[]),
	]);
	return results.flat();
}

// ─── Lucide ────────────────────────────────────────────────────────────

async function loadLucide(): Promise<Icon[]> {
	const mod = await import('lucide');
	const entries = Object.entries(mod) as [string, any][];
	const seen = new Set<string>();
	const out: Icon[] = [];
	for (const [rawName, def] of entries) {
		if (!Array.isArray(def) || !/^[A-Z]/.test(rawName) || seen.has(rawName)) continue;
		seen.add(rawName);
		const name = kebab(rawName);
		const body = (def as any[]).map((entry: any) => {
			if (!Array.isArray(entry) || entry.length < 2) return '';
			const [tag, attrs] = entry;
			return `<${tag} ${attrsToString(attrs)}/>`;
		}).join('');
		if (!body) continue;
		const svg = `<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">${body}</svg>`;
		out.push({ id: `lucide:${name}`, set: 'lucide', name, svg });
	}
	return out;
}

// ─── Phosphor ──────────────────────────────────────────────────────────
// Phosphor ships SVG files under /assets/<weight>/<name>.svg; we load
// "regular" as the default weight at build time. Vite's import.meta.glob
// bundles them as raw strings.
//
// Six weights exist (bold, duotone, fill, light, regular, thin). We ship
// "regular" only to keep the icon list manageable; users can target other
// weights via future expansion of this module.

const phosphorModules = import.meta.glob('../../../node_modules/@phosphor-icons/core/assets/regular/*.svg', { query: '?raw', import: 'default', eager: false }) as Record<string, () => Promise<string>>;

async function loadPhosphor(): Promise<Icon[]> {
	const out: Icon[] = [];
	const entries = Object.entries(phosphorModules);
	// Parallel load; Vite inlines each SVG as a string import
	await Promise.all(entries.map(async ([path, loader]) => {
		try {
			const svg = await loader();
			const nameMatch = path.match(/\/([^/]+)\.svg$/);
			if (!nameMatch) return;
			out.push({
				id: `phosphor:${nameMatch[1]}`,
				set: 'phosphor',
				name: nameMatch[1],
				svg,
			});
		} catch { /* skip */ }
	}));
	return out;
}

// ─── Heroicons ─────────────────────────────────────────────────────────

const heroiconsModules = import.meta.glob('../../../node_modules/heroicons/24/outline/*.svg', { query: '?raw', import: 'default', eager: false }) as Record<string, () => Promise<string>>;

async function loadHeroicons(): Promise<Icon[]> {
	const out: Icon[] = [];
	const entries = Object.entries(heroiconsModules);
	await Promise.all(entries.map(async ([path, loader]) => {
		try {
			const svg = await loader();
			const nameMatch = path.match(/\/([^/]+)\.svg$/);
			if (!nameMatch) return;
			out.push({
				id: `hi:${nameMatch[1]}`,
				set: 'heroicons',
				name: nameMatch[1],
				svg,
			});
		} catch { /* skip */ }
	}));
	return out;
}

// ─── Feather ───────────────────────────────────────────────────────────
// feather-icons exports a map { 'activity': { contents, attrs, ... } }

async function loadFeather(): Promise<Icon[]> {
	// @ts-expect-error feather-icons ships no type declarations
	const mod = await import('feather-icons');
	const feather = (mod as any).default ?? mod;
	const icons = feather.icons ?? {};
	const out: Icon[] = [];
	for (const [name, icon] of Object.entries(icons) as [string, any][]) {
		const attrs = icon.attrs ?? {};
		const attrStr = attrsToString({
			xmlns: 'http://www.w3.org/2000/svg',
			width: 24, height: 24,
			viewBox: '0 0 24 24',
			fill: 'none',
			stroke: 'currentColor',
			'stroke-width': 2,
			'stroke-linecap': 'round',
			'stroke-linejoin': 'round',
			...attrs,
		});
		const svg = `<svg ${attrStr}>${icon.contents}</svg>`;
		out.push({ id: `feather:${name}`, set: 'feather', name, svg });
	}
	return out;
}

// ─── helpers ───────────────────────────────────────────────────────────

function kebab(pascal: string): string {
	return pascal.replace(/([a-z0-9])([A-Z])/g, '$1-$2').toLowerCase();
}

function attrsToString(attrs: Record<string, any>): string {
	return Object.entries(attrs).map(([k, v]) => `${k}="${v}"`).join(' ');
}

/**
 * Wrap a raw icon SVG with `cn-icon cn-icon-<set>-<name>` classes and
 * `data-icon="<id>"` attribute — used when inserting into a note so
 * future Style Settings CSS can target Constellation-inserted icons.
 */
export function wrapForInsertion(icon: Icon): string {
	const className = `cn-icon cn-icon-${icon.set}-${icon.name}`;
	return icon.svg.replace('<svg', `<svg class="${className}" data-icon="${icon.id}"`);
}
