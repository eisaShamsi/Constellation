//! Constellation Sight v3 — projection foundation (MIG-018).
//!
//! Computes a deterministic 2D Landmark-MDS embedding of the user's
//! knowledge graph and persists it to the `sight_v3_layout` SQLite
//! table. The frontend reads the cache at Sight-v3 toggle time and
//! projects to screen coordinates via either Lambert azimuthal
//! equal-area or stereographic projection (user-toggle in Settings).
//!
//! See: `docs/Constellation-Sight-v3-Concept-Paper-v1.1.md` §3.
//! Architect: `lab/reports/MIG-018-V3-PROJECTION-FOUNDATION-ARCHITECT.md`.
//! Plan:      `lab/reports/MIG-018-V3-PROJECTION-FOUNDATION-PLAN.md`.
//!
//! ── §1B status (this commit) ──
//! Real Landmark-MDS compute. Reads the graph from `note_meta` +
//! `note_links` (active links only); builds adjacency; computes
//! Brandes betweenness centrality (sampled when n > 500); picks the
//! top-k betweenness nodes as MDS landmarks; computes the V×k BFS
//! distance matrix; runs classical MDS on the k×k landmark sub-matrix
//! via power iteration with deflation; triangulates non-landmarks via
//! inverse-square-distance-weighted average of landmark positions;
//! normalizes to the unit disk; persists to `sight_v3_layout`.
//!
//! Always recomputes on IPC call (no cache-read-first). The cache is
//! populated for future-MIG cache-read-first (§1D), but §1B keeps the
//! simple "always recompute" semantics so correctness is unconditional
//! while the analytic surface is being built up.
//!
//! `community_id` is persisted as 0 (placeholder); §1E computes Louvain
//! frontend-side to avoid duplicating the existing `clusterEngine.ts`
//! Louvain implementation in Rust.
//!
//! ── MIG-019 §2A additions (this commit) ──
//! TF-IDF content-similarity compute (PJ-035 → Milky Way density wash).
//! Reads note bodies from `note_meta`, tokenizes via Constellation's
//! existing FTS5 tokenizer (`fts5_tokenizer::tokenize_to_vec`) for
//! cross-cutting consistency with search. Computes sparse TF×IDF
//! vectors (top-k terms per note), builds an inverted-index of those
//! top terms, finds candidate similar pairs via inverted lookup,
//! computes exact cosine similarity for candidates, persists pairs
//! above the threshold to `sight_v3_similarity_edges`. Returns sorted
//! `(path_a, path_b, similarity)` triples for the frontend Milky Way
//! renderer.

use rusqlite::Connection;
use serde::Serialize;
use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::OnceLock;
use tauri::Manager;

use crate::search::SearchState;

/// One row in `sight_v3_layout`. Returned by `compute_layout_embedding`
/// to the frontend; one entry per note in the queried library set.
///
/// Coordinate system: `(embed_x, embed_y)` lie on the unit disk
/// (`embed_x² + embed_y² ≤ 1.0`). The frontend's `projection.ts`
/// applies either Lambert or stereographic to map disk → screen.
///
/// `centrality_norm` is normalized to `[0, 1]`: `1.0` = highest
/// betweenness centrality node in the universe; `0.0` = a leaf
/// (degree 1 with no shortest paths through it).
///
/// `community_id` is the Louvain community assignment. §1B persists 0;
/// §1E populates from the frontend's existing Louvain machinery.
#[derive(Debug, Clone, Serialize)]
pub struct LayoutPoint {
    pub note_path: String,
    pub embed_x: f32,
    pub embed_y: f32,
    pub community_id: i32,
    pub centrality_norm: f32,
}

// ─────────────────────────────────────────────────────────────────────
// Public IPC
// ─────────────────────────────────────────────────────────────────────

/// `constellation_sight_v3_layout` — the v3 layout-cache IPC.
///
/// Frontend calls this on Sight-v3 toggle. Implementation behavior in
/// §1B: always recompute the embedding on call, persist, return.
/// Cache-read-first will land in §1D when warm-toggle latency matters.
///
/// `library_paths` — list of `(library_path, library_name)` pairs,
/// matching the shape of `constellation_sight_centrality` so the
/// frontend can pass the same `libPaths` array.
///
/// `k_landmarks` — number of MDS landmarks. Pinned in the IPC signature
/// so future tuning (e.g., larger k for higher-fidelity embedding) is
/// non-breaking. Clamped to `[2, n]` internally.
#[tauri::command]
pub fn constellation_sight_v3_layout(
    app: tauri::AppHandle,
    library_paths: Vec<(String, String)>,
    k_landmarks: usize,
) -> Result<Vec<LayoutPoint>, String> {
    let state = app.state::<SearchState>();
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let conn = db.as_ref().ok_or("Search DB not initialized")?;

    let lib_hash = compute_library_set_hash(&library_paths);
    let current_version = get_or_init_graph_version(conn, &lib_hash)?;

    // ── Read graph ───────────────────────────────────────────────────
    let (paths, edges) = read_graph(conn, &library_paths)?;
    let n = paths.len();
    if n == 0 {
        return Ok(Vec::new());
    }
    if n == 1 {
        // Trivial single-node universe — place at origin.
        let single = vec![LayoutPoint {
            note_path: paths[0].clone(),
            embed_x: 0.0,
            embed_y: 0.0,
            community_id: 0,
            centrality_norm: 1.0,
        }];
        persist_layout(conn, &lib_hash, current_version, &paths, &[(0.0, 0.0)], &[1.0])?;
        return Ok(single);
    }

    // ── Build adjacency ──────────────────────────────────────────────
    let adj = build_adjacency(&edges, n);

    // ── Centrality (top-k will be landmarks) ─────────────────────────
    let centrality = compute_centrality_normalized(&adj, n);

    // ── Pick landmarks ───────────────────────────────────────────────
    let k_actual = k_landmarks.clamp(2, n);
    let landmarks = pick_landmarks(&centrality, k_actual);

    // ── Distance matrix (V × k) via BFS from each landmark ───────────
    let dist_matrix = compute_landmark_distances(&adj, &landmarks, n);

    // ── Classical MDS on the k×k landmark sub-matrix ─────────────────
    let landmark_coords = classical_mds_2d(&dist_matrix, &landmarks);

    // ── Triangulate non-landmarks ────────────────────────────────────
    let coords = triangulate(&landmarks, &landmark_coords, &dist_matrix, n);

    // ── Normalize to unit disk ───────────────────────────────────────
    let normalized = normalize_to_unit_disk(coords);

    // ── Persist ──────────────────────────────────────────────────────
    persist_layout(conn, &lib_hash, current_version, &paths, &normalized, &centrality)?;

    // ── Return ───────────────────────────────────────────────────────
    let result: Vec<LayoutPoint> = (0..n)
        .map(|i| LayoutPoint {
            note_path: paths[i].clone(),
            embed_x: normalized[i].0 as f32,
            embed_y: normalized[i].1 as f32,
            community_id: 0,
            centrality_norm: centrality[i] as f32,
        })
        .collect();
    Ok(result)
}

