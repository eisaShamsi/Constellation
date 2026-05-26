# MIG-055 Boss-test — Stage 1 of 5

**Stage 1 validates the canonical happy path: the system-shipped "Five Acts → Observation — Recent Captures" host note appears in the sidebar, opens cleanly, and renders the last 14 days of your notes as a clickable list.**

If Stage 1 passes, the foundation of MIG-055 (the entire §A–§G stack on your real universe) is verified. We'll move to Stage 2 once you confirm Stage 1.

---

## What MIG-055 is (one paragraph)

MIG-055 is the **clean rebuild** of Constellation's lens/base system. Where the old Bases tried to be a generic table of any frontmatter property, the new Constellation Base is **purpose-built for the Five Acts of Knowledge Creation** — Observation, Connection, Tension, Synthesis, Conviction. v1 ships the first Act: **Observation — Recent Captures**, the "intake queue" view of the last 14 days of notes across your universe. The lens definition lives inside a normal markdown note in a fenced code block; the rendered view is a live list of notes with their NSC headlines. File-Over-App: delete the markdown, the lens is gone. No proprietary container.

---

## Stage 0 — Verify the binary

Before testing the feature, confirm you're running the right build. The installer for this test is:

```
E:\مشاريع كلاود\Constellation\src-tauri\target\release\bundle\nsis\Constellation_0.1.0_x64-setup.MIG055.exe
```

(I'll rename the freshly-built `Constellation_0.1.0_x64-setup.exe` to add the `MIG055` suffix once the build finishes. The file's modification timestamp will be **today, 2026-05-26**.)

**To verify before testing:**

1. Right-click the installer file in File Explorer → **Properties**.
2. Look at "Date modified". It must say **today's date (2026-05-26)** with a recent time.
3. If the date is older than today, **STOP** — that's a stale binary and the test results would be misleading. Tell me, and I'll rebuild.

**Install / launch:**

4. Close any running Constellation instance first (right-click the taskbar icon → Close, or use Task Manager to confirm `constellation.exe` has no processes).
5. Double-click `Constellation_0.1.0_x64-setup.MIG055.exe` → click through the installer with defaults.
6. Launch Constellation from the Start Menu (or the desktop shortcut).

You should land on whichever universe was active last.

---

## Stage 1 — The canonical Recent Captures lens

### What we're testing

When MIG-055 boots into a universe (any universe, including yours that has thousands of notes), it should:
1. Auto-create the file `{universe}\Five Acts\Observation — Recent Captures.md` if it doesn't exist (transfer-on-edit honored if it does — your edits are never overwritten).
2. Show a new "**Five Acts**" section in the left sidebar.
3. List "**Observation — Recent Captures**" as a clickable entry in that section.
4. When you click it, the host note opens as a normal markdown tab — with your usual prose at the top, then the embedded lens block rendering the last 14 days of notes as a list with headlines.

### Step-by-step

**Step 1.1 — Look at the left sidebar after Constellation finishes booting.**

