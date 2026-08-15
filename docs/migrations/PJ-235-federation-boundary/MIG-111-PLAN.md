# MIG-111 — Full Cross-Universe Operations · PLAN (Phase 2)

**2026-08-12** · For Boss approval. Nothing builds before it.

> **The concept, one line (R37):** *Sovereignty with seamlessness — each corpus keeps its own
> truth; the mind over them is one.*

**Inputs, all banked in this directory:** the Architect doc + 196 KB evidence · the
Boss-mandated Concept Panel (**5/5 VALIDATED**, 37 binding requirements R1–R37, 3 conflicts
C1–C3) · the WA#5 prior-art sweep (no shipping PKM product does this; DEVONthink and Tana each
solve half) · the engineering prior-art (SQLite ATTACH/backup-API semantics, lock-file
protocols, IMAP-MOVE-style two-phase transfer) · two adversarial attacks on the draft plans
(**both SOUND-WITH-AMENDMENTS** — every amendment folded in below, notably: the vocabulary
threading must cover the maintenance *computation* functions, not just the gates, and the
harness must diff aggregate VALUES (H1); owner resolution must intersect the parent-walk with
the federation tree, fail-closed (H2); the Windows lock protocol must keep owner info readable
under a mandatory lock (H4); child-DB preparation never on the debounced save (B3); routing
ships before any door (B4)).

---

## Phase 0 — FOUNDATIONS (no user-visible change; each step one commit)

*The panel's hard preconditions: nothing after Phase 0 starts until every item here is green.*

| step | what | binds | verification |
|---|---|---|---|
| 0.1 | **Ban `fs::copy` of a live-WAL DB** — `federation/migrate.rs` backup via SQLite backup API / checkpoint-TRUNCATE; restore never over a live handle. Runs on TODAY's boot path — fixed first | R11 | red→green test: backup under concurrent writer bit-identical; restore refuses on open handle |
| 0.2 | **The per-universe owner lock** — `LockFileEx` (Windows) / `flock` (macOS) behind `#[cfg]`; lock range separate from a readable owner-info region so the refusal message can always say WHO (attack H4); heartbeat + stale-lock recovery; NFC/NFD- and case-aware path identity; covers search.db, the earned ledger, review-pulse.json, registry JSONs. The WAL `BEGIN EXCLUSIVE` probe is retired from this duty | R5 | two real processes: idle-holder correctly detected (the case the old probe missed); stale lock recovered; macOS arm compiles |
| 0.3 | **`link_life` ledger cross-process lock** (its `FILE_LOCK` is process-local today) | R5, cond 5 | two-process append-vs-compact test: no line lost, no fold-back reversal |
| 0.4 | **The five unguarded writers onto the boundary** (`ensure_cid_cn_cmd`, `sources_set_manual`, `write_conflict_sidecar`, `update_base_columns`/`update_base_order`) | R1 | each refuses a foreign path; suite red if the guard is deleted (wiring tests) |
| 0.5 | **Boss gate — the seven rulings** (see §Decisions below). Phase 1 does not start until ruled | R31 R35 R36 C1 C2 C3 PJ-224 | recorded in the ledger, item by item |

## Phase 1 — THE ROUTER (seamless editing of linked notes; the ad-hoc class ends here)

| step | what | binds | verification |
|---|---|---|---|
| 1.1 | **`resolve_owner(path)`** — universe roots enumerated from the federation tree itself (`universe.json` children, recursive — never from library lists); longest-match with the nested-child check (attack H3); parent-walk fallback INTERSECTED with {active ∪ federation roots} — an unlinked universe on disk is Err, fail-closed (attack H2) | R2 | unit + routing assertions incl. nested-cUniverse-under-root; unlinked universe refused |
| 1.2 | **The routed context pool** — lazy child-DB open (never boot; prep at link-time or first-foreign-OPEN with the progress strip, never the debounced save — attack B3, R10); schema gate, per-connection tokenizer, **per-universe link vocabulary threaded through trigger DDL, the `index_note` parse chain, AND the maintenance computation functions** (`maintain_incoming_after_save`, the sky write-time maintenance, the rank-CASE/IN-list generators — attack H1); routed opens never write fingerprint stamps (R4) | R3 R4 R10 | **the H1 harness**: one note indexed under two differing vocabularies — aggregate VALUES diffed, not just rows; red→green |
| 1.3 | **Kill Class D** — `constellation_search_reindex` stops trusting the frontend's `library_name`; owner resolution Rust-side at all 22 sites; **post-write attribution assertion** (row landed in the DB whose root owns the path), surfaced to the user on failure, never a log line | R2 | assertion test; the 22 register sites enumerated as router callers in the commit |
| 1.4 | **Routed operations wave 1** — edit/save, tag, property, task toggle on a linked note, seamlessly; lock probed at note-open/first-edit-intent (R6): a locked universe's note opens read-only with the C2-ruled presentation; typed input never discarded | R6 R9 | Editor-Surface Gate checklist on a FEDERATED note; keystroke latency indistinguishable from an own note (R33, measured) |
| 1.5 | **Frontend identity, whole-ecosystem in one pass** — the Place Line (universe › library, silence means home, planet mark means elsewhere — R15); identity on every co-mingled row per the C3 ruling (R16); one mark one meaning (R17); `--cuniverse-accent` token via Style Setter (R18); identity keyed (universe, library), never display-name (R19); pickers re-list the umbrella grouped under universe header rows (R21); creation defaults to the active universe, no sticky cross-boundary destination (R20); the Boss-ruled naming applied ×15 locales, RTL-verified (R31 R32) | R15–R21 R31 R32 | ui-inspector pass on every named surface; i18n parity gate |
| 1.6 | Interim guards for wave-1 ops dissolve (refusals for not-yet-shipped ops remain and now route the user to what exists — R25) | R25 | no verb without a door: sweep of offered-verbs-vs-doors |

