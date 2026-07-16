# Writing in Arabic and Mixed Scripts

Constellation's editor is built language-first: Arabic, Hebrew, Persian, Urdu, and bilingual notes are not an add-on — the caret, the selection, and the direction of every paragraph follow the same rules Microsoft Word uses on Windows, so your muscle memory carries over. This topic covers everything about *writing* in right-to-left and mixed text: how the caret moves, how to select by word, sentence, line, paragraph, or page, and how to force a paragraph's direction when the automatic detection isn't what you want.

(For how Constellation *understands* Arabic — roots, search, and the morphological engine — see the **Arabic Engine** topic.)

---

## How the caret moves

- **Arrow keys move by one character of the text, in reading order** — never by one position on the screen. In pure Arabic or pure English this looks exactly like the arrow you pressed. At a seam between Arabic and English (an Arabic sentence containing an English word, say), the caret steps through each character in writing order and visibly "hops" across the seam — that hop is correct; it is what stops the caret from feeling stuck at the boundary.
- **Home** goes to the reading **start** of the line — the *right* edge of an Arabic line. **End** goes to the reading **end** — the *left* edge. Hold **Shift** with either to select to that edge.
- **Enter** on an Arabic line puts the caret of the new line on the **right** — the natural writing position.
- A **Latin word at the end of an Arabic line** keeps a clear, stable caret instead of losing its direction.

Every rule above works identically in the standard editor, in Focus mode, and in the conflict-merge view.

---

## Selecting by unit

Every unit of text has a fast selector, in any language and any mix:

| Unit | How |
|---|---|
| **Word** | Double-click it |
| **Sentence** | **Ctrl+click** anywhere in it — or press **Ctrl+Shift+S** with the caret inside it |
| **Line** | **Ctrl+L** |
| **Paragraph** | **Ctrl+Shift+L** — or triple-click it |
| **Screenful** | **Shift+Page Down** / **Shift+Page Up** |
| **Everything** | **Ctrl+A** |

Details worth knowing:

- **Sentence selection understands Arabic punctuation.** It ends a sentence at **؟ ۔ !** and the full stop — but the Arabic semicolon **؛** is a pause *inside* a sentence, so selection correctly runs past it. Decimal numbers like 3.14 never split a sentence.
- A **paragraph** is a block of text with an empty line above and below it — exactly like Word. Line and paragraph selections hug the text: on an Arabic line the highlight stops at the words instead of stretching across the empty left side.
- Ctrl+click *replaces* the old "add another cursor" gesture on that key — sentence selection is what the click does now.

## Moving by paragraph

- **Ctrl+↓** jumps to the start of the **next** paragraph; **Ctrl+↑** to the start of the **current** one (press again for the previous one). Add **Shift** to select paragraph-by-paragraph as you jump. This is the Word convention, and "next" simply means further down the page — it works identically in Arabic, English, and mixed notes.

---

## Forcing a paragraph's direction

Constellation detects each line's direction automatically from its first letters. Usually that is exactly right — but sometimes you want to overrule it: an Arabic paragraph that opens with an English brand name, or a mostly-English paragraph you want to read right-to-left.

**Press and release Ctrl+Shift on the RIGHT side of your keyboard** → the paragraph your cursor is in becomes **100% right-to-left**.
**Press and release Ctrl+Shift on the LEFT side** → **100% left-to-right**.

This is the Microsoft Word convention. Things to know:

- **It fires on release** — press the two keys together, let go, and don't press anything else in between. That is why Ctrl+Shift+S, Ctrl+Shift+L, and every other shortcut keep working normally: the moment a third key joins, the direction switch stands down.
- **It is a hard override** — it wins over the automatic detection, and it applies to the whole paragraph (or every paragraph a selection touches).
- **It is saved inside the text itself** as an invisible direction character, so it survives closing the note, restarting the app, and syncing — and it even travels with the text if you paste it into Word or Obsidian.
- **One Ctrl+Z undoes it.** Pressing the same side twice does nothing extra.
- **Markdown stays safe.** Lists stay lists, headings stay headings, quotes stay quotes. Code blocks, tables, and horizontal rules are deliberately left untouched. A line that *begins* with a #tag keeps its automatic direction (a forced mark there would break the tag) — the rest of the paragraph still flips.

---

## Fonts and the interface

- **Script fonts**: configure Arabic, Hebrew, and CJK fonts independently in **Settings → Language**.
- **Script toolbars**: language-specific symbol and punctuation buttons.
- **Tashkeel highlighting**: toggle Arabic diacritics highlighting from the editor toolbar.
- Selecting Arabic or Hebrew as the interface language flips the whole app RTL.

---

## Glossary

- **Reading order** — the order the characters are written and read in, regardless of where they sit on screen.
- **Seam** — the boundary between a right-to-left run and a left-to-right run on the same line.
- **Hard override** — an explicit direction you set, which beats the automatic first-letter detection.
- **Direction mark** — the invisible character (RLM/LRM) that stores your override inside the text itself.
