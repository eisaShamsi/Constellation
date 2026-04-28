# MIG-003 — Human-name Filenames Overlay

**Owner**: Eisa ALSHAMSI
**Migration lead**: Claude
**Opened (architect phase)**: 2026-04-28
**Status**: Phase 1 — Architect (awaiting decision on approach)

---

## The user-visible problem

In Windows File Explorer (and any external file manager / sync client / git diff / backup tool), users see canonical filenames like `20260426T140737Z_NOTE_E561.md` instead of human-readable names like `Apple Tree Fruit.md`. Inside Constellation panels everything is fixed (MIG-008 did that); outside Constellation, the canonical id is still the only name available.

## Why canonical filenames exist (the design rationale)

The canonical-filename architecture (orientation v1.6 §6, `docs/CANONICAL-FILENAME-ARCHITECTURE.md`) was deliberate:

- **Immutable identifier on disk**: `cid_cn` (= filename stem) never changes, even when the user renames the title. Wikilinks always resolve to the same file regardless of title edits.
- **Sync-conflict resistance**: if two devices rename the same note differently, canonical filenames keep the same file across both — no duplicate-file collision.
- **Git diff stability**: a title rename produces a `.md` content diff, not a file-rename diff. Easier review.
- **Title collision tolerance**: two notes can have the same title (`Untitled`, `Apple`) with no filename collision.

The cost paid for these benefits: external tools see opaque ids.

## Five approaches considered

### Approach A — De-canonicalize: rename files back to title-based names

Use the existing `de_canonicalize_library` (canonical.rs:901, shipped) to rename files: `20260426T140737Z_NOTE_E561.md` → `Apple Tree Fruit.md`. `cid_cn` stays in frontmatter; aliases preserved; `.meta.json` sidecars deleted.

| Aspect | Detail |
|---|---|
| Pros | Implementation already exists. File Explorer shows human names immediately. Wikilinks still resolve via title. cid_cn preserved for living-link traversal weights. |
| Cons | Title changes require file renames on disk (slow on 7,600-note libraries). Title collisions need disambiguation (append " 1", " 2", or warn user). Special filesystem chars in titles need sanitization. Sync conflicts on simultaneous renames produce duplicate files. Git logs see file-renames not content-diffs on every title change. |
| Reversibility | `inject_cid_library` already exists — can reverse, so this is opt-in per library. |
| Effort | Already shipped. User triggers it manually. |

### Approach B — Symlink overlay

Keep canonical files as source of truth. Auto-generate symlinks (Windows: NTFS junctions / `mklink`; macOS: aliases; Linux: symlinks) that point at canonical files but display with human names.

| Aspect | Detail |
|---|---|
| Pros | Canonical immutability preserved. File Explorer shows human names. |
| Cons | **Cross-platform implementation pain**: Windows requires admin rights or Developer Mode for symlinks; cross-platform symlink semantics differ. Backup tools may copy both link AND target = duplicate data. Git typically commits symlinks as text files with paths = confusing diffs. **Most sync clients (Syncthing, OneDrive, Dropbox) handle symlinks unevenly** — some follow them, some treat them as files. Title change requires symlink delete + recreate. |
| Reversibility | Delete symlinks, canonical files remain. |
| Effort | Substantial implementation; OS-specific code paths; risk of stranded symlinks. |

### Approach C — Frontmatter-driven rename on save

Every time a note is saved with a changed `title:` field, the file is renamed to match. Continuous: no "two modes" — canonical vs human. Files always reflect current title.

| Aspect | Detail |
|---|---|
| Pros | Same File Explorer benefit as A. Filenames track titles automatically. |
| Cons | Same Cons as A, **plus**: every title edit becomes a file rename. File watchers (OneDrive, Dropbox) race against the rename. Multi-window Constellation (second screen) must coordinate. **Wikilink cascading on rename is BUG-013 + BUG-015 territory** — we already have a fragile cascade walker (MIG-006 §3 was reverted). Adding automatic-on-every-save renames multiplies that fragility. |
| Reversibility | Same as A. |
| Effort | Substantial. Risky. |

### Approach D — Windows Shell extension (or platform-specific overlay)

