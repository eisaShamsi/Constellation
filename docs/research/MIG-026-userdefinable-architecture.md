---
research_for: MIG-026 (register-set expansion + user-definable architecture)
research_date: 2026-05-16
agent: general-purpose (parallel agent 2 of 2)
agent_task_id: ab0749e212c27e2d5
status: Architecture survey — informs the user-definable approach choice for MIG-026
prompt_summary: Survey how mature notes/PKM/knowledge apps let users author their own taxonomies, schemas, or classification systems. Four candidate approaches for Constellation: declarative JSON, bounded DSL, sandboxed code (Wasm/QuickJS/Lua), TypeScript plugin. Mature precedent, eng cost, UX cost, security tradeoffs for each.
---

# Survey: User-Authored Taxonomies, Schemas, and Classification Systems in PKM / Notes / Knowledge Apps

Compiled for Constellation's "user-definable epistemic registers" architecture decision. Mid-2026 state. All claims either cite a source or are marked **unverified**.

---

## §1 Executive Summary

The mature-systems landscape clusters into **four authoring layers**, in increasing order of expressive power:

| Layer | What the user authors | Representative systems |
|---|---|---|
| **L1 — Labels only** | Tag/category names | Mem.ai (auto-tags), Reflect (manual links), classic Roam pre-attributes |
| **L2 — Labels + typed fields** | Name + field schema (text, number, date, select, relation) | Notion databases, Obsidian Properties + Bases, Capacities object types, Tana fields, AppFlowy databases, Roam attributes, DEVONthink custom metadata, Logseq page properties, TiddlyWiki tiddler fields |
| **L3 — Labels + fields + structural behavior** | Same as L2 + templates, views, queries, formulas, sub-type hierarchy | Notion (formulas + templates), Tana (supertags with views, AI fields, slash commands), Anytype (types + relations + layouts + templates), TheBrain (thought types with hierarchical typing), Capacities (templates + two-way relations) |
| **L4 — Labels + fields + behavior + arbitrary code** | Same as L3 + executable extensions | Obsidian community plugins, Logseq plugins, VS Code extensions, Figma plugins, TiddlyWiki plugins, Quarto Lua filters, Hammerspoon Spoons, Tana via Input API + MCP, Anytype via gRPC API |

**Mapping to Constellation's four candidate approaches:**

- **Approach 1 (Pure declarative JSON/YAML)** → mature precedent at L1/L2 (Notion, Capacities, Tana, Anytype core, Obsidian Bases `.base` files, AppFlowy)
- **Approach 2 (Bounded DSL)** → mature precedent in TiddlyWiki filter language and Notion formulas
- **Approach 3 (Sandboxed code: Wasm / QuickJS / Lua)** → mature precedent in Figma (QuickJS+Wasm), Logseq (iframe sandbox), Quarto (Lua), Neovim (Lua), Hammerspoon (Lua), Extism/moonrepo (Wasm), VS Code (process isolation)
- **Approach 4 (TypeScript plugin, full trust)** → mature precedent in Obsidian community plugins, TiddlyWiki JS modules

**The dominant hybrid pattern in real systems**: declarative schema for "structure" (types, fields, relations, templates) + plugin/code layer for "behavior" (custom views, integrations, computations). Obsidian Bases + community plugins, Tana supertags + Input API, Anytype types + gRPC API, and VS Code language config files + extensions all instantiate this hybrid.

---

## §2 Real Systems

### Obsidian (Properties + Bases + community plugins)

