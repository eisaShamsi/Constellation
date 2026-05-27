# MIG-056 Boss-test (§K Gate)

**Tests cross-universe federation on your live Eisa Universe + 2 cUniverses (Eisa Cognitive Knowledge + كون عيسى).**

Five staged tests per `feedback_staged_tests.md` — one stage at a time; you confirm pass/fail; we move to the next.

---

## What MIG-056 is (one paragraph)

MIG-056 closes the architectural gap surfaced by MIG-055 §I Stage 5: cross-universe data (counts, lens federation, global search) was invisible because each universe has its own `search.db`. MIG-056 adds a **federation layer** that ATTACHes each cUniverse's `search.db` to the active universe's connection at boot (background thread, post-paint), so all read-side query paths (lens, status bar, libraryStats, lexical search) see notes across the federated set. **Read-only in v1**; writes stay scoped to their source universe. Failures (missing/locked/corrupt cUniverse) degrade gracefully to a status-bar warning badge — they don't break the parent universe's search. Cross-universe `JOIN`s, AI/semantic operations, and structured/semantic search modes are explicitly out-of-scope for v1 (documented gaps; future MIGs).

---

## Stage 0 — Verify the binary

**Installer (timestamp: today, 2026-05-26):**
```
E:\مشاريع كلاود\Constellation\src-tauri\target\release\bundle\nsis\Constellation_0.1.0_x64-setup.MIG056.exe
```

1. Right-click → **Properties** → "Date modified" should say **today** (latest build).
2. Close any running Constellation (taskbar right-click → Close, or Task Manager → end `constellation.exe`).
3. Double-click the installer → click through with defaults.
4. Launch Constellation from the Start Menu.

---

## Stage 1 — MIG-055 §I Stage 5 RE-RUN (the gate failure)

**This is the test that failed in MIG-055.** With MIG-056 shipped, it should now PASS.

### What we're testing

The Observation — Recent Captures lens in Eisa Universe should return rows from BOTH Eisa Universe's own libraries AND its linked cUniverses (Eisa Cognitive Knowledge, كون عيسى). In MIG-055 it returned only 1 row (Eisa Universe's). Now it should return more.

### Steps

1. Make sure your **active universe is Eisa Universe** (the one with cUniverses linked). Check the bottom-left Universe pill in the sidebar.
2. From the **Five Acts** sidebar section, click **"Observation — Recent Captures"**.
3. Wait ~5 seconds (background-attach of cUniverses runs post-boot; per Architect §6.3).
4. Look at the lens block.

### Expected

