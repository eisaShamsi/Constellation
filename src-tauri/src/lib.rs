mod ai;
// MIG-013 §1A: visibility widened so `build_concept_vectors` [[bin]]
// can name `arabic::Lang` (the language enum used by lexicon::ConceptRecord).
pub mod arabic;
mod bases;
// MIG-013 §1A: CTSE Bridge Vector Store. Public so the offline
// `build_concept_vectors` [[bin]] can reference layout constants
// (ASSET_MAGIC, VECTOR_DIM) when emitting the binary asset.
pub mod bridge_vectors;
mod canvas;
mod boot_bundle;
mod cache;
mod canonical;
// MIG-013 §1B: CTSE Bridge Adapter (term → M11 concept resolver).
// Public so future write-time hooks (in §1C) can call it from any
// crate-internal module without going through a re-export chain.
pub mod ctse;
// MIG-013 §1A: visibility widened from `mod` to `pub mod` so the
// offline `build_concept_vectors` [[bin]] target can call
// `embeddings::embed_passages_standalone`. No behavioral change —
// purely additive: in-crate access paths are unchanged.
pub mod embeddings;
mod embeds;
mod dataview;
mod file_kinds;
mod fts5_tokenizer;
mod inspector360;
mod importers;
mod libraries;
mod mig003_step4;
mod sight;
mod sight_layout;
mod sources;
mod classifier;
// MIG-040: Note Summary Creator (NSC) — extractive embedding-based TextRank.
mod nsc;
// MIG-021v3 V3-§1 — Constellation Epistemic Content Engine (CECE).
// Cataloger Ensemble Architecture. See lab/reports/MIG-021v3-EPISTEMIC-
// CONTENT-ENGINE-ARCHITECT.md for the architecture spec.
pub mod cece;
mod lenses;
// MIG-055 §A — Constellation Base, the lens module (clean rebuild post-MIG-054 revert).
pub mod lens;
// MIG-056 §A — Cross-Universe Federation module. Provides FederationContext +
// (in later steps) attach/migrate/query helpers for the 4 federation consumers
// (lens / status bar / libraryStats / global search).
pub mod federation;
// MIG-028 (2026-05-18) — Sight v5 module retired with the v5 codebase.
// Plan/Architect docs at lab/reports/MIG-024-SIGHT-V5-* stay on disk
// as historical record.
// MIG-025 §A — Sight v6 layout cache + IPCs (per Concept Paper v4.0
// §9.3; was B2 dual-mounted with v5; v5 retired in MIG-028).
pub mod sight_v6;
// MIG-036 P1 (2026-05-19) — Sight v7 Form-Aligns-To-Purpose redesign.
// B2 dual-mount with v6: both modules coexist during v7 dev. v7
// reuses sight_v6's cache schema (LayoutCacheRow shape unchanged);
// v7's IPCs land in P3+ as they're built. See
// lab/reports/MIG-036-SIGHT-V7-ARCHITECT.md.
pub mod sight_v7;
// MIG-013 §1A: visibility widened so the offline `build_concept_vectors`
// [[bin]] can call `lexicon::parse`. The M11 zero-diff invariant covers
// `src-tauri/src/lexicon/**` (the data + module sources), not `lib.rs`.
// Verified by `git diff src-tauri/src/lexicon/` returning empty.
pub mod lexicon;
mod search;
mod sky_backfill;
mod links_backfill;
mod note_body_backfill;
mod tag_counts;
mod incoming_links_backfill;
mod link_boot_index;
mod reconcile;
mod link_types;
mod style_presets;
mod map;
mod maturity;
mod perf_trace;
mod provenance;
mod review;
mod review_backfill;
#[cfg(test)]
mod review_rehearse;
mod strata;
mod tension;
mod trails;
mod tasks;
mod universe;
mod watcher;
mod watcher_suppress;
pub mod write_gate;

use tauri::{Emitter, Manager};

#[tauri::command]
fn open_second_screen(app: tauri::AppHandle) -> Result<(), String> {
    if let Some(win) = app.get_webview_window("second-screen") {
        win.show().map_err(|e| e.to_string())?;
        win.set_focus().map_err(|e| e.to_string())?;
        // Force WebView2 to repaint after showing a hidden window
        let _ = win.eval("void(0)");
    }
    Ok(())
}

