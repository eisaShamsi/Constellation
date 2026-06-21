# Right-Click (Context-Menu) Reference — Obsidian targets (Boss-supplied 2026-06-21)

> **Status: REFERENCE for LATER.** Boss supplied these Obsidian context menus as the design target for Constellation's right-click work. They are NOT to be built now (MIG-080 is being finished first, then the bring-up §D). They feed:
> - **`00-MASTER` §7 Debt Register B** (right-click gaps — MIG-077 incomplete; hand-rolled menus to fold into the shared `<ContextMenu>`; many surfaces missing a menu).
> - **Every per-function concept paper's §5 (Right-click / context menu)** — the acceptance gate "right-click menu present + correct (shared `<ContextMenu>`)".
>
> **Adaptation rule:** these are Obsidian's menus. Constellation adapts, it does not copy blindly — **"vault" → "Library"** (terminology rule), Obsidian-only affordances (Obsidian URL, canvas, bases, presentation, version history, web viewer) are kept ONLY where Constellation has the equivalent, and items are localized ×15 + routed through the shared `buildContextMenu()`/`<ContextMenu>`. Faithfully transcribed from Boss's screenshots; the real per-surface design is the bring-up's job.

---

## 1. Note (file) right-click — *image 1*
- Open in new tab
- Open to the right
- Open in new window
- — — —
- Make a copy
- Move file to…
- Bookmark…
- Merge entire file with…
- — — —
- **Copy path ▸** : as Obsidian URL · from vault folder · from system root  *(Constellation: "from Library folder" / "from system root"; "Obsidian URL" → a Constellation deep-link if/when one exists)*
- — — —
- Start presentation
- Open version history
- — — —
- Open in default app
- Show in system explorer
- — — —
- Rename…
- Delete

## 2. Folder right-click — *image 2*
- New note
- New folder
- New canvas
- New base
- — — —
- Make a copy
- Move folder to…
- Search in folder
- Bookmark…
- — — —
- **Copy path ▸** : from vault folder · from system root  *(→ "from Library folder")*
- — — —
- Show in system explorer
- — — —
- Rename…
- Delete

## 3. Link / editor-selection right-click (on a note's content) — *image 3*
- Open in new tab
- Open to the right
- Open in new window
- — — —
- Add link
- Add external link
- — — —
- Edit link
- **Paragraph ▸** (see §4)
- **Insert ▸** (see §4)
- — — —
- Cut
- Copy
- Paste
- Paste as plain text
- Select all
- — — —
- **Copy path ▸**
- — — —
- Rename…
- Move file to…
- Bookmark…
- — — —
- Start presentation
- — — —
- Open in default app
- Show in system explorer
- Reveal file in navigation
- — — —
- New drawing

## 4. Editor empty-area right-click (formatting) — *images 4–6*
- Add link
- Add external link
- — — —
- **Format ▸** : Bold · Italic · Strikethrough · Highlight · — · Code · Math · Comment · — · Clear formatting
- **Paragraph ▸** : Bullet list · Numbered list · Task list · — · Heading 1 · Heading 2 · Heading 3 · Heading 4 · Heading 5 · Heading 6 · Body (✓ current) · — · Quote
- **Insert ▸** : Footnote · Table · Callout · Horizontal rule · — · Code block · Math block · New base
- — — —
- Cut
- Copy
- Paste
- Paste as plain text
- Select all

---

## Cross-check vs. Constellation today (for the bring-up)
- The **editor formatting menu (§4)** maps to the **Note-Editor concept paper (`01-Note-Editor.md`) §5** + the editor's CM6 extensions; Constellation already has Bold/Italic/Highlight/Code/callouts/tables — the gap is the *right-click surface* exposing them.
- The **file/folder menus (§1/§2)** map to the **File-Tree paper (`02-file-tree.md`) §5** — Constellation has MIG-077's partial right-click; Debt B lists File-Tree's menu as needing the shared `<ContextMenu>` audit.
- The **link menu (§3)** maps to the editor + the Living-Link surfaces.
- **All items localize ×15** (multilingual-by-default) and route through the shared `buildContextMenu()` — the two cross-cutting requirements of every concept paper.