*Gate: measurement on the 7,600+-note corpus (boot, typing, routed-write latency, first-open
cost, child WAL growth) — any regression blocks (R33). Boss journey via
tutorial-auditor → ui-inspector (R34). Diff-scoped inspection per build.*

## Phase 2 — THE TRANSFER ENGINE (move & copy across universes)

*Entry: Phase 1 green + the R35 ruling (whether PJ-262 ships first — the disk layer would
structurally shrink this phase's riskiest machinery).*

| step | what | binds | verification |
|---|---|---|---|
| 2.1 | **The journaled two-phase move**: payload durable in the DESTINATION root before the fs move; replay ordered before boot reconcile / cold-start auto-index on BOTH sides; the earned-data census IS the cargo manifest; weight recomputed never copied; `created` carried; on-disk companions travel; in-transaction aggregate verification | R7 | **every crash window enumerated and red→green** before the door opens; a crash-resumed transfer announces completion |
| 2.2 | **cid collision**: check-first; on collision re-mint AND re-key the travelling earned rows in ONE transaction — never sever; the inbound `target_cid_cn` rewrite specified as its own cascade | R8 | collision harness red→green |
| 2.3 | **Receipt + genuine Undo** — the transfer engine runs in reverse; ceremonies per the C1 ruling | R22 | undo round-trip test: byte- and row-identical |
| 2.4 | Trash follows the owner (`<owner root>/.trash`, listing/restore across roots); earned companions follow; second screen verified on moved notes | R14 | whole-ecosystem sweep of owner-following surfaces |

## Phase 3 — FULL OPERATIONS + LINK CONTINUITY

create-in / rename-in / delete-in linked universes; **the cross-universe rename cascade**:
every referrer in every reachable universe healed quietly, unreachable referrers become
durable pending-heals completing on next reachability, the receipt states both counts (R23 —
the acceptance test of the whole migration); unlink = dormancy not death, re-link = Renewal
(R29); coverage honesty on every federated surface ("2 of 3 universes" — R24); lifecycle math
crossing-invariant (R30). PJ-253's case-fold ruling lands here with the cascade.

## Phase 4 — THE DIAGNOSTIC UMBRELLA

Typed-link queries, contradicts/tension, dormancy analysis federate by default; universe
scoping is a visible filter, never a silent boundary (R26); provenance visible AND queryable
(R27); vocabulary intelligibility ×15 across universes, cross-edge behaviour for
universe-custom types specified before build (R28); the plain search box per the PJ-224
ruling. Concept-paper 25 amended; the repeals executed in text (R36); whole-app inspection
sweep at close (R33); orientation + manual ×15.

---

## Decisions required at the §0.5 gate

1. **Approve this Plan** (Plan approval = build approval; stops only at Boss-test clauses and genuine surprises).
2. **C1 — ceremonies:** Undo-replaces-confirmation (UX/AD position), or keep per-op confirmations? *Recommended: Undo.*
3. **C2 — locked universe:** persistent quiet indicator on the read-only note, or transient message only? *Recommended: persistent quiet line — it prevents the confusion, and it is passive.*
4. **C3 — tab identity:** planet mark only (AD), or mark + universe name (UX)? *May be delegated to the Art Director & team per 2026-07-10.*
5. **R31 — the user-facing name**, once, ×15 languages: "Linked Universe" or "child Universe"? *Recommended: Linked Universe.*
6. **R35 — sequencing:** ship PJ-262 (the Living-Link disk layer) before Phase 2? *Honest trade: it delays the move door but structurally shrinks the riskiest machinery — the earned cargo becomes files that simply travel.*
7. **R36 — the repeals table** (Architect §5), ratified item by item.
8. **PJ-224** — does the plain search box span the umbrella by default? *The diagnostic surfaces will regardless (R26).*
