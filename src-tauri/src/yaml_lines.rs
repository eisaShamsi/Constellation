//! PJ-182 — the ONE place that answers *"what kind of frontmatter line is this?"*.
//!
//! ## Why this module exists
//!
//! Constellation hand-scans note frontmatter line by line in a dozen places (the rename
//! cascade, the Bases cell writer, canonicalization, template merge, the PJ-065 parent
//! resolver, the link extractors). Every one of them has to answer the same question —
//! *is this line a continuation of the previous key's block sequence, or a new top-level
//! key?* — and **eight of them answered it from LEADING WHITESPACE**:
//!
//! ```text
//! let is_list_item = indented && t.starts_with("- ");   // <- wrong
//! let is_top_level = !line.starts_with(' ') && !line.starts_with('\t');   // <- wrong
//! ```
//!
//! That is not YAML's rule. **A block sequence may be indented at the SAME level as its
//! parent mapping key** (YAML 1.2), and hand-authored and imported vaults are full of it:
//!
//! ```text
//! tags:
//! - alpha        <- column 0. Valid. `is_indented` is false; it is still a list item.
//! - beta
//! ```
//!
//! What identifies a sequence item is the **dash**: a mapping key can never begin with one.
//! Where the eight sites got it wrong the consequences were not cosmetic — a rename spliced
//! its own indented alias in beside the user's column-0 items and produced frontmatter that
//! **no longer parses at all**, after which every later property edit on that note was
//! silently discarded forever, because the frontend's compose path passes unparseable YAML
//! through untouched and reports success.
//!
//! The nine surfaces that already handled this input correctly all used the trimmed-line
//! test — `search::parse_frontmatter` even documents it: *"a line beginning `- ` is a LIST
//! ITEM, never a key."* The rule was right there, written down, in exactly one of the places
//! that needed it. This module is that sentence, made callable, so there is one definition
//! instead of eight opinions (LL-038 rule 5).
//!
//! Its JS twin is `isYamlSeqItem` in `src/lib/libraries/store.ts`. Keep the two honest
//! about each other: the `/simplify` pass on this very change caught them already
//! disagreeing about comments, and only the Rust half was wrong.

/// A YAML block-**sequence** item: `- x`, `  - x`, `-\tx`, or a bare `-` on its own line.
///
/// **Indentation is deliberately not part of this test.** See the module docs: a block
/// sequence is allowed to sit at its parent key's indentation, and the dash is what makes
/// it a sequence item.
pub(crate) fn is_seq_item(line: &str) -> bool {
    seq_item_value(line).is_some()
}

/// The PAYLOAD of a sequence item — the text after the dash — or `None` if `line` is not
/// one. A bare `-` (an empty entry) yields `Some("")`.
///
/// This exists because a predicate alone is a trap. `is_seq_item` deliberately accepts
/// shapes the hand-rolled `starts_with("- ")` tests did not — a bare `-`, and `-\titem` —
/// and every one of those sites then slices `line[2..]` to get the value. That slice
/// **panics** on a bare `-`. Routing a site through the predicate without also routing its
/// extractor converts a silent miss into a crash; keeping the two together makes that
/// impossible rather than remembered.
pub(crate) fn seq_item_value(line: &str) -> Option<&str> {
    let t = line.trim_start().trim_end_matches(['\r', '\n']);
    let rest = t.strip_prefix('-')?;
    if rest.is_empty() {
        return Some(rest); // a bare `-` — an empty sequence entry
    }
    // `-foo` is a plain scalar, not a sequence item; the indicator must be followed by
    // whitespace (YAML's own rule).
    if rest.starts_with(' ') || rest.starts_with('\t') {
        Some(rest.trim_start())
    } else {
        None
    }
}

/// Leading whitespace, matching `trim_start()`'s Unicode notion of it.
///
/// ASCII-only (`starts_with(' ') || starts_with('\t')`) is what every site this module
/// replaced used, and keeping it here would have left the module internally inconsistent —
/// `is_seq_item` trims Unicode whitespace while `is_top_level_key_line` did not, so an
/// NBSP-indented nested key read as TOP-LEVEL. That is the 2026-07-21 app-killer shape (an
/// indented `title:` matched at root) re-opened for non-ASCII indentation, and NBSP /
/// U+3000 indented lines are first-class here — `search.rs` documents handling them.
pub(crate) fn is_indented(line: &str) -> bool {
    line.len() != line.trim_start().len()
}

/// The leading whitespace of `line`, verbatim — the indentation an item appended to that
/// line's block must reuse.
///
/// Appending at a hardcoded `"  "` into a block whose items sit at column 0 mixes two
/// indentations inside one sequence, which is invalid YAML. Three writers did exactly that.
pub(crate) fn indent_of(line: &str) -> &str {
    &line[..line.len() - line.trim_start().len()]
}

/// A YAML comment line, at any indentation.
///
/// The module needs this because its consumers are WRITERS. A comment sitting among a
/// block's items is neither a sequence item nor a key, so a writer replacing that key
/// treated it as the end of the block — and then emitted the items after it beneath the new
/// scalar, orphaned. The TS twin has always folded comments into the block; this is the
/// Rust half of the same rule.
pub(crate) fn is_comment(line: &str) -> bool {
    line.trim_start().starts_with('#')
}

/// True when `line` opens a **top-level mapping key** — unindented, and not a sequence item.
///
/// The second half is the part that was missing everywhere: `- name: X` at column 0 is
/// unindented AND contains a colon, so every site that tested only indentation accepted it
/// as a key literally named `- name`.
pub(crate) fn is_top_level_key_line(line: &str) -> bool {
    !is_indented(line) && !is_seq_item(line)
}

/// True when `line` is part of the block VALUE under the key above it — a sequence item at
/// any indentation, or an indented continuation line (a seq-of-map's `role: Y`).
///
/// Comments are deliberately NOT included: a writer replacing the key must keep the user's
/// comment while still dropping the items around it, so comments are handled separately by
/// the caller rather than swallowed here.
pub(crate) fn is_block_value_line(line: &str) -> bool {
    is_seq_item(line) || is_indented(line)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_dash_is_a_sequence_item_at_any_indentation() {
        for l in [
            "- alpha",
            "  - alpha",
            "\t- alpha",
            "    - name: X",
            "-",
            "  -",
            "-\talpha",
            "- alpha\r",
        ] {
            assert!(is_seq_item(l), "should be a seq item: {l:?}");
        }
    }

    #[test]
    fn a_key_is_never_a_sequence_item() {
        for l in [
            "tags:",
            "  title: X",
            "title: -- dashes --",
            "",
            "   ",
            "--not-a-list",
            "#- a comment",
            "--- ",
        ] {
            assert!(!is_seq_item(l), "should NOT be a seq item: {l:?}");
        }
    }

    #[test]
    fn top_level_key_excludes_column_zero_sequence_items() {
        assert!(is_top_level_key_line("tags:"));
        assert!(is_top_level_key_line("title: X"));
        // The whole point: unindented and colon-bearing, but a list item.
        assert!(!is_top_level_key_line("- name: X"));
        assert!(!is_top_level_key_line("- alpha"));
        assert!(!is_top_level_key_line("  title: X"));
        assert!(!is_top_level_key_line("\ttitle: X"));
    }

    #[test]
    fn indent_is_returned_verbatim() {
        assert_eq!(indent_of("- alpha"), "");
        assert_eq!(indent_of("  - alpha"), "  ");
        assert_eq!(indent_of("\t- alpha"), "\t");
        assert_eq!(indent_of("    title: X"), "    ");
        assert_eq!(indent_of(""), "");
    }
}
