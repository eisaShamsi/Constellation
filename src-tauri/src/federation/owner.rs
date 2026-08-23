//! MIG-111 Phase 1.1 (R2) — **which universe owns this path?**
//!
//! The Router's first question, and the one every routed operation is built on: an operation on a
//! note whose home is universe U must do its bookkeeping in U's database, not in whichever
//! universe happens to be active. Everything downstream (the routed context pool, killing Class D,
//! the transfer engine) needs one honest answer to this, so it lives alone and is tested alone.
//!
//! ## Three properties, each of them a defect the Architect's adversarial pass found
//!
//! **1. Roots come from the FEDERATION TREE, never from library lists.** `universe.json`'s
//! `children`, recursively. A library list cannot answer this: `universe_notes` is registered with
//! `path == the Universe root`, so a library-derived resolver reports the parent for every path in
//! a nested child — the exact confusion that made the first PJ-235 guard admit what it existed to
//! refuse.
//!
//! **2. LONGEST match wins** (attack H3). MIG-108 puts linked universes UNDER the active root, so
//! `E:/U/Linked/note.md` is under BOTH `E:/U` and `E:/U/Linked`. Shortest- or first-match hands
//! the note to the parent and writes a child's row into the parent's database. The deepest root
//! that contains the path is its owner, and that is the whole of the nesting story.
//!
//! **3. Unknown is an ERROR, never the active universe** (attack H2). A parent-walk that falls
//! back to "must be ours, then" turns an unlinked universe sitting on disk — a folder the user
//! dragged in, a stale path, a sibling universe never linked — into a write target. The candidate
//! set is exactly `{active} ∪ {federation descendants}`, and a path under none of them is `Err`.
//! Fail-closed: the Router refuses rather than guessing, because guessing writes into a corpus
//! nobody authorised.
//!
//! ## On normalisation — the §0.4 lesson, applied before it can bite again
//!
//! Comparison uses the same form `libraries::path_is_under_any` uses: backslashes to forward
//! slashes, trailing slash trimmed, lowercased. In §0.4 a canonicalized path (`\\?\E:\…` on
//! Windows) was compared against raw registry roots, and the guard could never match: a dead no-op
//! that read green for a whole round because its test checked where the call sat rather than
//! whether it fired.
//!
//! **This module shipped with that same defect, and the per-build inspection found it.** The first
//! version of this comment asserted that "the roots here come from `universe.json` and the
//! active-universe pointer, which are raw" — and it was **false on the federation side**:
//! `resolve_child_universe_roots_recursive` builds its list with `fs::canonicalize`, so every child
//! root arrived verbatim while the active root arrived raw. A nested linked universe — the DEFAULT
//! shape under MIG-108 — could therefore never match, and every note inside one resolved to the
//! ACTIVE PARENT with `is_active: true`: attack H3, defeated in the pure function and reintroduced
//! by the wrapper. A linked *sibling* failed the other way, becoming permanently unroutable. All
//! nine unit tests stayed green, because every one of them drove the pure function with hand-built
//! RAW paths and nothing exercised the form the app actually supplies.
//!
//! The fix is not "remember to pass raw paths" — that is the promise a caller must keep, and this
//! is the second time in one migration that promise was broken. `norm` now **strips the verbatim
//! prefix**, so the comparison is total over path forms and no caller can get it wrong; and the
//! returned `Owner::root` is stripped too, so one universe has exactly ONE identity for whatever
//! keys a connection pool or a lock downstream. Two identities would mean two locks, which is no
//! lock at all.

use std::path::{Path, PathBuf};

/// Who owns a path, and whether that is the universe currently active.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Owner {
    /// The owning universe's root, in the **stripped** form — no `\\?\` prefix, whichever form the
    /// candidate arrived in.
    ///
    /// **Deriving a lock or pool key from this: go through `universe_lock::canon`, never through
    /// string equality.** That function deliberately keeps the verbatim form, because it also
    /// BUILDS the lock file's path and the verbatim form is what lets a deep universe exceed
    /// Windows' 260-character limit. The two notions reconcile through `canon` — canonicalizing the
    /// stripped form and canonicalizing the verbatim form give the same answer — but only if
    /// everything that needs identity calls it, rather than comparing these strings directly. Two
    /// keys for one universe would mean two locks, which is no lock at all.
    pub root: PathBuf,
    /// True when the owner IS the active universe — the fast path, where today's code already
    /// does the right thing and the Router adds nothing.
    pub is_active: bool,
}

