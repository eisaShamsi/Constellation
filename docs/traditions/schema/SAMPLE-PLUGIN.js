// MIG-026 Phase κ.2 — sample plugin (Path A: .js, native ESM,
// no eval, dynamic-import via Tauri asset:// URL).
//
// Drop a copy of this file into <Universe>/.constellation/traditions/
// (rename or keep the name — anything ending in .js is detected). On
// next Sight open, a consent banner appears. Click "Enable plugin" →
// the plugin's default export registers as a user-defined tradition
// and appears in the chip dropdown's "User-defined" section.
//
// Why .js and not .ts: Constellation's CSP forbids `unsafe-eval` so
// runtime TypeScript transpilation isn't an option. Develop in .ts on
// your side, compile to .js with `tsc`, drop the .js here. Matches
// the Obsidian-plugin pattern.
//
// Security model: Obsidian-trust. Once you click "Enable plugin",
// this file's code runs with the same privileges as the main app
// (can call Tauri IPCs, manipulate state, etc.). Only enable plugins
// from sources you trust.

export default {
	id: 'user-sample-three-acts',
	name: 'Three Acts (sample plugin)',
	shape: 'sectoral',
	family: 'user-defined',
	tooltip: 'Sample plugin tradition demonstrating κ.2 dynamic loading.',
	scope: 'Plugin variant of the κ.1 EXAMPLE.json. Identical 3-wedge shape; the difference is that this one ships as JavaScript so you can swap in custom remap logic.',
	citation: 'Sample — not a real scholarly tradition. Authored as a template for the κ.2 .js plugin loader (MIG-026 Phase κ.2, 2026-05-18).',

	// remapStarPosition: arbitrary user-supplied function. Receives
	// the note's row data, its default Aristotelian position, and the
	// dome layout. Returns the position the note should occupy under
	// this tradition. Must be deterministic per (row, defaultPos) so
	// hit-test + repaint produce the same coordinates.
	//
	// This sample sorts notes into 3 wedges deterministically by a
	// hash of the notePath, then keeps the original radial distance
	// (preserves the stratum encoding inside each wedge).
	remapStarPosition(row, defaultPos, layout) {
		const hash = fnv1a(row.notePath);
		const bucket = hash % 3;
		// Wedge centers at -π/2 (top), -π/2 + 2π/3, -π/2 + 4π/3
		const wedgeCenter = -Math.PI / 2 + (bucket * 2 * Math.PI) / 3;
		// Spread within the wedge by a secondary hash so notes don't
		// all stack on the wedge centerline.
		const jitter = (fnv1a(row.notePath + 'jitter') & 0xffff) / 0xffff;
		const wedgeAngle = wedgeCenter + (jitter - 0.5) * (2 * Math.PI / 3) * 0.85;
		const dx = defaultPos.x - layout.centerX;
		const dy = defaultPos.y - layout.centerY;
		const radial = Math.hypot(dx, dy);
		return {
			x: layout.centerX + Math.cos(wedgeAngle) * radial,
			y: layout.centerY + Math.sin(wedgeAngle) * radial,
		};
	},

	// sectorDividers: returns the visual divider strokes drawn on
	// the dome to mark wedge boundaries. Angles in radians; canvas
	// math convention (0 = east, increases clockwise).
	sectorDividers(_layout) {
		return [
			{ angleStart: -Math.PI / 2 - Math.PI / 3, angleEnd: -Math.PI / 2 + Math.PI / 3, label: 'observation' },
			{ angleStart: -Math.PI / 2 + Math.PI / 3, angleEnd: -Math.PI / 2 + Math.PI, label: 'connection' },
			{ angleStart: -Math.PI / 2 + Math.PI, angleEnd: -Math.PI / 2 + (5 * Math.PI) / 3, label: 'synthesis' },
		];
	},
};

// Local helper — plugin files are self-contained (no `import` from
// outside the file because Vite's bundler doesn't transitively load
// arbitrary runtime modules). Inline whatever utilities you need.
function fnv1a(str) {
	let h = 0x811c9dc5;
	for (let i = 0; i < str.length; i++) {
		h ^= str.charCodeAt(i);
		h = Math.imul(h, 0x01000193);
	}
	return h >>> 0;
}
