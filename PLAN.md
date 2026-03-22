# Plan: Migrate from TipTap to CM6

## Current State
- `NotePane.svelte` switches between `TipTapEditor` (document mode) and `CodeMirrorEditor` (markdown mode) based on `$appSettings.editorType`
- `CodeMirrorEditor.svelte` is already a full-featured CM6 editor with: toolbars, context menus, live preview, RTL, autocomplete, wikilinks, smart lists, slash commands, table editing, fold, indent guides
- All CM6 packages are already installed
- TipTap adds: font family picker, color picker, subscript/superscript, callout insertion, find & replace — these need to be ported to CM6

## Migration Steps

### Step 1: Port missing features from TipTap to CodeMirrorEditor
Add to CodeMirrorEditor what TipTap has that CM6 doesn't:
- Font family dropdown (insert `<span style="font-family:...">` around selection)
- Font size control
- Text color picker (insert `<span style="color:...">`)
- Subscript/superscript buttons (insert `<sub>`/`<sup>` tags)
- Callout insertion (insert `> [!type]` blocks)
- Find & Replace bar (CM6 has `@codemirror/search` — just wire it up)

### Step 2: Make CM6 the default editor
- Change `DEFAULT_SETTINGS.editorType` from `'markdown'` to `'markdown'` (it already is!)
- Remove the `'document'` option from Settings
- Keep the switch button in breadcrumb but change its function: toggle between "Live Preview" and "Source" (both CM6 modes)

### Step 3: Remove TipTap
- Remove TipTapEditor.svelte
- Remove TipTap npm packages (@tiptap/*)
- Remove turndown dependency (no more HTML→MD conversion)
- Remove marked dependency from editor (still used elsewhere for preview)
- Clean up NotePane.svelte — remove the document/markdown branching

### Step 4: Update Settings
- Remove "Editor type" (Document/Markdown) setting from SettingsModal
- Remove "Default editing mode" and "Default view for new tabs" settings
- Keep editor behavior settings (line numbers, fold, indent, spellcheck, etc.)

### Step 5: Update help files and i18n
- Update User Manual and help files
- Clean up i18n keys related to removed settings
