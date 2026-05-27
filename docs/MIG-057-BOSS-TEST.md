# MIG-057 Boss-test

**Tests the lexicon-expansion + prefix-wildcard coexistence fix on your live Eisa Universe + cUniverses.**

Single stage. Same test query that surfaced the bug yesterday — `الربا` typed in the search box, expecting `الرباط` (the city of Rabat) to show up at the top of results.

---

## What MIG-057 fixes (one paragraph)

When you typed a short Arabic word like `الربا` (which is BOTH a corpus lemma — "interest/usury" — AND a prefix of `الرباط`), the search expanded to multi-language exact-phrase OR and lost the prefix-wildcard substring semantics. The note titled `الرباط` disappeared from results entirely. MIG-057 makes the expansion ALSO include the literal prefix wildcard (`الربا*`) alongside the lemma's translations — so notes whose tokens start with what you typed still appear, with BM25's title-weight boost putting exact-title matches at or near the top.

Pre-existing bug in single-schema mode; MIG-056's federation just made it more visible because federated search puts more notes in the candidate pool.

---

## Stage 0 — Verify the binary

**Installer:**
```
E:\مشاريع كلاود\Constellation\src-tauri\target\release\bundle\nsis\Constellation_0.1.0_x64-setup.MIG057.exe
```

1. Right-click → Properties → "Date modified" should say **today, afternoon**.
2. Close any running Constellation.
3. Double-click the installer → click through with defaults.
4. Launch Constellation from the Start Menu.

---

## Stage 1 — `الربا` search returns `الرباط` at the top

### What we're testing

The exact scenario you hit yesterday: typing the short Arabic word `الربا` (5 characters: ا-ل-ر-ب-ا) in the QuickSwitcher and getting `الرباط` (the city of Rabat, 6 characters: ا-ل-ر-ب-ا-ط) at or near the top of results, with its `جغرافيا` library label.

### Steps

1. Confirm **active universe is Eisa Universe** (bottom-left pill).

2. **Wait 10 seconds** for federation to attach (background-attach takes a few seconds + the 3-second post-boot stats refresh).

3. Press **Ctrl+O** to open the QuickSwitcher search.

4. Type **`الربا`** — the same 5-character Arabic input you tried yesterday.
   - **Note:** if MIG-058 (Arabic input truncation) is still affecting you, paste the text instead of typing. The 5 characters need to actually land in the search box.

5. Wait ~1 second after typing for the search to settle.

6. Look at the results.

### Expected

- **`الرباط`** (library: `جغرافيا`) appears in the results.
- Ideally `الرباط` is at rank 1 or close to it because the literal title match earns the BM25 column-10 weight boost.
- Other notes from the cross-language expansion still appear (notes about Rabat / interest / usury / `ربا`-rooted content) — those didn't go away.
- Result count is ~15 (the search LIMIT) with a mix of title matches and content matches.

Compare to yesterday's screenshot where `الرباط` was MISSING entirely from the top 12 results. Today, it should be visible and prominent.

### Fail modes

- **`الرباط` still missing entirely** → the fix didn't engage. Possible causes:
  - Federation hasn't attached yet (re-test after waiting 15 seconds).
  - The `الربا` token doesn't actually appear in your cUniverse's note "الرباط" body text (the fix relies on FTS5 matching tokens starting with `الربا`). If the cUniverse note's BODY doesn't include `الرباط` either, the prefix won't catch it. Tell me; we'd look at the indexed tokens.
- **`الرباط` appears but at the bottom** → BM25 isn't ranking title matches first. Possible, but unusual. Tell me + paste the diagnostic log if relevant.
- **App crashes** → tell me. P1 hotfix.

### Pass criteria

`الرباط` is visible in the results (within the top 15), preferably near the top. Reply **"MIG-057 passes"**.

---

## What MIG-058 / MIG-059 still need (NOT in scope of MIG-057)

The Arabic input truncation issue (MIG-058) and the slow cu1 branch (MIG-059) are NOT fixed by MIG-057. They're tracked as separate MIGs. If you can't get the full `الربا` typed in (the truncation), you'll need to either type fast or paste. If the search is slow (~25s), that's MIG-059's problem; the result set is still correct (per MIG-057).

After MIG-057 passes you've got the federation foundation + correct results. The other two issues remain UX papercuts but don't break correctness.

---

## After MIG-057 passes

The three pre-existing issues surfaced during MIG-056 §K Boss-test now have 1/3 resolved (lexicon boundary). The other two (Arabic input truncation, slow federated search) remain for future sessions per your decision.