A Windows shell extension intercepts `.md` file display in Explorer and shows the human title from frontmatter instead of the canonical filename.

| Aspect | Detail |
|---|---|
| Pros | Filename on disk stays canonical (best for sync, git, backup). User sees human names in Explorer. |
| Cons | **Cross-platform incompatible**: Windows shell extensions are Win-only. Finder has different mechanisms; Linux file managers vary. Constellation is local-first multi-platform — a Win-only fix is a non-starter for the architecture. Implementation requires registry mods, Administrator install, separate distribution. |
| Reversibility | Uninstall extension. |
| Effort | Massive. Out of scope for cross-platform desktop app. **Reject.** |

### Approach E — Per-library "filename mode" setting (recommended)

Each library has a `canonical_mode` field (already partially implemented per orientation v1.6 §6 — "library modes (native/canonical/compatible)"). User picks per-library:

| Mode | Filename style | When to use |
|---|---|---|
| **canonical** (current default for Universe-creation flow) | `YYYYMMDDTHHMMSSZ_KIND_XXXX.ext` | Sync-heavy libraries, git-tracked archives, libraries with many title collisions or frequent renames |
| **human** (new — what MIG-003 enables) | Title-derived (`Apple Tree Fruit.md`) | Libraries the user manages by hand outside Constellation, public-shareable libraries, libraries where File Explorer readability matters |
| **compatible** (existing) | Whatever's already there (Obsidian vault import) | Imported / shared vaults that should keep their original layout |

Implementation:
1. Each library's mode flag is read at write time. NEW notes in a `human` library are created with title-derived filenames; rename = file rename. NEW notes in a `canonical` library use the existing `YYYYMMDDTHHMMSSZ_KIND_XXXX.ext` flow.
2. **Mode change** for a library triggers a one-shot conversion (`canonical → human` runs `de_canonicalize_library`; `human → canonical` runs `inject_cid_library` + rename pass). Reversible.
3. UI: Settings → Libraries → per-library mode picker.

| Aspect | Detail |
|---|---|
| Pros | User chooses trade-off per library. Existing `de_canonicalize_library` / `inject_cid_library` do the heavy lifting. New libraries can default to `human` if user prefers. Sync-critical libraries stay `canonical`. |
| Cons | Splits user mental model: "this library uses canonical, this one doesn't." Need clear UX to communicate the trade-off. Title collision in `human` mode needs disambiguation (`Apple.md`, `Apple 1.md`). |
| Reversibility | Built in: switch mode, conversion runs. |
| Effort | **Moderate**. Most plumbing exists. New work: (a) make `human` mode actually create title-derived filenames in note-create flow (currently the only paths are canonical or compatible); (b) Settings UI; (c) title collision handling in `create_note` and `rename_item`; (d) filesystem-char sanitization in human-mode title→filename. |

---

## Recommendation

**Approach E (per-library mode)** is the architecturally honest answer. It respects:

- The user's design rationale for canonical (sync-resistance, git-stability) — keeps it available.
- The user's pain (File Explorer readability) — solves it for libraries that want it.
- The existing implementation (`de_canonicalize_library`, `inject_cid_library`, `set_library_canonical_mode`, library `canonical_mode` field) — most of the foundation is already there.
- Cross-platform consistency — no platform-specific code paths.

Approaches A and C work but force a global decision (every library either canonical or human). E lets the user keep canonical for the libraries that benefit (research archives, git-tracked vaults) and switch human for the libraries that don't.

Approach B (symlinks) is a fragile maybe; D (shell extension) is platform-incompatible.

## What Phase 2 Plan would cover (if E is approved)

**Step 0** — Inventory the existing `canonical_mode` field. Confirm every read/write path that branches on it. Identify gaps (places that ignore the field and always go canonical, places that ignore and always go human, etc.).

**Step 1** — `human` mode write path. New `create_note` flow when library is human-mode: filename = sanitize(title), append `.md`. Collision: append ` 1`, ` 2`. Special-char sanitize: replace `/`, `\`, `:`, `*`, `?`, `"`, `<`, `>`, `|`, leading/trailing whitespace.