/// `constellation_sight_v3_invalidate_layout` — bump graph_version for
/// every library set. Frontend calls this after batch graph mutations
/// (mass rename, link backfill, etc.). Mirrors v2's `lensDataStale`.
///
/// §1B: implemented but unused; the frontend wiring lands in §1C/§1D
/// where the cache-read-first behavior makes invalidation observable.
#[tauri::command]
pub fn constellation_sight_v3_invalidate_layout(
    app: tauri::AppHandle,
) -> Result<(), String> {
    let state = app.state::<SearchState>();
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let conn = db.as_ref().ok_or("Search DB not initialized")?;

    conn.execute(
        "UPDATE sight_v3_graph_version SET version = version + 1, bumped_at = strftime('%s','now')",
        [],
    )
    .map_err(|e| format!("bump graph_version: {}", e))?;

    Ok(())
}

// ─────────────────────────────────────────────────────────────────────
// DB helpers
// ─────────────────────────────────────────────────────────────────────

/// Deterministic hash of the library set (sorted paths joined and
/// hashed via DefaultHasher). Used as the cache-key library_set_hash.
fn compute_library_set_hash(library_paths: &[(String, String)]) -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let mut sorted: Vec<&str> = library_paths.iter().map(|(p, _)| p.as_str()).collect();
    sorted.sort();
    let mut hasher = DefaultHasher::new();
    for path in &sorted {
        path.hash(&mut hasher);
    }
    format!("{:x}", hasher.finish())
}

/// Reads existing version or initializes to 1.
fn get_or_init_graph_version(conn: &Connection, lib_hash: &str) -> Result<i64, String> {
    let existing: Option<i64> = conn
        .query_row(
            "SELECT version FROM sight_v3_graph_version WHERE library_set_hash = ?1",
            rusqlite::params![lib_hash],
            |r| r.get(0),
        )
        .ok();
    if let Some(v) = existing {
        return Ok(v);
    }
    conn.execute(
        "INSERT INTO sight_v3_graph_version (library_set_hash, version, bumped_at)
         VALUES (?1, 1, strftime('%s','now'))",
        rusqlite::params![lib_hash],
    )
    .map_err(|e| format!("init graph_version: {}", e))?;
    Ok(1)
}

/// Reads the graph for the given library set:
///   - paths: every `note_meta.path` whose path starts with one of the
///            library paths.
///   - edges: undirected, deduplicated `(src_idx, tgt_idx)` pairs from
///            `note_links` (status='active') resolved via
///            `target_name → name_lower → idx` lookup. Ignores edges
///            where source or target is outside the library set.
fn read_graph(
    conn: &Connection,
    library_paths: &[(String, String)],
) -> Result<(Vec<String>, Vec<(usize, usize)>), String> {
    let lib_set: HashSet<String> = library_paths.iter().map(|(p, _)| p.clone()).collect();

    let mut paths: Vec<String> = Vec::new();
    let mut path_to_idx: HashMap<String, usize> = HashMap::new();
    let mut name_lower_to_idx: HashMap<String, usize> = HashMap::new();

    let mut stmt = conn
        .prepare("SELECT path, name FROM note_meta")
        .map_err(|e| format!("prepare note_meta: {}", e))?;

    let rows = stmt
        .query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?)))
        .map_err(|e| format!("query note_meta: {}", e))?;

    for row in rows {
        let (path, name) = row.map_err(|e| format!("read note_meta row: {}", e))?;
        let in_set = lib_set.iter().any(|lib_path| {
            path.starts_with(lib_path)
                && (path.len() == lib_path.len()
                    || path.as_bytes().get(lib_path.len()) == Some(&b'/')
                    || path.as_bytes().get(lib_path.len()) == Some(&b'\\'))
        });
        if !in_set {
            continue;
        }
        let idx = paths.len();
        path_to_idx.insert(path.clone(), idx);
        // Last-write-wins on name collisions across libraries (matches
        // v2 centrality's behaviour: cross-library name collisions
        // resolve to whichever was scanned last; rare in practice).
        name_lower_to_idx.insert(name.to_lowercase(), idx);
        paths.push(path);
    }

    let mut edge_set: HashSet<(usize, usize)> = HashSet::new();
    let mut link_stmt = conn
        .prepare("SELECT source_path, target_name FROM note_links WHERE status = 'active'")
        .map_err(|e| format!("prepare note_links: {}", e))?;
    let link_rows = link_stmt
        .query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?)))
        .map_err(|e| format!("query note_links: {}", e))?;
    for link_row in link_rows {
        let (source_path, target_name) = link_row.map_err(|e| format!("read note_links: {}", e))?;
        let src = path_to_idx.get(&source_path);
        let tgt = name_lower_to_idx.get(&target_name.to_lowercase());
        if let (Some(&s), Some(&t)) = (src, tgt) {
            if s != t {
                let edge = if s < t { (s, t) } else { (t, s) };
                edge_set.insert(edge);
            }
        }
    }

    Ok((paths, edge_set.into_iter().collect()))
}

/// Reads the cached layout for `(lib_hash, version)` from
/// `sight_v3_layout`. Returns `None` if no rows exist.
///
/// Added in MIG-019 §2A+§2B redesign: the density compute needs the
/// embedding coords (from MDS) to project pairs into grid space, so
/// it reads the layout cache rather than recomputing MDS.
fn read_cached_layout(
    conn: &Connection,
    lib_hash: &str,
    version: i64,
) -> Result<Option<Vec<LayoutPoint>>, String> {
    let mut stmt = conn
        .prepare(
            "SELECT note_path, embed_x, embed_y, community_id, centrality_norm
             FROM sight_v3_layout
             WHERE library_set_hash = ?1 AND graph_version = ?2",
        )
        .map_err(|e| format!("prepare layout read: {}", e))?;
    let rows: Vec<LayoutPoint> = stmt
        .query_map(rusqlite::params![lib_hash, version], |row| {
            Ok(LayoutPoint {
                note_path: row.get(0)?,
                embed_x: row.get::<_, f64>(1)? as f32,
                embed_y: row.get::<_, f64>(2)? as f32,
                community_id: row.get(3)?,
                centrality_norm: row.get::<_, f64>(4)? as f32,
            })
        })
        .map_err(|e| format!("query layout: {}", e))?
        .filter_map(|r| r.ok())
        .collect();
    if rows.is_empty() { Ok(None) } else { Ok(Some(rows)) }
}

/// Persists the computed layout to `sight_v3_layout`. Wipes any stale
/// rows for the same `(lib_hash, version)` first (idempotent re-run).
fn persist_layout(
    conn: &Connection,
    lib_hash: &str,
    version: i64,
    paths: &[String],
    coords: &[(f64, f64)],
    centrality: &[f64],
) -> Result<(), String> {
    conn.execute(
        "DELETE FROM sight_v3_layout WHERE library_set_hash = ?1 AND graph_version = ?2",
        rusqlite::params![lib_hash, version],
    )
    .map_err(|e| format!("clear stale layout: {}", e))?;

    let tx = conn
        .unchecked_transaction()
        .map_err(|e| format!("begin tx: {}", e))?;
    {
        let mut stmt = tx
            .prepare(
                "INSERT INTO sight_v3_layout
                 (note_path, library_set_hash, graph_version,
                  embed_x, embed_y, community_id, centrality_norm)
                 VALUES (?1, ?2, ?3, ?4, ?5, 0, ?6)",
            )
            .map_err(|e| format!("prepare insert: {}", e))?;
        for i in 0..paths.len() {
            stmt.execute(rusqlite::params![
                paths[i],
                lib_hash,
                version,
                coords[i].0,
                coords[i].1,
                centrality[i],
            ])
            .map_err(|e| format!("insert layout row {}: {}", i, e))?;
        }
    }

    tx.execute(
        "INSERT OR REPLACE INTO sight_v3_layout_cursor
         (library_set_hash, graph_version, completed, started_at, completed_at)
         VALUES (?1, ?2, 1, strftime('%s','now'), strftime('%s','now'))",
        rusqlite::params![lib_hash, version],
    )
    .map_err(|e| format!("update cursor: {}", e))?;

    tx.commit().map_err(|e| format!("commit: {}", e))?;
    Ok(())
}

