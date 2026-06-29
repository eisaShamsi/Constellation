# PJ-065 — Boss Test Tutorial (the Structural / Parent-TOC Link)

Staged. Read Stage 1, do it, tell me what you see; then I send Stage 2; then Stage 3.

---

## What this feature is (plain language)

Constellation already has **8 "thinking" links** — *supports, contradicts, causes…* — the vocabulary you use to relate one idea to another. They carry weight, confidence, and they age.

PJ-065 adds a **9th kind that is deliberately NOT a thinking link**: the **structural link** — the *compositional spine* of a work. It answers a different question: *"what is the ordered shape of the work I'm building from these notes?"* — a book's chapters, a screenplay's scenes, a Map-of-Content's outline. It is **teal**, it carries **order** (Chapter 1, 2, 3…), and — crucially — it is **invisible to all the thinking/scoring tools**: putting a note "under a Book" must never change that note's maturity, its connection counts, or its Sky View presence. A table of contents is authorship, not a claim to be judged.

You author it in a note's frontmatter, two ways (you only ever type one side — the reverse is figured out for you):
- On a **child**: `parent: "[[The Book]]"`
- On a **parent**, in order: `contains: ["[[Chapter 1]]", "[[Chapter 2]]"]`

You read it in a new **Structure** tab in the right sidebar: an **outline** of everything beneath the open note, plus a **breadcrumb** showing where the open note sits.

---

## Stage 0 — open the test book (one-time setup)

I made a ready-to-use test book so you don't have to author anything by hand.

1. Launch the freshly-built Constellation (I'll give you the exact `.exe` path with this tutorial).
2. **Open the test-book folder as a Library.** In the sidebar toolbar, click **New Library** → **Link existing folder** (the "link a folder" option, not "create"), and choose:
   `E:\مشاريع كلاود\Constellation\lab\PJ-065-test-book`
3. You should see ~10 notes appear: **The Atlas of Lost Places**, three **Part** notes, four **Chapter** notes, one **Scene**, a **README**, and a **Guard Tests** subfolder.
   - **If the notes don't appear:** the folder wasn't registered as a library — re-do step 2 and make sure you picked the `PJ-065-test-book` folder itself.

---

## Stage 1 — the Structure outline + breadcrumb (the headline test)

**The point:** confirm the structural spine renders — the outline of a work, in order, and the breadcrumb of where you are.

1. In the file list, click **The Atlas of Lost Places** to open it.
2. Look at the **right sidebar's tab strip** (the little icons at the top). There is a **new tab** — a small "list/tree" icon, sitting **right after the Backlinks tab**. Hover it; the tooltip says **"Structure"**. Click it.
   - *Pre-state:* the right sidebar was showing Properties (or whatever you last used).
   - *Action:* click the Structure tab.
   - *Post-state:* the panel switches to **Structure**, headed "OUTLINE", showing a **teal-bulleted, indented tree**:
     ```
     • Part I - The Cartographer
       • Chapter 1 - The Old Atlas
         • Scene 1 - Dust and Ink
       • Chapter 2 - A Crease in the Paper
     • Part II - The Voyage
       • Chapter 3 - The Storm
       • Chapter 4 - Landfall
     • Part III - The Shore
     ```
   - The count next to "OUTLINE" should read **8** (every descendant of the Book).
3. **Click "Scene 1 - Dust and Ink"** in the outline. It opens that note.
   - Now look at the **top of the Structure panel** — a **teal breadcrumb** should read:
     **The Atlas of Lost Places › Part I - The Cartographer › Chapter 1 - The Old Atlas**
   - That's the scene's full path up the spine. Clicking any crumb jumps to that note.

**What proves what:**
- The indented tree proves the *ordered* spine renders (Chapter 1 before Chapter 2 — that order came from the Book/Part's `contains:` list).
- Part II's chapters (3, 4) appearing proves the **other** authoring direction works (they each declared `parent: "[[Part II - The Voyage]]"` — the Book never lists them).
- The 4-level breadcrumb proves ancestors are walked correctly.

**Failure modes (tell me if you see these):**
- *No Structure tab at all* → the tab didn't register.
- *Tab is there but the outline is empty / "No structural children"* → the structural links aren't being read (or the notes didn't index).
- *The order is wrong* (e.g. Chapter 2 before Chapter 1) → the `seq` ordering isn't being applied.
- *The breadcrumb is missing or wrong* → ancestor resolution is off.

**Also please sanity-check (you're the native speaker):** switch the app UI language to **Arabic** (Settings → language) and re-open the Structure tab. The two teal pill words I added are **`يحتوي`** (contains) and **`أصل`** (parent). Tell me if those read naturally for "contains / parent" in this table-of-contents sense, or if you'd word them differently — I'll fix across all languages.

---

## Stage 2 — *(sent after Stage 1)* the no-inflation guarantee

**The point:** a structural placement must add **nothing** to a note's cognitive scores.

1. Open **Chapter 2 - A Crease in the Paper**. This note has BOTH a structural placement (it's under Part I) AND one real thinking-link (`supports: "[[Chapter 1 - The Old Atlas]]"`).
2. Open the **Backlinks** tab and the note's **maturity/health** indicators.
   - *Expected:* its **outgoing connections show only the one `supports` link** — the structural "under Part I" relationship does **not** appear as a connection, and does **not** bump its maturity. (Chapter 1, conversely, has one inbound `supports` from Chapter 2.)
3. Open the **Structure** tab while on Chapter 2 — its breadcrumb shows **The Atlas of Lost Places › Part I - The Cartographer**, and its outline is empty (it has no children).
   - *This is the whole design:* the same note shows its structural spine in the Structure tab, and its cognitive links in Backlinks — and the two never bleed into each other.

**Failure mode:** if Chapter 2's connection count or maturity looks inflated by its structural placement (e.g. counts the "under Part I" edge), that's the LL-023 leak — tell me and I'll trace it.

---

## Stage 3 — *(sent after Stage 2)* the safety guards

**The point:** malformed structure (a loop, or two parents fighting over one child) must resolve **cleanly and predictably**, never hang, never rewrite your files.

Open the **Guard Tests** subfolder.

1. **Cycle:** open **Loop Note Alpha** → Structure tab. Alpha says its parent is Beta; Beta says its parent is Alpha (an impossible loop). The breadcrumb/outline should render **the chain and then stop cleanly** (you may see a small **↻** marker where the loop was cut). It must **not** hang or freeze.
2. **Contested parent:** open **Contested Child** → Structure tab. Two notes claim it (Owner A via the child's own `parent:`, Owner B via its `contains:` list). The breadcrumb should deterministically show **Owner A** (the child's own declaration wins) — and your files are never silently changed.

**Failure mode:** any freeze/hang on the loop note, or a non-deterministic / wrong parent on Contested Child.

---

*On all-stages PASS, I close §7, then ship §8 (rename-safety probe + finalize the 15-language labels + docs) and run the Phase-4 audit.*
