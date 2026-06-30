---
aliases:
  - Cognitive colours
  - Cognitive colors
  - Property tags styling
  - Taxonomy pills styling
  - Maturity colours
  - Confidence colours
  - Origin colours
  - Stage colours
  - Match category colours
  - Right-click menu
  - Context menu
  - Note body right-click
  - Property right-click
  - Search result right-click
  - Unify on demand
description: Restyle the frontmatter Property tags and Taxonomy pills, set one shared colour for every cognitive state (Maturity, Confidence, Origin, Stage, Match category) so all surfaces unify on demand, and use the app-wide right-click menus on the note body, the Properties panel, and search results.
---

# Cognitive Colours and Right-Click Menus

This topic covers two things that arrived together: **two new Style Setter categories** — **Properties** (restyle the little tags in your frontmatter) and **Cognitive colours** (one colour control per cognitive state, shared across the whole app) — and the **app-wide right-click menus** that put the right actions one click away on the note body, on a frontmatter property, and on a search result.

> The Style Setter is the full-page design studio you open from **Settings → Appearance → "✦ Open Style Setter,"** or from its own **Style Setter** tab in the Settings sidebar. The two categories below sit in the left-hand list of *Surfaces* alongside Interface, Editor, Links, and the rest. For the Setter's general behaviour — Inspect, Keep / Discard / Reset, saved Styles — see [[Appearance and Themes]].

---

## Style Setter → Properties

The **Properties** category restyles the small tags that appear inside a note's **frontmatter** (its YAML properties block) — the chips you see for `tags`, `aliases`, and the like in the Properties panel and at the top of the note. Until now these were fixed; now they are yours to shape.

Open **Style Setter** and click **Properties** in the left list. The centre shows a live preview of the property pills; click a control on the right and the preview restyles as you edit. Two elements:

### Property tags

The ordinary frontmatter tag chips (for example, each value in a `tags` list). Four controls:

- **Tag background** — the chip's fill colour.
- **Tag text** — the colour of the text inside the chip.
- **Tag radius** — how rounded the chip's corners are (0 px = square, up to 20 px = fully rounded).
- **Height** — the chip's height in pixels (14–32 px).

### Taxonomy pills

The pills used for taxonomy-style values. Three controls:

- **Background** — the pill's fill colour.
- **Text** — the text colour inside the pill.
- **Radius** — corner rounding (0–20 px).

> **Nothing changes until you touch a control.** Every value starts at exactly the look you have today, so the Properties category leaves your notes looking identical until you deliberately pick a colour or drag a slider. Click **Keep** to save the look for this Universe.

---

## Style Setter → Cognitive colours

Constellation paints your **cognitive vocabulary** in colour — a note's *maturity*, a link's *confidence*, where an idea *came from*, what *stage* of life it's in, and *why* a search result matched. The trouble was that each of those colours was decided separately on each surface: a "wilting" note could be one green in the file tree and a different green in Sky View. The **Cognitive colours** category gives you **one colour control per state**, and everything that shows that state follows it.

Open **Style Setter** and click **Cognitive colours** in the left list. The centre shows a colour legend for whichever set you're editing; pick a control on the right and the legend updates live. There are five sets.

### Maturity — how settled an idea is

Five states, youngest to most settled: **Seed**, **Sapling**, **Evergreen**, **Canonical**, **Wilting**. Each gets one colour, used by the file-tree note dots, the tab maturity marker, and the note inspector.

### Confidence — how certain a link is

Four states: **Hypothesis**, **Evidence**, **Established**, **Contested**. One colour each.

### Origin — where an idea came from

Four states: **Received** (taken from a source), **Discovered** (your own), **Mixed**, and **None**. One colour each.

### Stage — where a note sits in its life

Six states, in order: **Spark**, **Birth**, **Growth**, **Maturity**, **Dormancy**, **Archival**. One colour each.

### Match category — why a search result matched

Seven kinds of match: **Title**, **Content**, **Tag**, **Wikilink**, **Property**, **Semantic** (a meaning-based match, not an exact word), and **Structured** (a property-query match). The colour you set here is shared by the in-editor search highlight, the match badge, and the highlight on the result row in the search panel.