#[tauri::command]
fn constellation_show_in_folder(path: String) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("explorer")
            .arg(format!("/select,{}", path))
            .spawn()
            .map_err(|e| e.to_string())?;
    }
    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .args(["-R", &path])
            .spawn()
            .map_err(|e| e.to_string())?;
    }
    #[cfg(target_os = "linux")]
    {
        if let Some(parent) = std::path::Path::new(&path).parent() {
            std::process::Command::new("xdg-open")
                .arg(parent)
                .spawn()
                .map_err(|e| e.to_string())?;
        }
    }
    Ok(())
}

#[tauri::command]
fn open_path(path: String) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("cmd")
            .args(["/c", "start", "", &path])
            .spawn()
            .map_err(|e| e.to_string())?;
    }
    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .arg(&path)
            .spawn()
            .map_err(|e| e.to_string())?;
    }
    #[cfg(target_os = "linux")]
    {
        std::process::Command::new("xdg-open")
            .arg(&path)
            .spawn()
            .map_err(|e| e.to_string())?;
    }
    Ok(())
}

/* ------------------------------------------------------------------ */
/*  Monitor / display detection                                        */
/* ------------------------------------------------------------------ */

#[derive(serde::Serialize, Clone)]
pub struct MonitorInfo {
    pub name: Option<String>,
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
    pub scale_factor: f64,
    pub is_primary: bool,
}

#[tauri::command]
fn list_monitors(app: tauri::AppHandle) -> Vec<MonitorInfo> {
    let mut monitors = Vec::new();
    let primary = app.primary_monitor().ok().flatten();
    let primary_name = primary.as_ref().and_then(|m| m.name().map(|n| n.to_string()));

    if let Ok(available) = app.available_monitors() {
        for mon in available {
            let pos = mon.position();
            let size = mon.size();
            let name = mon.name().map(|n| n.to_string());
            let is_primary = match (&name, &primary_name) {
                (Some(n), Some(pn)) => n == pn,
                _ => false,
            };
            monitors.push(MonitorInfo {
                name,
                x: pos.x,
                y: pos.y,
                width: size.width,
                height: size.height,
                scale_factor: mon.scale_factor(),
                is_primary,
            });
        }
    }
    monitors
}

/// Open the second screen, auto-positioning on a secondary monitor if available.
#[tauri::command]
fn open_second_screen_on_monitor(app: tauri::AppHandle) -> Result<(), String> {
    let win = app.get_webview_window("second-screen")
        .ok_or_else(|| "second-screen window not found".to_string())?;

    let monitors: Vec<_> = app.available_monitors().map_err(|e| e.to_string())?
        .into_iter().collect();
    let primary = app.primary_monitor().ok().flatten();
    let primary_name = primary.as_ref().and_then(|m| m.name().map(|n| n.to_string()));

    // Strategy 1: Find monitor by name (different from primary)
    let secondary = monitors.iter().find(|m| {
        match (m.name().map(|n| n.to_string()), &primary_name) {
            (Some(n), Some(pn)) => n != *pn,
            _ => false, // Don't assume unnamed monitors are secondary
        }
    }).or_else(|| {
        // Strategy 2: Pick the monitor NOT at origin (0,0) — primary is usually there
        monitors.iter().find(|m| {
            let pos = m.position();
            pos.x != 0 || pos.y != 0
        })
    }).or_else(|| {
        // Strategy 3: If all monitors are at (0,0), pick the second one
        if monitors.len() > 1 { Some(&monitors[1]) } else { None }
    });

    if let Some(mon) = secondary {
        let pos = mon.position();
        let size = mon.size();
        let win_w = (size.width as f64 * 0.8) as u32;
        let win_h = (size.height as f64 * 0.8) as u32;
        let win_x = pos.x + ((size.width - win_w) / 2) as i32;
        let win_y = pos.y + ((size.height - win_h) / 2) as i32;

        use tauri::PhysicalPosition;
        use tauri::PhysicalSize;
        let _ = win.set_position(PhysicalPosition::new(win_x, win_y));
        let _ = win.set_size(PhysicalSize::new(win_w, win_h));
    }

    win.show().map_err(|e| e.to_string())?;
    win.set_focus().map_err(|e| e.to_string())?;
    let _ = win.eval("void(0)");
    Ok(())
}

