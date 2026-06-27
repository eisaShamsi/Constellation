//! CNS (Constellation Nervous System) — the network-analysis engine.
//!
//! (File renamed from `lens.rs` to `sight.rs` 2026-04-27 per MIG-009 to
//! match the then-current surface name. The `Lens*` type names below
//! are internal Rust-side identifiers; renaming them would churn the
//! wire format unnecessarily — they're only ever consumed by the
//! `+layout.svelte` `toggleLens()` flow as a typed JSON payload. The
//! surface itself is the CNS gravity well, `ConstellationSight2.svelte`,
//! per the ratified Constellation-Nervous-System-Concept-Paper.)
//!
//! - Betweenness centrality (Brandes' algorithm) — finds Bridge notes.
//!
//! MIG-075 §A1 — the centrality input is read from `note_links` (the
//! write-time-maintained link record, MIG-067-correct) instead of
//! re-reading every .md via scan_library_links (Perf Rule 8: reads are
//! cheap indexed lookups; the corpus is never walked on the open path).
//! The command is async so Brandes never blocks the WebView2 UI thread.
//! The DB lock is held for the row read only, released before any
//! graph computation.
//!
//! The frontend handles community detection (Louvain, clusterEngine.ts),
//! structural gap detection, entropy, and the cohesion score.

use petgraph::graph::{NodeIndex, UnGraph};
use serde::Serialize;
use std::collections::{HashMap, VecDeque};
use tauri::Manager;

/// Weight for each typed link — higher = stronger connection.
fn link_type_weight(link_type: &Option<String>) -> f64 {
    match link_type.as_deref() {
        Some("supports") => 1.0,
        Some("causes") => 0.9,
        Some("contradicts") => 0.8,
        Some("derives-from") => 0.7,
        Some("generalizes") => 0.8,
        Some("exemplifies") => 0.6,
        Some("part-of") => 0.7,
        Some("associative") => 0.5,
        _ => 1.0, // default: untyped wikilink
    }
}

/// Result of centrality computation — sent to frontend via IPC.
/// (MIG-075 §A1 dropped the `contradictions` field — its only frontend
/// consumer was a dead prop; the pair list is `detect_tensions`' per the
/// ratified CNS paper §5. The MIG-075 audit follow-up dropped
/// `diversivity` — computed + serialized for ~7.6k nodes per open with
/// zero readers; reinstate from history if a register ever wants it.)
#[derive(Debug, Clone, Serialize)]
pub struct LensCentralityData {
    /// Map from note_id (lowercase name) to normalized betweenness centrality (0.0–1.0).
    pub centrality: HashMap<String, f64>,
    pub node_count: u32,
    pub edge_count: u32,
}

/// Compute betweenness centrality for the active universe's link graph.
///
/// Reads `(source_name, target_name, link_type)` from `note_links`
/// (`status='active'`) — one indexed read; the DB lock is released
/// before any computation. Uses Brandes' algorithm (O(VE)) on an
/// undirected graph. Node IDs are lowercase note names (matching the
/// frontend's SimNode ids). Scope = the active universe's own DB —
/// exact parity with the retired fs walk, whose cUniverse scans failed
/// library validation and were silently swallowed.
#[tauri::command(async)]
pub fn constellation_sight_centrality(
    app: tauri::AppHandle,
) -> Result<LensCentralityData, String> {
    let rows: Vec<(String, String, Option<String>)> = {
        let state = app.state::<crate::search::SearchState>();
        let db = state.db.lock().map_err(|e| e.to_string())?;
        let conn = db.as_ref().ok_or("Search DB not initialized")?;
        // PJ-065 — the structural (parent/TOC) lane is non-cognitive: it must not
        // enter the Brandes centrality graph. No-op until §5.
        let sx = crate::link_types::snapshot().structural_not_in_clause("link_type");
        let mut stmt = conn
            .prepare(&format!("SELECT source_name, target_name, link_type FROM note_links WHERE status = 'active'{}", sx))
            .map_err(|e| e.to_string())?;
        let mapped = stmt
            .query_map([], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, Option<String>>(2)?,
                ))
            })
            .map_err(|e| e.to_string())?;
        mapped.filter_map(|r| r.ok()).collect()
    };

    Ok(compute_centrality_from_links(rows))
}