// ─────────────────────────────────────────────────────────────────────
// Graph computation
// ─────────────────────────────────────────────────────────────────────

fn build_adjacency(edges: &[(usize, usize)], n: usize) -> Vec<Vec<usize>> {
    let mut adj = vec![Vec::new(); n];
    for &(a, b) in edges {
        adj[a].push(b);
        adj[b].push(a);
    }
    adj
}

/// Brandes' betweenness centrality, normalized to [0, 1].
/// For n > 500, sources are deterministically sub-sampled (every k-th
/// node) so per-call cost stays bounded. Sampling is a well-established
/// approximation for visualization-quality centrality (the exact value
/// only matters for ranking, which is preserved).
///
/// MIG-019 §2E.3 (Boss-test memory fix 2026-05-07): pre-allocate the
/// per-iteration working buffers ONCE outside the source loop and reuse
/// them across all sources via `.clear()` / scalar resets. Previously
/// each iteration did `vec![Vec::new(); n]` which on Boss's universe
/// (7,334 notes × 217k links × 200 sources) added up to ~36 MB of
/// allocator churn — Windows' default allocator doesn't promptly return
/// freed memory to the OS, so the Rust process bloated despite the
/// per-iteration math being O(small).
///
/// Also reduced the source sample from n/200 to n/100 (≈ 100 sources).
/// 100 sources is well-established as adequate for ranking-quality
/// centrality on visualization-scale graphs; the sample density only
/// affects absolute centrality values, not the rank order that drives
/// landmark picking + star-magnitude scaling.
fn compute_centrality_normalized(adj: &[Vec<usize>], n: usize) -> Vec<f64> {
    let mut cb = vec![0.0_f64; n];
    let sources: Vec<usize> = if n > 500 {
        let step = (n / 100).max(1);
        (0..n).step_by(step).collect()
    } else {
        (0..n).collect()
    };

    // Pre-allocate working buffers ONCE; reuse across all source iterations.
    let mut stack: Vec<usize> = Vec::with_capacity(n);
    let mut pred: Vec<Vec<usize>> = (0..n).map(|_| Vec::new()).collect();
    let mut sigma = vec![0.0_f64; n];
    let mut dist: Vec<i64> = vec![-1; n];
    let mut delta = vec![0.0_f64; n];
    let mut queue: VecDeque<usize> = VecDeque::with_capacity(n);

    for &s in &sources {
        // Reset state. .clear() preserves capacity so subsequent
        // iterations don't reallocate the inner Vecs.
        stack.clear();
        for p in pred.iter_mut() { p.clear(); }
        for v in sigma.iter_mut() { *v = 0.0; }
        for v in dist.iter_mut() { *v = -1; }
        for v in delta.iter_mut() { *v = 0.0; }
        queue.clear();
        sigma[s] = 1.0;
        dist[s] = 0;
        queue.push_back(s);

        while let Some(v) = queue.pop_front() {
            stack.push(v);
            for &w in &adj[v] {
                if dist[w] < 0 {
                    dist[w] = dist[v] + 1;
                    queue.push_back(w);
                }
                if dist[w] == dist[v] + 1 {
                    sigma[w] += sigma[v];
                    pred[w].push(v);
                }
            }
        }
        // Accumulate dependency
        while let Some(w) = stack.pop() {
            for &v in &pred[w] {
                delta[v] += (sigma[v] / sigma[w]) * (1.0 + delta[w]);
            }
            if w != s {
                cb[w] += delta[w];
            }
        }
    }

    let max_score: f64 = cb.iter().cloned().fold(0.0_f64, f64::max);
    if max_score > 0.0 {
        cb.iter_mut().for_each(|x| *x /= max_score);
    }
    cb
}

/// Pick top-k by centrality (descending). Ties broken by index ascending.
fn pick_landmarks(centrality: &[f64], k: usize) -> Vec<usize> {
    let mut idx: Vec<usize> = (0..centrality.len()).collect();
    idx.sort_by(|&a, &b| {
        centrality[b]
            .partial_cmp(&centrality[a])
            .unwrap_or(std::cmp::Ordering::Equal)
            .then(a.cmp(&b))
    });
    idx.into_iter().take(k).collect()
}

/// V×k matrix: `dist_matrix[v][l_idx]` = shortest-path distance from
/// node `v` to landmark `landmarks[l_idx]`. Disconnected pairs get a
/// sentinel value (max finite distance + 1) so MDS doesn't see infs.
///
/// MIG-019 §2E.3: same allocation-recycling pattern as
/// compute_centrality_normalized — pre-allocate `dist` and `queue`
/// once, reuse across landmarks via `.clear()`/reset.
fn compute_landmark_distances(adj: &[Vec<usize>], landmarks: &[usize], n: usize) -> Vec<Vec<f64>> {
    let k = landmarks.len();
    let mut dist_matrix = vec![vec![f64::INFINITY; k]; n];

    let mut dist = vec![f64::INFINITY; n];
    let mut queue: VecDeque<usize> = VecDeque::with_capacity(n);

    for (l_idx, &landmark) in landmarks.iter().enumerate() {
        for v in dist.iter_mut() { *v = f64::INFINITY; }
        queue.clear();
        dist[landmark] = 0.0;
        queue.push_back(landmark);
        while let Some(v) = queue.pop_front() {
            let dv = dist[v];
            for &w in &adj[v] {
                if dist[w] == f64::INFINITY {
                    dist[w] = dv + 1.0;
                    queue.push_back(w);
                }
            }
        }
        for v in 0..n {
            dist_matrix[v][l_idx] = dist[v];
        }
    }

    let max_finite: f64 = dist_matrix
        .iter()
        .flat_map(|row| row.iter())
        .filter(|d| d.is_finite())
        .cloned()
        .fold(0.0_f64, f64::max);
    let inf_replacement = max_finite + 1.0;
    for row in dist_matrix.iter_mut() {
        for d in row.iter_mut() {
            if d.is_infinite() {
                *d = inf_replacement;
            }
        }
    }

    dist_matrix
}

/// Classical MDS in 2D applied to the k×k landmark sub-matrix.
/// Returns 2D coords for each landmark.
fn classical_mds_2d(dist_matrix: &[Vec<f64>], landmarks: &[usize]) -> Vec<(f64, f64)> {
    let k = landmarks.len();
    if k == 0 {
        return Vec::new();
    }
    if k == 1 {
        return vec![(0.0, 0.0)];
    }

    // Build k×k squared-distance matrix among landmarks
    let mut d2 = vec![vec![0.0_f64; k]; k];
    for i in 0..k {
        for j in 0..k {
            let d = dist_matrix[landmarks[i]][j];
            d2[i][j] = d * d;
        }
    }

    // Double-centering: B[i][j] = -0.5 × (D²[i][j] − row_mean_i − col_mean_j + grand_mean)
    let row_means: Vec<f64> = d2
        .iter()
        .map(|row| row.iter().sum::<f64>() / k as f64)
        .collect();
    let grand_mean: f64 = row_means.iter().sum::<f64>() / k as f64;

    let mut b = vec![vec![0.0_f64; k]; k];
    for i in 0..k {
        for j in 0..k {
            b[i][j] = -0.5 * (d2[i][j] - row_means[i] - row_means[j] + grand_mean);
        }
    }

    // Power iteration: top-2 eigenvectors of B (symmetric).
    let (lambda1, vec1) = power_iteration(&b, 200);

    // Deflate: B' = B − λ₁ × v₁ × v₁ᵀ
    let mut b_def = b;
    for i in 0..k {
        for j in 0..k {
            b_def[i][j] -= lambda1 * vec1[i] * vec1[j];
        }
    }

    let (lambda2, vec2) = power_iteration(&b_def, 200);

    let s1 = lambda1.max(0.0).sqrt();
    let s2 = lambda2.max(0.0).sqrt();
    (0..k).map(|i| (vec1[i] * s1, vec2[i] * s2)).collect()
}

