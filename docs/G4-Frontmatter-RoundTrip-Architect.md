# G4 — Round-Trip-Safe Frontmatter Parser (Architect)

**Status:** Architect complete → awaiting Plan approval
**Opened:** 2026-07-08 · Safety Audit remediation group **G4** · Owner: Claude · Boss: Eisa
**Analysis:** workflow `wf_e553a7ff-b0d` (11 agents): JS + Rust census · invariants + Reproduce-First recipe · WA#5 YAML-library research · design synthesis · adversarial refutation (**all 5 design claims refuted → 5 hardening requirements**).

---

## Concept (the horse)

> **A note's YAML frontmatter must survive a read → edit → write round-trip intact — only the fields the user actually changed may change. Nothing silently dropped, nothing corrupted, ever.**

This is the **content-integrity class** (LL-014, Solve-the-Class). The fix is the structural end-state — a **single frontmatter authority** that every reader and every writer goes through — not another hand-rolled patch (LL-014's three-strike law on this class is already spent: BUG-012/015/019/023).

---

## 1. Confirmed app-killers (Reproduce-First — both fire on the running app, no error surfaced)

The hand-rolled `parseFrontmatter` (`store.ts:1073`) / `reconstructFrontmatter` (`store.ts:1244`) pair is the write authority for **every** save (via `noteModel.openModel`/`compose` → `buildFullContent`). It is a line-scanner with **no YAML parser**, so:

**Recipe A — SILENT KEY LOSS (nested map + block scalar).** A note (Obsidian/Git-authored) with:
```
source:
  author: Ibn Khaldun
  year: 1377
description: |
  First line.
  Second line stays too.
```
Open it, type one char in the body (or add a tag), let it autosave. → `source:` becomes an **empty key** (both children dropped — the indented lines fail the `!startsWith(' ')` guard and no sub-loop catches them), and `description:` becomes the literal string **`"|"`** (the block body gone). **APP-KILLER** — permanent, unrecoverable knowledge loss.

**Recipe B — ACCUMULATING QUOTE CORRUPTION.** Add a property `quote` = `He said: "hi"`. First save writes valid `quote: "He said: \"hi\""`. Second save: `parseFrontmatter` does `slice(1,-1)` with **no unescape**, so the value keeps literal backslashes, and the next write escapes them **again** → backslashes **double every save**, unbounded. **APP-KILLER** — monotonic corruption of any quoted value with an embedded quote.

Other confirmed loss cases (same mechanism): folded scalars `>`, chomping `|-`/`>-`, block arrays of maps, flow arrays with quoted commas, YAML comments, key order, anchors/aliases.

## 2. Census (who touches frontmatter)

- **JS write authority:** `noteModel.ts` `openModel`(84)/`adoptDisk`(208) → `parseFrontmatter`; `compose`(166) → `buildFullContent` → `reconstructFrontmatter`. **~10 call sites** reserialize the whole block (every autosave, PropertyEditor edit, add-tag open+closed, add-link open+closed, second-screen, ExpressionForge). `rawYaml` is captured but used ONLY as a read-only `<pre>` in NotePane — the write path can never recover dropped lines.
- **Rust:** `search.rs parse_frontmatter` (line-scanner, READ-ONLY for the index → `properties_json`/aliases/tags/`cid_cn`/`title`/`kind`); `create_note` builds `fm_lines` by hand; `update_frontmatter_title`/`set_frontmatter_parent` do split-and-rebuild single-field edits. `extract_aliases`/`has_title`/`has_alias` each re-scan.

## 3. WA#5 research — the battle-tested answer

- **JS: eemeli `yaml`** (the `yaml` npm package) — the JS analog of Python's ruamel.yaml, **the** library built for "edit one field, keep the rest." Document API: `parseDocument(raw)` → `setIn/deleteIn` only the changed field → `String(doc)`. Preserves comments, key order, nested maps, block scalars, and (serializer-owned quoting) round-trips embedded quotes correctly. ~21–24 KB gz, zero deps, **save-path only** (Rule 6 untouched). js-yaml / gray-matter LOSE comments (same defect class, quieter). **Obsidian itself does NOT preserve rich frontmatter** (`processFrontMatter` strips comments/formatting — a low bar; we beat it).
- **Rust:** do NOT route the index reader through **typed** `serde_yaml` — it errors on any malformed/partial user YAML and would **drop the whole note from the index** (a worse app-killer). Keep a tolerant reader. (`serde_yaml` is archived/RustSec-flagged; its only use is the developer `.base` format — migrate that to `serde-saphyr` as a *separate* PJ.)

## 4. Adversarial hardening requirements (all 5 naïve-design claims were REFUTED — each is now a build requirement)

**H1 — Malformed YAML must NOT lose content. [CRITICAL]** `Document.toString()` *throws* "Document with errors cannot be stringified" when `doc.errors` is non-empty (e.g. an unterminated flow `tags: [a, b`). → After `parseDocument`, **check `doc.errors`**; on errors, **do NOT stringify** — preserve the original raw frontmatter bytes verbatim and apply the user's single field change via a minimal targeted edit (or, if even that is unsafe, keep the raw block and surface a non-destructive notice). Never throw away frontmatter or body on a parse error.

**H2 — Rust must decode what JS emits (JS↔Rust consistency). [CRITICAL]** Once JS is a real YAML serializer it emits forms the hand-rolled Rust scanner can't decode — **single-quoted** scalars (`title: 'He said: "go"'`), block scalars, flow maps. A finite Rust "hardening list" cannot close this. → The Rust index reader must become a **tolerant REAL YAML reader** (untyped/error-tolerant parse that decodes single/double-quoted + block-scalar + flow forms, with best-effort line-scan fallback on malformed input — never a typed `from_str` that drops the note). `create_note`'s WRITE must emit via the same canonical contract so Rust-written === JS-read. Otherwise `title` decodes differently → wrong `name_lower` → wrong index/collision/links.

**H3 — Byte-fidelity of UNTOUCHED keys (Git/Syncthing churn). [Boss decision]** The Document API re-emits the whole block in canonical style — it preserves unchanged *values* but **re-indents** a 4-space nested map to 2, may move a `# comment`, restyle quotes. For a File-Over-App + Git-sync app that means every save churns bytes the user never edited. The **CST API** is byte-perfect for everything untouched (token-range edit of only the changed field) but is more complex to drive. → **Boss decision (open Q1):** ship Document-level (kills the app-killers; untouched *values* safe, formatting may normalize) vs invest in CST (byte-perfect untouched bytes, smallest Git diff). *Recommendation: CST for the untouched-key path* given the explicit File-Over-App/Git-sync stance — but it is the larger build.

**H4 — The save contract is whole-array-replace, not a diff.** The model's `setProps` replaces the entire `FrontmatterProperty[]`, and that array is a *lossy* projection (a block scalar can't be represented as a flat property). Mapping "props changed" back onto the retained Document is the concentrated risk. → The **retained YAML Document/CST becomes the sole write authority**; the `FrontmatterProperty[]` is a *display/edit projection* only, and compose applies the **real field diff** (added/removed/changed keys) to the Document — never a full rebuild from the lossy array. Built and proven in isolation (Phase 1) before it touches the live path.

