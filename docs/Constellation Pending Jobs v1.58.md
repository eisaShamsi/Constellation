# Constellation Pending Jobs

**Version 1.58 | 2026-07-29**

> **What changed in v1.58** (**PJ-181 CLOSED and Boss-validated — and the build's own inspection found an APP-KILLER IN THE FIX before it shipped. One new law, LL-040. Ultracode**):
>
> **► NEXT ACTION — MIG-104 Slice 8 + 8b.** The archive hook MUST go BEFORE the `DELETE FROM note_meta` at `search.rs:9845`: FK enforcement fires the CASCADE there, so a hook at the later explicit purge archives **NOTHING** (`tests_stage0_delete_order_defect`). 8b adds the note BODY (Boss decision #6). Then Slices 9–15, then **MIG-105 Phase 2**. **Owed first, though: a Boss triage of PJ-187** (5 APP-KILLERs, headline `PropertyEditor.svelte:974`).
>
> ### ✅ PJ-181 — CLOSED. A merely-VIEWED note overwriting a newer external edit.
> View a note, type nothing, close it — `NoteEditor`'s teardown still stashes a write-ahead entry, and nothing clears it for a closed note. The note is then edited OUTSIDE Constellation (Syncthing / another device / `git pull`; the watcher ignores it because the note is closed). `cid_cn` is the note's IDENTITY and never changes with an edit, so `resolveNoteContent`'s cid check passed, the stale view won the screen, the model was born DIRTY with it, and the first tab switch wrote it over the newer file. **Measured: the flush returned `{ok:true}` and the externally-added paragraph was gone from disk.**
>
> **The root:** the net entry recorded WHAT it held and never WHY. It now carries `snapshot?: boolean` — "this content was already durable when stashed" — and `resolveNoteContent` refuses a snapshot whose content no longer matches disk. A genuine recovery copy (PJ-102 Recipe S) is untouched, and so is a legacy entry with no flag: unprovable → treated as real work, the direction that never discards.
>
> ### ★★ THE INSPECTION FOUND AN APP-KILLER **IN THE FIX** — measured, before it shipped
> The flag was first hard-coded `true` in the `!needsDiskSave` branch, on the reasoning that reaching there means "nothing changed since the last durable save". **It does not.** `needsDiskSave` is NotePane's view-level `dirty`, cleared at save-REQUEST time (`NotePane.svelte:340`) and never restored on failure. So after a FAILED save any teardown re-stashed the user's ONLY copy flagged *already durable*, and the new branch rejected and **deleted** it. The fix would have destroyed exactly what the net exists to protect. Now derived from the MODEL, which owns durability.
>
> **And two of the three new tests were worthless when written** — one passed for a harness bug (a path where a save ENV was expected), one passed with the flag hard-coded. Both replaced; the surviving assertion is proven load-bearing against the old predicate.
>
> ### ⚖️ ONE NEW LAW — LL-040
> *A flag named for an INTENT is not a fact about DURABILITY — read every assignment site, and derive safety-critical predicates from the layer that owns the truth. And a test of the CONSEQUENCE cannot pin a DECISION whose consequence is ambiguous: assert the predicate at the point of decision, then prove the assertion load-bearing by replacing it with the old value.*
>
> ### NEWLY FILED
> - **PJ-188 — the write-ahead net's localStorage blob is unbounded.** Never pruned, never capped, and `setWriteAhead` swallows a quota exception with an empty `catch` — so on quota exhaustion the crash-recovery net silently stops persisting. Pre-existing; surfaced while sizing the PJ-181 fix (which is why that fix carries a boolean and not a second copy of every note's bytes).
> - **PJ-189 — net entries from the PREVIOUS build carry no `snapshot` flag**, so they keep the pre-PJ-181 behaviour until the note is next opened-and-closed under the new build. Deliberate (unflagged → treated as real work → never discards), but the fix self-heals per note rather than instantly.
>
> **Verification:** vitest **63 files / 722 tests** (was 62/716) · svelte-check **0** · Sight perf SERIAL 31/31 · Rust untouched (frontend-only). Boss-validated twice — once on the original fix, once on the corrected build.
>
> ---

**Version 1.57 | 2026-07-29**

> **What changed in v1.57** (**PJ-182 CLOSED — and it was 20 surfaces across two languages, not one function. `/simplify` then found three more defects in the fix itself. Ultracode**):
>
> ### ★ THE RE-PRIORITIZED BACKLOG
>
> **► NEXT ACTION — PJ-181** (APP-KILLER, `store.ts:2448`): the write-ahead net is restored on a `cid_cn` match with **no freshness check against the disk bytes it just read**, and a net entry is stashed for merely-VIEWED notes. View → close → the note is edited externally (Syncthing / second device / git pull; the watcher ignores it because the note is closed) → reopen shows the **stale** content with the model born dirty → the first tab switch writes it over the newer file. `restoreSessionTabs` (`store.ts:2930-2940`) already solves exactly this on the sibling path — copy that arbitration. Then **MIG-104 Slice 8 + 8b** (the archive hook must precede `DELETE FROM note_meta` at `search.rs:9845` — FK enforcement fires the CASCADE there, so a hook at the later purge archives **nothing**; `tests_stage0_delete_order_defect`). Then Slices 9–15, then **MIG-105 Phase 2**.
>
> ### ✅ PJ-182 — CLOSED. A reproduced content-loss bug that was six times larger than filed.
>
> Filed as *"a zero-indent YAML block list projects as an empty list — `store.ts:2009`"*. Verified by running, then swept ecosystem-wide per the Whole-Ecosystem Fix Law (38-agent workflow, every candidate adversarially refuted): **20 confirmed surfaces**, including **six APP-KILLERs not previously known**. Boss ruling: *fix the whole thing now*, five slices.
>
> **The single root cause.** Eight independent sites answered *"is this line a continuation of the previous key's block sequence?"* from **leading whitespace**, when YAML's rule is *"the trimmed line begins with a dash"* — a mapping key can never begin with one. `search.rs::parse_frontmatter` had carried the correct rule all along, in a comment that states it explicitly. **The rule was written down in exactly one of the nine places that needed it.**
>
> **What it cost the user.** The frontend READ the list as empty, so the panel showed nothing and the next write to that key deleted every item from the `.md` — silently, re-parsing cleanly afterwards. The Rust WRITERS were worse: renaming a note, editing a Bases cell, or resolving a contested parent spliced their own indented item in beside the user's column-0 ones and produced frontmatter that **no longer parses at all**, after which every future property edit on that note was silently discarded forever. `aliases:` was hit identically (breaking every backlink through that alias), and typed links written this way never reached `note_links` at all — the Living Link graph silently lost those edges, while the structural-link guard simultaneously failed OPEN.
>
> **Fixed:** one shared predicate per language (`isYamlSeqItem` + `yamlSeqItemValue` in `store.ts`; the new `src-tauri/src/yaml_lines.rs`), routed through **every** surface — 14 sites, not the 8 the sweep reported. Three further shapes closed in the same pass: **block scalars** (`desc: |` projected as editable text valued `"|"`, and the cache wrote `desc: "|"`), **flow sequences on the next line** (read-only, so the user could not edit their own tags), and **comments inside a block**. Plus, found in-pass and indentation-independent: **every ikhtilāf edit was a silent no-op**, because `nestedObjects` — the property's actual content — was compared by nothing and carried by no intent.
>
> **Verification:** every fix RED-proven by removing it, each with a control proving the indented form is untouched. Suite **678 → 716** (vitest 62 files) · Rust **1261 → 1277** · svelte-check **0** · Sight perf SERIAL 31/31.
>
> ### ★★ `/simplify` FOUND THREE DEFECTS IN THE FIX ITSELF — all three reproduced, all three fixed
> 1. A **block scalar was outside `immutableBlockKeys`**, so Slice 4 gave it the read-only widget and not the write-path refusal. Omitting the row from a props array **deleted the prose from the file**.
> 2. **`nestedObjects` was threaded through three layers and stopped at the fourth** — `composeFrontmatter`'s unchanged-check still decided from the display summary, so a row deletion was dropped. *A chain is exactly as strong as its last link.*
> 3. **The Rust twin had no comment concept**: widening the continuation-skip meant a `#` line among the items ended the skip and orphaned everything after it. The JS twin had always been right — **only the Rust half was wrong.** (LL-038 rule 4.)
>
> And the reuse review found **eight more sites of the same concern still hand-rolling the predicate** — including the very function the new module quotes as the rule's origin, and the direct siblings of two functions that WERE routed. **The Whole-Ecosystem Fix Law caught me inside the change made to obey it.** All now routed.
>
> ### NEWLY FILED
> - **PJ-183 — the Rust frontmatter BLOCK WALKER.** Eleven sites still own a private state machine for "consume/track the block under key K" (`skipping_list_items`, `in_alias_list`, `in_old_list`, `in_list`, `in_aliases_block`, `in_aliases`, `skip_block` ×2, `current_key`, `current_structural`, `in_tags`). The shared *predicate* stopped the bleeding; the shared *walker* is the end-state. Two of those state machines are verbatim copies of each other (`sources/mod.rs`).
> - **PJ-184 — give a block scalar its own `PropertyType`.** It currently borrows `nested-map`, smuggles its `|` indicator through `value`, puts a prose preview in `nestedKeys`, and is labelled **"Nested map" to the user in 15 languages**. Form-Aligns-To-Purpose: the row must express what the thing is. Small — the read-only rendering already exists.
> - **PJ-185 — `parseFrontmatterDoc` returns `props: []` on a parse error.** If PJ-137 ever swaps the projection, a note with broken frontmatter would show NO rows where it shows readable ones today. That is a Boss ruling, not a silent consequence.
> - **PJ-186 — the flow-sequence branch covers only a single-line `[...]`.** A multi-line flow sequence, and a flow MAPPING on the next line, still fall to read-only. Correct and inert, so not urgent.
> - **PJ-187 — the 2026-07-29 whole-app inspection register.** 50 unique confirmed sites (5 APP-KILLER · 10 HIGH · 29 MED · 7 LOW), owed a per-cycle triage with the Boss. `lab/reports/SAFETY-INSPECTION-2026-07-29-pj182.md`. Headline: **APP-KILLER `PropertyEditor.svelte:974`** — the right-sidebar Properties panel is never `{#key}`-remounted, so a pending 800 ms debounce that survives an in-place navigation (click a wikilink within 800 ms of a property edit) writes note **A**'s properties onto note **B**, durably and silently. Its NotePane twin is protected by a `{#key}`; the sidebar one is not. Also inside it: **APP-KILLER `yamlDoc.ts:311`** — `composeFrontmatter`'s malformed-YAML passthrough discards every property edit on such a note and reports the save as **successful** (`hasErrors` has zero consumers outside `yamlDoc.ts`), and **`yamlDoc.ts:362`** — the CST splice+append deletes YAML comments attached to the edited key, on the app's single most common frontmatter edit (adding a tag).
> - **PJ-166 — EIGHTH strike.** This build's inspection was invoked diff-scoped with `args.files` and returned `mode: "whole-app"` again: 88 agents, ~10.8 M tokens, ~30 minutes for a 13-file gate. It earned its cost *again* (it caught the half-routed `sources/mod.rs`), but **the per-build gate the standing order actually requires still does not exist, eight attempts in.** This is now the oldest unfixed process defect in the ledger.
>
> ### RE-RANKED
> - **PJ-137** (retire the hand-rolled `store.parseFrontmatter` for one YAML-document authority — `/migration`-sized) is now at **strike SIX**: PJ-136, MIG-101, the 2026-07-24 seq-of-maps inspection, and now PJ-182's three shapes all originate in the parser split. Recorded for whoever opens it: `projectProps` / `parseFrontmatterDoc` are **test-only today** — a spec-compliant projector already sits beside the hand-rolled one, dark, which is the cheapest possible starting point. The gaps it must close first are type detection (`date`/`datetime`/`link`/`nested-object-list` + key-driven typing), date normalisation, ikhtilāf rows, and PJ-185's error behaviour.
> - **PJ-180** gains evidence: `buildFullContent` is no longer *destructive* on the RICH fixture (PJ-182 fixed that at the source), but it still re-serializes — `'He said: "hi"'` comes back double-quoted. The `tests/g4/composeUpdated.test.ts` case that used to assert the destruction now asserts the re-quoting, because asserting the old damage would have pinned a defect in place.
> - **PJ-146** unchanged and now more visible: the Properties help topic is **English only**; the 14 translated help dirs do not carry it. The User Manual's Properties section WAS updated in all 15 languages this pass.
>
> **STILL OPEN:** **PJ-181** (► next) · **PJ-187** (the new whole-app register — 5 APP-KILLERs, owed a Boss triage) · **MIG-104** Slices 8–15 · **PJ-145 / MIG-105** · PJ-164 (= MIG-104 Slice 12) · PJ-150 · PJ-152 · PJ-158 · PJ-159 · PJ-160 · PJ-162 · PJ-163 · **PJ-166 (8th strike — the per-build inspection gate still does not exist)** · PJ-167 · PJ-168 · PJ-169 (MIG-106) · PJ-170 · PJ-171 · PJ-172 · PJ-173 (→ MIG-104 Slice 14) · **PJ-174** (the earlier sweep register) · PJ-176 · PJ-177 · **PJ-180** · **PJ-183** · **PJ-184** · **PJ-185** · **PJ-186** · **PJ-137 (strike six)** · PJ-140 (~37) · PJ-142/143/144/146/147/148 · PJ-136 · PJ-135 · PJ-125–139.
>
> ---

**Version 1.56 | 2026-07-29**

> *(See `Constellation Pending Jobs v1.56.md` — the trail is durable, never overwritten.)*
