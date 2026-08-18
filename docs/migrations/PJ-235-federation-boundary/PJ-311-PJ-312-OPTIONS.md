# Options Paper — Search Hub: the invisible stall (PJ-311) and the silent failure (PJ-312)

Prepared for Eisa. Every number below was read from source or queried from your live universe database this session. Where evidence is thin I say so.

---

## 1. PJ-311 — what actually happens today

**The feature.** Under each Search Hub result there is a faint italic line — the *headline* — that tells you what that note is **about**, in one sentence, so you don't have to open it to find out.

**How it is produced.** There is no step in Constellation that writes a note's headline when you save the note. Instead, the headline is produced **at the moment you look at it**. Search Hub hands the app a list of note paths and says "give me headlines for these"; the app walks that list **one note at a time**, and for any note it doesn't already have a stored headline for, it reads the file and, in some cases, runs the small language model to rank the note's sentences.

**How long that list is.** Your Search Hub asks the index for six kinds of match — titles, contents, tags, properties, wikilinks, semantic — and each of those six is allowed up to 200 rows (`search.rs:13540-13546`). So one query can return up to **1,200 rows**. Search Hub then asks for a headline for **every single one** of them — no limit, no "just the ones on screen" (`SearchHub.svelte:62-86`, read this session; the comment in that code claims it uses "currently-visible result paths", which is not what the code does). The result list is not windowed either: every row is built into the page (`SearchHub.svelte:646`), while roughly **11–16 rows** fit on screen at once (arithmetic from the CSS at the default 14px text size — *UNVERIFIED: I did not measure the real panel height on your screen*).

**Here is the part that surprised me, and it is the part that matters most for you.** A note that *already has* a stored headline is **not free**. For every path in that list of up to 1,200, the app still:

- takes the database's **writing lane** — twice per note (`nsc/mod.rs:605` and `nsc/mod.rs:611`);
- pulls that note's **entire body text** out of the database (`nsc/mod.rs:472-487`);
- computes a fingerprint over **every byte** of that body to check freshness (`nsc/mod.rs:559-563`, and it is not truncated).

