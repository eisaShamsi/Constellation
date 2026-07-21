# MIG-103 R1 — Standards and Case Studies for "Save as Template"

**Date:** 2026-07-21 · **Status:** Research synthesis, verified — five tracks (A standards, B shipped products, C galleries, D repo/plugin architecture, E Arabic book structure), every claim carries its evidence grade `[sourced]` / `[recalled]` / `[unknown]`, and every claim that went through the adversarial verify pass carries its verdict. Corrections from verification are folded in — where the verify pass refuted something, the refuted version is struck here and recorded in §6.

**The question (verbatim):** *"R1: What are the standards? Is there a case study we can follow?"* — about what Save-as-Template should keep vs strip.

**Standing rulings this paper serves:** R2 Studio = core plugin / app-within-app · R3 Arabic defaults from old and new manuscripts · R4 Studio gets its own independent style setter · R5 templates live in a visual library.

---

## 1. The direct answer to R1

### 1.1 The standards that exist — and what they actually say

There **are** two international file-format standards for templates, and they agree with each other completely:

- **OOXML (Word's `.dotx` vs `.docx`)** — a template is *structurally the same file* as a document: same ZIP package, full content, full styles, full metadata (the official registration even warns it "may contain personal information"). The ONLY difference is an internal type label. Converting a document to a template is literally a one-line re-label (`ChangeDocumentType`) — nothing is removed. `[sourced — IANA registration + Open XML SDK; verify: CONFIRMED]`
- **ODF (LibreOffice's `.ott` vs `.odt`)** — identical story, verified directly against the OASIS 1.3 spec text: one conformance class, distinguished **only** by the MIME string in the package's `mimetype` file. The spec's Packages part contains *zero* occurrences of "template" — package structure and metadata rules are the same for both. `[sourced — OASIS ODF 1.3 Part 3 §2.2.3, Part 2; verify: CONFIRMED]`

**Conclusion 1: No standard anywhere defines "template = structure only."** In every spec examined (OOXML, ODF) and every vendor flow examined (Word, LibreOffice, Google Docs, Apple Pages), a template is a **full document whose type marker changes what happens when you open it** — it opens as a fresh copy instead of opening itself. `[sourced — synthesis]`

Two more things the standards give us for free:

- **Placeholder standards exist — for content.** Word's content controls (`w:sdt` + placeholder prompt text), PowerPoint's placeholder shapes ("Click to add…"), and ODF's `<text:placeholder>` all model a template slot as **structure present + value empty + prompt shown**. That is exactly Constellation's "keep the key, blank the value" idea — but the standards apply it to *content slots*, never to metadata properties. Applying it to frontmatter is our **extension**, honestly labeled as such: no standard forbids it; none mandates it. `[sourced]`
- **Provenance backlinks are standardized.** ODF's `<meta:template>` element and OOXML's `w:attachedTemplate` both record, in the *instantiated* document, a link back to the template it came from. Both optional, both cheap. `[sourced — OASIS v1.2 Part 1 §4.3.2.12; ISO/IEC 29500-1 §17.15.1.60; verify: CONFIRMED]`
- **The only "scrubbing" the office world knows is manual and opt-in.** Word's Document Inspector strips author/personal metadata *on demand*; the documented Save-as-Template flow never invokes it. Silent stripping has no precedent — and would also violate Constellation's own "never modify file content silently" principle. `[sourced; the "never auto-invokes" reading is absence-of-evidence, flagged as such]`

### 1.2 The dominant shipped pattern — the product count

Eleven products were examined. The score:

| Pattern | Count | Products |
|---|---|---|
| **Keep everything — full snapshot; the author curates the template by editing it** | **8** | Evernote, Word, OneNote, Craft, Coda, Obsidian, Anytype, Figma |
| Full copy **minus/transformed temporal data** (dates, completion state) | 2–3 | Trello (strips dates + completion), Asana (relative-shifts dates, role-abstracts assignees), Notion (per-field restamp option) |
| **Structure-only extraction** | 1 | Airtable — and only as an **opt-in toggle** ("duplicate base without records"), in a *database* product |

`[sourced — Track B aggregation; each product individually graded; the four spot-verified claims (Evernote flag-in-place, Evernote date complaint, Notion authored-not-converted, Word full-snapshot) all CONFIRMED]`

**Conclusion 2: No notes product on the market auto-blanks values or strips structure at save-as-template.** The one field class products *do* actively handle is **dates** — Trello strips them, Asana relativizes them, Notion offers restamp-on-duplicate — because instance-temporal data is the one thing that is provably wrong to inherit (see §2).

### 1.3 The case study to follow

**Primary case study: Evernote's current (2024+) "Save as Template."** `[sourced; verify: CONFIRMED]`

It is chosen because it is the *only* shipped flow that is the exact same gesture Constellation is building — **an existing note in a notes product becomes a template** — and its behavior is fully attested:

- It does **not** copy-and-strip. It **flags the existing note as a template** via metadata; the note stays where it is.
- Body, formatting, tables, checklists, attachments, tags, and tasks all carry into notes created from it.
- Templates are **edited exactly like notes** — curation is the author's job, done with the normal editor, not a special stripping dialog.

And it comes with the single most valuable lesson in the whole corpus — **the one documented failure**: notes created from an Evernote template inherit the **template's creation date**, and users on Evernote's own forums call it out in at least four threads ("CRITICAL — Applying new templates changes creation date"; "That's a real problem"). The keep-everything model fails at exactly one point: **identity and time must be re-stamped at instantiation.** `[sourced; verify: CONFIRMED — multiply attested]`

Two supporting case studies frame the edges:

- **Trello** — the precedent for *which* class of field a convert-to-template flow may legitimately strip: start/due dates and completion status — instance-temporal data, nothing else. `[sourced]`
- **Airtable** — the precedent for structure-only extraction: it exists in the wild only as an **explicit, opt-in, clearly-labeled secondary option**, never a silent default. `[sourced — community-attested, not read on an official docs page]`

Counter-model worth knowing: **Notion** sidesteps keep-vs-strip entirely by making templates *authored from scratch* — and the cost is a whole third-party workaround ecosystem for users who want note→template, the exact gesture Constellation is adding. `[sourced; verify: CONFIRMED]` **Obsidian** — Constellation's file-over-app neighbour — has *no* conversion gesture at all: a template is just a note in a folder, with zero keep/strip machinery. A real gap Constellation fills. `[sourced]`

### 1.4 Recommendation for Constellation

1. **Default = the standards model + the Evernote gesture.** "Save as Template" keeps the **full note** — body, structure, frontmatter keys — and flips the **type marker** (Constellation already has the `TMPL` file kind in `file_kinds.rs`; templateness is a re-type, not an amputation — the `.dotx` precedent exactly). The author then curates the template by editing it like any note, replacing instance text with placeholder slots where wanted (the `w:sdt`/`text:placeholder` precedent).
2. **Strip or transform exactly one class automatically: instance-temporal identity** — and even that mostly at *instantiation*, not at save (see §2). If anything is touched at save time, the defensible set per shipped precedent is dates + traversal/completion-like state (the Trello class), and it should be **visible, not silent**.
3. **Offer "strip personal metadata" as an explicit option, never a default** — the Document Inspector precedent, and required anyway by "never modify file content silently."
4. **Two menu items: yes.** *"Save as Template"* (full snapshot — the default, the 8-of-11 pattern) and *"Save as Template (structure only)"* (keys kept, values blanked, body reduced to headings/slots — the Airtable opt-in model, and our honest extension of the placeholder standards to frontmatter). The second item is where Constellation *differentiates*; the first is where it *follows*. Neither is silent about what it does.

---

## 2. What the case studies teach about instantiation

Every standard and every product puts the identity question at **new-from-template**, not at save-as-template:

- **Word/LibreOffice/Google/Pages:** instantiation always yields an **untitled, never-saved copy** ("A template is a document type that creates a copy of itself when you open it"; LibreOffice: "always named Untitled N"; Google: "A copy of the template opens"). Editing the template itself requires a deliberate separate action. `[sourced; verify: CONFIRMED]`
- **The specs are silent on timestamp re-stamping at instantiation** — that is application behavior, not file-format law — which means Constellation is free *and obliged* to define its own rule. `[recalled/unknown for Word's internal behavior — honestly flagged]`
- **The Evernote failure** shows what happens when the rule is skipped: inherited creation dates, angry users. `[sourced; CONFIRMED]`
- **Fresh identity is the unstated universal:** every product creates a new object at instantiation; none documents ID semantics explicitly. `[recalled — implied by every flow read]`

**Mapping onto Constellation:**

| Field | Rule at new-from-template |
|---|---|
| `cid_cn` / filename identity (`YYYYMMDDTHHMMSSZ_NOTE_XXXX.md`) | **Minted fresh** at instantiation — never inherited from the template file |
| `created` | **Re-stamped** to the instantiation moment (the anti-Evernote rule) |
| `title` | Fresh/prompted — the note opens as a new, unsaved-until-named thing |
| Everything else (body, structure, frontmatter keys, tags) | **Copied wholesale** from the template |
| Born-from provenance | **One new frontmatter key** recording the source template — the `<meta:template>` / `w:attachedTemplate` precedent, standardized in both formats, costs nothing, and gives the Studio "which notes came from this mold" for free |

`[design consequence — grounded in sourced findings above]`

---

## 3. R5 — the visual library

### 3.1 The four preview mechanisms, and how each one fails

`[sourced — Track C aggregation; leg (c)'s Canva component is recalled-grade]`

| Mechanism | Who ships it | Failure mode |
|---|---|---|
| **(a) Auto-render default** | Figma (first page of the file, author can override with any frame) `[CONFIRMED]` | Always current; can render an unrepresentative view of large content |
| **(b) Save-time stored thumbnail** | OOXML `docProps/thumbnail.emf` behind the Save-Thumbnail checkbox; ODF **spec-mandated** `Thumbnails/thumbnail.png` (256×256, PNG; forbidden for encrypted docs) `[PARTLY_CONFIRMED / CONFIRMED — see §6]` | Current as of last save — but **opt-in in Word, so usually absent**; most Word templates show a generic icon |
| **(c) Author-designated region** | Figma frame, Miro "preview area", Canva first page `[Miro's stale-preview bug: sourced, fetched]` | **Desyncs on edit** — Miro users document losing the preview after every template edit |
| **(d) Hand-uploaded artwork** | FigJam publish dialog, Notion gallery (editorial screenshots), Typora themes `[CONFIRMED for FigJam]` | Prettiest and the **most reliably stale** — nothing ties the image to the template's content |

### 3.2 Does a STRUCTURE preview have precedent?

**Yes — one, and it is huge: PowerPoint's slide-layout gallery.** Every layout is previewed as an arrangement of dotted-line *placeholder boxes* — the mold, not a cast — and Microsoft's own doc labels it exactly that way ("showing the placement of various placeholders for text or graphics"). Shipped to a billion users for two decades. `[sourced — fetched]`

**And none in the Markdown world.** No Markdown tool found presents templates as a visual gallery at all, and none previews structure: Obsidian plugins show first-lines text snippets, Zettlr templates are picked from a text autocomplete, Typora uses hand-shot screenshots. (Absence can't be proven, but the searches were thorough — see §6 caveats.) `[sourced for what was found; absence honestly flagged]`

Provenance banding also has precedent everywhere: Office's Featured/Personal tabs, Miro's Personal/Team/Company, FigJam's org-named tab, Pages' "My Templates." The three-bands ruling (your molds / noticed / on offer) extends an established pattern — with the fix that Constellation's bands should *always* be present (Office's Personal tab vanishes if a folder path isn't configured — a documented fragility). `[sourced]`

### 3.3 Recommendation for Markdown templates

- **Preview = a skeleton render derived from the template itself** — headings + frontmatter field slots drawn as the card. It is mechanism (a) done cheaply (parse, don't fully render), deterministic, theme-aware, and **structurally immune to the stale-thumbnail bug class** — there is no stored image to desync.
- **Regenerate it in the same transaction that writes the template** — this is Rule 8 (write-time derivation) applied to previews, and it is exactly what the ODF spec formalizes (embedded preview refreshed at save). Make it **always-on**, fixing Word's opt-in failure mode.
- **RTL for free:** a text-shaped skeleton mirrors natively under `dir` (per-mold `detectDir()`); a bitmap thumbnail does not. No product-specific RTL gallery evidence exists anywhere `[could not establish]` — Constellation would be first here too.
- **Hand artwork is defensible only for a future curated "on offer" band** (the Notion-gallery end of the spectrum), never for personal molds.
- Privacy carve-out from the ODF spec: **no content previews for protected material** — bank the rule now for any future encrypted-note feature.

---

## 4. R2 — app-within-app, grounded in the repo

### 4.1 What "core plugin" already means — externally and in this codebase

External precedent, both verified verbatim: **Obsidian** — core plugins are "officially built and supported… included within the application," toggled under Settings → Core plugins, some off by default `[CONFIRMED]`. **VS Code** — "many core features of VS Code are built as extensions and use the same Extension API," shipping from the in-repo `extensions/` folder `[CONFIRMED]`. The shared discipline: **in-box, toggleable, and no private back-doors** — a built-in surface consumes the same APIs as everything else.

Constellation **already has this architecture and already uses the term** — "Core Plug-in" appears 8 times across 5 files: `[sourced; verify: CONFIRMED, with count correction — 25 toggles, not ~22]`

- `src/lib/libraries/store.ts:5031-5073` — the `enabledFeatures` block ("// Built-in features," 25 boolean toggles); `cece` commented as "MIG-039: CECE ('The Cataloger') left-dock Core Plug-in" (:5066-5069), `ccs` likewise (:5070-5072); the loader even accepts a legacy `enabledPlugins` key (:5539).
- `src/lib/components/SettingsModal.svelte:225-259` — the plugin catalog rows `{id, name, desc, icon}`; toggle machinery at :497-508.
- `src/routes/+layout.svelte` — the five-wire mount recipe, proven ~15 times over: `showX` boolean (:633 for Reviewer), membership in `fullPageActive` (:1358), mutual-exclusion `$effect` (:1367-1373), dock button clearing peers (:6781-6783), full-page overlay (:7578-7608).
- `src/lib/sight/engine.ts:146` + `store.ts:5528-5539` — the MIG-038 Wings pattern: disable = flag off, **code stays on disk, reversible**. There is no plugin registry object; the `enabledFeatures` record *is* the plugin boundary.

### 4.2 The smallest honest Template Studio architecture

One new boolean plus proven wiring — **nothing needs inventing**:

1. `enabledFeatures.templateStudio` (one line in the Built-in features block).
2. One catalog row in SettingsModal (`id: 'templateStudio'`), optionally + a `templateStudio` config namespace distinct from the toggle (the cece precedent, `store.ts:5066-5069`).
3. `showTemplateStudio` in `+layout.svelte`, joined to `fullPageActive` and the mutual-exclusion guard; one dock button; one overlay mount.
4. Optional dev kill-switch const during bring-up (the `SIGHT_V6_ENABLED` pattern), retired at ship.

### 4.3 R4 — the "independent style setter," without re-opening BUG-015

The Style Setter is a **singleton** whose Apply path ends at ONE `$effect` — commented in the code itself as "the SINGLE writer of body CSS vars (the BUG-015 guard)" (`+layout.svelte:2072-2090`; persistence via `mergeStyleOverride` → `appSettings.styleOverride`, `store.ts:5781-5782`). Every prior plugin got its **own category inside the one Setter** — sky, cns, calendar, org, cataloger (`StyleSetter.svelte:699-717`) — never a second instance. `[sourced]`

So R4's honest reading: **a "studio" category with Studio-namespaced variables (`--studio-*`), written through the existing single-writer path, deep-linked from inside the Studio via `openStyleSetterToCategory('studio')`** (the exact idiom OrgChart uses at `+layout.svelte:5942`). From inside the Studio it *feels* independent — its own entry point, its own elements, its own preview surface — while architecturally there is still exactly one draft/apply engine and one body-var writer. A literal second Setter instance would duplicate the draft engine and add a second writer — **the BUG-015 class**, forbidden.

**Open item before building R4:** whether `styleOverride` is per-Universe or global was not traced (`could_not_establish` D-4) — verify before persisting Studio styles.

---

## 5. R3 — the Arabic skeletons

Both skeletons are drafted from attested secular sources (EI2, Wikisource primary texts, OASIS-grade product of Track E), with the verification pass's corrections **already folded in**. Every element carries its grade.

### 5.1 Skeleton 1 — the Arabic BOOK template (three files, not one with modes)

The classical/modern split is genuine — the variants share almost nothing element-for-element — so ship **three template files**: تراثي، ديوان، حديث.

**■ «الكتاب التراثي» (classical):**

1. **البسملة** `[sourced — EI2: universal opener of the prose preface form, secular included; CONFIRMED]`
2. **خطبة الكتاب** — the attested tripartite form, fixed order: الحمدلة والتصلية → **«أما بعد»** → **سبب التأليف** (the form *guarantees* this: the amma-baʿd section states the real reason for writing) `[sourced; CONFIRMED]` → تسمية الكتاب `[recalled]` → **خطة الكتاب وترتيبه** — the Ibn Khaldun model: name the divisions AND the rationale for their order («فلا جرم انحصر الكلام في هذا الكتاب في ستة فصول» + why bedouin precedes urban) `[sourced — Wikisource, verified verbatim]`
3. **المتن with a division-vocabulary PICKER** — because there is **no universal classical nesting order**, this is the key structural finding, *strengthened* by verification: the Muqaddima's own text interchanges THREE labels (the author says فصول, the printed headings say أبواب, and Bab 1 nests **مقدمات** — not فصول, a verify-pass correction); the Qanun's hierarchy differs *between its own books* (Book 1: فن → تعليم → جملة → فصل; Books 3–4: فن → **مقالة** → فصل; smallest unit فصل throughout — the originally claimed "…فصل → باب, bab smallest" ladder was **REFUTED**: باب is not a structural heading anywhere in the Qanun). Default: **كتاب → باب → فصل** (commonest); picker offers فن، مقالة، تعليم، جملة، مسألة; plus a **flat variant** — one numbered run of titled أبواب, the Kitab Sibawayh shape (570+ chapters, no hierarchy). `[sourced; Qanun correction per §6]`
4. **الخاتمة** (optional) `[recalled — generality across secular works unattested; validation list]`
5. **حرد المتن / الكولوفون** — completion formula (تمّ / تمت) + الاسم + التاريخ + المكان `[sourced for contents; the label حرد المتن itself is recalled — the tradition famously had "no name" for it]`

**■ «الديوان» (poetry):** two attested organizations — (a) default: sections by **حرف الروي** (rhyme letter — the convention systematized by the 10th-c. redactors; rests on limited sourcing, flagged for owner validation) ; (b) by **genre/أغراض** (مدح، رثاء، هجاء، غزل، خمريات، طرديات، زهد) — the Abu Nuwas recension model (al-Suli, Hamza al-Isfahani, Ibrahim al-Tabari), verified, though *not* unique to Abu Nuwas (verify-pass softening: "the most famous genre-arranged diwan, not the only one"). `[PARTLY_CONFIRMED — see §6]`

**■ «الكتاب الحديث» (modern):** صفحة العنوان → الإهداء (اختياري) → الشكر والتقدير → الملخص (عربي + إنجليزي for academic) → **فهرس المحتويات — at the FRONT** `[sourced — attested modern academic norm]` → **المقدمة** (compact: الموضوع، المنهج، الحدود) → **التمهيد** (اختياري — a real Arabic convention with no exact English equivalent: separate from and following the مقدمة, carries background; the template must NOT be a translated English skeleton) `[sourced — four consistent sources; page counts are convention, not regulation]` → **المتن: باب → فصل → مبحث → مطلب** — here the order IS standardized and can be hardcoded, with the two-minimum rule as guidance text (باب ≥2 فصول, etc.) and كتاب/جزء as optional top level `[sourced — Imam Muhammad ibn Saud University guide]`; فصول-only for trade books `[recalled — NOT attested]` → **الخاتمة** (النتائج، التوصيات) `[خاتمة sourced; contents recalled]` → قائمة المصادر والمراجع ↔ الملاحق (mutual order unsettled across guides — ship both, let the user choose) `[sourced]` → الفهارس الفنية (أعلام، أماكن…) at back `[recalled]`

**On the folklore:** the claim that Arabic books "traditionally place the TOC at the back" **could not be established** — Arabic sources on fihris-writing say explicitly front *or* back with no fixed convention; the only attested end-placement national convention found was **French** publishing. Do NOT ship "TOC at back" as heritage. Default front (the attested academic norm); offer back as a *choice*, labeled as a choice. `[sourced absence]`

### 5.2 Skeleton 2 — the Arabic BOOK-NOTE template «قراءة في كتاب»

Not a review form — a **formulation instrument**: identity → classification → the book's own map → the author's stated purpose → the reader's synthesis → typed links.

1. **هوية الكتاب** — العنوان الكامل، المؤلف (+ سنة الوفاة للتراثي)، المحقق/الطبعة/الدار/السنة `[recalled — bibliographic convention]`
2. **التصنيف** — الجنس (أخبار، لغة، ديوان، أدب، فلسفة، طب…) `[sourced — the owner's own genres survey, Media-Containers-Synthesis-v2.md:124]` + the owner's five dimensions (function · content kind · provenance · actionability · maturity) with shipped MATS/CONF vocabularies `[sourced — Note-Shape-and-Template-Studio-Brainstorm.md:55-76, 2026-07-19]`
3. **خريطة الكتاب** — the book's division vocabulary **as the book itself uses it** — justified precisely because vocabulary is attestedly per-work, so recording it is real information `[sourced]`
4. **مقصد المؤلف** — the strongest field: the classical preface form **guarantees** the author states his reason for writing after «أما بعد», so the template can *promise* the reader will find it `[sourced — EI2; CONFIRMED]`
5. **أبرز المسائل والمواضع** — key passages with locators `[recalled convention]`
6. **الخاتمة والمحصول** — the book's conclusion + the reader's synthesis (maps to the Five Acts → Conviction) `[in-house]`
7. **بيانات النسخة** (manuscripts) — الناسخ، تاريخ النسخ، المكان — the attested colophon triple `[sourced]`
8. **الروابط** — typed links (supports / contradicts / derives-from…) into the constellation `[in-house — shipped link types]`

### 5.3 Needs the owner's validation (he reads classical Arabic; he is the validator of record)

1. TOC placement in modern Arabic **trade** (non-academic) books — no dominant convention found.
2. Whether classical manuscripts carried contents lists (fahrasa) and where.
3. The term **حرد المتن** as the colophon label — recalled only.
4. How widespread a formal book-final **خاتمة** is across classical secular works, and its contents.
5. Modern trade-book division practice (فصول-only?) — all attested material is thesis-side.
6. The **rhyme-letter default** for diwans — attested but on limited sourcing; "originally/most commonly" overstates the earliest period.
7. Classical use of **مبحث/مسألة** as division levels in named secular works.
8. Back-matter internal order (ملاحق vs مراجع first) — varies by university guide.
9. تسمية الكتاب as a fixed خطبة element — recalled.
10. فهارس فنية at the back of tahqiq editions — recalled.

---

## 6. Verification record — everything refuted or corrected

Sixteen claims went through the adversarial verify pass. Verdicts: **13 CONFIRMED, 3 PARTLY_CONFIRMED, with one embedded REFUTATION.** The corrections, all folded into the sections above:

| # | Claim | Verdict | Correction |
|---|---|---|---|
| A1 | OOXML template = same package, type-label only | CONFIRMED | Nuance: extension alone isn't sufficient to convert (internal content type is authoritative); Document Inspector *can* strip personal info but is optional user action, not format behavior. |
| A2 | ODF template = MIME-only distinction | CONFIRMED | Nuance: the MIME table (Appendix C) is non-normative; the normative distinction is the §2.2.3 conformance clause + mimetype-file rule — which say exactly what the claim asserts. |
| A3 | Identity at instantiation (untitled copy) | CONFIRMED | Nuance: Word's double-click yields unsaved "Document1" (strengthens the thesis); "right-click > Open" is shell behavior, File > Open is the documented editing path. |
| A4 | Provenance elements (`meta:template`, `attachedTemplate`) | CONFIRMED | Nuances: both OPTIONAL; "fully realized when linkStyles present" was a gloss — attachedTemplate alone links; ODF's is the purer immutable born-from record, OOXML's is a mutable attachment pointer. |
| B1 | Evernote flag-in-place, keep-everything | CONFIRMED | Nuance: official launch straddles Dec 2024–Jan 2025; the internal "metadata" wording rests on a Certified-Expert walkthrough (official help page 403-blocked), but observable behavior is independently corroborated. |
| B2 | Evernote template-date complaint | CONFIRMED | Strengthened: 4+ threads on Evernote's own forum, not just one blog comment. Complaint attested as of early 2025; whether since patched, undetermined. |
| B3 | Notion templates authored, not converted | CONFIRMED | Nuances: relation warning is conditional, not a flat prohibition; "blank if not set" is correct inference, not quoted doc text. |
| B4 | Word Save-as-Template = full snapshot | CONFIRMED | Nuance: "nothing stripped" is supported by absence of stripping language (macros-need-.dotm already carved out), not an affirmative sentence. |
| C1 | Figma thumbnail auto-render + override | CONFIRMED | None. Only frames can be designated. |
| C2 | FigJam manual thumbnail at publish | CONFIRMED | Nuances: FigJam *file covers* do auto-generate (separate feature); "require" slightly stronger than the doc's literal wording. |
| C3 | OOXML `docProps/thumbnail.emf` mechanism | **PARTLY_CONFIRMED** | **Corrections:** the emf path is *Word's* convention, not an OOXML mandate — OPC locates thumbnails by relationship type, and PowerPoint uses `thumbnail.jpeg`; `savePreviewPicture` mandates generation but its *absence doesn't forbid* saving one, and the flag exists only in WordprocessingML; regenerate-on-save is design behavior but some Office builds were unreliable. Design consequence (write-time preview, opt-in failure mode) survives intact. |
| C4 | ODF `Thumbnails/thumbnail.png` spec | CONFIRMED | Nuances: 256×256 sizing is a SHOULD in a non-normative note; generating a preview at all is SHOULD-level — the shalls are conditional (path, PNG, no frames) plus the unconditional encrypted-content prohibition. |
| D1 | Obsidian core-plugin definition | CONFIRMED | Page says "enable" rather than "toggle" — same surface. |
| D2 | VS Code core-features-as-extensions | CONFIRMED | None. |
| D3 | "Core Plug-in" in repo, store.ts refs | CONFIRMED | **Count correction: 25 toggles, not ~22.** Bonus: the term appears 8× across 5 files — a repo-wide convention. |
| D4 | SettingsModal catalog refs | CONFIRMED | Toggle mechanism spans :497-508 (not 498-506) — edges off by 1–2 lines. |
| E1 | Classical tripartite khutbat al-kitab | CONFIRMED | Nuance: it is an "independent literary form," not an "independent genre" (EI2's own distinction). |
| E2 | Ibn Khaldun plan announcement + باب headings | **PARTLY_CONFIRMED** | **Correction: فصول are NOT nested inside EACH باب — Bab 1's subdivisions are labeled مقدمات.** This *strengthens* the vocabulary-looseness consequence: three interchanging labels in one canonical work. |
| E3 | Qanun ladder فن→تعليم→جملة→فصل→**باب** | **PARTLY_CONFIRMED — ladder REFUTED** | **Refuted against the primary Arabic text:** Book 1 = فن→تعليم→(جملة in anatomy)→فصل; Books 3–4 = فن→**مقالة**→فصل; smallest unit is **فصل** throughout; **باب is not a structural heading anywhere in the Qanun** — the "bab-below-fasl, exact reverse of Ibn Khaldun" contrast collapses. Confirmed: five books, ~1025, Books 2/5 as described. Design consequence *survives strengthened* (no universal order — the Qanun differs even between its own books); any UI copy citing the Qanun must use the corrected structure and مقالة must be in the picker. |
| E4 | Diwans rhyme-alphabetical; Abu Nuwas the genre exception | **PARTLY_CONFIRMED** | Abu Nuwas half fully confirmed (three recensions, compilers, genre list). Corrections: "most commonly alphabetical" attested in only one reference — the scheme was *systematized by 10th-c. redactors* and became dominant later, so "originally" overstates; and Abu Nuwas's is "the most famous genre-arranged diwan, not the only one" (al-Suli's Abu Tammam recension also groups by genre). |

**Standing could-not-establish items** (carried honestly, never to be presented as fact): Word's instantiation-time core-props re-stamp behavior; exact ECMA-376 clause numbers read from the ECMA PDF itself; Pages `.template` package internals; Google gallery source-document handling beyond the documented checkbox; Obsidian core plugins' internal API; Blender/DAW app-within-app precedents (not researched — no claims made); `styleOverride` per-Universe vs global scoping (verify before R4 build); all ten items on the §5.3 owner-validation list; and the RTL-gallery evidence gap (§3.3).

---

*Prepared for MIG-103 (Template Studio). Companion in-house sources: `docs/concept-papers/Note-Shape-and-Template-Studio-Brainstorm.md` (2026-07-19, the owner's five-dimension taxonomy) · `docs/concept-papers/Media-Containers-Synthesis-v2.md` (Arabic genres survey) · `src-tauri/src/file_kinds.rs` (the shipped `TMPL` container kind).*