/// Drop Windows' verbatim prefix, which `fs::canonicalize` puts on everything it returns.
///
/// `\\?\E:\U` and `E:\U` are the same directory and no amount of slash-folding makes them equal, so
/// this runs BEFORE any comparison and on anything this module hands back. `\\?\UNC\server\share`
/// is the network form of the same thing and folds back to `\\server\share`.
pub(crate) fn strip_verbatim(p: &Path) -> PathBuf {
    let s = p.to_string_lossy();
    if let Some(rest) = s.strip_prefix(r"\\?\UNC\") {
        return PathBuf::from(format!(r"\\{rest}"));
    }
    match s.strip_prefix(r"\\?\") {
        Some(rest) => PathBuf::from(rest),
        None => p.to_path_buf(),
    }
}

/// Normalise for comparison. Matches `libraries::path_is_under_any` exactly, deliberately: two
/// notions of "the same place" is how a boundary check and the thing it guards come apart — plus
/// the verbatim strip, because one side of this comparison is canonicalized and the other is not.
fn norm(p: &str) -> String {
    strip_verbatim(Path::new(p))
        .to_string_lossy()
        .replace('\\', "/")
        .trim_end_matches('/')
        .to_lowercase()
}

/// Is `path` inside `root`? Equal counts — a universe root is inside itself.
fn under(path_n: &str, root_n: &str) -> bool {
    path_n == root_n || path_n.starts_with(&format!("{}/", root_n))
}

/// The decision itself, free of `AppHandle` so the tests drive THIS function rather than a
/// re-implementation that would keep passing after this one changed. (The `require_own_library_in`
/// pattern — the PJ-235 panel found the first version's tests exercised only the primitives, so
/// deleting the real check left the suite green.)
pub fn resolve_owner_in(path: &str, active: &Path, federation: &[PathBuf]) -> Result<Owner, String> {
    let path_n = norm(path);

    let mut best: Option<(usize, PathBuf, bool)> = None;
    let mut consider = |root: &Path, is_active: bool| {
        let root_n = norm(&root.to_string_lossy());
        if root_n.is_empty() || !under(&path_n, &root_n) {
            return;
        }
        // LONGEST match wins — the deepest root containing the path is the owner. The root is
        // stored STRIPPED so a universe has one identity regardless of which side it arrived from.
        if best.as_ref().is_none_or(|(len, _, _)| root_n.len() > *len) {
            best = Some((root_n.len(), strip_verbatim(root), is_active));
        }
    };

    consider(active, true);
    for root in federation {
        consider(root, false);
    }

    match best {
        Some((_, root, is_active)) => Ok(Owner { root, is_active }),
        // Fail-CLOSED. Not "assume the active universe" — that is how an unlinked folder on disk
        // becomes a write target.
        None => Err(format!(
            "That path is not inside this universe or any universe linked to it, so Constellation \
             cannot tell which one owns it — and will not guess. Nothing was changed. ({path})"
        )),
    }
}

/// The app-level entry point: enumerate the federation tree, then decide.
///
/// Roots are read fresh rather than from `load_all_libraries`' cache, because that cache can hold
/// a DEGRADED resolve for a whole session when a child registry is briefly unreadable (PJ-300) —
/// and an owner resolver that silently loses a universe would route that universe's writes into
/// the active database, which is precisely the class this migration exists to end.
pub fn resolve_owner(app: &tauri::AppHandle, path: &str) -> Result<Owner, String> {
    let active = crate::universe::active_universe_dir(app)
        .map_err(|e| format!("Cannot resolve the active universe ({e}). Nothing was changed."))?;
    // MIG-111 §1.2/A4 — the STRICT enumeration. The lenient one answers "no children" for an
    // unreadable manifest, and under MIG-108 nesting that turns a child's note into a
    // confident `is_active: true` for the PARENT. Refusing is the only safe answer to "I
    // cannot tell what is linked."
    let federation = crate::universe::resolve_child_universe_roots_recursive_strict(&active)?;
    resolve_owner_in(path, &active, &federation)
}

