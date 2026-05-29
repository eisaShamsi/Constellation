# MIG-062 — Boss Test: Federate the filesystem-walk sidebar surfaces

**Date:** 2026-05-29
**Plan:** `docs/MIG-062-filesystem-federation-ARCHITECT-PLAN.md`

---

## What shipped

Three sidebar surfaces now see your cUniverses — **read-only**, so each cUniverse keeps its own files and detach is lossless:

1. **Tag Browser** — was a frozen-on-mount reactivity bug; now refreshes with federated tags.
2. **Five Acts** sidebar — now shows each cUniverse's own "Observation — Recent Captures" under a collapsible per-universe group.
3. **Workspace Bases** — now shows each cUniverse's saved bases under a collapsible per-universe group.

The core guarantee (your principle): the parent universe only **reads and displays** cUniverse files. It never writes, moves, or deletes them. Detach a cUniverse → its Five Acts + bases are fully intact, works standalone.

---

## Stages

### Stage 1 — Tag Browser federation

**Pre-state:** **Eisa Universe** (federated). Open the left sidebar's **Tags** browse mode (the notebook navigator's tag view).

**Action:** Look at the tag list. Wait a few seconds after boot if needed (the federated refresh fires when federation settles).

**Expected:** tags from cUniverse notes appear (not just the parent universe's). Tag counts include cUniverse occurrences.

**Counter-check:** **Eisa Cognitive Knowledge** (single universe) → tag list looks exactly as before.

**Failure:** *only parent tags* → the `$effect` re-sync didn't fire; tell me.

---

### Stage 2 — Five Acts federation

**Pre-state:** **Eisa Universe**. Look at the **Five Acts** section in the left sidebar (top, above Workspace Bases / Universe Notes).

**Expected:**
- The parent universe's **"Observation — Recent Captures"** shows directly, as before.
- Below it, **one collapsible group per cUniverse** that has a Five Acts note — labeled with the cUniverse's name (Arabic names render right-to-left) + a count.
- Click a cUniverse group header → it expands to reveal that cUniverse's Observation note.
- Click that note → it opens (in the correct cUniverse context).

**Counter-check:** **Eisa Cognitive Knowledge** → no cUniverse groups appear (it has none); the section looks exactly as before.

**Failure modes:**
- *No cUniverse groups in Eisa Universe* → the federation enumeration isn't finding cUniverse Five-Acts dirs; tell me.
- *Clicking a cUniverse note opens the wrong note / errors* → the path-based open resolution mis-fired.

---

### Stage 3 — Workspace Bases federation

**Pre-state:** **Eisa Universe**. Look at the **Workspace Bases** section. (This only shows cUniverse groups if a cUniverse actually has saved `.base` files — if none of your cUniverses have bases, this stage shows nothing extra, which is correct.)

**Expected (if any cUniverse has bases):**
- Parent's bases show directly with their normal right-click menu.
- cUniverse bases appear under collapsible per-universe groups.
- cUniverse base items are **open-only** — right-clicking them shows **no delete/rename menu** (read-only protection; you can't accidentally delete another universe's base from here).

**Failure:** *right-click on a cUniverse base offers delete* → the read-only guard is missing; tell me immediately (that would break the principle).

---

### Stage 4 — Standalone integrity (the principle)

This is the important one conceptually. **You don't need to actually detach** — the guarantee is structural: the backend code only ever *reads* cUniverse directories (verified: `fs::read_dir` only, no writes, no `create_dir_all` into cUniverses). So:

- Viewing a cUniverse's Five Acts / bases in the parent sidebar changes **nothing** on disk in the cUniverse.
- If you later detach that cUniverse (or open it as its own Universe), its Five Acts notes and bases are exactly as they were.

If you want to confirm: note a cUniverse's Five Acts file's modified-time before viewing it federated, view it, check the modified-time is unchanged.

---

## After all stages

Reply **"All pass"** → I cascade to **§G PCS** (orientation v2.42 + MoCh + 15-locale help-docs batched from MIG-061 + milestone tag + ZIP). That closes MIG-062 (8 of 14 federation surfaces done) and the next is MIG-063 (P2 read-paths).

If anything fails, paste which stage + symptom and I'll triage before §G.

---

## Build

`npm run tauri build`. The frontend changes are the bulk; Rust recompiles (universe.rs / bases.rs / system_notes.rs touched). Binary mtime should post-date commit `56cfa153`. Close + relaunch fresh.
