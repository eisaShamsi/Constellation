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
/// Split an inline YAML flow sequence's INNER text (`a, b, c` from `[a, b, c]`) into items,
/// **respecting quotes**.
///
/// PJ-207 §15 — the Rust half of the codebase had eight hand-rolled `inner.split(',')` sites, so
/// a quoted item containing a comma was torn in two and its real value destroyed. The shape is
/// ordinary, not exotic: `aliases: ["Ibn Khaldūn, ʿAbd al-Raḥmān"]`, `parent: "[[Foo, Bar]]"` —
/// a comma is exactly why such a value was quoted in the first place. The TypeScript side had
/// already been fixed (`splitFlowSeqItems`, store.ts, 2026-08-01); the Rust side was never swept.
///
/// Deliberately a scanner rather than a YAML parser: one pass, no per-character allocation. It
/// strips ONE matching quote pair per item and does not decode escapes — a `\"` inside a
/// double-quoted item stops the item being split in the wrong place and rides through verbatim,
/// which is what every other quoted value here already does.
pub(crate) fn split_flow_seq_items(inner: &str) -> Vec<String> {
    let mut items: Vec<String> = Vec::new();
    let mut start = 0usize;
    let mut quote: Option<char> = None;
    let bytes: Vec<(usize, char)> = inner.char_indices().collect();
    let mut i = 0usize;
    while i < bytes.len() {
        let (idx, ch) = bytes[i];
        if let Some(q) = quote {
            if ch == '\\' && q == '"' {
                i += 2; // YAML escapes only exist inside double quotes
                continue;
            }
            if ch == q {
                quote = None;
            }
        } else if ch == '"' || ch == '\'' {
            quote = Some(ch);
        } else if ch == ',' {
            items.push(inner[start..idx].to_string());
            start = idx + ch.len_utf8();
        }
        i += 1;
    }
    items.push(inner[start..].to_string());
    items
        .into_iter()
        .map(|s| unquote_one_pair(s.trim()))
        .filter(|s| !s.is_empty())
        .collect()
}

/// Strip ONE matching pair of surrounding quotes. Mirrors the TS `unquote`.
fn unquote_one_pair(s: &str) -> String {
    let b = s.as_bytes();
    if b.len() >= 2 && ((b[0] == b'"' && b[b.len() - 1] == b'"') || (b[0] == b'\'' && b[b.len() - 1] == b'\'')) {
        return s[1..s.len() - 1].to_string();
    }
    s.to_string()
}

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
/// PJ-207 §15 — **does this line END a block value we are dropping?**
///
/// Only a new TOP-LEVEL KEY does. Everything else still belongs to the block: its sequence
/// items, an item's indented continuation, a comment sitting between items, and blank lines.
///
/// The rule this replaces was "is it a sequence item? if not, the block is over", which ended
/// the block at the first comment or continuation and PUSHED THE REMAINING ITEMS BACK into the
/// frontmatter — under a key that had already been removed. The result is a sequence with no
/// key: invalid YAML, and an unparseable note is exactly the state in which every later
/// property edit silently vanishes. Two strip loops had it (`sources:` and `content_type:`);
/// they now share this one so they cannot drift apart again.
pub(crate) fn ends_dropped_block(line: &str) -> bool {
    // PJ-207 §15 (second pass) — a COMMENT never ends the block.
    //
    // `is_top_level_key_line("# note")` is true: a column-0 comment is unindented and is not a
    // sequence item. So a comment written flush-left INSIDE a block ended it here, and every
    // remaining `- ` item was pushed back out under a key that had already been removed —
    // a sequence with no key, which is unparseable frontmatter, which is the state where every
    // later property edit silently vanishes. The first pass fixed only the INDENTED-comment half
    // of this exact shape and shipped a regression test that used `  # a note to self`; the
    // flush-left half, which is how most people write a frontmatter comment, was still broken.
    !line.trim_start().is_empty() && !is_comment(line) && is_top_level_key_line(line)
}

/// as a key literally named `- name`.
pub(crate) fn is_top_level_key_line(line: &str) -> bool {
    !is_indented(line) && !is_seq_item(line)
}

// PJ-234 / PJ-240 — `is_block_value_line` was DELETED here, 2026-08-11.
//
// It answered "is this line part of the block value?" as `is_seq_item(line) || is_indented(line)`,
// which is FALSE for a blank line. Every writer that used it to drop a replaced block therefore
// stopped at the first blank and emitted the remaining items under the new scalar — a sequence
// with no key, i.e. unparseable YAML, i.e. the state in which every later property edit on that
// note silently vanishes.
//
// PJ-207 §15 wrote the correct rule (`ends_dropped_block`: only a new TOP-LEVEL KEY ends the
// block) and swept it into `sources/mod.rs` — and left three writers on the old one, where it sat
// for weeks. The predicate is gone rather than merely unused, because a wrong answer left in the
// codebase is how this defect reached its fourth and fifth shapes: the next person to write a
// block-drop loop cannot now reach for it by mistake. `ends_dropped_block` is the only answer.

#[cfg(test)]
mod tests {
    /// PJ-207 §15 — a quoted item containing a comma is ONE item. The shape is ordinary:
    /// `aliases: ["Ibn Khaldūn, ʿAbd al-Raḥmān"]` — a comma is exactly why it was quoted.
    /// Eight Rust sites split on raw commas and tore such values in half.
    #[test]
    fn a_quoted_comma_does_not_split_the_item() {
        assert_eq!(
            super::split_flow_seq_items(r#""Ibn Khaldūn, ʿAbd al-Raḥmān""#),
            vec!["Ibn Khaldūn, ʿAbd al-Raḥmān".to_string()]
        );
        assert_eq!(
            super::split_flow_seq_items(r#"alpha, "Rosenthal, F.", beta"#),
            vec!["alpha".to_string(), "Rosenthal, F.".to_string(), "beta".to_string()]
        );
        // single quotes too
        assert_eq!(super::split_flow_seq_items("'a, b', c"), vec!["a, b".to_string(), "c".to_string()]);
        // and the ordinary case is unchanged
        assert_eq!(super::split_flow_seq_items("a, b, c"), vec!["a".to_string(), "b".to_string(), "c".to_string()]);
        // empties dropped, whitespace trimmed
        assert_eq!(super::split_flow_seq_items(" a ,, b "), vec!["a".to_string(), "b".to_string()]);
    }

    /// A wikilink whose TITLE contains a comma survives — the case that destroys a link.
    #[test]
    fn a_wikilink_with_a_comma_survives() {
        assert_eq!(
            super::split_flow_seq_items(r#""[[Foo, Bar]]""#),
            vec!["[[Foo, Bar]]".to_string()]
        );
    }

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