Your notes average **33,993 characters** and the largest is **366,909** (queried from your live index this session, 8,031 notes). So a broad search drags on the order of **44 MB of note text** through the database — measured in the mapping pass at **~2.5 seconds** on a cold disk, ~0.08 s warm (*that timing was measured with Python against your live database, not with the app's own database code — treat it as the right order of magnitude, not a benchmark*).

**Why that is a stall and not just waste.** The writing lane is the **same lane your next search needs** (`search.rs:13455`) and the same lane a note-save needs. So the app is not "busy in the background" — it is standing in the doorway your next query has to walk through. It cannot be cancelled, it reports no progress, and nothing on screen tells you it is happening.

**When it bites and when it does not — plainly:**

| Situation | Does it bite? |
|---|---|
| Your normal narrow search (3–4 hits) | **No.** Four notes, four quick reads. Nothing to fix here. |
| A broad/common-word search on your 8,031-note universe | **Yes, today** — the ~44 MB / two-lock-per-note pass, on a fully warm cache. |
| Typing a broad query letter by letter | **Yes, and it stacks.** The search itself is properly debounced and cancels stale results (`SearchHub.svelte:163, 206`), but the headline pass has **no such guard** and nothing cancels it — two of them can run at once. |
| Collapsing a category to reduce noise | **Yes, perversely** — collapsing re-derives the list and re-fires the whole pass (`SearchHub.svelte:335`). |
| A brand-new universe, a fresh import, or the day someone bumps the summary algorithm version | **Worst case.** That version bump invalidates all 7,814 of your stored summaries at once, by design (`nsc/mod.rs:555`), and every broad search becomes a from-scratch compute run. |

**A correction to the brief you were given.** It says a cold cache is "the default state." On *your* machine it is not — I queried your live index: **7,814 of 8,031 notes already carry a current headline; none are blank; 217 have no summary row at all.** You have evidently run the manual build. And the model-inference horror story is also smaller on your data than it sounds: of your stored summaries, **7,553 came from a `> [!summary]` callout already written in the note** (the Wikipedia importer's work), 259 needed the model, 2 came from frontmatter. So roughly 97% of misses stop at a file read and never touch the model at all.

**So the honest headline for PJ-311 is:** *today, on your data, this is a database-lane and file-reading stall, not a model stall. The model stall is real in the code and would arrive in full on a fresh import or an algorithm-version bump.*

**One more finding, unprompted:** 443 of your notes have a file timestamp newer than their stored summary. Today that self-corrects invisibly, because the read path recomputes what it finds stale — which is exactly the behaviour that costs the 44 MB. Any fix that stops recomputing must decide what happens to those 443.

---

## 2. PJ-311 — the options

### Comparison

| Option | What you'd notice | Effort | Risk | What it gives up |
|---|---|---|---|---|
| **A — Read only what's stored; never compute during a search** | Broad searches get instantly cheap. But every note **you** write from now on shows **no headline** until you press a button in Cataloger you have no reason to know exists. | Small for the read part (1 day); the part that makes it honest — deriving headlines when a note is saved — is a full migration (weeks) | **Medium–High** | The app's ability to fill in a missing headline on its own. Turns a stall into a blank. |
| **B — Ask for fewer: cap the request, fill in small chunks, remember what came back empty** | Broad searches stop stalling. Rows past the cap have no headline — and look identical to a note that genuinely has none. Your narrow searches are unchanged. | ~½–1 day (cap + chunk) / 2–4 days if the list is also windowed | **Low** (cap+chunk) / **Medium-High** (windowing) | Headline completeness on long result lists. Doesn't make the underlying work cheap — only rarer. |
| **C — Give the pass a progress bar and a Cancel button, like the manual build already has** | A strip in the status bar saying "Building note summaries… 340 / 1,200", with Cancel. Cold headlines arrive **much later** — the politeness is paid in ~36 seconds of deliberate pausing on a 1,200-note run. | Multi-session, full migration | **Medium–High** | Speed, for visibility. And it entrenches compute-on-read rather than fixing it. |
| **B+ — B, plus move the reads out of the database's writing lane** | Everything B gives, **plus** the stall stops blocking your next search and your saves | ~1–1.5 days | **Low** | Nothing meaningful. This is the combination I'd recommend. |

### Option A — read only what's stored

**For.** It is the only option that attacks the *shape* of the problem rather than its size. CLAUDE.md Rule 8 (Write-Time Derivation) says derived views should be built when a note is written and merely *looked up* when read; summaries are the one subsystem that never got that treatment — the only place in the whole Rust codebase that writes a stored summary is the read path itself (`nsc/mod.rs:655`). Doing the read as a single lookup query is genuinely cheap: measured at 0.007 s for 1,200 paths.

**Against — and this is the finding that changed my ranking.** Option A only works if something *else* produces the headlines. The proposal is to derive them when a note is indexed, using the two cheap non-model routes (a `> [!summary]` callout in the note, or a frontmatter summary field), and the claim is that covers ~97%. I checked that claim **against the libraries you actually write in**, and it collapses:

| Where | Notes | Covered by the cheap write-time route |
|---|---|---|
| Physics / Biology / Literature / History / Philosophy / Music / CS / Architecture / Earth Sciences / Film / Linguistics / تاريخ عربي وإسلامي / جغرافيا / علوم عربية / أدب وتراث | ~6,900 | ~100% (they are importer output, every one has a callout) |
| **التصوير** | 185 | **0** — zero callouts, zero frontmatter summaries |
| **Eisa Test** | 83 | 1 |
| **Notes at your universe root** | 44 | 1 |
| **3mooR** | 17 | 1 |

(Queried this session from your live index, grouped by top-level library.)

The 97% figure is a property of the **imported reference corpus**, not of your writing. Under Option A as proposed, essentially **every note you author from now on would have no Search Hub headline**, silently, until you manually ran a build. And a missing headline is invisible — all three render points are "show it if it exists" with no placeholder (`SearchHub.svelte:576, 623, 667`), so you could not tell "not built yet" from "this note has nothing to say."

**Attacks it survived:** the concurrent-save case (a read-only lane genuinely does stop the fighting); the mechanics of the new query (the columns and index are right).
**Attacks it failed:** cold-cache outcome (fast, silent, permanent blank); the coverage claim above; and a serious one — if the "remember what came back empty" bookkeeping is put in the **shared** summary store rather than in Search Hub itself, it poisons **nine other surfaces**, including the headline under the open note's own title (`NoteEditor.svelte:176`). The prior art it cites as precedent (`SourceReviewPanel.svelte:238-240`) keeps that bookkeeping **local to the component** — I read it, and the attack is right.

### Option B — ask for fewer

**For.** Two sibling panels in your own app already do exactly this and already settled the design question. Backlinks and Outgoing both cap the headline request at 120 rows and say in their own comment that "rows past the cap render without a headline (a soft enhancement)" (`BacklinksPanel.svelte:139`, `OutgoingLinksPanel.svelte:93`). Both also honour the **Settings → Panels → Summaries → "Note summaries"** switch, which is **off by default** (`store.ts:7111`). **Search Hub is the only headline consumer in the app that honours neither.**

And this exact regression has already been caught once — by you. The Source Review panel's code says so in plain words: *"the first cut eagerly computed summaries for all ~80 visible notes at once… spiking embedding work (the regression Eisa caught). Now we fill in small chunks, debounced, and PAUSE while a classifier scan runs"* (`SourceReviewPanel.svelte:221-225`). Search Hub asks for up to **1,200** — fifteen times the number that already burned you — with none of those guards. So this is not a new design decision; it is applying a decision already made.

**Against.** A cap bounds the *count*, not the *cost*: forty large notes cost far more than forty small ones. And the sharpest objection is one of your own principles — **Form-Aligns-To-Purpose**. Your categories are ordered titles → contents → tags → properties → wikilinks → semantic (`SearchHub.svelte:90`), and a common word fills the titles bucket to its full 200. So a simple "first N" cap would spend its entire budget on **title** matches, where the note's name already answers "what is this about?", and give **zero** to contents and semantic matches, where the name does *not*. The fix is to spread the budget across categories rather than take the head — which is more than "fifteen lines," but still small.

**Attacks it survived:** keystroke responsiveness (no invoke on the typing path either way); the concurrent-save case is unchanged, not worsened.
**Attacks it failed:** a plain head-cap starves the categories that need headlines most (above); a cap alone does not bound a *cold* run, because one un-cached note can be a whole-file read or a model run — that needs the chunking, not the cap; and capping makes "we skipped this row" indistinguishable from "this note has no summary," which is a new small silent failure unless a placeholder is added.

### Option C — give it a progress bar and a Cancel button

**For.** The cold case is genuinely invisible today, and one instance of it is unavoidable by design: bumping the summary algorithm version invalidates all 7,814 rows at once. On that day, every broad search becomes a compute storm with no strip, no cancel, and errors going to a log channel that release builds have no console for (`nsc/mod.rs:581`, and the app runs without a console window, `main.rs:2`).

**Against.** Three things, and they are decisive.

1. **It makes the feature slower on purpose.** The machinery it copies pauses 30 ms between every note (`backfill.rs:32, 198`). Over 1,200 notes that is **36 seconds of pure waiting** added to the run. The headline exists to tell you what a result is about *while you are scanning results*. A headline that lands 40 seconds later, after you have already clicked through, has no purpose left.
2. **It would ship a new false-success.** The progress machinery counts a note as "completed" even when that note errored, and per-note errors are never emitted (`backfill.rs:182-187`). So a run where every federated result failed would display "1,200 / 1,200 — done." Fixing a false-void defect by shipping a false-success surface is the wrong direction.
3. **Its most valuable piece needs none of the rest.** The one part of C that actually removes work is capping the request — which is Option B. Everything else reschedules or narrates.

**Attacks it survived:** the *principle* behind its cheapest part — that these reads should not be on the database's writing lane — survives completely and is correct.
**Attacks it failed:** the "one big query on the reading lane" version of that fix holds the reading lane for ~2.5 seconds, which this repo has already explicitly ruled unacceptable in writing, after a safety sweep (`target_base_backfill.rs:152-160`: *"Slow and correct beats fast and blocking"*) — it would sit in front of the rename cascade. And the cancel-and-replace behaviour it needs is genuinely new concurrency code with a race that can leave the run **un-cancellable**.

### Ranking

1. **B+ (B combined with C's reading-lane principle, done in small chunks).** Highest value, lowest risk, precedented in your own codebase twice over.
2. **A's write-time producer — later, as its own migration**, and only if scoped honestly (it must cover notes *you* write, which means it cannot be callout-only).
3. **A's read-only mode alone** — do not ship without a producer.
4. **C's job machinery** — do not build.

**They combine.** B (cap + chunk + remember-the-empties + honour the collapse state) and C1 (get these reads off the writing lane, chunked) are complementary, land in one pass, and together remove both halves of the measured cost. A's producer is a separate, later migration that would eventually make B's cap almost invisible.

---

## 3. PJ-312 — what actually happens today

When a Search Hub query fails at the back end, the app throws the failure away without binding it, without logging it, and without telling you (`SearchHub.svelte:223`). What you see depends on which mode you were in:

- **In advanced mode** (any operator syntax) you get **"No results found"** (`SearchHub.svelte:630`, and `sidebar.noResults` = "No results found", `en.json:947`). Byte-identical to a genuine zero-hit query. This is the false-success the ticket describes.
- **In normal mode** it is worse. I traced the render chain and there is no final "otherwise" branch: after a failure, no branch matches at all and **nothing renders** — a blank white panel, and the result count in the header disappears too (`SearchHub.svelte:546-683`, `:474`). Not "looks like no matches" — **looks like the app forgot to draw.**
- **In advanced multi-term mode** a mid-run failure leaves the groups that already succeeded on screen, counted in the header as the total, with no sign that the rest never ran.

Release builds have no developer console, and the failure writes no log line, so **today a back-end search failure leaves no trace anywhere in the app.**

Two things I verified that change how this should be fixed:

1. **A normal universe switch does not produce this error.** Both search commands re-open the index themselves if it isn't open (`search.rs:13461-13469`). The "Search index not available" error is only reached when a *second* switch lands during an in-progress initialisation (`search.rs:11681-11692`). It is rare, not common.
2. **The two other ways a search can fail are permanent, not transient** — a corrupted lock, or an index that cannot be opened at all. A retry would fail twice instead of once.
3. **Most wrong-empty answers never reach the frontend at all.** Below the search command, every one of the six category lookups turns a database error into an empty list (e.g. `search.rs:13565-13580`). A corrupt index, a schema mismatch, or a quotes-only query returns a confident "nothing found" that **no amount of frontend error handling can ever see.**

There is also a separate, related defect on the same screen: the "..." spinner can stick **forever**. Type one character and backspace within 300 ms and the spinner is switched on but never switched off (`SearchHub.svelte:161-162` versus `:224`). When that happens, nothing else on the panel can render.

---

## 4. PJ-312 — the options

| Option | What you'd notice | Effort | Risk | What it gives up |
|---|---|---|---|---|
| **A — Say it failed** (bind the error; add the missing "otherwise" row; fix the stuck spinner in the same commit) | Instead of a blank panel or a false "No results found," a line saying the search could not run — and no more permanent "..." | ~½ day + one phrase in 15 languages | **Low** | Doesn't touch the majority failure family (the back end reporting success for its own failures) |
| **B — Retry once, then show a banner** | Rare failures self-heal; the rest appear on the red bar at the top of the window | ~1 day + a test round | **Medium** | Adds up to 1.5 s of delay to **six** search surfaces to cure a failure that, verified, is neither common nor retry-able |
| **C — Make the back end stop reporting success for its own failures** | An empty result caused by a broken index would say "Titles and Contents could not be searched," with a route to Repair | Multi-session migration | **High** | Ships this cycle |

**For A.** It is provable without a reproduction harness — you can reach the blank-pane state by setting three variables, so it clears the Reproduce-First bar cheaply. Search Hub already has its own fallback helper for a phrase whose translation hasn't landed yet (`SearchHub.svelte:38`), so it ships in one commit. And it fixes the worst-looking symptom on the screen.

**Against B — and this is why I would not do it.** The retry is copied from the note-reindex path, where its own comment says it absorbs a transient lock contention (`store.ts:4205-4209`). But in *search*, that same contention is not an error — it is swallowed into an empty result list and never rejects. So the retry would fire only on the three rejections that exist, and **two of the three are permanent.** It would also put the delay in the shared wrapper used by the Map, Sight, GraphMind and OrgChart while giving the message to Search Hub only — the cost on six surfaces, the benefit on one. And it would post to a notice bar that holds only **one** message at a time and is currently used for "N items could not be moved" and "could not be renamed" — a repeating search failure would evict an unread data-integrity warning (`+layout.svelte:546-547`).

**Against C.** It is the right end-state and the wrong next step. Turning six "return nothing on error" paths into "propagate the error" converts today's graceful degradation into hard failures on a screen you use constantly. That is a migration with an Architect doc, not a fix.

**Attacks A survived:** keystroke rules, the concurrent-stall interaction (it adds nothing to the working path).
**Attacks A failed:** it leaves the largest silent-failure family untouched — so it must not be recorded as "PJ-312 closed."

---

## 5. Recommendation

**PJ-311 — do B+ now.** One pass, one migration-sized review, roughly one to one-and-a-half days:

1. **Cap the request**, spread across the six categories rather than taking the first N — so contents and semantic matches, where the headline actually earns its place, still get one.
2. **Fill in small chunks with a short delay**, exactly as Source Review already does (chunks of 6, half-second debounce) — this is the only thing that bounds a *cold* run.
3. **Remember which paths came back empty**, in a set **local to Search Hub** (not in the shared store — that would blank the headline under your open note's title), cleared when a new search runs and when the library changes.
4. **Move these reads off the database's writing lane**, in chunks, never one long query — so a headline pass can never stand in front of your next search or your next save.
5. **Give the headline pass the same stale-guard the search already has**, so typing a new query abandons the old pass.
6. **Add a placeholder** for a row whose headline was deliberately skipped, so "we didn't ask" is distinguishable from "this note has nothing to say."

**Conditions.** Do not gate Search Hub on the "Note summaries" switch without ruling on it yourself — that switch's own on-screen text says it governs the Backlinks and Outgoing panels, so quietly honouring it here would blank your headlines from a control that never claimed to govern them. And per Reproduce-First: the stall has been established from source and from your database, but **not yet reproduced on the running app** — the first shippable piece is arguably a timing marker in the diagnostics log that proves the recipe, and that is cheap.

**PJ-312 — do A now, in one commit, with the stuck-spinner fix.** Then file the back-end swallowing (option C) as its own numbered job with an explicit ruling from you, rather than letting it be quietly absorbed. Do **not** record PJ-312 as closed by A alone: A catches the minority of failures; the majority never reach the frontend.

---

## 6. What I would NOT do

- **I would not ship "read only what's stored" without a producer.** It trades a stall for a silent blank, and — verified against your own libraries — the blank would land on the notes *you write*, not the ones you imported. That is a worse trade than the stall.
- **I would not clone the manual-build job machinery.** It adds 36 seconds of deliberate waiting to a 1,200-note run, its payoff (yielding the model) is inert on default settings and ~97% inert on your corpus, and it would ship a progress bar that reports "done" over notes that failed.
- **I would not put a "one big query" on the reading lane.** Your codebase already ruled that exact shape unacceptable after a safety sweep, in writing.
- **I would not add the retry to the search path.** Two of the three failures it would catch are permanent, the common one it claims to catch is not actually produced by a normal universe switch, and it would slow six surfaces to help one.
- **I would not put the search-failure message on the persistent store-health bar.** That bar shows one condition at a time by design; a per-query message there could hide "your settings are not being saved" for a whole session.
- **I would not fix only Search Hub.** Three other panels (Index, Digest, Reviewer) call the same compute-on-read command with neither a cap nor the settings gate. Per the Whole-Ecosystem Fix Law, the cap and the reading-lane change belong in one shared place so they cannot drift apart again.

---

## 7. Open questions only you can answer

1. **When a headline is deliberately skipped for cost, should the row show something?** A faint "—" or nothing at all? Nothing is cheaper and matches Backlinks; something is honest and matches your Form-Aligns-To-Purpose principle. I lean to a placeholder, but this is your call about your screen.
2. **Should the "Note summaries" switch in Settings govern Search Hub too?** Today it does not, and its own text says it governs two other panels. Either answer is defensible; whichever you pick, that text needs rewording in all fifteen languages.
3. **How many headlines is enough on one screen of results?** The siblings chose 120. The visible window is roughly 11–16 rows. 40 spread across the six categories would be my proposal — but the number is a judgement about how you scan a long result list, and you are the person who scans it.
4. **For the 443 notes whose file is newer than their stored headline** — should a stale headline be shown as-is, or hidden until rebuilt? Today they silently self-correct, at the cost we are trying to remove.
5. **Do you want a way to build summaries for notes you write, without pressing a button in Cataloger?** That is the real Rule 8 gap. It is a migration, and it is the thing that would eventually make all of this a non-issue.
6. **Is the back-end reporting success for its own failures (PJ-312's option C) worth a migration this cycle, or is it a numbered job for later?** It is the family most likely to give you a wrong "No results found" on a 2 GB index — but fixing it changes how the search reports errors everywhere.

---

**Unverified, stated plainly:** I did not run the app; nothing here has been observed as a user-visible symptom. I did not run the language model, so every inference-cost figure is the codebase's own comment, not a measurement. The 2.5-second / 44 MB figure was measured against your live index with Python, not with the app's own database code — right order of magnitude, not a benchmark. I did not check whether federated (child-universe) results reach the headline lookup at all; the code path suggests they would fail and be swallowed, but that depends on your attached universes, which I did not inspect. And I did not measure the real height of the results panel on your screen, so "11–16 visible rows" is arithmetic, not observation.