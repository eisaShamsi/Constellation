

==========================================================================================
## CHAIR :: inspector
==========================================================================================

All evidence read — the Architect doc, the Option A/B adversarial verdicts, the 22-site write register, the earned-data census, and the Charter taxonomy. Composing the chair's verdict.

---

# SAFETY INSPECTOR — CONCEPT VALIDATION: "Universe of Universes" / the Universe Router

**Chair:** Constellation Safety & Integrity Audit (docs/Constellation-Safety-Audit-CHARTER.md). Enemy: the silent app-killer.
**Evidence read this session:** docs/migrations/PJ-235-federation-boundary/MIG-111-ARCHITECT.md (full), MIG-111-ARCHITECT-EVIDENCE.md (write-site register §2, operation inventory §3, earned-data census, cross-process risk map §(4), Option A and Option B adversarial verdicts), Charter taxonomy §1–8.

## VERDICT: VALIDATED-WITH-REQUIREMENTS

## Reasoning

**1. The status quo is not the safe baseline — it is the most dangerous state.** The verified write-site register shows 22 sites where the current "read-only federation" already crosses universes with broken bookkeeping: delete a linked note and its rows live forever in a DB the purge never reaches; toggle a task and both indexes go silently stale; import into a linked library and the files are indexed nowhere; mark a foreign note reviewed and the mark lands in the WRONG universe's pulse file. Every one of these is Charter class 3 (false success) or class 4 (index↔disk divergence) — my classes. The concept's central claim — "what made the old crossings app-killers was never the crossing, it was the SILENT crossing with broken bookkeeping" — is not rhetoric; it is confirmed by the register. A half-open door is worse than an open door with a doorman. Rejecting this concept would preserve a documented silent-failure surface, not prevent one.

**2. The Router is the structurally correct shape for this chair.** One resolver, one choke point where locks, vocabulary, schema gates, and attribution are enforced, replaces 22+ per-site guards that provably drift (the LL-023 class; the register itself found five writers that today's guards never reached). This is the cure this codebase keeps arriving at — replace the promise a caller must remember with a structure that cannot forget — applied at the write boundary. Killing the Class-D frontend-supplied `library_name` trust at the Rust root is, by itself, a safety win independent of federation.

**3. But an invisible layer's own failures are invisible — that is the concept's structural risk.** If the Router resolves the wrong owner, the bookkeeping lands in the wrong DB with no error: false success plus divergence, the exact taxonomy-top classes, now manufactured by the safety layer itself. And the adversarial pass proved this is not hypothetical: routed writes with the wrong vocabulary silently skip incoming-aggregate maintenance at the fingerprint gates (`incoming_links_backfill.rs:49`, `links_backfill.rs:99`) — a silent-divergence hole the design's own enumeration missed. Seamlessness is certifiable only if wrong-routing is made impossible or self-detecting (Req. 2, 3).

**4. Seamlessness removes pause points — so the certification line is: quiet SUCCESS is acceptable; quiet FAILURE never is.** Confirmation ceremonies are a weak safety control anyway (users click through them); their removal costs little IF every refusal, partial state, and incomplete transfer becomes loud instead. The concept as ruled ("no confirmation ceremonies") is compatible with my chair only under that asymmetry, stated as a binding requirement (Req. 7).

**5. Earned data crossing databases is the highest-stakes cargo in the app, and physics is against atomicity.** `search.db` is the ONLY home of traversal counts, confidence, status, `created`, review state, state history, shape history, rename aliases (census verified; `search.rs:11543-11566` states it in-code). WAL means a cross-DB transfer can never be one transaction. The concept's answer — journaled, resumable, crash-surviving two-phase transfer — is the correct and only answer, but the adversarial pass found the journal as first specced would strand the earned half in ZERO recoverable places after a mid-transfer crash (destination cold-start re-indexes at hypothesis defaults; source boot reconcile deletes the rows the journal still needs). The concept survives; that journal placement does not (Req. 5).

**6. The three pre-existing cross-process hazards are disqualifying if not fixed FIRST.** (a) The `link_life` ledger lock is a process-local mutex — cross-process append-vs-compaction silently reverses user decisions while every step logs success (the Slice-7 bug shape, across the process boundary). (b) The two-instance probe (`BEGIN EXCLUSIVE`) is a false NEGATIVE in the routine case — it cannot see an idle second instance, so the defense as designed admits the write precisely when it must refuse. (c) `federation/migrate.rs` backs up a live-WAL DB with `fs::copy` (no `-wal` sibling) and restores over a live file — a corruption vector that can roll back committed writes. All three run on TODAY's paths; writes crossing universes converts them from latent to routine. The Architect names all three and conditions on them — correct — and I convert those conditions to certification gates (Req. 4, 9).

**7. Why not REJECTED, and why not clean VALIDATED.** Not rejected: the concept reduces the confirmed silent-failure surface relative to today, and the Router is the only shape reviewed (Option A) where portability and full operations are the same property rather than a trade-off — Option B's two-copy sync contract adds two app-killer-class cross-process holes of its own; Option C fails structurally. Not cleanly validated: the concept's headline virtue (invisibility) is exactly the property my Charter exists to distrust, and the adversarial evidence shows the first draft of the design already contained one silent-divergence hole and one non-functioning defense. The concept holds the line only with the following made binding.

## REQUIREMENTS (each binding on the Plan)

1. **Single mandatory choke point, enforced by structure, with no grandfathered silent crossings.** No write may reach any universe's DB, earned ledger, `review-pulse.json`, `.trash`, or registry except through the Router — enforced by construction (routed connections and vocabulary snapshots obtainable only from the Router; the global registry accessor unable to satisfy a routed path's type), never by per-site discipline. The five writers currently outside ANY boundary (`ensure_cid_cn_cmd`, `sources_set_manual`, `write_conflict_sidecar`, `update_base_columns`/`update_base_order`) are brought onto the boundary before any door opens.

2. **Wrong-owner attribution is an app-killer: make it impossible or self-detecting.** Owner resolution is Rust-side only (kills Class D at the root, all 22 call sites), resolves by universe ROOT — not registered-library roots (the documented `libraries.rs:320-323` blind spot) — and every routed write carries a post-write attribution assertion (row landed in the DB whose root owns the path). A failed assertion surfaces to the user; it is never a log line.

