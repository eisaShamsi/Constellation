# MIG-060 — Boss Test: Threading Gestures from Lens Rows

**Date:** 2026-05-28
**Architect:** `docs/MIG-060-base-phase-1.5-host-note-gestures-ARCHITECT.md`
**Plan:** `docs/MIG-060-base-phase-1.5-host-note-gestures-PLAN.md`

---

## What is this feature?

Until today, the only thing you could do with a lens row was **click the note name** to open it in a regular tab. That covers reading the note — but it doesn't open the note in any of Constellation's deep-read surfaces:

- **360.3D Inspector** — the multi-dimensional view that shows the note's properties as orbiting rings around the note's body.
- **CNS (Constellation Nervous System)** — the gravity-well view that shows Universe Health, Communities, Top Bridges, and Blind Spots. (Internally still wired through `lensActive` / `toggleLens()` — old code names retained; the user-facing label is CNS. Not to be confused with the retired Constellation Sight dome, which is a future plugin candidate.)
- **The Cataloger** — the structured-cataloguing view (epistemic taxonomy + Sources).

**MIG-060 closes that gap.** Every lens row now carries three small icon buttons on its trailing edge. One click sends the note **directly into the surface you want**:

```
[ Note name ] — [ headline ]                          🔮  👁️  📚
                                                       ↑   ↑   ↑
                                                       │   │   └─ Open in The Cataloger
                                                       │   └───── Open in CNS
                                                       └───────── Open in 360.3D
```

(The actual SVG icons are different from the emoji above — those are just shorthand for this doc.)

## Why does this matter?

Constellation is the only PKM where four surfaces — **Note → 360.3D → CNS → Cataloger** — see the SAME note through different cognitive lenses. Before MIG-060, jumping between them was a click-the-note-then-go-find-the-dock-button dance. With the threading gestures, the four-surface workflow is now **one gesture from anywhere a lens lives** — five-acts notes, custom lens blocks, the dashboard, anywhere.

The icons are intentionally small (12px) and live faded at 55% opacity until you hover the row. They never compete with the note name for attention; they're there when you reach for them.

---

## Test stages

