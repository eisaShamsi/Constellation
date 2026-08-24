//! MIG-111 §1.2 — **the routed write scope.**
//!
//! ## The concept (the horse)
//!
//! *An operation on a note does its bookkeeping in the database of the universe that owns
//! that note, using that universe's link vocabulary.*
//!
//! Today every write path answers both questions ambiently: the connection comes from
//! `SearchState.db` (the ACTIVE universe's) and the vocabulary from `link_types::active_universe_vocabulary()`
//! (the ACTIVE universe's), regardless of where the note actually lives. For a note in the
//! active universe those answers are right. For a note in a linked universe they are wrong,
//! and wrong in the worst available way — every row count still comes out correct.
//!
//! A `WriteScope` is the pair of answers, resolved once, carried explicitly.
//!
//! ## Why not simply swap the global registry around the write
//!
//! Because `link_types::REGISTRY` is read at CALL time by every parser and SQL generator in
//! the process, and the app has other threads. The debounced save, the backfill and the
//! watcher all land inside a swap window, and what they produce is not detectably wrong —
//! it is a note's links classified under a vocabulary belonging to a different universe.
//! That is LL-047, and the harness pins it. A routed write carries its vocabulary; it does
//! not borrow the process's.
//!
//! ## The refusals
//!
//! A routed scope is constructed or it is refused — there is no degraded form. Four things
//! must hold before this type will hand out a connection to a universe the user is not in,
//! and each refuses BY NAME, because the user has several universes and the one in front of
//! them is not the one at fault:
//!
//! 1. **The owner must be resolvable** — through the fail-closed federation enumeration
//!    (§A4). "I cannot tell what is linked" is never answered with "then nothing is."
//! 2. **`search.db` must already exist.** `Connection::open` CREATES the file it cannot
//!    find, so an unchecked open against a mistyped or half-removed root does not fail — it
//!    silently founds an empty second index for that universe.
//! 3. **The bookkeeping triggers must be present**, probed against the ACTIVE universe's own
//!    inventory rather than a list written here. Both universes run this binary and the same
//!    `init_db`; a child missing what the parent has is a child whose derived surfaces would
//!    silently stop being maintained by our write (PJ-302 is exactly how that happened).
//! 4. **The vocabulary must be readable** — strictly, via `link_types::registry_for_root`.
//!
//! Boss ruling 2 (2026-08-17): the parent NEVER writes schema into a universe it does not
//! own. Every one of these is a refusal, never a repair.

use crate::link_types::LinkTypeRegistry;
use rusqlite::Connection;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

/// Where a scope's connection comes from.
///
/// **Deliberately holds no `AppHandle`.** A scope is a VALUE — a root, a vocabulary, and at
/// most an owned connection — so it can be constructed and asserted over without a running
/// Tauri app. The active arm takes the handle at `with_conn` time instead, which every write
/// path already has in hand.
///
/// The general rule, which this codebase had already found once at `search.rs`'s `WalkCtx`
/// and never generalized: **a type that answers a question must be constructible without a
/// running application — because a type you cannot construct in a test is a type whose
/// refusals you cannot prove.**
///
/// This was not only a taste decision. While `WriteScope` held an `AppHandle`, any unit test
/// that constructed one made the whole lib test binary **fail to start**
/// (`STATUS_ENTRYPOINT_NOT_FOUND`, 0xc0000139, before a single test ran) while the identical
/// production code linked and ran fine. Bisected 2026-08-20: a test calling `init_db` in this
/// same module was fine at 86,441,472 bytes; the same test calling `routed_at` was unloadable
/// at 86,816,256. Removing the field made the failure go away.
///
/// **MECHANISM UNCONFIRMED — do not cite this as an explanation.** "Returning one instantiated
/// the Wry runtime's type graph" is a hypothesis that was never measured. At least two others
/// fit the same eight data points: a size threshold somewhere in the 86,441,472..86,816,256
/// gap (the current passing binary sits inside it), and a stale sibling `constellation_lib.dll`
/// in `deps/` — `Cargo.toml` builds a `cdylib`, and 0xc0000139 is by definition a named export
/// missing from a DLL the loader DID resolve. `dumpbin /imports`, diffed between a passing and
/// a failing binary, is the measurement that discriminates them; `/dependents` (which is what
/// was run) lists DLL names only and cannot settle it. See the Lessons-Learned entry.
///
/// **The rule below stands regardless of how that resolves**, which is why it is stated on its
/// own terms and not on the linker's.
enum Target {
    /// The active universe — the fast path. The connection is `SearchState.db`, reached
    /// exactly as every write path reaches it today, so this arm is byte-identical to the
    /// pre-MIG-111 behaviour and adds nothing but the explicit vocabulary.
    Active,
    /// A linked universe — a dedicated connection on ITS `search.db`, opened in the same
    /// shape as `reconcile_filesystem`'s walk connection.
    Routed(Mutex<Connection>),
}