**H5 — Canonical fields + Arabic/RTL through BOTH sides.** `create_note` writes `title: "The \"Big\" Idea"` by raw string-building; Rust readers `.trim()` rather than YAML-decode. The class fix must include the Rust side (H2), and the shared fixtures must assert canonical fields (`cid_cn`/`title`/`kind`/`created`) + Arabic/RTL values are byte-identical after a JS-write → Rust-read → JS-read cycle.

## 5. Design options

| | Approach | Speed | Effort | Risk |
|---|---|---|---|---|
| **A ✅** | **Single frontmatter authority**: JS retains the parsed `yaml` Document/CST as the write authority (props = display projection, compose applies the field diff), + hardened tolerant **real-YAML Rust reader** + malformed-input contract | Medium | Medium-High | **Low-Med** — the library is the field standard; risk is concentrated in the props-diff→Document mapping (H4) + the JS↔Rust contract (H2), both proven in isolation |
| B | Targeted hand-rolled fixes (unescape-on-read + passthrough unmodeled) | Fast for B1, open-ended for the rest | Deceptive | **HIGH — rejected.** Re-implements a YAML library by hand; every construct is a fresh seam; spends LL-014's already-exhausted three-strike law on this class again |
| C | Hybrid — library only as passthrough-preserver, keep hand-rolled for modeled keys | Medium | Medium | **Med-High** — two serializers for one document = the §CB "half-migration" shape LL forbids |

