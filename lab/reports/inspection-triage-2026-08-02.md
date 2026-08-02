# Consolidated Safety Triage — 2026-08-02

Three inspection registers (2026-08-02 whole-app · 2026-08-01 · 2026-07-30) verified against
current code, deduplicated, and ranked. Produced by the `register-triage` workflow (6 agents,
374 tool calls, ~27 min). **Read-only — no code was changed.**

**Every finding was re-opened at its cited symbol in the CURRENT source before being ranked**,
because a great deal landed on 1–2 August after these registers were written. 17 were found
genuinely dead and are excluded.

---

## CONSOLIDATED SAFETY TRIAGE — 2026-08-02
### Three registers, verified against today's code

---

## 1. HEADLINE

**The three registers hold ~95 raw entries. They describe 31 actual problems.**

| | |
|---|---|
| Raw entries across the three registers | 95 (44 + 24 + 27), plus PJ-187's 19 overlapping M-cost items |
| **Already fixed** by the 1–2 August work | **17** — genuinely dead, verified |
| Still live | 74 |
| **…which collapse to** | **31 distinct defects** |

The 95-to-31 collapse is the single most useful fact here. The same defect was found independently by as many as **thirteen** different agents looking at different files. The four settings-file problems alone account for 13 entries. You are not looking at 95 things to fix.

**Severity of the 31:** 4 app-killers · 13 high · 12 medium · 2 low.

**How I checked.** I opened every cited file at the cited symbol and read the current source. Line numbers had drifted badly (one finding moved 250 lines, another 370) — I located each by name, never by line, so nothing was dismissed for having moved. Three examples of things I confirmed *dead*: the boot-time settings read now refuses to latch on an empty result; the rename-cascade flush now carries its re-index; the rename re-base fix landed. The 17 already-fixed are not padding.

---

## 2. FIX NOW — can silently cost you knowledge

Eight items. Nothing here shows you an error message; that is what puts them on this list.

**1. Four saved-settings files treat "couldn't read this" as "you have none."** *(app-killer, medium)*
A one-second file lock from a sync tool or antivirus makes Constellation decide you have nothing — and the next click saves that emptiness permanently. At stake: **every Universe you ever set up**, your entire review history, your custom link vocabulary and colours, all saved visual styles. The cure is already written and proven in this codebase — four sibling files got it on 1–2 August; these four were missed.

**2. "Overwrite" on a name clash can destroy the surviving note.** *(app-killer, small)*
The replaced note goes to the trash but its tab stays open still believing it owns that file. On app close it writes itself back over the note you kept. I confirmed the safety gate that would catch this is running in observe-only mode, so nothing stops it.

**3. After a failed save, restarting throws away the app's own rescue copy.** *(app-killer, small)*
On restart the recovery copy is re-labelled "already safe on disk," and the next teardown deletes it. Your work is gone — on the app's own documented recovery route.

**4. If the editor falls back to simple mode, nothing you type is ever written.** *(app-killer, small)*
Rare, but total: no autosave, no idle save, for the rest of the session. Honest scope — closing the tab or app still captures the text, so it is not guaranteed loss. But nothing reaches disk while you work.

**5. The properties writer misjudges where a block starts and ends — five note shapes get corrupted.** *(high, medium)*
A note starting with a blank line, a comment in a list, an indented sub-line, a multi-line block, a plain alias. Each makes the app write a broken properties block **into your .md file** — after which every later property edit on that note is silently discarded. A previous pass fixed half this loop and left the other half hand-rolled; the code comment says so in writing.

**6. Five surfaces paste raw text into note properties, and the shared quoter mishandles backslashes.** *(high, medium)*
Any imported cell, canvas name or title containing a colon, bracket, quote or backslash produces a corrupted note — and the import reports every row as a success. This is **one** fix (one correct writer, used by all of them), not five. The correct escaper already exists on the Rust side.

**7. Words the app doesn't recognise in Sources / Content-type are deleted from your file.** *(high, medium)*
Your own citation or hand-typed label is removed from disk the first time Constellation touches that note. "Approve-All" was built specifically never to subtract, and it subtracts — its own comment claims the opposite.

**8. If a note can't be saved at the moment you rename it, the rename silently undoes itself.** *(high, small)*
The old title returns, the alias vanishes, and every link the app just rewrote points at a title that exists nowhere. The branch immediately above this one documents this exact mechanism as an app-killer and fixes it. This branch never got the fix.

> **Recommended as one batch.** Six of the eight are small. Items 1, 5, 6 and 7 share a root — see §5.

---

## 3. FIX SOON — real, bounded, not silent data loss

