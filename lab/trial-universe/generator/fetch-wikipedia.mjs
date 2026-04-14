/**
 * Wikipedia fetcher. Uses MediaWiki REST API (page/summary) for lightweight
 * intro and the Action API (parse) for full HTML when we need wikilinks.
 *
 * Polite policy:
 *  - Rate-limited to 1 req/s (configurable via RATE_MS).
 *  - Custom User-Agent per Wikipedia requirements.
 *  - Caches to disk so resumed runs don't re-fetch.
 */

import { writeFile, readFile, mkdir, access } from 'node:fs/promises';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';

const USER_AGENT = 'ConstellationTrialUniverse/0.1 (https://github.com/eisaShamsi/Constellation; eisa@uconstellation.world) Node';
const RATE_MS = Number(process.env.RATE_MS ?? 1100);
const CACHE_DIR = join(dirname(fileURLToPath(import.meta.url)), '..', '.cache');

let lastRequestAt = 0;

async function rateGate() {
	const dt = Date.now() - lastRequestAt;
	if (dt < RATE_MS) await new Promise(r => setTimeout(r, RATE_MS - dt));
	lastRequestAt = Date.now();
}

function cachePath(kind, title) {
	const safe = title.replace(/[^\w.-]+/g, '_');
	return join(CACHE_DIR, kind, `${safe}.json`);
}

async function readCache(kind, title) {
	const p = cachePath(kind, title);
	try {
		await access(p);
		return JSON.parse(await readFile(p, 'utf8'));
	} catch { return null; }
}

async function writeCache(kind, title, data) {
	const p = cachePath(kind, title);
	await mkdir(dirname(p), { recursive: true });
	await writeFile(p, JSON.stringify(data, null, 2));
}

/** Fetch a page summary (lightweight: title, extract, thumbnail, coordinates). */
export async function fetchSummary(title) {
	const cached = await readCache('summary', title);
	if (cached) return cached;
	await rateGate();
	const url = `https://en.wikipedia.org/api/rest_v1/page/summary/${encodeURIComponent(title)}?redirect=true`;
	const res = await fetch(url, { headers: { 'User-Agent': USER_AGENT, Accept: 'application/json' } });
	if (!res.ok) {
		if (res.status === 404) return null;
		throw new Error(`Wikipedia summary ${title}: ${res.status}`);
	}
	const data = await res.json();
	await writeCache('summary', title, data);
	return data;
}

/** Fetch full parsed HTML + wiki links + sections for a page. */
export async function fetchParsed(title) {
	const cached = await readCache('parsed', title);
	if (cached) return cached;
	await rateGate();
	const params = new URLSearchParams({
		action: 'parse',
		page: title,
		prop: 'text|sections|links|images|categories',
		redirects: '1',
		format: 'json',
		formatversion: '2',
		disableeditsection: '1',
		disabletoc: '1',
		origin: '*',
	});
	const url = `https://en.wikipedia.org/w/api.php?${params}`;
	const res = await fetch(url, { headers: { 'User-Agent': USER_AGENT } });
	if (!res.ok) throw new Error(`Wikipedia parse ${title}: ${res.status}`);
	const data = await res.json();
	if (data.error) {
		if (data.error.code === 'missingtitle') return null;
		throw new Error(`Wikipedia parse ${title}: ${data.error.info}`);
	}
	await writeCache('parsed', title, data.parse);
	return data.parse;
}

/** Get image info (canonical URL + license) for a File: title from Commons. */
export async function fetchImageInfo(fileTitle) {
	const cached = await readCache('image', fileTitle);
	if (cached) return cached;
	await rateGate();
	const params = new URLSearchParams({
		action: 'query',
		titles: fileTitle.startsWith('File:') ? fileTitle : `File:${fileTitle}`,
		prop: 'imageinfo',
		iiprop: 'url|extmetadata|size',
		iiurlwidth: '480',
		format: 'json',
		formatversion: '2',
		origin: '*',
	});
	const url = `https://commons.wikimedia.org/w/api.php?${params}`;
	const res = await fetch(url, { headers: { 'User-Agent': USER_AGENT } });
	if (!res.ok) throw new Error(`Commons imageinfo ${fileTitle}: ${res.status}`);
	const data = await res.json();
	const page = data?.query?.pages?.[0];
	if (!page || page.missing) return null;
	const info = page.imageinfo?.[0];
	if (!info) return null;
	const meta = info.extmetadata ?? {};
	const result = {
		title: page.title,
		thumbUrl: info.thumburl ?? info.url,
		originalUrl: info.url,
		license: meta.LicenseShortName?.value ?? meta.License?.value ?? 'Unknown',
		licenseUrl: meta.LicenseUrl?.value ?? '',
		artist: (meta.Artist?.value ?? '').replace(/<[^>]+>/g, '').trim(),
		description: (meta.ImageDescription?.value ?? '').replace(/<[^>]+>/g, '').trim(),
	};
	await writeCache('image', fileTitle, result);
	return result;
}

/** Download an image file into attachments/ and return the local relative path. */
export async function downloadImage(imageInfo, targetDir) {
	if (!imageInfo?.thumbUrl) return null;
	const name = imageInfo.title.replace(/^File:/, '').replace(/[^\w.-]+/g, '_');
	const outPath = join(targetDir, name);
	try { await access(outPath); return outPath; } catch {}
	await rateGate();
	const res = await fetch(imageInfo.thumbUrl, { headers: { 'User-Agent': USER_AGENT } });
	if (!res.ok) return null;
	const buf = Buffer.from(await res.arrayBuffer());
	await mkdir(dirname(outPath), { recursive: true });
	await writeFile(outPath, buf);
	return outPath;
}