/// Power iteration to find the dominant eigenvalue + eigenvector of a
/// symmetric matrix. Convergence: ~20-50 iterations for well-separated
/// eigenvalues; we cap at the caller-provided `iters` to bound cost.
///
/// Deterministic seed: `sin(i + 1)` per row, so identical inputs yield
/// identical output coordinates across runs (Concept Paper §3.2).
fn power_iteration(m: &[Vec<f64>], iters: usize) -> (f64, Vec<f64>) {
    let n = m.len();
    if n == 0 {
        return (0.0, Vec::new());
    }

    let mut v: Vec<f64> = (0..n)
        .map(|i| ((i as f64 + 1.0) * 0.7853981633).sin())
        .collect();
    let mut norm: f64 = v.iter().map(|x| x * x).sum::<f64>().sqrt();
    if norm < 1e-12 {
        return (0.0, vec![0.0; n]);
    }
    for x in v.iter_mut() {
        *x /= norm;
    }

    let mut lambda = 0.0_f64;
    let mut v_new = vec![0.0_f64; n];

    for _ in 0..iters {
        v_new.iter_mut().for_each(|x| *x = 0.0);
        for i in 0..n {
            for j in 0..n {
                v_new[i] += m[i][j] * v[j];
            }
        }
        norm = v_new.iter().map(|x| x * x).sum::<f64>().sqrt();
        if norm < 1e-12 {
            return (0.0, vec![0.0; n]);
        }
        for x in v_new.iter_mut() {
            *x /= norm;
        }

        let diff: f64 = v.iter().zip(v_new.iter()).map(|(a, b)| (a - b).abs()).sum();
        std::mem::swap(&mut v, &mut v_new);
        lambda = norm;
        if diff < 1e-9 {
            break;
        }
    }

    (lambda, v)
}

/// Place each non-landmark via inverse-square-distance-weighted average
/// of all landmark positions. Robust to disconnected graphs (sentinel
/// distance handled in `compute_landmark_distances`); landmarks
/// themselves keep their MDS coords verbatim.
fn triangulate(
    landmarks: &[usize],
    landmark_coords: &[(f64, f64)],
    dist_matrix: &[Vec<f64>],
    n: usize,
) -> Vec<(f64, f64)> {
    let k = landmarks.len();
    let mut coords = vec![(0.0_f64, 0.0_f64); n];

    let landmark_to_pos: HashMap<usize, usize> = landmarks
        .iter()
        .enumerate()
        .map(|(i, &v)| (v, i))
        .collect();

    for v in 0..n {
        if let Some(&pos) = landmark_to_pos.get(&v) {
            coords[v] = landmark_coords[pos];
            continue;
        }

        let mut w_sum = 0.0_f64;
        let mut x_sum = 0.0_f64;
        let mut y_sum = 0.0_f64;
        for l_idx in 0..k {
            let d = dist_matrix[v][l_idx];
            // Inverse-square weight: closer landmarks dominate
            let w = 1.0 / (d + 1.0).powi(2);
            w_sum += w;
            x_sum += w * landmark_coords[l_idx].0;
            y_sum += w * landmark_coords[l_idx].1;
        }
        if w_sum > 0.0 {
            coords[v] = (x_sum / w_sum, y_sum / w_sum);
        }
    }

    coords
}

/// Scale coords so max radius = 0.95 (leaves a margin around the dome).
fn normalize_to_unit_disk(coords: Vec<(f64, f64)>) -> Vec<(f64, f64)> {
    if coords.is_empty() {
        return coords;
    }
    let max_r: f64 = coords
        .iter()
        .map(|(x, y)| (x * x + y * y).sqrt())
        .fold(0.0_f64, f64::max);
    if max_r < 1e-12 {
        return coords;
    }
    let scale = 0.95 / max_r;
    coords
        .into_iter()
        .map(|(x, y)| (x * scale, y * scale))
        .collect()
}

// ─────────────────────────────────────────────────────────────────────
// MIG-019 §2A+§2B redesign — TF-IDF density grid (PJ-035 → Milky Way)
// ─────────────────────────────────────────────────────────────────────
//
// Architecture (post Eisa's 2026-05-07 "Don't patch it. Solve it.")
//
// The Concept Paper v1.1 §5.1 specifies the Milky Way as a "density
// field, not a set of edges." The v2 shipping shape (edge list +
// BlurFilter) accumulated O(candidate_pairs) heap allocating two
// cloned path strings per edge — OOM at ~7,600 notes / ~656k links.
//
// The v3 shape: every candidate pair above the similarity threshold
// rasterizes a line in a fixed-size 2D density grid (256×256 f32),
// accumulating the similarity weight per cell. The grid is then
// Gaussian-blurred to smooth the discrete lines into a continuous
// band texture. The grid (≈ 256 KB) is the IPC payload — universe
// size becomes irrelevant.
//
// Memory profile (worst case): grid 256 KB + sparse vectors ~10 MB +
// inverted index ~10 MB + per-iteration candidates HashSet ~24 KB.
// Bounded by output, not input.

const DENSITY_GRID_SIZE: usize = 256;
const BLUR_PASSES: usize = 3;

/// Returned by `constellation_sight_v3_density_field`. The frontend
/// builds a Pixi Texture from `values` and renders it as a single
/// Sprite filling the dome — one draw call regardless of universe size.
///
/// `values` length == `width * height`. Row-major; `values[y * width + x]`.
/// Each cell holds the accumulated, blurred similarity weight for the
/// pairs whose connecting line passed through that cell.
///
/// `max_value` is the maximum across the grid — frontend uses it to
/// normalize alpha into [0, 1] for rendering.
#[derive(Debug, Clone, Serialize)]
pub struct DensityField {
    pub width: u32,
    pub height: u32,
    pub max_value: f32,
    pub values: Vec<f32>,
}