| # | What breaks | Size |
|---|---|---|
| 9 | The properties panel's 0.8-second save delay can lose an edit **or paste it onto a different note** — the contamination class Constellation has been burned by repeatedly | medium |
| 10 | Fast typing during a save drops that batch and nothing retries it; the editor has already declared itself saved | small |
| 11 | Switching Universe leaves the previous one's saved layouts loaded — the next save writes them into the wrong Universe's file | small |
| 12 | Renaming a note only rewrites links **inside one library** — contradicts your standing "It is ONE universe" ruling | medium |
| 13 | Renaming a saved table (a Base) strips its file type; the table vanishes with no error | small |
| 14 | Opening one table right after another shows the first one's rows under the second one's name — and edits save into the wrong file | small |
| 15 | Renaming a folder freezes the window, and the follow-up work can be abandoned; links keep pointing at the old location | small |
| 16 | Two operations lock the whole window: a Universe switch waiting on background work, and "+ Add column" (~10s). Both essentially one-line changes | small |
| 17 | **The app's repair pass can never run**, and its error message tells you to click a button that does not exist. Several fixes elsewhere are only safe *because* they assume this works | medium |

Item 17 is the one I would move up. It is the safety net the rest of the system quietly leans on.

---

## 4. ACCEPT OR SCHEDULE

**Schedule (12 medium items).** Genuinely wrong, none loses a file: multi-line property values shown truncated; the second screen breaking its read-only promise in three places (it currently edits your Markdown — a change you never made, visible in Git); tab-switch losing crash-recovery and cursor position; a ticked task staying ticked when the file never changed; link confidence/archive failing silently on data that has **no on-disk backup**; link bookkeeping drifting after a rebuild; a rename writing the new title into the old file then reporting failure; a template half-applying; read-only fields that accept edits and discard them; a property change at tab-close never reaching search; the legacy-version upgrade deleting bookmarks after a copy step that **always** fails (I confirmed the command it calls does not exist); and a classifier timeout that does not time anything out.

Recommend as **one scheduled remediation cycle** after §2 and §3. Individually small; collectively they are what makes the app feel untrustworthy at the edges.

**Accept (2 low).** Undo history dropped in the first seconds after launch; a library register failure printed only to a console that does not exist in a release build. The second is a one-line fix — take it whenever anything else in that file is open.

**No migration-sized item in this set.** Every one of the 31 is a local fix. That is good news, and it is unusual.

---

## 5. PATTERNS — what to change about how the code is written

This is the section worth acting on. Six classes explain nearly all 31.

**① "Half a sweep" — a fix applied to one branch and not its identical twin. This is the dominant pattern by a wide margin.**
I verified eleven instances: the "refuse to overwrite" cure exists in one settings file and not four siblings · delete closes tabs, overwrite doesn't · collections resets on Universe switch, workspaces doesn't · the rename fix landed on one branch of one function and not the branch below it · note-rename runs its heavy work in the background, folder-rename runs it inline · one Base command is non-blocking, its sibling isn't · one Tasks screen reverts a failed checkbox, the other doesn't · one frontmatter helper is used by half the loop that needs it.
**This is your own Whole-Ecosystem Fix Law being violated by the very passes run to enforce it.** The change: a fix is not done when the reported site is fixed — it is done when every sibling call site has been found and brought along, in the same commit, ideally behind one shared helper so they cannot drift again. Treat "one of N identical sites changed" as an automatic inspection finding.

**② "Couldn't read it" → "you have none."**
Four stores still do this; four others were cured in the last two days. The cure is proven and short. The change: **one shared helper for every persisted settings file** — read-succeeded latch, refuse-to-overwrite, atomic write — so a new store physically cannot be added without it. Today each store hand-rolls its own and each one is a coin flip.

**③ Reporting success for work that didn't happen.**
Eight items. Two code shapes cause all of them: a function that gives up with a bare `return` when its caller cannot tell that apart from success, and a Rust write whose result is discarded. Both are one-character-cheap to write and invisible in review. The change: on any write path, ban both shapes.

**④ Write first, check second.**
Five items where the write lands and *then* the guard refuses — with nothing undoing what already happened. Overwrite trashes before it renames; the properties panel projects before it validates; the rename commits the title before the move can fail; the template applies properties before refusing the body; the upgrade deletes before verifying the copy. The change: **the check comes first, always.** Where two steps must both land, the failure of the second must undo the first.