#[cfg(test)]
mod tests_mig111_11_resolve_owner {
    use super::*;

    fn p(s: &str) -> PathBuf {
        PathBuf::from(s)
    }

    /// The ordinary case: a note in the active universe belongs to it.
    #[test]
    fn a_note_in_the_active_universe_is_ours() {
        let o = resolve_owner_in(r"E:\U\Notes\a.md", &p(r"E:\U"), &[]).unwrap();
        assert_eq!(o.root, p(r"E:\U"));
        assert!(o.is_active);
    }

    /// **Attack H3 — the nesting story.** MIG-108 puts linked universes UNDER the active root, so
    /// the path is inside BOTH. Shortest- or first-match hands the note to the parent and writes a
    /// child's row into the parent's database; the DEEPEST containing root is the owner.
    #[test]
    fn a_nested_linked_universe_owns_its_own_notes() {
        let fed = vec![p(r"E:\U\Linked")];
        let o = resolve_owner_in(r"E:\U\Linked\Their Lib\x.md", &p(r"E:\U"), &fed).unwrap();
        assert_eq!(o.root, p(r"E:\U\Linked"), "the deepest containing root wins");
        assert!(!o.is_active);
    }

    /// Depth is what decides, not the order roots happen to arrive in.
    #[test]
    fn the_deepest_root_wins_regardless_of_order() {
        let deep = p(r"E:\U\Linked\Deeper");
        let shallow = p(r"E:\U\Linked");
        for fed in [vec![deep.clone(), shallow.clone()], vec![shallow.clone(), deep.clone()]] {
            let o = resolve_owner_in(r"E:\U\Linked\Deeper\n.md", &p(r"E:\U"), &fed).unwrap();
            assert_eq!(o.root, deep, "order must not change the answer");
        }
    }

    /// **Attack H2 — unknown is an ERROR.** A universe sitting on disk that nobody linked is not
    /// ours to write to, and a parent-walk that falls back to the active universe would make it
    /// a write target.
    #[test]
    fn an_unlinked_universe_on_disk_is_refused() {
        let err = resolve_owner_in(r"E:\Somewhere Else\note.md", &p(r"E:\U"), &[]).unwrap_err();
        assert!(err.contains("will not guess"), "the refusal must say it refuses to guess: {err}");
    }

    /// Linking the sibling is what makes it resolvable — the candidate set is exactly
    /// {active} ∪ {federation}, so the SAME path flips from refused to owned.
    #[test]
    fn the_same_path_resolves_once_that_universe_is_linked() {
        let path = r"E:\Sibling\note.md";
        assert!(resolve_owner_in(path, &p(r"E:\U"), &[]).is_err());
        let o = resolve_owner_in(path, &p(r"E:\U"), &[p(r"E:\Sibling")]).unwrap();
        assert_eq!(o.root, p(r"E:\Sibling"));
        assert!(!o.is_active);
    }

    /// A universe root is inside itself — `<root>/.constellation/...` and the root path itself
    /// both resolve, which the transfer engine and the lock will both need.
    #[test]
    fn a_root_owns_itself() {
        let o = resolve_owner_in(r"E:\U", &p(r"E:\U"), &[]).unwrap();
        assert_eq!(o.root, p(r"E:\U"));
    }

    /// Slash direction and case must not change the answer — the §0.4 defect was exactly a
    /// comparison between two path FORMS.
    #[test]
    fn slash_direction_and_case_do_not_change_the_answer() {
        let fed = vec![p("E:/U/Linked")];
        for candidate in [r"E:\U\Linked\x.md", "E:/U/Linked/x.md", r"e:\u\linked\X.MD"] {
            let o = resolve_owner_in(candidate, &p(r"E:\U"), &fed).unwrap();
            assert_eq!(o.root, p("E:/U/Linked"), "form must not decide ownership: {candidate}");
        }
    }

    /// A SIBLING directory whose name merely starts with the root's must not match — the classic
    /// prefix bug (`E:\U2` is not inside `E:\U`).
    #[test]
    fn a_sibling_with_a_shared_prefix_is_not_inside() {
        let err = resolve_owner_in(r"E:\U2\note.md", &p(r"E:\U"), &[]);
        assert!(err.is_err(), "E:/U2 is not inside E:/U");
    }