### "Unify on demand" — the rule that makes this safe

Cognitive colours follow a deliberate rule: **nothing changes until you pick a colour.** Every surface keeps the colour it has today as its own fallback. The moment you set a state's colour here, **every** surface that shows that state snaps to your colour at once — file tree, tabs, the inspector, search highlights, and so on. Set "Evergreen" once, and every Evergreen marker across the app agrees. Leave a state untouched and it looks exactly as it did before.

This is why the category can ship without altering a single existing look: it unifies *on demand*, never by default. Click **Keep** to save your colours for this Universe.

---

## Right-click menus across the app

Constellation now gives you a full right-click (context) menu in the three places you most often want one: the **note body**, a **frontmatter property**, and a **search result**. Each menu only offers actions that make sense where you clicked.

### Right-click the note body

Right-click anywhere in the text of a note to get the editing menu:

- **Add link** / **Add external link** — wrap the selection (or insert at the cursor) as a `[[wikilink]]` or a `[text](url)` link.
- **Format ▸** — a fly-out submenu: Bold, Italic, Underline, Strikethrough, Highlight, Inline code, Math, Toggle comment, Superscript, Subscript, Clear formatting.
- **Paragraph ▸** — a fly-out: Bullet list, Numbered list, Task list, the heading levels **H1–H6** and **Body**, and Blockquote.
- **Insert ▸** — a fly-out: Footnote, Table, Callout, Horizontal rule, Code block, Math block, Image.
- **Clipboard** — Cut, Copy, Paste, Paste as plain text, Select all.
- **Style…** — jumps straight into the **Style Setter** focused on the **Editor** category, so you can restyle the very thing you right-clicked.

### Right-click a frontmatter property

Right-click a property **row** in the Properties panel (or in the properties block at the top of the note) and you get property actions on top of the full editing menu:

- **Copy value** — copies the property's value to the clipboard.
- **Copy name** — copies the property's key.
- **Remove property** — deletes that property row.
- **Add property** — adds a new, empty property row.
- …followed by the same **Format / Paragraph / Insert / clipboard** items as the note body, and a **Style…** item that opens the Style Setter focused on the **Properties** category — so "Style…" on a property tag styles property tags, not the note body.

### Right-click a search result

Right-click a result in the search panel for a **safe** set of note actions — the ones that never put your files at risk:

- **Open** — open the note.
- **Open in new tab** — open it alongside what you have.
- **Reveal in tree** — highlight the note in the file tree so you can see where it lives.
- **Copy link** / **Copy path** — copy a wikilink to the note, or its file path.
- **Bookmark** — add the note to your bookmarks.
- **Show in explorer** — reveal the file in your operating system's file manager.
- **Open in default app** — open the file in whatever app your system uses for Markdown.
- **Style…** — open the Style Setter focused on the **Cognitive colours** category (where the search match colours live).

> **By design, the search-result menu has no Rename, Move, or Delete.** A search panel shows results from across your whole Universe and does not keep its own up-to-the-second copy of the file tree, so a destructive action there could act on a stale view. Constellation keeps those operations in the file tree (and the Notes Navigator), where the view is always current. The search menu is for *getting to* a note safely, not for restructuring your library.

---

## Good to know

- **Local and private.** All of this is computed from your own notes and settings on your device. Nothing is sent anywhere.
- **It speaks your language.** Every menu item, every category name, every state label appears in your chosen interface language and mirrors correctly for right-to-left languages. The cognitive-state colours themselves are universal — a colour means the same state in every language.
- **Style… always lands on the right surface.** Each "Style…" entry opens the Style Setter focused on the category for the thing you right-clicked: the note body → **Editor**, a property → **Properties**, a search result → **Cognitive colours**. You never have to hunt for the right controls.

---

## Related

- [[Appearance and Themes]] — the Style Setter's general behaviour, themes, fonts, and saved Styles
- [[Properties]] — viewing and editing the frontmatter properties whose tags you restyle here
- [[Search]] — the search panel whose results carry the right-click menu
- [[Cognitive Engine]] — what Maturity, Confidence, Origin, and Stage mean as knowledge measures
- [[Knowledge Formulation]] — the living-link confidence tiers the Confidence colours represent
