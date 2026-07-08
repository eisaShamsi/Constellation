# G4 Phase 4 — Harden the Rust Frontmatter Index Reader (Architect)

**Status:** Architect complete → awaiting Plan approval
**Opened:** 2026-07-08 · G4 (frontmatter round-trip) Phase 4 · Owner: Claude · Boss: Eisa
**Analysis:** workflow `wf_6bdcdc87-55c` (9 agents): Rust reader census · JS writer output forms (real yaml@2.9.0) · divergence/impact · WA#5 research · design · adversarial verify (**all 4 claims refuted → 3 corrections**).

---

## Concept (the horse)

> **The Rust reader must decode a frontmatter value to the SAME string the JS writer wrote — so search, `name_lower`, wikilinks, aliases, and `cid_cn` stay accurate with disk — and must NEVER drop a note from the index on a YAML form it doesn't recognize.**

## 1. The gap (pre-existing, sharpened by G4)

The Rust index reader is a tolerant hand-rolled line-scanner. Every one of its 7 value-decode sites does `trim() + trim_matches(quotes) + fold` — it **strips outer quotes but never UNESCAPES, and reads a block scalar's value as the literal `|`/`>`.** This mis-decodes the proper YAML forms the G4 JS writer now emits on edits (single-quoted with `''`, double-quoted with `\`-escapes, block scalars, flow) — but it also mis-decodes forms that already exist in **Obsidian-imported notes today**.

**CATASTROPHIC pre-existing case:** a note with `title: |` (block scalar, multi-line title) indexes as **`name = "|"` → `name_lower = "|"`**. Every such note collides on the same key, is **unfindable by title**, every `[[Title]]` wikilink to it **breaks**, and the MIG-099 collision check mis-fires. Same for a `|`/`>` on `cid_cn` or any property.

**Impact by field** (all fed by the naive decode): `name_lower` (→ wikilink resolution, MIG-099 collision, search-by-title), `cid_cn` (→ identity; and TWO cid_cn decoders at `search.rs:4451` vs `2392` currently DISAGREE on quoted cids), `aliases` (→ alias links), `properties_json` (→ property search), `tags_json`.

**CRITICAL invariant confirmed & must be preserved:** no reader site errors-out or drops a note today — `parse_frontmatter` always returns its tuple; `index_note` only Errs on fs-read/SQL, never on parse; a mis-decode yields a **WRONG row, never a MISSING row**. The hardening must keep this — an un-decodable scalar degrades to best-effort (today's trimmed literal is the safe floor), **never** an Err/skip that omits the note.

## 2. WA#5 research — the only safe shape

- **(a) Hand-rolled shared `decode_yaml_scalar` threaded into the existing line-scanner ✅** — inherits line-independence: an un-parseable value degrades to best-effort, never drops the note. ~120–180 LOC, zero deps, provable byte-for-byte against the JS writer.
- **(b) Tolerant untyped lib (saphyr/yaml-rust2 `load_from_str`) ❌** and **(c) typed serde ❌** — both are **ALL-OR-NOTHING**: one malformed line errors the whole parse → zero fields → the note **vanishes** from the index/collision/wikilinks. That is strictly worse than a mis-decoded key — the exact app-killer the hand-rolled scanner exists to avoid. (Per-field library calls buy little over (a) and add an archived dep.)

**YAML 1.2 escaping to encode:** SINGLE-quoted has ONE escape (`''`→`'`; backslash & `"` literal; no `\n`). DOUBLE-quoted has C/JSON escapes (`\" \\ \/ \0 \a \b \t \n \v \f \r \e \N \_ \L \P`, `\xNN`, `\uNNNN`, `\UNNNNNNNN`, + backslash-newline line-continuation). BLOCK `|`(literal)/`>`(folded) + chomp `clip`/`strip(-)`/`keep(+)` require stateful indented-body consumption.

## 3. Adversarial corrections (all 4 claims refuted → build requirements)

**C1 — JS FOLDS long values → decode them single-line by DISABLING folding on the JS side. [key simplifier]** eemeli `yaml.stringify` wraps a value past `lineWidth` (default 80) across continuation lines. A per-line Rust decoder can't reconstruct a folded quoted value. → **Fix on the JS writer: emit with `lineWidth: 0` (folding OFF)** in `serializeLine`/the CST stringify so every scalar the writer produces stays on ONE line. This makes the Rust single/double-quoted decode single-line (simple + robust) and is a strict improvement (no gratuitous wrapping in the user's file). *(Block scalars are still inherently multi-line — C2 handles them.)*

**C2 — one stateful block-scalar consumer, shared across the whole scan. [identity-critical]** A `|`/`>` header on ANY key means the following more-indented lines are its BODY, not keys. If a block scalar's body contains a `cid_cn:` line, the current scanner mis-reads it as the note's cid → **identity corruption**. → **Implement the block-header detection + indented-body skip ONCE**, used by every frontmatter scan (parse_frontmatter, has_title/has_alias), so body lines are consumed, never re-scanned as bogus keys.

**C3 — `has_alias` must fold with `normalize_alias_for_match` (fold_match_key + Arabic tashkeel strip), not bare `fold_match_key`. [Arabic-critical]** The alias index (`note_aliases.alias_lower`) is `normalize_alias_for_match`; the federated `has_alias` walk uses bare ASCII/`to_lowercase`. For an Arabic alias `مُحَمَّد` (tashkeel), the own-library index resolves `[[محمد]]` (strips tashkeel) but the federated walk fails → divergence. → `has_title`/`has_alias` must decode with the shared decoder AND fold with the SAME fold as the index (fold_match_key for titles, normalize_alias_for_match for aliases).

