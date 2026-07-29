# MIG-107 — Single Ownership for PROPERTIES

**Architect + Plan, combined.** Status: **FOR BOSS APPROVAL. No product code written.**
Date 2026-07-28. Repo `main` @ `d1a808db` (PJ-174 AK-1 shipped).

> Combined into one document at the Boss's direction: the territory was already mapped by the
> reproduction, so a separate Architect round would restate it. Every mechanism claim below carries
> a `file:line` **read this session**; anything unverified is marked **[UNVERIFIED]**.

---

## 1. Function in hand + the concept (the horse)

**Function in hand:** the **Properties panel** — the frontmatter editor that appears in two places at
once: inside the note (NotePane's embedded block) and in the right sidebar's Properties tab.

**The concept (the horse):**

> **A note's properties have exactly ONE owner — the note model. Every panel is a window onto that
> owner, never a holder of its own version of it.**

The carriage is a props array. The horse is that the user's frontmatter — their stage, their
sources, their tags, their typed links — is knowledge they authored, and **two components must not
each hold a private opinion about what it currently is.** Today they do, and the one that saves
second silently deletes what the other one wrote to disk.

---

## 2. The defect, REPRODUCED (Reproduce-First satisfied before any design)

`tests/pj-174/propsOwnership.test.ts` — 3 tests: 1 green (the root cause), **2 RED** (the damage).

```
AK-2:  expected [ 'title', 'cid_cn', 'stage' ] to include 'tags'
AK-3:  expected 'seed' to be 'sapling'
```

* **AK-2** — a tag added from the file-tree menu is **erased from the `.md`** by the next unrelated
  property edit in the panel.
* **AK-3** — a `stage` edit made in one panel is **reverted on disk** by the other panel's next save,
  while the first panel still displays the new value, so the UI actively hides the loss until the
  note is closed and reopened.

Both are silent: no error, no dirty flag, no conflict sidecar, no save-health banner. The re-parse
after the write is clean, so nothing downstream notices.

**The green third test is the root cause, stated as an assertion:** after `saveTabContent` the model
has the new value and `tab.content` is **byte-identical to before**, with no store notification.

---

## 3. The territory — verified this session

### 3.1 The model already IS the authority for the WRITE path

`composeModel(m)` composes from `m.props` (`noteModel.ts:123`), so what reaches disk is the model's
array and nothing else. **This migration does not need to move the authority — it already lives in
the right place.** What is missing is that *nobody reads from it* and *every writer overwrites it*.

### 3.2 Both panels are seeded from a stale projection, and there are two of them

| | mount | `properties` comes from |
|---|---|---|
| in-note block | `NotePane.svelte:1639` | `NoteEditor.svelte:669` ← `parsed.properties`, and `parsed` = `parseFrontmatter($openTabs.find(...).content)` (`NoteEditor.svelte:133-136`) |
| right sidebar | `+layout.svelte:9018` | `sidebarProperties` (`+layout.svelte:1519`) ← `sidebarParsed.properties`, same `tab.content` origin |

Both are **projections of `tab.content`**, i.e. the file as it was when the tab was opened or last
reloaded. Each panel then copies that into its own `editableProps` (`PropertyEditor.svelte:161`) and
edits the copy.

### 3.3 The projection is deliberately never refreshed by a save

`saveTabContent` pushes to the model (`store.ts:1465`) and explicitly does **not** touch
`tab.content` — the in-code comment is *"Do NOT update the store during autosave"*. That decision is
**correct and must be preserved**: a store notification on the save path drives reactive cascades
and `{#key}` remounts on the typing path (Rule 1/2, the BUG-015 class). So the panels' seed can
never be current, by design.

The panel's own re-seed `$effect` (`PropertyEditor.svelte:360-366`) compares a JSON snapshot of the
same unchanged `properties` prop and therefore **skips**, which is why the staleness is invisible.

### 3.4 Every write is a WHOLESALE REPLACE with no staleness check

```rust
// noteModel.ts:207-217
export function setProps(id, props, expectPath?) {
    …
    m.props = cloneProps(props);   // ← the whole array, unconditionally
    m.version++;
}
```

There is an identity guard (`expectPath`) but **no freshness guard**. A panel holding a 20-minute-old
array can replace the model's current one in full.

### 3.5 The write surface is BOUNDED — counted, not estimated · **CLASSIFIED at Slice 1 (2026-07-28)**

Swept with **no exclusions** (LL-038 rule 3, which exists because yesterday's sweep excluded the file
being edited and hid an APP-KILLER). Every writer of a model's props, all seven:

| site | reads its array FROM | await between read & write? | verdict |
|---|---|---|---|
| `store.ts:1465` (`saveTabContent`) | **the panel's `editableProps`** — a projection captured at open | stale **by construction** | ⚠ **THE DEFECT** → intents |
| `NoteEditor.svelte:205` (stage promote/demote) | `getModel(tab.id).props` | **none** | ✅ sound; convert for uniformity |
| `NoteEditor.svelte:478` (shape) | `model.props` | **none** | ✅ sound; convert for uniformity |
| `+layout.svelte:5064` (template apply) | `getModel(tab.id).props` | **none** | ✅ sound; a batch of `addProp` |
| `noteModel.ts:234` (`replaceContent`) | authored/merged content, **and re-bases `m.base`** | — | ✅ **legitimate wholesale — KEEP** |
| `noteModel.ts:362` (`adoptDisk`) | disk, **re-bases + syncs `diskBaseline`** | — | ✅ **legitimate wholesale — KEEP** |
| `noteModel.ts:214` (`setProps`) | the caller's array | — | the primitive; retained for the two above |

**The finding that matters, stated honestly: the three non-panel writers are NOT defects.** Each reads
the model and writes back a derived array **with no intervening `await`**, so the array it writes is
derived from current truth. The bug is unique to the PropertyEditor path, where the array originates
from a projection captured minutes earlier.

⇒ **The intent API is not needed to *fix* these three — it is needed to make the shape uniform so a
correct read-modify-write cannot later regress into a stale one** (e.g. someone adding an `await`
between the read and the write). Slice 5 converts them for that reason, not to repair them, and it
is therefore the lowest-risk slice rather than a second front.

### 3.5.1 Why the panel path is the broken one — the mechanism, exactly

`setProps` changes `m.props` **without advancing `m.base`** (contrast `replaceContent` and
`adoptDisk`, which both re-base). That is correct and deliberate: the base must stay at the
last-known-disk bytes for the byte-perfect in-place contract. But it means **every compose is a diff
against the file as it was when the note was opened.**

With ONE source for the new array that is fine — the diff is complete. With two, it is not:

1. Writer A adds `tags`. Model has `tags`; disk gets `tags`; **`m.base` still has none.**
2. Writer B (panel) submits an array built from the open-time projection — also no `tags`.
3. compose: `tags` is in **neither** `oldByKey` (the base) nor `newByKey` → **no REMOVE is emitted
   and no ADD is emitted**, and the CST is re-parsed from `m.base.rawYaml`, the open-time bytes.
4. The write is clean, and the tag is simply **not in it**.

**This is precisely why single ownership fixes it rather than papering over it:** once writer B reads
the model, its array *contains* `tags`, so compose sees `tags` present in `newByKey` but absent from
the base — an **ADD** — and emits it. The existing diff algorithm is already correct; it was being
fed a lie.

Inside `PropertyEditor.svelte`, `editableProps` is mutated at ~10 sites (`:334`, `:526`, `:570`,
`:592`, `:605`, `:609`, `:660-661`, `:668`), every one a simple array operation — set-one-field,
append, remove-at-index, reorder. **They map one-to-one onto intents; none needs invention.**

### 3.6 The model is deliberately NON-REACTIVE

Established and must be preserved (`noteModel` is a plain module store, not `$state`). So panels
cannot subscribe to it directly — §5.2 is the bridge that respects this.

---

## 4. Why the obvious fixes are WRONG (options, honestly)

### Option A — refresh `tab.content` after every save, so the projection stays current

**Rejected.** It reverses the deliberate decision at §3.3 and puts a store notification back on the
save path — the reactive-cascade hazard the codebase spent BUG-015 learning. And it does **not** fix
the wholesale replace: two panels can still clobber each other inside one notification cycle. It
keeps **two copies of the truth and syncs them harder**, which is the definition of muddling rather
than securing.

### Option B — mount only ONE PropertyEditor per note

**Rejected, and it is worth saying why, because it is the instinctive answer.** Two *views* of one
truth is correct and desirable — the user should see properties in the note and in the sidebar. The
bug is not that there are two panels; it is that each panel owns a private **copy**. Removing a
panel would hide AK-3 while leaving AK-2 (one panel vs. the tree-menu writer) completely intact, and
would cost the user a surface for no structural gain. **Keep both views; remove both copies.**

### Option C — add a version/staleness check to `setProps` and reject stale writes

**Rejected as the primary fix.** It converts a silent loss into a silent *no-op* — the user's edit
simply doesn't apply, with nothing said. It also cannot distinguish "stale for the key I'm editing"
from "stale for a key I never touched", so it would reject legitimate edits constantly.

### Option D — SINGLE OWNERSHIP: panels read the model, and write INTENTS ✅ **RECOMMENDED**

The structural end-state CLAUDE.md's Solve-the-Class rule requires for the content-integrity class.
Detailed in §5.

---

## 5. The design, locked

### 5.1 The two changes, and that is the whole design

**1. Panels READ from the model, not from `tab.content`.**
**2. Panels WRITE field-level intents, not whole arrays.**

Adding a tag in the sidebar then **does not touch `stage` at all** — so there is nothing for a stale
copy to revert, because there is no stale copy and no operation that spans keys it was not asked
about. The bug becomes *unrepresentable* rather than *guarded against*.

### 5.2 The read bridge — one signal, not reactivity in the model

The model stays non-reactive (§3.6). A single `propsVersion` store is bumped whenever a model's props
change; panels render via `$derived.by(() => { $propsVersion; return getModel(tabId)?.props ?? [] })`.

* **One tick per PROPERTY change — never per keystroke.** Body typing does not touch props, so the
  signal is silent during writing. This is the difference from Option A, which would notify on the
  save path for every edit including body saves.
* Both panels subscribe to the same signal, so they show the same truth **in the same frame** — the
  render↔state agreement LL-034 demands.

### 5.3 The write intents — the complete, closed set

Derived from the ~10 mutation sites in §3.5, not invented:

| intent | replaces |
|---|---|
| `setPropValue(id, key, value, listItems?)` | `:334`, `:526`, `:570`, `:592`, `:660-661`, `:668` |
| `addProp(id, prop)` | `:605`, `:661` (the append branch) |
| `removeProp(id, key)` | `:609` |
| `reorderProps(id, fromKey, toKey)` | the drag-to-reorder path |
| `renamePropKey(id, oldKey, newKey)` | the key-edit path |

Each applies to the model's **current** array. `setProps` (wholesale) survives **only** for the two
callers that genuinely mean "replace everything": opening/re-seeding a note, and the PJ-088
conflict-merge rebase.

### 5.4 Key vs. row id — ANSWERED at Slice 0 (2026-07-28): **KEY**

Measured, not reasoned — `tests/pj-174/propsContract.test.ts`, 6 tests, all green:

* **`composeFrontmatter` is entirely key-addressed** (`yamlDoc.ts:338-343` builds `oldByKey` /
  `newByKey` as `Map`s keyed by `p.key`), so the persisted representation **structurally cannot
  carry duplicate keys**. Probed on the real function: `stage: sapling` + `stage: wilting` →
  `stage: wilting`, the first silently discarded, and no arrangement of the array yields two
  `stage:` lines.
* **Position is not identity**: shuffling the array with no value change produces byte-identical
  output.

⇒ **Intents key on the property key**, which is the only identity the file format has. Row ids would
invent an identity the substrate cannot persist.

**The objection that made this look impossible, and its resolution.** `addProperty()`
(`PropertyEditor.svelte:605`) appends `{ key: '', value: '', type: 'text' }`, and nothing filters
empty keys — so two blank rows *can* coexist in a panel. But that is an **editing state of one
panel, not a state of the shared authority**: a half-typed row stays local to the panel until it has
a non-empty key, and only then becomes an `addProp` intent. The model therefore holds exactly what
can be persisted — well-formed, key-unique properties.

A **key collision on rename** (typing a key that already exists) stops being a silent last-wins drop
and becomes an explicit, surfaced decision. Design ruling: **reject the rename and tell the user**,
rather than overwrite a property they did not mean to touch.

### 5.5 A live defect Slice 0 uncovered — PJ-178

Reachable **today**, independently of this migration: click "+" in Properties, then edit any other
property, and the blank row is flushed with the array into `composeFrontmatter`, which serialises it
as a literal `"": ""` line in the note's frontmatter. Pinned by
`propsContract.test.ts::an_empty_key_row_is_written_to_the_file`, which currently asserts the
**broken** behaviour and flips to `not.toContain` when the fix lands. The single-ownership design
closes it structurally (§5.4), so it is fixed by Slice 4 rather than separately patched.

---

## 6. Invariants that must not break

1. **The model stays non-reactive.** The bridge is a separate signal; no `$state` inside the model.
2. **No store notification on the save path** (§3.3) — the reason Option A is rejected.
3. **Zero new work on the keystroke path.** Body typing must not tick `propsVersion`.
4. **`expectPath` identity guards stay on every write** (`noteModel.ts:213`) — the new-note-leak poison.
5. **Compose still diffs against the open-time base** (`m.base`); intents change *what* the model
   holds, never *how* it is serialised. Non-projectable YAML (nested maps, block scalars) must round-
   trip byte-identically — the PJ-136 `nestedRaw` contract.
6. **The Editor-Surface Gate checklist, all eight**, including **Focus mode** (not optional — it was
   the site of the 2026-06-12 corruption) and the standalone PropertyEditor instance.
7. **`saveTabContent` keeps its cascade gate BELOW the model push** (PJ-174 #1c, shipped `d1a808db`).

---

## 7. The slices

Each lands alone, each has a verification clause. **Toggle: `PROPS_SINGLE_OWNERSHIP`.**

| # | What lands | Files | Boss-testable? |
|---|---|---|---|
| **0** | ✅ **DONE 2026-07-28.** §5.4 answered by measurement (**KEY**, not row id) + the substrate contract pinned: duplicates collapse last-wins, position is not identity, remove splices only its key, rename = remove+add, untouched keys are never rewritten. Uncovered **PJ-178** (a blank row is written as `"": ""`). `propsContract.test.ts` 6/6 | `tests/pj-174/` | no |
| **1** | ✅ **DONE 2026-07-28.** All **seven** prop writers swept (no exclusions) and classified — §3.5. The three non-panel writers are **sound** (each reads the model with no intervening await); the two model-internal ones legitimately re-base and are **kept**. Mechanism pinned in §3.5.1, and **the design proven at the substrate**: the identical two-writer sequence is lossless when writer B reads from the model. No behaviour change | — | no |
| **2** | DONE 2026-07-28. Five intents in the model (`setPropValue` / `addProp` / `removeProp` / `renamePropKey` / `reorderProps`), each identity-guarded and version-bumping ONLY on a real change; `setProps` retained for the two legitimate wholesale callers. The signal lives in a NEW leaf `propsSignal.ts` — NOT in the model, which stays non-reactive per C-2 — ticked by the `noteSession` wrappers so the announce sites stay explicit. Verified inert: no app consumer, and with zero subscribers a tick is a counter increment. `propsIntents.test.ts` 19/19 | `noteModel.ts`, `noteSession.ts`, `propsSignal.ts` (new) | no |
| **3** | DONE 2026-07-28, **Boss-validated**. `PROPS_SINGLE_OWNERSHIP` added; the panel seeds from `sourceProps` (the model, invalidated by `$propsVersion`) instead of the `tab.content` projection, with a `?? properties` fallback for hosts that mount before a model exists. Guard added: never re-seed over an un-flushed local edit. Still writes whole arrays — the race is Slice 4 | `PropertyEditor.svelte`, `ownershipFlag.ts` | yes |
| **4** | DONE 2026-07-28, **Boss-validated (5/5)**. The swap: `propsCommit.plan/apply` turns the panel's rows into per-key operations; `saveTabContent(..., propsAlreadyInModel)` writes from the model. A commit may ADD/SET freely but may only REMOVE a key in `seededKeys`, and may only SET a key in `touchedSince()`. Both AK reproductions flipped to green; **PJ-178 closed**. Four defects fixed in-pass — see 5.6 | `PropertyEditor.svelte`, `propsCommit.ts` (new), `store.ts` | yes |
| **5** | DONE 2026-07-29, **Boss-validated 4/4**. All three converted; the case-insensitive key match preserved (a note spelling it `Stage:` must not gain a second key). **No whole-array property write remains in the app.** Plus a Boss-found fix: the header badge and file-tree stage icon now derive from the MODEL, because the sidebar Properties mount was never given the `onstagechange` callback the in-note mount has — fixed by removing the dependency, not by adding the missing wire | `NoteEditor.svelte`, `+layout.svelte` | yes |
| **6** | DONE 2026-07-29. `/simplify` (4 agents) applied — a real aliasing bug (my hand-rolled clone missed `nestedObjects`), a quadratic ~100-allocations-per-edit ordering waste, a no-op commit still hitting disk, and redundant already-drifted state. Docs: User Manual + Properties help topic. Safety inspection run. **Toggle RETAINED by Boss ruling** — removed when MIG-104 closes | — | no |

**Not coverable by vitest, and not pretended otherwise.** The Editor-Surface Gate items that need
the running app — **Focus mode** (enter/type/exit round-trip and the teardown flush), the
**standalone PropertyEditor** instance, the **second screen**, and render↔state same-frame agreement
— are live-gate items on Slices 3/4, not harness tests. jsdom can prove the offset-pure half only
(LL-034's corollary); claiming otherwise would be the over-privileged-fixture mistake of LL-036.

**Slices 3 and 4 are separated deliberately.** The read half is provable on its own (both panels show
the same truth), and if the swap has to be reverted, reverting Slice 4 alone leaves a correct,
already-validated read path in place. That is the "one validated swap" the rule asks for, without
making the swap larger than it needs to be.

### The headline verification clause (Slice 4) — the Boss test

> **What this is.** Constellation shows your note's properties in two places — inside the note, and
> in the right sidebar. Until now each of those kept its own copy of what your properties were,
> taken when you opened the note. Whichever one you used last would write its whole copy over your
> file — quietly deleting anything the other one had changed in the meantime.
>
> **Step 1.** Open a note with the sidebar Properties tab showing, and the in-note Properties block
> expanded, so you can see both at once.
> **Step 2.** In the **in-note** block, change the note's **stage**. Watch the **sidebar** — it now
> shows the same new stage immediately. (Before this build it would still show the old one.)
> **Step 3.** In the **sidebar**, add a **tag**. Watch the in-note block — the tag appears there too,
> and the stage you set in Step 2 is **still your new value**.
> **Step 4.** Close the note and reopen it. Both the stage and the tag are there.
>
> **Failure modes.** The two panels disagree at any point → the read bridge is not firing. The stage
> reverts after adding the tag → the intent path is still replacing the whole array. Typing in the
> body feels even slightly less instant → stop; that is a revert, not a tune (invariant #3).

---

## 8. Risk register

| invariant | risk | guard |
|---|---|---|
| Non-projectable YAML round-trips | an intent path re-serialises a nested map the old path left alone | Slice 0 recipe asserts byte-identity on `nestedRaw`; compose is untouched by design (§6.5) |
| Keystroke latency | the `propsVersion` signal ticks on body edits | Slice 0 asserts zero ticks across a body type-burst |
| Focus mode | props edited while in Focus, or a Focus teardown replaying stale props | Editor-Surface Gate #2/#4, in the harness before Slice 3 |
| Second screen | a props edit on one window not reflected on the other | the signal is per-window; **[UNVERIFIED]** how the second screen currently receives props — resolved at Slice 1 |
| Revert path | Slice 4 misbehaves live | toggle off restores the whole-array path, which Slice 3 leaves intact and validated |

---

## 9. What this migration does NOT do

* It does **not** remove either Properties panel (§4 Option B).
* It does **not** change how frontmatter is serialised, or touch `composeFrontmatter` / the CST.
* It does **not** make the note model reactive.
* It does **not** address the other 44 findings in the 2026-07-28 sweep register (PJ-174), or PJ-176
  / PJ-177, which stay filed for the Boss's own sequencing.

---

## 10. Boss approval checklist

1. **The concept** (§1) — properties have one owner; panels are windows, not holders. Sound?
2. **Keeping BOTH panels** (§4 Option B) — the instinctive fix is to delete one; I recommend against.
3. **Slices 3 and 4 split** (§7) — read half provable alone, swap revertible alone.
4. **The Slice-4 Boss test** (§7) — is that the behaviour you want to see?
5. Anything in §9 you expected to be in scope and is not.


---

## 5.6 The four defects Slice 4 surfaced — three by inspection, one by the Boss

| # | what | how found |
|---|---|---|
| **#1d** | **APP-KILLER, introduced by my own AK-1 #1b fix.** The dirty-rename branch kept the user's model with `repathNoteModel` alone — which moves `m.path` and nothing else — so the G4 write-base stayed at the PRE-rename bytes. compose then diffed base against props, saw zero difference, and re-emitted the OLD frontmatter: the title reverted, the alias `rename_item` had just stamped was deleted, and every wikilink the cascade had rewritten pointed at a title that existed nowhere. On a canonical note that undoes the rename entirely. Silent — the write is watcher-suppressed, and `markSaved` + `diskBaseline` then made the stale model permanently self-consistent. **Fixed:** keep the user's BODY, but re-base the frontmatter to the renamed file. | safety inspection |
| **#1e** | `prevPropsSnapshot` advanced OUTSIDE the re-seed guard, so a model change landing while a save was pending was marked "seen" without reaching the rows — and never re-seeded after. Compounded by the commit writing back every row it held, reverting another writer's change on a key BOTH panels show. **Fixed:** advance the snapshot only on an actual re-seed, and only SET keys the user edited. | safety inspection |
| **#1f** | My comment claimed ordering never moves an unseen key. **False** — a trailing `beforeKey: null` means "move to the END", displacing any foreign key after it. Worse, I had written a **test asserting the wrong contract**, which makes a mistake look verified. **Fixed:** anchor only BETWEEN known keys; test corrected. | safety inspection |
| **#1g** | **The Boss found it.** `touchedKeys` was hand-marked from the panel's edit handlers — wired at **3 of this component's 16** mutation sites. The tag editor was one of the 13 missed, so a tag added in one panel reached neither the other panel nor the file. **Fixed by removing the hand-marking entirely:** `touchedSince(seededRows, localRows)` derives it by comparison, so any edit from any path is detected, including sites added later. | Boss test |

**#1g is the second time in two days I fixed only the sites I happened to look at** (#1b was the first). The durable correction is not "be more careful with the list" — it is **stop keeping a list**: a completeness requirement that depends on every future contributor remembering is a defect waiting for its next author. Recorded as LL-038 rule 6.


---

## 5.7 Slice 6 — what was NOT done, and why

**The toggle stays.** Boss-ruled 2026-07-29. The trade was put honestly: a rollback flag is only
worth having if the path behind it still works, and that path has not been exercised since Slice 3.
Against that, this migration produced seven in-pass defects and **four were found by the Boss in
live testing, not by the suite**. Removal moves to the MIG-104 close.

**Two false claims from the Slice-5 commit, corrected.** The altitude review caught them and I
verified both by grep: (a) *"no whole-array property write remains in the app"* — `addTagToNote`
(`+layout.svelte:6536`) still does one, and it is the exact writer `propsCommit`'s header names as
the canonical foreign key; (b) *"there is no callback left to omit"* — the push channel
(`onStageChanged`, `+layout:8533`/`:8742`) is still fully wired, so I added a pull mechanism BESIDE
it rather than replacing it. That display fact now has two owners — the same shape as the defect
this migration removes, one layer up.

**PJ-180 — the altitude review's larger findings**, deferred as a second pass rather than patched at
the end of a long session: a by-name `setPropByName` intent (so the case-fold / empty-means-remove
triad has one home); a generic `noteProp(id, key)` read facade so *any* display surface reads a
property the same way — which is what lets the push channel be **deleted** rather than shadowed;
splitting `saveTabContent`'s two modes; moving the seed/commit/re-seed bookkeeping into a
`propsCommit` draft handle; and `buildFullContent`, which is a **lossier** composer than the
`compose()` every other writer uses.
