# Concept Paper — The Broken Universe Link

**v1.1 · 2026-08-20 · supersedes v1.0 (kept alongside, per the versioning discipline).**

Answers Boss decision 3 of 2026-08-20, in his words:
> *"No [do not refuse entirely]. But the app should provide options to choose from."*

**v1.0 was sent back by the panel: concept ACCEPTED, diagnosis REJECTED ON FACT.** This version
rewrites every section the panel faulted. **What changed, and why, is listed in §10** — v1.0 was
wrong in ways worth keeping on the record.

**Status:** concept only. Two defects that v1.0 *exposed* have been fixed already (§2.3); no code for
the concept itself should be written until this version is accepted.

---

## 1. The concept (the horse) — unchanged, and the panel accepted it unanimously

> **A link to another universe is a claim about something outside this universe's control.**
> When that claim stops being true, Constellation's job is to say *precisely how* it stopped being
> true, and to offer only the actions that are honest for that specific condition — never to guess,
> never to act on its own, and never to carry a broken link silently as though it were whole.

The question it answers, in the user's words: **"You said this universe was linked. It isn't
working. What exactly happened, and what can I do about it?"**

**Boundary, stated because v1.0 left it vague.** This paper answers the **dangling** half of the
Boss's question — a link whose target has stopped being usable. It does **not** answer the
**unregistered** half, and it does **not** touch the create / remove / re-open doors that actually
cost him time on 2026-08-20. Those need their own concept.

---

## 2. What is actually broken — corrected

### 2.1 The disease is SILENCE, not a bad sentence *(v1.0 had this inverted)*

v1.0 claimed the app blames a missing `search.db` for four different conditions. **That is false,
and the panel proved it at file:line.**

`attach_all` never reads the `children` list. It reads the resolved *library* list
(`federation/attach.rs:132`), and derives universe roots by walking **upward from each library path**
(`attach.rs:105-118`). A child that is gone or unreachable is dropped two layers earlier:

```rust
// universe.rs:641-644 (resolve_libraries_recursive)
let child_canon = match fs::canonicalize(child_path_str) {
    Ok(p) => p,
    Err(_) => continue,      // ← the link disappears here, silently
};
```

No libraries → no cUniverse root → `attach.rs:158-159` never runs → **no warning, no badge**
(`+layout.svelte:10547` renders only when `federationWarnings.length > 0`).

So `"search.db missing"` is reachable only for a child that *exists, is readable, contributes at
least one library, and has no index* — where the message is roughly correct. **For the two
conditions that matter most, the app says nothing at all.**

### 2.2 Two conditions produce a WRONG answer, not silence *(found by the panel; in neither v1.0 nor two of three reviews)*

`resolve_libraries_recursive` loads the child's `libraries.json` at step 1 (`universe.rs:615-624`)
**before and independently of** reading its `universe.json` at step 2 (`universe.rs:634-651`).

- **Manifest damaged** — the child's libraries still load, and `find_universe_root`
  (`attach.rs:112`) tests only that `.constellation/universe.json` **exists**, never that it parses.
  The child **attaches normally and emits nothing.**
- **No manifest** — the libraries still load into the merged list, but the upward walk finds no
  manifest and keeps climbing to the *active* universe, where `attach.rs:88-91` discards it. Net:
  **a dead child's libraries remain in the active universe's own library list with no database
  attached for them.**

*The downstream user-visible consequence across every consumer of that list is **not verified**.*

### 2.3 One sentence was genuinely wrong — and it was mine, from this morning. FIXED.

`universe.rs` preserved `PersistedError::{Unreadable, Corrupt}` deliberately, under a comment saying
`Corrupt` is permanent and retrying is pointless — and `mig108.rs` then **flattened both to one
string one call later**, so the unify dialog told the user *"this is usually temporary… Try again"*
about a file that will never repair itself.