/// The resolved answer to "whose database, and whose vocabulary?" for one operation.
pub struct WriteScope {
    root: PathBuf,
    is_active: bool,
    vocabulary: LinkTypeRegistry,
    target: Target,
}

impl WriteScope {
    /// Resolve the scope for a note by its path.
    ///
    /// The only entry point production code should use: it resolves the owner itself rather
    /// than being told, so a caller cannot pass the active universe for a note that lives
    /// somewhere else.
    pub fn for_note(app: &tauri::AppHandle, note_path: &str) -> Result<Self, String> {
        let owner = super::owner::resolve_owner(app, note_path)?;
        if owner.is_active {
            return Ok(Self {
                root: owner.root,
                is_active: true,
                // The active universe's vocabulary IS the global one — but read HERE, once,
                // and carried, so every consumer downstream takes it as a parameter and an
                // eleventh ambient reader cannot appear without a compile error.
                vocabulary: crate::link_types::active_universe_vocabulary(),
                target: Target::Active,
            });
        }
        let required = expected_trigger_floor()?;
        Self::routed_at(&owner.root, required)
    }

    /// The routed construction, taking its inputs as arguments so the refusals can be tested
    /// over real directories without an `AppHandle` — and so production calls the SAME
    /// function the tests do (LL-048: a test driving a convenience wrapper has not tested
    /// the thing production calls).
    pub fn routed_at(root: &Path, required_triggers: &[String]) -> Result<Self, String> {
        let name = universe_name(root);
        let db = crate::universe::constellation_dir(root).join("search.db");

        // Refusal 2 — `Connection::open` creates what it cannot find.
        if !db.exists() {
            return Err(format!(
                "The universe \"{name}\" has no search index yet, so Constellation will not \
                 write to it (opening one would create an empty second index). Nothing was \
                 changed."
            ));
        }

        let mut conn = Connection::open(&db).map_err(|e| {
            format!(
                "Could not open the search index of the universe \"{name}\" ({e}). \
                 Nothing was changed."
            )
        })?;
        // The `reconcile_filesystem` shape. `recursive_triggers=ON` is not decoration: the
        // sky triggers fire from inside other triggers' writes, and SQLite defaults it OFF.
        conn.execute_batch(
            "PRAGMA journal_mode=WAL; PRAGMA synchronous=NORMAL; PRAGMA recursive_triggers=ON;",
        )
        .map_err(|e| {
            format!("Could not prepare the search index of \"{name}\" ({e}). Nothing was changed.")
        })?;
        conn.busy_timeout(std::time::Duration::from_secs(30))
            .map_err(|e| format!("busy_timeout on \"{name}\": {e}"))?;
        // Tokenizers are connection-local: without this the FTS triggers' INSERT fails with
        // "no such tokenizer" on this connection alone.
        crate::search::register_fts5_tokenizer(&mut conn)?;

        // Refusal 3 — the bookkeeping must be there before we rely on it.
        let present = triggers_in(&conn)?;
        let missing: Vec<&str> = required_triggers
            .iter()
            .filter(|t| !present.contains(*t))
            .map(|s| s.as_str())
            .collect();
        if !missing.is_empty() {
            return Err(format!(
                "The universe \"{name}\" is missing the bookkeeping that keeps its search and \
                 link data up to date ({}), so Constellation will not write to it. Nothing \
                 was changed.",
                missing.join(", ")
            ));
        }

        // Refusal 4 — whose vocabulary, read strictly.
        let vocabulary = crate::link_types::registry_for_root(root)?;

        Ok(Self {
            root: root.to_path_buf(),
            is_active: false,
            vocabulary,
            target: Target::Routed(Mutex::new(conn)),
        })
    }