- The bordered card renders with **more rows than before** (specifically, rows from Eisa Cognitive Knowledge + كون عيسى should appear alongside Eisa Universe's own).
- The count chip in the header shows a number **greater than 1** (likely 5-10+ depending on how recent notes are in each cUniverse).
- Hover over each row name — the **tooltip path** tells you which universe the note is from. Rows from cUniverses have paths outside `E:\Constellation Universes\Eisa Universe\`.

### Fail modes

- **Still shows only 1 row from Eisa Universe** → federation didn't engage. Most likely cause: background-attach hasn't completed. Refresh the note (close + reopen the Observation tab). If still 1 row → tell me.
- **Red error message** → tell me the exact text.
- **App crashes** → tell me. P1 hotfix needed.

### Pass criteria

Rows from at least one cUniverse visible in the lens. Reply **"Stage 1 passes"** when confirmed.

---

## Stage 2 — Status bar total notes

### What we're testing

The bottom-right "X notes" figure in the status bar should reflect main + cUniverses combined, not just Eisa Universe's own.

### Steps

1. Look at the bottom-right corner of the sidebar.
2. Note the current "X libraries · Y notes" figure.

### Expected

- Pre-MIG-056 you saw "**25 libraries · 1101 notes**" (only Eisa Universe's own notes counted).
- Post-MIG-056 the notes figure should be **larger** — total notes across Eisa Universe + Eisa Cognitive Knowledge + كون عيسى.
- The libraries count (25) stays the same — that was already federated.

### Fail modes

- Notes figure stays at 1101 → federation didn't engage. See Stage 1 fail mode.
- Notes figure drops to something obviously wrong (0, or way less than 1101) → bug. Tell me.

### Pass criteria

Notes figure > 1101. Reply **"Stage 2 passes"**.

---

## Stage 3 — Sidebar cUniverse library badges

### What we're testing

In the sidebar, expand a cUniverse entry (e.g., **Eisa Cognitive Knowledge**). Each of its libraries should now show a non-zero star_count badge (previously they all showed 0 because their notes lived in a different search.db).

### Steps

1. In the sidebar's cUniverse section, click the chevron next to **Eisa Cognitive Knowledge** to expand it.
2. Look at each library entry inside. Each should have a small count badge.

### Expected

- Library entries inside cUniverses show **non-zero counts** (the per-library note counts).
- Pre-MIG-056 they showed 0 — that was the underlying federation gap.

### Fail modes

- All counts still 0 → federation aggregation not reaching cUniverse rows. Tell me.

### Pass criteria

At least one cUniverse library shows a non-zero badge. Reply **"Stage 3 passes"**.

---
## Stage 4 — Global search across cUniverses

### What we're testing

The search bar (Ctrl+O or the search field at the top of the sidebar) should now find notes from cUniverses, not just from Eisa Universe.

### Steps

1. Pick a word/phrase you know exists in a note in **Eisa Cognitive Knowledge** but NOT in Eisa Universe's notes. E.g., a name that only appears in one of your old cUniverse notes.
2. Press **Ctrl+O** (or click the search bar) to open the search.
3. Type the word/phrase.
4. Look at the results.

### Expected

- The matching note from Eisa Cognitive Knowledge appears in the results.
- The result row shows the note's library_name (which is one of Eisa Cognitive Knowledge's libraries, not Eisa Universe's).

### Fail modes

- The matching note doesn't appear at all → federated FTS5 search not engaging. Tell me + I'll diagnose.
- The result shows but with the wrong library_name → routing bug. Tell me.

### Pass criteria

A cUniverse-only note is findable via search. Reply **"Stage 4 passes"**.

---

## Stage 5 — Failure UX (skip_unavailable model)

### What we're testing

When a cUniverse is unavailable (e.g., its search.db is renamed/moved/locked), the federation gracefully skips it + surfaces a warning badge in the status bar — **without breaking the parent universe's search**.

### Steps

1. **Close** Constellation.
2. Open File Explorer. Navigate to one of your cUniverse roots — e.g., `E:\Constellation Universes\Eisa Cognitive Knowledge\.constellation\`.
3. **Rename** `search.db` to `search.db.OFFLINE` (just for the test — we'll undo it).
4. **Launch** Constellation.
5. Wait ~5 seconds for background-attach.
6. Look at the bottom-right corner of the sidebar.

### Expected

- A small **triangle warning icon** appears with a count (e.g., "⚠ 1") between the notes count and the Universe pill.
- Click the warning icon → a popup appears showing the unavailable cUniverse path + the reason ("search.db missing").
- The rest of the app keeps working — lens still renders (with rows from Eisa Universe + any other available cUniverses), status bar shows reduced total notes, search still finds notes from available cUniverses.

### Cleanup (after pass)

7. Close Constellation.
8. In File Explorer, rename `search.db.OFFLINE` back to `search.db`.
9. Launch Constellation. Warning badge should disappear; full federation restored.

### Fail modes

- No warning badge appears → frontend didn't pick up the warning. Tell me.
- App crashes when a cUniverse is missing → fatal error path; should be skip_unavailable. Tell me + I'll diagnose.
- Lens block shows a red error → fatal-error propagation bug. Tell me.

### Pass criteria

Warning badge visible + popup informative + app keeps working. Reply **"Stage 5 passes"**.

---

## After all 5 stages pass

§K Boss-test gate is GREEN. We proceed to §L (PCS — combined MIG-055 + MIG-056 release):
- Orientation v2.37 documenting both MIGs
- 15-locale help-doc additions for the lens system + federation
- Git tag `milestone/mig-055-mig-056-combined`
- Final session log + MoCh entry

Total today: **~38 commits** across MIG-054 revert + MIG-055 (Architect/Plan/§A–§H + audit + 4 Boss-test stages + 4 hotfixes) + MIG-056 (Architect/Plan + 4-agent SME research + §A–§I + 3-agent audit pending). The MIG-055 federation limitation that we deferred at MIG-055 §J is now fixed in this release.
