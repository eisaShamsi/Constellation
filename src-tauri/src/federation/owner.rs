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
//! slashes, trailing slash trimmed, lowercased — and **raw paths on both sides, never
//! `fs::canonicalize`**. In §0.4 a canonicalized path (`\\?\E:\…` on Windows) was compared against
//! raw registry roots, and the guard could never match: it was a dead no-op that read green for a
//! whole round because its test checked where the call sat rather than whether it fired. The roots
//! here come from `universe.json` and the active-universe pointer, which are raw; so is the path.

use std::path::{Path, PathBuf};

/// Who owns a path, and whether that is the universe currently active.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Owner {
    /// The owning universe's root, as it was given (raw, not canonicalized).
    pub root: PathBuf,
    /// True when the owner IS the active universe — the fast path, where today's code already
    /// does the right thing and the Router adds nothing.
    pub is_active: bool,
}

/// Normalise for comparison. Matches `libraries::path_is_under_any` exactly, deliberately: two
/// notions of "the same place" is how a boundary check and the thing it guards come apart.
fn norm(p: &str) -> String {
    p.replace('\\', "/").trim_end_matches('/').to_lowercase()
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
        // LONGEST match wins — the deepest root containing the path is the owner.
        if best.as_ref().is_none_or(|(len, _, _)| root_n.len() > *len) {
            best = Some((root_n.len(), root.to_path_buf(), is_active));
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
    let federation = crate::universe::resolve_child_universe_roots_recursive(&active);
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
}
