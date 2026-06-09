import { defineConfig } from 'vite';
import { svelte, vitePreprocess } from '@sveltejs/vite-plugin-svelte';
import { resolve } from 'node:path';

/**
 * MIG-072 §5 fix — standalone PRODUCTION build for the second-screen window.
 *
 * The main app is SvelteKit (vite.config.js). That build copies static/screen.html VERBATIM into
 * build/, so on its own the released second screen pointed at the dev-only /src/screen-entry.ts and
 * rendered a blank white window (the script 404s; nothing mounts). This second, isolated pass
 * actually COMPILES screen-entry.ts → SecondScreenPage and rewrites screen.html to the hashed
 * bundle, writing both into build/.
 *
 * Safety: `emptyOutDir: false` means this NEVER wipes the SvelteKit output that ran first
 * (package.json enforces order: `vite build && vite build --config vite.screen.config.js`).
 * It only adds build/screen.html + build/assets/screen-entry.<hash>.js — different paths from the
 * SPA's _app/immutable/*, so there is no collision and the main window is untouched.
 *
 * The second-screen graph (SecondScreenPage and its imports) uses no SvelteKit virtuals
 * ($app/$env/$service-worker) — verified — so the only alias it needs is $lib. `configFile: false`
 * keeps this build independent of the SvelteKit-oriented svelte.config.js.
 */
export default defineConfig({
	root: resolve('static'),
	plugins: [svelte({ configFile: false, preprocess: vitePreprocess() })],
	resolve: {
		alias: { $lib: resolve('src/lib') },
	},
	build: {
		outDir: resolve('build'),
		emptyOutDir: false,
		rollupOptions: { input: resolve('static/screen.html') },
	},
});