#[tauri::command]
fn close_second_screen(app: tauri::AppHandle) -> Result<(), String> {
    if let Some(win) = app.get_webview_window("second-screen") {
        win.hide().map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[tauri::command]
fn is_second_screen_open(app: tauri::AppHandle) -> bool {
    if let Some(win) = app.get_webview_window("second-screen") {
        win.is_visible().unwrap_or(false)
    } else {
        false
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Install panic hook to log crashes before the process exits
    std::panic::set_hook(Box::new(|info| {
        let msg = format!("[CONSTELLATION PANIC] {}", info);
        eprintln!("{}", msg);
        // Also write to a crash log file next to the executable
        if let Ok(exe) = std::env::current_exe() {
            if let Some(dir) = exe.parent() {
                let log_path = dir.join("constellation-crash.log");
                let _ = std::fs::write(&log_path, &msg);
            }
        }
    }));

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        // MIG-076 §A1 — point the WriteGate's journal at the app-data dir
        // (best-effort: a missing dir only disables journaling, never writes).
        .setup(|app| {
            use tauri::Manager;
            if let Ok(dir) = app.path().app_data_dir() {
                crate::write_gate::init_journal(dir);
            }
            Ok(())
        })
        .manage(watcher::WatcherState::new())
        .manage(universe::UniverseState::new())
        .manage(search::SearchState::new())
        .manage(embeddings::EmbeddingState { engine: std::sync::Mutex::new(None), term_embed_cancel: std::sync::atomic::AtomicBool::new(false) })
        // MIG-021v2 §1F' — background scan state.
        .manage(classifier::scan_job::ScanState::new())
        // MIG-040 — NSC summary backfill state (Rule 8 first-time population).
        .manage(nsc::backfill::NscBackfillState::new())
        // MIG-021v2 §1F'.b — bulk Approve All state.
        .manage(sources::bulk_ops::BulkAcceptState::new())
        .invoke_handler({
            // Round 6 diagnostic (2026-04-19) — IPC arrival tracer.
            //
            // Round 5 (scan_library_tags → async) + DashboardView gate
            // (`{#if false}`) + JS heartbeat (max_gap = 112 ms) all
            // falsified their respective hypotheses: Criterion 2 still
            // fails at ~19 s with core_queue_ms ≈ 18.6 s, JS is alive,
            // and no single frontend gate moves the needle.
            //
            // The remaining unknown is what happens between JS
            // `postMessage` and the Rust command body entering. We wrap
            // `generate_handler!` in a closure that stamps a Unix-ms
            // timestamp on every dispatch into `perf_trace::TRACE_LOG`,
            // then forwards to the inner handler unchanged. The log is
            // fetched by the frontend at `boot:hydrated` via
            // `get_perf_trace_log` and bundled into the boot-perf JSON.
            //
            // Overhead is a single Mutex lock per command + a
            // `(String, u64)` push; negligible vs. any actual IPC cost.
            //
            // The `Box<dyn Fn(...)>` annotation pins the runtime to `Wry`
            // at the binding site — without it, the macro's `R: Runtime`
            // generic is unresolvable in a `let` binding (it only infers
            // when passed directly to `invoke_handler`).
            let inner: Box<dyn Fn(tauri::ipc::Invoke<tauri::Wry>) -> bool + Send + Sync + 'static> = Box::new(tauri::generate_handler![
            ai::ai_send_message,
            ai::ai_validate_connection,
            ai::ai_list_models,
            libraries::list_libraries,
            libraries::add_library,
            libraries::remove_library,
            libraries::read_library_tree,
            libraries::read_note,
            libraries::read_note_title,
            libraries::get_note_headings,
            libraries::write_note,
            libraries::pick_folder,
            libraries::create_new_library,
            libraries::create_new_library_at,
            libraries::get_all_library_stats,
            libraries::search_stars,
            libraries::search_by_property,
            libraries::create_note,
            libraries::create_folder,
            libraries::rename_item,
            libraries::move_item,
            libraries::list_universe_folders,
            libraries::delete_item,
            libraries::resolve_wikilink,
            libraries::resolve_wikilink_cross_library,
            libraries::read_library_appearance,
            libraries::scan_library_links,
            libraries::scan_unlinked_mentions,
            libraries::scan_library_tags,
            libraries::collect_library_notes,
            libraries::collect_library_notes_with_metadata,
            libraries::get_daily_note_path,
            libraries::quick_capture,
            libraries::update_links_on_rename,
            libraries::read_note_preview,
            libraries::save_clipboard_image,
            libraries::resolve_embed_image,
            embeds::resolve_embed,
            embeds::read_vault_config_cmd,
            embeds::invalidate_vault_index_cmd,
            libraries::scan_note_stages,
            // MIG-021 §1A — Sources subsystem IPCs
            sources::sources_get_for_note,
            sources::sources_set_manual,
            sources::sources_clear,
            // MIG-021 §1B — Tier-1 classifier IPC (on-demand single-note)
            classifier::classifier_suggest_for_note,
            // MIG-021v2 §1F' — background scan IPCs
            classifier::scan_job::classifier_scan_start,
            classifier::scan_job::classifier_scan_cancel,
            classifier::scan_job::classifier_scan_status,
            // MIG-021v3 V3-§8.r1.f — Sibling Disambiguation pick resolver
            classifier::cece_resolve_disambiguation,
            // MIG-021v3 V3-§9.C.2 — dual-axis reliability update IPC
            classifier::cece_record_correction_for_card,
            // MIG-040 — Note Summary Creator (NSC) batched + single get-or-compute.
            nsc::nsc_get_summaries_for_notes,
            nsc::nsc_get_summary,
            // MIG-040 — NSC summary backfill (background, resumable).
            nsc::backfill::nsc_backfill_start,
            nsc::backfill::nsc_backfill_cancel,
            nsc::backfill::nsc_backfill_status,
            // MIG-021v3 V3-§10.A — per-Library calibration view data
            cece::reliability::cece_get_reliability_for_active_library,
            cece::reliability::cece_get_active_library_root,
            // MIG-022 §B.4 — note state history query API
            cece::history::cece_get_note_history,
            cece::history::cece_query_history,
            // MIG-028 (2026-05-18): 4 sight_v5_* IPCs retired with the v5 module set.
            // MIG-025 §A.5 — Sight v6 layout cache IPCs
            sight_v6::sight_v6_get_layout,
            sight_v6::sight_v6_get_link_set_for_notes,
            sight_v6::sight_v6_warm_cache,
            // MIG-026 §κ.1 — user-defined tradition reader
            sight_v6::sight_v6_read_user_traditions,
            // MIG-026 §κ.2 — user-defined .js plugin reader (paths only)
            sight_v6::sight_v6_read_user_plugins,
            // MIG-021v2 §1F'.b — bulk Approve All / Reject All
            sources::bulk_ops::sources_accept_all_pending,
            sources::bulk_ops::sources_bulk_accept_cancel,
            sources::bulk_ops::sources_bulk_accept_status,
            sources::bulk_ops::sources_reject_all_pending,
            // MIG-021 §1C — Source Review queue IPCs
            sources::sources_get_suggestions,
            sources::sources_list_pending_suggestions,
            sources::sources_reject_suggestion,
            // MIG-021v2 §1A' — Content-type subsystem IPCs (vertical axis)
            sources::content_type_get_for_note,
            sources::content_type_set_manual,
            sources::content_type_clear,
            // MIG-021v2 §1A' — Taxonomy IPCs (single source of truth in Rust)
            sources::sources_get_horizontal_taxonomy,
            sources::sources_get_vertical_taxonomy,
            strata::compute_note_strata,
            maturity::compute_note_maturity,
            tension::detect_tensions,
            tension::note_tension_status,
            provenance::get_provenance_chain,
            provenance::compute_note_origins,
            review::get_due_notes,
            review::get_note_review_status,
            review::mark_reviewed,
            review::snooze_note,
            review::dismiss_note,
            trails::list_trails,
            trails::read_trail,
            canvas::list_canvases,
            inspector360::get_360_view,
            sight::constellation_sight_centrality,
            sight_layout::constellation_sight_v3_layout,
            sight_layout::constellation_sight_v3_invalidate_layout,
            sight_layout::constellation_sight_v3_density_field,
            boot_bundle::constellation_boot_bundle,
            cache::cache_boot_snapshot,
            cache::cache_boot_snapshot_core,
            cache::cache_boot_snapshot_graph,
            cache::cache_full_links,
            cache::get_backlink_rows,
            cache::get_outgoing_rows,
            cache::cache_boot_snapshot_sky,
            cache::cache_is_populated,
            cache::cache_reconcile,
            cache::cache_mark_search_ready,
            cache::write_boot_perf_report,
            cache::read_boot_perf_report,
            search::constellation_search_init,
            search::constellation_search,
            search::constellation_search_reindex,
            search::diag_log_line,
            search::constellation_search_store_embedding,
            search::constellation_search_similar,
            search::constellation_search_universal,
            search::constellation_search_link_counts,
            search::constellation_link_stats,
            search::constellation_link_traverse,
            search::constellation_debug_link_state,
            search::constellation_link_dormant,
            search::constellation_link_decay,
            search::constellation_link_set_confidence,
            search::constellation_link_backfill_confidence,
            search::constellation_link_archive,
            search::constellation_link_unarchive,
            search::constellation_link_archived,
            search::constellation_formulation_analysis,
            search::constellation_knowledge_health_snapshot,
            search::constellation_ccs_snapshot,
            embeddings::constellation_init_embeddings,
            embeddings::constellation_embed_text,
            embeddings::constellation_embed_texts,
            embeddings::constellation_embed_notes,
            embeddings::constellation_embedding_status,
            map::constellation_map_data,
            map::constellation_map_universe,
            canvas::read_canvas,
            canvas::write_canvas,
            canvas::create_canvas,
            lenses::list_lenses,
            lenses::save_lenses,
            lenses::apply_lens,
            libraries::export_note_html,
            libraries::move_to_trash,
            libraries::delete_path,
            write_gate::read_write_journal_stats,
            libraries::scan_library_index,
            libraries::read_index_entries,
            libraries::read_term_mentions,
            libraries::read_cooccurring_terms,
            lexicon::lexicon_expand_for_filter,
            // MIG-013 §1D — CTSE Bridge Adapter (query-time concept
            // expansion). The earlier index-time backfill / first-fill
            // pipeline was retired in favour of the dominant industry
            // pattern (Lucene SynonymGraphFilter, SQLite FTS5 Method 2,
            // CLIR query-translation, Primo controlled-vocabulary
            // expansion): expand at query time, not at index time. One
            // Tauri command serves the IndexPanel filter — given a
            // user query, return the subset of `term_vocab` terms
            // that resolve to the same M11 concepts as the query
            // (the multilingual `≈ similar` row in the dropdown).
            ctse::search::ctse_search_terms_by_concept,
            libraries::read_index_history,
            libraries::write_index_history_entry,
            libraries::clear_index_history,
            watcher::watch_library,
            watcher::unwatch_library,
            bases::update_note_property,
            bases::create_base,
            // MIG-065 — convert an old MVP `.base` (BaseDefinition JSON) to the
            // new LensDefinition YAML (in place, on the user's explicit choice).
            bases::convert_base,
            bases::list_workspace_bases,
            bases::create_workspace_base,
            bases::save_workspace_base,
            bases::delete_workspace_base,
            // MIG-055 §C — Constellation Base (lens) execute command.
            // Clean rebuild post-MIG-054 revert. MIG-065: curated dimensions
            // + raw frontmatter (`prop.*`) columns via json_extract; strict
            // YAML schema, federation-aware scoping. Path `lens::query::execute_lens`
            // (not the re-exported `lens::execute_lens`) because Tauri's
            // `generate_handler!` macro resolves the `__cmd__` shim at
            // the function's definition site, not through re-exports.
            lens::query::execute_lens,
            // MIG-065 §E — frontmatter-key discovery for the Base add-column
            // picker's "Your fields" tier (federated json_each over note_meta).
            lens::query::discover_base_properties,
            // MIG-065 §G — persist the add/remove-column gesture: rewrite a
            // `.base` file's `columns:` (round-trip through LensDefinition).
            lens::query::update_base_columns,
            // MIG-065 §G.2 — persist the click-header / multi-sort gesture:
            // rewrite a `.base` file's `order:` (round-trip through LensDefinition).
            lens::query::update_base_order,
            // MIG-055 §F — Five Acts sidebar enumerator. Lists `.md` files
            // in `{universe}/Five Acts/`. Returns FiveActsNoteEntry tuples
            // with display name + relative path + absolute path.
            lens::system_notes::list_five_acts_notes,
            // MIG-056 §H — Cross-universe federation warning surface.
            // Returns the current FederationContext.warnings list (per
            // the skip_unavailable model). Consumed by the frontend
            // status-bar badge + popup.
            federation::federation_get_warnings,
            universe::list_universes,
            universe::create_universe,
            universe::set_active_universe,
            universe::get_active_universe_path,
            universe::remove_universe_from_registry,
            universe::check_migration_needed,
            universe::add_child_universe,
            universe::remove_child_universe,
            universe::resolve_universe_libraries,
            universe::read_universe_settings,
            universe::save_universe_settings,
            universe::read_universe_bookmarks,
            universe::save_universe_bookmarks,
            universe::read_universe_workspaces,
            universe::save_universe_workspaces,
            universe::read_universe_property_types,
            universe::save_universe_property_types,
            link_types::read_universe_link_types,
            link_types::save_universe_link_types,
            link_types::list_link_types,
            style_presets::load_style_presets,
            style_presets::save_style_presets,
            style_presets::export_style_preset,
            style_presets::import_style_preset,
            universe::migrate_legacy_data,
            universe::open_existing_universe,
            universe::get_child_universes,
            universe::read_child_universe_libraries,
            universe::link_library_as_universe,
            universe::scaffold_starter_library,
            universe::get_templates_dir,
            universe::list_templates,
            universe::rename_universe,
            arabic::overrides::read_arabic_overrides,
            arabic::overrides::add_arabic_override,
            arabic::overrides::remove_arabic_override,
            arabic::overrides::reindex_arabic_overrides,
            libraries::get_file_metadata,
            libraries::notes_by_tag,
            dataview::execute_dataview_query,
            tasks::scan_library_tasks,
            tasks::scan_note_tasks,
            tasks::toggle_task,
            tasks::scan_library_note_dates,
            importers::import_pick_source,
            importers::import_preview,
            importers::import_execute,
            importers::import_with_canonical,
            file_kinds::classify_file_cmd,
            canonical::generate_canonical_name,
            canonical::canonicalize_preview,
            canonical::canonicalize_execute,
            canonical::auto_canonicalize_all,
            canonical::inject_cid_library,
            canonical::de_canonicalize_library,
            canonical::repair_external_libraries_on_startup,
            canonical::ensure_cid_cn_cmd,
            libraries::set_library_canonical_mode,
            perf_trace::get_perf_trace_log,
            perf_trace::clear_perf_trace_log,
            constellation_show_in_folder,
            open_path,
            list_monitors,
            open_second_screen,
            open_second_screen_on_monitor,
            close_second_screen,
            is_second_screen_open
            ]);
            move |invoke: tauri::ipc::Invoke<tauri::Wry>| -> bool {
                perf_trace::record(invoke.message.command());
                inner(invoke)
            }
        })
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                if window.label() == "second-screen" {
                    // Intercept close on second-screen: hide instead of destroy
                    api.prevent_close();
                    let _ = window.hide();
                    // Notify frontend that screen was closed
                    let _ = window.emit("screen-hidden", ());
                } else if window.label() == "main" {
                    // Main window closing: also close the second screen
                    if let Some(second) = window.app_handle().get_webview_window("second-screen") {
                        let _ = second.destroy();
                    }
                }
            }
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
