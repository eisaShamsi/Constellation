# Structure

*(The compositional spine — where this note sits in the whole work)*

Constellation already gives you eight **thinking links** — *supports, contradicts, causes, exemplifies, generalizes, derives-from, part-of, supersedes* — the vocabulary you use to relate one idea to another. **Structural links** are a deliberately different kind. They don't relate idea to idea; they lay out the **ordered shape of a work** you're building from your notes: Book → Part → Chapter → Scene, or any Map-of-Content outline. The **Structure** panel is where you read that shape.

The one question Structure answers is: **"Where does this note sit in the whole work?"** — *not* "how does this idea relate to that one." That second question belongs to the Backlinks and Outgoing Links panels, and Structure stays out of their way.

---

## Why structural links are kept separate from your thinking

A structural placement is **authorship, not a claim to be judged**. Putting a scene under a chapter, or a chapter under a book, is a decision about the *shape of your manuscript* — it is not evidence, not an argument, not something that can be contradicted or grow more certain over time.

So structural links are deliberately invisible to every thinking, maturity, and connection measure:

- They do **not** count as connections in a note's backlinks or outgoing links.
- They do **not** raise a note's maturity.
- They do **not** appear in Sky View or the graph.

A table of contents shouldn't make a note look more "connected" than it is. Your thinking links and your manuscript's outline are two separate things, and Constellation keeps them that way.

---

## The two kinds — you only ever type one side

You declare structure from whichever end is convenient, and Constellation figures out the reverse for you. You never have to maintain both ends.

| Property | What it means |
|---|---|
| **`parent`** | *This note's* place under one parent. (A chapter says which part it belongs to.) |
| **`contains`** | *This note's* ordered list of children. (A book lists its parts, in reading order.) |

Declaring a child's `parent` and listing it in a `contains` list are two ways to say the same thing. Use whichever fits how you think — top-down (a book that *contains* its parts) or bottom-up (a chapter that names its *parent*).

---

## Authoring a structural link — step by step

You author structure in a note's **Properties** — the Properties tab in the right sidebar, or the properties block at the top of the note.

1. Click **+ Add property**.
2. For the key, type **`parent`** or **`contains`**.
3. In the value, type the **target note's name** — just the name, for example `Part I - The Cartographer`. **You do not type the square brackets.** Constellation wraps the name into a `[[link]]` for you automatically. (If you paste a name that already has brackets, it's cleaned up to a single `[[name]]` — never a double `[[[ ]]]`.)
4. For **`contains`**, add each child as its own chip — type a name, press Enter, type the next. **The order you add them in is the reading order** of the outline.

> **They rename safely.** Rename a chapter and its place in the structure follows automatically — the link points at the note itself, not at a frozen piece of text. You never have to hunt down and fix an outline after renaming.

---

## Reading the Structure panel

Open the **Structure** tab in the right sidebar — just after the Backlinks tab.

- **The outline.** Headed **OUTLINE** with a count, the panel shows the **whole work** as a teal-bulleted, indented tree — every descendant of the work, in order — not just the open note's own children. So even when you're standing on a single scene, you see the entire book around it.
- **"You are here."** The note you're currently viewing is **highlighted** inside the outline, so you always know where you stand.
- **The breadcrumb.** Along the top, a teal breadcrumb shows the path up the spine — for example *The Atlas of Lost Places › Part I › Chapter 1*. Click any crumb (or any row in the outline) to jump straight to that note.
- **Whole work ⇄ This note.** A toggle at the top-right switches between the entire work and just the open note's own branch. It appears only when the note has a parent (otherwise the two views would be identical).

> **A loop never hangs it.** If the structure accidentally circles back on itself — note A's parent is B, and B's parent is A — the outline draws the chain and then stops cleanly, marking the cut point with a small **↻**. Hover it for a one-line explanation.

---

## When two notes claim the same child — "Contested"

Structure is meant to be a clean tree, so a child should have exactly one parent. If two notes both claim the same child — one through the child's own **`parent`**, the other through its **`contains`** list — Constellation does **not** silently pick one and drop the other. Instead, that row is flagged **Contested** with an amber **⚠** badge naming the other claimant, so you can see the conflict and decide.

Two one-click buttons resolve it:

- **Keep** — keep the child's own declared parent. (This note releases its claim on the child.)
- **Move here** — accept this note as the parent. (The child's `parent` switches to this note.)

Either choice updates the note files directly and refreshes the outline. **Nothing is ever changed without your click** — Constellation flags the conflict and waits for your decision.

---

## Good to know

- **Local and private.** The outline is read from your own notes on demand; nothing is sent anywhere.
- **Fast on big works.** Long outlines (past about 50 rows) get their own scrollbar and render only the rows on screen, so a large manuscript opens and scrolls smoothly.
- **It speaks your language.** The panel's labels, the breadcrumb, and the resolve buttons all appear in your chosen interface language and mirror correctly for right-to-left languages. The `parent` / `contains` property *keys* stay in canonical English in the file (so the structure reads the same in every language), while their on-screen pill labels are localized.
