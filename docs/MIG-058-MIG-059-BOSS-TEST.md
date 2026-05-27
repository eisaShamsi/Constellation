# MIG-058 + MIG-059 Combined Boss-test

Three stages — same install, three observations. Tests the combined federation-speed + Arabic-input fix.

---

## What the combined fix delivers (one paragraph)

Two issues from the §K Boss-test, both with sourced fixes after 4 parallel research agents:

**MIG-059 (federation speed):** SQLite FTS5 was picking catastrophic query plans for 9-term OR expressions because `sqlite_stat1` (the query planner's statistics table) wasn't populated on the per-cUniverse Connection. Fix: `PRAGMA optimize` in init_db + on per-cUniverse Connection (refreshes stats), plus `PRAGMA mmap_size=256MB` (OS-shared page cache so the standalone Connection doesn't pay cold-cache cost). Expected speedup: ~25s → ~1s. Source: SQLite Forum thread "JOINs with FTS5 are very slow" where ANALYZE took 170s → 0.259s.

**MIG-058 (Arabic input truncation):** NOT the bind:value race I'd hypothesized — Svelte 5's source already guards against that. The actual cause is synchronous main-thread pressure: `filtered` was a `$derived.by` that rebuilt on every keystroke (walking 1101 notes + filter + slice), and the keyed `{#each filtered (note.path)}` re-rendered. Under WebView2 pressure, Arabic keystrokes got dropped at slow typing speed. Fix: `filtered` is now `$state` updated only inside the 300ms debounced effect; typing no longer triggers per-keystroke filter + re-render. Plus composition-event handlers as free insurance for CJK users.

---

## Stage 0 — Verify the binary

**Installer:**
```
E:\مشاريع كلاود\Constellation\src-tauri\target\release\bundle\nsis\Constellation_0.1.0_x64-setup.MIG058-059.exe
```

1. Right-click → Properties → Date modified should say **today, late afternoon/evening, post-17:00**.
2. Close Constellation.
3. Install.
4. Launch.

---

## Stage 1 — Federation status bar shows the federated count

### What we're checking

The MIG-059 revert restored federation but verified-MIG-057 federation attaches normally. The new MIG-059 pre-warm replacement (PRAGMAs only, no blocking pre-warm) should NOT delay attach. The status bar should show the federated total within ~5 seconds.

### Steps

1. Confirm **active universe is Eisa Universe**.
2. Wait **10 seconds** after launch.
3. Look at the bottom-right of the sidebar.

### Expected

- Status bar: "**25 libraries · 8751 notes · ⚠ 1 · Eisa Universe**" (or your exact count — 1101 main + cu1's notes).
- The `⚠ 1` warning badge appears for `كون عيسى` (search.db missing — documented).

### Fail criteria

- If status bar still shows `1101 notes` and no warning badge → federation broke again. Tell me; revert candidate.

---

## Stage 2 — First federated search is fast

### What we're checking

The `PRAGMA optimize` + `PRAGMA mmap_size` changes should make the per-cUniverse Connection's first BM25 query run in ~1 second instead of ~25 seconds.

### Steps

1. After Stage 1 passes (status bar = 8751), press **Ctrl+O** to open search.
2. Type or paste **`الرباط`**.
3. **Time how long until results appear.**

### Expected

- Results appear within roughly **1 second**.
- Same result quality as MIG-057's verified output: `الرباط` at rank 1-2 (highlighted), surrounded by the geography cluster (`المرابطون`, `الموحدون`, `الدار البيضاء`, `المغرب`, `مراكش`, `نواكشوط`, `فاس`, etc.).

### Fail criteria

- If results take ~25 seconds → planner stats fix didn't engage. Tell me; we re-research.
- If `الرباط` is missing from results → MIG-057 fix regressed somehow. Tell me.

---

## Stage 3 — Arabic slow-typing lands all characters

### What we're checking

The frontend fix (debouncing `filtered` instead of `$derived` rebuild every keystroke) should remove the main-thread pressure that was dropping Arabic keystrokes.

### Steps

1. Press **Ctrl+O** to open search (close any previous search).
2. **Type `الرباط` slowly** — about 300-400ms between each character. Don't paste; actually type each one.
3. After completing all 6 characters (ا-ل-ر-ب-ا-ط), **look at the search input box**.

### Expected

- The input box shows the full `الرباط` (6 characters).
- After ~300ms of no further typing, results populate showing `الرباط` at the top (or near it).

### Fail criteria

- If input still truncates to `الربا` or shorter → frontend fix didn't engage OR there's a deeper WebView2 issue beyond what the research revealed. Tell me; we re-investigate.

---

## Pass summary

If all 3 stages pass:

- **MIG-059 closed:** federation search runs at active-mode speed (~1s).
- **MIG-058 closed:** Arabic slow-typing lands all characters.
- All four MIG-056-§K Boss-test follow-ups (MIG-057, MIG-058, MIG-059) are now done.

Reply with which stages pass / fail. Each one is informative:

- Stage 1 fails → federation regression; we go back to MIG-056 baseline.
- Stage 1 passes, Stage 2 fails → planner stats fix wrong; need different SQLite approach.
- Stages 1+2 pass, Stage 3 fails → frontend fix incomplete; need to investigate WebView2 + Arabic deeper, possibly the dev-build devtools test the Agent 4 research recommended.
- All three pass → ship + close all three MIGs in §L orientation v2.39 bump.