3. **Per-universe context is COMPLETE, and proven so.** Link-vocabulary threading reaches trigger DDL, the `index_note` parse chain, AND the `is_built`/`stored_vocab_fingerprint` gates — with a red→green harness test indexing one note under two differing vocabularies and diffing maintenance outcomes. Routed opens NEVER write vocabulary-fingerprint stamps (only the owner's completed boot backfill stamps — otherwise stale aggregates are blessed permanently, the MIG-056 marker shape). Cached child registries are staleness-checked per routed write.

4. **Cross-instance policy is refusal, never racing — via a mechanism that actually works.** An OS-level per-universe lock file (owner lock held by the instance with the universe active; routed writers acquire share/deny-write or refuse with a plain message). The WAL `BEGIN EXCLUSIVE` probe is certified insufficient (false negative on an idle instance) and may not carry this defense. The lock protocol must cover the earned ledger (append-vs-compaction), `review-pulse.json`, and the registry JSONs — not just `search.db`.

5. **The earned-data transfer: journaled two-phase, resumable at every seam, payload durable in the DESTINATION root BEFORE the fs move, and journal replay ordered before boot reconcile / cold-start auto-index on BOTH sides.** The payload is the evidence file's earned-data census — that census IS the cargo manifest and the Plan may not re-derive it from memory. Weight recomputed, never copied; `created` carried (it exists nowhere else); on-disk companions (`earned.jsonl`, `note-history.jsonl`, `review-pulse.json` with paths rewritten) travel with it; transfer verified by in-transaction aggregate checks (the mig108 precedent). Every crash window enumerated and red→green in the harness before the door opens (Reproduce-First).

6. **cid collision: check-first, re-mint-and-re-key in ONE transaction, never sever.** Journal replay keyed (cid, destination path), refuse-and-report on duplicate cids — the `link_life_restore` skip-on-ambiguity rule, never distribute-on-guess. The inbound `target_cid_cn` rewrite after a re-mint is itself a cascade and must be specified, not implied.

7. **The seamlessness boundary: quiet success, loud failure.** Certified for the success path only. Every refusal (lock held elsewhere, child not ready, replay pending, collision ambiguity), every partial state, every incomplete transfer surfaces to the user in plain language at the moment it happens. No routed write may fail silently, be silently queued, or silently fall back to the active universe's DB. The planet mark may be quiet; errors may not be.

8. **Nothing heavy on the save tail.** First-routed-write child preparation (schema migrate, FTS rebuild, backup) runs at link-time or first-foreign-open with visible progress — never on the debounced save (and never colliding with the PJ-103 5s close-flush cap). Until the child is ready, writes refuse visibly per Req. 7.

9. **`fs::copy` of a live-WAL database is banned, before any door opens.** Backup via the SQLite backup API (or checkpoint-TRUNCATE then copy); restore never lands over a live handle. This is fixed first because it already runs on today's boot path.

10. **Automatic writes still never cross — ruled row by row.** A Boss-ruled write-site → intent table covering all 22+ sites, with the ambiguous rows ruled explicitly: `ensure_cid_cn` on foreign-note open (automatic identity injection into another universe's file — today's live silent crossing), watcher-adopt reindex, `sources_set_manual`. MIG-065 §J survives as the refusal mechanism for the automatic class.

11. **Whole-Ecosystem completeness at certification.** Trash listing/restore learns the owner's `.trash` (or a deleted foreign note is unrestorable from anywhere the user looks); second-screen/cross-window sync verified on routed notes; the Editor-Surface Gate checklist passes on federated notes exactly as on own notes; and the directional-federation limit (universes that link the child but are not in the active view stay unhealed by a rename cascade) is stated honestly in UI and docs — never marketed as closed.

12. **Standing inspection cadence + measurement gates every wave.** Diff-scoped `safety-inspection` on every build of every wave; whole-app sweep at the migration close; measured on the 7,600+-note corpus before each commit: boot time, typing latency, steady-state routed-write latency, first-write pool-open cost, child WAL growth under sustained parent writes. Any regression blocks the wave.

13. **The PJ-262 sequencing question goes to the Boss before the transfer engine is built.** Earned data being DB-only is what makes the transfer module app-killer-class; if the Living Link disk layer ships first, the riskiest module shrinks structurally. Present the option — do not decide it silently in either direction.

**Certification statement:** with Requirements 1–10 as blocking gates and 11–13 in the Plan, the Universe Router concept does not merely hold the line against the silent-failure class — it shrinks today's confirmed surface, because it replaces 22 drifting per-site guards with one structurally-enforced boundary and converts every known silent crossing into either a correct routed operation or a loud refusal. Without Requirements 2, 3, 4, 5, and 7, I would reject it: an invisible layer that can misroute, skip maintenance, or strand earned data quietly is the app-killer class given an architecture.

— Safety Inspector, Constellation Safety & Integrity Audit · 2026-08-12

==========================================================================================
## CHAIR :: auditor
==========================================================================================

All spot-checks confirm the evidence file's claims: the `REGISTRY` OnceLock is process-global and active-only (`link_types.rs:350`); `is_stamped` compares the child's stored fingerprint against the ACTIVE universe's registry snapshot (`incoming_links_backfill.rs:49`); and the `is_cuniverse_open_elsewhere` probe is `BEGIN EXCLUSIVE; ROLLBACK` — which in WAL mode does not detect an idle reader instance (the in-code comment even documents the opposite failure mode). The Architect doc's territory is honest. I can now give the verdict.

VERDICT: **VALIDATED-WITH-REQUIREMENTS**

REASONING — the concept tested against Constellation's own laws (all read from CLAUDE.md at E:\مشاريع كلاود\Constellation\CLAUDE.md; Architect doc + evidence at docs/migrations/PJ-235-federation-boundary/, load-bearing code claims spot-verified in src-tauri/src/link_types.rs:350, incoming_links_backfill.rs:49, federation/migrate.rs:191-208):

**Concept Before Function — the horse is sound.** One-line purpose test: *"One umbrella, full agency: any operation the user commands on any note in the federation executes correctly, with the bookkeeping done in the note's own universe."* That is a clear, sound concept, and it is the designer's own stated intent (the Boss's 2026-08-12 ruling; the 2026-07-05 "It is ONE universe" ruling it completes). The read-only contract was an implementation assumption (MIG-056/MIG-065 §J), not the design. The Router's own horse also states cleanly: *"resolve the owner, supply the owner's complete context, so operations stay universe-blind."* Both pass.

**Does the Router genuinely end the ad-hoc class?** Yes — with one honesty correction and one structural demand. The guard-becomes-router move (`require_own_library` → `route_write`) is exactly this codebase's repeated cure: one answer, many callers. The Class-D kill (Rust-side attribution replacing frontend-supplied `library_name`) fixes 22 call sites with zero per-site changes — strictly better than 22 patches, and the proof the shape is right. The complexity that remains after the Router (vocabulary threading, the two-phase transfer journal) is *essential* domain complexity, not incidental dispatch complexity — no design evades it, Options B and C just hide it worse. The honesty correction: "the operation never knows universes exist" is true for the **single-owner class** (edit, tag, property, review, delete, link-author-side). It is *not* true for the two-party class — cross-universe move, the multi-universe rename cascade, the link-type palette on a foreign note — which are inherently universe-aware and journaled. The concept survives this; the claim must be scoped honestly (Requirement 2).

**Rule 8 Write-Time Derivation — passes only via the conditions.** Boot untouched (pool is lazy, never opened on boot — verified stated in §4 and Option A §3). But the adversarial pass proved the design's own first draft violated WTD silently: the backfill fingerprint gates compare child stamps against the **active** registry (confirmed in source), so a routed write would skip incoming-aggregate maintenance with no error — the exact divergence class this migration exists to kill, inside its own recommended option. WTD holds per-universe only if the registry threading reaches the gates (Requirement 3) and pool-open never stamps fingerprints (Requirement 4).

**File Over App / Local-First — Option A is the only shape where the law is preserved *by construction*.** No child row ever lives in the parent; copy the folder, get everything; unlink, lose nothing. Two real threats found and must be bound: the coordinator journal as first-specced breaks self-containment across a crash (payload must be durable in the destination root before the fs move), and `ensure_cid_cn` writing frontmatter into a foreign file on mere open is a *silent* file modification — the precise class the law forbids (Requirements 5, 7).

**Performance Rules — passes with one cliff bound.** Steady-state routed write = own write + a map lookup; boot zero change; reads zero change; nothing touches the keystroke path. The one violation found: pool-open can trigger a §C schema migrate (seconds-to-minutes on a large child) on the debounced *save tail*, colliding with PJ-103's 5s close-flush cap (Requirement 6).

**Language-First — the per-universe vocabulary threading IS this law applied to federation.** A child's link vocabulary (which may be in any language) parsing under the parent's registry is a single-vocabulary assumption baked into the write path — the same sin as a single-language assumption. The Router's owner-context supply is the correct cure.

**Cross-Platform — the lock file is the one place the concept currently speaks Windows.** "Acquired share/deny-write" is Win32 semantics; macOS offers POSIX advisory locks with different guarantees, and stale-lock recovery after a crash differs per platform. NFD/NFC also touches the router's longest-prefix ownership match (Requirement 8).

**Constraint as Design / Form-Aligns-To-Purpose — one tension to surface, not paper over.** The concept brief says "no confirmation ceremonies"; the Architect doc's own §6.2 reserves "planet icon + confirmation" as a Phase-2 **Boss decision**, and the 2026-08-10 amended ruling makes the clearly-marked affordance load-bearing. Seamlessness rightly applies to the *mechanism*; the *crossing gesture* must remain visible and deliberate, because "what made the old crossings app-killers was the SILENT crossing" — the concept's own §0 says so. The concept must not pre-empt the Boss's §6.2 ruling (Requirement 9).

