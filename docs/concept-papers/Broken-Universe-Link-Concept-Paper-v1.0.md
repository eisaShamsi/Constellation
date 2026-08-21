# Concept Paper — The Broken Universe Link

**v1.0 · 2026-08-20 · written in response to Boss decision 3 (2026-08-20):**
> *"No [do not refuse entirely]. But the app should provide options to choose from."*

**Status:** concept only. **No code is proposed here and none should be written until this paper's
concept is accepted and the panel has ruled on scope and surface.** Under *Concept Before Function*,
a menu of buttons invented ahead of the concept is a carriage with no horse.

---

## 1. The concept (the horse)

> **A link to another universe is a claim about something outside this universe's control.**
> When that claim stops being true, Constellation's job is to say *precisely how* it stopped being
> true, and to offer only the actions that are honest for that specific condition — never to guess,
> never to act on its own, and never to carry a broken link silently as though it were whole.

The question this function answers, in the user's words: **"You said this universe was linked. It
isn't working. What exactly happened, and what can I do about it?"**

Everything below follows from that sentence. If a proposed button cannot be traced back to it, it
does not belong.

---

## 2. What is actually broken today — verified, not assumed

A child universe is declared as a path string in the parent's `universe.json` `children` array. The
folder it names is not owned by this universe and can change underneath it at any time.

**Three separate places read that list, and each degrades differently:**

| where | what it does when the child is not usable | verified at |
|---|---|---|
| `federation::attach::attach_all` | emits **one** warning, `"search.db missing"` | `attach.rs:158-159` |
| `universe::get_child_universes` | falls back to the **folder name** and reports **0 libraries** | `universe.rs` (the name/count fallbacks) |
| `universe::resolve_child_universe_roots` | **silently drops** the child (`canonicalize().ok()`, `is_dir()`) | `universe.rs:693-694` |

### The defect, stated plainly

**`"search.db missing"` is what the user is told when the folder has been deleted, when the drive
holding it is unplugged, when someone removed its `.constellation`, and when the universe genuinely
has no index yet.** Four conditions, one message — and the message names a file the user has never
heard of, in a folder that may not exist.

Meanwhile the Dashboard lists that same child as a normal entry contributing **0 libraries**, with
no indication anything is wrong, and the resolver quietly behaves as though it were never linked.

This is the same disease this session has been cataloguing everywhere else: **one answer standing in
for several different conditions.** It is not a missing feature; it is a *lost distinction*.

---

## 3. What Constellation can honestly distinguish

Not a wish list — each of these is decidable from facts already on disk, using `fs::metadata`
(which separates "absent" from "could not be read", where `Path::exists()` does not).

| # | condition | how it is known | is it the user's problem? |
|---|---|---|---|
| 1 | **Healthy** | directory + readable manifest | no — say nothing |
| 2 | **Gone** | path lookup returns *not found* | yes, and it is permanent |
| 3 | **Unreachable** | path lookup fails for any *other* reason — permission, offline placeholder, disconnected share | **usually temporary** |
| 4 | **No longer a universe** | directory is there; no manifest | yes, and it is permanent |
| 5 | **Manifest damaged** | manifest is there; unparseable | yes, and **not ours to fix** |

**States 2 and 3 must never be conflated, and that is the single most important line in this paper.**
"Gone" invites removing the link. "Unreachable" is a laptop away from its external drive — and
offering to remove the link there would destroy a correct configuration in response to an unplugged
cable. *Being wrong in that direction is unrecoverable for the user and trivially avoidable for us.*

---

## 4. The principle the options obey

> **The option set is derived from the condition. The app never offers an action that would be wrong
> for the condition it just diagnosed.**

That is the whole design. A fixed menu shown for every failure is the thing to avoid, because it
necessarily contains an option that is wrong for most of the situations it appears in.

| condition | what it says | what it offers | what it must NOT offer |
|---|---|---|---|
| **Gone** | "That folder no longer exists." | Remove the link · Point it somewhere else · Leave it | — |
| **Unreachable** | "Can't reach it right now — it may be on a drive that isn't connected." | Try again · Leave it | **Remove the link** — the configuration is probably correct |
| **No longer a universe** | "The folder is there, but it isn't a universe any more." | Remove the link · Point it somewhere else · Leave it | — |
| **Manifest damaged** | "Its details file can't be read." | Leave it · Remove the link | **Repair it** — see §5 |

**"Leave it" is always present, and is always the default.** A broken link is not an emergency and
the user is allowed to decide later. Nothing here ever acts on its own.

**"Point it somewhere else"** exists because the commonest real cause is a folder that *moved*, and
today the only path back is to remove the link and add it again — losing the user's place for a
problem that was a rename.

---

## 5. What this function refuses to do, and why

- **It never repairs another universe's files.** Boss ruling 2 (2026-08-17) is that the parent never
  writes into a universe it does not own. A damaged manifest belongs to that universe; offering to
  rewrite it here would be this app deciding what another universe's identity is. Say what is wrong,
  point at the file, stop.
- **It never removes a link on its own** — not on a timer, not "helpfully", not at boot. A link is
  the user's statement about their own knowledge; only they retract it.
- **It never treats "unreachable" as "gone."** §3.
- **It never blocks startup.** This is information and an offer, never a gate.

---

## 6. Where it belongs — complement, do not duplicate

Constellation **already has** a surface for this: a status-bar **warning badge** (a triangle with a
count) that opens a popup listing each unavailable child universe with a reason
(`+layout.svelte`, fed by `federation_get_warnings`). **This concept is not a new panel.** It is:

1. **A better reason string** — the popup already shows one verbatim; today it says
   `"search.db missing"` for four different conditions. The distinction in §3 is what that string
   should carry.
2. **Actions in that popup**, per §4 — it currently offers none, so a user who reads it learns that
   something is wrong and has nowhere to go.
3. **Honesty in the Dashboard's child-universe list**, which today renders a broken child
   indistinguishably from a healthy one contributing nothing.

**On the MIG-108 unification proposal specifically** — the surface Boss decision 3 arose from — the
concept says the plan should *state what it found and continue*, not refuse: a dangling link is a
fact about the federation, not a reason to withhold a proposal about this universe's own libraries.

---

## 7. What this paper does NOT settle

Named explicitly so nothing here is mistaken for a decision already taken:

- **Scope.** Whether v1 is only the reason string (cheap, honest, no new UI), or the reason string
  plus actions. **Recommend the reason string first** — it is the part that is currently *wrong*,
  as opposed to merely absent.
- **Whether "Point it somewhere else" is v1 or later.** It needs a folder picker and a re-link path,
  and it is the only option here that writes.
- **Whether an unregistered-but-present child is a condition at all.** Being federated and being in
  the switcher's registry are different axes, and it is not established that a user would consider
  the combination broken. **Not verified — do not build on an assumption about it.**
- **The exact wording**, which is a translation surface across 15 locales and should be drafted once
  the conditions are agreed.

---

## 8. The one-line test for anything built from this

> *Could the user, reading only what the app said, tell which of the five conditions they are in —
> and is every button in front of them one they would not regret pressing?*

If the answer to either half is no, the concept has not been implemented, whatever has been built.