## 4. Design — Option A (recommended)

A single pure `pub(crate) fn decode_yaml_scalar(raw: &str) -> String` (never Result, never panics; best-effort fallback = today's trimmed literal), dispatched on the value's first non-space byte: PLAIN (trim, no unescape) · SINGLE (strip + `''`→`'`) · DOUBLE (non-greedy close + full escape table) · EMPTY/`~`/`null`→empty · flow `[…]`→decode-each→JSON array, `{…}`→best-effort JSON object. Block scalars handled by the shared C2 consumer. Threaded into all sites (one decoder, one fold policy). Combined with the JS-side `lineWidth: 0` (C1).

**Unify these sites onto the one decoder + correct fold:** `search.rs` generic arm (4451, name/cid/props), tags arms (4429/4441), `extract_frontmatter_cid_cn` (2392, the 2nd cid reader — must agree), `normalize_alias_for_match` (4896, alias_lower), `libraries.rs` `has_title` (2109) + `has_alias` (2133/2138/2144, + block-guard the list-item arm), `sources/mod.rs` `push_source` (281).

## 5. Phased plan (each step = one commit + verification clause; Reproduce-First)

- **§P4-0 — Repro harness (RED).** Shared fixture `src-tauri/tests/fixtures/g4_scalar_forms.json` = {yaml_input, expected_decoded} GENERATED by the real eemeli JS writer (single/double/block/flow/empty/Arabic/greedy-quote). Rust `decode_matches_js_writer` (current path asserts RED on the new forms) + `malformed_frontmatter_still_indexes` (tab-indent/unterminated/anchor/dup-key → note_meta row still produced, GREEN — the floor invariant).
- **§P4-1 — Pure decoder: single/double/plain/empty.** `decode_yaml_scalar` + isolation unit tests; a proptest asserts it's TOTAL (never panics) over random bytes; the ~7,647 plain-ASCII names decode byte-identically (zero-regression). **Plus: set the JS writer to `lineWidth: 0` (C1)** + a JS test that no emitted value wraps. Not wired into sites yet.
- **§P4-2 — Block scalars + flow (C2).** The shared stateful block-header + indented-body consumer (advance past body lines; a body `key:` line is NOT mis-indexed); flow `[…]`/`{…}` → JSON. `malformed_frontmatter_still_indexes` STILL green.
- **§P4-3 — Unify the WRITE-PATH sites** (name_lower / cid_cn / properties_json / tags): 4451 + 4429/4441 + route 2392 through the same decoder (the two cid readers can no longer disagree). Verify: `name_lower == fold_match_key(decode(...))` == the JS value across all on-disk forms; a large-universe reindex (7,600+) shows ZERO change to already-correct rows (pure widening) + boot time unchanged.
- **§P4-4 — Unify ALIAS + FEDERATED-WALK** (C3): `normalize_alias_for_match` through the decoder; `has_title`/`has_alias` compare `fold(decode(value))` vs `fold(target)` with the SAME fold as the index, + block-guard the has_alias list-item arm. Verify: a federated linked-probe-pair (single/double-quoted title) resolves via BOTH the own-library seek AND the federated walk; the has_alias over-match regression test green.
- **§P4-5 — Adjacent site + full gate.** `sources/mod.rs push_source` through the decoder. Full G4 round-trip (JS writes → Rust decodes → identical for every form); diff-scoped safety-inspection; whole-app reindex no-regression; boot/typing latency unchanged. Then PCS (session log + Orientation v-bump + help/manual).

## 6. Decisions for Eisa (defaults in **bold**)

1. **Block-scalar `title`/`cid` (multi-line value):** **collapse to a single line (join with space) for `name`/`name_lower`/`cid`**, preserve verbatim in `properties_json`. *(Titles are single-line in practice.)*
2. **Flow-list property shape:** `authors: [Ada, Alan]` currently stored as the raw string `"[Ada, Alan]"` in `properties_json`; the fix stores a JSON array `["Ada","Alan"]` (matches the JS side). **Ship the array shape** — but it's a change that *could* affect an existing Base/Dataview query matching the raw string. Confirm.
3. **Space-significant aliases** (`aliases: [' a b ']`): **accept the trim** (fold normalizes anyway; no known user relies on padded aliases). Or preserve exactly?
4. **`has_alias` over-match fix** (list-item arm matches ANY `- ` line, e.g. a tags item — a pre-existing bug): **fix it here** (WA#6). It changes federated resolution for a note that today resolves via a non-alias `- ` line (rare). Or split out?
5. **`sources/mod.rs` hardening** (same defect class, but feeds Sight — a currently-disabled Wing): **include for completeness** (cheap, same decoder) or defer until Sight re-enables?

## 7. Invariants (audit checklist)
never drop a note (no error-out from_str) · `name_lower`/`cid_cn`/`aliases`/`properties_json` decode == JS writer output · the two `cid_cn` readers agree · `has_title`/`has_alias` fold == the index fold (fold_match_key for title, normalize_alias_for_match for alias) · block-scalar body lines never mis-indexed as keys · pure widening (already-correct plain rows unchanged) · malformed frontmatter still indexes · boot/typing latency unchanged on 7,600 notes · JS writer emits no folded values (`lineWidth: 0`).