**Two-instance safety — concept valid only with a real protocol.** The existing probe is proven blind to the routine case (idle second instance in WAL). "Refuse, never last-writer-wins" is the right posture; the mechanism must be rebuilt (Requirement 10).

REQUIREMENTS (each binding on the Plan):

1. **Router as the only door, structurally.** A write connection to any universe DB (active or foreign) on a routed operation path must be obtainable *only* through `route_write` — enforce with a handle/snapshot type the global accessors cannot satisfy, so a new write site cannot compile without declaring `WriteIntent`. The known unguarded writers (`sources_set_manual`, `update_base_columns/order`, `ensure_cid_cn_cmd`) come onto the router in Phase 1, before any door opens. Enumeration (the 22-site register) is the audit; the type system is the guarantee.
2. **Scope the concept statement honestly in the Plan's opening.** Universe-blindness covers the single-owner operation class; move, cross-universe rename cascade, and foreign-note link-palette are explicitly two-party, journaled, and universe-aware. The rename cascade's reach is the *visible federation set only* (directional) — stated, never marketed as "closed."
3. **Registry threading provably complete before any foreign index write ships** — through `index_note`'s parse chain, trigger-DDL generation, AND the fingerprint gates (`links_backfill.rs:99`, `incoming_links_backfill.rs:49`), with a red→green test indexing one note under two differing vocabularies and diffing `note_links` plus maintenance outcomes. Phase 1, gated. Plus a per-write staleness check on the cached child registry (stat `link-types.json`).
4. **Pool-open may regenerate child triggers from the child's registry but must NEVER write vocab-fingerprint stamps** — only the owner's completed backfill stamps. (The MIG-056 marker-clearing lesson: the stamp is what schedules the child's own boot-time repair; blessing it from the parent makes stale data permanent.)
5. **Transfer journal:** full earned payload durable in the *destination root's* own breadcrumb before the fs move; replay ordered before boot reconcile/de-index on both sides; replay keyed on (cid_cn, exact destination path) with refuse-and-report on duplicate cids; cid collision re-mints AND re-keys travelling earned rows in the same transaction — never severs. Every crash window red→green in the harness before the door opens (Reproduce-First).
6. **Pool open (including any §C child migrate) off the save tail** — run at link-time or first foreign-tab-open with progress UI; saves refuse visibly until the child is ready; child-DB backup via SQLite backup API or after checkpoint-TRUNCATE, never bare `fs::copy` beside a live `-wal`.
7. **Write-site → intent table Boss-ruled in the Plan**, with `ensure_cid_cn`-on-foreign-open and watcher-adopt-reindex ruled explicitly. Automatic writes never cross on their own (MIG-065 §J survives as mechanism); no automatic frontmatter injection into a foreign file without a ruling.
8. **Cross-platform lock protocol designed once, for both OSes:** per-universe owner lock with platform-appropriate primitives (`LockFileEx`/share-mode on Windows, `flock`/`fcntl` on macOS behind `#[cfg]`), heartbeat + stale-lock recovery (a crashed instance must never permanently brick routed writes), and the router's longest-prefix ownership match normalization-aware (NFC/NFD) and case-sensitivity-aware. Flag every macOS implication in the Plan, never silently.
9. **The crossing stays visible at the point of choice.** Planet mark + universe name on every federated destination and every federated note identity surface; whether any operation additionally warrants a confirmation is the Boss's §6.2 decision — the Plan presents it as a decision, and "no confirmation ceremonies" is not assumed. PJ-224 (search-box federation) likewise remains the Boss's gated ruling — suggested, not assumed.
10. **Replace the write-time lock probe** — the WAL `BEGIN EXCLUSIVE` probe cannot carry the two-instance defense (false negative on an idle instance, verified in source). Real cross-process presence signal; routed writes refuse with a plain message while the child is active elsewhere; the single-instance/cross-process-signal follow-up filed as its own PJ. Refusal, never merge semantics, pending the Boss's §6.4 confirmation.
11. **Trash, restore, and every sibling surface follow the owner in the same pass** (Whole-Ecosystem Fix Law): owner's `<root>/.trash`, trash listing/restore across owner roots, owner's `earned.jsonl`/`note-history.jsonl`/review-pulse.json, owner's `libraries.json` on library rename. No surface left reading the active root for a routed note.
12. **Measure before commit on the 7,600+ note universe** (CLAUDE.md hard constraint): boot time unchanged, typing latency unchanged, steady-state routed-write latency, first-write pool-open cost, child WAL growth under sustained parent writes. **Surface the PJ-262 sequencing choice to the Boss before the transfer engine is planned** — the Living-Link disk layer shipping first shrinks the transfer engine's riskiest module and the earned-data exposure both.
13. **The repeals table (§5) is confirmed by the Boss explicitly, item by item, as Phase 2's first act** — the concept supersedes three standing contracts (MIG-056 read-only, concept-paper 25 §3/§10, MIG-065 §J-as-contract) and amends one ruling (2026-08-10); a concept that silently overwrites prior Boss rulings would itself violate the house's constitution, however sound its engineering.

Bottom line: the horse is sound and it is the Boss's own horse — cUniverses were designed for full agency, and the Router (Option A) is the only shape found where "full operations between universes" and "each universe remains a self-contained, portable unit" are the same property rather than a trade-off. It genuinely ends the ad-hoc refusal class the way this codebase has always ended such classes: one answer, many callers. It is not a courtesy pass — the concept as briefed contained one materially false claim (backfill-gate threading), one defense that does not work (the WAL probe), and one Windows-only assumption (the lock), all confirmed in source; the thirteen requirements above are what make the concept true, and each is binding on the Plan.

==========================================================================================
## CHAIR :: ux
==========================================================================================

VERDICT: **VALIDATED-WITH-REQUIREMENTS**

---

## REASONING (UX Chair)

**The concept is correct from the user's seat — and the strongest argument for it is the current state, which I verified in the evidence file and source.** Today the interface already *behaves* seamlessly while the system underneath does not: `OpenTab` carries no universe field, the editor mounts pass no `readOnly`, and typing in a linked-universe note already saves to the other universe's disk (frontend-model map, `NoteEditor.svelte:284-339`); context menus offer Rename/Move/Delete on federated notes on ~26 list surfaces with no gate; while pickers and the batch bar silently exclude federated destinations. That is a border the user discovers only by falling over it — the worst possible geometry: invisible where it should whisper, and enforced at random depths where it should not exist. The Router concept — operations just work, bookkeeping lands in the owner's database, identity shown quietly — replaces an incoherent boundary with an honest one. For a knowledge worker holding ~7,800 notes across 3 universes, "one umbrella" is the only mental model that matches how they think; nobody thinks in databases.

**The Boss's prohibition on border-control ceremonies is also correct craft, not just preference.** Confirmation dialogs do not prevent cross-universe mistakes; they train click-through. The proven pattern in every mature "one space, many places" interface is the same pair: **quiet passive identity + reversibility.** Unified inboxes (Apple Mail, Gmail) show account identity as a secondary chip/color on each row and ask nothing — except at *compose time*, where the From-account is made explicit, because creation is the one act where place must be chosen. File managers (Finder, Explorer) run identical verbs on network and local volumes, mark the place with a subdued icon and an always-on path bar, and surface network states (disconnected, busy) passively. Gmail's archive has no "are you sure" — it has "Moved. Undo." These precedents map one-to-one onto this design, and the requirements below bind the Plan to them.

**Where seamlessness will confuse, verified concretely, if the Plan does not close it:**

1. **"Where does this note live?" is currently unanswerable at a glance.** The tab shows only `libraryName` + color; the Quick Switcher row shows only `libraryName` (`QuickSwitcher.svelte:201`); backlinks and sky edges are name-keyed, so a same-named note in a linked universe *attracts the other note's backlinks* with no visible cue (identity-links map §4). Two universes each holding a "Projects" library are indistinguishable everywhere notes co-mingle. Once editing is sanctioned, this stops being cosmetic — it is how the accidental cross-universe edit happens.
2. **"Which universe am I creating into?"** Re-listing linked universes in pickers (Phase-2 decision 2) reopens creation targeting. The compose-account trap is sticky defaults: a "last-used destination" carrying across the boundary silently redirects tomorrow's note into the work corpus.
3. **"Why did search results change?"** Federated search is scatter-gather over attached universes; an unmounted drive or a locked universe silently shrinks coverage. The user experiences it as data loss.
4. **The refusal-at-save trap.** The two-instance lock (Condition 2: "refuse, never last-writer-wins") is right — but if refusal lands at *save time*, the user has already typed a paragraph the system now holds hostage. Refusal timing is a UX property, not a locking detail.
5. **Rename healing must not swing to the other extreme.** Today cross-universe inbound links break silently (identity-links map §3). The fix must be quiet-heal + receipt — not a confirmation ceremony, and not silence when a universe is unreachable and healing is only partial.

None of these reject the concept; all of them define where the one space must still whisper. The whisper principle, stated once: **place is metadata the note always wears, never a question the interface asks.**

---

## REQUIREMENTS (binding on the Plan)

1. **The Place Line.** Every editing surface must answer "where does this note live?" passively and always: an always-visible universe › library trail on the note surface (status-bar/title affordance — Finder path-bar precedent). Foreign universes carry the planet mark; own-universe notes carry nothing (silence means home). No hover-only, no menu-dive.

2. **Identity travels with the note row everywhere notes co-mingle.** Tab strip (requires adding universe identity to tab state — `OpenTab` has no field today), Quick Switcher rows, search results, backlinks/outgoing panels, list surfaces, pickers, and the second screen all render the same quiet mark + universe name at secondary visual weight on foreign notes. Unified-inbox account-chip precedent.

3. **One mark, one meaning, one grammar.** Reuse the existing planet/orbit mark (`LibraryIcon` `kind='cuniverse'`) as the sole cross-universe identity mark, in its neutral identity color — never a warning color, never paired with a confirmation. Universe names localized and RTL-correct (Arabic universe names like كون عيسى must render with correct direction and mark placement per Language-First law).

4. **Creation never crosses by default.** Ctrl+N, quick capture, daily note, and template-create always default to the active universe (ratifies the amended 2026-08-10 ruling). A foreign destination is reachable only by deliberate selection of a planet-marked, universe-grouped entry in the picker; no sticky "last-used destination" may pre-select across the universe boundary — ever.

5. **Refusal happens before input, never after it.** The per-universe lock must resolve at open or first-edit-intent. A note in a universe held by another instance opens read-only with a one-line passive explanation ("This universe is open in another Constellation window"). A save-time refusal after accepted typing is forbidden; if lock acquisition is lazy, failure at first keystroke converts the surface to read-only and preserves whatever was typed in a recoverable buffer.

6. **Undo replaces confirmation.** Every cross-universe operation (move, copy, rename-with-cascade) completes without an "are you sure" ceremony and ends with a quiet receipt carrying Undo. The journaled two-phase transfer engine (Condition 3) must be designed to run in reverse so the receipt's Undo is real. Reversibility is the safety model — this is also the existing law "every link operation must be reversible."

7. **Link continuity is part of the operation's definition.** A cross-universe move or rename heals every referrer in every *reachable* universe quietly; unreachable referrers become a durable pending-heal that completes on next reachability, and the receipt states both ("12 links updated · 3 pending in كون العمل"). Silent breakage (today's behavior) and blocking dialogs are both forbidden outcomes.