**Step 2** — `human` mode rename path. Title change = file rename. Cascade walker (MIG-006 §2) already rewrites wikilinks; needs to also fire here. cid_cn stays.

**Step 3** — Mode-change command. `set_library_canonical_mode(library_path, new_mode)`. If switching `canonical → human`, run `de_canonicalize_library`. If `human → canonical`, run `inject_cid_library` + rename pass to canonical. With user-confirm dialog ("This will rename N files. Continue?").

**Step 4** — Settings UI. Settings → Libraries → per-library row → mode dropdown (canonical / human / compatible). Triggers Step 3.

**Step 5** — Default mode for new universes. Add to first-run setup: "Default filename mode for new libraries: canonical / human." Saved as a Universe-level setting, used as default for new libraries.

**Step 6** — Documentation. Update `docs/CANONICAL-FILENAME-ARCHITECTURE.md`, help topic, all 14 translated User Manuals, orientation §6.

**Step 7** — Phase 4 audit (3 parallel agents per the migration skill).

Total estimate: substantial. Each Step ships testable. Most steps require Tauri rebuild + user verification.

## Risks worth naming explicitly

1. **Title-rename cascade is BUG-013 / BUG-015 territory.** We already have a fragile cascade walker. Adding human-mode title→file-rename multiplies the surfaces. Plan needs explicit BUG-015-style audit at every step.
2. **Sync conflict scenarios** (two devices rename same note differently): canonical mode is conflict-free; human mode produces two files with different names. The user should know which they're choosing.
3. **Filesystem char sanitization** is unicode-aware on modern systems but Windows still has reserved names (`CON`, `PRN`, `NUL`, `COM1-9`, `LPT1-9`). Must filter.
4. **Title collision** (two notes both titled "Untitled" or "Apple") needs deterministic disambiguation. The cleanest scheme: append space + integer, lowest unused.
5. **Length limits**: Windows MAX_PATH (260 chars) + filename length (255 chars). Long titles need truncation.

## Decision requested

If you approve **Approach E**, I'll write Phase 2 Plan with detailed per-step verification clauses, then cascade through build per Plan-Approval-Equals-Build-Approval.

If you prefer a different approach, or want a hybrid, tell me — I'll re-architect.

If you want a simpler scoped variant — e.g., **just expose `de_canonicalize_library` as a Settings → Libraries action button** without building the full per-library-mode system — that's a 1-day fix that gives you the File Explorer readability for libraries you choose, without the infrastructure investment. Effectively a "manual one-shot" version of E.

---

## Owner decisions (2026-04-28)

- ✅ **Approach E** approved with the architectural refinement that **all libraries use human filenames on disk** (no per-library "canonical mode" for filenames). The "compatible" mode for imported Obsidian vaults stays.
- ✅ **Internal-keying strictness: γ (Strict)**. `cid_cn` becomes the PRIMARY KEY of `note_meta`. Every cross-table reference uses `cid_cn` as the foreign key. Path is demoted to a non-key display column; title and filename are the human display layer; `cid_cn` is the only stable internal identifier.

The architectural shift in plain terms: *the cid_cn lives in frontmatter and the database; the filename on disk equals the title; renaming the title renames the file too; the cid_cn never moves.*

---

# Phase 2 — Plan

The migration is large (schema + filesystem + code + doc + 14 i18n manuals) but each step is verifiable in isolation. Per the staged-tests SO, user-testable steps pause for verification before the next step ships.

## Sequencing principle

**Add the new key before retiring the old one.** Each step keeps both keys (path + cid_cn) live until the next step finishes. This avoids any moment where the schema is inconsistent with the running code. Final step (Step 8) drops the redundancy.

## Step-by-step

### Step 0 — Helpers (no user-facing change)

- `note_display_filename(title) → String` — sanitizes a title for filesystem use: strip / replace `/` `\` `:` `*` `?` `"` `<` `>` `|` and Windows reserved names (`CON`, `PRN`, `NUL`, `COM1-9`, `LPT1-9`); trim leading/trailing whitespace; truncate to 240 chars (leaves room for ` 1`, ` 2` disambiguation suffix + `.md` extension within Windows MAX_PATH 260 limit).
- `resolve_filename_collision(dir, base_name, ext) → String` — returns `base_name.ext` if free, else `base_name 1.ext`, `base_name 2.ext`, …, with cap at ` 999`.
- `ensure_cid_cn_in_frontmatter(content, path) → String` — wraps existing `ensure_cid_cn` from canonical.rs; injects cid_cn for any frontmatter that lacks it.