/// Pure core (testable without an AppHandle): build the weighted
/// undirected graph from link rows and run Brandes.
pub(crate) fn compute_centrality_from_links(
    rows: Vec<(String, String, Option<String>)>,
) -> LensCentralityData {
    struct RawLink { source: String, target: String, link_type: Option<String> }
    let mut all_links: Vec<RawLink> = Vec::with_capacity(rows.len());

    for (source, target, link_type) in rows {
        let source = source.to_lowercase();
        let target = target.to_lowercase(); // lowered at index time; defensive
        if source.is_empty() || target.is_empty() || source == target { continue; }
        // Weight parity with the retired fs walk: the indexer stores plain
        // untyped links as 'associative' (and legacy rows as 'relates'),
        // while the walk's resolver returned None for them → default
        // weight. Null-type membership is defined ONCE in link_types.rs.
        let link_type = match link_type {
            Some(ref lt) if crate::link_types::is_null_type(lt) => None,
            other => other,
        };
        all_links.push(RawLink { source, target, link_type });
    }

    // 2. Build weighted petgraph undirected graph (edge weight = typed link weight)
    let mut graph = UnGraph::<String, f64>::new_undirected();
    let mut name_to_idx: HashMap<String, NodeIndex> = HashMap::new();

    for rl in &all_links {
        if !name_to_idx.contains_key(&rl.source) {
            let idx = graph.add_node(rl.source.clone());
            name_to_idx.insert(rl.source.clone(), idx);
        }
        if !name_to_idx.contains_key(&rl.target) {
            let idx = graph.add_node(rl.target.clone());
            name_to_idx.insert(rl.target.clone(), idx);
        }
    }

    // Deduplicate edges, keep max weight for each pair
    let mut edge_weights: HashMap<(NodeIndex, NodeIndex), f64> = HashMap::new();
    for rl in &all_links {
        let si = name_to_idx[&rl.source];
        let ti = name_to_idx[&rl.target];
        let key = if si < ti { (si, ti) } else { (ti, si) };
        let w = link_type_weight(&rl.link_type);
        let entry = edge_weights.entry(key).or_insert(0.0);
        if w > *entry { *entry = w; }
    }
    for ((si, ti), w) in &edge_weights {
        graph.add_edge(*si, *ti, *w);
    }

    let n = graph.node_count();
    let e = graph.edge_count();

    if n == 0 {
        return LensCentralityData {
            centrality: HashMap::new(),
            node_count: 0,
            edge_count: 0,
        };
    }

    // 3. Brandes' betweenness centrality
    // For large graphs (>500 nodes), use approximate centrality via sampling
    // to keep computation under 2 seconds. Sampling-based approximation is
    // well-established in network science literature.
    let centrality_scores = if n > 500 {
        let sample_size = std::cmp::min(200, n); // sample 200 source nodes max
        brandes_betweenness_approx(&graph, sample_size)
    } else {
        brandes_betweenness(&graph)
    };

    // 4. Normalize centrality to 0.0–1.0
    let max_score = centrality_scores.values().cloned().fold(0.0_f64, f64::max);
    let mut centrality: HashMap<String, f64> = HashMap::new();

    for (idx, score) in &centrality_scores {
        let name = &graph[*idx];
        let normalized = if max_score > 0.0 { score / max_score } else { 0.0 };
        centrality.insert(name.clone(), normalized);
    }

    LensCentralityData {
        centrality,
        node_count: n as u32,
        edge_count: e as u32,
    }
}

