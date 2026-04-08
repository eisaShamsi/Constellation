#!/usr/bin/env node
/**
 * Release script for Constellation.
 * Usage: node scripts/release.mjs <patch|minor|major|x.y.z>
 *
 * Bumps version in package.json + tauri.conf.json + Cargo.toml,
 * commits, tags, and pushes — triggering the GitHub Actions release.
 */
import { readFileSync, writeFileSync } from 'fs';
import { execSync } from 'child_process';
import { resolve, dirname } from 'path';
import { fileURLToPath } from 'url';

const __dirname = dirname(fileURLToPath(import.meta.url));
const root = resolve(__dirname, '..');

function readJSON(path) {
	return JSON.parse(readFileSync(path, 'utf-8'));
}

function bump(current, type) {
	const [major, minor, patch] = current.split('.').map(Number);
	if (type === 'patch') return `${major}.${minor}.${patch + 1}`;
	if (type === 'minor') return `${major}.${minor + 1}.0`;
	if (type === 'major') return `${major + 1}.0.0`;
	if (/^\d+\.\d+\.\d+$/.test(type)) return type;
	console.error(`Invalid version type: ${type}. Use patch, minor, major, or x.y.z`);
	process.exit(1);
}

const type = process.argv[2];
if (!type) {
	console.error('Usage: node scripts/release.mjs <patch|minor|major|x.y.z>');
	process.exit(1);
}

// 1. Read current version from package.json
const pkgPath = resolve(root, 'package.json');
const pkg = readJSON(pkgPath);
const oldVersion = pkg.version;
const newVersion = bump(oldVersion, type);

console.log(`Bumping ${oldVersion} → ${newVersion}`);

// 2. Update package.json
pkg.version = newVersion;
writeFileSync(pkgPath, JSON.stringify(pkg, null, '\t') + '\n');

// 3. Update tauri.conf.json
const tauriConfPath = resolve(root, 'src-tauri/tauri.conf.json');
const tauriConf = readJSON(tauriConfPath);
tauriConf.version = newVersion;
writeFileSync(tauriConfPath, JSON.stringify(tauriConf, null, '  ') + '\n');

// 4. Update Cargo.toml
const cargoPath = resolve(root, 'src-tauri/Cargo.toml');
let cargo = readFileSync(cargoPath, 'utf-8');
cargo = cargo.replace(/^version = ".*?"$/m, `version = "${newVersion}"`);
writeFileSync(cargoPath, cargo);

// 5. Git commit + tag + push
execSync('git add package.json src-tauri/tauri.conf.json src-tauri/Cargo.toml', { cwd: root, stdio: 'inherit' });
execSync(`git commit -m "release: v${newVersion}"`, { cwd: root, stdio: 'inherit' });
execSync(`git tag v${newVersion}`, { cwd: root, stdio: 'inherit' });
execSync(`git push origin main v${newVersion}`, { cwd: root, stdio: 'inherit' });

console.log(`\n✅ Released v${newVersion} — GitHub Actions will build and publish the update.`);