> **Stage 0** (Claude's side, already done): Eisa does not need to touch this — it's the pre-flight check that the running binary actually has the MIG-060 code. If the test fails because the icons aren't there, that's the first thing to check.

### Stage 1 — Visual: do the icons appear?

**Pre-state:** Constellation is open on your Eisa Universe. The active library has at least one note that contains a lens block (the easiest one is the **"Observation — Recent Captures"** five-acts note in the Library Notes, but ANY note with a working ` ```base ` block will do).

**Action:**
1. Open the **"Observation — Recent Captures"** note (or any note with a lens block).
2. Look at any lens row.
3. Hover your mouse cursor over the row.

**Expected post-state:**
- Three small grey icons appear on the row's trailing edge (right side in English rows, left side in Arabic rows).
- The icons are faintly visible BEFORE hovering, and turn fully opaque ON hover.
- Each icon has a tooltip that appears if you rest the cursor on it:
  - Leftmost (when LTR) shows tooltip: **"Open in 360.3D"** (or its translation in your active locale).
  - Middle: **"Open in CNS"**.
  - Rightmost (when LTR): **"Open in The Cataloger"**.

**Failure modes:**
- *No icons visible at all.* → §B didn't ship to the running binary, OR §D CSS didn't apply. Check that the binary's modification time is AFTER today's commits.
- *Icons visible but no tooltip on hover.* → §A i18n didn't load, OR the locale file is missing the new keys.
- *Only 2 icons visible (CNS missing).* → Check **Settings → Core Plug-Ins → "Constellation Nervous System (CNS)"**. If OFF, that's expected behavior (§B Architect lock Q4) — turn it ON to see all three. (Internal note: the user-flag is `enabledFeatures.constellationSight`, kept under the old name from when this surface was called Sight v2; the user-facing label is CNS.)

---

### Stage 2 — 360.3D gesture

**Pre-state:** You can see the three icons on at least one lens row (Stage 1 passed).

**Action:**
1. Click the **360.3D** icon (the one with the concentric circles + crosshair lines — leftmost in LTR rows).
2. Watch what happens.

**Expected post-state:**
- The clicked note opens as the active tab in the main editor pane.
- The **360.3D Inspector** view activates, replacing the editor view.
- The Inspector shows the note's properties as orbiting rings.
- The dock button for 360.3D is highlighted (active state).
- The Inspector is showing data for the JUST-CLICKED note, not whatever note was open before.

**Failure modes:**
- *Nothing happens.* → §C listener didn't ship, OR the event detail shape is wrong.
- *Note opens but 360.3D doesn't activate.* → §C `case '360.3d':` branch is missing/wrong.
- *360.3D opens for the wrong note.* → `await tick()` between openNoteTab and surface flip didn't fire, OR the host note never became the active tab.

---

### Stage 3 — CNS gesture (only if you have CNS enabled)

**About CNS:** Constellation Nervous System — the gravity-well + Universe Health + Communities + Blind Spots surface. Internally still wired through `lensActive` / `toggleLens()` (the names date from when this surface was called "Sight v2"); the user-facing label has been "CNS" since the rename. **CNS is a live, active core surface** — not to be confused with the retired Constellation Sight (the dome view that was carved out to be a future external plugin per MIG-038).

**Pre-state:** Stage 1 + Stage 2 passed. **Check Settings → Core Plug-Ins → "Constellation Nervous System (CNS)" is ON.** If OFF, skip this stage — the CNS icon shouldn't be visible anyway, and §B's gating is correct.

**Action:**
1. Click the **CNS** icon (the eye-shape icon — middle).
2. Watch what happens.

**Expected post-state:**
- The clicked note opens as the active tab.
- The **CNS** view activates (gravity-well overlay).
- The dock button for CNS is highlighted (active state).
- CNS is operating on the JUST-CLICKED note.

**Failure modes:**
- *Nothing happens despite icon being visible.* → §C `case 'cns':` branch missing.
- *Note opens but CNS doesn't activate.* → `toggleLens()` was not called, or `lensActive` was already true and the toggle flipped it OFF instead of leaving it ON.
- *CNS opens for the wrong note.* → same as §2's wrong-note failure.

---

### Stage 4 — Cataloger gesture

**Pre-state:** Stages 1 + 2 (and 3 if applicable) passed.

**Action:**
1. Click the **Cataloger** icon (the layered-stack / 3D-cube icon — rightmost in LTR rows).
2. Watch what happens.

**Expected post-state:**
- The clicked note opens as the active tab.
- **The Cataloger** view activates.
- The dock button for Cataloger is highlighted.
- The Cataloger is showing the JUST-CLICKED note.

**Failure modes:**
- *Nothing happens.* → §C `case 'cataloger':` branch missing.
- *Cataloger opens for the wrong note.* → tick race; see §2.

---

### Stage 5 — RTL parity

**Pre-state:** You have an Arabic-named note (or at least a row whose `name` is Arabic) visible in a lens.

**Action:**
1. Look at a row with an Arabic name.
2. Observe where the three icons appear.

**Expected post-state:**
- The icons appear on the **visual right** of the row (which is the DOM left, because RTL flips the row's visual ordering).
- Tooltips on hover still work.
- Clicking any icon still opens the host note in the requested surface (Stages 2-4 logic is direction-independent).

**Failure modes:**
- *Icons on the visual left (wrong side) in RTL.* → CSS `marginInlineStart: auto` didn't auto-flip; check that the row's `dir` attribute is set to `rtl` (it should be — `_renderRow` calls `detectDir(row.name)` on every row).
- *Icons on the right but tooltips read left-to-right.* → That's actually fine. Tooltips render in the system's UI direction, not the row's.

---

## After all stages pass

Reply with **"All pass"** and Claude will:

1. Cascade to **§G (PCS)** — orientation bump to v2.40 + MoCh + help-doc updates (English + 15 locales) + milestone tag `milestone/mig-060-base-phase-1.5-shipped` + ZIP backup.
2. Mark MIG-060 as **shipped** in the next orientation entry.

If anything fails, paste the failure mode (per the "Failure modes" sections above) and Claude will triage before shipping §G.

---

## Build status

To run this test you need a build that contains commits §A through §E (today's MIG-060 work). Build the dev binary with:

```
npm run tauri dev
```

or build the release with:

```
npm run tauri build
```

(The dev build is faster to spin up and supports devtools; the release build is what you'd ship.)

After build completes, the binary's modification time should be **after** the §E commit `b5e35112` — that's your Stage 0 sanity check.
