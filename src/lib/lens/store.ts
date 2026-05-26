/**
 * MIG-055 §D — Lens TS bridge.
 *
 * Wraps the `execute_lens` Tauri command (`src-tauri/src/lens/query.rs`).
 * Frontend code that needs to evaluate a lens (`LensBlock.svelte`, future
 * sidebar previews, future row-gestures into 360.3D / CNS) calls
 * `executeLens(lensYaml)` from here — the Rust pipeline:
 *
 *   parse_lens_yaml → validate → resolve_libraries → build_sql →
 *   execute_query → materialize → LensResult
 *
 * Type definitions mirror the Rust types in `src-tauri/src/lens/`:
 *   `LensResult` ↔ `src/lens/query.rs::LensResult`
 *   `LensRow` ↔ `src/lens/query.rs::LensRow`
 *   `DimensionValue` ↔ `src/lens/query.rs::DimensionValue` (serde untagged)
 *
 * Keep these in sync with the Rust side. The Rust tests in
 * `lens::query::tests` exercise the JSON shape that round-trips here.
 */

import { invoke } from '@tauri-apps/api/core';

/** One row of a lens result. Mirrors `LensRow` in Rust. */
export interface LensRow {
	/** Note's filesystem path (always present). */
	note_path: string;
	/** Note's name as stored in note_meta (always present). */
	name: string;
	/** Library this note belongs to (always present). */
	library_name: string;
	/** Library's filesystem path (resolved from the universe's libraries.json). */
	library_path: string;
	/** Dimensions the lens declared in `columns:`, keyed by dimension name. */
	dimensions: Record<string, DimensionValue>;
}

/**
 * A single dimension value. Untagged on the Rust side, so JSON arrives
 * as a natural shape: string / number / boolean / null. Timestamps come
 * as numbers (Unix seconds since epoch).
 */
export type DimensionValue = string | number | boolean | null;

/** Whole result of a lens evaluation. Mirrors `LensResult` in Rust. */
export interface LensResult {
	rows: LensRow[];
	total_count: number;
	query_time_ms: number;
	lens_name: string;
	template: string | null;
}

/**
 * Evaluate a lens (YAML text from a ` ```base ` code block) against the
 * active universe's federated library set. Resolves with the materialized
 * `LensResult`; rejects with the Rust error string (parser / validator /
 * SQL / IPC).
 *
 * The Rust function signature is `execute_lens(app, lens_yaml)`. Tauri
 * converts `lens_yaml` to camelCase on the wire, so we pass `lensYaml`.
 */
export async function executeLens(lensYaml: string): Promise<LensResult> {
	return await invoke<LensResult>('execute_lens', { lensYaml });
}