/// `constellation_sight_v3_density_field` — the v3 Milky Way IPC.
///
/// Frontend calls this after `fetchLayout` (asynchronously, so the
/// chart doesn't block on the density compute). Cache-hit reads the
/// persisted BLOB; cache-miss runs cold compute and persists.
///
/// `library_paths` — `(library_path, library_name)` tuples matching
///   the layout IPC's signature.
/// `k_top_terms` — top-k TF×IDF terms per note for the inverted-index
///   candidate filter. Default 50.
/// `similarity_threshold` — cosine similarity floor. Pairs below this
///   are dropped. Default 0.3 per Concept Paper v1.1 §3.3.
#[tauri::command]
pub fn constellation_sight_v3_density_field(
    app: tauri::AppHandle,
    library_paths: Vec<(String, String)>,
    k_top_terms: usize,
    similarity_threshold: f32,
) -> Result<DensityField, String> {
    let state = app.state::<SearchState>();
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let conn = db.as_ref().ok_or("Search DB not initialized")?;

    let lib_hash = compute_library_set_hash(&library_paths);
    let current_version = get_or_init_graph_version(conn, &lib_hash)?;

    // Cache-read-first: BLOB lookup is sub-millisecond.
    if let Some(cached) = read_cached_density(conn, &lib_hash, current_version)? {
        return Ok(cached);
    }

    // ── Read layout positions from sight_v3_layout cache ─────────────
    // The frontend calls this IPC AFTER fetchLayout (which populates
    // sight_v3_layout via §1B's compute). We read those embedding
    // coords here to project pairs into grid space. If the layout cache
    // is empty (race or upstream error), return an empty grid.
    let layout = match read_cached_layout(conn, &lib_hash, current_version)? {
        Some(rows) if !rows.is_empty() => rows,
        _ => return Ok(empty_density_field()),
    };
    let mut path_to_coords: HashMap<String, (f32, f32)> = HashMap::with_capacity(layout.len());
    for pt in &layout {
        path_to_coords.insert(pt.note_path.clone(), (pt.embed_x, pt.embed_y));
    }

    // ── Stream-read paths + bodies, tokenize inline ──────────────────
    // The body is dropped at end of each iteration; only the per-note
    // token counts (HashMap) survive. Memory peaks at ~10 MB on
    // Boss-scale graphs (7,600 notes × ~100 distinct terms × ~30 bytes).
    const SIMILARITY_BODY_CAP: usize = 64 * 1024;
    let lib_set: HashSet<String> = library_paths.iter().map(|(p, _)| p.clone()).collect();
    let mut note_paths: Vec<String> = Vec::new();
    let mut note_coords: Vec<(f32, f32)> = Vec::new();
    let mut token_sets: Vec<HashMap<String, u32>> = Vec::new();
    let stopwords = stopwords_cached();
    {
        let mut stmt = conn
            .prepare("SELECT path, body FROM note_meta")
            .map_err(|e| format!("prepare note_meta density read: {}", e))?;
        let rows = stmt
            .query_map([], |r| {
                Ok((r.get::<_, String>(0)?, r.get::<_, Option<String>>(1)?.unwrap_or_default()))
            })
            .map_err(|e| format!("query note_meta density read: {}", e))?;
        for row in rows {
            let (path, body) = row.map_err(|e| format!("read row: {}", e))?;
            let in_set = lib_set.iter().any(|lib_path| {
                path.starts_with(lib_path)
                    && (path.len() == lib_path.len()
                        || path.as_bytes().get(lib_path.len()) == Some(&b'/')
                        || path.as_bytes().get(lib_path.len()) == Some(&b'\\'))
            });
            if !in_set {
                continue;
            }
            // Skip notes without a layout entry (orphans / new since last
            // MDS compute) — they can't contribute to the density field
            // until the next layout recompute.
            let coords = match path_to_coords.get(&path) {
                Some(c) => *c,
                None => continue,
            };
            let clipped = clip_utf8(&body, SIMILARITY_BODY_CAP);
            let tokens = crate::fts5_tokenizer::tokenize_to_vec(clipped, stopwords);
            let mut counts: HashMap<String, u32> = HashMap::with_capacity(tokens.len().min(256));
            for t in tokens {
                *counts.entry(t).or_insert(0) += 1;
            }
            note_paths.push(path);
            note_coords.push(coords);
            token_sets.push(counts);
        }
    }

    let n = note_paths.len();
    if n == 0 {
        return Ok(empty_density_field());
    }

    // ── Build IDF + sparse TF×IDF vectors ────────────────────────────
    let idf = compute_idf(&token_sets);
    let mut sparse_vectors: Vec<Vec<(String, f64)>> = Vec::with_capacity(n);
    for counts in &token_sets {
        sparse_vectors.push(build_sparse_tfidf(counts, &idf, k_top_terms));
    }
    drop(token_sets); // free per-note token-count maps now that sparse vectors are built
    drop(idf);

    // ── Build inverted index: term → Vec<note_idx> for candidate lookup
    let mut inverted: HashMap<String, Vec<usize>> = HashMap::new();
    for (i, vec_i) in sparse_vectors.iter().enumerate() {
        for (term, _) in vec_i {
            inverted.entry(term.clone()).or_default().push(i);
        }
    }

    // ── Initialize density grid ──────────────────────────────────────
    let gw = DENSITY_GRID_SIZE;
    let gh = DENSITY_GRID_SIZE;
    let mut grid = vec![0.0_f32; gw * gh];

    // ── Compute similarities, rasterize into grid ────────────────────
    // For each candidate pair above the threshold: rasterize a line in
    // grid space (DDA), accumulating similarity weight per cell. The
    // pair is then dropped — no edge list, no path-string cloning.
    // Memory for the accumulator is fixed at gw*gh*4 bytes regardless
    // of universe size.
    let threshold = similarity_threshold as f64;
    for i in 0..n {
        let vec_i = &sparse_vectors[i];
        if vec_i.is_empty() {
            continue;
        }
        let mut candidates: HashSet<usize> = HashSet::new();
        for (term, _) in vec_i {
            if let Some(notes) = inverted.get(term) {
                for &j in notes {
                    if j > i {
                        candidates.insert(j);
                    }
                }
            }
        }
        if candidates.is_empty() {
            continue;
        }
        let (xi_e, yi_e) = note_coords[i];
        for j in candidates {
            let vec_j = &sparse_vectors[j];
            if vec_j.is_empty() {
                continue;
            }
            let dot = cosine_sparse(vec_i, vec_j);
            if dot >= threshold {
                let (xj_e, yj_e) = note_coords[j];
                let (gxi, gyi) = embed_to_grid(xi_e, yi_e, gw, gh);
                let (gxj, gyj) = embed_to_grid(xj_e, yj_e, gw, gh);
                rasterize_line(&mut grid, gw, gh, gxi, gyi, gxj, gyj, dot as f32);
            }
        }
    }

    // ── Smooth: separable Gaussian blur, BLUR_PASSES iterations ──────
    for _ in 0..BLUR_PASSES {
        gaussian_blur_separable(&mut grid, gw, gh);
    }

    // ── Find max value for frontend normalization ────────────────────
    let max_value = grid.iter().cloned().fold(0.0_f32, f32::max);

    let field = DensityField {
        width: gw as u32,
        height: gh as u32,
        max_value,
        values: grid,
    };

    persist_density(conn, &lib_hash, current_version, &field)?;
    Ok(field)
}

/// Empty density field returned when there's no data to compute over.
fn empty_density_field() -> DensityField {
    DensityField {
        width: DENSITY_GRID_SIZE as u32,
        height: DENSITY_GRID_SIZE as u32,
        max_value: 0.0,
        values: vec![0.0_f32; DENSITY_GRID_SIZE * DENSITY_GRID_SIZE],
    }
}

/// Map an embedding-space point in `[-1, 1] × [-1, 1]` to grid coords
/// in `[0, gw) × [0, gh)`. Clamps to grid bounds.
#[inline]
fn embed_to_grid(x: f32, y: f32, gw: usize, gh: usize) -> (f32, f32) {
    let gx = ((x + 1.0) * 0.5 * gw as f32).clamp(0.0, (gw - 1) as f32);
    let gy = ((y + 1.0) * 0.5 * gh as f32).clamp(0.0, (gh - 1) as f32);
    (gx, gy)
}