/// Brandes' algorithm for betweenness centrality on an undirected unweighted graph.
///
/// Reference: Ulrik Brandes (2001), "A Faster Algorithm for Betweenness Centrality"
/// Complexity: O(V × E) for unweighted graphs.
fn brandes_betweenness(graph: &UnGraph<String, f64>) -> HashMap<NodeIndex, f64> {
    let n = graph.node_count();
    let mut cb: HashMap<NodeIndex, f64> = HashMap::with_capacity(n);

    // Initialize all centrality scores to 0
    for idx in graph.node_indices() {
        cb.insert(idx, 0.0);
    }

    // For each source node s, run BFS and accumulate dependency
    for s in graph.node_indices() {
        // BFS from s
        let mut stack: Vec<NodeIndex> = Vec::with_capacity(n);
        let mut pred: HashMap<NodeIndex, Vec<NodeIndex>> = HashMap::new();
        let mut sigma: HashMap<NodeIndex, f64> = HashMap::new(); // # shortest paths
        let mut dist: HashMap<NodeIndex, i64> = HashMap::new();  // distance from s

        for v in graph.node_indices() {
            pred.insert(v, Vec::new());
            sigma.insert(v, 0.0);
            dist.insert(v, -1);
        }
        *sigma.get_mut(&s).unwrap() = 1.0;
        *dist.get_mut(&s).unwrap() = 0;

        let mut queue: VecDeque<NodeIndex> = VecDeque::new();
        queue.push_back(s);

        while let Some(v) = queue.pop_front() {
            stack.push(v);
            let d_v = dist[&v];

            for w in graph.neighbors(v) {
                // w found for the first time?
                if dist[&w] < 0 {
                    queue.push_back(w);
                    *dist.get_mut(&w).unwrap() = d_v + 1;
                }
                // shortest path to w via v?
                if dist[&w] == d_v + 1 {
                    *sigma.get_mut(&w).unwrap() += sigma[&v];
                    pred.get_mut(&w).unwrap().push(v);
                }
            }
        }

        // Accumulation: back-propagate dependencies
        let mut delta: HashMap<NodeIndex, f64> = HashMap::new();
        for v in graph.node_indices() {
            delta.insert(v, 0.0);
        }

        while let Some(w) = stack.pop() {
            for v in &pred[&w] {
                let d = (sigma[v] / sigma[&w]) * (1.0 + delta[&w]);
                *delta.get_mut(v).unwrap() += d;
            }
            if w != s {
                *cb.get_mut(&w).unwrap() += delta[&w];
            }
        }
    }

    // For undirected graphs, each shortest path is counted twice
    for score in cb.values_mut() {
        *score /= 2.0;
    }

    cb
}

/// Approximate betweenness centrality via random sampling.
/// Instead of running BFS from ALL nodes, sample `k` random source nodes.
/// Produces proportionally accurate rankings for large graphs.
fn brandes_betweenness_approx(graph: &UnGraph<String, f64>, k: usize) -> HashMap<NodeIndex, f64> {
    let n = graph.node_count();
    let mut cb: HashMap<NodeIndex, f64> = HashMap::with_capacity(n);
    for idx in graph.node_indices() { cb.insert(idx, 0.0); }

    // Collect all node indices and sample k of them
    let all_indices: Vec<NodeIndex> = graph.node_indices().collect();
    let sample_size = std::cmp::min(k, all_indices.len());

    // Deterministic sampling: pick evenly spaced indices for reproducibility
    let step = if sample_size > 0 { all_indices.len() / sample_size } else { 1 };
    let sources: Vec<NodeIndex> = (0..sample_size).map(|i| all_indices[i * step]).collect();

    for s in sources {
        let mut stack: Vec<NodeIndex> = Vec::with_capacity(n);
        let mut pred: HashMap<NodeIndex, Vec<NodeIndex>> = HashMap::new();
        let mut sigma: HashMap<NodeIndex, f64> = HashMap::new();
        let mut dist: HashMap<NodeIndex, i64> = HashMap::new();

        for v in graph.node_indices() {
            pred.insert(v, Vec::new());
            sigma.insert(v, 0.0);
            dist.insert(v, -1);
        }
        *sigma.get_mut(&s).unwrap() = 1.0;
        *dist.get_mut(&s).unwrap() = 0;

        let mut queue: VecDeque<NodeIndex> = VecDeque::new();
        queue.push_back(s);

        while let Some(v) = queue.pop_front() {
            stack.push(v);
            let d_v = dist[&v];
            for w in graph.neighbors(v) {
                if dist[&w] < 0 {
                    queue.push_back(w);
                    *dist.get_mut(&w).unwrap() = d_v + 1;
                }
                if dist[&w] == d_v + 1 {
                    *sigma.get_mut(&w).unwrap() += sigma[&v];
                    pred.get_mut(&w).unwrap().push(v);
                }
            }
        }

        let mut delta: HashMap<NodeIndex, f64> = HashMap::new();
        for v in graph.node_indices() { delta.insert(v, 0.0); }

        while let Some(w) = stack.pop() {
            for v in &pred[&w] {
                let d = (sigma[v] / sigma[&w]) * (1.0 + delta[&w]);
                *delta.get_mut(v).unwrap() += d;
            }
            if w != s {
                *cb.get_mut(&w).unwrap() += delta[&w];
            }
        }
    }

    // Scale to approximate full centrality
    let scale = if sample_size > 0 { n as f64 / sample_size as f64 } else { 1.0 };
    for score in cb.values_mut() {
        *score = (*score * scale) / 2.0; // /2 for undirected
    }

    cb
}