**Fixed in the same session the panel found it** (Working Agreement #6): the kind now crosses the IPC
boundary as a machine-readable marker, the dialog renders a different body for a damaged file, and
**the Try-again button is not rendered at all** when retrying cannot work. A second defect — a
malformed user-facing sentence carrying 22 literal spaces, the *only* prose Constellation currently
produces about a broken child link — was fixed in the same pass.

**This is the concept's own thesis, found operating inside the code written to serve it.** It is
recorded here rather than quietly repaired because it is the best available evidence that the concept
describes something real.

---

## 3. What Constellation can honestly distinguish — re-derived

Re-derived against `universe.rs:615-651`, not asserted. **Every condition is stated with where it is
decidable**, because v1.0's conditions were decidable in principle and computed nowhere.

| # | condition | decidable at | what the app does today |
|---|---|---|---|
| 1 | **Healthy** | — | correct |
| 2 | **Gone** — the folder is not there | `universe.rs:641` (`canonicalize` fails) | **silent drop** |
| 3 | **Unreachable** — the volume or share is not available | `universe.rs:641` | **silent drop** |
| 4 | **No longer a universe** — folder present, no manifest | `universe.rs:634-651` / `attach.rs:112` | **libraries leak into the active universe's list** |
| 5 | **Manifest damaged** — present, unparseable | `universe.rs:634-651` | **attaches as if healthy** |

**The condition must be computed at `universe.rs:641`, where the link is currently dropped.** That
is the single point that sees every child before anything downstream can lose it. Deciding it
anywhere else re-creates the drift this paper complains about.

### 3.1 The Gone/Unreachable test — v1.0's rule was WRONG, and so was the first fix proposed for it

v1.0 said: *not found* ⇒ Gone; *any other error* ⇒ Unreachable. **The panel measured it on this
machine and it fails:**

| path | Windows code |
|---|---|
| a deleted folder on a present drive | **2** (file not found) |
| a missing *ancestor* on a present drive | **3** (path not found) |
| **`Z:\foo` — no such drive (the unplugged case)** | **3** (path not found) |
| `\\nosuchhost\share\x` | 53 |

**A missing folder on a present drive and an absent drive return the same code.** v1.0's rule
collapses its own canonical Unreachable example into Gone — and then offers the one button it calls
unrecoverable. A reviewer's proposed fix (use the raw OS code instead of the error kind) **fails
identically**, and the panel refuted it.

> **The sound discriminator is a volume/share-root probe:** a missing folder on a *resolvable*
> volume is **Gone**; any path on an *unresolvable* volume or unreachable host is **Unreachable** —
> whatever the error code says.

*How Rust's `ErrorKind` maps codes 3/15/53/67 on this toolchain is **not verified**. It does not
change the rule: the split must not be built on `ErrorKind`, nor on the raw code alone.*

### 3.2 Reuse the mechanism; do not invent a fourth *(panel amendment, accepted)*

This distinction already exists here three times, and v1.0 introduced `fs::metadata` as though it
were new:

- `read_persisted_json` + `PersistedError::{Unreadable, Corrupt}` (`universe.rs:234-290`), with the
  doctrine written out: *"Only 'not found' is trustworthy emptiness."*
- `resolve_child_universe_roots_strict` (`universe.rs`) — already splits `NotFound` from every other
  error on a declared child.
- `carries_universe_manifest` (`mig108.rs`) — **already uses `fs::metadata`, not `Path::exists()`**,
  with v1.0's own reasoning already in its comment.

---

## 4. The principle the options obey — unchanged, and accepted

> **The option set is derived from the condition. The app never offers an action that would be wrong
> for the condition it just diagnosed.**

**"Leave it" is always present and always the default.** Nothing here ever acts on its own.

**But see §7: no buttons ship in v1**, because §6.1 reveals that the engine behind the most obvious
one does not exist.

---

## 5. What this function refuses to do

- **It never writes into a universe it does not own.** Boss ruling 2 (2026-08-17) is that the parent
  never writes **schema** into a universe it does not own; the principle extends to a damaged
  manifest, which is that universe's statement of its own identity. Say what is wrong, name the
  file, stop. *(v1.0 misquoted this ruling as "never writes"; corrected.)*
- **It never removes a link on its own** — not on a timer, not at boot.
- **It never treats "unreachable" as "gone."** §3.1.
- **It never blocks startup.**

---

## 6. Where it belongs — corrected and completed

### 6.1 The finding that reorders this whole paper: **you cannot unlink a child universe**

A child universe can be **added** from two places (`LibrarySwitcher.svelte:31`,
`UniverseManager.svelte:128`). The only **removal** control is `handleRemoveChild` in
`UniverseSetup.svelte:206` — the **first-run setup wizard**, reached when there is no universe
configured or the last one was removed (`+layout.svelte:10605`, `onRemoveLast`).

> **In normal operation there is no way to unlink a child universe.** The command exists
> (`remove_child_universe`); `UniverseManager.svelte:7` even *imports* the wrapper — and never calls
> it.

So **"Remove the link" is not an option to design — it is a capability the running app does not
have.** Any version of this concept that offers that button is proposing a feature, not a message.
This is the single strongest reason v1 must be detection-only.

### 6.2 The surfaces — five, not three

| surface | what a broken child looks like today |
|---|---|
| **Main sidebar** (`+layout.svelte:8374-8391`) | chevron + planet icon + name + a `0`, expands to nothing. A permanent row — **v1.0 missed this entirely** |
| **Library switcher** (`LibrarySwitcher.svelte:118-133`) | "Child Universes" section, name + "0 libraries" |
| **Dashboard** (`DashboardView.svelte:200-228`) | a card reading 0 / 0 / 0, no distress cue |
| **OrgChart** (`OrgChart.svelte:86-96, 995`) | not examined |
| **Status-bar badge** (`+layout.svelte:10547-10589`) | a triangle + count; popup shows the path and a verbatim reason |

**The first four and the badge are not two views of one list.** The first four come from
`get_child_universes`, which reads `meta.children` **directly and non-recursively**. The badge comes
from `attach_all`, which is **recursive but library-derived**. A broken *grandchild* appears in
neither; a Gone child appears in the first four but never the badge.

### 6.3 Two further defects in the badge itself

1. **It can silently never appear.** Warnings are fetched at boot and re-polled **once at ~3
   seconds** (`+layout.svelte:3079-3106`). The `federation:ready` listener (`:3568`) re-fetches sky
   and graph but **not warnings** — its only three call sites are `:2840`, `:3081`, `:3098`. Any
   attach settling after 3 s leaves the badge empty for the whole session.
2. **The reason cannot be translated.** `+layout.svelte:10582` renders the raw Rust string with no
   `$t()`, and the `federation` i18n block has no key for reason text. **A better reason string is
   therefore an IPC-contract change, not wording** — it must cross as a code plus data, or it ships
   English-only in 15 locales and any condition-derived buttons would have to pattern-match English
   prose. *(v1.0 filed this under "the exact wording". Wrong.)*

### 6.4 One concept, three names on screen — and all three are RETIRED

`federation.warningBadge` = "cUniverses unavailable", `popupTitle` = "Federation warnings",
`cuniverseLabel` = "cUniverse" — while the switcher, sidebar and Dashboard say **"Child Universes"**
and the setup wizard says **"Add Child Universe."**

**The word is settled and it is none of those: it is a "Linked Universe."** Boss ruling, re-stated
2026-08-20. "cUniverse" is jargon; "Child" implies a subordinate, and a linked universe is a peer
whose libraries are federated in.

> **The panel recommended "Child Universe" — and was wrong, through no fault of its own.** The
> ruling had been taken once and never written into `CLAUDE.md` or the orientation doc, both of
> which still *defined* the level as "cUniverse (Child Universe)". Every agent read the documents,
> and the documents were stale. **A decision that lives only in conversation will be contradicted by
> your own records and then re-recommended back to you.** Recorded now in `CLAUDE.md`, in memory, and
> as PJ-331 with the exact 12-string inventory.

This matters to *this* concept directly: a perfectly-worded reason string, under a heading naming a
thing the user does not recognise, still fails §9's test.

### 6.5 MIG-108 — condition by condition *(v1.0 issued one instruction and was wrong for half of it)*

v1.0 said the unification proposal *"should state what it found and continue, not refuse."*
Verified behaviour:

- **Gone child** → the declared path is kept → `assemble_foreign_roots` succeeds → **preflight
  already proceeds. v1.0's recommendation was already the shipped behaviour.**
- **Unreachable child** → `Unreadable` → preflight refuses → the blocked card. **That refusal is
  Boss decision 1, taken the same morning:** *"a plan that moves directories must refuse rather than
  guess."*

**v1.1 does not propose changing either.** What should improve is that the refusal names the
condition in the user's language — which §2.3 has now begun.

---

## 7. Scope for v1 — OVERTURNED from v1.0

v1.0 recommended "the reason string first." **The panel overturned it and is right:** for Gone and
Unreachable there is no warning row for a better string to live in (§2.1), so a string-only v1 would
improve the least-wrong case and leave the two that matter invisible.

**v1 is DETECTION ONLY, and it is honest about having no buttons:**

1. Compute the condition at `universe.rs:641`, where the link is dropped, using the §3.1 volume probe.
2. Carry it to the surfaces as a **code**, not prose (§6.3).
3. Make a broken child **look broken** on the four `get_child_universes` surfaces — the sidebar row
   being the open design question (§8).
4. **No actions.** Every button in §4 needs an engine, and §6.1 shows the most obvious one has none.

---

## 8. What this paper still does NOT settle

- **What a broken child looks like as a tree node** in the sidebar. Named by the panel's user-truth
  lens as the first thing v1.1 must answer, and it is a design question, not an engineering one.
- **Whether the five conditions collapse to two behavioural classes and five sentences.** A fair
  simplification; secondary to §2.2.
- **Whether a legacy-layout child can ever produce a warning at all** — `attach.rs:112` accepts only
  `.constellation/universe.json`, while four other readers also accept a root-level `universe.json`.
  Raised by a skeptic, **not adjudicated**.
- **The word** (§6.4), and **the wording**, which is a 15-locale surface.
- **The unregistered half** of the Boss's question, and the create/remove/re-open doors. Out of scope
  by §1.

---

## 9. The one-line test for anything built from this

> *Could the user, reading only what the app said, tell which of the conditions they are in — and is
> every button in front of them one they would not regret pressing?*

**The shipping app fails the first half today.** Under §7 it would pass the first half and have no
buttons to fail the second — which is the honest place to start.

---

## 10. What changed from v1.0, and why

| § | v1.0 said | corrected to |
|---|---|---|
| 2 | one wrong string for four conditions | **silence** for the two that matter; the wrong string reached only condition 1 — **inverted** |
| 2.2 | — | conditions 4 and 5 produce **wrong answers**; a dead child's libraries leak into the active list |
| 3.1 | *not found* vs *other error* | **measured to fail** — a deleted folder and an absent drive both return code 3; use a **volume probe** |
| 3.2 | introduced `fs::metadata` as new | it already exists three times here; cite and reuse |
| 5 | "never writes into a universe it does not own" | the ruling says **never writes schema**; misquotation corrected |
| 6 | three readers | **five surfaces**, plus **you cannot unlink a child at all** (§6.1) |
| 6.3 | the reason string is "wording" | it is an **IPC-contract change** |
| 6.5 | "state what it found and continue" | **already true** for Gone; would **reverse Boss decision 1** for Unreachable |
| 7 | reason string first | **detection only, no buttons** |

**Four of the nine were factual errors, not differences of judgment.** Recorded in full because a
concept paper that quietly absorbs its own corrections teaches nobody anything.