/// DDA line rasterization: walks a line from (x0, y0) to (x1, y1) in
/// grid space, accumulating `weight` per visited cell.
fn rasterize_line(grid: &mut [f32], gw: usize, gh: usize, x0: f32, y0: f32, x1: f32, y1: f32, weight: f32) {
    let dx = x1 - x0;
    let dy = y1 - y0;
    let steps = dx.abs().max(dy.abs()).max(1.0) as usize;
    let step_x = dx / steps as f32;
    let step_y = dy / steps as f32;
    for s in 0..=steps {
        let x = x0 + step_x * s as f32;
        let y = y0 + step_y * s as f32;
        let gx = x.floor() as i32;
        let gy = y.floor() as i32;
        if gx >= 0 && (gx as usize) < gw && gy >= 0 && (gy as usize) < gh {
            grid[(gy as usize) * gw + (gx as usize)] += weight;
        }
    }
}

/// Separable 3-tap Gaussian blur (kernel [1, 2, 1] / 4). One pass =
/// horizontal blur followed by vertical blur. Three passes ≈ a 7×7
/// Gaussian — enough to smooth the discrete rasterized lines into the
/// soft band texture the Milky Way needs.
fn gaussian_blur_separable(grid: &mut Vec<f32>, gw: usize, gh: usize) {
    let mut tmp = vec![0.0_f32; gw * gh];
    // Horizontal pass: tmp[y][x] = (g[y][x-1] + 2*g[y][x] + g[y][x+1]) / 4
    for y in 0..gh {
        for x in 0..gw {
            let l = if x == 0 { x } else { x - 1 };
            let r = if x + 1 >= gw { x } else { x + 1 };
            let s = grid[y * gw + l] + 2.0 * grid[y * gw + x] + grid[y * gw + r];
            tmp[y * gw + x] = s * 0.25;
        }
    }
    // Vertical pass: write back into grid
    for y in 0..gh {
        let u = if y == 0 { y } else { y - 1 };
        let d = if y + 1 >= gh { y } else { y + 1 };
        for x in 0..gw {
            let s = tmp[u * gw + x] + 2.0 * tmp[y * gw + x] + tmp[d * gw + x];
            grid[y * gw + x] = s * 0.25;
        }
    }
}

/// Process-wide stopwords cache. Same idiom as `ctse::hooks::stopwords_cached`.
fn stopwords_cached() -> &'static HashSet<String> {
    static SW: OnceLock<HashSet<String>> = OnceLock::new();
    SW.get_or_init(crate::libraries::build_stopwords)
}

/// IDF formula: `log((N + 1) / (df_t + 1)) + 1`. Smoothed so unique terms
/// don't divide by zero and trivial corpora (single doc) give sensible
/// non-zero IDF. Mirrors sklearn's TfidfVectorizer with smooth_idf=True.
fn compute_idf(per_doc_counts: &[HashMap<String, u32>]) -> HashMap<String, f64> {
    let n_f = per_doc_counts.len() as f64;
    let mut df: HashMap<&str, u32> = HashMap::new();
    for counts in per_doc_counts {
        for term in counts.keys() {
            *df.entry(term.as_str()).or_insert(0) += 1;
        }
    }
    df.into_iter()
        .map(|(term, doc_freq)| {
            let v = ((n_f + 1.0) / (doc_freq as f64 + 1.0)).ln() + 1.0;
            (term.to_string(), v)
        })
        .collect()
}

/// Build the TF×IDF sparse vector for one doc, top-k by score, L2-normalized.
/// L2 normalization makes cosine similarity == dot product downstream.
fn build_sparse_tfidf(
    counts: &HashMap<String, u32>,
    idf: &HashMap<String, f64>,
    k_top_terms: usize,
) -> Vec<(String, f64)> {
    let total_terms: u32 = counts.values().sum();
    if total_terms == 0 {
        return Vec::new();
    }
    let total_f = total_terms as f64;
    let mut scored: Vec<(String, f64)> = counts
        .iter()
        .map(|(term, count)| {
            let tf = *count as f64 / total_f;
            let idf_val = idf.get(term).copied().unwrap_or(0.0);
            (term.clone(), tf * idf_val)
        })
        .collect();
    scored.sort_by(|a, b| {
        b.1.partial_cmp(&a.1)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then(a.0.cmp(&b.0))
    });
    scored.truncate(k_top_terms.max(1));
    let norm: f64 = scored.iter().map(|(_, s)| s * s).sum::<f64>().sqrt();
    if norm > 1e-12 {
        for (_, s) in scored.iter_mut() {
            *s /= norm;
        }
    }
    scored
}

/// Cosine similarity of two L2-normalized sparse vectors == dot product.
/// Iterates the smaller vector's terms; lookups are O(1) per term.
fn cosine_sparse(a: &[(String, f64)], b: &[(String, f64)]) -> f64 {
    let (small, large) = if a.len() <= b.len() { (a, b) } else { (b, a) };
    let map_l: HashMap<&str, f64> = large.iter().map(|(t, s)| (t.as_str(), *s)).collect();
    small
        .iter()
        .filter_map(|(t, ss)| map_l.get(t.as_str()).map(|sl| ss * sl))
        .sum()
}

/// Module-level body-cap constant retained for future callers that want
/// the CTSE convention. The MIG-019 similarity IPC uses a tighter local
/// cap (`SIMILARITY_BODY_CAP = 64KB`) inline because TF-IDF doesn't
/// benefit from the lede-and-then-some that 1MB allows.
#[allow(dead_code)]
const BODY_CAP_BYTES: usize = 1024 * 1024;

fn clip_utf8(s: &str, cap: usize) -> &str {
    if s.len() <= cap {
        return s;
    }
    let mut end = cap;
    while end > 0 && !s.is_char_boundary(end) {
        end -= 1;
    }
    &s[..end]
}

/// Read the cached density grid for `(lib_hash, version)` from
/// `sight_v3_density_grid`. Returns `None` on cache miss.
///
/// The BLOB column holds the `Vec<f32>` as little-endian f32 bytes
/// (4 bytes per cell × width × height). Decoding is a single
/// chunks-of-4 scan; sub-millisecond on a 256 KB BLOB.
fn read_cached_density(
    conn: &Connection,
    lib_hash: &str,
    version: i64,
) -> Result<Option<DensityField>, String> {
    let row = conn
        .query_row(
            "SELECT width, height, max_value, data
             FROM sight_v3_density_grid
             WHERE library_set_hash = ?1 AND graph_version = ?2",
            rusqlite::params![lib_hash, version],
            |r| {
                Ok((
                    r.get::<_, i64>(0)? as u32,
                    r.get::<_, i64>(1)? as u32,
                    r.get::<_, f64>(2)? as f32,
                    r.get::<_, Vec<u8>>(3)?,
                ))
            },
        )
        .ok();
    match row {
        Some((width, height, max_value, blob)) => {
            // Decode little-endian f32 bytes into Vec<f32>.
            let n = (width as usize) * (height as usize);
            if blob.len() != n * 4 {
                return Err(format!(
                    "density grid BLOB size mismatch: expected {} bytes, got {}",
                    n * 4,
                    blob.len()
                ));
            }
            let mut values = Vec::with_capacity(n);
            for chunk in blob.chunks_exact(4) {
                values.push(f32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]));
            }
            Ok(Some(DensityField {
                width,
                height,
                max_value,
                values,
            }))
        }
        None => Ok(None),
    }
}