    /// The vocabulary of the universe that owns the note — the answer every parser and SQL
    /// generator below this point must take as a parameter.
    pub fn vocabulary(&self) -> &LinkTypeRegistry {
        &self.vocabulary
    }

    /// True when the owner is the universe the user is currently in.
    pub fn is_active(&self) -> bool {
        self.is_active
    }

    /// The owning universe's root.
    pub fn root(&self) -> &Path {
        &self.root
    }

    /// Run `f` against the owning universe's connection.
    ///
    /// The active arm goes through `SearchState.db` exactly as before, including
    /// `ensure_search_db_ready`, so nothing about the ordinary case changes. `app` is ignored
    /// by the routed arm — it is taken here rather than stored so the scope stays a value
    /// (see `Target`).
    pub fn with_conn<T>(
        &self,
        app: &tauri::AppHandle,
        f: impl FnOnce(&Connection) -> Result<T, String>,
    ) -> Result<T, String> {
        match &self.target {
            Target::Active => {
                use tauri::Manager;
                crate::search::ensure_search_db_ready(app)?;
                let state = app.state::<crate::search::SearchState>();
                let guard = state.db.lock().map_err(|e| e.to_string())?;
                match guard.as_ref() {
                    Some(conn) => f(conn),
                    None => Err("The search index is not ready yet. Nothing was changed.".into()),
                }
            }
            Target::Routed(m) => {
                let guard = m.lock().map_err(|e| e.to_string())?;
                f(&guard)
            }
        }
    }

    /// The routed arm's connection, without an `AppHandle`. Returns `None` for an active
    /// scope, whose connection belongs to `SearchState` and genuinely needs the app.
    pub fn with_routed_conn<T>(
        &self,
        f: impl FnOnce(&Connection) -> Result<T, String>,
    ) -> Option<Result<T, String>> {
        match &self.target {
            Target::Active => None,
            Target::Routed(m) => Some(
                m.lock()
                    .map_err(|e| e.to_string())
                    .and_then(|g| f(&g)),
            ),
        }
    }
}

/// The trigger names present on a connection.
fn triggers_in(conn: &Connection) -> Result<std::collections::HashSet<String>, String> {
    let mut stmt = conn
        .prepare("SELECT name FROM sqlite_master WHERE type='trigger'")
        .map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map([], |r| r.get::<_, String>(0))
        .map_err(|e| e.to_string())?;
    let mut out = std::collections::HashSet::new();
    for r in rows {
        out.insert(r.map_err(|e| e.to_string())?);
    }
    Ok(out)
}

/// **The floor a linked universe must clear: what THIS BINARY'S initializer creates.**
///
/// The first version read the ACTIVE universe's live `sqlite_master` and used that as the
/// floor. The panel refuted it: `TriggerWindow::open` (index_repair.rs:428-440) deliberately
/// DROPS the outgoing, incoming and sky-aggregate triggers on the active universe for the
/// duration of a repair run — and `mig108` and `cece::history` do the equivalent for their own
/// bulk passes, several on background threads. A routed write resolving inside any of those
/// windows would read a SHORT floor and admit a linked universe missing exactly the triggers
/// the guard exists to catch. A self-calibrating check is only as good as the thing it
/// calibrates against, and that one moves.
///
/// So the floor is derived from a scratch database that `init_db` builds fresh, in memory,
/// once per process. It is by construction "what a correctly-initialised universe has,"
/// immune to whatever the active universe is doing at this instant, and it still cannot go
/// stale when a fifteenth trigger is added — the property the original design wanted.
///
/// **Known limit, stated where the code is:** this compares trigger NAMES. Trigger BODIES are
/// vocabulary-interpolated (search.rs:5563), so a universe whose triggers exist but were
/// generated under a different vocabulary passes this check. That is not an oversight here —
/// it is what the explicit vocabulary in `WriteScope` exists to handle.
pub(crate) fn expected_trigger_floor() -> Result<&'static [String], String> {
    static FLOOR: std::sync::OnceLock<Result<Vec<String>, String>> = std::sync::OnceLock::new();
    FLOOR
        .get_or_init(|| {
            let conn = crate::search::init_db(std::path::Path::new(":memory:"))
                .map_err(|e| format!("Could not determine the expected bookkeeping ({e})."))?;
            let mut names: Vec<String> = triggers_in(&conn)?.into_iter().collect();
            names.sort();
            if names.is_empty() {
                return Err(
                    "Could not determine the expected bookkeeping (the initializer produced no                      triggers). Nothing was changed."
                        .into(),
                );
            }
            Ok(names)
        })
        .as_ref()
        .map(|v| v.as_slice())
        .map_err(|e| e.clone())
}

