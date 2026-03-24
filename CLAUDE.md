# Constellation — Claude Instructions

## Project
Tauri v2 desktop app (Rust + SvelteKit/Svelte 5) for managing Markdown note libraries.

## Before Starting Work
1. Always `git pull origin main` first to sync changes from other devices/sessions.
2. Check `git log --oneline -5` to understand recent work.

## Conventions
- **Terminology**: Use "Library" everywhere, never "vault" (except for Obsidian import compatibility).
- **Svelte 5 runes**: Use `$state`, `$derived`, `$derived.by`, `$effect`, `$props` — no legacy Svelte 4 patterns.
- **i18n**: All user-facing strings go through `$t()`. Update all 15 locale files (ar, de, en, es, fa, fr, he, hi, ja, ko, pt, ru, tr, ur, zh).
- **RTL support**: Use `dir` attributes, `detectDir()` from `$lib/utils`. Flip chevrons/arrows in RTL.
- **Cross-window sync**: Second screen is a separate Tauri window. Use `emit`/`listen` from `@tauri-apps/api/event` for communication. Settings changes must call `notifySettingsChanged()` to propagate.
- **CSS**: NotePane uses `.pane` (not `.note-pane`). Override child styles with `:global()` + `!important` when needed.
- **Fonts**: Global fonts from `appSettings` (interfaceFont, textFont, monoFont, fontSize, scriptFonts). Per-library fonts from `libraryAppearances`. Both must be applied in main window AND second screen.

## Editor Parity Rule
- **All note views must have identical markdown rendering.** Standard NotePane, Focus mode (PaperOnDesk), and any future note types must share the same CM6 extensions (livePreview, callouts, syntax highlighting, etc.).
- New markdown features added to the editor MUST work in ALL note modes — never add a feature to only one view.
- The shared extension set lives in `$lib/editor/` and is imported by every editor instance.

## Don't
- Don't use preview/screenshot tools unless essential.
- Don't add unnecessary abstractions or over-engineer.
- Don't use "vault" terminology in new code.