/// Persist the density grid to `sight_v3_density_grid`. Idempotent:
/// `INSERT OR REPLACE` on the (lib_hash, version) primary key.
///
/// Encoding: `values` is serialized as a flat Vec<u8> of little-endian
/// f32 bytes. For a 256×256 grid: 262,144 bytes ≈ 256 KB.
fn persist_density(
    conn: &Connection,
    lib_hash: &str,
    version: i64,
    field: &DensityField,
) -> Result<(), String> {
    let n = (field.width as usize) * (field.height as usize);
    if field.values.len() != n {
        return Err(format!(
            "density grid value/dimension mismatch: width*height={} but values.len()={}",
            n,
            field.values.len()
        ));
    }
    let mut blob: Vec<u8> = Vec::with_capacity(n * 4);
    for v in &field.values {
        blob.extend_from_slice(&v.to_le_bytes());
    }
    conn.execute(
        "INSERT OR REPLACE INTO sight_v3_density_grid
         (library_set_hash, graph_version, width, height, max_value, data, computed_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, strftime('%s','now'))",
        rusqlite::params![
            lib_hash,
            version,
            field.width as i64,
            field.height as i64,
            field.max_value as f64,
            blob,
        ],
    )
    .map_err(|e| format!("persist density grid: {}", e))?;
    Ok(())
}

