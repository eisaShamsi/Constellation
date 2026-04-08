# Plan: Second Screen Redesign — Interactive Note Viewer

## Goal
Simplify the second screen to be a **reactive note viewer** that responds to main window actions. Remove mode buttons, add a right sidebar, and bold the active index link.

## Changes

### 1. SecondScreenPage.svelte — Remove mode buttons, simplify to note viewer
- **Remove** the Grid/Star/Detail mode switcher buttons from the toolbar
- **Remove** Grid view (`NoteGrid` component) and Star view (`FullStarView` component)
- **Remove** keyboard shortcuts for G/E/D mode switching
- **Remove** the "linked browsing" toggle — it's always linked now
- Keep: tab bar for multiple open notes, NotePane for reading/editing, close button
- The second screen is now always in "detail" mode, receiving notes from the main window

### 2. SecondScreenPage.svelte — Add interactive right sidebar
- Add a collapsible right sidebar panel with:
  - **Backlinks** section: notes that link to the current note
  - **Forward links** section: notes linked from the current note
  - **Tags** section: tags found in the current note
- Toggle button to show/hide the sidebar
- Clicking a backlink/forward-link opens that note in the second screen itself (new tab)

### 3. IndexPanel.svelte — Bold the active note link
- Add `activeNotePath?: string` optional prop
- When a `.gp-ref` button's `mention.note_path === activeNotePath`, apply `.gp-ref.active` class
- CSS: `.gp-ref.active { font-weight: 700; }`

### 4. +layout.svelte — Track active index note
- Add `indexActiveNotePath` state variable
- In `handleIndexNoteClick`: set `indexActiveNotePath = filePath` when sending to second screen or opening in index split pane
- Pass `activeNotePath={indexActiveNotePath}` to `<IndexPanel>`
- Clear when second screen closes or index note pane closes

## Files to modify
1. `src/lib/components/SecondScreenPage.svelte` — major simplification + sidebar
2. `src/lib/components/IndexPanel.svelte` — add `activeNotePath` prop + bold styling
3. `src/routes/+layout.svelte` — track and pass `indexActiveNotePath`
