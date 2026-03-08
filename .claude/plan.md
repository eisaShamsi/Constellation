# Constellation — AI Integration, Skills System & i18n Plan

## Overview

Three major features to add to the Constellation scaffold:

1. **AI Integration** — Multi-provider AI engine (OpenAI, Claude, Gemini, Ollama)
2. **AI Skills System** — Plugin-like AI workflows users can install/create
3. **Internationalization (i18n)** — Arabic (RTL) + English (LTR) with language switcher

---

## 1. AI Integration Architecture

### Provider Abstraction Layer

Create a unified AI provider interface so the app works with any provider:

```
src/lib/ai/
├── providers/
│   ├── openai.ts        — OpenAI/ChatGPT adapter
│   ├── anthropic.ts     — Claude/Anthropic adapter
│   ├── gemini.ts        — Google Gemini adapter
│   └── ollama.ts        — Local Ollama adapter
├── provider.ts          — Common provider interface/types
├── engine.ts            — AI engine that routes to active provider
└── store.ts             — Svelte store for AI state & settings
```

**Provider Interface** (`provider.ts`):
- `sendMessage(prompt, options)` → response
- `streamMessage(prompt, options)` → async iterable
- `listModels()` → available models
- `validateKey()` → test connection

**Rust Backend** (`src-tauri/src/ai/`):
- API calls go through Rust (not browser fetch) for security
- API keys stored encrypted in local config via Tauri's secure storage
- No keys ever touch the frontend directly

### AI Features (powered by provider layer)

| Feature | How It Works |
|---|---|
| Note summarization | Send note content → AI returns summary |
| Smart Q&A | RAG-like: search vault index → send relevant chunks + question → AI answers |
| Writing assistance | Send selected text + instruction → AI returns edit |
| Auto-linking | Send note → AI extracts topics → match against vault index → suggest links |
| Infographic/charts | Send notes → AI generates structured data → render with chart library |

### Settings UI

New settings page where users:
- Choose their AI provider from a dropdown
- Enter their API key (stored encrypted locally)
- Select model (e.g., gpt-4o, claude-sonnet, gemini-pro)
- Test connection with a "Verify" button

---

## 2. AI Skills System

### What is a Skill?

A Skill is a packaged AI workflow — a combination of:
- A prompt template (with variables like `{{note_content}}`, `{{vault_name}}`)
- Configuration (which provider features it needs)
- UI definition (what inputs to show the user)
- Output format (text, markdown, JSON for charts, etc.)

### Skill Definition Format

Skills are defined as JSON/YAML files:

```
skills/
├── builtin/
│   ├── summarize.json
│   ├── translate.json
│   ├── meeting-notes.json
│   ├── research-assistant.json
│   └── chart-generator.json
└── community/          — user-installed skills
```

### Skill Structure:

```json
{
  "id": "summarize",
  "name": "Summarize Note",
  "name_ar": "تلخيص الملاحظة",
  "description": "Generate a concise summary of any note",
  "icon": "sparkles",
  "inputs": [
    { "type": "note-select", "label": "Note to summarize" }
  ],
  "prompt": "Summarize the following note concisely:\n\n{{note_content}}",
  "output": "markdown"
}
```

### Built-in Skills (ship with app):

1. **Summarize Note** — Condense any note
2. **Smart Q&A** — Ask questions about your vaults
3. **Writing Assistant** — Expand, rewrite, improve text
4. **Auto-Linker** — Suggest cross-vault connections
5. **Translate Note** — Translate between languages
6. **Meeting Notes** — Structure raw meeting notes
7. **Chart Generator** — Create charts from note data
8. **Research Assistant** — Analyze and synthesize across notes

### Skill Manager UI

- Browse installed skills
- Enable/disable skills
- Install from file (drag & drop a .json skill file)
- Create custom skills (simple form-based editor)

---

## 3. Internationalization (i18n)

### Approach: Svelte i18n with RTL Support

```
src/lib/i18n/
├── index.ts          — i18n setup, locale store, helper functions
├── en.json           — English translations
└── ar.json           — Arabic translations
```

### How RTL Works:

- `<html dir="rtl" lang="ar">` when Arabic is selected
- `<html dir="ltr" lang="en">` when English is selected
- CSS uses logical properties: `margin-inline-start` instead of `margin-left`
- Layout automatically mirrors for RTL

### Language Switcher:

- Toggle in the app header/settings
- Persisted in local storage
- App restarts in chosen language

### Translation Coverage:

- All UI labels, buttons, menus
- Skill names and descriptions (dual `name` / `name_ar` fields)
- Error messages
- Settings page

---

## Implementation Steps

### Step 1: i18n Foundation
- Create `src/lib/i18n/` with locale store and translation files
- Add `en.json` and `ar.json` with initial UI strings
- Update `app.html` to support dynamic `dir` and `lang` attributes
- Add language switcher component
- Convert all existing text to use translation keys

### Step 2: AI Provider Layer (Frontend)
- Create `src/lib/ai/provider.ts` — TypeScript interfaces
- Create `src/lib/ai/engine.ts` — Provider routing engine
- Create `src/lib/ai/store.ts` — Svelte store for AI state
- Create provider adapters (OpenAI, Anthropic, Gemini, Ollama)

### Step 3: AI Provider Layer (Rust Backend)
- Create `src-tauri/src/ai/` module
- Add Tauri commands: `ai_send_message`, `ai_validate_key`, `ai_list_models`
- Add `reqwest` crate for HTTP calls to AI APIs
- Add `keyring` or Tauri's secure storage for API key encryption
- Register new commands in `lib.rs`

### Step 4: Settings Page
- Create `/settings` route with AI provider configuration
- Language preference selector
- API key input with validation
- Model selector per provider

### Step 5: Skills System
- Create skill definition types in `src/lib/skills/`
- Add built-in skill JSON files
- Create skill runner that combines skill template + AI provider
- Create skill browser/manager UI
- Add skill execution panel (input → AI → output)

### Step 6: Update Welcome Page
- Replace default page with Constellation-branded landing
- Show connected provider status
- Quick-access to skills
- Language-aware layout

### Step 7: Update README & Push
- Document AI features, skills system, i18n in README
- Commit and push all changes

---

## New Dependencies

### Frontend (npm)
- None required for i18n (custom lightweight solution)
- Chart rendering library (later, for infographic skill)

### Backend (Cargo.toml)
- `reqwest` — HTTP client for AI API calls
- `tokio` — Async runtime (required by reqwest)
- `serde` — Already included
- `tauri-plugin-store` — Persistent local storage for settings/keys

### Tauri Capabilities Needed
- `store:default` — For persistent settings
- `core:default` — Already included
- `http:default` — For AI API HTTP calls (via Rust)