- **What users can author**: Free-form YAML frontmatter "Properties" on any markdown note; structured "Bases" (.base files) that filter/sort/group notes by their properties; community plugins (Dataview, MetaBind) for inline queries and interactive widgets.
- **Minimum viable interaction**: Add `tags: [foo]` to a note's YAML; create a `.base` file with a few filter clauses.
- **Storage format**: YAML frontmatter inside the markdown file. Bases are `.base` files in YAML, hand-editable in any text editor.
- **Limits**: There is no enforced central schema — properties drift unless the user maintains a "master schema" note by hand. Bases is the database layer (shipped in Obsidian 1.9.10) and is now a core feature, not a plugin.
- **Layer**: L2 declarative + L4 plugin extension. Strong "file-over-app" alignment.
- Sources: [Obsidian Help — Properties](https://help.obsidian.md/properties), [Obsidian Help — Introduction to Bases](https://help.obsidian.md/bases), [Obsidian Forum — Bases YAML in source mode](https://forum.obsidian.md/t/bases-open-in-source-mode-easy-access-to-base-yaml-file-view/103196).

### Anytype (Types + Relations + Custom Layouts)

- **What users can author**: Custom Object Types with name, layout, templates, and a chosen set of Properties (formerly Relations). Properties can be text, number, select, multi-select, date, file, object-link.
- **Minimum viable interaction**: Click "New Type", pick a layout, attach properties.
- **Storage format**: Internal Any-Block protocol (Protocol Buffers, with auto-generated JSON Schemas). Recent (2025) Markdown export now preserves types and properties as YAML frontmatter — this is the closest Anytype gets to a portable user-readable schema file.
- **2026 roadmap signal — "Collections 2.0"**: Anytype's blog explicitly poses "why must an object only have one type?" and is moving from category-based to relation-based organization. **Unverified whether this has shipped** — the Feb 2026 community update describes it as in-design.
- **Limits**: Types and properties live in Anytype's internal database. The user can't open a `.json` for a type definition and edit it the way they can edit a markdown file. Plugin extensibility is via gRPC API + AI Agents, not in-process modules.
- **Layer**: L3 declarative + L4 external-API extension.
- Sources: [Anytype Docs — Types](https://doc.anytype.io/anytype-docs/getting-started/types), [Anytype API — Create property](https://developers.anytype.io/docs/reference/2025-05-20/create-property/), [Big Community Update Feb 2026](https://blog.anytype.io/february-community-update-2026/).

### Tana (Supertags + Fields)

- **What users can author**: Supertags = a class with attached Fields (text, date, number, select, instance-link, AI). Each supertag has templates, views, commands, and a per-tag color.
- **Minimum viable interaction**: Type `#supertagName` on a node — create-on-the-fly. Open the wrench panel to add fields.
- **Storage format**: Cloud (Tana's hosted database); JSON schema for each supertag can be retrieved via the "Show API Schema" command on the tag. Workspace export is Markdown or JSON. Tana Local API enables CLI extraction via the community `supertag-cli`.
- **Limits**: Not local-first. The schema is portable as JSON, but the tags themselves live in Tana's cloud.
- **Layer**: L3 declarative + L4 via Input API and MCP. **No in-app code plugin layer.**
- Sources: [Tana Docs — Supertags](https://tana.inc/docs/supertags), [Tana Docs — Input API](https://tana.inc/docs/input-api).

### Notion (Databases + Property Types + Formulas + Templates)

- **What users can author**: Database with typed properties (text, number, select, multi-select, date, files, people, relation, rollup, **formula**), per-database templates, and now a formula language with seven data types and access to related-database properties.
- **Minimum viable interaction**: Create a database, click "+ Add property", pick a type.
- **Storage format**: Internal cloud database; exports to Markdown + CSV (which lose formula/relation richness).
- **Limits**: Not local-first; not file-over-app. Schema is a UI artifact, not a file the user can edit and version.
- **Layer**: L3 declarative; no user-side code plugin layer (only API integrations).
- Sources: [Notion Help — Database properties](https://www.notion.com/help/database-properties), [Notion — New Formulas](https://www.notion.com/help/guides/new-formulas-whats-changed).

### Roam Research (Attributes + Datalog queries)

- **What users can author**: Free-form `attribute::value` notation on any block — no schema, no type system. Queries are written in Roam's Datalog dialect.
- **Storage format**: Internal database; JSON export possible.
- **Limits**: Schemaless model trades simplicity for queryability — community has long requested attribute-typed queries (still open). 2026 market analysis notes Tana's structured supertags drew away Roam users who found Datalog too complex.
- **Layer**: L1/L2 + L4 plugin layer ("Roam extensions") via JS.

### Logseq (Page properties + Datalog queries + plugins)

- **What users can author**: Page-level and block-level properties (key::value). Datalog advanced queries via Datascript. Plugins in JS or (recent) ClojureScript.
- **Storage format**: Markdown / Org files on disk (file-over-app).
- **Plugin sandbox**: Plugins run in iframe / shadow-DOM, communicate with the main app via message passing.
- **Layer**: L2 declarative + L4 via sandboxed plugins.
- Sources: [Logseq Plugin System — DeepWiki](https://deepwiki.com/logseq/logseq/6.1-plugin-system).

### TheBrain (Thought Types + Tags)

- **What users can author**: Hierarchical Thought Types (classification, like a class hierarchy), each with font color and icon. Thought Tags are additive attributes (many per thought).
- **Storage format**: Proprietary `.brz` brain archive; not file-over-app.
- **Layer**: L3 declarative (hierarchy + visual properties).

### DEVONthink (Custom Metadata)

- **What users can author**: Custom metadata fields (Pro/Server only) — typed fields added in Preferences → Data.
- **Storage format**: SQLite-backed DEVONthink databases; fields can be referenced in smart rules and AppleScript.
- **Layer**: L2 declarative.

### Capacities (Object Types)

- **What users can author**: Custom Object Types with custom Properties (text, number, date, URL, select, "Object select" = typed relation as of July 2025). Templates per type. Two-way property linking.
- **Storage format**: Capacities' hosted DB; export to Markdown is possible.
- **Layer**: L3 declarative.

### AppFlowy (Custom Field Types + databases)

- **What users can author**: Databases with Fields. Custom property types can be added by developers via the `FieldType` + `TypeOption` Rust traits — the type system is open-ended at the source-code level, not the end-user level.
- **Storage format**: Rust backend with derive macros; collab-rs CRDT format.
- **Layer**: L2 declarative for end users; L4 source-level extensibility for developers.

### TiddlyWiki (Tiddler fields + types + plugins)

- **What users can author**: Each tiddler is a name:value bag — field names can be any characters. A `type` field gives the MIME content type. Filters are a DSL for selecting/transforming tiddler sets. Plugins are themselves tiddlers (or bundles of them) and can include JavaScript modules.
- **Storage format**: Everything is a tiddler — including plugins. Filters in TiddlyWiki's bracket-based syntax. Tiddler files can be exported as individual `.tid` text files.
- **Plugin sandbox**: None — JS modules run in the same context.
- **Layer**: L2 declarative (freeform fields) + L2.5 DSL (filter language) + L4 JS plugin.

### Org-roam (Capture templates + DB schema)

- Capture templates are defined in Emacs Lisp, with `:target`, prefilled text, and user-chosen properties. The node DB schema is in `org-roam-db--table-schemata`.
- User authoring requires writing elisp — full L4 code authoring is the entry point, with no L2/L3 declarative layer for end-users.

---

## §3 Academic / Standards Prior Art

### Topic Maps (ISO/IEC 13250)

- Three primitives: **topics, associations, occurrences**. All three are typeable; the user-defined set of types forms the "ontology" of the topic map. XTM XML interchange syntax + CTM compact textual syntax.
- **Adoption**: Originated late 1990s in back-of-the-book indexing. Alive in library/archives/government metadata circles; never reached consumer PKM.
- Sources: [Topic Maps — Wikipedia](https://en.wikipedia.org/wiki/Topic_map), [The TAO of Topic Maps](https://www.ontopia.net/topicmaps/materials/tao.html).

### RDF Schema + OWL

- User-definable classes, properties, restrictions, inferences. Maximally expressive.
- **Why it didn't reach consumer PKM**: OWL "requires reasoners and ontology editors, which add complexity"; tooling disparity — RDF has wide triple-store / SPARQL support but OWL tooling is sparser.
- **Where it's alive**: linked-data publishing, biomedical ontologies (MeSH, Gene Ontology), library catalogs, government open data.
- **Where it's dead**: end-user PKM. No consumer note app uses OWL as its schema authoring layer.

### Microformats and Schema.org

- **Microformats** = community-curated grassroots vocabularies; a vocabulary becomes "a Microformat" only after community process.
- **Schema.org** = three-vendor (Google/Microsoft/Yahoo) Microdata vocabulary set; very large coverage but vendor-controlled.
- **Both allow user-defined vocabularies on top** via the RDFa/Microdata substrate. Schema.org has an explicit extension mechanism.
- **Lesson for Constellation**: a curated baseline + extension mechanism is the dominant pattern when there's a curating body.

### Dublin Core Application Profiles (DCAP)

- **Pattern**: An "application profile" is a declarative specification of which terms a community uses, with added constraints, drawn from multiple metadata vocabularies.
- **Lesson**: when a curating body wants to allow domain extension without forking the core standard, they ship a "profile" — a declarative document constraining and extending the core terms. This is essentially L2 declarative authoring with a formal community-review path.

### JSON Schema

- Declarative validation for JSON structures, with mature 2020-12 draft features. Heavily used as a substrate by other systems (TypeBox, Zod, AppFlowy field types, Anytype auto-generated schemas).
- **Where it works as a user-authoring substrate**: when wrapped by a UI (e.g. JSON-Schema-driven form builders like AEM Adaptive Forms, RJSF, JSON-Forms). End users don't author raw JSON Schema — they author through a UI that emits/validates against it.
- **Where it stops**: It validates structure, not behavior. Geometry functions, layout computation, formulas — these need a separate layer.

### Stencila / Quarto extensions (academic content extensibility)

- Quarto extensions add formats and filters. Filters are **Lua programs** that walk the Pandoc AST. Formats are YAML manifests under `_extension.yml` that may bundle filters, shortcodes, templates, and stylesheets.
- **Lesson**: academic publishing tooling treats "declarative manifest + Lua code module" as the natural extension shape. The manifest is the discoverable, portable, human-readable part; the Lua is for arbitrary transforms.

---

## §4 Plugin Systems & Sandboxing

### VS Code extensions — process isolation

- Each extension runs in a separate **extension host** Node.js process; recent (2022+) migration moves extension host into a utility process spawned by the main process, communicating with the renderer via message ports. The extension host uses a restricted official VS Code API.
- **Property**: high-performance, full Node.js capability, isolated **process** boundary (no syscall sandbox — extensions can spawn child processes and reach the filesystem). This is "isolation for stability" rather than "isolation for security."

### Obsidian community plugins — minimal sandbox, social trust

- Restricted Mode default-off + community-review on submission. Once enabled, plugins inherit **full access** of the host (filesystem, network, shell via Shell Commands plugin).
- **Property**: maximum power, minimum sandbox; security model = community review + user consent + post-publication governance.
- Sources: [Plugin security — Obsidian Help](https://help.obsidian.md/plugin-security), [Obsidian Shell Commands Abuse — Penligent](https://www.penligent.ai/hackinglabs/obsidian-shell-commands-abuse-shows-a-new-malware-playbook/).

### Logseq plugins — iframe / shadow-DOM sandbox + message passing

- Plugins run in iframe or shadow-DOM, cannot directly touch Logseq state, communicate via message passing. Plugin install requires developer mode acknowledgment.
- **Property**: structural isolation in the browser engine sense, not a syscall sandbox.

### Figma plugins — QuickJS WebAssembly sandbox

- Plugin code runs in a **QuickJS** instance compiled to Wasm — a minimal JS environment without browser APIs. UI runs in a separate iframe with full browser APIs. The two communicate by message passing.
- **History**: Figma originally used Realms shim; switched to QuickJS+Wasm after sandbox-escape vulnerabilities were found.
- **Property**: this is the **strongest in-process sandbox** of the consumer-facing plugin systems surveyed. The plugin literally cannot see the browser DOM, fetch, etc., unless host explicitly proxies it.
- Sources: [How Plugins Run — Figma Dev Docs](https://developers.figma.com/docs/plugins/how-plugins-run/), [How we built the Figma plugin system](https://www.figma.com/blog/how-we-built-the-figma-plugin-system/).

### Wasm-based plugin systems (Extism / Wasmer / Wasmtime)

- **Extism**: cross-language Wasm plugin framework — host writes the API surface, plugin authors write Rust/Go/C/JS/Python compiled to Wasm. Each plugin runs in a Wasmtime (default) or Wasmer instance; host functions are explicitly declared.
- **Security model**: capability-oriented — the host explicitly grants memory, fuel limits, and the specific host functions the plugin can call. Plugin **cannot** make syscalls except through declared WASI imports.
- **Property**: the strongest sandbox of all the plugin patterns. The trade-offs: serialization across the Wasm boundary, and language ecosystem constraints.
- **Used in production**: moonrepo's plugin system, Dylibso, growing in 2025.

### QuickJS / Duktape embedded directly in Rust

- **quickjs-rs**, **quickjs-rusty**, **duktape-rs** are Rust wrappers for embedded JS engines. QuickJS supports full ES2020; Duktape is ES5-ish but extremely portable.
- **Sandboxing**: QuickJS itself has no syscall capability — it's pure JS. But it has no built-in memory/fuel limits either; embedders set them.
- **Use case fit**: ideal for trusted-but-isolated user scripts. Not ideal for community plugin distribution at scale.

### Lua plugin systems — Neovim, Hammerspoon

- **Neovim**: Lua 5.1 / LuaJIT permanent interface. **No formal sandbox** — Lua plugins have full process power.
- **Hammerspoon**: Lua bridge to macOS APIs. Plugins are "Spoons" — pure Lua or Lua + Objective-C hybrids. **No sandbox** — full system access by design.
- **Property**: Lua has small binary, fast embed, low-latency host calls. But there is no isolation; Lua plugin authoring is trusted-user territory.

---

## §5 The Four Approaches for Constellation

| Approach | Mature precedent | Eng cost | UX cost | Security | Geometry coverage |
|---|---|---|---|---|---|
| **A1. Pure declarative JSON/YAML** | Obsidian Bases (.base YAML), Anytype types (Any-Block JSON), Tana JSON schema export, Notion property defs, Capacities object types, DCMI Application Profiles, schema.org extensions | **Low** — define JSON Schema, build a UI to edit it, store on disk | **Low** — point-and-click in a settings panel; advanced users can hand-edit the file | **Highest** — no code execution, only data | Covers angles, sectors, rings, bands, named regions, gradients-by-formula; does NOT cover Polanyi-style arbitrary continuous fog or anything requiring custom math beyond a fixed vocabulary |
| **A2. Bounded DSL** | TiddlyWiki filter language, Notion formulas, CSS Grid syntax, Quarto YAML manifest, Mermaid diagram DSL | **Medium** — design grammar, write parser+validator+UI, debugger surface, error messages | **Medium-low** — users learn a small language; better than code, worse than UI | **High** — DSL is bounded by definition; risk only if DSL grows undisciplined | Covers more geometries than A1 if DSL is rich (e.g. coordinate expressions); designing the right grammar is the hardest part |
| **A3. Sandboxed code (Wasm / QuickJS / Lua)** | Figma (QuickJS+Wasm), Logseq (iframe), Extism (Wasm cross-lang), Quarto (Lua), Neovim (Lua, no sandbox) | **High** — embed runtime, design host API, marshal types across boundary, fuel/memory limits | **High** — user must write code, learn the host API | **Medium-high (Wasm/QuickJS) to Low (Lua, no sandbox)** — depends entirely on which substrate | Covers arbitrary geometry — register author writes `remapStarPosition` directly |
| **A4. TypeScript plugin (full trust)** | Obsidian community plugins, TiddlyWiki JS modules, Org-roam elisp, Hammerspoon Spoons, Roam extensions | **Medium** — plugin loader, manifest spec, plugin store / distribution | **High** — requires JS/TS literacy | **Low** — same trust level as Obsidian today (full filesystem, network, etc.); social trust + community review = the security model | Maximum — register is just code |

**Engineering-cost ordering** (lowest to highest for Constellation specifically, given Tauri+Svelte5+Rust stack): A1 < A4 ≈ A2 < A3. A4 is lower than A3 because the Svelte/TS frontend already loads TS modules; the lift is a dynamic-import contract, not a runtime embed.

**UX-cost ordering** (lowest barrier to highest): A1 < A2 < A4 ≈ A3. Once you're asking the user to write code, the difference between TS-with-types and Lua-without-types is small relative to the gap between "fill a form" and "write code at all."

**Geometry coverage**: only A3 and A4 cover *arbitrary* geometry. A1 covers fixed parameterized shapes (sectors, rings, gradients-by-known-formula). A2 covers as much as the DSL admits.

---

## §6 Hybrid Approaches in Mature Systems

Yes — **declarative baseline + plugin/code extension is the dominant hybrid pattern** in mature PKM and editor tooling:

| System | Declarative layer | Code layer | How they coexist |
|---|---|---|---|
| **Obsidian** | YAML Properties + `.base` files | Community plugins (full-trust TS) | Bases is the in-core declarative view layer; plugins add behavior. |
| **VS Code** | `package.json` extension manifest with declarative contributions (commands, languages, themes, settings schemas) | Activation handlers in TS/JS | Manifest declares *what* the extension contributes; code activates *when needed*. Declarative contributions don't require activation — they're indexed at install. |
| **Quarto** | `_extension.yml` (formats, filters list, shortcodes) | Lua filters | Manifest is portable + reviewable; Lua handles the AST transforms the manifest can't express. |
| **Microsoft 365 Copilot agents** | Declarative JSON manifest defines agent + capability boundaries | Optional API plugins for actions | "Declarative agents don't have any code … configuration happening through JSON-based files. Actions are optional JSON objects … and can be considered as plugins." |
| **TiddlyWiki** | Tiddlers with fields (free-form name:value) + filter language | Plugin tiddlers can carry JS modules | Plugins *are* tiddlers — same storage model, plus the option to ship code. |
| **Schema.org / Microformats** | Curated vocabulary + an *extension mechanism* (subclass a Type, mint new properties via RDFa) | (n/a — markup standards) | Closest standards-world analog: curated baseline, well-defined hook for going further. |
| **DCMI Application Profiles** | Declarative profile document selecting and constraining vocabularies | (no code; community/governance process) | Pattern is "central vocabulary, profile-level extension." |

**What works in these hybrids**:

1. The declarative layer is **portable, version-controlled, peer-reviewable, AI-readable** — exactly the file-over-app properties. Even non-plugin users get the curated baseline + their own declarative customizations.
2. The code layer is **opt-in for the minority who need it**. Most users never touch it.
3. The two layers are **discoverable from the same place** (one manifest references the optional code module). Constellation's chip can list both kinds of registers with no UI distinction.

**What doesn't work**:

1. When the declarative layer is *implicit* (TiddlyWiki's free-form fields, Roam's free-form attributes) you get drift — no two users mean the same thing by `author`. Obsidian users in 2025 are actively writing "master schema notes" by hand to fight this. **Lesson**: a declarative authoring layer benefits from a real schema (types, constraints), not just key:value freedom.
2. When the code layer has **no sandbox** (Obsidian, TiddlyWiki, Org-roam, Neovim), you get the supply-chain-attack surface the 2025 BigGo and Penligent write-ups call out. The community pattern is "default-off + manual enable + post-hoc review" — workable but lossy.
3. When **only** a code layer exists (Org-roam — capture templates require elisp; Hammerspoon — every customization is Lua), adoption is limited to technical users. Tana drew users away from Roam in part by *adding a structured-yet-declarative* layer on top of Roam's schemaless attributes.

**Verified open question for Constellation**: among the systems surveyed, **none** clearly combines "declarative-baseline-with-real-schema" + "Wasm-sandboxed plugin layer for behavior" + "file-over-app schema-on-disk" in the consumer-PKM space. Figma has the strongest sandbox but is not file-over-app; Anytype is local-first and has a structured type system but no in-process plugin layer (only external gRPC); Obsidian has plugins on disk and Bases on disk but no plugin sandbox. The intersection Constellation is considering is **novel ground in consumer PKM**, not a pattern with established precedent.

---

## §7 Honest Unknowns

Things I could **not** verify and would need to prototype or read source to confirm before committing:

1. **Anytype "Collections 2.0"** — the Feb 2026 community update describes a shift from one-type-per-object to relation-based organization. Whether this has shipped or is still in design is unverified.
2. **Tana's "Show API Schema" output shape** — multiple sources say it exists; none of the public docs surveyed showed a full example of the emitted JSON Schema.
3. **Anytype's user-authored Type on-disk format** — Any-Block is the documented protocol (Protobuf + auto-generated JSON Schemas), but I did not find confirmation that a user can author a Type by writing a JSON file on disk (vs. only via the GUI).
4. **Figma QuickJS startup latency** — Figma cites QuickJS for security; I did not measure how much it costs vs. native JS. For Constellation's "user adds a register" UX, a >100ms cold-start per register would be visible.
5. **Extism's binary footprint when embedded in Tauri** — public sources cite Extism is "lightweight" but I did not find a specific size figure for a Tauri-bundled Wasmtime + Extism runtime. Could be a meaningful add to Constellation's installer.
6. **Logseq's recent ClojureScript plugin SDK** — mentioned in 2025 changelogs; I did not verify sandboxing parity with the JS plugin layer.
7. **Whether any of the surveyed systems lets a user-authored "type" carry a *geometry/layout* function (not just fields + view configuration)** — I found no system whose user-authored schema includes a function that computes a 2D position from a record. Notion/Anytype/Tana let users pick a *built-in layout* (table, board, calendar, gallery) but not author the layout's coordinate function. Figma plugins can draw arbitrary geometry but plugins are not "schemas" in the PKM sense. **This is genuinely new territory.** A Constellation register's `remapStarPosition` has no direct mature precedent in the PKM tradition — the closest is "Quarto custom format that ships its own template," and even that's static styling, not per-record geometry computation.
8. **Whether bundling QuickJS, Lua, or Wasm into Tauri's WebView2/WKWebView is materially different on Windows vs macOS vs Linux** for sandbox guarantees. Would need a per-platform spike.
9. **The performance cost of evaluating an A3 (sandboxed code) `remapStarPosition` per star on a 7,600-note Universe** — this is purely a measurement question. A QuickJS call from Rust through Tauri is on the order of microseconds per call; 7,600 calls is ~10–100ms in the cheap case, potentially much worse with bridge marshaling. Needs benchmarking before betting the dome's frame rate on it.

---

**End of report.**
