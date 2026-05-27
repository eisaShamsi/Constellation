# MIG-059 Boss-test

**Tests the per-cUniverse Connection pre-warm fix on your live Eisa Universe + cUniverses.**

Two checks: (1) federated search returns results within ~1s instead of the ~25s wait you saw during §K.3; (2) the Arabic input truncation (MIG-058) may resolve as a side effect — if the search is fast enough that the async never completes mid-typing, the IME-disruption hypothesis is wrong and MIG-058 doesn't need its own fix.

---

## What MIG-059 fixes (one paragraph)

The per-cUniverse Connection in §K.3 was opened cold — no WAL checkpoint, no FTS5 vtable initialization. The first federated search had to scan the cUniverse's accumulated WAL file AND lazily initialize hundreds of FTS5 segment-index pages from cold disk — 15-27 seconds on your cu1 data. MIG-059 adds two best-effort pre-warm steps right after the Connection opens (background-attach thread, off the UI critical path): a passive WAL checkpoint and a no-op `SELECT COUNT(*) FROM notes_fts` to force FTS5 vtable init. Total boot cost: ~50-200ms per cUniverse. Search latency: ~1s instead of ~25s on first query.

---

## Stage 0 — Verify the binary

**Installer:**
```
E:\مشاريع كلاود\Constellation\src-tauri\target\release\bundle\nsis\Constellation_0.1.0_x64-setup.MIG059.exe
```

1. Right-click → Properties → "Date modified" should say **today, afternoon, post-13:36**.
2. Close any running Constellation.
3. Double-click the installer → click through.
4. Launch Constellation from the Start Menu.

---

## Stage 1 — First federated search returns in ~1 second

### What we're testing

The most-obvious symptom of §K.3's first-search penalty. Open the search, type something, and time how long until results appear. Pre-fix: 15-27 seconds. Post-fix: ~1 second.

### Steps

1. Confirm **active universe is Eisa Universe** (bottom-left pill).

2. **Wait 15 seconds** after launch. The pre-warm runs during background-attach (which itself takes a few seconds for federation to complete); waiting 15s ensures the warm-up is fully done before you search.

3. Press **Ctrl+O** to open the QuickSwitcher.

4. Type or paste **`الرباط`** (the same query from MIG-057's test).

5. **Time how long until results appear.**

### Expected

- Results appear within roughly **1 second** (similar latency to single-schema search).
- The result list is the same as MIG-057's test — `الرباط` at rank 1-2, geography cluster following.

### Pass criteria

Federated search returns results within ~1-2 seconds. Reply **"MIG-059 passes"** with rough timing if you can.

### Failure modes

- **Still 15-25 seconds** → pre-warm didn't engage. Possible: you launched too fast (re-test with 20s wait). Tell me if it persists.
- **App crashes during boot** → tell me. The pre-warm code is `let _ = ...` swallowed so failures shouldn't crash; if it does, P1 fix.
- **No results at all** → unrelated regression. Compare to MIG-057 behavior.

---

## Stage 2 — Bonus: does MIG-058 (Arabic input truncation) also resolve?

### What we're testing (hypothesis)

If MIG-059's root cause (slow async search resolving mid-typing) was ALSO the cause of MIG-058 (Arabic input truncation), then fixing MIG-059 fixes both. The simple test: type a longer Arabic word slowly and see if it gets cut off.

### Steps

1. Press **Ctrl+O** (close the previous search if open).

2. Type **`الرباط`** slowly — about 300-400ms between each character. Don't paste; actually type each character.

3. After you finish typing all 6 characters (ا-ل-ر-ب-ا-ط), look at the search input box.

### Expected (if hypothesis correct)

- The search box shows the full `الرباط` (6 characters).
- Results below show the expected geography cluster.

### Expected (if hypothesis WRONG — MIG-058 is a separate Svelte/IME issue)

- The search box shows truncated text — `الربا` or shorter.
- We confirm MIG-058 is genuinely a separate problem and needs its own investigation.

### Pass criteria

- **Best case:** input shows full `الرباط` → reply **"MIG-058 also resolved"** (we close both at once).
- **Worse case:** input still truncates → reply **"MIG-058 still broken"** (we open MIG-058 for real Svelte/IME investigation next session).

Either outcome is useful information.

---

## After MIG-059 passes

The original three pre-existing issues from MIG-056 §K Boss-test are now: 2/3 shipped + 1/3 either side-effect-resolved or scoped for separate investigation. State:

| MIG | Status |
|---|---|
| MIG-057 — Lexicon expansion + prefix-wildcard | ✅ Shipped + verified |
| MIG-058 — QuickSwitcher Arabic truncation | Hypothesis-dependent (test Stage 2) |
| MIG-059 — Slow per-cUniverse search | ✅ Shipped — verify Stage 1 |

After Stage 1 + Stage 2 results, you decide what's next: pause, open MIG-058 for real if Stage 2 fails, or move to entirely different priorities.