**⑤ Frontmatter and YAML are hand-rolled at every writer.**
Correct shared helpers exist for splitting frontmatter, classifying YAML lines, and escaping values. Multiple writers ignore them and re-implement it badly. Every corruption finding in §2 comes from this. The change: one reader, one writer, everything else calls them — and the shared quoter needs a backslash fix it never got (the Rust one has it; the frontend one doesn't).

**⑥ Comments that promise a guarantee the code does not provide.**
Four found: a timeout comment says the classifier is isolated after five seconds — the code waits however long it takes. An error message tells you to press a Rebuild button that does not exist. A bulk operation's comment says it "never drops a value the user set by hand" — it drops them. The second screen's comment says "the 2nd screen never writes" — it writes to your Markdown.
**This class is the most dangerous of the six**, because a confident comment stops the next reader from checking. It is how a defect hides for weeks. The change: when a comment asserts a safety property, the inspection must verify the property, not read the comment.

---

## 6. WHAT I COULD NOT DETERMINE

Stated plainly, because a triage you can't trust is worse than none.

1. **I did not re-verify all 17 "already fixed."** I spot-checked three at random; all three were genuinely dead, and one carried an explicit comment reasoning about exactly the failure the register described. That is good evidence the dedup pass was honest — it is not proof for all 17.

2. **The 2026-07-30 run lost 25 candidates to server errors.** The register says so itself: *"those candidates died unverified — an under-count, not an all-clear."* Those 25 have never been triaged by anyone. They are not in the 31, and I have no idea what is in them.

3. **One finding fell out of the consolidated set and I could not rank it.** A notification the source-review path sends may be filtered out when it arrives within two seconds of an autosave. I confirmed the sending code is unchanged; I did not trace the receiving filter, so I cannot say whether it is live. It belongs in the next pass.

4. **I can tell you what happens, not how often.** "A sync tool locks a file for one second" is an everyday event — but I have no measurement of how often it actually hits these four files on your machine. Ranking here is by *what you lose*, not by *how likely*.

5. **Two claims I verified structurally but not end to end.** For the settings app-killer I walked two of roughly twelve write-back sites (Universe creation, mark-as-reviewed) and confirmed both do read-then-write-back; I did not walk the other ten. For the template half-apply I confirmed properties are committed before the body step that can refuse, but did not read the refusal itself.

6. **I did not run the app.** Everything above is read from source. Per your own Reproduce-First rule, none of these is proven until it fires under instrumentation — that is the first step of any fix, not of this triage.

---

### Recommended decision

Approve §2 as **one batch of eight** — six are small, and items 1/5/6/7 share a root, so fixing the root fixes four findings at once. Then §3 as a second batch, with item 17 (the unreachable repair pass) moved to the front of it, since other fixes assume it works.

The 31 are all local. The six patterns in §5 are not — and pattern ① alone explains a third of the register.

---

## Appendix — the 31 deduped concerns, machine-readable

### 1. [APP-KILLER] Four of your saved-settings files still treat "I couldn't read this" as "you have none" — and the next thing the app does is save that emptiness over the real file

- **Sites:** src-tauri/src/universe.rs:113-124 (write-backs at 701/706, 976/981, 1055/1061, 1250/1253); src-tauri/src/review.rs:826-833 (write-backs at 720/733, 755/760, 782/788); src-tauri/src/link_types.rs:514-518 and 537-540; src/lib/libraries/linkTypeRegistry.ts:155-159; src-tauri/src/style_presets.rs:36 and 51; src/lib/libraries/stylePresets.ts:123-130; src/lib/components/StyleSetter.svelte:980-985. The cure already exists in the same codebase: src-tauri/src/libraries.rs:78-88 ("Refusing to overwrite it.") and src-tauri/src/universe.rs:153 (atomic_write).
- **Why it matters:** A file being locked for one second by a sync tool, antivirus or a backup job is an everyday event. When it happens to these four files the app decides you have nothing — and the very next click makes it permanent. Worst case (the Universe list): Constellation forgets every Universe you ever set up. Also at stake: your entire review history, your custom link vocabulary and colours, and every saved visual style. Nothing is shown to you at any point. Four sibling files got exactly this protection on 1-2 August; these four were missed.
- **Class:** Failed-read-presents-as-empty / Ok-on-None false success — the class the 2026-08-01 pass closed for settings, workspaces, collections and property-types. Leaving four stores behind is the Whole-Ecosystem Fix Law violation named in the law itself.
- **Fix size:** medium · user-visible: True · register entries merged: 13

### 2. [APP-KILLER] Choosing "Overwrite" when a note name already exists can destroy the surviving note

- **Sites:** src/routes/+layout.svelte:6970-6974 (rename) and :4516 (create); src/lib/libraries/store.ts:4652-4655 (moveToTrash — no tab/model cleanup) vs :4623-4638 (deleteWithSetting — does both); store.ts:3200-3206 (app-close flush); src-tauri/src/write_gate.rs:41 (WRITE_GATE_ENFORCE = false, so the mismatch is recorded but the write still lands).
- **Why it matters:** Two ways to lose a note in one dialog. (a) The note you replaced goes to the trash but its tab stays open and still believes it owns that spot on disk — so when you close the app it quietly writes itself back over the note you just renamed. (b) If the second half fails after the first note is trashed, you end up with neither note and no message. Verified live: the write-gate that would catch (a) is in observe-only mode, so nothing stops it.
- **Class:** Content-integrity / single content ownership (the LL-014 three-strike class) — a resident in-memory note outliving the file it claims.
- **Fix size:** small · user-visible: True · register entries merged: 2

### 3. [APP-KILLER] After a failed save, restarting the app throws away its own rescue copy of your unsaved writing

- **Sites:** src/lib/libraries/store.ts:3444-3453 (restoreSessionTabs seeds the recovered tab as "clean"); markModelRecoveredFromNet is called only at store.ts:1905, 3129, 3149 — never here; the chain that then discards it: NoteEditor.svelte:388, store.ts:333-336, store.ts:2847 and 2868-2870.
- **Why it matters:** The rescue copy is the last line of defence when a save fails. On restart the app re-labels it "already safe on disk", and the next teardown deletes it. The work is gone with no warning — on the app's own documented recovery route.
- **Class:** LL-040 verifier-blind-spot — the staleness check is handed a flag that certifies the very thing it is supposed to test.
- **Fix size:** small · user-visible: True · register entries merged: 1

### 4. [APP-KILLER] If the editor fails to start and falls back to simple mode, nothing you type is ever written to the file

- **Sites:** src/lib/components/NotePane.svelte:723-743 (the fallback editor); the only change-listener in the file is at :633, inside the normal editor; the save path it feeds is at :338-345 and the idle timer at :962.
- **Why it matters:** Rare but total: it needs the editor to fail to start (which is why the fallback exists at all). When it happens, the note looks and feels normal, but no autosave, no idle save and no background save ever fires for the rest of the session. A crash or power cut loses everything typed. Honest scope: closing the tab or the app normally does still capture the text, so this is not guaranteed loss — but nothing reaches disk while you work.
- **Class:** Ok-on-None false success — a degraded mode that presents as fully working.
- **Fix size:** small · user-visible: True · register entries merged: 1

### 5. [HIGH] The properties-block editors misjudge where a block starts and ends — five different note shapes come out corrupted

- **Sites:** src-tauri/src/sources/mod.rs:452, :491, :497-509, :1077, :1104-1120 (the same raw test also at :231 and :955); src-tauri/src/libraries.rs:1975-1998 (with the append at :1956-1961). The shared, correct helpers already exist: src-tauri/src/search.rs:2885-2889 (split_frontmatter) and src-tauri/src/yaml_lines.rs:96-124 (is_comment / is_block_value_line).
- **Why it matters:** Five ordinary note shapes — a note starting with a blank line, a comment inside a list, an indented sub-line, a multi-line block of text, a single plain alias — each make the app write a broken properties block into the .md. The repo's own notes on this class say the consequence is that every later property edit on that note is silently thrown away. It corrupts the file itself, which is the source of truth. Notably, a previous pass (PJ-182) fixed half of this exact loop and left the other half hand-rolled.
- **Class:** Indentation-is-data, plus the "half a sweep" failure the Whole-Ecosystem Fix Law is named for — the shared helpers exist and these writers don't use them.
- **Fix size:** medium · user-visible: True · register entries merged: 5

### 6. [HIGH] No single place writes note properties — five surfaces paste raw text straight in, and the one shared quoter mishandles backslashes

- **Sites:** src-tauri/src/importers.rs:469-479 (CSV) and :383 (Evernote tags); src/lib/components/ExpressionForge.svelte:139; src/lib/components/SenseMakingCanvas.svelte:306; src/lib/libraries/store.ts:2493-2509 and 2567-2572; src-tauri/src/libraries.rs:1080, 1895-1896, 1987, 2038. The correct escaper already exists at src-tauri/src/canonical.rs:411-413.
- **Why it matters:** Any imported cell, canvas name or composition title containing a colon, bracket, hash, quote or backslash produces a note whose properties are broken — and the import still reports every row as a success. Once broken, the app can't find the note by its title and every later property edit on it is discarded. This is one fix (one correct writer, used by all of them), not five.
- **Class:** Same corruption outcome as the block-boundary concern above; also Ok-on-None false success (the importer counts a corrupted row as imported).
- **Fix size:** medium · user-visible: True · register entries merged: 6

### 7. [HIGH] Words the app doesn't recognise in a note's Sources / Content-type list are deleted from the file

- **Sites:** src-tauri/src/sources/mod.rs:278-292 (push_source) and :995-1012 (push_content_type) — both end in "Silent drop of unknown values"; the rewrite that then re-emits only the approved list at :518-524; src-tauri/src/sources/bulk_ops.rs:451-458 (Approve-All builds its "keep what's there" union through the same dropping reader).
- **Why it matters:** If you (or a vault you imported) put your own wording in those fields — a citation, a hand-typed label — the first time Constellation touches that note it removes your wording from the file on disk. Not moved, not flagged. Approve-All was specifically built to never subtract, and it subtracts.
- **Class:** Silent data loss on a "normalise" path — the taxonomy is treated as more authoritative than the user's own file.
- **Fix size:** medium · user-visible: True · register entries merged: 2

### 8. [HIGH] The properties panel's short delay before saving loses your edit — or writes it onto a different note

- **Sites:** src/lib/components/PropertyEditor.svelte:1024-1040 (writes the app's copy of the note BEFORE the safety check at :962-968 can refuse), :587-613 (a pending edit is dropped during a reload or rename); src/lib/libraries/store.ts:4441-4444 (the rename re-base puts your unsaved body back but not your unsaved properties).
- **Why it matters:** Three ways one property edit disappears or lands wrong, all inside a 0.8-second window you can't see. Click a link too quickly and the first note's properties get glued onto the second note in the app's working copy — which is what feeds the second screen, the raw-source view and the crash-recovery buffer. Edit during a rename and the value is simply gone. This is the defect class Constellation has already been burned by repeatedly.
- **Class:** Content-integrity / single content ownership (LL-014 three-strike class) — the same shape as the earlier cross-note contamination bugs.
- **Fix size:** medium · user-visible: True · register entries merged: 5

### 9. [HIGH] If you keep typing while a save is finishing, that batch of edits is dropped and nothing retries it

- **Sites:** src/lib/components/NoteEditor.svelte:279-284 (refuses while a save is in flight) and :318 (the flag clears only when the write finishes); src/lib/components/NotePane.svelte:338-345 (marks itself clean BEFORE handing over) and :962 (the idle retry goes through the same already-clean check).
- **Why it matters:** Ordinary fast typing triggers this: the editor has already declared itself saved for a save that was refused. Nothing re-arms it, and because the save never ran, the crash-recovery copy isn't written either. The text usually survives until you type again or close the note — but you are relying on that, not on the save.
- **Class:** Ok-on-None false success — the clean flag is set for work that never happened.
- **Fix size:** small · user-visible: True · register entries merged: 3

### 10. [HIGH] If a note can't be saved at the moment you rename it, the rename silently undoes itself minutes later

- **Sites:** src/lib/libraries/store.ts:4445-4450 — the branch taken when the pre-rename save fails. The sibling branch immediately above (:4426-4444) documents this exact mechanism as an APP-KILLER and fixes it; this branch never got the fix.
- **Why it matters:** The old title comes back, the alias the rename added disappears, and every link the app just rewrote across your library now points at a title that exists nowhere. Nothing surfaces it — the app's own bookkeeping then treats the reverted state as correct.
- **Class:** A known fixed defect re-appearing on the untreated branch of the same function — the "half a sweep" pattern again.
- **Fix size:** small · user-visible: True · register entries merged: 1

### 11. [HIGH] Switching Universe leaves the previous Universe's saved layouts loaded — and the next save writes them into the new Universe's file

- **Sites:** src/lib/libraries/store.ts:6985-6999 — no reset of the list or the "read succeeded" flag before reading the new Universe. The identical bug was fixed for collections on 2026-07-29 with a two-line cure at store.ts:2082-2090 ("the latch must mean THIS universe's file was read, never some universe's file was once read").
- **Why it matters:** Open a second Universe and its Workspaces panel shows the first Universe's — so you save a layout believing you're in one place and overwrite the other. The write path was hardened on 1 August; the Universe-switch half of the same file was not.
- **Class:** Cross-universe stale state — the exact collections bug, on the sibling store.
- **Fix size:** small · user-visible: True · register entries merged: 1

### 12. [HIGH] Renaming a note only rewrites the links inside ONE library

- **Sites:** src/routes/+layout.svelte:7098 — the cascade is handed a single library, resolved by longest-path match at :6577.
- **Why it matters:** Every link to that note from any other library in the Universe keeps pointing at the old name, while the rename reports complete success. This directly contradicts the standing "It is ONE universe" ruling (2026-07-05) that every name resolver must span all libraries and federated Universes. The 1 August work made per-file failures visible but left the scope wrong.
- **Class:** The first-match / single-scope resolver class — a resolver that stops at the first library instead of spanning the Universe.
- **Fix size:** medium · user-visible: True · register entries merged: 1

### 13. [HIGH] Renaming a saved table (a Base) strips its file type and the table vanishes from the app

- **Sites:** src/routes/+layout.svelte:6950-6952 (anything not ending .md is treated as a folder); reachable from the tree at :6312 and the workspace Bases list at :7862; src/lib/components/FileTree.svelte:79 and :84 show Bases as renameable; src-tauri/src/libraries.rs:1379 then renames it extensionless.
- **Why it matters:** One rename and the table is gone from Constellation with no error. The file still exists on disk under a name the app no longer recognises, so the user has no way to tell what happened or get it back.
- **Class:** "Not .md means folder" — a two-kind assumption in a three-kind world.
- **Fix size:** small · user-visible: True · register entries merged: 2

### 14. [HIGH] Opening one saved table right after another leaves the first one's setup on screen — and edits then write into the wrong file

- **Sites:** src/lib/lens/BaseTab.svelte:65-99 — the results, error and loading state are never reset on a path change and the disk read has no "is this still the note I asked for?" re-check; src/routes/+layout.svelte:8851 and :8813 mount it with no remount key (compare the FocusPane sibling three lines below at :8858, which is keyed).
- **Why it matters:** You see one table's rows under another table's name, and changing a column then saves that layout into the new file. Same shape as the note-contamination bugs, on the Bases surface.
- **Class:** Stale-async-response / content-integrity — no request-identity re-check when the answer comes back.
- **Fix size:** small · user-visible: True · register entries merged: 1

### 15. [HIGH] Renaming a folder freezes the app, and the rename's follow-up work can be abandoned

- **Sites:** src-tauri/src/libraries.rs:1405-1431 — the whole descendant cascade plus a per-note re-index runs inside the awaited command, holding the search database's writer lock across the loop. The note-rename branch in the same function detaches exactly this work (:1518-1520), and a ready-made detached version exists at :2319.
- **Why it matters:** Rename a folder holding hundreds of notes and the window stops responding. Worse than the wait: links and search keep pointing at the old location afterwards, with no error. The fix already exists two branches away in the same file.
- **Class:** The PJ-066 rule (multi-second, lock-holding work must run detached) — and again a fix applied to one branch and not its sibling.
- **Fix size:** small · user-visible: True · register entries merged: 2

### 16. [HIGH] Two operations lock the entire window with no explanation

- **Sites:** src-tauri/src/universe.rs:902-918 — the lock every path lookup needs is held while waiting on the search database (src-tauri/src/search.rs:9776); src-tauri/src/lens/query.rs:328 — the "+ Add column" scan runs on the main dispatch thread (its sibling at :77 does not).
- **Why it matters:** Switching Universe while anything is running in the background can hang the whole app until that background job finishes — not just the switch. Clicking "+ Add column" in a Base can freeze it for around ten seconds, stalling any save queued behind it. Both are essentially one-line changes.
- **Class:** The PJ-066 rule again — commands that touch locks must not run on the IPC thread.
- **Fix size:** small · user-visible: True · register entries merged: 2

### 17. [HIGH] The app's own repair pass can never be run, and its error message tells the user to click a button that doesn't exist

- **Sites:** src/routes/+layout.svelte:2889-2895 — the only boot route to the repair pass, gated on a completely empty index; the other three callers are new-library only (:4690, :5971, :5984). The user-facing text at :558 (and src/lib/i18n/en.json:4245) says "Settings → Rebuild Index will restore it" — src/lib/components/SettingsModal.svelte has only an orphan style rule at :2868 and no such control. Five maintenance sites in src-tauri/src/search.rs (:1745, :1814, :9497, :10917) defer to this pass as "the authoritative self-heal".
- **Why it matters:** Once a backlink count, a tag count or a Sky level goes wrong — or notes deleted while the app was closed linger in search — there is no route back to correct, and the app tells the user to press something that isn't there. Several fixes elsewhere in the code are only safe *because* they assume this pass exists.
- **Class:** Behaviour-vs-comment drift (LL-023 shape): code and user-facing text both promise a self-heal that is not wired to anything.
- **Fix size:** medium · user-visible: True · register entries merged: 3

### 18. [MED] A property whose value runs across two lines is shown truncated, and editing that row deletes the rest from the file

- **Sites:** src/lib/libraries/store.ts:2295 (a continuation line matches no branch and is dropped) and :2456-2463 (a list that closes on the next line fails the closing-bracket test and projects as raw text); the read-only protection at src/lib/libraries/yamlDoc.ts:198-199 covers neither shape, so the row stays editable.
- **Why it matters:** The panel shows half the value with nothing marking it as incomplete, and saving writes the half back. The same function's own notes call an editable row that doesn't hold the real value "a data-destroying invitation" — these are two shapes that still are one.
- **Class:** Indentation-is-data — same family as the block-boundary concern, on the reading side.
- **Fix size:** small · user-visible: True · register entries merged: 2

### 19. [MED] The second screen breaks its own read-only promise in three ways

- **Sites:** src/lib/libraries/store.ts:2989-3012 — writes a hidden identity line to the .md with no display-only check, 21 lines below the guard that has one (:2970); src/lib/components/SecondScreenPage.svelte:132-136 — subscribes to note changes but not to rename/move/delete; src/routes/+layout.svelte:803 — the main window's belief about whether the second screen is open is never re-synced after a reload (11 gates read it).
- **Why it matters:** The second screen is defined as a display that never writes. Today, opening a note there edits your Markdown file — showing up as a change you never made in Git or Syncthing. It also keeps tabs pointing at deleted or renamed files and records those dead paths into the saved session, so they return next launch; and after a main-window reload it silently stops receiving updates.
- **Class:** The PJ-108 display-only-window rule — the same contract, violated at three new points.
- **Fix size:** small · user-visible: True · register entries merged: 4

### 20. [MED] Switching tabs right after typing leaves that text with no crash-recovery copy, and forgets where you were in the note

- **Sites:** src/lib/components/NoteEditor.svelte:335 — the identity check sits above the code that stores the recovery copy and the cursor/scroll position; src/lib/libraries/store.ts:3535 — switchTab performs no flush at all.
- **Why it matters:** Within about a second and a half of your last keystroke, switching tabs means that text exists only in memory. A crash then loses it, and returning to the note puts you back at the top instead of where you were working.
- **Class:** The same save/flush-on-departure gap that PJ-103 closed for app close and universe switch — the tab-switch departure was left out.
- **Fix size:** small · user-visible: True · register entries merged: 1

### 21. [MED] Ticking a task can leave the box ticked on screen while the file was never changed

- **Sites:** src/lib/libraries/store.ts:1294 (returns as if successful when the note couldn't be flushed) and the same shape at :5494; src/routes/+layout.svelte:9460-9473 with src/lib/components/TasksPanel.svelte:137-142 (the checkbox is never put back). The full-screen Tasks view was already fixed for exactly this at src/lib/components/GlobalTasksView.svelte:146-159.
- **Why it matters:** You believe a task is done; the note says otherwise. The panel even clears its own error line, actively asserting success. The fix was written for the other Tasks surface and never applied here.
- **Class:** Ok-on-None false success, plus a Whole-Ecosystem miss — one of two sibling surfaces was fixed.
- **Fix size:** small · user-visible: True · register entries merged: 2

### 22. [MED] Link judgements — confidence, archive, connect — can fail completely while the interface reports success

- **Sites:** src/lib/components/ConfidencePicker.svelte:54-71 — the menu closes before the write is attempted and any failure is discarded; src/lib/libraries/store.ts:1416 — connecting a note returns as success when a rename is in progress, and src/lib/components/RelatedCandidates.svelte:146-152 then removes the suggestion from the list.
- **Why it matters:** Per the project's own architecture notes, a link's confidence and archived state live ONLY in the search database — there is no copy in your files. A failed write here is simply lost, and the suggestion you thought you connected is gone from the list with no link created and no way to retry.
- **Class:** Swallowed write errors + Ok-on-None false success, on data that has no on-disk backup.
- **Fix size:** small · user-visible: True · register entries merged: 3

### 23. [MED] Link bookkeeping silently disagrees with reality in two places

- **Sites:** src-tauri/src/link_life_restore.rs:398 — restores link states without recomputing what they feed, unlike its twins at src-tauri/src/search.rs:9498 and :9635 which both do; src-tauri/src/search.rs:7050 — a link's target is only ever resolved while indexing the note that contains it, and the one-shot back-fill at :3556 is keyed on a column nothing populates.
- **Why it matters:** After a database rebuild, links you retired come back counted in backlink totals and Sky View levels while the restore reports success. And a link written before its target note existed is never fully registered — so the Reviewer never tells you that note needs revisiting when what it depends on changes.
- **Class:** Derived-data drift — the write-time-derivation rule applied to one writer and not its siblings.
- **Fix size:** medium · user-visible: True · register entries merged: 2

### 24. [MED] A rename can write the new title into the OLD file and then report that the rename failed

- **Sites:** src-tauri/src/write_gate.rs:716-717 (the title and aliases are committed to the old file) then :738-744 (the move is attempted five times and can still fail, with nothing undone).
- **Why it matters:** You are told the rename failed, but the note's stored title has already changed — so later the app starts calling that note by the new name anyway. The file's name and its contents disagree, and neither you nor the app is told.
- **Class:** Partial-commit with no rollback — a two-step write where only the first step is guaranteed.
- **Fix size:** small · user-visible: True · register entries merged: 1

### 25. [MED] Applying a template in split view can add its properties to a note while reporting that nothing was written

- **Sites:** src/routes/+layout.svelte:5277 — the template's properties are committed first; the refusal ("no live editor for the target note; nothing written") comes afterwards, with no undo of what was already applied.
- **Why it matters:** Properties you never accepted appear on the note, and the message on screen says the opposite. Half a template is worse than none, because you don't know to check.
- **Class:** Partial-commit with no rollback; also a false report to the user.
- **Fix size:** small · user-visible: True · register entries merged: 1

### 26. [MED] Structured property blocks that are meant to be read-only render as ordinary editable fields, and edits there are silently refused

- **Sites:** src/lib/components/PropertyEditor.svelte:453-469 (a registered type overrides the parsed one), with the editable branch at :1143 taking precedence over the read-only branch at :1298 and the delete button guarded only at :1388; the refusal happens deep in src/lib/libraries/yamlDoc.ts:348-354 and is never reported.
- **Why it matters:** The file is safe — but the screen tells the user their change (or deletion) worked when it did not. They will discover it later, if ever.
- **Class:** Ok-on-None false success at the UI layer — the refusal is correct, its silence is not.
- **Fix size:** small · user-visible: True · register entries merged: 1

### 27. [MED] A property change saved as you close the tab is written to the file but never reaches search

- **Sites:** src/lib/libraries/store.ts:1707 — the re-index only runs if the tab still exists when the write finishes; the properties panel's final save fires exactly at unmount, i.e. when the tab is closing.
- **Why it matters:** The app's own writes are deliberately hidden from the file watcher, and startup does not re-walk an indexed library — so nothing catches up. The note keeps showing its old details in search results and panels indefinitely.
- **Class:** Index-vs-disk divergence with no self-heal — made permanent by the unreachable repair pass listed above.
- **Fix size:** small · user-visible: True · register entries merged: 1

### 28. [MED] The one-time upgrade from a pre-Universe version deletes the old data after a copy step that always fails

- **Sites:** src/lib/components/UniverseSetup.svelte:226-273 — four copy steps whose errors are all discarded, then an unconditional delete. One of the four calls `save_universe_bookmarks`, which I confirmed does not exist in the app's Rust side (only the read half is registered, src-tauri/src/lib.rs:638) — it was retired by MIG-092 §9. A second dead caller sits at src/lib/universe/store.ts:110.
- **Why it matters:** Anyone upgrading from an older Constellation loses their bookmarks outright: the copy fails 100% of the time and the original is deleted anyway. Only affects the legacy upgrade path, but for that user it is unrecoverable.
- **Class:** Swallowed write errors followed by an unconditional delete — the same delete-before-verify shape as the Overwrite defect above.
- **Fix size:** small · user-visible: True · register entries merged: 1

### 29. [MED] The classifier's safety timeout does not actually time anything out

- **Sites:** src-tauri/src/cece/orchestrator.rs:153-181 — the scoped thread is joined before the function can return, so the timeout at :172 only chooses which verdict to report; the call still waits for the classifier to finish. The comment above it states the opposite. Budgets are 0.5-5 seconds; the feature is on by default.
- **Why it matters:** The guard that is supposed to stop one slow classifier from holding up the app doesn't. If a classifier stalls, the app waits however long it takes, and the log will say it was isolated after five seconds.
- **Class:** LL-040 verifier-blind-spot — a guard that reports a verdict it cannot actually enforce.
- **Fix size:** medium · user-visible: True · register entries merged: 1

### 30. [LOW] Changing a note's shape in the first seconds after launch records no undo entry

- **Sites:** src-tauri/src/shape.rs:239 — four silent early exits before the history write, and both writes discard their result; the caller at :204 has already changed the file.
- **Why it matters:** The file changes but the undo record — which exists nowhere else — is thrown away. Pressing Revert later just says there is nothing to undo.
- **Class:** Swallowed write errors — the operation half-succeeds and reports success.
- **Fix size:** small · user-visible: True · register entries merged: 2

### 31. [LOW] A brief file lock during a folder rename can make a whole library disappear at next launch

- **Sites:** src-tauri/src/libraries.rs:391 — the failure to update the library register is printed to a console and nothing else; the function returns nothing, so all three callers report success. (The read half was hardened on 2026-08-01 but is likewise console-only, and console output goes nowhere in a release build on Windows.)
- **Why it matters:** The library stays registered at a path that no longer exists, so it silently vanishes from the sidebar next time the app starts, with no explanation and no obvious way to get it back.
- **Class:** Swallowed write errors — a failure reported only to a console the user never sees.
- **Fix size:** one-line · user-visible: True · register entries merged: 1