**Verification**: cargo unit tests for each helper (sanitization edge cases, collision suffix sequence, idempotent cid_cn injection).

### Step 1 — Schema migration: add `cid_cn` to `note_meta` (no UI change)

- Bump `schema_versions.note_meta` to next value.
- ALTER `note_meta` ADD COLUMN `cid_cn TEXT NOT NULL DEFAULT ''` (transient — populated below).
- One-shot migration on app start when `schema_versions.note_meta < new`: walk every note, read frontmatter, populate `cid_cn`. For notes that lack cid_cn, inject via `ensure_cid_cn` (file write).
- After backfill: ALTER `note_meta` ADD UNIQUE INDEX on `cid_cn`.
- Preserve existing `path` PK for now.

**Verification**: SQL inspection — every row has `cid_cn` populated; no duplicates.
**Stage 1 user test**: launch the app on a real Universe; confirm boot completes; pick a known canonical note in DevTools (or via SQL via Python) and confirm its cid_cn matches frontmatter.

### Step 2 — Add `cid_cn` columns to dependent tables (no UI change)

- `note_links`: add `source_cid_cn TEXT`, `target_cid_cn TEXT` (target_cid_cn nullable for unresolved/broken wikilinks).
- `sky_nodes`: add `cid_cn TEXT UNIQUE`.
- `note_aliases`: add `cid_cn TEXT NOT NULL`.
- `note_embeddings`: add `cid_cn TEXT UNIQUE`.
- Migration: backfill all four via JOIN on existing `path` columns.
- Triggers updated to maintain `cid_cn` columns alongside path columns.

**Verification**: every row in dependent tables has cid_cn populated; SQL JOINs on cid_cn return same row counts as JOINs on path.
**Stage 2 user test**: app boots; Sky View renders unchanged (uses sky_nodes); Backlinks panel renders unchanged.

### Step 3 — Switch internal joins to `cid_cn`-keyed (no user-visible change)

- Wikilink resolution: title → look up cid_cn via `note_meta` → use cid_cn for downstream joins. (Title-side resolution unchanged — wikilinks still target titles.)
- Boot snapshot (`cache_boot_snapshot_*`): JOINs use cid_cn.
- Triggers (`note_meta_sky_*`, `note_links_sky_*`): use cid_cn.
- Read-side scanners (map.rs, strata.rs, maturity.rs, etc.): emit cid_cn as the stable id; UI consumers can use it for client-side joins / dedupe / consistency.

**Verification**: side-by-side; old (path-keyed) and new (cid_cn-keyed) JOINs return identical results across the trial Universe.
**Stage 3 user test**: Sky View, Backlinks, Outgoing Links, search, Constellation Map — all render correctly. No behavior change visible to user.

### Step 4 — Filesystem migration: canonical → human names (BIG user-visible change)