- Expected: above the existing **Workspace Bases** section (if you had one), you see a new section header:
  - A chevron (▸) that points right when collapsed, down when expanded
  - A small clock-circle icon
  - The text **Five Acts** (or, if you're testing in a non-English locale, the translated name — Arabic: **الأفعال الخمسة**, Spanish: **Cinco actos**, etc.)
- Failure modes:
  - **"Five Acts" section is missing entirely** → the §E system-note bootstrap didn't run, or `init_five_acts_system_notes` returned an error. Tell me what universe you're in.
  - **The section is there but empty** → the system note wasn't created, or the sidebar enumerator can't see it. Check whether `{universe}\Five Acts\Observation — Recent Captures.md` exists on disk via File Explorer.

**Step 1.2 — Click the "Five Acts" header to expand it (if it isn't expanded already).**

- Expected: the chevron rotates to point down. Below the header, a single entry appears:
  - A small file/page icon
  - The text **Observation — Recent Captures**
- Failure modes:
  - **The chevron rotates but no entries appear** → the file exists on disk but the Tauri command `list_five_acts_notes` is failing. Tell me.
  - **A different file name shows up** → that means `{universe}\Five Acts\` has other `.md` files in it. That's fine — those will also show. The canonical one is what we care about.

**Step 1.3 — Click "Observation — Recent Captures".**

- Expected: a new editor tab opens. The tab title shows "**Observation — Recent Captures**" (or close to that). The tab is for an ordinary `.md` file, so it gets the regular NotePane editor.
- Failure modes:
  - **Nothing happens** → the `openNoteTab` handler didn't route. Check whether the entry shows as "active" (highlighted) afterwards.
  - **"File not found" error** → the relative path from the sidebar didn't resolve to the actual file. Tell me the exact error message.

**Step 1.4 — Look at the note's contents.**

You should see three regions (top to bottom):

a) **A frontmatter banner / collapsed YAML block** (depending on your livePreview setting) containing two keys:
   - `template: five-acts.observation`
   - `description: "The intake queue — last 14 days of notes. Browse what you've recently captured."`

b) **Prose:**
   - A heading: **Observation — Recent Captures**
   - A paragraph starting "The Observation Act of knowledge formulation is **noticing**..."
   - A second short paragraph starting "Scan, read, mark as processed, or develop further..."

c) **The lens block.** This is the key visual — it's where MIG-055's new code lives:
   - A bordered rectangle with rounded corners, slightly shaded background.
   - Header row inside: the lens name "**Recent Captures**" on the left, and a small count chip on the right showing how many notes matched (e.g., `12`, or `0` if your universe has no notes from the last 14 days).
   - Below the header: either
     - A list of rows, one per recent note, OR
     - An italic "**No notes match this lens.**" if zero notes match.
   - Footer row at the bottom: a small "Xms" timing (the lens query time — typically under 50ms on a small universe, can be a few hundred ms on a large one).

**Failure modes for the lens block:**

- **You see a raw YAML block instead** (lines starting with `schema: 1`, `lens: "Recent Captures"`, etc., in a code-block style with no border) → the CM6 widget didn't replace the fenced code block. Most likely cause: cursor was placed inside the block (the block becomes editable when the cursor enters it — try clicking outside).
- **You see "Loading lens…" forever** → the `execute_lens` IPC call hung or never returned. Tell me — this would be a hot bug.
- **You see "Lens error: ..."** in red → the validator rejected the YAML. Tell me the exact error message — most likely it means the canonical YAML drifted, which the §G drift catch was meant to prevent.
- **The block shows the header but the rows section is empty / showing 0 even though you have recent notes** → either `note_meta.created_at` is missing values, or the SQL query is wrong. Tell me whether the count chip says 0 or some other number.

**Step 1.5 — Look at the rows in the lens block.**

Each row should look like:

```
<note name> — <NSC headline sentence>
```

Examples (depending on what you've recently captured):
```
Apple Tree Fruit — Reflects on the conditions for synthesis between observed contradictions.
Lunch Plan — Documents the next two days' constraints in plain prose.
```

- The note name is **clickable** — it should look styled like a link (probably purple/accent color, possibly underlined on hover).
- The headline (the sentence after the —) is the NSC-generated summary; it may be empty for notes that don't have a summary yet, in which case the row just shows the name with no separator.

**Step 1.6 — Click any row's note name.**

- Expected: that note opens in a new tab (or the active tab switches to it). The note loads in the regular editor.
- Failure modes:
  - **Nothing happens** → the `constellation:open-note` event didn't route. Tell me — this would mean the §H.1 hotfix is incomplete.
  - **A different note opens** → the libraryName routing went wrong. Tell me which note opened vs. which you clicked.

---

## What "Pass" looks like for Stage 1

- Five Acts section appears in the sidebar.
- Observation — Recent Captures entry expands and is clickable.
- The host note opens with prose + lens block visible.
- The lens block shows your actual recent notes (or "No notes match this lens" if your universe legitimately has zero in the last 14 days).
- Clicking a row name opens that note.

If all six steps pass, reply **"Stage 1 passes"** and I'll send Stage 2.

If anything fails, copy/paste any error messages and tell me which step broke. I'll diagnose before moving on.

---

## What we'll cover in Stages 2-5 (preview only)

- **Stage 2** — Empty universe / no-NSC-headlines edge cases. UI states.
- **Stage 3** — Federation: rows from cUniverse-linked universes appear correctly.
- **Stage 4** — Edit the host note's prose, save, reload — content persists, lens still works.
- **Stage 5** — Universe switch (the §H.1 hotfix path). Switch to another universe; the Five Acts section updates to that universe's set.

(These are listed for your awareness only. **Do not run them yet** — let's confirm Stage 1 first per the staged-tests preference.)