    /// And the trailing-slash form of a root is the same root.
    #[test]
    fn a_trailing_slash_on_the_root_changes_nothing() {
        let o = resolve_owner_in(r"E:\U\a.md", &p(r"E:\U\"), &[]).unwrap();
        assert!(o.is_active);
    }

    // ── The FORM the app actually supplies. ──────────────────────────────────────────────────
    //
    // Everything above hands `resolve_owner_in` hand-built RAW paths, and all of it passed while
    // the app entry point returned the INVERTED answer — because `resolve_child_universe_roots_
    // recursive` builds its list with `fs::canonicalize`, which on Windows yields the verbatim
    // `\\?\E:\…` form, while the active root comes raw from the registry. Nine green tests over a
    // wrapper nothing exercised: the same shape as the §0.4 base guard that could never fire and
    // whose test asserted where the call SAT rather than whether it FIRED.
    //
    // These tests use REAL directories and REAL `fs::canonicalize`, so the OS states the premise
    // instead of me asserting it.

    fn real_dir(tag: &str) -> PathBuf {
        let d = std::env::temp_dir().join(format!(
            "constellation_mig111_owner_{}_{}",
            tag,
            std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()
        ));
        std::fs::create_dir_all(&d).expect("tmp dir");
        d
    }

    /// **THE REGRESSION GUARD.** A nested linked universe, with its root in the exact form the
    /// production caller produces, must still own its own notes. Before the fix this returned the
    /// ACTIVE PARENT with `is_active: true` — attack H3, defeated in the pure function and
    /// reintroduced by the wrapper, which is the shape that writes a child's rows into the
    /// parent's database with every row count still looking right.
    #[test]
    fn a_canonicalized_child_root_still_owns_its_notes() {
        let base = real_dir("nested");
        let active = base.join("U");
        let child = active.join("Linked");
        std::fs::create_dir_all(child.join("Their Lib")).unwrap();
        let note = child.join("Their Lib").join("x.md");

        // EXACTLY what `resolve_child_universe_roots_recursive` puts in the list.
        let fed = vec![std::fs::canonicalize(&child).unwrap()];

        let o = resolve_owner_in(&note.to_string_lossy(), &active, &fed).unwrap();
        assert!(!o.is_active, "the child owns it, not the active parent");
        assert_eq!(
            norm(&o.root.to_string_lossy()),
            norm(&child.to_string_lossy()),
            "the owner must be the child universe"
        );
    }

    /// A linked SIBLING (not nested) must remain resolvable. Before the fix this came back as the
    /// fail-closed refusal — turning a universe the user deliberately linked into one that can
    /// never be written to.
    #[test]
    fn a_canonicalized_sibling_root_is_still_resolvable() {
        let base = real_dir("sibling");
        let active = base.join("U");
        let sibling = base.join("Sibling");
        std::fs::create_dir_all(&active).unwrap();
        std::fs::create_dir_all(&sibling).unwrap();
        let note = sibling.join("note.md");

        let fed = vec![std::fs::canonicalize(&sibling).unwrap()];
        let o = resolve_owner_in(&note.to_string_lossy(), &active, &fed)
            .expect("a linked sibling must be resolvable, not refused");
        assert!(!o.is_active);
        assert_eq!(norm(&o.root.to_string_lossy()), norm(&sibling.to_string_lossy()));
    }

    /// **ONE universe, ONE identity.** The returned root is what downstream will key a connection
    /// pool and a lock by. If the active branch returns a raw path and the federation branch
    /// returns a verbatim one, the same universe has two keys — two pool entries, two locks, and
    /// therefore no mutual exclusion at all.
    #[test]
    fn the_owner_root_has_one_form_whatever_form_it_arrived_in() {
        let base = real_dir("identity");
        let child = base.join("U").join("Linked");
        std::fs::create_dir_all(&child).unwrap();
        let note = child.join("n.md");
        let active = base.join("U");

        let raw = resolve_owner_in(&note.to_string_lossy(), &active, &[child.clone()]).unwrap();
        let canon = resolve_owner_in(
            &note.to_string_lossy(),
            &active,
            &[std::fs::canonicalize(&child).unwrap()],
        )
        .unwrap();
        assert_eq!(raw.root, canon.root, "one universe must not have two identities");
    }
}
