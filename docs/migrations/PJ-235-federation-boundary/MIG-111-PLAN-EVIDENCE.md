

==========================================================================================
## PRIOR :: pkm-field
==========================================================================================

# WA#5 Prior-Art Check: Multi-Base Federation with Full Cross-Base Operations in PKM

## Verdict up front

**No shipping PKM product offers multiple fully-independent knowledge bases federated into one workspace with FULL seamless cross-base operations.** Every product surveyed either (a) achieves seamlessness by being a single store (Notion teamspaces, Evernote notebooks), or (b) supports multiple stores but with hard seams at name-based links, type/schema vocabulary, or clone/identity semantics. The two closest models — **DEVONthink** (mature, desktop, local) and **Tana** (modern, cloud) — each solve roughly half the problem, and their failure points are the most instructive input for the design doc.

---

## Per-product findings

### Obsidian — vaults: hard walls, plugin duct tape
- Core Obsidian: one vault per window instance; no multi-vault view. Cross-vault links are a **long-standing open feature request, not implemented** ([forum FR #28798](https://forum.obsidian.md/t/cross-vault-links/28798)). A community member's summary in that thread: "Obsidian exists within its vault."
- Workarounds are all plugins: [Multi-Vault Navigator](https://community.obsidian.md/plugins/multi-vault-navigator) (`[[VaultName::NoteTitle]]` → read-only preview), [Vault-Linker](https://github.com/Caffa/Vault-Linker) (rewrites unresolved links to `obsidian://` URIs that *launch the other vault*), [Vault Transfer](https://github.com/ImaginaryProgramming/obsidian-vault-transfer) (copies note text to another vault's folder — desktop only, per-note).
- **Metadata carriage: none beyond the file itself.** A transferred note's links break unless the target vault happens to resolve the same names. Backlinks, graph position, plugin state — all lost. Notably, the FR thread's UUID workaround (**Advanced URI plugin — UUID-based linking "to avoid breakage when notes move"**) shows the community independently converging on global-identity as the fix.

### Logseq — switch-only
- Multiple graphs supported but **one at a time**; a multi-graph workflow is an open feature request ([discuss.logseq.com #8202](https://discuss.logseq.com/t/creating-a-multi-graph-workflow/8202)). No cross-graph links. Having multiple graphs open even breaks git autocommit ([issue #11011](https://github.com/logseq/logseq/issues/11011)). Users split work/home graphs for privacy and accept total isolation ([#11217](https://discuss.logseq.com/t/separate-graphs-for-work-and-home/11217)).

### Roam — fully separate graphs
- Multiple graphs = separate silos; switch between them; no cross-graph linking. Community threads show friction, e.g. daily notes fragmenting across graphs ([Roam forum #1160](https://forum.roamresearch.com/t/people-using-multiple-graphs-can-you-explain-how-you-use-it/1160)).

### Notion — seamless only inside one store
- **Teamspaces are NOT federation** — they are permission-scoped views inside one workspace/one database, which is why linking and moving across them is seamless ([Notion: intro to workspaces](https://www.notion.com/help/intro-to-workspaces), [teamspaces guide](https://www.notion.com/help/guides/teamspaces-give-teams-home-for-important-work)).
- Across **workspaces** (the actual separate stores): "Workspaces are completely separate, so you won't be able to link any content between them." Moving a page between workspaces can break **links, relations, permissions, and page history**, and the page lands unsorted in Private ([Notion: move & duplicate content](https://www.notion.com/help/transfer-content-to-another-account), [thomasjfrank.com guide](https://thomasjfrank.com/how-to-move-pages-between-workspaces-in-notion/), [connex.digital](https://connex.digital/blog/how-to-move-notion-pages-between-workspaces-without-losing-data/)).
- **Lesson: Notion is the canonical negative example** — cross-store move as lossy copy.

### DEVONthink — the closest classical model (multi-store, one window)
- **Multiple databases open simultaneously in one window** since v2.0; open/close individually ([DEVONtech: use multiple databases](https://www.devontechnologies.com/blog/tipusemultipledatabases), [2.0 spotlight](https://shop.devontechnologies.com/blog/devonthink20spotlightmultipledatabasesandmore)).
- **Search spans all open databases** ([forum](https://discourse.devontechnologies.com/t/search-across-multiple-databases/13498)). **Move/duplicate between databases is a first-class operation**, and the database format "stores metadata with every record" — so record-level metadata travels with the record.
- **Identity**: every item gets a **UUID at creation**; the item link `x-devonthink-item://UUID` "will only and always point to the item to which it is assigned" ([official: Understanding Item Links](https://www.devontechnologies.com/blog/20240502-understanding-devonthink-item-links)). Custom-metadata *fields* are defined app-wide, so a record's custom metadata stays meaningful in any database.
- **The seams (study these)**:
  - **WikiLinks (name-based) do not cross databases — "Short answer: no… will remain impossible"**; the sanctioned workaround is UUID item links ([forum: WikiLinks and multiple databases](https://discourse.devontechnologies.com/t/wikilinks-and-multiple-databases/57398)).
  - **Replicants (clones/transclusion) cannot cross databases** — same-DB only; attempting it silently produces a *copy* ([forum](https://discourse.devontechnologies.com/t/possible-to-replicate-between-databases-in-dtp3/54301), [official: duplicates & replicants](https://www.devontechnologies.com/blog/20230524-duplicates-replicants)).
  - **AI features (classify, see-also, concordance) are per-database** ([forum: single or multiple databases](https://discourse.devontechnologies.com/t/dt-dt2g-single-or-multiple-databases-and-why/63840)).
  - Sync-time **UUID collisions on cross-DB moves are a reported bug class** ([forum](https://discourse.devontechnologies.com/t/douplicate-uuid-problem-on-sync-after-moving-databases/22339)).
- **Honest unknown**: I could not find an explicit source confirming that an item link keeps resolving *after* the item is moved to a different database. The official "only and always points to the item" phrasing and the UUID-collision-on-move bug reports both imply the UUID travels with the record, but no source I read states it outright. If the design doc leans on this, verify hands-on before citing.

### Evernote — account = wall
- Notebooks inside one account: one store, seamless. Across **accounts**: no cross-account note links (viewer must be signed into an account with permission), **no cross-account search**; Teams keeps personal/business strictly split with separate tags/reminders/shortcuts ([Evernote: switch between individual and team notes](https://help.evernote.com/hc/en-us/articles/115000716428-Switch-between-individual-and-team-notes)). The sanctioned path is consolidation — moving notes into one account ([merge two accounts](https://help.evernote.com/hc/en-us/articles/209004867-How-to-Merge-The-Contents-of-Two-Evernote-Accounts)).

### Tana — the closest living model (true federation, schema seam)
- **Multiple workspaces visible in one sidebar simultaneously**; private workspace on top, others below ([Tana docs: workspaces](https://outliner.tana.inc/learn/features/workspaces)).
- **"Allow content from…"** — a per-workspace, per-direction consent setting that federates **search and autocomplete/@-references** across workspaces. Cross-workspace references work; if you lack membership in the owning workspace you see an **alias you cannot open** — graceful degradation, not a broken link.
- **`Move to [workspace]`** "changes which workspace *owns* the node" — the node plus its owned subtree transfers, framed as **ownership transfer, not copy**.
- **The seams**: Schemas and Libraries do **not** federate; moving supertags creates "*many* hidden dependencies… easier and faster to recreate the supertags and fields in the new workspace" (docs). A community guide reports a **5,000-node cross-workspace move limit** ([tananodes.com](https://tananodes.com/sharing-content-between-multiple-workspaces/)); the Input API is capped at workspaces <750k nodes. Merging workspaces has no supported path ([Tana Ideas](https://ideas.tana.inc/posts/440-merge-workspaces)).
- **Lesson: the TYPE SYSTEM is the hardest part of federation.** Tana got references and moves working; the schema/vocabulary layer is where it still tears.

### Capacities — explicit isolation by design
- "Spaces are completely separate… Objects in one space cannot reference objects in another" ([docs](https://docs.capacities.io/reference/spaces), [community write-up](https://medium.com/@pkmbeth/how-many-capacities-spaces-37b027edf507)). Moving = export/import, and "links between objects will not carry over." Multi-space workflows are a feedback-board request ([capacities.io feedback](https://capacities.io/feedback/p/spaces-20-enhanced-spaces-organization-multi-space-workflows)).

### Anytype — feature requests only
- Multiple spaces exist; **cross-space links** ([community #12573](https://community.anytype.io/t/linking-between-spaces/12573)) and **object migration between spaces** ([#12236](https://community.anytype.io/t/migration-of-objects-between-spaces/12236), [#12234](https://community.anytype.io/t/is-it-possible-to-transfer-things-between-spaces/12234)) are open requests, not shipped features per these sources.

---

## Comparison at a glance

| Product | Multi-store in one view | Cross-store search | Cross-store links | Cross-store move | Metadata carried on move |
|---|---|---|---|---|---|
| Obsidian | No (core) | No | No (FR open; plugin hacks) | Plugin copy only | File only; links break |
| Logseq | No (switch) | No | No | No | — |
| Roam | No (switch) | No | No | No | — |
| Notion | N/A (workspaces separate) | No | **No** | Lossy copy | Loses permissions, history, relations |
| DEVONthink | **Yes** | **Yes (all open DBs)** | UUID item links only; WikiLinks no; replicants no | **Yes, first-class** | **Yes — record-level metadata travels; app-wide field definitions** |
| Evernote | No (per account) | No | No | Account consolidation | Partial |
| Tana | **Yes (one sidebar)** | **Yes (opt-in per workspace)** | **Yes (alias if no access)** | **Yes (ownership transfer)** | Content yes; **schema/supertags no** |
| Capacities | No | No | No | Export/import; links lost | No |
| Anytype | Spaces exist | No | Requested | Requested | — |

---

## What Constellation can learn / steal

1. **From DEVONthink — identity is the foundation.** Creation-time, store-agnostic UUID per item; links reference identity, never path+store. Both DEVONthink's item links and Obsidian's community workaround (Advanced URI UUIDs) converge here. For cross-Universe ops, a note's identity must survive crossing a Universe boundary, and link records must bind to that identity.
2. **From DEVONthink — two link classes will emerge; design for it.** Name-based resolution (WikiLinks) stayed per-store *forever* ("will remain impossible") while UUID links crossed freely. Constellation already resolves wikilinks Universe-wide via `resolve_universe_libraries` ("It is ONE universe" ruling) — the design doc must decide whether that resolver spans the *federation* (parent + cUniverses) with the same guarantee, or Constellation inherits DEVONthink's two-tier seam.
3. **From Tana — consent-based, per-direction federation.** "Allow content from X" as an explicit opt-in per linked store matches The Constellation Way (user decides). Steal the **alias degradation** pattern too: a cross-Universe reference that can't resolve (cUniverse unmounted/removed) renders as an inert alias, never a silently broken or resurrected link.
4. **From Tana — move = ownership transfer of a subtree, not copy.** And its schema failure is the warning: **the type vocabulary is the hardest thing to federate.** Constellation's 8 link types are app-level cognitive vocabulary (an advantage — the core schema is global by design), but anything user-extensible per-Universe (custom link types via Style Setter, custom stages, per-library appearances) is exactly Tana's supertag trap. The design doc must inventory which vocabularies are app-global vs per-Universe *before* specifying cross-Universe move.
5. **From Notion/Evernote — the negative lesson.** Cross-store move as lossy copy (dropping history, permissions, relations) is the failure mode users hate most. For Constellation this maps directly onto the earned link data (weight, confidence, traversal count, archival state) that today lives only in `search.db` — a cross-Universe move that drops the earned half of a Living Link is the Notion failure reproduced.
6. **From DEVONthink — per-store compute is acceptable, per-store *knowledge* is not.** Users tolerate AI/classify being per-database; they do not tolerate broken links or lost metadata. Prioritize identity + link + metadata integrity across Universes; federated analytics (Sky View across the federation, etc.) can come later.

## Honest conclusion

"Universe of Universes with full seamless cross-base operations" is **genuinely unshipped in PKM** — but it is not un-attempted: it is the visible frontier both closest competitors are stuck at. DEVONthink proved multi-store + global search + metadata-preserving cross-store move on a UUID identity spine (1 seam: name-links/clones/AI stay per-store). Tana proved simultaneous federation + cross-store references + ownership-transfer moves (1 seam: the schema layer). Nobody has closed both seams at once. Constellation's differentiators — recursive cUniverse federation, links as first-class objects with earned properties, and an app-global cognitive vocabulary — attack precisely the two seams the field is stuck on, provided (a) note/link identity is made Universe-independent and (b) the earned link data survives the crossing.

Sources: [Obsidian FR: cross-vault links](https://forum.obsidian.md/t/cross-vault-links/28798) · [Multi-Vault Navigator](https://community.obsidian.md/plugins/multi-vault-navigator) · [Vault-Linker](https://github.com/Caffa/Vault-Linker) · [obsidian-vault-transfer](https://github.com/ImaginaryProgramming/obsidian-vault-transfer) · [Logseq multi-graph FR](https://discuss.logseq.com/t/creating-a-multi-graph-workflow/8202) · [Logseq #11011](https://github.com/logseq/logseq/issues/11011) · [Roam forum: multiple graphs](https://forum.roamresearch.com/t/people-using-multiple-graphs-can-you-explain-how-you-use-it/1160) · [Notion: workspaces](https://www.notion.com/help/intro-to-workspaces) · [Notion: move & duplicate](https://www.notion.com/help/transfer-content-to-another-account) · [thomasjfrank: move pages between workspaces](https://thomasjfrank.com/how-to-move-pages-between-workspaces-in-notion/) · [connex.digital: move without losing data](https://connex.digital/blog/how-to-move-notion-pages-between-workspaces-without-losing-data/) · [DEVONtech: use multiple databases](https://www.devontechnologies.com/blog/tipusemultipledatabases) · [DEVONtech: understanding item links](https://www.devontechnologies.com/blog/20240502-understanding-devonthink-item-links) · [DEVONtech: duplicates & replicants](https://www.devontechnologies.com/blog/20230524-duplicates-replicants) · [DT forum: search across DBs](https://discourse.devontechnologies.com/t/search-across-multiple-databases/13498) · [DT forum: WikiLinks and multiple databases](https://discourse.devontechnologies.com/t/wikilinks-and-multiple-databases/57398) · [DT forum: replicate between DBs](https://discourse.devontechnologies.com/t/possible-to-replicate-between-databases-in-dtp3/54301) · [DT forum: UUID sync problem](https://discourse.devontechnologies.com/t/douplicate-uuid-problem-on-sync-after-moving-databases/22339) · [DT forum: single or multiple DBs](https://discourse.devontechnologies.com/t/dt-dt2g-single-or-multiple-databases-and-why/63840) · [Evernote: individual vs team notes](https://help.evernote.com/hc/en-us/articles/115000716428-Switch-between-individual-and-team-notes) · [Evernote: merge accounts](https://help.evernote.com/hc/en-us/articles/209004867-How-to-Merge-The-Contents-of-Two-Evernote-Accounts) · [Tana docs: workspaces](https://outliner.tana.inc/learn/features/workspaces) · [tananodes: sharing between workspaces](https://tananodes.com/sharing-content-between-multiple-workspaces/) · [Tana Ideas: merge workspaces](https://ideas.tana.inc/posts/440-merge-workspaces) · [Capacities docs: spaces](https://docs.capacities.io/reference/spaces) · [Capacities feedback: multi-space workflows](https://capacities.io/feedback/p/spaces-20-enhanced-spaces-organization-multi-space-workflows) · [pkmbeth on Capacities spaces](https://medium.com/@pkmbeth/how-many-capacities-spaces-37b027edf507) · [Anytype: linking between spaces](https://community.anytype.io/t/linking-between-spaces/12573) · [Anytype: migration of objects](https://community.anytype.io/t/migration-of-objects-between-spaces/12236) · [Anytype: transfer between spaces](https://community.anytype.io/t/is-it-possible-to-transfer-things-between-spaces/12234)

==========================================================================================
## PRIOR :: engineering
==========================================================================================

# WA#5 Prior-Art Report: Routing Operations Across Multiple SQLite Databases in One App

Research for MIG-111's Router. Five areas studied via authoritative docs (sqlite.org, RFC editor, MS/Mozilla source docs, vscode wiki), each concluding with the transferable pattern.

---

## 1. SQLite ATTACH with write — the actual atomicity guarantee

**The guarantee, verbatim from sqlite.org/lang_attach.html:**

> "Transactions involving multiple attached databases are atomic, assuming that the main database is not ':memory:' and the journal_mode is not WAL."

**The WAL exception, verbatim:** if journal_mode is WAL (or main is `:memory:`), "transactions continue to be atomic within each individual database file. But if the host computer crashes in the middle of a COMMIT where two or more database files are updated, some of those files might get the changes where others might not."

**This kills ATTACH as a cross-store write-atomicity mechanism for Constellation**, because Constellation's `search.db` runs WAL (the LL-XXX 3 GB WAL incident in CLAUDE.md confirms it). Switching to rollback journaling to get cross-file atomicity would sacrifice WAL's concurrent-reader performance — the wrong trade for a keystroke-latency-sensitive app.

**How rollback mode achieves it (the mechanism worth stealing):** sqlite.org/atomiccommit.html §5 "Multi-file Commit" — the **super-journal** (formerly "master journal"). Each participating DB gets its own rollback journal; a super-journal file (`<main>-mj_HHHHHHHH`) "contains the full pathnames for rollback journals for every database that is participating in the transaction"; each rollback journal's header records the super-journal's path. Sequence: create per-DB journals → create + sync super-journal → write journal headers → update all DBs → **delete the super-journal (this single unlink IS the commit point)** → clean up per-DB journals. Recovery rule: a journal is "hot" (must roll back) only if it has no super-journal name in its header **or the super-journal still exists on disk**. One file's existence flips N stores between "all roll back" and "all committed."

**Confirmation of non-atomicity in WAL from independent sources:** a KAIST ICACT paper ("Atomic Multi-database Transaction of WAL Journaling Mode in SQLite", ieeexplore 7890219, with a patched fork at github.com/purpleblues/sqlite) exists precisely because stock SQLite does not provide it — proof this must be solved above SQLite, not assumed from it.

**Secondary ATTACH facts:** attach count is capped (`SQLITE_LIMIT_ATTACHED`; default 10, hard max 125); unqualified table names resolve by attach order on collision — so any cross-store SQL must always schema-qualify (`storeA.note_meta`). ATTACH remains fine for **read-only** cross-store queries.

**Pattern extracted:** cross-DB write atomicity in WAL mode must be provided by an application-level journal whose *deletion is the commit point*, with per-store operations individually atomic underneath — i.e., a super-journal analog.

---

## 2. Connection-per-store designs — Thunderbird and Firefox profiles

**Thunderbird (source-docs.thunderbird.net):** historical model = one Mork database per folder (`INBOX.msf` etc.) holding per-message metadata; **Gloda** (`global-messages-db.sqlite`) was added in TB3 as a *single global SQLite index layered over the per-folder stores* to work around Mork's limitations; the current **Panorama** project is replacing the per-folder databases with *one* global SQLite DB, explicitly "to address some of the various database quirks, pain points and limitations that have accreted over time."

Two honest lessons, not one:
- **The global-view lesson (Gloda):** when stores are plural, the cross-store view is a *derived global index maintained at write time* — never an ad-hoc runtime join across N stores. (This is Constellation's Rule 8, Write-Time Derivation, independently arrived at by Mozilla.)
- **The granularity lesson (Panorama):** per-*folder* databases were too fine and became a 20-year liability. Store granularity should match the **ownership and transfer boundary** — for Constellation that is the Universe (one `search.db` per Universe), never finer (not per-library, not per-folder).

**Firefox profiles:** one profile = many task-scoped SQLite files (`places.sqlite`, `cookies.sqlite`, …), each with its own connection — cross-DB transactions are simply never needed because no operation spans them. The profile itself is made **single-process** via `parent.lock`, which sidesteps cross-process DB contention entirely (see §5). Pattern: *partition stores so that hot-path operations never span two of them; reserve cross-store operations for explicit, rare, journaled transfers.*

---

## 3. VS Code multi-root workspaces — the routing model itself

From the microsoft/vscode wiki "Adopting Multi-Root Workspace APIs":

- **The routing primitive:** `workspace.getWorkspaceFolder(uri)` — every operation carries a resource URI; the API resolves it to its owning root (or `undefined` if outside all roots). Extensions never enumerate-and-guess; they resolve once at the boundary.
- **Config resolution is a cascade keyed by resource:** settings are either *window-scoped* (whole instance) or *resource-scoped*; `getConfiguration(section, resource)` finds the owning `WorkspaceFolder` first — "if such a folder exists, and that folder defines the setting, it will be returned. Otherwise, the normal logic applies... the workspace file or the user level." Folder → workspace → user.
- **Service multiplexing:** one language server handling multiple folders is preferred ("more resource-friendly than running multiple servers"); a server-per-folder is legitimate when isolation is needed. `vscode-languageclient` supports both.
- The workspace manifest is a single JSON file (`.code-workspace`) listing roots + shared settings — the identity of the multi-root set lives in one small manifest, not in the roots themselves.

**Pattern extracted:** a Router is (a) a resolve-resource-to-owning-store function called once at every operation boundary, (b) a per-store settings/vocabulary cascade with fallback to universe-level then app-level, (c) a policy decision per subsystem of "one service spanning stores" vs "one service per store."

---

## 4. Identity + transfer — Syncthing and Git

**Syncthing (docs.syncthing.net):** a synced folder's identity is its **Folder ID** — stable, shared across devices, and explicitly decoupled from both the local path and the human label ("the label of a folder is a human readable and descriptive local name. May be different on each device"). Devices are identified by a **Device ID** derived from a TLS certificate hash — identity from a minted credential, never from location. **Pattern: identity = minted stable ID in the manifest; path and display name are mutable attributes that never participate in identity.** (Constellation's rename-cascade and MIG-108 path-rewrite pain both trace to path-as-identity; this is the industry answer.)

**Git:** every repo is self-contained (`.git`); cross-repo transfer is content-addressed (objects are idempotent to re-copy), and the *commit point* of any transfer is a single small atomic update (a ref) done via the lockfile pattern (§5). Nothing is ever "half-moved" — objects may exist redundantly in both repos, and only the ref flip changes what's true. **Pattern: make the bulk transfer idempotent/re-runnable, and make the state flip a single tiny atomic write.**

---

## 5. Two-phase / journaled cross-store moves

**IMAP MOVE (RFC 6851)** — the definitive semantics for "move an item between stores you don't co-transact over":
- Old client-driven pattern was COPY → STORE \Deleted → EXPUNGE — three operations with crash windows between them; the standardized MOVE exists because that sequence, done client-side, could strand state.
- RFC 6851's per-item invariant: the server **"MUST NOT leave the message in neither mailbox"** and SHOULD NOT leave it in both, *even when the command fails* — and critically, the guarantee is **per message, not per command**: a multi-message MOVE may fail partway with some messages moved and some not, but no individual message is ever lost.
- With UIDPLUS, the target assigns **new UIDs** and reports the old→new mapping via `COPYUID` — the moved item gets a *new identity in the destination store's namespace*, with an explicit mapping receipt.

**Filesystem cross-volume moves:** `rename()` fails with `EXDEV` across devices; every `mv` implementation falls back to **copy + delete, which is not atomic** — and naive fallbacks have corrupted data when interrupted (npm, pnpm, electron-store all shipped EXDEV bugs). The proven safe shape: copy to a temp name *on the destination volume*, sync, `rename()` into place (atomic within the destination), only then delete the source. Failure direction is always **duplicate, never absent**.

**Pattern extracted (all three sources agree):** cross-store transfer = **copy-forward, verify, then delete-back, under a journal; the tolerated failure mode is "exists in both," the forbidden one is "exists in neither"; the destination mints new IDs and the journal records the old→new mapping.**

---

## 6. Single-writer locking across processes — Windows-verified

- **SQLite itself (sqlite.org/lockingv3.html):** on Windows it uses `LockFile()/LockFileEx()/UnlockFileEx()` **byte-range locks on the database file itself** — SHARED = lock one random byte in a designated range (so multiple readers rarely collide), RESERVED and PENDING = designated single bytes, EXCLUSIVE = the whole range. These are OS-enforced and work across processes with zero extra files. Lesson: within-DB single-writer is *already solved by SQLite* — the Router needs `busy_timeout` and one writer connection per store, not its own DB lock.
- **git `index.lock` (git-scm.com api-lockfile):** create `<file>.lock` with `O_CREAT|O_EXCL` (atomic create-if-absent; `CREATE_NEW` on Windows), write the *new content* into it, then **rename over the target** to commit-and-unlock. "Readers do not block, but they are guaranteed to see either the old contents or the new contents." Stale-lock cleanup via `atexit`/signal handlers. This is the pattern for every manifest/JSON write (`libraries.json`, `universe.json`, transfer journals).
- **LibreOffice `.~lock.<name>#`:** an *advisory, human-readable* lock containing user name, machine ID, and timestamp — its job is UX (tell the second opener *who* has it) and stale-lock detection (timestamp threshold → offer override). Not tamper-proof, and not liveness-proof.
- **Firefox `parent.lock`:** profile-scope exclusivity — one process owns the whole profile. Cautionary bug on record (Bugzilla 726759): the Windows profile manager treated *mere existence* of `parent.lock` as "locked," breaking after crashes. **Existence of a lock file is not proof of liveness; the lock must be a *held* OS handle (or contain liveness info), with the file's contents serving the UX.**

**Pattern extracted:** two layers — (1) DB-level: let SQLite's own cross-process byte-range locking do its job; (2) store/Universe-level: an **exclusively-held open file handle** for real mutual exclusion (crash releases it automatically on Windows), whose **contents are LibreOffice-style metadata** (process, machine, user, timestamp) for stale-detection UX, acquired via atomic create, never trusted on existence alone.

---

## Conclusion — the pattern set MIG-111's Router should adopt

| # | Router pattern | Named precedent |
|---|---|---|
| **R1** | **Resolve-at-the-boundary routing:** every operation carries the resource; the Router resolves it once to an owning-store context (connection handle + schema version + vocabulary + settings). Subsystems hold store handles from the Router, never raw connections. | VS Code `workspace.getWorkspaceFolder(uri)` |
| **R2** | **One connection (single designated writer) per store DB; ATTACH never used for cross-store writes** — WAL forfeits cross-file COMMIT atomicity (sqlite.org/lang_attach.html, verbatim above). ATTACH permitted only for read-only cross-store queries, always schema-qualified. | SQLite official docs; Firefox profile (N task DBs, no cross-DB transactions on any hot path) |
| **R3** | **Store granularity = the transfer boundary (Universe), no finer.** Cross-store views are write-time-derived global indexes, never runtime joins over N stores. | Thunderbird: Panorama consolidating away per-folder DBs; Gloda as the derived global index (= Constellation Rule 8) |
| **R4** | **Identity is a minted stable ID in the manifest; path and label are mutable non-identity attributes.** Cross-store references use the ID; a transferred item receives a new ID in the destination namespace plus a recorded old→new mapping. | Syncthing Folder ID / Device ID; IMAP UIDPLUS `COPYUID` |
| **R5** | **Cross-store transfer = journaled copy-forward-then-delete-back:** (0) write a transfer journal at a well-known location (source, dest, item IDs, phase); (1) COPY into dest in one dest-local transaction, fsync; (2) verify; (3) DELETE from source in one source-local transaction; (4) **delete the journal — that unlink is the commit point**; boot-time recovery reads any surviving journal and rolls forward or back. Invariant: an item may transiently exist in **both** stores, **never in neither** — per item, so a multi-item transfer interrupted midway strands nothing. Any accompanying file move across roots is copy-to-temp-on-dest + rename-into-place, never cross-root `rename()`. | SQLite super-journal (atomiccommit.html §5 — delete-journal-as-commit-point + hot-journal recovery rule); RFC 6851 IMAP MOVE ("MUST NOT leave in neither", per-message guarantee); EXDEV copy+rename+delete discipline |
| **R6** | **Per-store vocabulary/settings as a resource-keyed cascade:** store-level value if defined, else universe-level, else app-level — resolved through the same Router context as R1, so schema/vocabulary can never be read from the wrong store. | VS Code resource-scoped `getConfiguration(section, resource)` folder → workspace → user cascade |
| **R7** | **Two-layer cross-process locking:** DB layer — trust SQLite's own `LockFileEx` byte-range protocol + `busy_timeout`; Universe layer — an exclusively-**held** open lock-file handle (atomic `CREATE_NEW` acquisition, auto-released by the OS on crash), containing user/machine/timestamp metadata for who-has-it UX and stale detection; existence alone never treated as locked. Manifest and journal writes use write-to-`.lock`-sibling + atomic rename. | SQLite lockingv3 (Windows byte-range locks); git `index.lock` (`O_CREAT|O_EXCL` + rename-to-commit); LibreOffice `.~lock` metadata; Firefox `parent.lock` + Bugzilla 726759 (the existence-is-not-liveness bug) |

**The one anti-pattern to name in the design doc:** relying on ATTACH + BEGIN/COMMIT for a cross-Universe move because "SQLite transactions are atomic." In WAL mode that guarantee explicitly does not hold across files — a crash mid-COMMIT can commit the destination insert but not the source delete (duplicate — recoverable) *or the reverse (loss — the app-killer)*. R5's journal exists precisely to make the loss direction impossible; the KAIST patch proves stock SQLite will not do it for us.

Sources:
- [SQLite: ATTACH DATABASE (atomicity guarantee + WAL exception)](https://sqlite.org/lang_attach.html)
- [SQLite: Atomic Commit In SQLite — §5 Multi-file Commit (super-journal)](https://sqlite.org/atomiccommit.html)
- [SQLite: File Locking And Concurrency In SQLite Version 3 (Windows LockFileEx byte-range locks)](https://sqlite.org/lockingv3.html)
- [SQLite: Write-Ahead Logging](https://www.sqlite.org/wal.html)
- [IEEE ICACT: Atomic Multi-database Transaction of WAL Journaling Mode in SQLite](https://ieeexplore.ieee.org/document/7890219/) / [patched fork](https://github.com/purpleblues/sqlite)
- [SQLite Forum: Questions about WAL commits over multiple databases](https://sqlite.org/forum/info/77f56fa7734598ba5155a3836ab85c47f7f448cb89e940afbef51a133a174e86)
- [Thunderbird Source Docs: Message Database (per-folder .msf)](https://source-docs.thunderbird.net/en/latest/backend/message_database.html)
- [Thunderbird Source Docs: New Database (Panorama)](https://source-docs.thunderbird.net/en/latest/panorama/index.html)
- [MDN (archived): Gloda — global-messages-db.sqlite](http://www.devdoc.net/web/developer.mozilla.org/en-US/docs/Mozilla/Thunderbird/gloda.html)
- [microsoft/vscode wiki: Adopting Multi-Root Workspace APIs](https://github.com/microsoft/vscode/wiki/Adopting-Multi-Root-Workspace-APIs)
- [VS Code docs: Multi-root Workspaces](https://code.visualstudio.com/docs/editing/workspaces/multi-root-workspaces)
- [Syncthing docs: Configuration (folder ID vs label vs path)](https://docs.syncthing.net/users/config.html) / [FAQ](https://docs.syncthing.net/users/faq.html)
- [RFC 6851: IMAP MOVE Extension](https://datatracker.ietf.org/doc/html/rfc6851)
- [git api-lockfile documentation](https://git-scm.com/docs/api-lockfile) / [git lockfile.h](https://github.com/git/git/blob/master/lockfile.h)
- [alexwlchan: Atomic, cross-filesystem moves in Python (EXDEV discipline)](https://alexwlchan.net/2019/atomic-cross-filesystem-moves-in-python/)
- [Apache OpenOffice forum: lock file behavior (.~lock contents)](https://forum.openoffice.org/en/forum/viewtopic.php?t=111180)
- [Bugzilla 726759: Profile manager assumes Windows profile is locked if parent.lock exists](https://bugzilla.mozilla.org/show_bug.cgi?id=726759)
- [Mozilla Support: "Firefox is already running" (parent.lock semantics)](https://support.mozilla.org/en-US/kb/firefox-already-running-not-responding)

==========================================================================================
## DRAFTS :: core
==========================================================================================

All load-bearing claims verified in source this session (`require_own_library` libraries.rs:295-345, `SearchState` search.rs:1341-1416, `schema_gate` search.rs:1448-1463, `REGISTRY`/`snapshot`/`load_active` link_types.rs:351/498/522, fingerprint gates incoming_links_backfill.rs:49 + links_backfill.rs:99, `constellation_search_reindex` search.rs:12251, `reindex_single_note`/`index_note(conn,…)` search.rs:12682/12718, `find_universe_root` attach.rs:105-118, `link_life::append`/`FILE_LOCK` link_life.rs:191/205, `try_load_libraries_at` libraries.rs:90). The design below is the deliverable.

---

# MIG-111 — THE UNIVERSE ROUTER · Core Design + Build Plan (Plan-phase input)

## 0. Concept (the horse)

> **The Universe Router is the single answer to "whose note is this?"** Every operation resolves ownership once, at one choke point, and receives the note's home universe's full context — DB writer, link vocabulary, `.constellation` root. Own notes flow through untouched at zero cost. The umbrella is ONE space; the machinery is invisible.

**The governing invariant, one line:** *a routed operation is indistinguishable from the same operation performed with the note's home universe active.* Every design decision below is mechanical once this is fixed: the owner's registry attributes, the owner's vocabulary parses, the owner's fingerprint gates decide maintenance, the owner's ledger receives appends, the owner's `.trash` receives deletions. This is MIG-100's "writable only through its own universe's identity," promoted from a guard's doc-comment (libraries.rs:266-269) to the system's architecture.

---

## 1. The Router core (`federation/router.rs`, new)

### 1.1 Signatures

```rust
pub enum Owner {
    /// The active universe — the fast path. No pool, no lock probe, no vocab load.
    Active,
    /// A linked universe; `root` contains .constellation/universe.json.
    Linked { root: PathBuf },
}

pub enum WriteIntent {
    /// A gesture the user commanded (save of their edit, tag, property, task,
    /// move, traverse, review action). May cross universes.
    UserCommanded,
    /// Ambient maintenance (watcher flush, boot repair, reconcile, canonicalize
    /// sweeps). NEVER crosses — MIG-065 §J retained as mechanism.
    Automatic,
}

/// Everything an operation's DB/ledger tail needs, resolved once.
pub struct HomeCtx<'a> {
    pub conn: ConnGuard<'a>,          // Active → SearchState.db guard (the existing writer);
                                       // Linked → the pool entry's writer guard
    pub vocab: VocabHandle,            // Active → global-registry passthrough (zero cost);
                                       // Linked → child snapshot from its link-types.json
    pub universe_root: PathBuf,        // home root; .constellation/ = ledger, review-pulse, .trash
    pub library_name: Option<String>,  // longest-root-wins over the OWNER's libraries.json
}

pub fn resolve_owner(app: &AppHandle, path: &str) -> Result<Owner, String>;

pub fn with_home_context<T>(
    app: &AppHandle, state: &SearchState, path: &str, intent: WriteIntent,
    f: impl FnOnce(&mut HomeCtx) -> Result<T, String>,
) -> Result<T, String>;
```

Closure-scoped on purpose (Constraint as Design): the guard's lifetime is the closure, so no caller can accidentally hold two universes' writers — only the Wave-2 transfer engine gets the explicit two-context API (`with_context_pair`, ordered lock acquisition source-then-dest by normalized root sort, deadlock-free by construction).

### 1.2 Ownership resolution — one rule, not two boundaries

**Longest-prefix match over ALL known universe roots** (active root + every cUniverse root from `resolve_universe_libraries`, universe.rs:1513), on normalized paths (NFC, separator-neutral, case-folded per platform — the mig108.rs:1122-1136 "never SQL replace()" normalized-matching helpers reused; macOS NFD noted). Fallback: `find_universe_root` (the parent-walk to `universe.json`, attach.rs:105-118, promoted `pub(crate)`). No root found → `Err`, fail-closed.

This single rule **subsumes today's two-boundary hack**: a cUniverse nested under the active root defeats prefix-alone (the documented trap at libraries.rs:312-318), but under longest-match the nested child's root is *longer* than the active root and wins automatically. It also **fixes the guard's filed known-limit** (libraries.rs:320-323 — foreign set held library roots, not universe roots, so a linked universe with no root-registered library was invisible): the router matches by *universe* root, always.

### 1.3 Composition with SearchState — the active fast path costs nothing

`Owner::Active` short-circuits: one cached-string prefix compare, then `state.db.lock()` — **byte-for-byte today's code path**. `VocabHandle::ActiveGlobal` reads through the existing global registry exactly as the 30+ sites do now (no snapshot clone, no churn at the non-indexing read sites). `library_name` = `owning_own_library_name_in` as today. Nothing new is allocated, opened, statted, or locked. Boot: the Router adds **zero** work — no pool entry exists until the first routed write (Rule 8: child DBs open lazily, NEVER on boot). Typing: routed work only ever sits on the debounced save tail or an explicit gesture; the keystroke path keeps zero `invoke()` and zero router calls.

`invalidate_search_state` (search.rs:11228-11284) additionally drains the pool (close connections, drop vocab caches) under the existing `federation_generation` bump — the proven capture-before-init / check-before-publish discipline (search.rs:11537-11539, 11635-11648) applies to every pool open.

### 1.4 `LinkedDbPool` — lazy side-by-side writers

Added to `SearchState`: `linked_pool: Mutex<HashMap<PathBuf /*normalized root*/, Arc<LinkedDbEntry>>>` where `LinkedDbEntry { writer: Mutex<Option<Connection>>, ready: AtomicBool, init_lock: Mutex<()>, vocab_cache: Mutex<Option<VocabCache>>, generation: u64 }` — the proven single-slot pattern (`db`/`db_ready`/`init_lock`) replicated per child. The ro-ATTACH read layer (`cu0..cu24`) is untouched; writes never go through an attached alias (WAL forbids cross-file COMMIT atomicity — sqlite.org/lang_attach.html; rw-open-while-ro-attached has the in-repo precedent `federation_prewarm`, search.rs:11358-11474).

**Open sequence (first routed write to that universe, never boot):**

1. Capture `federation_generation`; take the entry's `init_lock`.
2. **Owner-lock probe** (§1.6). Held-elsewhere → typed refusal with who-has-it.
3. **Schema gate — owner-respecting posture.** Read `<root>/.constellation/search.version` and run `schema_gate` (search.rs:1457). Match → proceed. Mismatch or structurally incomplete → **the parent NEVER rename-asides or rebuilds a child's DB** (the 2026-07-24 lesson generalized: destructive surgery on a database whose 14 backfills we cannot run). Instead return typed `ChildNeedsUpgrade`; the frontend surfaces a door ("This linked universe needs a one-time upgrade — run now?") which runs the MIG-056 §C safeguard flow (`init_db_schema_only`, backup **via the SQLite backup API, never `fs::copy`** — Phase 0 fix) with a progress strip, **off the save tail always** (verdict condition 5; saves refuse visibly until ready).
4. Open rw `Connection`; standard pragmas (`busy_timeout=5000`, `synchronous=NORMAL`, `recursive_triggers=ON`; WAL is a file property, inherited); **register the FTS5 tokenizer** (connection-local — MIG-056 §K.1 proved silent zero-row MATCH without it, search.rs:1558-1581).
5. **Load the child's vocabulary** from `<root>/.constellation/link-types.json` into `VocabCache { registry, mtime+len }`.
6. **Trigger-DDL completeness check**: if the child's registry-generated trigger set is stale/missing (`ForeignSchemaOnly` deliberately skips it, search.rs:4571-4589), regenerate the DDL **from the child's own registry** via a third scope `InitScope::LinkedFullWithRegistry(&child_registry)` — safe precisely because the DDL source is the child's vocabulary (the entire reason `ForeignSchemaOnly` exists, PJ-230/PJ-232). **The pool NEVER writes the vocab-fingerprint backfill stamps** (verdict condition 2 — stamping would bless stale aggregates permanently, the migrate.rs:88-95 shape); only the child's own completed backfill stamps, so its next boot heals its aggregates.
7. Generation check → publish (`ready = true`).

**Per-routed-write staleness check** (verdict condition 6): stat `link-types.json` (mtime+len); changed → reload vocab, re-run step 6's check. One stat on the save tail — negligible, never on a keystroke.

### 1.5 Vocabulary threading — through the fingerprint gates (Architect condition 1)

`VocabHandle` is the structural enforcement: the indexing chain — `index_note`'s parse callees (`is_known_type` decisions at search.rs:7244/7371, structural exclusions at 8018/8485/8561/9550), trigger-DDL generation, **and both maintenance gates** — takes `&VocabHandle` instead of reading the global:

```rust
// incoming_links_backfill.rs:49 and links_backfill.rs:99 become:
is_built(conn) && stored_vocab_fingerprint(conn) == vocab.fingerprint()
//                                                   ^^^^^ the HOME universe's, never the process's
```

This kills the adversarial pass's app-killer: today the gate compares the child's stored fingerprint against the **active** registry's fingerprint, so with differing vocabularies every routed write silently skips incoming/outgoing maintenance while `is_built` stays true. Under threading, a routed write behaves exactly as the child's own process would in the same DB state: gate-true → maintain; gate-false (child's vocabulary edited, backfill pending) → skip, and **the child's own next boot heals** — deferred-to-owner, never divergent. Red→green harness test required before any foreign index write ships: index one probe note under two deliberately different vocabularies; diff `note_links` and the maintenance outcomes (Phase 1 gate, blocking).

The 30+ non-indexing read sites (active-UI queries) keep reading the global — `VocabHandle::ActiveGlobal` is a passthrough, so they don't churn.

### 1.6 The owner lock-file protocol (`federation/owner_lock.rs`, new — macOS-neutral)

One file per universe: `<root>/.constellation/owner.lock`. **Byte-range locks, two roles** — `LockFileEx` on Windows, `F_OFD_SETLK` on macOS/Linux (OFD, not classic `fcntl`, to avoid the close-any-fd-releases hazard; both APIs do shared+exclusive ranges — the same mechanism SQLite itself uses, lockingv3):

- **ACTIVE role** — the instance activating universe U takes an **exclusive** lock on byte 0, held for the whole session, released by universe-switch or process exit (**the OS releases on crash — liveness is the held handle, never file existence**; the Bugzilla 726759 lesson). Acquisition failure after a ~2s bounded retry → refuse activation with a plain message built from the file's contents.
- **ROUTED role** — a routed write takes a **shared** lock on byte 0 for the duration of that one operation (acquire at op start on the save tail, release at op end). Succeeds when nobody has U active; fails instantly when another instance holds U active → **refuse with a plain message, never last-writer-wins** (Architect condition 2). Two routed writers from two parent processes may hold shared locks concurrently — safe; WAL + `busy_timeout` serializes them at the DB layer.
- **Contents** (UX only, never authority): JSON `{ pid, host, user, universe_name, acquired_at }` — LibreOffice-style "who has it" metadata for the refusal message.

**One lock kills two hazards at once.** (a) It replaces the WAL-blind `BEGIN EXCLUSIVE` probe (`is_cuniverse_open_elsewhere`, migrate.rs:191-208 — false-negative on an idle open-elsewhere child, the refuted defense). (b) It closes the cross-process ledger race for free: `link_life`'s compaction runs only in a universe's ACTIVE instance (which holds exclusive byte 0); a routed `earned.jsonl` append holds shared byte 0 — the two conflict, so append-during-compaction across processes is structurally impossible, extending the in-process `FILE_LOCK` (link_life.rs:191) across the process boundary without touching the ledger code's hot path.

Path handling: `PathBuf` throughout, no separators in literals, NFC normalization at comparison sites only, `#[cfg(windows)]/#[cfg(unix)]` confined to `owner_lock.rs` — the macOS port cost is one small module.

### 1.7 Seamlessness contract (the ruling's adopted consequences)

No border-control dialogs anywhere in the edit class. The planet mark renders as **identity information** (tab strip, breadcrumb, pickers — the renderer already exists: `LibraryIcon kind='cuniverse'`, `MoveDialog.iconKind`), never as a warning modal. Refusal messages remain for exactly two states, both honest: `Owner::Unknown` (fail-closed) and owner-lock-held-elsewhere ("open in another window/instance"). `ChildNeedsUpgrade` is a door with a progress strip, not a wall.

---

## 2. The 22 register sites become Router callers

The register (write-surface map, re-derived at `7921e593`): 16 Class-A + 5 Class-B + 1 latent = 22 (+ Class-D attribution trust, + Class-E DB-only earned writers). The collapse comes from two central moves: **(i)** authorization at every site becomes the ONE call `route_write(app, path, intent)` (the evolved `require_own_library`), replacing today's per-site mix of `validate_path_in_any_library` / own-guards / prefix checks / nothing; **(ii)** `constellation_search_reindex` resolves owner+library Rust-side (kills Class D at search.rs:12251/12718, closes PJ-275) — so **all 22 frontend `reindexNote` call sites need zero change**, and every site whose only DB tail is the frontend-fired reindex needs zero site-specific routing code.

| # | Site | Router disposition | Wave |
|---|---|---|---|
| 1 | `write_note` | **Collapses entirely** — disk half already crosses; tail routes centrally inside the reindex command | 1 |
| 2 | `create_note` | Door: routed create-reindex + collision check against OWNER's index (replaces the skip at libraries.rs:1436-1451) | 3 |
| 3 | `create_folder` | **Collapses entirely** — auth swap only; no DB tail | 3 (picker) |
| 4 | `rename_item` | Routed `migrate_note_db_paths` + alias row + reindex on owner conn (all conn-parameterized already, libraries.rs:1532) | 3 |
| 5 | `resolve_structural_conflict` | **Collapses** — auth swap; tail rides central reindex | 1 |
| 6 | `move_item` | The transfer engine (§3, Wave 2) — dest refusal at libraries.rs:2673 becomes the door | 2 |
| 7 | `get_daily_note_path` | Routed create-index (today: created, indexed nowhere) | 3 |
| 8 | `quick_capture` | Same | 3 |
| 9 | `update_links_on_rename` | The multi-universe cascade (§5.2) — foreign-boundary drops become routing | 3 |
| 10 | `save_clipboard_image` | **Collapses entirely** — disk-only, auth swap | 1 |
| 11 | `delete_path` | Routed purge + `DeleteReason` archive in owner DB; trash → OWNER's `.trash`; restore surfaces extended (Whole-Ecosystem law) | 3 |
| 12 | `toggle_task` | One-line: the in-command reindex (tasks.rs:533) goes through `with_home_context` | 1 |
| 13 | `write_canvas` | **Collapses entirely** — unindexed by design | 1 |
| 14 | `create_canvas` | **Collapses entirely** | 1 |
| 15 | `import_execute` | Routed post-import indexing + TARGET universe's `file_kinds.json` (fixes importers.rs:828-830) | 3 |
| 16 | `import_with_canonical` | Same | 3 |
| 17 | `create_base` | **Collapses** — auth unification onto router; no DB tail | 1 |
| 18 | `update_base_columns` | **Collapses** — router auth closes the nested-cUniverse prefix hole (lens/query.rs:368) | 1 |
| 19 | `update_base_order` | Same (lens/query.rs:410) | 1 |
| 20 | `ensure_cid_cn_cmd` | **Boss-ruled row**: identity injection on *open* of a foreign note is an Automatic write → skip; proposed: defer the mint to the first UserCommanded write on that note, through the router (no spurious write at open — Editor-Surface Gate #2's spirit) | 1 |
| 21 | `sources_set_manual` / `content_type_set_manual` | Router auth + tail rides central reindex | 1 |
| 22 | `move_to_trash` (latent) | Owner-trash routing via router | 3 |
| — | `write_conflict_sidecar` | Sidecar stays beside the note (a safety artifact belongs with its note); router used for attribution only | 1 |
| E | `constellation_link_traverse` / confidence / archive / unarchive / dormant / decay; `mark_reviewed` / `snooze` / `dismiss`; `set_review_priority` | Route by row owner (from `source_path` / note path): UPDATE in owner DB, ledger append to owner's `earned.jsonl` under the shared owner-lock, pulse entry in owner's `review-pulse.json` (the `universe_root` param at review.rs:744 completes into Rust-side resolution). Earned life on linked notes stops silently dying | 1 |

Net: **11 of the 22 collapse to a one-line authorization swap with zero site-specific routing logic.** The real per-site work concentrates exactly where it belongs: move (transfer engine), rename (cascade), delete (purge+trash), the create class (collision+index), import.

---

## 3. Phase-by-phase build plan

Order is **routing-first** (deliberately different from the Architect §6.3 sketch, which is superseded by this assignment: the transfer engine must build on a proven Router, not precede it). Every step = one landable commit + verification clause; per-build diff-scoped safety-inspection; Boss tests every build before commit; measured on the 7,600+ universe where flagged. Interim guards (7921e593) come down door by door — the dissolution is stated per step.

### Phase 0 — pre-existing hazards (Architect condition 5; blocking)

- **§0.1 Retire `fs::copy` backup/restore in `federation/migrate.rs`** → rusqlite backup API (or checkpoint-TRUNCATE window). *Verify:* unit test — backup of a DB with a hot `-wal` restores to a consistent snapshot; existing migrate tests green.
- **§0.2 `owner_lock.rs`** + active-instance acquisition at universe activation, release on switch/exit. *Verify:* two-process test — second activation of the same universe refuses with who-has-it; `taskkill /F` releases the lock (next acquisition succeeds). *Boss-observable:* two Constellation instances on one universe → the second shows the plain-language message.

### Phase 1 — the Router core (zero behavior change; the vocabulary gate)

- **§1.1 `resolve_owner`** — longest-prefix + parent-walk fallback, normalized. *Verify:* unit tests incl. nested-cUniverse-under-active-root, NFC variance, case folding; active-path resolution is one compare (bench asserted).
- **§1.2 `VocabHandle` threading** through `index_note`'s parse chain, trigger-DDL generation, and **both fingerprint gates** (incoming_links_backfill.rs:49, links_backfill.rs:99). *Verify (the Phase-1 blocking gate):* red→green two-vocabulary harness — one probe note indexed under two differing registries; diff `note_links` + maintenance outcomes; active-path output byte-identical to pre-change (golden test).
- **§1.3 `LinkedDbPool`** — lazy open sequence §1.4, `InitScope::LinkedFullWithRegistry`, `ChildNeedsUpgrade` refusal, never-stamp rule. *Verify:* fixture-child tests — trigger regen from child vocab; assert fingerprint stamps untouched; pool open cost measured (<100 ms warm target); assert zero pool activity on boot (Rule 8).
- **§1.4 `route_write` / `with_home_context`** — guard→door conversion of `require_own_library` with `WriteIntent`; refusal strings rewritten (reserved for `Owner::Unknown` + `Automatic` + lock-held). No caller crosses yet. *Verify:* full suite green; routed-write vs own-write latency measured on the big universe (own-path delta ≈ 0).
- **§1.5 Rust-side attribution** — `constellation_search_reindex` resolves owner+library itself; Class D dies; PJ-275 closes; foreign resolution still refuses (doors not open). *Verify:* crafted wrong `library_name` from the frontend can no longer mis-file; nested-library attribution correct. *Boss-observable:* nothing changes — that is the test.

### Wave 1 — seamless edit / tag / property / task / earned life on linked notes

- **§2.1 Routed save tail** — reindex + re-embed route via `with_home_context(UserCommanded)` (the user's edit carries the intent; watcher/boot paths stay `Automatic`-refused). **Dissolves:** the save-tail foreign skip. *Boss test:* edit a linked note from the parent seat; open that universe standalone → search finds the new text, backlinks/sky current.
- **§2.2 Tag add** (closed-note branch + batch): rides central reindex; `writableSelection` re-admits federated members for tag. **Dissolves:** batch-bar tag exclusion. *Boss test:* tag a closed linked note; verify standalone.
- **§2.3 Property edit** — `update_note_property` own-guard → router (bases.rs:390). **Dissolves:** its foreign refusal. *Boss test:* base-cell edit on a linked note.
- **§2.4 Task toggle** — tasks.rs:533 reindex routes. **Dissolves:** the both-universes-stale defect. *Boss test:* toggle in parent; verify standalone.
- **§2.5 Earned life + review routing** (Class E): traverse/confidence/archive route by row owner; ledger appends under the shared owner-lock; review actions Rust-resolve the root. **Dissolves:** `set_review_priority`'s "permanently unindexed" refusal (review.rs:705-707); earned life on linked notes stops dying. *Boss test:* traverse + archive a link in a linked note; restart the child standalone → state persists.
- **§2.6 `ensure_cid_cn` deferred-mint** policy (Boss-ruled): no write at open; mint at first user-commanded write via router. *Verify:* Editor-Surface Gate #2 (no spurious write at Focus enter) on a linked note.
- **§2.7 Affordance + docs pass** — planet mark as identity info (tab/breadcrumb), refusal-string i18n ×15, help files + User Manual. *Boss test:* the full Editor-Surface Gate checklist run on a **linked** note (the checklist's own requirement: federated notes exactly as own notes).

### Wave 2 — move/copy: the journaled earned-cargo transfer

- **§3.1 `federation/transfer.rs`** — payload extractor (the earned-data census §5 payload: `note_meta` 3 cols, `note_links` 5 scalars — `created` exists only in the row, search.rs:470 — `review_schedule` 4 cols, ordered state/shape history, rename/import aliases, cid-keyed ledger lines, path-rewritten pulse entries; weight recomputed, never copied). Journal protocol per the engineering prior art (R5): **payload durable in the DESTINATION root before the fs move**; replay ordered before boot reconcile on both sides; keyed `(cid, path)`, refuse-and-report on duplicate cids; journal deletion is the commit point; invariant *may transiently exist in both, never in neither*. *Verify (Reproduce-First):* crash harness — kill between every seam; every window red→green; earned payload intact both directions.
- **§3.2 cid collision re-key** — destination cid index checked FIRST; on collision re-mint AND re-key the travelling earned rows in the same destination transaction (never sever — the anti-MIG-003-dup-rule); old→new mapping recorded in journal + ledger. *Verify:* harness case — move a note whose cid already exists at dest.
- **§3.3 The move door** — dest pickers re-list linked destinations (restore the `iconKind:'cuniverse'` producer at +layout.svelte:6921-6927; planet mark = identity, no confirmation modal); `move_item` dest goes through the router door; tail = transfer engine; suppression stamped for both roots. **Dissolves:** the move-dest refusal (libraries.rs:2673) and the Move-picker filter. *Boss test (tutorial):* move a probe note carrying earned links across universes; verify both universes standalone; verify undo-by-moving-back.
- **§3.4 Copy variant** — copy mints a fresh cid at dest and carries **content only** (earned life belongs to the original; Constraint as Design — duplicate-earned is a Boss decision if ever wanted). *Verify:* copied note indexes clean at dest; original untouched.
- **§3.5 PJ-227 one-shot** — the 13 residual foreign rows migrate to their owners' DBs via the transfer engine's row mover (sequenced behind the PJ-224 ruling it is filed against). **Dissolves:** the phantom-row exemption; boot reconcile's `foreign_rows` skip becomes an assertion (foreign rows in the active DB = reportable bug).

### Wave 3 — create / rename / delete-in + cascade healing + PJ-224

- **§4.1 Create-in doors** — create pickers list linked destinations (planet-marked); `create_note` collision check reads the OWNER's index; routed create-reindex; quick capture / daily note / import gain the routed indexing they lack today; importer reads target `file_kinds.json`. **Dissolves:** picker filters (`LibraryPicker items`, `buildUniverseFolderEntries`), the create-reindex skip. *Boss test:* create a note in a linked library from Ctrl+N; it appears in search immediately, both seats.
- **§4.2 Rename-in + cross-universe cascade healing (PJ-253 family)** — routed `migrate_note_db_paths` + alias row + reindex on owner conn; the referrer seek generalizes across `main` + ro-attached `cuN.note_links.target_base` (read suffices for the SEEK); each referrer's rewrite reindexes routed to ITS owner; `retarget_registered_libraries` writes the child's `libraries.json` under the owner lock. Honesty stated in UI/help: healing reaches the *visible* federation set only (federation is directional). **Dissolves:** the cascade foreign-boundary refusals (libraries.rs:6932-6942, 7071-7096, 7199-7203) and PJ-207 §15's refusal. *Boss test:* rename a linked note that an own note links to, and vice versa; both sides' links heal; verify standalone.
- **§4.3 Delete-in door** — routed purge + archive in owner DB; trash to OWNER's `.trash`; **trash listing/restore surfaces extended to owner roots in the same commit** (Whole-Ecosystem law). **Dissolves:** the delete guard for plain notes (foreign-LIBRARY-under-path refusal stays — deleting someone's library is a different act). *Boss test:* delete + restore a linked note from the parent seat.
- **§4.4 PJ-224** — the ordinary search box: **Boss ruling first, asked not assumed**; if yes, SearchHub routes through the existing federated read path (attach layer already serves it).
- **§4.5 Close-out** — PJ-219 watcher-adopt ruling (should an external edit to a linked note route its reindex home? filed as the one deliberate `Automatic` exception for the Boss to rule); concept-paper 25-federation §3/§10 amendment; repeal table (Architect §5) confirmed; orientation bump; whole-app safety-inspection sweep (the migration-close cycle).

### Standing obligations across all phases

- **Measurements before each commit that touches the write path** (verdict condition 10): steady-state routed-write latency, first-write pool-open cost, child WAL growth under sustained parent writes — on the 7,600+ universe.
- **PJ-262 sequencing** surfaced to the Boss before Wave 2 starts: shipping the Living-Link disk layer first shrinks §3.1's payload extractor (the design's riskiest module). Waves 0–1 are independent of that decision and proceed regardless.
- **Guard-dissolution ledger:** each step's commit message names the guard it retires; any guard not named stays up. Never all at once.

---

## 4. What this design refuses (Constraint as Design)

No provenance columns in any DB (Option B's ghost-row class never exists). No prewarming of child DBs (first-write lazy is the contract). No background sync contract (one truth per universe, readers read it). No new frontend tab state (`OpenTab` unchanged — the editor already writes; correctness lives in the Rust tail). No confirmation dialogs on the edit class (seamless is the ruling). No merge semantics under contention — refusal with a face and a name, always.

Files touched (core): `src-tauri/src/federation/router.rs` (new), `federation/owner_lock.rs` (new), `federation/transfer.rs` (new), `search.rs` (pool in `SearchState`, routed reindex, `InitScope::LinkedFullWithRegistry`, vocab threading), `link_types.rs` (`VocabHandle`, per-root load), `libraries.rs` (guard→door, cascade fan-out, owner-registry attribution, trash routing), `incoming_links_backfill.rs` + `links_backfill.rs` (gate threading), `review.rs`, `link_life.rs` (rooted appends), `canonical.rs` (deferred mint), `bases.rs`/`lens/query.rs`/`tasks.rs` (auth unification), `federation/migrate.rs` (backup API), `+layout.svelte` (affordance pass), i18n ×15, help/manual.

==========================================================================================
## DRAFTS :: experience
==========================================================================================

All sources verified. Here is the design draft.

---

# MIG-111 — THE SEAMLESS EXPERIENCE · Frontend Design + Plan (Phase-2 input)

**Working on:** the user-visible surface of full cross-universe operations — what the umbrella looks like when it is ONE space (MIG-111 Option A, route-to-owner; Architect doc §6.2/§6.3 answered here).

**The concept (the horse):** *The umbrella is one knowledge space; a note's home universe is a fact about the note, not a wall around it.* Therefore the interface never asks permission to cross, never warns at a border, and never dresses a linked note differently — it states the note's home the way it states its library: quietly, in the same breath. All safety machinery (router, owner locks, journaled transfers, cid protocol) is invisible; the system speaks only in the three rare moments where honesty requires it, and then in plain words.

Design maxim, applied throughout: **identity is information; ceremony is friction.** The planet mark (the existing `LibraryIcon kind="cuniverse"` orbit glyph — `src/lib/components/LibraryIcon.svelte:30-35`, the ONE shared mark per the Whole-Ecosystem rule written into that file's header) is the entire visual vocabulary of federation. No banners, no confirmation dialogs, no "(linked)" suffixes, no color-coded alarm.

---

## 1 · The experience, surface by surface

### 1.1 Pickers — the whole umbrella, one list

Every picker that chooses a place (Move, new-note-from-template destination, Ctrl+N library picker, importer target) lists the **entire umbrella**: own libraries first (unchanged), then each linked universe as a group — a non-selectable group row bearing the planet mark + the universe's name, its libraries and folders indented beneath with their normal building icons and own colors. Picking a destination inside a group **is** the deliberate act; there is no follow-up confirmation (adopted consequence: no border-control dialogs — this refines Architect §6.2, which had floated "planet icon + confirmation"; the confirmation half is dropped, the Boss ratifies this in the Phase-2 sitting).

The machinery for this is a restoration, not an invention — verified in source:
- `MoveDialog.svelte:27` still types `iconKind?: 'root' | 'library' | 'cuniverse'`; the renderer survived PJ-235.
- `buildUniverseFolderEntries` (`+layout.svelte:6911-6935`) is where the producer was removed — the `!isChildUniverseLib(l.path)` filter comes out, and entries gain a `groupUniverse?: string` so linked libraries render under their universe row (grouping convention already exists: `bookmarkLocation`'s "cUniverse / library / folder" breadcrumb, `+layout.svelte:6528-6549`).
- `LibraryPicker` already takes `items` from the caller by design (`LibraryPicker.svelte:7-17` documents exactly this seam); the caller passes the grouped umbrella instead of `ownUniverseLibraries` when its wave ships.
- The Rust twin `list_universe_folders` un-narrows in the same wave (evidence, frontend-model §3).

One deliberate asymmetry survives, per the amended 2026-08-10 ruling (Architect §5): **deliberate choice spans the umbrella; automatic targeting stays home.** Default destinations the user never chose per-gesture — quick-capture inbox, daily-note folder — keep resolving own-scope unless the user explicitly configures a linked location in Settings.

### 1.2 Editing a linked note — zero ceremony, literally zero frontend change

The evidence's central finding (frontend-model §1): a federated note's tab is already structurally identical to an own note's — `OpenTab` has no federated field, the editor already saves, the save pipeline already runs. **Seamlessness is achieved by adding nothing.** No banner, no read-only veil, no mode. The user opens the note, types, and it saves — correctness moves entirely to the Rust routed tail (Option A §1.1.E: `constellation_search_reindex` resolves owner+library itself, so all 22 frontend call sites need zero changes).

The one perceptible seam Rust may introduce — the lazy pool-open on the *first* routed write to a universe — is invisible in the warm case (~tens of ms, post-debounce, off the keystroke path). The rare cold case (child DB needs the §C schema-only migrate: seconds to minutes) surfaces as a status-bar strip, the app's established idiom for background work — a fifth consumer of the parameterized `JobProgressStrip` (`+layout.svelte:10474-10489`), label: *"Preparing '{universe}' for its first change from here…"*. The strip is the visible exception, exactly as the Architect priced it.

One genuinely new frontend piece in this class: **the link-type palette must offer the note's home vocabulary** (Option A §2 Link (a) — a type unknown to the source's registry silently isn't a link). The palette component gains a vocabulary source keyed by the note's home universe (fetched once per universe, cached, never on a keystroke). For own notes: unchanged.

### 1.3 Tabs — the quiet planet

The tab already shows `tab.libraryName` as a chip with the library's color (`+layout.svelte:8589, 8597-8598`). For a note whose home is a linked universe, the chip gains a small planet mark inline-start of the library name (LibraryIcon, ~9px, the federation indigo `#6366f1` already used at `+layout.svelte:8346` and `DashboardView.svelte:211`), and the chip's `title` tooltip reads *"Lives in {universe}"*. Nothing else changes: same color, same typography, same close button. Derivation is `isChildUniverseLib(tab.libraryPath)` — an O(1) set lookup over state that already exists in `+layout.svelte` (the module that renders tabs), so **no `OpenTab` field, no IPC, no hot-path cost**.

Housekeeping fix folded in: `libraryColorMap` is keyed by library *name* (`+layout.svelte:8589`) — two same-named libraries in different universes collide. Re-key by normalized `libraryPath` with a name fallback (small, mechanical, Whole-Ecosystem: tab chip + sidebar + picker rows share the map).

### 1.4 Status bar — the breadcrumb states the home

Today the left side reads `{libraryName} · {name}` (`+layout.svelte:10462-10467`). For a note living in a linked universe it becomes:

> ⟨planet⟩ {universe} · {library} · {note}

— the same segment style, `dir="auto"` per segment like today's items, planet mark sized to the 24px bar. For own notes: **unchanged** — the active universe is the ambient home; naming it would be noise (Form-Aligns-To-Purpose: the segment exists only where it informs). This mirrors the app's one existing breadcrumb convention (`bookmarkLocation`, which already prefixes the cUniverse name).

### 1.5 Sidebar and the rest of the read layer — already correct

The sidebar's grouping (planet-marked universe rows with nested libraries, `+layout.svelte:8330-8387`) is already the design; it stays. Predicates (`isChildUniverseLib`, `childUniverseLibPaths` and family) are **retained and re-purposed**: from write-boundary enforcers to identity/grouping providers. One drift fixed in passing: `LibrarySwitcher.svelte:108` renders the full federated list under an "Own Libraries" label (evidence, frontend-model §2) — filter it with the same predicate its own Child-Universes section already uses.

### 1.6 Batch bar, context menus, bases

- `writableSelection` / batch-disabled predicates (`+layout.svelte:5115-5116, 8441-8443`) widen wave-by-wave as each verb's routed tail ships — mixed own+linked selections simply work.
- Tree/list context menus already offer every verb on federated items (frontend-model §2, "sites that DO NOT branch") — those verbs stop lying the moment routing ships beneath them. No menu changes.
- The workspace-bases "no context menu" carve-out (`+layout.svelte:8264-8287`) lifts when base ops route.
- Interim-guard refusal strings ("reads but never writes…", `libraries.rs:330-343`) retire door-by-door per Architect §5; the only permanent refusal is `Owner::Unknown`, reworded: *"Constellation can't tell which universe owns that location, so nothing was changed."*

### 1.7 Second screen — nothing, deliberately

Verified (frontend-model §4): the second screen is read-only by window (`ssReadOnly = true`, `SecondScreenPage.svelte:205-208`), loads the federated list, syncs federated saves identically, and forwards all write verbs to main (`requestNoteActionOnMain` → `handleOrgNodeMenuAction`). So it inherits every new power automatically — a right-click move on a federated note there opens main's now-umbrella-wide Move dialog. **No code.** Planet marks are *not* added to SS surfaces: SS has no `childUniverseLibPaths` (grep-verified in evidence), the marks are a write-context aid and SS has no write context; adding the side-map to the sync payload is filed as an optional follow-up, not built (Constraint as Design).

---

## 2 · Disposition table — every frontend own-vs-federated branch

| Site (evidence, frontend-model §2) | Disposition |
|---|---|
| `OpenTab` shape | **Unchanged** — no new field |
| `deriveLibraryForPath`, `tab.libraryName` | **Unchanged** — already federated-correct |
| `isChildUniverseLib` / `childUniverseLibPaths` / `getChildUniverseLibs` | **Retained** — re-purposed to identity/grouping (feed tab planet, status-bar segment, picker groups) |
| `ownLibraries` (sidebar grouping) | **Retained** — display grouping only |
| `ownUniverseLibraries` (`1617-1623`) | **Narrowed role**: stays for *automatic* targeting defaults; Ctrl+N picker stops consuming it at Wave 3 (deliberate choice → grouped umbrella) |
| `buildUniverseFolderEntries` filter (`6921`) + Rust `list_universe_folders` | **Removed** at Wave 1 — cuniverse branch restored, grouped |
| `LibraryPicker items` (`10272`) | **Widened** at Wave 3 — grouped umbrella, planet-marked group rows |
| `ImporterModal` filter (`10197-10203`) | **Widened** at Wave 3 |
| `writableSelection` + batch predicates (`5115`, `8441/8443`) | **Retired** wave-by-wave as verbs route |
| Workspace-bases no-context-menu (`8264-8287`) | **Lifted** when base ops route |
| Sidebar planet groups, `revealInTree`, `bookmarkLocation`, `wiwTitle`, federation warnings badge | **Unchanged** — already the quiet-identity design |
| `LibrarySwitcher` "Own Libraries" labeling drift (`:108`) | **Fixed** in F0 |
| `libraryColorMap` keyed by name | **Re-keyed** by path in F0 |

---

## 3 · The honest exceptions — the three moments the system speaks

Constellation voice: plain words, no jargon (no "cid", "journal", "instance", "WAL"), name the thing that matters, end on the standing convention *Nothing was changed / Nothing was lost*. All via toast/receipt idiom, all through `$t()` ×15.

**1. Two-instance refusal** (`federation.instanceBusy`) — a routed write finds the note's universe open as active in another Constellation window (the owner-lock refusal, Architect condition 2; refuse, never last-writer-wins):

> **"'{universe}' is open in another Constellation window right now. To keep both windows safe, this change wasn't made — finish your work there, or close that window and try again. Nothing was changed."**

**2. Fresh identity on arrival** (`federation.freshIdentity`) — a cross-universe move hits a cid collision; the system re-mints and re-keys the earned rows in the same transaction (Architect condition 4). Auto-resolved, so it's a *receipt*, never a question:

> **"A note in '{universe}' already carried this note's internal identity, so Constellation issued it a fresh one. Everything it has earned — links, history, review record — came along. Nothing was lost."**

**3. Interrupted transfer, resumed at boot** (journal replay). While finishing, the status-bar strip shows *"Finishing an interrupted move…"*; then one receipt, whichever is true:

> Rolled forward — **"A move into '{universe}' was interrupted last time. It has now been completed: '{note}' is in place, with everything it earned. Nothing was lost."**
> Rolled back — **"A move into '{universe}' was interrupted before anything had traveled. '{note}' stayed where it was. Nothing was lost."**

Supporting strings (same namespace): `federation.movingLabel` *"Moving '{note}' to {universe}…"* (transfer strip); `federation.preparingUniverse` *"Preparing '{universe}' for its first change from here…"* (§C-migrate strip); `federation.livesIn` *"Lives in {universe}"* (tab tooltip); `federation.renameHealed` *"Also updated {n} linking notes in '{universe}'."* (rename receipt — count via `$tn('plurals.notes', n)` per the MIG-087 plural rule).

---

## 4 · i18n ×15 and RTL

- All new strings land under the existing `federation.*` namespace (`en.json:3522`) in **all 15 locales in the same commit as their surface** (parity guarded by `scripts/i18n-parity.mjs`). No new nouns invented: group rows show the universe's own name; where a label is needed the established "Linked Universe" vocabulary is reused (`en.json` `linked_universe`, Style Setter).
- RTL: every name segment uses `dir="auto"` (the existing `sb-item` / `tab-lib-name` pattern); the planet mark is non-directional (orbit ellipse — no mirroring); the `·` separators are direction-neutral; picker group indentation must use logical properties (`padding-inline-start`) — a verification item in each picker phase, since MoveDialog's depth indent predates the RTL engine. Arabic pass is an explicit Boss test in F4.

## 5 · Performance and platform guarantees

- **Zero typing cost:** every identity mark is a `$derived` O(1) lookup over `childUniverseLibPaths` (already in memory in the only module that renders these surfaces); zero `invoke()` on any hot path; the link-type vocabulary for a foreign note is fetched once per universe on palette open, cached.
- **Rule 8 / boot:** no boot recompute anywhere in this design; child DBs open lazily on first routed *write* only (never on tab open, never on boot); pickers group from lists already loaded.
- **macOS-neutral:** frontend paths already normalize separators (`replace(/\\/g,'/')` throughout); the owner lock is Rust-side (held OS handle — `LockFileEx` share modes on Windows, `flock` behind `#[cfg]` on macOS, per the engineering prior-art R7); no keyboard changes in this design.

---

## 6 · Phase-by-phase frontend plan — one commit each, Boss-observable each

Every Boss verification below is delivered through the mandated pipeline (tutorial-auditor → ui-inspector → Boss) at build time; the lines here are the observable contract, not the tutorial.

**F0 — The quiet identity layer** *(display-only; ships before any door opens — zero write risk).*
Planet mark + universe segment in the status bar for federated notes; planet mark + "Lives in {universe}" tooltip in the tab chip; `libraryColorMap` re-keyed by path; `LibrarySwitcher` label drift fixed; `federation.livesIn` ×15.
*Boss sees:* open a note from a linked universe → tab chip shows the small planet before the library name; status bar reads planet + universe · library · note. Open an own note → nothing changed anywhere.

**F1 — The Move door** *(lands with Rust Wave 1: transfer engine).*
Move + template-destination pickers re-list the umbrella (filter removed, `iconKind:'cuniverse'` group rows restored, grouped entries); transfer strip (5th `JobProgressStrip` consumer); the three exception messages + receipts + `movingLabel` ×15; interim move-refusal string retires.
*Boss sees:* right-click a note → Move → the picker shows own libraries, then each linked universe with the planet mark and its folders beneath; pick one → brief strip → the note sits in the linked universe's tree, opens with links/history intact; move it back the same way. Kill the app mid-move (staged test) → next launch shows the finishing strip and the honest receipt.

**F2 — Editing in place becomes true** *(lands with Rust Wave 2: routing layer).*
Frontend near-nil by design: batch bar widens for tag/property verbs; Base-cell edit un-refuses; link-type palette reads the note's home vocabulary; `preparingUniverse` strip wired.
*Boss sees:* type in a linked note — saves with zero ceremony; add a tag from a list surface; add a typed link and the type list is the linked universe's own; open that universe standalone later — every edit is there and indexed.

**F3 — Create, rename, delete across the umbrella** *(lands with Rust Wave 3).*
Ctrl+N `LibraryPicker` gets the grouped umbrella; `ImporterModal` widens; rename receipt gains the `renameHealed` cross-universe line; workspace-bases context-menu carve-out lifts; remaining interim refusals retire.
*Boss sees:* Ctrl+N straight into a linked universe's library; rename a note that linked-universe notes cite → receipt says how many linking notes were updated and where; delete a linked note → it lands in *that* universe's trash and restores from there.

**F4 — Close-out sweep.**
Second-screen end-to-end verification on federated notes (forwarded verbs now umbrella-wide — no code expected); full Arabic/RTL pass over every new surface; i18n parity ×15; help files + User Manual ×15; diff-scoped safety inspection; orientation + PJ-ledger reconciliation.
*Boss sees:* in Arabic, the status-bar breadcrumb, picker groups, and toasts all read correctly RTL; on the second screen, right-click → Move on a federated note offers the whole umbrella.

---

**Open items this design hands to Phase 2 (Boss decisions):** (a) ratify dropping the per-crossing confirmation (this doc's §1.1, refining Architect §6.2); (b) confirm Wave order F1→F3 matches the recommended Rust wave order (Architect §6.3); (c) PJ-224 (does the ordinary search box federate?) remains gated and untouched by this design.

**Key verified sources:** `docs/migrations/PJ-235-federation-boundary/MIG-111-ARCHITECT.md` (all §§); `MIG-111-ARCHITECT-EVIDENCE.md` §MAPS::frontend-model, §OPTIONS::A §1.3/§2, §MAPS::write-surface §3-4, §MAPS::identity-links; source spot-checks this session: `src/routes/+layout.svelte:8330-8420, 8584-8605, 6905-6950, 6528-6549, 10461-10505`; `src/lib/components/LibraryIcon.svelte`; `src/lib/components/LibraryPicker.svelte:1-60`; `src/lib/components/MoveDialog.svelte:27`; `src-tauri/src/libraries.rs:295-345`; `src/lib/i18n/en.json` (`federation.*` at 3522).

==========================================================================================
## ATTACKS :: core
==========================================================================================

All verification complete. Composing the adversarial report.

---

# ADVERSARIAL ATTACK REPORT — MIG-111 Plan Draft

## VERDICT: **SOUND-WITH-AMENDMENTS**

The draft is a faithful translation of the Architect's Option A under the governing ruling, and its citation hygiene is excellent: **every file:line claim I checked verified in source this session (~35 citations, zero fabrications)** — `require_own_library` libraries.rs:295-345 with the trap comment at 312-318 and known-limit at 320-323; `SearchState` search.rs:1341-1416; `schema_gate` 1448-1463; `REGISTRY`/`snapshot`/`load_active` link_types.rs:351/498/522; both fingerprint gates; `constellation_search_reindex` 12251; `find_universe_root` attach.rs:105-118 (private today — promotion is real work); `FILE_LOCK`/`append` link_life.rs:191/205; `is_cuniverse_open_elsewhere` migrate.rs:191-208; `fs::copy` backup migrate.rs:80-82; `InitScope::ForeignSchemaOnly` 4571-4589; tokenizer 1558-1581; generation discipline 11537/11635-11648; `init_db_schema_only` exists (search.rs:4597); the 22-site register (16 A + 5 B + 1 latent, evidence §2); `validate_base_path` prefix hole (bases.rs:18-43); `write_note`'s federated disk half (libraries.rs:981); pragmas match (4620, 4641); the removed `'cuniverse'` picker branch (+layout.svelte:6921-6928); PJ-227-blocked-on-PJ-224 in-code (libraries.rs:6932-6935); `mark_reviewed`'s `universe_root` param (review.rs:744). The routing-first phase order, the never-stamp rule, the (cid,path) journal keying, the D.9/D.10 standing obligations, and PJ-224 asked-not-assumed are all genuinely scheduled.

But the attack found six defects that must be amended before Boss approval — two of them in the Router core's own correctness story.

---

## HIGH — blocking amendments

**H1 — §1.5 misdiagnoses the vocabulary-gate mechanism, and its sketched fix is wrong at both cited sites.** The draft claims routed writes "silently skip incoming/outgoing maintenance while `is_built` stays true." Verified false as a mechanism: the **write-side** maintenance gates on `is_built` *alone* — version-only, deliberately fingerprint-free per the 2026-08-01 inspection doc at incoming_links_backfill.rs:60-73 ("WRITERS ask do these columns EXIST → `is_built` (version only)"), and the live call sites confirm it (search.rs:12712, 12419, 11037). So on a *built* child DB, routed maintenance would **RUN, not skip** — and `maintain_incoming_after_save`'s recompute SQL is generated from the active registry's IN-list + rank CASE (incoming_links_backfill.rs:40-41). Result: **parent-vocabulary aggregate values written into the child's rows, which the child's own boot then serves as trustworthy** (its stamp matches its own registry) — silent corruption of the PJ-230/232 class, strictly worse than the skip the draft describes. Separately, the sketched one-line replacement is the `is_stamped` shape; links_backfill.rs:99 is `is_needed` (`!=`, no `is_built` conjunct) — applied literally the sketch inverts it. **Amendment:** the threading enumeration must name the maintenance *computation* functions (`maintain_incoming_after_save`, the sky write-time maintenance at search.rs:12759+, the rank-CASE/IN-list generators), not just "both gates"; write-side keeps `is_built` semantics; and the Phase-1 red→green harness must diff **aggregate values**, not only `note_links` rows, or it can pass green while the corruption path stays open.

**H2 — `resolve_owner` as specced contains an authorization hole and cites the wrong root source.** The fallback — parent-walk `find_universe_root`, "no root found → Err" — is fail-closed only against *no* universe.json. Against a universe.json that is **not in the federation** (any unrelated universe on disk, a crafted or buggy frontend path), the walk returns its root → `Owner::Linked` → the router opens and **writes a universe the user never linked**. Today `require_own_library` refuses that path; the router as written admits it. The primary enumeration is also mis-sourced: `resolve_universe_libraries` (universe.rs:1513) returns **libraries**, not universe roots — the exact known-limit at libraries.rs:320-323 ("foreign set held library roots, not cUniverse roots") the draft claims longest-match fixes. **Amendment:** enumerate universe roots from the federation tree itself (the `universe.json` children, recursive — the attach layer's `cu_roots` source), and the parent-walk result must be intersected with {active root ∪ federation roots}; outside that set → Err, fail-closed.

**H3 — §1.3's fast path contradicts §1.2's own rule.** "One cached-string prefix compare" resolves `Active` incorrectly whenever a linked universe nests **under** the active root — the documented trap at libraries.rs:312-318 that §1.2 correctly says longest-match subsumes. One compare cannot implement longest-match. **Amendment:** fast path = active-root prefix match **plus** a check against the cached (usually empty) set of federation roots nested under the active root; state it, bench it, and add the nested-child case to the §1.1 unit tests as a *routing* assertion, not just a resolution one.

**H4 — The Windows owner-lock file is unreadable exactly when its contents are needed.** `LockFileEx` locks are **mandatory**: an exclusive lock on byte 0 blocks other processes' reads of that range, so a routed writer that fails the shared-lock probe **cannot read the JSON at byte 0 to build the who-has-it refusal message** (`ERROR_LOCK_VIOLATION` on the read). The draft even cites SQLite's lockingv3 — whose locking range sits at a reserved offset past 1 GB for precisely this reason. **Amendment:** lock a reserved high-offset byte range (or lock `owner.lock` while keeping metadata in a sibling `owner.json`); the two-process test in §0.2 must assert the refusal message actually renders the holder's identity on Windows.

**H5 — "Two routed writers … safe; WAL + busy_timeout serializes them at the DB layer" is false for earned mutations.** `constellation_link_traverse` is read-modify-write — SELECT `traversal_count` → compute weight in Rust → per-id UPDATE (verified search.rs:9749+, deliberately two-step for the `ln()` reason) — serialized today only by the process-local `state.db` mutex. Two parent processes routing into the same child hold compatible shared locks (the design permits it) and race to a **lost traversal/confidence update**. This is the evidence file's Option-B condition A3, which the Option-A verdict never carried only because Option A's attack predated the two-shared-writers decision — the draft introduced the concurrency and dropped the discipline. **Amendment:** every routed earned mutation runs `BEGIN IMMEDIATE` + bounded retry (or a single-statement arithmetic UPDATE); add a two-process concurrent-traverse case to the harness.

**H6 — A user-commanded write surface is missing from the register table: the shape/stage family.** `apply_shape` → `set_note_shape`/`clear`/`revert` (shape.rs:168) is own-guarded today (evidence §2 "own-guarded already — for contrast") and appears in **no row and no wave** — yet `shape_history` is earned cargo in the §3.1 payload, and the draft's own Wave-1 exit test (§2.7: full Editor-Surface Gate on a linked note) exercises **checklist #5, stage promote**. As written, Wave 1 either fails its own exit test or ships a seamlessness hole the ruling forbids. **Amendment:** add the shape family as a Wave-1 row, and sweep the rest of the own-guarded contrast list for the same gap (Whole-Ecosystem Fix Law — the register counted *federated-authorized* writers; the seamlessness contract governs *all user-commanded* writers).

## MEDIUM

**M1 — Architect condition 6 (watcher + suppress universe-aware) is not scheduled.** The Architect said "verify, don't assume, in Plan"; the draft never does. The watcher **does** watch federated libraries (evidence :102, `+layout.svelte:2944` over `$libraries` = `resolve_universe_libraries`), so an unsuppressed routed save fires the parent's own watcher → adopt/remount churn on the very tab that saved. Suppression is stamped only in §3.3 (move). Amendment: a §2.1 verification clause that `gate_write`'s path-keyed suppression covers routed paths, asserted in the Wave-1 harness.

**M2 — Boss gates sequenced too late.** Architect §5 says the repeals table is confirmed "Phase 2, first item"; the draft parks it at §4.5 close-out — after every repealed contract has already been dismantled in code. The two-instance refusal policy (Architect §6.4) ships in Phase 0, and the wave-order change ("superseded by this assignment" — the assignment ruling does not actually rule wave order), without explicit Boss line-items. Amendment: repeals table + two-instance policy + wave order + the WriteIntent table (incl. row 20 and PJ-219, per condition D.7 — the draft's "Boss-ruled row" label on a row whose content says "proposed" is exactly the ambiguity D.7 exists to kill) are confirmed at **plan approval**, before §0.1.

**M3 — The child schema-gate's third state is unhandled.** `schema_gate` returns `stamp_absent` on a missing marker (search.rs:1461); the draft's §1.4 step 3 covers match and mismatch only. If the pool stamps an absent marker on a child (the active-path behavior), it blesses a possibly-old schema as current and the child's own migration never runs — the migrate.rs:88-95 shape through a different door. Amendment: extend the never-stamp rule to `search.version`; absent marker on a child → treat as `ChildNeedsUpgrade` (or verify structure first), never stamp.

**M4 — D.5 deviation unacknowledged.** Evidence condition 5: pool open "never on the save tail … link-time or first foreign-tab-open with progress UI." The draft moves only the §C *migrate* off the save tail; the ordinary open sequence (rw open, tokenizer, vocab load, trigger-DDL completeness check) runs on the first routed write — the save tail — with only a *warm* <100 ms target measured. Amendment: open at first foreign-tab-open per the condition, or justify the deviation with a measured **cold**-open bound on a 7,600-note child.

**M5 — Deferred-mint (§2.6) breaks the cid-keyed machinery it feeds.** A foreign note opened but never user-written has no `cid_cn`; §2.5's ledger appends are cid-keyed and §3.1's journal is keyed (cid, path). Is a traverse (DB-only UserCommanded write) a minting event — a file write the Editor-Gate spirit forbids at that moment — or does the ledger line go keyless? Undefined seam; define mint semantics for DB-only gestures and for transfer of cid-less notes.

**M6 — §4.1 and §4.2 are not single landable commits.** §4.1 bundles six surfaces (pickers, collision, create-reindex, quick capture, daily note, importer + file_kinds); §4.2 bundles routed rename + the cross-universe cascade + writing another universe's `libraries.json`. Split each; every fragment has its own verification clause.

**M7 — Embedding compatibility never scheduled.** §2.1 routes "re-embed" into child `note_embeddings`; evidence honesty item 8 explicitly flags parent-process embedding model/dimension compatibility as "not verified … a Plan-phase verification item." The draft drops it.

## LOW

**L1** — §4.2 partial-heal accounting: a referrer's owner active elsewhere → per-universe refusal mid-cascade; the cascade must count and surface refused universes, and the seek's ro-ATTACH cap (25) is a second honesty bound not stated. **L2** — `ARCHITECT-INPUT-federated-write-sites.md` is **0 bytes**; the register the draft calls its base exists only inside the evidence file §2 — re-bank it (the 7921e593 commit message claims it banked). **L3** — search.rs:12266's in-code "21 frontend callers" vs the register's 22 — reconcile when touched. **L4** — §1.6's cross-process ledger claim is sound for append-vs-compaction (compaction's sole runtime caller is the restore thread, link_life_restore.rs:165 — active-instance-only holds today) and append-vs-append is line-atomic (one `write_all` per line, link_life.rs:197-205) — but state the "routed paths never call `maybe_compact`" invariant explicitly so it survives refactors.

## What the attack could NOT break

Route-to-owner as the one choke point; closure-scoped `HomeCtx` with the pair-API confined to the transfer engine; never-stamp for fingerprints; owner-respecting schema posture (no rename-aside of child DBs); the (cid,path)-keyed journal with dest-durable-before-fs-move and both-sides replay-before-reconcile; lazy pool with generation discipline and zero boot cost; Class-D death via Rust-side attribution with zero frontend churn; copy-carries-content-only; the seamlessness contract's two honest refusal states; OFD-not-classic-fcntl on macOS; `#[cfg]` confinement to one module. The 11-of-22-collapse claim held up under the register audit. No half-migrated seam beyond H6: each wave leaves a coherent guard set, and the guard-dissolution ledger is the right discipline.

**Amended verdict path:** fold H1–H6 into the Plan text, add M1–M7 as scheduled steps/gates (M2's Boss confirmations first), and this Plan is fit to put before the Boss.

==========================================================================================
## ATTACKS :: experience
==========================================================================================

All verification complete. Here is the adversarial report.

---

# MIG-111 Plan Draft — Adversarial Attack Report

**VERDICT: SOUND-WITH-AMENDMENTS** — the surface design (pickers, quiet identity, three honest exceptions, wave shape) is aligned with the governing ruling and its source citations are almost uniformly accurate. But the draft is **not landable as scheduled**: it silently diverges from the binding Concept-Panel requirements it selectively cites, mis-sequences against the panel's hard preconditions, contains two data-loss-shaped UX flows, and one Boss-test clause describes an app surface that does not exist.

## A. Source claims verified — accurate (draft's credibility base)

Verified this session, all correct: `LibraryIcon.svelte:30-35` (planet glyph + Whole-Ecosystem header); `MoveDialog.svelte:27` (`iconKind` incl. `'cuniverse'` survived PJ-235); `LibraryPicker.svelte:7-17` (caller-supplied `items`); `+layout.svelte` — `isChildUniverseLib` 1602-1608, `ownUniverseLibraries` 1617-1623, `writableSelection` 5115-5116, `bookmarkLocation` 6528-6549, `buildUniverseFolderEntries` filter 6921 (+ producer comment 6911-6920, planet case removed 6925-6927), bases carve-out 8264-8287, sidebar cU groups 8332-8387 with planet `#6366f1` at 8346, batch buttons 8441/8443, tab chip 8589/8597-8598, status bar 10462-10467, ImporterModal 10197-10203, LibraryPicker mount 10272, JobProgressStrip ×4 at 10478-10489; `LibrarySwitcher.svelte:108` drift (renders full `$libraryStats` under "Own Libraries"); `libraries.rs:330-343` refusal strings; `en.json` `federation.*` at 3522, `linked_universe` at 3704; `DashboardView.svelte:211`; `SecondScreenPage.svelte:205-208`. The `OpenTab.libraryPath`-derivation claim and "22 reindex call sites need zero changes" are evidence-consistent.

## B. BLOCKING findings

**B1. The draft cherry-cites and then ignores the binding Concept Panel.** `docs/migrations/PJ-235-federation-boundary/MIG-111-CONCEPT-PANEL.md` (same directory, newer than the Architect doc) binds the Plan to **37 requirements (R1–R37)** and **3 chair conflicts (C1–C3) requiring Boss rulings**, with hard sequencing preconditions "before ANY door opens: R1, R3, R5, R7 (crash windows red→green), R11, R35, R36" (panel :371). The draft cites "R7" (proving its author saw the panel) yet contradicts or omits R6, R10, R16, R18, R22, R23, R33, R35, R36 and never surfaces C1–C3. And the citation itself is wrong: **"the engineering prior-art R7" does not exist** — the lock design is **R5** (panel :304; also Inspector req 8 at :103); R7 is the transfer engine. "Prior-art" mislabels a panel requirement. Grep of both Architect docs finds no LockFileEx/flock outside the panel.

**B2. The two-instance refusal is a save-time refusal after accepted typing — forbidden by R6, and C2 is unresolved.** Draft §3.1 fires when "a routed write finds the note's universe open as active in another window" — i.e., after the user typed and the debounced save ran. R6 (panel :305): "Refusal resolves BEFORE input, never after: lock at open or first-edit-intent; a locked-universe note opens read-only with a passive one-line explanation; save-time refusal after accepted typing forbidden; typed input preserved in a recoverable buffer." The draft's message says "this change wasn't made" with no word on where the typed content goes. Amend: lock probe at note-open/first-edit-intent; the toast becomes the fallback for the race only, with the typed-buffer recovery specified; carry C2 (persistent passive indicator vs transient message) to the Boss.

**B3. Child-DB preparation on the debounced save violates R10 and collides with PJ-103.** Draft §1.2 places the cold §C-migrate (seconds to minutes) on "the first routed *write*" — the debounced save tail. R10 (:315): child preparation "runs at link-time or first-foreign-open with visible progress — **never on the debounced save**, never colliding with the PJ-103 5s close-flush cap; writes refuse visibly until the child is ready." A minutes-long migrate triggered by a save that the close-flush then caps at 5s is a data-loss window. Amend: prep at first-foreign-note-open (or cUniverse link time); keep the strip; saves refuse plainly until ready.

**B4. Wave order F1-before-routing creates a half-migrated seam the draft's own evidence forbids.** With F1 (move door, "Rust Wave 1: transfer engine") shipping before Wave 2 (routing layer / Class-D kill), the live `constellation_search_reindex` still trusts the frontend name (evidence write-surface Class D): the **first edit of a just-moved note misfiles fresh rows into the ACTIVE DB while the destination DB — which just received the journaled payload — goes stale; "move it back" then exports from that stale DB.** The evidence's own risk ranking says vocabulary threading + routed indexing "must be Phase 1, gated, not incremental" (OPTIONS::A §3 Risk), and the panel's preconditions put R1/R3 before ANY door. Architect §6.3's "Wave 1 = move + copy" contradicts both; the draft asks the Boss to *confirm* §6.3 instead of surfacing the contradiction. Amend: routing layer (router + pool + vocabulary threading + Class-D kill) is the precondition wave; the move door follows it. F1's entry conditions must name R1, R3, R5, R7-harness-green, R11, R35, R36 explicitly.

**B5. Architect conditions 5 and 6 appear nowhere; condition 4 is written but never exercised.** Condition 5 (link_life OS lock; `migrate.rs` `fs::copy`-of-live-WAL retired — R11 says "fixed FIRST — it runs on today's boot path") and condition 6 (watcher + suppress stamped for BOTH roots) are absent from every F-phase and every verification clause — a suppress miss after a move surfaces as adopt-echo/conflict sidecars, which F1's "the note sits in the tree" test would pass right through (green-while-broken). Condition 4's `freshIdentity` receipt is drafted (§3.2) but **no staged test ever provokes a cid collision** — the string ships untested. Amend: add both conditions to the plan with owners/waves; add a staged cid-collision test to F1; add "no conflict banner / no external-edit adopt fires after the move" as an explicit F1 failure mode.

**B6. F3's trash clause describes a surface that does not exist.** "delete a linked note → it lands in *that* universe's trash **and restores from there**" — grep finds **no in-app trash listing or restore surface anywhere** (zero frontend hits; no Rust restore-from-trash command). The Boss cannot perform "restores from there." This is the Never-Describe-The-App-Without-Looking class inside the plan itself, and R14 requires "trash listing/restore across owner roots" as buildable scope. Amend: either schedule the trash surface or rewrite the clause to what is observable (file present under `<that universe>/.trash` — via the OS file manager, stated honestly).

**B7. The Boss-decision list is incomplete.** Draft hands over (a) confirmation-drop ratification, (b) wave order, (c) PJ-224. Missing: the **full §5 repeals table item-by-item** (Architect §6.1, R36 — "Phase 2's first act"), the **two-instance policy confirmation** (Architect §6.4), the **PJ-262 sequencing question** (R35 — "goes to the Boss BEFORE the transfer engine is planned"), and conflicts **C1/C3** (C1 is arguably resolved by the 2026-08-12 ruling — say so explicitly rather than skipping it; C3 the draft silently decides in the Art-Director's favor).

## C. MAJOR amendments

**C1. `libraryColorMap` re-key is neither "small, mechanical" nor a fix.** The name-keyed map is consumed at ~100 sites in `+layout.svelte` plus ~25 components (BacklinksPanel, DashboardView, OrgChart, GlobalTasksView, ConstellationMap legend at :791, TasksPanel…), and the second screen builds its **own copy** (`SecondScreenPage.svelte:392`). Many consumers possess only a library NAME at the lookup site (`task.library_name`, `bl.libraryName`, `note.libraryName`) — "path-keyed with a name fallback" collapses to the same collision exactly at the colliding sites. R19 demands (universe, library)-keyed identity plus persisted-tab-color reconciliation. Amend: own step (not F0 housekeeping), designed against R19, or descoped to collision-detection + suffixing with Boss approval.

**C2. JobProgressStrip cannot be "a fifth consumer" as claimed.** Its label is a static `$t(labelPrefix.label)` — **no `{note}`/`{universe}` interpolation exists** (`JobProgressStrip.svelte:98-110`), and a Cancel button is mandatory for in-flight jobs (:113-120) — cancelling a journaled two-phase transfer needs defined semantics (rollback at which seam? forbidden?) the draft never states. Amend: schedule the component extension (dynamic label args, optional/no-cancel mode) and rule the transfer-cancel semantics; ties to R22.

**C3. The vocabulary concern is under-enumerated — the exact Whole-Ecosystem shape the law forbids.** The palette is `LinkTypePicker.svelte`, but the same registry feeds the **CM6 `[[` autocomplete** (`completions.ts` uses `getLinkTypes()` — per LinkTypePicker's own header) — which is **on the typing path**, so the foreign registry must be synchronously cached before completion time — plus `livePreview.ts`/`linkDisplay.ts` rendering (a child's custom type renders as plain text in the parent's editor — the "same note looks different" contradiction) and `LinkTypePill`. No Rust command to read a foreign registry exists (`link_types.rs` has only `load_active`). Amend: F2 names all vocabulary surfaces + the new IPC command + the cache store.

**C4. R16 one-pass identity vs the draft's three surfaces.** R16 (:327): identity travels "everywhere notes co-mingle — tab strip, Quick Switcher, search rows, backlinks/outgoing, list surfaces, pickers, second screen — applied whole-ecosystem in one pass." The draft covers tab + status bar + pickers, omits the rest, and **explicitly excludes the second screen** with a rationale ("the marks are a write-context aid") that contradicts its own maxim ("identity is information") and the ruling's "planet mark = identity information." Also, the predicate is module-local to `+layout.svelte` — reaching those components requires exposing the side-map, which weakens the "no new state" claim. Amend: either extend the identity pass (SS side-map is small — `ScreenNote` already carries `libraryPath`) or take a Boss ruling narrowing R16; never silently diverge.

**C5. R17/R18 mark-and-color drift.** F0 adds a **new hard-coded `#6366f1`** — R18 bans per-surface literals (tokenize `--cuniverse-accent` first, Style-Setter-exposed). And "the ONE shared mark" is aspirational: `LibrarySwitcher`'s Child-Universes rows draw their own globe SVG (:125-129) and the workspace-bases cu-group draws its own inline planet (:8270) — neither uses `LibraryIcon`. Amend: F0 converges both onto `LibraryIcon` and lands the token.

**C6. Interim honesty through F0–F2.** Context menus (main and second screen) keep offering rename/delete/tag on federated notes while those verbs corrupt **today** (delete: file gone, foreign DB rows orphaned forever, remains dragged into the ACTIVE universe's `.trash` — evidence §3.6; rename: foreign bookkeeping stale — §3.4). The draft's §1.6 admits they "stop lying the moment routing ships" — i.e., they lie until F3, while F0's quiet-identity layer actively invites engagement. The Boss's ruling keeps the walls up until each door ships. Amend: interim-seal these verbs on federated items at F0 (the predicate exists) with the plain refusal routing to the coming door, per R25.

**C7. The picker perf claim is false and unmeasured.** "pickers group from lists already loaded" — `buildUniverseFolderEntries` awaits `invoke('list_universe_folders')` on **every dialog open** (:6891-6894); un-narrowing makes that a folder walk of *every linked universe's tree*. Not a keystroke-path issue, but R33 + Rule 8's hard constraint (measure on the 7,600+ corpus before committing) apply; consider per-universe-group lazy expansion. Relatedly, **R33's measurement gates are absent from every F-phase** — add boot/typing/routed-write-latency measurement to each wave's exit criteria.

**C8. R22/R23 receipts.** No receipt carries Undo ("move it back the same way" is not Undo; the transfer engine must be designed to run in reverse — R22, and it is the ruling-consistent replacement for the dropped confirmation). The rename receipt carries one count; R23 requires both counts including durable pending-heals for unreachable universes. Amend both message specs.

**C9. F1's "links/history intact" is not Boss-observable.** Earned scalars (traversal, weight, confidence promotions, review rows) live only in the DB; the Boss cannot see "history intact" from the note. Amend: name observable proxies (confidence popover, Review Pulse record) and state the R7 harness (every enumerated crash window red→green) as F1's entry gate — the single staged mid-move kill is one window, not "resumable at every seam."

## D. MINOR amendments

1. Fix the "R7" citation → R5; add R5's NFC/NFD + case-aware ownership matching to the macOS paragraph.
2. R31: surface the one-time Boss naming decision ("child Universe" vs "Linked Universe"); the existing `federation.warningBadge` string "cUniverses unavailable" violates the jargon ban — schedule its rewording.
3. LibrarySwitcher fix wording: its Child-Universes section uses the `childUniverses` list, not a lib-path predicate, and the component has no access to `childUniverseLibPaths` — the fix needs a prop/side-map, not "the same predicate."
4. R28 (cross-edge carrying a universe-custom type the other side lacks) must be specified before Build; the draft's palette item covers only the foreign-SOURCE case.
5. R37: carry the panel's uniqueness one-liner in the Plan preamble; per-phase "which requirement this advances" annotations.
6. R24/R27 (coverage states; provenance-queryable surfaces) — name the surfaces even if Rust-wave-scoped.

## E. What survives the attack intact

The governing-ruling translation itself is right: grouped-umbrella pickers with no follow-up confirmation (ruling-adopted; formalize as C1's resolution), the quiet planet + status-bar Place Line (matches R15/R20/R21 exactly), edit-in-place as "adding nothing," the three-exceptions voice and Nothing-was-changed/lost convention, deliberate-vs-automatic asymmetry, PJ-224 kept gated, RTL/logical-properties pass, and the SS forward-to-main inheritance (verified: `ssReadOnly`, `requestNoteActionOnMain` → ungated `handleOrgNodeMenuAction`). Nearly every file:line in the draft is real — the failure mode here is not invention; it is **selective deafness to the panel that binds the Plan, and sequencing that opens doors before their preconditions.**

**Disposition: rework the draft against B1–B7 and C1–C9 before it goes to the Phase-2 sitting. The design core is keepable; the schedule and the decision list are not.**

Key paths: `docs/migrations/PJ-235-federation-boundary/MIG-111-CONCEPT-PANEL.md` (the binding requirements the draft must reconcile), `MIG-111-ARCHITECT.md` §3/§6, `MIG-111-ARCHITECT-EVIDENCE.md` (OPTIONS::A §3 Risk; MAPS::write-surface Class D/§3.6), `src/lib/components/JobProgressStrip.svelte`, `src/lib/components/LinkTypePicker.svelte`, `src/routes/+layout.svelte:6891-6942`, `src/lib/components/LibrarySwitcher.svelte:104-135`.