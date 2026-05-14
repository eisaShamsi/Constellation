# Boss-Test — Sight v6.0 (MIG-025 §A.14 ship gate)

**Date**: 2026-05-14
**Build path**: `E:\مشاريع كلاود\Constellation\src-tauri\target\release\bundle\nsis\Constellation_0.3.4_x64-setup.exe` (123 MB, built 2026-05-14 09:28, 1m 37s build time)
**Estimated test time**: 15–20 minutes
**What you're testing**: Sight v6.0 — Phase 1 of the Coordinated Views architecture (Concept Paper v4.0).

---

## §1 — What Sight v6 is, in plain language

Sight is the screen in Constellation that shows you your **whole knowledge universe at one glance**. Every note you've written becomes a star on a circular dome. Where the star sits on the dome tells you where the note lives in your thinking.

**v6 is the rebuild we did after you said v5 was confusing.** It replaces v5's seven-mode toggle bar (where you had to click R / L / T / C / S / A / P to see different parts of the picture) with **one always-on dome** that shows multiple things at once — radial position is the maturity stratum (Foundation in the middle, Edge of Knowing at the rim), the angle around the dome is the month you wrote the note, the shape of each star is the library it belongs to, brightness is how confident you are in the note, a tiny color dot inside the star marks its lifecycle stage, and lines between stars show typed connections.

**Why it matters**: this is the test of whether the Suwaidi-style "one look, whole story" goal you set actually works. If on first open you can read the universe's health without clicking anything — where it's dense, where it's thin, where you've been thinking lately, what's connected to what — then v6 has done its job. The four remaining build phases (mini-domes, register chip, polish, automated tests) all build on what you're testing today.

---

## §2 — Before you install

**Back up nothing — there's no risk.** Sight v6 reads from a brand-new SQLite cache table (`sight_v6_layout`) that the app creates on first launch. It does NOT modify your notes, your existing v5 cache, your settings, or any file on disk except for adding fields to your settings file. v5 stays fully working alongside v6 (the dual-mount pattern).

**Two things will change in your settings file** quietly on first launch:

1. The `sight.lastMode` field gets removed (v6 has no modes).
2. A new `sight.v6MigrationDone: true` flag gets added.

Other v6-specific settings (`tourSeen`, `proMode`, `activeRegister`, `hexBinThreshold`, `linkFadeThreshold`) get their defaults (false / `'aristotelian'` / 5000 / 800).

**Existing v5 cache stays intact.** If anything goes wrong with v6, you can flip back to v5 immediately by uninstalling this build and reinstalling the previous v5 build — your data is safe.

---

## §3 — Install the build

**Installer**: `E:\مشاريع كلاود\Constellation\src-tauri\target\release\bundle\nsis\Constellation_0.3.4_x64-setup.exe`
**Size**: 123 MB
**Format**: NSIS .exe (per your stated preference over MSI). An MSI was also built alongside; ignore it.