// (MIG-075 §A3 removed the Shared-Tag Edges section — `TagEdge`,
// `constellation_sight_tag_edges`, `scan_note_tags_recursive`. The
// command had ZERO frontend callers (verified repo-wide) and was an
// fs walk besides. Shared-tag analysis, if ever wanted again, reads
// `json_each(note_meta.tags_json)` — see tension.rs's loader.)

// ─── MIG-075 §A1 tests ─────────────────────────────────────────────

#[cfg(test)]
mod tests_mig075_sight {
    //! Pins the DB-sourced centrality core: graph shape, the known-bridge
    //! ranking, the untyped-weight parity mapping, and row hygiene
    //! (self-links / empty targets skipped; per-pair edge dedupe).
    use super::*;

    fn row(s: &str, t: &str, lt: Option<&str>) -> (String, String, Option<String>) {
        (s.to_string(), t.to_string(), lt.map(|x| x.to_string()))
    }

    /// Two clusters joined ONLY through "bridge" — Brandes must rank the
    /// bridge note strictly highest (normalized 1.0).
    #[test]
    fn bridge_note_ranks_highest() {
        let rows = vec![
            // cluster 1
            row("a1", "a2", Some("supports")),
            row("a2", "a3", None),
            row("a3", "a1", Some("associative")), // untyped-stored form
            // the bridge
            row("a1", "bridge", Some("derives-from")),
            row("bridge", "b1", Some("supports")),
            // cluster 2
            row("b1", "b2", None),
            row("b2", "b3", Some("contradicts")),
            row("b3", "b1", None),
        ];
        let data = compute_centrality_from_links(rows);
        assert_eq!(data.node_count, 7, "7 distinct notes");
        assert_eq!(data.edge_count, 8, "8 deduped undirected edges");
        assert_eq!(
            data.centrality.get("bridge").copied(),
            Some(1.0),
            "the sole connector normalizes to 1.0: {:?}",
            data.centrality
        );
        // Every other note routes fewer shortest paths than the bridge.
        for (name, score) in &data.centrality {
            if name != "bridge" {
                assert!(*score < 1.0, "{name} must rank below the bridge");
            }
        }
    }

    /// Self-links and empty targets are dropped; duplicate (source,target)
    /// rows collapse to ONE edge (max weight wins) — the documented
    /// occurrence-dedup delta vs the retired fs walk.
    #[test]
    fn row_hygiene_and_edge_dedupe() {
        let rows = vec![
            row("x", "x", None),            // self-link → dropped
            row("x", "", Some("supports")), // empty target → dropped
            row("x", "y", None),
            row("x", "y", Some("supports")), // same pair, typed → one edge
            row("y", "x", Some("causes")),   // reverse direction → same undirected pair
        ];
        let data = compute_centrality_from_links(rows);
        assert_eq!(data.node_count, 2);
        assert_eq!(data.edge_count, 1, "one undirected edge for the x↔y pair");
    }

    /// The empty universe returns an empty payload, not an error.
    #[test]
    fn empty_rows_empty_payload() {
        let data = compute_centrality_from_links(vec![]);
        assert_eq!((data.node_count, data.edge_count), (0, 0));
        assert!(data.centrality.is_empty());
    }
}