fn universe_name(root: &Path) -> String {
    root.file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| root.display().to_string())
}



#[cfg(test)]
mod tests_mig111_12_write_scope {
    use super::*;

    fn tmp_universe(tag: &str) -> PathBuf {
        let d = std::env::temp_dir().join(format!(
            "constellation_ws_{}_{}",
            tag,
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir_all(crate::universe::constellation_dir(&d)).expect("tmp universe");
        d
    }

    /// A real owned universe, built by the PRODUCTION initializer — so the trigger inventory
    /// under test is the one the app actually creates, not one this test invented.
    fn built_universe(tag: &str) -> PathBuf {
        let root = tmp_universe(tag);
        let db = crate::universe::constellation_dir(&root).join("search.db");
        drop(crate::search::init_db(&db).expect("init_db"));
        root
    }

    /// The floor production uses. **Deliberately NOT derived from the database under test.**
    /// The first version of these tests built `required` from the same freshly-initialised DB
    /// it then probed, so `required == present` by construction and the trigger check could
    /// not fail — a test that asserted nothing. Caught by the panel.
    fn floor() -> Vec<String> {
        expected_trigger_floor().expect("floor").to_vec()
    }

    fn custom(id: &str) -> crate::link_types::LinkTypeDef {
        crate::link_types::LinkTypeDef {
            id: id.into(),
            label: id.into(),
            parent: None,
            color: "#123456".into(),
            order: 9,
            builtin: false,
            emoji: None,
            desc: None,
            structural: false,
        }
    }

    /// The whole point: a universe that has been through the real initializer satisfies the
    /// preconditions and yields ITS vocabulary, not the process's.
    #[test]
    fn a_properly_built_universe_yields_its_own_vocabulary() {
        // **This test was itself bitten by what the migration exists to remove.** The guard
        // below reads the process-global vocabulary, and `vocab_harness` mutates it via
        // `set_active`; run concurrently, the harness had "refutes" installed at the moment
        // this read it, and the assertion failed in the full suite while passing in
        // isolation. Taking the harness's own serialization lock is the interim answer — the
        // harness restores seeds-only on Drop *before* releasing it, so acquiring it
        // guarantees a known global. A5 deletes the ambient readers and this coupling with
        // them. (LL-047, and `RestoreVocabulary`'s own note that a non-harness test running
        // concurrently can still read the mutated global — this was that test.)
        let _serial = crate::federation::vocab_harness::HARNESS_LOCK
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        let root = built_universe("ok");
        let required = floor();
        crate::link_types::write_link_types_at(
            &crate::link_types::link_types_file_in(&root),
            &[custom("refutes")],
        )
        .expect("write vocabulary");

        let scope = WriteScope::routed_at(&root, &required).ok().expect("scope");
        assert!(!scope.is_active(), "a routed scope is never the active universe");
        assert!(
            scope.vocabulary().is_known("refutes"),
            "the scope carries the OWNER's vocabulary"
        );
        assert!(
            !crate::link_types::active_universe_vocabulary().is_known("refutes"),
            "guard: the process-global vocabulary does NOT know this type, so the assertion              above cannot be satisfied by the ambient answer"
        );
        // The pragma the sky triggers depend on and SQLite defaults OFF.
        let recursive: i64 = scope
            .with_routed_conn(|c| {
                c.query_row("PRAGMA recursive_triggers", [], |r| r.get(0))
                    .map_err(|e| e.to_string())
            })
            .expect("a routed scope has its own connection")
            .expect("pragma");
        assert_eq!(recursive, 1, "recursive_triggers must be ON for a routed write");
        let _ = std::fs::remove_dir_all(&root);
    }

    /// Refusal 2 — the one that would otherwise CREATE an empty second index.
    #[test]
    fn a_universe_with_no_index_is_refused_not_created() {
        let root = tmp_universe("noindex");
        let db = crate::universe::constellation_dir(&root).join("search.db");
        let err = WriteScope::routed_at(&root, &[]).err().expect("must refuse");
        assert!(
            err.contains(root.file_name().unwrap().to_str().unwrap()),
            "the refusal names the universe: {err}"
        );
        assert!(
            !db.exists(),
            "REFUSING MUST NOT CREATE THE FILE — Connection::open would have"
        );
        let _ = std::fs::remove_dir_all(&root);
    }

    /// Refusal 3 — PJ-302's shape. A universe whose bookkeeping was stripped is refused by
    /// name, and the message says WHICH bookkeeping is missing.
    #[test]
    fn a_universe_missing_its_bookkeeping_is_refused_by_name() {
        let root = built_universe("stripped");
        let required = floor();
        let db = crate::universe::constellation_dir(&root).join("search.db");
        {
            let conn = Connection::open(&db).unwrap();
            conn.execute_batch("DROP TRIGGER IF EXISTS note_meta_sky_ai;").unwrap();
        }
        let err = WriteScope::routed_at(&root, &required).err().expect("must refuse");
        assert!(
            err.contains(root.file_name().unwrap().to_str().unwrap()),
            "names the universe: {err}"
        );
        assert!(err.contains("note_meta_sky_ai"), "names what is missing: {err}");
        assert!(err.contains("Nothing was changed"), "{err}");
        let _ = std::fs::remove_dir_all(&root);
    }

    /// Refusal 4 — an unreadable vocabulary is a refusal, not the seeds. Falling back here
    /// would classify a linked universe's links under a vocabulary that is not its own,
    /// silently, with every row count still correct.
    #[test]
    fn a_universe_whose_vocabulary_cannot_be_read_is_refused() {
        let root = built_universe("badvocab");
        let required = floor();
        std::fs::write(crate::link_types::link_types_file_in(&root), b"[{\"id\":").unwrap();
        let err = WriteScope::routed_at(&root, &required).err().expect("must refuse");
        assert!(
            err.contains(root.file_name().unwrap().to_str().unwrap()),
            "names the universe: {err}"
        );
        let _ = std::fs::remove_dir_all(&root);
    }

    /// **The floor is what the initializer creates — not what the active universe happens to
    /// have at this instant.** `TriggerWindow::open` (index_repair.rs:428-440) drops the
    /// aggregate triggers on the active universe for the duration of a repair run; reading the
    /// floor from there would silently lower the bar for a linked universe at exactly the
    /// moment a background repair is running.
    #[test]
    fn the_floor_comes_from_the_initializer_and_is_not_empty() {
        let f = floor();
        assert!(!f.is_empty(), "an empty floor would admit ANY universe");
        for expected in ["note_meta_ai", "note_meta_sky_ai", "note_links_outgoing_ai"] {
            assert!(
                f.contains(&expected.to_string()),
                "the floor must include {expected}; got {f:?}"
            );
        }
    }

    /// A degraded parent must not admit a degraded child. Because the floor no longer comes
    /// from the parent at all, a child stripped of a trigger is refused even if the active
    /// universe is mid-repair and missing the same one.
    #[test]
    fn a_stripped_child_is_refused_against_the_initializers_floor() {
        let root = built_universe("degraded_pair");
        let db = crate::universe::constellation_dir(&root).join("search.db");
        {
            let conn = Connection::open(&db).unwrap();
            conn.execute_batch(
                "DROP TRIGGER IF EXISTS note_links_outgoing_ai;                  DROP TRIGGER IF EXISTS note_meta_sky_ai;",
            )
            .unwrap();
        }
        let err = WriteScope::routed_at(&root, &floor()).err().expect("must refuse");
        assert!(err.contains("note_links_outgoing_ai"), "names what is missing: {err}");
        assert!(err.contains("note_meta_sky_ai"), "names ALL of it, not just the first: {err}");
        let _ = std::fs::remove_dir_all(&root);
    }

    /// MIG-111 B1 / plan R4 — **a routed write changes NO `schema_versions` row.**
    ///
    /// `schema_versions` is where the back-fills stamp "this universe's derived surfaces are
    /// complete under vocabulary fingerprint F". The wrong-stamp class (PJ-332's shape): a write
    /// on behalf of universe A that stamps universe B's table — or stamps A's table under B's
    /// fingerprint — silently marks work done that never ran, and the next boot trusts it.
    ///
    /// The routed write path must therefore never touch the table. The full production tail a
    /// routed note travels (index + incoming + sky maintenance, the B6 composition) runs here
    /// against a routed scope, and the table is asserted byte-identical before and after —
    /// every module row, every version, in one ordered snapshot.
    ///
    /// The other half of R4 is call-graph, stated here because a test cannot execute an
    /// absence: the two stamp WRITERS — `links_backfill::finalize` (module `links_outgoing` +
    /// `links_vocab`) and the `incoming_links` / `incoming_links_vocab` stamp inside
    /// `incoming_links_backfill::run` — are both module-PRIVATE `fn`s whose only callers are
    /// their own `run(app: &AppHandle)` schedulers, which resolve the ACTIVE universe. No
    /// `WriteScope` method calls either module, so no routed write can reach a stamp. If either
    /// fn is ever made `pub` or gains a caller taking a bare `Connection`, this comment is the
    /// tripwire — re-prove R4 then.
    #[test]
    fn r4_a_routed_write_changes_no_schema_versions_row() {
        let root = built_universe("r4_stamps");
        crate::link_types::write_link_types_at(
            &crate::link_types::link_types_file_in(&root),
            &[custom("refutes")],
        )
        .unwrap();
        let target = root.join("Target.md");
        let referrer = root.join("Referrer.md");
        std::fs::write(&target, "the target's body").unwrap();
        std::fs::write(&referrer, "See [[refutes::Target]]").unwrap();

        let db = crate::universe::constellation_dir(&root).join("search.db");
        let snapshot = |label: &str| -> Vec<(String, i64)> {
            let conn = Connection::open(&db).unwrap();
            let mut st = conn
                .prepare("SELECT module, version FROM schema_versions ORDER BY module")
                .unwrap();
            let rows = st
                .query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, i64>(1)?)))
                .unwrap()
                .collect::<Result<Vec<_>, _>>()
                .unwrap();
            assert!(!rows.is_empty(), "{label}: init_db stamps rows — an empty snapshot would prove nothing");
            rows
        };
        let before = snapshot("before");