8. **Coverage honesty.** Any federated surface — search, Sky, Quick Switcher, backlinks — whose universe set is degraded (unmounted, locked, attach-failed) shows a quiet coverage state at the surface itself ("2 of 3 universes"). Results never silently shrink; the interface answers "why did results change?" before the user asks it.

9. **Same-name truthfulness.** Where cross-universe resolution or aggregation is name-keyed (backlink attraction, sky edges, `library:note` prefix shadowing — identity-links map §4), the Plan must either move those joins to identity-keyed or surface the ambiguity at the point of display. A user must never watch a same-named foreign note absorb another note's backlinks with no visible cue; at minimum Requirement 2's mark makes the colliding rows distinguishable.

10. **Transfer legibility and untouched latency.** Cross-universe moves show status-bar progress when perceptible and are resumable; a crash-resumed transfer announces its completion — the user never finds a half-moved note without explanation. Editing a federated note must be indistinguishable in keystroke latency from an own note (child DBs opened lazily, off the hot path — restating the invariant as a *perceived* requirement).

11. **No verb without a door.** At each shipped wave, every surface either performs the operation end-to-end or visibly routes to the sanctioned door. The current mix — menus offering verbs on federated notes that then hit an unguarded write or a buried refusal — must be eliminated wave-by-wave (Whole-Ecosystem Fix Law applied to affordances). An offered verb that fails is a border discovered by falling over it.

12. **Boss journey validation per wave.** Each wave's Boss test must include the cross-universe daily journeys end-to-end — edit a foreign note, move a note across, rename a cross-linked note, search the umbrella with one universe unavailable, open a second instance and hit the refusal — written per the Testing Instructions Rule, so seamlessness is validated as an experience, not as a set of passing commands.

Files: `E:\مشاريع كلاود\Constellation\docs\migrations\PJ-235-federation-boundary\MIG-111-ARCHITECT.md`, `E:\مشاريع كلاود\Constellation\docs\migrations\PJ-235-federation-boundary\MIG-111-ARCHITECT-EVIDENCE.md` (frontend-model and identity-links maps are the sections this verdict leans on), `E:\مشاريع كلاود\Constellation\src\lib\components\QuickSwitcher.svelte:201` (libraryName-only row identity, verified this session).

==========================================================================================
## CHAIR :: art-director
==========================================================================================

All surfaces verified in source. Delivering the Art Director & Team ruling.

---

**VERDICT: VALIDATED-WITH-REQUIREMENTS**