// ─────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    /// 6-node ring: 0-1-2-3-4-5-0. MDS should produce an embedding
    /// with all nodes roughly equidistant from the centroid.
    #[test]
    fn ring_graph_yields_circular_embedding() {
        let adj = vec![
            vec![1, 5], // 0
            vec![0, 2], // 1
            vec![1, 3], // 2
            vec![2, 4], // 3
            vec![3, 5], // 4
            vec![4, 0], // 5
        ];
        let n = 6;
        let centrality = compute_centrality_normalized(&adj, n);
        let landmarks = pick_landmarks(&centrality, 6);
        let dist_matrix = compute_landmark_distances(&adj, &landmarks, n);
        let landmark_coords = classical_mds_2d(&dist_matrix, &landmarks);
        let coords = triangulate(&landmarks, &landmark_coords, &dist_matrix, n);
        let normalized = normalize_to_unit_disk(coords);

        // All radii should be roughly equal
        let radii: Vec<f64> = normalized
            .iter()
            .map(|(x, y)| (x * x + y * y).sqrt())
            .collect();
        let min_r = radii.iter().cloned().fold(f64::INFINITY, f64::min);
        let max_r = radii.iter().cloned().fold(0.0_f64, f64::max);
        assert!(
            (max_r - min_r).abs() < 0.2,
            "Ring graph radii should be ~equal; got min={} max={}",
            min_r,
            max_r
        );
        // And the max radius should be normalized to ~0.95
        assert!(
            (max_r - 0.95).abs() < 0.01,
            "Max radius should be 0.95 after normalization; got {}",
            max_r
        );
    }

    /// Determinism: same input yields same output across calls.
    #[test]
    fn determinism_on_repeated_calls() {
        let adj = vec![
            vec![1, 2, 3],
            vec![0, 2],
            vec![0, 1, 3],
            vec![0, 2],
        ];
        let n = 4;

        let c1 = compute_centrality_normalized(&adj, n);
        let c2 = compute_centrality_normalized(&adj, n);
        assert_eq!(c1, c2);

        let l1 = pick_landmarks(&c1, 3);
        let l2 = pick_landmarks(&c2, 3);
        assert_eq!(l1, l2);

        let d1 = compute_landmark_distances(&adj, &l1, n);
        let d2 = compute_landmark_distances(&adj, &l2, n);
        assert_eq!(d1, d2);

        let m1 = classical_mds_2d(&d1, &l1);
        let m2 = classical_mds_2d(&d2, &l2);
        for i in 0..m1.len() {
            assert!((m1[i].0 - m2[i].0).abs() < 1e-9);
            assert!((m1[i].1 - m2[i].1).abs() < 1e-9);
        }
    }

    /// Disconnected-component handling: 2 separate triangles.
    #[test]
    fn disconnected_components_no_panic() {
        // 0-1-2-0  (triangle)   3-4-5-3 (triangle, disconnected)
        let adj = vec![
            vec![1, 2],
            vec![0, 2],
            vec![0, 1],
            vec![4, 5],
            vec![3, 5],
            vec![3, 4],
        ];
        let n = 6;
        let centrality = compute_centrality_normalized(&adj, n);
        let landmarks = pick_landmarks(&centrality, 4);
        let dist_matrix = compute_landmark_distances(&adj, &landmarks, n);
        let coords = classical_mds_2d(&dist_matrix, &landmarks);
        // Just confirm we don't panic + return valid coords
        assert_eq!(coords.len(), 4);
        for (x, y) in &coords {
            assert!(x.is_finite());
            assert!(y.is_finite());
        }
    }

    #[test]
    fn single_node_universe() {
        let adj: Vec<Vec<usize>> = vec![vec![]];
        let n = 1;
        let centrality = compute_centrality_normalized(&adj, n);
        assert_eq!(centrality, vec![0.0]);
    }

    #[test]
    fn library_set_hash_is_deterministic_and_order_invariant() {
        let lp1 = vec![
            ("a/b".to_string(), "Lib A".to_string()),
            ("c/d".to_string(), "Lib B".to_string()),
        ];
        let lp2 = vec![
            ("c/d".to_string(), "Lib B".to_string()),
            ("a/b".to_string(), "Lib A".to_string()),
        ];
        let h1 = compute_library_set_hash(&lp1);
        let h2 = compute_library_set_hash(&lp2);
        assert_eq!(h1, h2, "Hash should be order-invariant");
        // Hash should be stable across calls
        let h3 = compute_library_set_hash(&lp1);
        assert_eq!(h1, h3);
    }

    // ─── MIG-019 §2A — TF-IDF unit tests ────────────────────────────

    /// 3-document corpus. "shared" appears in all 3 → low IDF.
    /// "rare" appears in 1 → high IDF. Verify the formula matches:
    /// log((N+1)/(df+1)) + 1.
    #[test]
    fn idf_formula_basic_corpus() {
        let mut d1 = HashMap::new();
        d1.insert("shared".to_string(), 3u32);
        d1.insert("rare".to_string(), 1u32);
        let mut d2 = HashMap::new();
        d2.insert("shared".to_string(), 2u32);
        let mut d3 = HashMap::new();
        d3.insert("shared".to_string(), 1u32);
        d3.insert("medium".to_string(), 1u32);

        let docs = vec![d1, d2, d3];
        let idf = compute_idf(&docs);

        // df("shared") = 3, N = 3 → IDF = ln(4/4) + 1 = 1.0
        assert!((idf["shared"] - 1.0).abs() < 1e-9);
        // df("rare") = 1 → IDF = ln(4/2) + 1 = ln(2) + 1 ≈ 1.6931
        assert!((idf["rare"] - (2.0_f64.ln() + 1.0)).abs() < 1e-9);
        // df("medium") = 1 → same as rare
        assert!((idf["medium"] - idf["rare"]).abs() < 1e-9);
    }

    /// Build sparse TF×IDF and verify L2 norm ≈ 1.0 (so cosine
    /// becomes a dot product later).
    #[test]
    fn tfidf_l2_normalized() {
        let mut counts = HashMap::new();
        counts.insert("foo".to_string(), 2u32);
        counts.insert("bar".to_string(), 5u32);
        counts.insert("baz".to_string(), 1u32);

        let mut idf = HashMap::new();
        idf.insert("foo".to_string(), 1.5);
        idf.insert("bar".to_string(), 2.0);
        idf.insert("baz".to_string(), 1.0);

        let v = build_sparse_tfidf(&counts, &idf, 50);
        let norm_squared: f64 = v.iter().map(|(_, s)| s * s).sum();
        assert!(
            (norm_squared - 1.0).abs() < 1e-9,
            "L2-normalized vector should have norm² = 1.0; got {}",
            norm_squared
        );
        // top-k truncation: 3 terms requested via k=50 → all 3 kept
        assert_eq!(v.len(), 3);
    }

    /// Top-k truncation drops lowest-score terms first.
    #[test]
    fn tfidf_truncates_to_top_k() {
        let mut counts = HashMap::new();
        for term in &["a", "b", "c", "d", "e"] {
            counts.insert(term.to_string(), 1u32);
        }
        let mut idf = HashMap::new();
        idf.insert("a".to_string(), 5.0); // highest
        idf.insert("b".to_string(), 4.0);
        idf.insert("c".to_string(), 3.0);
        idf.insert("d".to_string(), 2.0);
        idf.insert("e".to_string(), 1.0); // lowest

        let v = build_sparse_tfidf(&counts, &idf, 3);
        assert_eq!(v.len(), 3);
        // Top 3 should be a, b, c (highest IDF * tf scores)
        let kept: HashSet<&str> = v.iter().map(|(t, _)| t.as_str()).collect();
        assert!(kept.contains("a"));
        assert!(kept.contains("b"));
        assert!(kept.contains("c"));
        assert!(!kept.contains("d"));
        assert!(!kept.contains("e"));
    }

    /// Identical L2-normalized vectors → cosine = 1.
    #[test]
    fn cosine_identical_vectors_eq_one() {
        let v: Vec<(String, f64)> = vec![
            ("a".to_string(), 0.6),
            ("b".to_string(), 0.8), // 0.6² + 0.8² = 1.0
        ];
        let s = cosine_sparse(&v, &v);
        assert!((s - 1.0).abs() < 1e-9);
    }

    /// Disjoint sparse vectors (no shared terms) → cosine = 0.
    #[test]
    fn cosine_disjoint_vectors_eq_zero() {
        let a: Vec<(String, f64)> = vec![("a".to_string(), 1.0)];
        let b: Vec<(String, f64)> = vec![("b".to_string(), 1.0)];
        let s = cosine_sparse(&a, &b);
        assert!(s.abs() < 1e-9);
    }

    /// Empty body → empty vector → cosine = 0.
    #[test]
    fn cosine_handles_empty_vector() {
        let a: Vec<(String, f64)> = vec![];
        let b: Vec<(String, f64)> = vec![("foo".to_string(), 1.0)];
        let s = cosine_sparse(&a, &b);
        assert_eq!(s, 0.0);
    }

    // ─── MIG-019 §2A+§2B redesign — density-grid unit tests ────────

    /// Single line through the grid centre accumulates weight along its
    /// trajectory and only along its trajectory.
    #[test]
    fn rasterize_line_accumulates_along_path() {
        let gw = 10;
        let gh = 10;
        let mut grid = vec![0.0_f32; gw * gh];
        rasterize_line(&mut grid, gw, gh, 0.0, 0.0, 9.0, 9.0, 1.0);
        // Diagonal from (0,0) to (9,9): cells (0,0), (1,1), ..., (9,9) all incremented
        for i in 0..10 {
            assert!(grid[i * gw + i] >= 1.0, "diagonal cell ({},{}) untouched", i, i);
        }
        // Off-diagonal cell (0, 9) should NOT be touched
        assert_eq!(grid[9 * gw + 0], 0.0);
        assert_eq!(grid[0 * gw + 9], 0.0);
    }

    /// Lines outside grid bounds clamp gracefully — no panic, no
    /// out-of-bounds writes, no spurious increments.
    #[test]
    fn rasterize_line_handles_out_of_bounds() {
        let gw = 5;
        let gh = 5;
        let mut grid = vec![0.0_f32; gw * gh];
        rasterize_line(&mut grid, gw, gh, -10.0, -10.0, -5.0, -5.0, 1.0);
        for v in &grid {
            assert_eq!(*v, 0.0);
        }
    }

    /// embed_to_grid maps unit-disk corners to grid corners.
    #[test]
    fn embed_to_grid_maps_corners() {
        let gw = 10;
        let gh = 10;
        let (gx, gy) = embed_to_grid(-1.0, -1.0, gw, gh);
        assert!((gx - 0.0).abs() < 1e-3);
        assert!((gy - 0.0).abs() < 1e-3);
        let (gx, gy) = embed_to_grid(1.0, 1.0, gw, gh);
        assert!((gx - 9.0).abs() < 1e-3); // clamped to gw-1
        assert!((gy - 9.0).abs() < 1e-3);
        let (gx, gy) = embed_to_grid(0.0, 0.0, gw, gh);
        assert!((gx - 5.0).abs() < 1e-3); // centre
        assert!((gy - 5.0).abs() < 1e-3);
    }

    /// Gaussian blur is mass-conserving (modulo border effects). A
    /// single hot cell spreads to 9 cells with the 3×3 separable kernel.
    #[test]
    fn gaussian_blur_spreads_single_hot_cell() {
        let gw = 5;
        let gh = 5;
        let mut grid = vec![0.0_f32; gw * gh];
        grid[2 * gw + 2] = 16.0; // centre
        gaussian_blur_separable(&mut grid, gw, gh);
        // Centre should receive the most weight; 4 neighbours next; 4 corners least.
        let centre = grid[2 * gw + 2];
        let cardinal = grid[1 * gw + 2]; // up
        let diagonal = grid[1 * gw + 1]; // up-left
        assert!(centre > cardinal);
        assert!(cardinal > diagonal);
        assert!(diagonal > 0.0);
    }

    /// Determinism: same corpus → same IDF + same sparse vectors,
    /// independent of HashMap iteration order quirks.
    #[test]
    fn tfidf_pipeline_deterministic() {
        let mut counts1 = HashMap::new();
        counts1.insert("alpha".to_string(), 3u32);
        counts1.insert("beta".to_string(), 1u32);
        let mut counts2 = HashMap::new();
        counts2.insert("alpha".to_string(), 1u32);
        counts2.insert("gamma".to_string(), 2u32);
        let docs = vec![counts1.clone(), counts2.clone()];

        let idf_a = compute_idf(&docs);
        let idf_b = compute_idf(&docs);
        for (term, value) in &idf_a {
            assert!((idf_b[term] - value).abs() < 1e-12);
        }

        let v_a = build_sparse_tfidf(&counts1, &idf_a, 50);
        let v_b = build_sparse_tfidf(&counts1, &idf_a, 50);
        assert_eq!(v_a.len(), v_b.len());
        for ((ta, sa), (tb, sb)) in v_a.iter().zip(v_b.iter()) {
            assert_eq!(ta, tb);
            assert!((sa - sb).abs() < 1e-12);
        }
    }
}
