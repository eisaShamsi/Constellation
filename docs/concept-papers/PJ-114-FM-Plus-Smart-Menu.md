# FM+ Smart Menu — the "Menu of Knowledge" for Focus mode

*PJ-114 · supersedes the rejected v1 clipboard-menu · dig `wf_b5c67f60-646` (PKM + writing-app research + Constellation engine inventory) · 2026-07-18 · awaiting Boss picks*

> **Why v1 failed:** Copy-link-text / Copy-path / Cut-Copy-Paste were clipboard verbs the **native** webview menu already does better. The FM+ menu carried zero knowledge value, so it was a pure downgrade.
>
> **The fix (concept):** the native menu keeps the **clipboard**; FM+ owns **knowledge formulation.** Every FM+ item is a superpower only Constellation has — make a link, *type* the relationship, extract a note, follow/annotate a connection. FM+ = weave knowledge while you write, without leaving the calm plain-text paper. Every action delegates to an existing Constellation engine (verified in the dig); none touches the markdown parser.

The menu is **never one long list** — it shows only the verbs whose *object* is under your pointer, so each context is short and everything in it is relevant.

---

## 🔗 On a `[[link]]` — the object is the connection

- **↪ Open in Focus (Follow)** — walk the link; target opens in Focus, link earns weight. **[ready]** `resolveWikilinkCrossLibrary` → `openNoteTab`.
- **🎓 Type this link ▸** — assign one of the 8 kinds (supports / contradicts / causes / exemplifies / generalizes / derives-from / part-of / supersedes); rewrites `[[Target]]` → `[[supports::Target]]`. *The crown jewel — no editor names a **relationship**.* **[ready]** `cognitiveLinkTypes()` / `LinkTypePicker`.
- **✍ Annotate this link…** — add *why* they relate: `[[supports::Target|because the 1971 data confirms it]]`. **[ready]** pure text edit (`parse_link_body`).
- **◐ Set confidence** — hypothesis → evidence → established → contested. **[ready]** `setLinkConfidence` / `ConfidencePicker`.
- **✎ Rename target… (cascade)** — rename the linked note + heal every `[[link]]` universe-wide. **[ready]** `handleRenameComplete`.
- **⤳ Who points here? (target's backlinks)** — bounded pop-list of notes linking to the target. **[some work]** `get_backlink_rows`.
- **⌦ Unwrap link** — strip `[[…]]`, keep the words. **[ready]** one-line regex.

## ✂ On a selection — the object is raw text becoming structure

- **🔗 Link this selection** — wrap as `[[selection]]`; resolves to a note or creates it. *Fastest concept → node.* **[ready]** text wrap + resolver.
- **🌱 Extract to new linked note** — the selection becomes a new note's body; a `[[link]]` to it replaces it. *The signature Zettelkasten move.* **[some work]** `createNote` + text-replace (~10 lines glue).
- **🎓 Type a link to…** — wrap + pick kind + pick target note in one gesture. **[some work]**
- **🔎 Search the Universe for this** — send selection to search. **[ready]** `constellationSearch` (FTS5).
- **#️⃣ Tag the selection** — insert `#tag` with suggestions. **[ready]** `allLibraryTags`.

## 📄 On plain text / at the cursor — the object is this note

- **✎ Rename this note… (cascade)** — the only in-Focus rename route (tree is hidden). **[ready]** `handleRenameComplete`.
- **➕ Insert a link…** — the `[[` autocomplete picker at the cursor. **[ready]** `completions.ts`.
- **⤴ Who points here? (this note's backlinks)** — bounded pop-list. **[some work]** `get_backlink_rows`.
- **📅 New / open daily note** — jump to today. **[ready]** `getDailyNotePath` (only if dailies are in use).

## 🚩 Out of scope for now (would need a NEW engine)
- **Peek/preview a link inline** — needs a rendering surface = the NotePane trap. **[new engine]**
- **AI suggest-connections** — no existing match surface. **[new engine]**
- **Set link weight manually** — weight is *earned only* (traversal + decay); a setter contradicts the model. **Exclude by design.**

---

## Recommended default set (ship first — all [ready] or thin glue)

| # | Action | Context | Five-Acts move |
|---|---|---|---|
| 1 | 🔗 Link this selection | selection | Connection |
| 2 | 🌱 Extract to new linked note | selection | Synthesis |
| 3 | 🎓 Type this link (8 kinds) | link | the differentiator |
| 4 | ↪ Open in Focus (Follow) | link | traverse + earn weight |
| 5 | ✍ Annotate · ◐ Set confidence | link | Tension → Conviction |
| 6 | ✎ Rename (cascade) | link & note | keeps the graph healthy |

**Open decisions (Boss):** (1) include *Extract to new linked note* (the one item with ~10 lines new glue)? — rec **yes**. (2) *Type this link* — one-click kind, with Annotate/Confidence as separate items, vs one bundled popover? — rec **one-click + separate**. (3) *Peek vs Follow* — rec **Follow only** (Peek is a new engine). (4) *Backlinks in Focus* — bounded hand-off list, or leave to NotePane? — rec **bounded list**, never a docked panel.