        let scope = WriteScope::routed_at(&root, &floor()).expect("routed scope");
        assert!(!scope.is_active());
        let empty: std::collections::HashSet<String> = std::collections::HashSet::new();
        scope
            .with_routed_conn(|conn| {
                for p in [&target, &referrer] {
                    let path = p.to_string_lossy();
                    crate::search::index_note_with(conn, &path, "universe_notes", true, scope.vocabulary())?;
                    crate::search::maintain_incoming_after_save(conn, scope.vocabulary(), &path, &empty, "", &empty)
                        .map_err(|e| e.to_string())?;
                    crate::search::maintain_sky_after_save(conn, scope.vocabulary(), &path, &empty, "", &empty)
                        .map_err(|e| e.to_string())?;
                }
                Ok(())
            })
            .unwrap()
            .unwrap();

        // The write itself happened — the guard that this test exercised a real tail,
        // not a no-op whose "no stamp" would be vacuous.
        {
            let conn = Connection::open(&db).unwrap();
            let links: i64 = conn
                .query_row(
                    "SELECT COUNT(*) FROM note_links WHERE link_type = 'refutes' AND status = 'active'",
                    [],
                    |r| r.get(0),
                )
                .unwrap();
            assert_eq!(links, 1, "the routed write really indexed the typed link");
        }

        let after = snapshot("after");
        assert_eq!(
            before, after,
            "R4: a routed write must not create, bump, or re-stamp ANY schema_versions row              (links_outgoing / links_vocab / incoming_links / incoming_links_vocab / sky included)"
        );
        let _ = std::fs::remove_dir_all(&root);
    }
}
