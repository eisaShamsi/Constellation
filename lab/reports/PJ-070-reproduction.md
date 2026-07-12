# PJ-070 — Reproduction record (Reproduce-First)

**Function in hand:** the **main-window file watcher's external-change reconciliation** — the code in
`src/routes/+layout.svelte` that reacts when a note's `.md` file is changed *outside* Constellation
(git-pull / Syncthing / Obsidian) while that note is **open** in a tab.

## The mechanism (read off the code, 2026-07-12)

Under Single-Ownership (MIG-076) the open editor's source of truth is the in-memory **note model**
(`src/lib/editor/noteModel.ts`), not the store's `tab.content`. The watcher flush
(`+layout.svelte` ~3218–3230) updates **only** `{ ...t, content }` in `openTabs` — it never calls
`adoptDisk` into the model and never bumps `tab.reloadVersion`. So on an external edit to an open note:

1. The file tree + search reindex (v3.32) — but the mounted editor + its model keep the **stale** body.
2. The user's next keystroke marks the stale model dirty.
3. The debounced `editor_save` composes the **stale** model and **durably overwrites** the external edit.
4. `reindex_changed_paths` then makes search agree with the stomp.

## Reproduction #1 — deterministic harness (on-demand) ✅ DONE

`tests/mig-076/runtimeHarness.test.ts` → **Recipe O** (3/3 pass, 2026-07-12):
- **RED** — skip the adopt → the keystroke save clobbers the external edit (`ADDED-BY-GIT-PULL` gone).
- **GREEN** — `externalChange` (= `adoptDisk`) adopts the fresh disk → both the external edit and the
  local keystroke survive; screen === disk.
- **DIRTY** — a mid-edit model refuses the adopt → local unsaved work is protected (last-writer-wins).

This is the "test harness" reproduction Reproduce-First permits. The residual it cannot see is purely the
**wiring** (does `+layout` actually call the adopt on the watcher path) — that is Reproduction #2.

## Reproduction #2 — running-app, write-journal instrumented (Boss Stage-1 "before")

Instrumentation already exists: every disk write is logged to
`C:\Users\ealsh\AppData\Roaming\world.uconstellation.app\write-journal.jsonl`
(`{ bytes, expected_cid, found_cid, hash, outcome, path, surface, ts }`). A keystroke save is
`surface: "editor_save"`.

**Steps (tutorial, for the Boss test phase):**
1. Launch Constellation; open a note — e.g. `E:\Constellation Universes\Scratch\Scratch me.md`.
2. Note its body text on screen. Leave the note **open**.
3. In a *separate* app (Notepad, or `git pull` / Syncthing), append a distinctive line to that same file
   on disk — e.g. `EXTERNAL-EDIT-2026-07-12` — and save it.
4. Wait ~1 s. **Observed bug part 1:** the open note in Constellation does **not** show the new line
   (the file tree/search do update — the editor doesn't).
5. Type one character in the open note. Wait ~2 s (the save debounce fires).
6. **Observed bug part 2:** the `EXTERNAL-EDIT-2026-07-12` line is **gone** — from the screen and from disk.
   The journal shows a fresh `editor_save` entry for that path writing the **pre-external** body/hash
   (the clobber). After the fix, the same steps show the external line adopted at step 4 and preserved at
   step 6.

*(No Constellation instance was running this session; #2 is run by the Boss in the migration's test phase,
or by me via computer-use if directed. #1 already locks the mechanism on demand.)*