1. **Close any running Constellation window.** (Important — the installer can't replace the running .exe otherwise.)
2. Run the `.exe` installer above. It's a standard NSIS install — Next, Next, Install. Windows may show "Unknown publisher" since this build isn't code-signed for distribution; click "More info" → "Run anyway".
3. Launch Constellation from the Start menu or the install shortcut.
4. Open your usual test Universe (your 7,636-note one, or whichever you prefer).

**Build provenance** (for the audit trail):
- Source: working tree at commit `5796f18` + uncommitted flip of `SIGHT_V6_ENABLED = true` in `src/lib/sight/engine.ts`.
- Build mode: `release` (optimized).
- v5 module set + dock button: still present (B2 dual-mount).
- Auto-update signing: skipped (no `TAURI_SIGNING_PRIVATE_KEY` set; the warning at end of build is benign for a Boss-test build).

---

## §4 — Test scenarios

For each scenario, the **expected outcome** is what should happen if Sight v6.0 is working. The **failure mode** is what to look for if it's broken — that's what to report back.

### Test 1 — First-open tour fires once, then never again

**Pre-state**: Constellation just launched on the upgraded Universe. You haven't opened Sight v6 yet in this Universe.

**Action 1.1**: In the bottom dock (the strip of buttons along the bottom), look for a star icon. You'll see two star icons now — one is the v5 Sight (existing) and the new one to its right is **Sight v6**. Hover the new one — its tooltip reads "Constellation Sight". Click it.

**Expected**:
- The full screen switches to a dark navy view with a circular dome in the middle.
- Within ~2 seconds, a centered card overlay appears titled "Welcome to Sight" with a paragraph about radial position and angle. The card has "Step 1 of 4" at top, dots showing your progress at the bottom, and "Skip tour" + "Next" buttons.

**Action 1.2**: Click "Next" three times to walk through all four steps. The fourth one's button reads "Done" instead of "Next". Click "Done".

**Expected**: The overlay disappears. The dome is now visible underneath.

**Action 1.3**: Press the **Esc** key to close Sight v6. Click the v6 dock icon again to re-open.

**Expected**: The dome appears immediately. **No tour overlay.** That's the test — the tour fires only once per Universe.

**Failure modes**:
- ❌ Tour doesn't fire on first open → tour file isn't being mounted; report.
- ❌ Tour fires every time → `tourSeen` flag isn't being saved; report.
- ❌ "Next" / "Done" doesn't advance → button click handler is broken.
- ❌ "Skip tour" doesn't dismiss → skip handler is broken.

---

### Test 2 — The dome shows your universe at a glance

**Pre-state**: Sight v6 open, tour dismissed (from Test 1).

**Action 2.1**: Just look at the dome for ~30 seconds without touching anything.

**Expected** (the §1.2 Suwaidi-criterion test):
- The dome occupies most of the screen; it's clearly the main element.
- 5 faint concentric circles divide the dome into bands. Italic labels run up the vertical center: "FOUNDATION" (innermost), "WORKING", "CONNECTION", "SYNTHESIS", "EDGE OF KNOWING" (outermost).
- 12 short month labels (JAN, FEB, …, DEC) sit just outside the outer rim, January at the top, going clockwise.
- Stars are scattered across the dome. Each star is a different shape (circle, square, diamond, triangle, hexagon) depending on which library it's from.
- Brighter stars vs dimmer ones — that's confidence (bright = established, dim = hypothesis).
- A few stars are noticeably **larger** than others — those are the top 10% most-connected notes (the "active" ones).
- Every star has a tiny colored dot in its center — that's the lifecycle stage (green = established, cyan = fresh, violet = growing, yellow = at-risk, gray = dormant). You may need to lean in to see them.
- Faint colored lines connect some stars — those are typed links between notes (green = supports, red dashed = contradicts, orange = causes, blue = exemplifies, etc.).

**At a glance, you should be able to tell**: where the cognitive density is, where the orphans are, which library dominates, when you've been thinking lately, and roughly how confident the universe is.

**Failure modes**:
- ❌ Dome is empty (no stars) → the backfill never finished or `sight_v6_layout` IPC failed; check console for errors.
- ❌ Dome appears but stays at "Preparing Sight v6 cache…" forever → backfill is hung.
- ❌ Stratum labels missing or in wrong order → font rendering or z-order issue.
- ❌ Stars all the same shape → library shape encoding broken.
- ❌ Stars all the same brightness → confidence opacity broken.

---

### Test 3 — Hover and click work

**Pre-state**: Sight v6 open with stars visible.

**Action 3.1**: Move your mouse pointer slowly over the stars without clicking.

**Expected**: When the pointer is near a star, a **gold ring appears** around it, and a small text bar in the bottom-left corner shows the note's filename / path. The cursor becomes a pointer (hand) over stars and stays a default arrow over empty dome space.

**Action 3.2**: Click any star.

**Expected**: Sight v6 closes, and the note opens in the editor as a new tab — same way as if you'd clicked a wikilink.

**Failure modes**:
- ❌ Hover gold ring doesn't appear → pointer-event wiring broken.
- ❌ Click doesn't open the note → onOpenNote callback or library-name resolution broken.
- ❌ Click closes Sight but no tab opens → openNoteTab call missing.

---

### Test 4 — Filter sidebar works (Folder TOP)

**Pre-state**: Sight v6 open.

**Action 4.1**: On the **left edge** of the dome, you'll see a thin (20 px wide) tab with a small "▶" mark. Click it.

**Expected**: A 180-pixel-wide sidebar slides out from the left, titled "FACETS" with a "◀" collapse button at the top right. Below the title you see 6 collapsible groups in this order:
1. **Folder** (this is the TOP — the LIS-critique fix)
2. Library
3. Stratum
4. Confidence
5. Stage
6. Provenance

Each group lists categories under it (e.g., Folder: "Research 1,247 / Projects 892 / …"). Each row shows the count right-aligned.

**Action 4.2**: Click "Library: Research" (substitute whichever library has the most notes).

**Expected**:
- The clicked row turns light blue.
- The dome instantly shrinks to show ONLY notes from that library.
- The counts in the OTHER facet groups (Folder, Stratum, etc.) **rebalance** to show how many of those Research notes fall into each category — that's the Hearst preview.

**Action 4.3**: Click the same "Library: Research" row again.

**Expected**: The blue highlight clears, the dome restores to all notes, all counts return to the universe-wide totals.

**Action 4.4**: Try a multi-facet filter — click "Library: Research" AND "Stratum: Working" AND "Stage: established".

**Expected**: The dome shows only Research-AND-Working-AND-established notes. Counts in OTHER facets rebalance to that intersection.

**Failure modes**:
- ❌ Sidebar tab doesn't expand → click handler broken.
- ❌ Folder facet missing or not at top → facets.ts ordering broken.
- ❌ Click on category doesn't filter the dome → toggleFilter / applyFilters broken.
- ❌ Counts don't rebalance → Hearst preview computation broken.

---

### Test 5 — Esc resets, sidebar collapses

**Pre-state**: Sight v6 open with active filter from Test 4.

**Action 5.1**: With at least one filter active and the sidebar expanded, press **Esc**.

**Expected**: Sight v6 closes (returns to the editor / dashboard you were on before).

**Action 5.2**: Re-open Sight v6 from the dock.

**Expected**: Sidebar is **collapsed** again (back to the 20 px tab). Filters are **cleared** (no blue rows). The dome shows all notes.

**Failure modes**:
- ❌ Esc doesn't close Sight → escape-handler in `+layout.svelte` for sightV6Active broken.
- ❌ Filters persist after re-open → that's actually fine for v6.0 (filters aren't persisted to settings — they're session-state); only flag as failure if you'd expected persistence.

---

### Test 6 — v5 still works (B2 dual-mount)

**Pre-state**: Constellation running, no Sight open.

**Action 6.1**: Click the v5 Sight dock icon (the original star icon, to the **left** of the new v6 one).

**Expected**: The v5 Sight surface mounts as it always has — the same one with the seven-mode toggle bar. Nothing about it changed.

**Action 6.2**: Click the v6 dock icon while v5 is open.

**Expected**: v5 closes; v6 opens. The two are mutually exclusive.

**Action 6.3**: Click the v5 icon while v6 is open.

**Expected**: v6 closes; v5 opens.

**Failure modes**:
- ❌ Clicking v5 doesn't mount v5 → §A.7 mutual-exclusivity broken; v5 is dead.
- ❌ Both visible at once → mutual-exclusivity broken.
- ❌ v5 missing from dock entirely → the v5 dock button block was accidentally removed.

---

### Test 7 — Settings migration is quiet

**Pre-state**: Constellation has been running v6 for a few minutes. (No need to do anything specific — this test inspects state.)

**Action 7.1**: Open your Universe's settings file (typically `<universe>/.constellation/settings.json` or wherever your Universe stores it). Search for `"sight":`.

**Expected** in the `sight` block:
- `lastMode` is **NOT** present (v6 quietly dropped it).
- `lastScope` IS still present (kept as a dead key for B2 safety).
- `v6MigrationDone: true` is present.
- `tourSeen: true` is present (from Test 1).
- `proMode: false`, `activeRegister: "aristotelian"`, `hexBinThreshold: 5000`, `linkFadeThreshold: 800` may or may not be present depending on whether the defaults were merged in.

**Failure modes**:
- ❌ `lastMode` is still present → migration didn't run.
- ❌ `lastScope` was removed → migration was too aggressive (would break v5 fallback).
- ❌ `v6MigrationDone` not present → sentinel didn't stamp; migration would re-run on next launch (annoying but not catastrophic).

---

### Test 8 — No console errors during normal operation

**Pre-state**: Open the developer console (Ctrl-Shift-I in the running Constellation window — Tauri allows this in dev / debug builds; release builds may not).

**Action 8.1**: Walk through Tests 1–7 again with the console visible. Watch the Console tab.

**Expected**: No red error rows. Yellow warnings are acceptable (most are pre-existing).

**Failure modes**:
- ❌ Red errors mentioning `sight_v6_*`, `SightV6`, `anchor.ts`, `facets.ts`, `tour.svelte`, or `backfillProgress` → genuine v6 bugs; copy the error text and report.
- ❌ Errors about "no such command sight_v6_*" → the IPC handler registration in `lib.rs` didn't take effect (possibly a stale build).

---

## §5 — Reporting back

For each test 1–8, please report **PASS** / **PARTIAL** / **FAIL** with:

- **PASS**: the expected outcome happened; no failure modes triggered.
- **PARTIAL**: mostly worked but with a specific oddity. Describe the oddity in 1–2 sentences.
- **FAIL**: a failure mode triggered. Quote the failure mode + any console error / screenshot.

Also welcome:
- **Suwaidi-criterion gut check**: when you opened the dome on first launch (Test 2), did you feel like you saw the universe's state at a glance? (yes / partial / no, plus 1-line reaction)
- **Anything weird** that doesn't fit a numbered failure mode but felt off — visual jankiness, layout flickers, stuck loading states, etc.

---

## §6 — What happens after the test

**On full PASS** (all 8 tests):
1. I commit the `SIGHT_V6_ENABLED = true` flip permanently as the §A.14 ship moment with a meaningful commit message.
2. I push to `main` (and `ConstellationMain` release branch if you want).
3. MIG-025 §A is closed; Phase 2 (§B mini-domes) opens for the next build cascade.
4. v6.0 is now what users see when they click the Sight dock icon (v5 stays parked behind the dual-flag for the dual-mount window).

**On FAIL** in any test:
1. The flag stays at `false` on `main` (your test was on a non-committed flip).
2. I file the failure as a §A.14-blocker bug, fix it, prepare another test build.
3. Re-run the failing scenario against the new build before declaring PASS.

**On PARTIAL**:
1. We discuss whether the oddity blocks ship or can be a v4.1 polish item.
2. Same path as PASS or FAIL based on that discussion.

---

*End of Boss-test tutorial. Awaiting your test session results.*