- Walk every library. For every `.md` whose filename is canonical (`YYYYMMDDTHHMMSSZ_KIND_XXXX.md` per the `is_canonical_filename` regex):
  - Read frontmatter title (already required to exist by Step 1's backfill).
  - Compute target filename via `note_display_filename(title)`, resolve collision via `resolve_filename_collision`.
  - `fs::rename` from canonical → human path.
  - Update `note_meta.path` for this row (cid_cn stays the same).
  - Cascade: `note_links.source_path`, `sky_nodes.path`, `note_aliases.path`, `note_embeddings.path` all update via triggers (which now know to update path-keyed columns when `note_meta.path` changes).
  - Append OLD canonical filename stem to frontmatter `aliases` (so that any external system referencing the file by its old canonical name finds it).
- Non-`.md` canonical files (IMG, AUD, VID, ATT) keep their canonical filenames. (User clarification needed — see Open Questions; my reading is "all files" but media/attachments may not have a derivable human name without a frontmatter title.)
- Update `.meta.json` sidecars for media files if/when they're migrated.

**Verification**: every `.md` file in every library has a human filename; `note_meta.path` matches; cid_cn unchanged; aliases preserve old canonical stem.
**Stage 4 user test**: open File Explorer on the Universe path. All `.md` files show human names. Sky View / Backlinks / search still render correctly. Wikilinks still resolve.

### Step 5 — Update `create_note` and `rename_item` flows (UI behavior change)

- `create_note`: new note created as `Untitled.md` (with collision resolution → `Untitled 1.md`, etc.). Frontmatter generated with cid_cn from new id. No more canonical filename in note creation.
- `rename_item`: title change → file rename. Compute new filename from new title; resolve collision; `fs::rename`; update `note_meta.path`; cascade to dependent tables; append old title to `aliases`; cascade walker (existing MIG-006 §2 logic) rewrites `[[OldTitle]]` → `[[NewTitle]]` in source notes' bodies.
- Existing canonical-named-file detection in `rename_item` (the special path that updates frontmatter title without renaming the file) is removed — no canonical files exist after Step 4.

**Verification**: create a new note, see `Untitled.md` in File Explorer; rename it, see filename change.
**Stage 5 user test**: create + rename round-trip. Wikilinks to the renamed note still resolve via aliases.

### Step 6 — Promote `cid_cn` to PRIMARY KEY (γ — schema-final-form)

- Drop the `path` UNIQUE index on `note_meta`. Make `cid_cn` PRIMARY KEY.
- For dependent tables, drop redundant path columns where `cid_cn` is sufficient. Keep `note_meta.path` (still load-bearing for filesystem ops). Drop `note_links.source_path`, `note_links.target_path` if they exist (use cid_cn lookups). Drop `note_aliases.path` (use cid_cn).
- Update `note_meta_sky_*` triggers to use cid_cn as the primary correlation key.

**Verification**: SQL schema inspection. Re-run all read paths from Step 3 to confirm no regression.
**No user-testable change**.

### Step 7 — Doc rewrite (`CANONICAL-FILENAME-ARCHITECTURE.md`)

Sections to rewrite:
- §1 Overview, §4 Generator, §5 Frontmatter Contract (cid → cid_cn + filename clarification), §6.3 Rename Safety, §6.4 Title Collision, §8 New Note Creation Flow, §9 Existing Library Migration, §11 Design Principles.

Companion updates:
- `docs/User Manual.md` chapter on canonical naming (if any) — update.
- 14 translated User Manuals — same.
- Help topics under `docs/help.uConstellation.World/` that reference canonical naming — update.

**No user-testable change** (documentation only).

### Step 8 — Phase 4 audit (3 parallel agents per migration skill)

- **4A Invariant Check**: cid_cn never duplicated; every cross-table reference resolves; wikilink resolution chain unbroken; all 7 cognitive operators still function; living-link traversal weights preserved through the migration.
- **4B Drift Check**: no remaining canonical filenames on disk; no remaining `path.file_stem()` in any user-visible label path (MIG-008 already cleaned this); no orphan path-keyed columns in tables that should be cid_cn-keyed.
- **4C Migration Path Check**: cold-boot from pre-γ schema runs migration cleanly; mid-migration crash leaves recoverable state (write to `.constellation/migrations/` with resumable cursor); rollback path exists for first few releases.

### Step 9 — PCS + orientation v1.7

- Single commit lands all the above as `§NN — MIG-003 closed: human-name filenames + cid_cn primary key`.
- Orientation v1.7.md written alongside v1.6 per SO #6 versions-stack rule.
- Memory entries for any new principles surfaced during build.

---

## Risk register

| Risk | Severity | Mitigation |
|---|---|---|
| Mid-migration crash (Step 4 — file rename interrupted) | High | Write resumable cursor to `.constellation/migrations/mig-003-cursor.json`. On boot, resume from cursor. Each note rename is atomic (rename + cascade) within a single transaction-like wrapper. |
| Schema migration fails on Step 1 (out-of-disk during backfill) | High | Pre-flight check: free space ≥ Universe size × 1.2. Migration aborts cleanly if insufficient, with no partial schema change. |
| Filename collision on Step 4 (50 notes titled "Untitled") | Medium | `resolve_filename_collision` handles up to 999 sequential suffixes. If exceeded, abort and ask user to disambiguate. |
| Filesystem-char sanitization edge case (Arabic / RTL filenames) | Medium | `note_display_filename` preserves Unicode but strips reserved ASCII chars only. Test with Arabic, Hebrew, mixed-script titles. |
| Sync-client conflict during migration (Syncthing renames) | Medium | Recommend user pause sync clients before launching. Pre-flight warning dialog. |
| BUG-013 / BUG-015-style cascade fragility on Step 5 rename flow | High | Phase-4-style architectural review of the new rename path BEFORE Step 5 ships. Spawn parallel agents. |
| Existing `de_canonicalize_library` / `inject_cid_library` commands assume the OLD architecture | Medium | Audit and update these commands in Step 5, or deprecate them entirely. |
| Non-`.md` canonical files (IMG, AUD, VID) | Open question | See "Open Questions" below. |

---

## Open questions for the owner before Build

1. **Media files (IMG, AUD, VID, ATT, CANVAS, DRAW)**: do canonical-named media files also migrate to human names? They lack frontmatter `title:` — only `.meta.json` sidecars contain a title. Two paths:
   - **(i)** Migrate them: read `.meta.json`, derive filename from title, rename file + sidecar together. Same disambiguation logic.
   - **(ii)** Keep them canonical: media files remain `20260315T120000Z_IMG_E5F6.png` because there's no clean human title source. cid_cn-on-disk is acceptable for non-text artifacts.
   - **(iii)** User decides per-library or per-import.
2. **Existing libraries with mixed canonical + human filenames** (created before any canonical-mode flag was respected): do non-canonical files get cid_cn injected during Step 1, leaving them with their existing human filename? My reading: yes — they already meet the new architecture's invariant (cid_cn in frontmatter + human filename). No file rename needed.
3. **First-run for a fresh Universe after MIG-003 ships**: does the user still see a "Universe filename mode" preference anywhere? My reading: no — the mode is now uniform across the app. No preference UI needed. Setting goes away.
4. **Backwards compatibility**: a user who downgrades to a pre-MIG-003 binary on a post-migration Universe — what happens? Pre-MIG-003 code expects `note_meta.cid_cn` not to exist; SQL queries against the schema fail. Recommendation: treat MIG-003 as a one-way migration; document that downgrade is not supported. Backup the Universe before launching.

---

## Owner answers to Open Questions (2026-04-28)

1. **Media files**: NOT migrated automatically. Canonical-named media keep their canonical filename unless the user has given them a human name (via UI / `.meta.json` title). The migration in Step 4 acts only on `.md` files.
2. **Existing canonical files**: every canonical `.md` file is restored to its **original human name**. Restoration order: `original_filename` (frontmatter) wins if present (the file came from a human-named source and was canonicalized at import); else the filename is derived from `title:` (the file was created inside Constellation with a canonical name). cid_cn stays in frontmatter; old canonical stem appended to aliases for forwards-compat with anything still pointing at it.
3. **First-run UI**: no "filename mode" preference. Uniform across the app.
4. **Backwards compatibility**: one-way migration. Document that downgrade is not supported. Recommend backup before launch.

## Owner direction on i18n (2026-04-28)

Step 7 must update **all 15 languages**, not just English and Arabic: `ar / de / en / es / fa / fr / he / hi / ja / ko / pt / ru / tr / ur / zh`. Help topics that mention canonical filenames also get translated updates.

Additionally, Step 0's `note_display_filename` helper must preserve non-ASCII characters across all 15 scripts (Latin, Arabic, Hebrew, Devanagari, CJK, Cyrillic, etc.). Unit tests cover each script explicitly. Only ASCII filesystem-reserved chars (`/ \ : * ? " < > |`) and Windows reserved names (`CON`, `PRN`, etc.) get stripped/escaped.

## APPROVED 2026-04-28 — cascading Phase 3 Build.

Stages 1–5 pause for user verification per the staged-tests Standing Order.