**REASONING (from the Art Director's chair, evidence read in source this session)**

The visual concept — one space, quiet identity — is not an import; it is already the repo's own visual DNA, built but incomplete. The evidence:

- The planet/orbit mark already exists as quiet identity, not warning: a 12–13px outline glyph in one shared component (`src/lib/components/LibraryIcon.svelte`, kind `cuniverse`), created explicitly under the Whole-Ecosystem Fix Law "so the icon can never drift between surfaces." It already renders in the sidebar (`+layout.svelte:8346`), Dashboard (`DashboardView.svelte:211`), MoveDialog rows (`MoveDialog.svelte:93`, `iconKind` plumbing already accepts `'cuniverse'`), OrgChart and Map nodes.
- The breadcrumb grammar for place already exists: bookmark rows (MIG-092, `+layout.svelte:6529`) render "cUniverse / library / folder…" with the universe segment appearing **only when the note is federated** — absence of the mark is the "home" signal. That is exactly Form-Aligns-To-Purpose: zero noise on own-universe notes, truthful place on linked ones.
- The identity carrier at library level (glyph + name + colour) is consistent across sidebar, tabs (`tab-lib-name` + `--library-color`, `+layout.svelte:8589–8598`), backlinks (`BacklinksPanel.svelte:166`), search (`sh-item-lib`, `SearchHub.svelte:546`).

So the concept passes: extending "quiet identity" one level up the hierarchy is a completion of an existing vocabulary, not a new one. And once writes are legitimate (the Router), warning-styling would *lie* — there is no danger to warn about. "Identity, not warning" is the honest rendering.

But the current vocabulary has five defects that would make the quiet identity either a **lie** or **noise** the moment linked universes become fully operable. These are the binding requirements.

**REQUIREMENTS (each binding on the Plan)**

1. **One concept, one mark.** Today "universe" has two unrelated glyphs: the planet/orbit mark (LibraryIcon) and the six-circle cluster with fixed brand hexes in the status bar (`+layout.svelte:10518`). The planet/orbit mark becomes THE mark of "a linked universe as a place" on every operable surface (sidebar, pickers, tabs, breadcrumb, search rows, backlinks rows, Move dialog). The cluster emblem is reserved solely for the Universe Manager entry point and must never appear beside a note, row, or tab.

2. **Tokenize the universe accent.** `#6366f1` is a hard-coded literal in at least 5 files (`+layout.svelte:8346`, `DashboardView.svelte:211,447-450`, `ConstellationMap.svelte:166`, `OrgChart.svelte:160,1000,1184`). It must become one theme token (e.g. `--cuniverse-accent`) defined in both light and dark palettes, consumed everywhere, and exposed through the Style Setter's existing cuniverse category (the `--ft-cuniverse-*` tokens at `+layout.svelte:11010-11014` are the precedent and the natural home). No literal may survive per-surface.

3. **Identity keyed by place, never by display-name.** `buildLibraryColorMap` (`src/lib/libraries/colors.ts:12`) keys colour by library **name**, index-assigned over the flattened federated list. Under the umbrella this lies twice: (a) two universes each containing "Projects" merge into one identity (same key, same colour, and `openNoteTab`/backlinks resolve colour by name); (b) attaching/detaching a linked universe reshuffles every library's colour (index modulo 10). The Plan must re-key visual identity by (universe, library) with order-stable colour assignment, and reconcile the tab-persisted `libraryColor` (`session.ts:47`) so a stale snapshot cannot show yesterday's colour. A colour that changes meaning is worse than no colour.

4. **The minimum place vocabulary = the existing breadcrumb grammar, applied whole-ecosystem.** Exactly two elements say "where": the planet mark (glyph) and the universe name (text segment), both appearing **only when the note lives in a linked universe**. Own-universe surfaces change nothing — zero added pixels. Apply in the same pass (Whole-Ecosystem Fix Law): NotePane breadcrumb (`NotePane.svelte:1511` gains the universe segment before the library segment), tab (planet mark ~10px before `tab-lib-name`, no text), SearchHub result rows, BacklinksPanel/Outgoing rows, MoveDialog (already plumbed), LibraryPicker, quick-switcher, bookmark rows (already done). No surface may invent a third element (no badges, no tints on the paper, no read-only banner styling — editing a linked note looks exactly like editing a note).

5. **Disclosure by structure, not by ceremony.** Ruling on the Architect doc's §6.2 open question: pickers show the whole umbrella with linked-universe destinations **grouped under their universe's named header row** (planet mark + universe name — the sidebar's existing collapsible grammar at `+layout.svelte:8334-8348`), never flattened into an undifferentiated list where "Projects" could be either universe. Grouping IS the disclosure; per-operation confirmation modals are ceremony and are rejected. The 2026-08-10 LibraryPicker finding ("universe-wide right for resolving, wrong for choosing") is honored visually: choosing surfaces show the boundary as structure; they just no longer refuse it.

6. **One name per language, jargon retired.** English currently ships three user-facing names for one concept — "Child Universe" (`en.json:2313,2493,2746`), "linked universes" (`en.json:2566-2567`), and raw "cUniverse" (`en.json:3523,3525,3614`); Arabic ships two (كون فرعي / الأكوان المرتبطة). The Plan must carry ONE Boss-confirmed user-facing term (his 2026-08-12 ruling says "child Universe (Linked Universe)" — the choice between them is his naming decision, taken once), applied across all 15 locales in the same commit, and the developer jargon "cUniverse" must not appear in any user-visible string, including the federation warning popup.

7. **RTL and bidi carry the identity intact.** The planet mark is direction-neutral — it must NOT be mirrored. Chevrons on universe/library rows must flip per the CLAUDE.md convention (verified gap: no `[dir="rtl"] .v-chev` rule exists in `+layout.svelte` — only `.tab-scroll-arrow` and `.index-return-btn` flip today; this becomes load-bearing when these rows appear in every picker). Every universe-name text segment uses per-segment direction (`dir="auto"` / `detectDir`, the pattern already at `MoveDialog.svelte:97` and `sh-item-lib`), and the breadcrumb separator must remain order-safe in mixed-script paths.

8. **The planet mark stays identity — never state.** Router internals (owner.lock refusal, journaled transfers, cid re-mint) must never surface as persistent decoration on the mark. A refused routed write is a plain transient message; a mid-transfer note shows the standard progress affordance; the federation-unavailable case keeps its separate triangle badge (`+layout.svelte:10510`). One mark, one meaning, or the vocabulary inflates and the quiet is lost.

9. **Verification clause for the visual pass:** the Plan's UI steps are gated per the Test Pipeline law (tutorial-auditor → ui-inspector → Boss), and every claim about the surfaces above must be re-verified against the build at that time — this validation is evidence of the concept, not a substitute for the per-step inspection.

**Summary for the Plan:** the umbrella needs no new visual invention — it needs the existing vocabulary (planet mark, universe segment, library colour) made **consistent** (Req 1, 4), **truthful** (Req 3), **tokenized** (Req 2), **structural instead of ceremonial** (Req 5), and **universal** (Req 6, 7). That is "simple yet powerful" rendered literally: two quiet elements, everywhere, never lying about place.

Key files: `src/lib/components/LibraryIcon.svelte` · `src/lib/libraries/colors.ts` · `src/lib/libraries/session.ts` · `src/routes/+layout.svelte` (sidebar 8290-8404, tabs 8583-8605, status bar 10500-10544, bookmarkLocation 6529-6550) · `src/lib/components/{MoveDialog,LibraryPicker,NotePane,SearchHub,BacklinksPanel}.svelte` · `src/lib/i18n/*.json` · `docs/migrations/PJ-235-federation-boundary/MIG-111-ARCHITECT.md`

==========================================================================================
## CHAIR :: philosopher
==========================================================================================

**VERDICT: VALIDATED-WITH-REQUIREMENTS**

---

## Reasoning — from the Knowledge-Formulation Chair

**Sources read:** `docs/CONSTELLATION-KNOWLEDGE-FORMULATION.md` (v1.0), `docs/concept-papers/00-Constellation-Core-Concept-Paper.md`, `docs/migrations/PJ-235-federation-boundary/MIG-111-ARCHITECT.md`.

### 1. Which of the Five Acts does Universe-of-Universes serve? Four of them, concretely — and the argument survives honest scrutiny.

The core paper's one test is: *which Act does it serve?* I tested the concept against each Act using what the Architect doc says the code actually does today (§1.6):

- **Connection (Act II) — served, at its largest scale, and the claim is honest.** The prompt asks me to argue or refute "work corpus and personal corpus connecting is Connection at its largest scale." I argue it — but with a sharper point than the slogan: the Act is *already being attempted by users and is currently producing defective vessels*. Today a cross-universe link lives only in the author's DB; the target universe never learns the edge exists; open it standalone and the backlink is absent; rename breaks the inbound link silently (Architect §1.6). By the formulation doc's own definition (§2.1), that is a **dead pointer** — the exact thing a Constellation link is defined NOT to be. So the concept doesn't merely *enable* Connection at corpus scale; it **repairs an Act the system currently performs falsely**. That is the strongest kind of justification the core paper recognizes.
- **Tension (Act III) — served.** The most valuable contradictions in one mind are between corpora: the professional stance in the work universe against the personal conviction in the personal universe. Today that `contradicts` edge is invisible from the target's side — an MRI (§5.3) that cannot see half the body. Federated diagnostics make cross-corpus tension *discoverable*, which is the whole point of Act III.
- **Synthesis (Act IV) — served.** Synthesis notes are born where the thinking happens and belong where the knowledge lives. The journaled cross-universe move with earned-data cargo is precisely the operation Act IV needs: a `generalizes` note can resolve a cross-corpus tension and then take its earned history home.
- **Conviction (Act V) — served, and currently split-brain.** Conviction is earned through weight, traversal, and confidence accumulation. Today those earned rows accrue only on the author's side of a crossing; the umbrella's conviction accounting is fragmented. Router bookkeeping "in the note's own universe's database" unifies the ledger of conviction.
- **Observation (Act I) — neutral.** No claim made, none needed.

A concept that concretely serves four Acts — and *repairs* two the system currently performs defectively — passes the core paper's §8 test decisively.

### 2. Does seamless federation strengthen or dilute the Living Link Architecture? Strengthens — conditionally, and the conditions are real.

**Strengthens:** the architecture's eight properties, lifecycle, and reversibility promise currently *stop at the universe boundary*. Cross-universe links have no functioning backlink, break on rename, and — the app-killer the Architect names in §1.5 — a cid collision on transfer re-mints and **severs every earned row**, directly violating "every link operation must be reversible; archival, not deletion." The Router design (conditions 3/4: journaled two-phase transfer, re-key-never-sever) is the Living Link Architecture *asserting itself across the boundary* rather than being diluted by it.

**Dilution risks I found, which become my requirements:**
- **Vocabulary intelligibility.** The 8 typed links + `associative` are the cognitive vocabulary — the shared grammar of the whole system. Per-universe `link_types` registries (Architect §1.3) mean a cross-universe edge could carry a type the other universe cannot read. A vessel whose cargo the recipient cannot decode is not a living link. This must be settled at the concept level, not discovered in Build.
- **Provenance erasure.** Seamlessness taken too far erases an epistemically meaningful fact: *which corpus a conviction was formed in is itself knowledge*. The "planet mark as quiet identity" is the right instinct — identity, not warning — but it must remain legible and queryable, or federation quietly destroys a dimension of the knowledge it federates.
- **Earned data in transit.** Per CLAUDE.md's storage warning, `search.db` is today the *system of record* for the earned half of the architecture — there is no disk layer yet (PJ-262). Cross-universe transfer moves the system-of-record between databases. The journaled protocol is necessary; the Plan must treat any earned-data loss during transfer as an app-killer, full stop.

### 3. Does it uphold "the search engine is a diagnostic instrument for intellectual life"? Only if the diagnostics federate.

A stethoscope that auscultates one lung of a two-lunged patient is a broken instrument. If the user's intellectual life spans linked universes, then `contradicts [[X]]`, confidence distribution, weight analysis, and dormancy autopsy **must span the umbrella**, or hidden tensions become *structurally* invisible — the precise failure §5.3 exists to prevent. I note carefully: PJ-224 (does the ordinary search *box* federate?) remains the Boss's gated ruling, and the Architect doc rightly asks rather than assumes. My chair's position: the **link-graph diagnostic surfaces** must federate for the concept to hold; the plain-text finder can be scoped by ruling. Scoping must always be an explicit user filter, never a silent boundary.

### 4. The uniqueness claim — precisely what makes this different from "just folders in one vault."

A folder in one vault has: no identity, no index of its own, no link vocabulary, no earned-data ledger, no lock, no portability. Its boundary is cosmetic — one authority, one grammar, boundaries you can rename away. A Universe under this concept has all of those; it is a **sovereign, self-contained, portable whole** (Architect invariant 1: copy the folder out and nothing dangles). What Universe-of-Universes composes is therefore not partitions of one corpus but **sovereign corpora under one umbrella of agency**: many authorities, one mind operating across them; boundaries that are *real* (ownership, provenance, portability, per-universe truth) but not *barriers* (operations flow). The one-sentence uniqueness statement the Plan should carry: **sovereignty with seamlessness — each corpus keeps its own truth; the mind over them is one.** Obsidian-class vaults offer the opposite corner: hermetic separation with no cross-vault links at all; folders-in-one-vault offer merger with no sovereignty. Neither offers this. The claim of genuine uniqueness is earned — *provided* the invariants hold; without them it degrades into exactly the "folders in one vault, with extra steps" it claims not to be.

### The Boss's own framing, tested

"NOT an old-fashioned ad hoc system" — the Router shape (one invisible layer, operations never know universes exist, no confirmation ceremonies) is the anti-ad-hoc shape, and it matches The Constellation Way (user commands, app never crosses on its own). "Simple yet powerful" — simple at the surface (editing a linked note is just editing a note), powerful underneath (journaled transfers, collision re-keying, per-universe locks). The concept and the proposed embodiment are consistent with each other and with the soul documents. The horse is sound; the carriage fits it.

---

## REQUIREMENTS (each binding on the Plan)

1. **Cross-universe links become full living vessels.** All 8 properties functional across the boundary; the target universe learns of inbound edges (backlinks exist from both sides); rename heals foreign referrers as a deliberate, surfaced part of the operation. A cross-corpus link that remains a dead pointer refutes the concept — this is the acceptance test of the whole migration, not a nice-to-have.
2. **Diagnostic surfaces federate.** Cognitive queries (by type, confidence, weight), tension surfaces (`contradicts`), backlink/dormancy analysis, and any formulation-analysis feature operate across the open umbrella by default; universe scoping is an explicit, visible user filter, never a silent boundary. PJ-224 (the plain-text search box) is presented to the Boss as its own gated decision — the Plan must not fold it in silently.
3. **Earned data is sacred cargo.** Every cross-universe transfer is journaled and resumable; cid collision re-keys the travelling earned rows in the same transaction — never severs; no operation may silently reset `traversal_count`, `confidence`, `created`, or archival status. "Archival, not deletion — every operation reversible" holds *across* universes exactly as within one. Any earned-data loss in transfer is classified app-killer.
4. **Provenance stays legible.** The owning universe of every note and every link edge is quietly visible (the planet mark) and *queryable* (filterable in search, shown in link details) at every surface. Seamless must never mean identity-erased: which corpus a conviction was formed in is itself knowledge, and the Plan must name the surfaces where provenance appears.
5. **Vocabulary intelligibility across the umbrella.** The 8 core types + `associative` render, translate (all 15 languages), and search identically in every universe. For universe-custom link types, the Plan must specify — before Build — how the other universe reads a cross-edge carrying a type it does not define (display, search, and trigger/fingerprint threading per Architect condition 1). A blank, wrong, or untranslated type on a cross-edge is a corrupted vessel.
6. **Unlink is dormancy, not death.** Detaching a cUniverse marks cross-edges dormant/archived and preserves all earned rows on both sides; re-linking restores them (Renewal, per the lifecycle §4). A universe folder copied out alone remains fully functional, with its foreign edges degrading gracefully — visible as external/dormant, never corrupting, never silently deleted.
7. **Lifecycle semantics are crossing-invariant.** A traversal is a traversal and weight/decay math is identical regardless of which universe the user was active in when it happened; federation state alters lifecycle only through requirement 6's dormancy path. No separate "federated weight" concept may be invented.
8. **Seamless but never automatic.** Only user-commanded operations cross universes; automatic/background writes never do (MIG-065 §J retained as mechanism). No new confirmation ceremonies beyond Constellation's existing patterns for destructive operations — the planet mark carries identity so that modals don't have to.
9. **The uniqueness statement is written down and gated on.** The Plan's concept preamble carries the one-line concept — *sovereignty with seamlessness: each corpus keeps its own truth; the mind over them is one* — and every phase/wave of the Plan states which requirement above it advances, so the Build cannot drift into "folders in one vault, with extra steps."

**File references:** `E:\مشاريع كلاود\Constellation\docs\CONSTELLATION-KNOWLEDGE-FORMULATION.md` · `E:\مشاريع كلاود\Constellation\docs\concept-papers\00-Constellation-Core-Concept-Paper.md` · `E:\مشاريع كلاود\Constellation\docs\migrations\PJ-235-federation-boundary\MIG-111-ARCHITECT.md`

==========================================================================================
## SYNTHESIS SHEET
==========================================================================================

# VERDICT SHEET — "Universe of Universes" Concept Validation (MIG-111 / PJ-235)

## OVERALL VERDICT: **THE CONCEPT STANDS — VALIDATED-WITH-REQUIREMENTS (5/5 chairs, 0 REJECTED)**

All five chairs validated. No chair rejected. The concept proceeds to Plan, bound by the 37 requirements below and blocked on 3 chair-conflicts that require Boss rulings.

## CHAIR VERDICTS

| Chair | Verdict | One-line basis |
|---|---|---|
| INSPECTOR (Safety) | VALIDATED-WITH-REQUIREMENTS | The status quo is the more dangerous state (22 broken silent crossings); the Router shrinks the silent-failure surface — but an invisible layer's failures are invisible, so wrong-routing must be impossible or self-detecting and every failure loud. |
| AUDITOR (Laws) | VALIDATED-WITH-REQUIREMENTS | The horse is sound and it is the Boss's own; Option A is the only shape where full operations and self-contained portability are the same property — but the brief contained one false claim (fingerprint-gate threading), one non-working defense (WAL probe), and one Windows-only assumption (the lock). |
| UX | VALIDATED-WITH-REQUIREMENTS | The interface already behaves seamlessly over a broken border; one umbrella is the only model matching how the user thinks — provided place is metadata the note always wears, never a question the interface asks. |
| ART-DIRECTOR | VALIDATED-WITH-REQUIREMENTS | Quiet identity is the repo's existing visual DNA, incomplete; no new invention needed — the existing vocabulary made consistent, truthful, tokenized, structural, and universal. |
| PHILOSOPHER (Knowledge Formulation) | VALIDATED-WITH-REQUIREMENTS | Serves four of the Five Acts and repairs two performed falsely today; sovereignty with seamlessness is genuinely unique — provided cross-edges become full living vessels and earned data is sacred cargo. |

## UNION OF REQUIREMENTS (de-duplicated)

**Router & attribution core**

- **R1.** Router is the ONLY door, enforced structurally (routed handle/vocabulary types unobtainable outside `route_write`; a new write site cannot compile without declaring intent); the five unguarded writers (`ensure_cid_cn_cmd`, `sources_set_manual`, `write_conflict_sidecar`, `update_base_columns`/`update_base_order`) brought onto the boundary in Phase 1, before any door opens. [INSPECTOR, AUDITOR] — PLAN
- **R2.** Wrong-owner attribution made impossible or self-detecting: owner resolution Rust-side only (kills Class D at all 22 sites), resolved by universe ROOT not registered-library roots; post-write attribution assertion that surfaces to the user, never a log line. [INSPECTOR] — PLAN+BUILD
- **R3.** Per-universe vocabulary/context threading provably COMPLETE — `index_note` parse chain, trigger DDL, and the fingerprint gates (`links_backfill.rs:99`, `incoming_links_backfill.rs:49`) — with a red→green harness test indexing one note under two differing vocabularies; per-write staleness check on cached child registries. [INSPECTOR, AUDITOR] — PLAN+BUILD
- **R4.** Routed/pool opens NEVER write vocabulary-fingerprint stamps — only the owner's completed boot backfill stamps (triggers may regenerate from the child's own registry). [INSPECTOR, AUDITOR] — PLAN

**Cross-instance safety**

- **R5.** Cross-instance policy = refusal via a real per-universe OS lock (LockFileEx/share-mode on Windows, flock/fcntl on macOS behind `#[cfg]`; heartbeat + stale-lock recovery; NFC/NFD- and case-aware ownership matching); the WAL `BEGIN EXCLUSIVE` probe is certified insufficient (false negative on an idle instance) and retired from this duty; the lock covers search.db, the earned ledger, review-pulse.json, and registry JSONs; never last-writer-wins or merge; cross-process-signal follow-up filed as its own PJ. [INSPECTOR, AUDITOR] — PLAN
- **R6.** Refusal resolves BEFORE input, never after: lock at open or first-edit-intent; a locked-universe note opens read-only with a passive one-line explanation; save-time refusal after accepted typing forbidden; typed input preserved in a recoverable buffer. [UX] — PLAN *(see conflict C2)*

**Earned-data transfer engine**

- **R7.** Journaled two-phase transfer, resumable at every seam: payload durable in the DESTINATION root before the fs move; replay ordered before boot reconcile / cold-start auto-index on BOTH sides; the earned-data census IS the cargo manifest (never re-derived from memory); weight recomputed never copied; `created` carried; on-disk companions (earned.jsonl, note-history.jsonl, review-pulse.json, paths rewritten) travel; in-transaction aggregate verification; every crash window enumerated and red→green before the door opens; any earned-data loss classified app-killer; perceptible transfers show progress and a crash-resumed transfer announces its completion. [INSPECTOR, AUDITOR, PHILOSOPHER, UX] — PLAN+BUILD
- **R8.** cid collision: check-first, re-mint AND re-key travelling earned rows in ONE transaction — never sever; replay keyed (cid, exact destination path); refuse-and-report on duplicate cids; the inbound `target_cid_cn` rewrite specified as its own cascade. [INSPECTOR, AUDITOR, PHILOSOPHER] — PLAN

**Failure surface & performance boundary**

- **R9.** Quiet success, loud failure: every refusal, partial state, and incomplete transfer surfaces to the user in plain language at the moment it happens; no routed write may fail silently, queue silently, or fall back silently to the active universe's DB. [INSPECTOR] — PLAN
- **R10.** Nothing heavy on the save tail: child preparation (§C schema migrate, FTS rebuild, backup) runs at link-time or first-foreign-open with visible progress — never on the debounced save, never colliding with the PJ-103 5s close-flush cap; writes refuse visibly until the child is ready. [INSPECTOR, AUDITOR] — PLAN
- **R11.** `fs::copy` of a live-WAL database banned — SQLite backup API or checkpoint-TRUNCATE then copy; restore never over a live handle; fixed FIRST (it runs on today's boot path). [INSPECTOR, AUDITOR] — PLAN

**Operation policy & scope honesty**

- **R12.** Boss-ruled write-site → intent table covering all 22+ sites, ambiguous rows ruled explicitly (`ensure_cid_cn` on foreign open, watcher-adopt reindex, `sources_set_manual`); automatic/background writes never cross on their own (MIG-065 §J survives as the refusal mechanism); no automatic frontmatter injection into a foreign file without a ruling. [INSPECTOR, AUDITOR, PHILOSOPHER] — PLAN
- **R13.** Honest scoping: universe-blindness covers the single-owner class only; move, cross-universe rename cascade, and foreign link palette are explicitly two-party, journaled, universe-aware; rename-cascade reach = visible federation set only (directional) — stated in UI and docs, never marketed as closed. [AUDITOR, INSPECTOR] — PLAN
- **R14.** Whole-Ecosystem owner-following in the same pass: owner's `<root>/.trash` with trash listing/restore across owner roots; earned.jsonl / note-history.jsonl / review-pulse.json / libraries.json follow the owner; second-screen sync verified on routed notes; Editor-Surface Gate checklist passes on federated notes exactly as on own notes. [INSPECTOR, AUDITOR] — PLAN+BUILD

**Identity & visual vocabulary**

- **R15.** The Place Line: always-visible universe › library trail on note surfaces; foreign carries the planet mark, home carries nothing (silence means home); never hover-only or menu-dive. [UX, ART-DIRECTOR] — PLAN
- **R16.** Identity travels with the note row everywhere notes co-mingle — tab strip (universe added to `OpenTab` state), Quick Switcher, search rows, backlinks/outgoing, list surfaces, pickers, second screen — at secondary visual weight, applied whole-ecosystem in one pass; no third element (no badges, tints, or warning styling). [UX, ART-DIRECTOR] — PLAN *(see conflict C3)*
- **R17.** One mark, one meaning: the planet/orbit mark (`LibraryIcon` `cuniverse`) is the SOLE cross-universe identity mark, neutral identity color, never warning-styled; the six-circle cluster emblem reserved for the Universe Manager entry only; the mark carries identity, never router state (refusals are transient plain messages; federation-unavailable keeps its separate triangle badge). [ART-DIRECTOR, UX] — PLAN
- **R18.** Universe accent tokenized: hard-coded `#6366f1` (5+ files) becomes one theme token (`--cuniverse-accent`) in light and dark, exposed via the Style Setter cuniverse category; no per-surface literal survives. [ART-DIRECTOR] — PLAN
- **R19.** Visual identity keyed by (universe, library), never display-name: order-stable colour assignment (no attach/detach reshuffle); persisted tab `libraryColor` reconciled; name-keyed joins (backlink attraction, sky edges, `library:note` shadowing) moved identity-keyed or the ambiguity surfaced at display. [ART-DIRECTOR, UX] — PLAN

**Creation, disclosure & reversibility**

- **R20.** Creation never crosses by default: Ctrl+N, quick capture, daily note, template-create default to the active universe (ratifies the amended 2026-08-10 ruling); foreign destination only by deliberate selection; no sticky last-used destination across the universe boundary, ever. [UX] — PLAN
- **R21.** Pickers disclose by structure: federated destinations grouped under universe-named header rows (planet mark + name, the sidebar's existing collapsible grammar), never flattened into an undifferentiated list. [ART-DIRECTOR, UX] — PLAN
- **R22.** Undo replaces ceremony: cross-universe operations end with a quiet receipt carrying real Undo; the two-phase transfer engine designed to run in reverse so the Undo is genuine (extends "every link operation must be reversible"). [UX] — PLAN *(see conflict C1)*
- **R23.** Link continuity is part of the operation's definition: cross-universe move/rename heals every referrer in every reachable universe quietly; unreachable referrers become durable pending-heals completing on next reachability; the receipt states both counts; silent breakage and blocking dialogs both forbidden; backlinks exist from both sides; all 8 link properties functional across the boundary — the acceptance test of the whole migration. [PHILOSOPHER, UX] — PLAN+BUILD
- **R24.** Coverage honesty: any federated surface with a degraded universe set (unmounted, locked, attach-failed) shows a quiet coverage state at the surface itself ("2 of 3 universes"); results never silently shrink. [UX] — PLAN
- **R25.** No verb without a door: at each shipped wave, every surface either performs the operation end-to-end or visibly routes to the sanctioned door; offered-verbs-that-fail eliminated wave by wave. [UX] — PLAN+BUILD

**Knowledge-formulation guarantees**

- **R26.** Diagnostic surfaces federate by default (typed-link queries, contradicts/tension, backlink/dormancy analysis); universe scoping is an explicit, visible user filter, never a silent boundary; PJ-224 (the plain-text search box) goes to the Boss as its own gated decision, never folded in silently. [PHILOSOPHER, AUDITOR] — PLAN
- **R27.** Provenance legible AND queryable: the owning universe of every note and link edge is visible (the mark) and filterable in search / shown in link details; the Plan names the surfaces where provenance appears. [PHILOSOPHER] — PLAN
- **R28.** Vocabulary intelligibility across the umbrella: 8 core types + `associative` render, translate (all 15 languages), and search identically in every universe; behavior of a cross-edge carrying a universe-custom type the other side does not define is specified BEFORE Build (display, search, trigger/fingerprint threading). [PHILOSOPHER] — PLAN
- **R29.** Unlink is dormancy, not death: detaching a cUniverse marks cross-edges dormant/archived, preserving earned rows on both sides; re-linking restores them (Renewal); a copied-out universe remains fully functional with foreign edges degrading gracefully — visible, never corrupting, never silently deleted. [PHILOSOPHER] — PLAN
- **R30.** Lifecycle semantics crossing-invariant: traversal/weight/decay math identical regardless of which universe was active; no separate "federated weight" concept invented. [PHILOSOPHER] — PLAN

**Language & naming**

- **R31.** One user-facing name per language: a single Boss-confirmed term (the choice between "child Universe" / "Linked Universe" is his naming decision, taken once), applied across all 15 locales in the same commit; the jargon "cUniverse" banned from every user-visible string. [ART-DIRECTOR] — PLAN
- **R32.** RTL/bidi identity intact: planet mark direction-neutral (never mirrored); chevrons on universe/library rows flip in RTL (the missing `[dir="rtl"] .v-chev` rule becomes load-bearing); per-segment `dir="auto"`/`detectDir` on universe names; breadcrumb separators order-safe in mixed-script paths. [ART-DIRECTOR, UX] — PLAN

**Process & verification gates**

- **R33.** Measurement + inspection gates every wave: on the 7,600+-note corpus before each commit — boot time, typing latency, steady-state routed-write latency, first-write pool-open cost, child WAL growth under sustained parent writes; any regression blocks the wave; diff-scoped safety-inspection every build, whole-app sweep at migration close; editing a federated note indistinguishable in keystroke latency from an own note. [INSPECTOR, AUDITOR, UX] — BUILD
- **R34.** Boss journey validation per wave, gated by the Test Pipeline (tutorial-auditor → ui-inspector → Boss): end-to-end cross-universe journeys — edit a foreign note, move across, rename a cross-linked note, search with one universe unavailable, second-instance refusal — per the Testing Instructions Rule; every UI claim re-verified against the build at that time. [UX, ART-DIRECTOR] — BUILD
- **R35.** The PJ-262 sequencing question goes to the Boss BEFORE the transfer engine is planned or built (the Living-Link disk layer shipping first structurally shrinks the riskiest module); presented as an option, never decided silently. [INSPECTOR, AUDITOR] — PLAN
- **R36.** The repeals table (Architect §5) confirmed by the Boss explicitly, item by item, as Phase 2's first act (MIG-056 read-only contract, concept-paper 25 §3/§10, MIG-065 §J-as-contract, the 2026-08-10 ruling amendment); the concept may not silently overwrite prior Boss rulings. [AUDITOR] — PLAN
- **R37.** The uniqueness statement written and gated: the Plan preamble carries the one-line concept — "sovereignty with seamlessness: each corpus keeps its own truth; the mind over them is one" — and every phase/wave states which requirement it advances, so the Build cannot drift into "folders in one vault, with extra steps." [PHILOSOPHER] — PLAN

## CHAIR CONFLICTS — BOSS DECISIONS REQUIRED

- **C1 — Confirmation ceremonies (Architect §6.2). ART-DIRECTOR + UX vs AUDITOR.** ART-DIRECTOR Req 5 RULES that per-operation confirmation modals are rejected ("grouping IS the disclosure"), and UX Req 6 forbids "are you sure" ceremonies outright (Undo replaces confirmation). AUDITOR Req 9 requires the opposite posture: §6.2 is an OPEN Boss decision the Plan must present and may not pre-empt — "no confirmation ceremonies" is not to be assumed. (INSPECTOR accepts no-ceremonies only under the quiet-success/loud-failure asymmetry of R9; PHILOSOPHER Req 8 holds a middle position — no NEW ceremonies beyond existing destructive-op patterns.) **The Boss must rule §6.2 directly; until then the Plan carries it as an open decision, not a settled default.**
- **C2 — Locked-universe presentation. UX vs ART-DIRECTOR.** UX Req 5 requires a note in a locked universe to open read-only with a persistent passive one-line explanation ("This universe is open in another Constellation window"). ART-DIRECTOR Req 4 forbids "read-only banner styling" as a third visual element on linked notes, and Req 8 permits only transient plain messages for refusals. Whether the lock state is an exception permitted a persistent passive indicator needs a ruling — likely reconcilable, but the letter of the two requirements collides.
- **C3 — Tab-strip identity rendering. UX vs ART-DIRECTOR.** UX Req 2 requires quiet mark + universe NAME on every co-mingled row including the tab strip; ART-DIRECTOR Req 4 specifies the tab gets the planet mark only, explicitly "no text." Minor spec-level conflict; the Boss may rule it or delegate it to the Art Director & Team per the 2026-07-10 ruling that the AD team owns UX/UI design.

## SYNTHESIS NOTES

- Unanimous convergence points (4–5 chairs independently): the Router as structural single door (R1), the journaled/re-key-never-sever transfer (R7/R8), automatic-writes-never-cross with a Boss-ruled intent table (R12), quiet passive identity via the planet mark (R15–R17), and per-wave measurement + Boss-journey gates (R33/R34).
- Hard sequencing pre-conditions before ANY door opens: R1 (unguarded writers on-boundary), R3 (vocabulary threading proven), R5 (real lock), R7 (crash windows red→green), R11 (live-WAL copy ban — fixed first), R35 (PJ-262 ruling), R36 (repeals ratified).
- PLAN-binding: R1, R4–R6, R8–R13, R15–R22, R24, R26–R32, R35–R37. BUILD-binding: R33, R34. Both: R2, R3, R7, R14, R23, R25.