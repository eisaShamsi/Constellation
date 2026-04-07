//! Constellation Lens — CE Layer 3: Network Analysis Engine.
//!
//! Applies graph algorithms to the user's knowledge graph:
//! - Betweenness centrality (Brandes' algorithm) — finds bridge notes
//! - Returns per-note centrality scores for the frontend to overlay on GraphMind
//!
//! The frontend handles community detection (Louvain, already in clusterEngine.ts),
//! structural gap detection, entropy, and universe health scoring.

use petgraph::graph::{NodeIndex, UnGraph};
use serde::Serialize;
use std::collections::{HashMap, VecDeque};

/// Result of centrality computation — sent to frontend via IPC.
#[derive(Debug, Clone, Serialize)]
pub struct LensCentralityData {
    /// Map from note_id (lowercase name) to normalized betweenness centrality (0.0–1.0).
    pub centrality: HashMap<String, f64>,
    pub node_count: u32,
    pub edge_count: u32,
}

/// Compute betweenness centrality for all notes across the given libraries.
///
/// Uses Brandes' algorithm (O(VE)) on an undirected graph built from wikilinks.
/// Node IDs are lowercase note names (matching StarNode.id in the frontend).
#[tauri::command]
pub fn constellation_lens_centrality(
    app: tauri::AppHandle,
    library_paths: Vec<(String, String)>, // (library_path, library_name) pairs
) -> Result<LensCentralityData, String> {
    // 1. Collect all links across all libraries
    let mut all_links: Vec<(String, String)> = Vec::new(); // (source_name, target_name) lowercase

    for (lib_path, lib_name) in &library_paths {
        let links = crate::libraries::scan_library_links(
            app.clone(), lib_path.clone(), lib_name.clone(),
        ).unwrap_or_default();

        for link in links {
            let source = link.source_name.to_lowercase();
            let target = link.target.to_lowercase();
            if source != target {
                all_links.push((source, target));
            }
        }
    }

    // 2. Build petgraph undirected graph
    let mut graph = UnGraph::<String, ()>::new_undirected();
    let mut name_to_idx: HashMap<String, NodeIndex> = HashMap::new();

    for (src, tgt) in &all_links {
        if !name_to_idx.contains_key(src) {
            let idx = graph.add_node(src.clone());
            name_to_idx.insert(src.clone(), idx);
        }
        if !name_to_idx.contains_key(tgt) {
            let idx = graph.add_node(tgt.clone());
            name_to_idx.insert(tgt.clone(), idx);
        }
    }

    // Deduplicate edges
    let mut edge_set = std::collections::HashSet::new();
    for (src, tgt) in &all_links {
        let si = name_to_idx[src];
        let ti = name_to_idx[tgt];
        let key = if si < ti { (si, ti) } else { (ti, si) };
        if edge_set.insert(key) {
            graph.add_edge(si, ti, ());
        }
    }

    let n = graph.node_count();
    let e = graph.edge_count();

    if n == 0 {
        return Ok(LensCentralityData {
            centrality: HashMap::new(),
            node_count: 0,
            edge_count: 0,
        });
    }

    // 3. Brandes' betweenness centrality algorithm (O(VE))
    let centrality_scores = brandes_betweenness(&graph);

    // 4. Normalize to 0.0–1.0
    let max_score = centrality_scores.values().cloned().fold(0.0_f64, f64::max);
    let mut centrality: HashMap<String, f64> = HashMap::new();

    for (idx, score) in &centrality_scores {
        let name = &graph[*idx];
        let normalized = if max_score > 0.0 { score / max_score } else { 0.0 };
        centrality.insert(name.clone(), normalized);
    }

    Ok(LensCentralityData {
        centrality,
        node_count: n as u32,
        edge_count: e as u32,
    })
}

/// Brandes' algorithm for betweenness centrality on an undirected unweighted graph.
///
/// Reference: Ulrik Brandes (2001), "A Faster Algorithm for Betweenness Centrality"
/// Complexity: O(V × E) for unweighted graphs.
fn brandes_betweenness(graph: &UnGraph<String, ()>) -> HashMap<NodeIndex, f64> {
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