**Recommended: A** — the only option that satisfies Solve-the-Class (one authority, built+proven in isolation, landed as one validated swap behind a flag), hardened by H1–H5.

## 6. Phased plan (each step = one commit + a verification clause; Reproduce-First + Editor-Surface Gate)

- **Phase 0 — Repro harness + shared fixtures (no behavior change).** Recipe A + Recipe B as golden round-trip tests (assert on-screen === disk after save; assert untouched-key byte-stability); matching Rust fixture. **Verify: all new tests FAIL RED** (the baseline every later phase turns green).
- **Phase 1 — Build `yamlDoc` module dark, behind `useYamlDoc` flag (off).** `npm i yaml`; `src/lib/editor/yamlDoc.ts`: `parseFrontmatterDoc(raw)→{doc, props}`, `applyPropDiff(doc, oldProps, newProps)` (field-diff, H4), `stringify(doc)` (with the **H1 malformed contract**). **Verify: Phase-0 fixtures GREEN against `yamlDoc` in isolation** (build the finished correct thing, prove it before it touches the live path). Diff-scoped safety-inspection.
- **Phase 2 — The ONE validated swap.** Wire `noteModel.openModel/adoptDisk/compose` to the retained Document; flip `useYamlDoc` on. **Verify: Recipe A + B RED→GREEN on the RUNNING app** (not static checks — Reproduce-First); full **Editor-Surface Gate** (all 8) with on-screen===disk after each; diff-scoped safety-inspection, every finding fixed before commit.
- **Phase 3 — Route remaining write sites through `yamlDoc`; delete `parseFrontmatter`/`reconstructFrontmatter`/`quoteIfNeeded`; remove the flag.** **Verify:** per-call-site round-trip tests (add-tag/add-link to a closed note) GREEN; the read-only `rawYaml <pre>` still renders.
- **Phase 4 — Harden the Rust tolerant reader to agree with JS (H2).** Tolerant real-YAML decode (single/double-quoted, block scalars, flow) with line-scan fallback; unify `extract_aliases`/`has_title`/`has_alias`/`extract_sources` on one `split_frontmatter`; scope `has_alias` to the `aliases:` block; symmetric unescape in `create_note`/canonical. **Verify:** shared fixtures → identical canonical fields + non-phantom `properties_json` vs JS-written disk; a malformed note is still indexed (NOT dropped); Rust suite + diff-scoped inspection.
- **Phase 5 — Cycle close + PCS.** Whole-app per-cycle safety-inspection; Orientation v-bump (SO#6, same commit); **help + User Manual ×15** ("Constellation now preserves your Obsidian/Git rich frontmatter"); session log; MoCh. **Verify:** inspection register appended, zero open app-killers; both recipes green.

## 7. Boss decisions (defaults in **bold**)

1. **Byte-fidelity tier (H3):** **CST — byte-perfect untouched keys** (smallest Git diff, honors File-Over-App) vs Document-level (simpler, untouched values safe but formatting may normalize). *Larger build for CST; recommended given the Git-sync stance.*
2. **Minimal-diff vs canonicalize-on-save:** **strictly touch only changed fields** (smallest diff) vs tidy/canonicalize the whole block. **Minimal-diff.**
3. **Existing silent date-normalization** (DD/MM/YYYY→ISO applied on READ to untouched fields — itself a File-Over-App violation): **drop from the untouched path; normalize only a date field the user actually edits.**
4. **Nested-map EDITING in PropertyEditor:** **preserve + display read-only now**, add nested-edit UI later.
5. **Rust nested-map indexing:** **opaque now** (never corrupt it), structured/searchable later.
6. **Already-corrupted existing files (Recipe B backslashes):** **stop-the-bleeding now**; a one-off opt-in detect-and-heal scan as a separate tool.
7. **`serde_yaml` (archived) → `serde-saphyr` for the `.base` dev format:** **separate PJ** (not user-file path).

## 8. Invariants (audit checklist)
no silent key drop · no quote/escape corruption (symmetric, serializer-owned) · canonical fields verbatim · untouched keys preserved (values always; bytes per H3) · comments/order preserved · JS↔Rust decode agreement (H2) · malformed-YAML never loses content (H1) · Obsidian round-trip ≥ parity · PropertyEditor typed values · RTL/Arabic exact · Editor-Surface Gate (all 8) · index (`name_lower`/aliases/`properties_json`) matches disk.
