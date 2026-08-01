# Session Log — 2026-08-01

## §1 — i18n locale parity: the 15 bundles re-synchronised, CLDR plurals repaired, and a guard so it cannot recur

**Function in hand:** the i18n locale files (`src/lib/i18n/*.json` — the 15 bundles behind `$t()` / `$tn()`).

**Concept (the horse):** the interface must speak the user's language *completely* — a locale bundle that silently
falls back to English is a promise the app breaks without ever saying so. The function (the carriage) is the
parity contract that makes the promise checkable.

### The brief vs. what the code actually said

The task arrived with a diagnosis: ~196 keys missing per locale against `ar.json` as the reference, including
`plurals.*` families, with en missing ~94 "notably `plurals.characters.two`, `plurals.links.many`,
`plurals.hidden.two`". Two premises did not survive reading the runtime (`src/lib/i18n/index.ts`):

1. **`en.json` is the SEVERE direction, not the mild one.** `t()` falls back active-locale → en → **raw key**.
   A key missing from a non-en locale renders **English** (degraded but readable). A key missing from **en**
   renders the literal key path (`styleSetter.labels.note_graph`) in **all 15 languages**. en's gap was the
   highest-impact set, not the lowest.

2. **The `plurals.*` "gaps" were not drift — they were correct CLDR.** `plurals.characters.two` exists only in
   ar/he because only Arabic and Hebrew *have* a `two` category; `few`/`many` only in ar/ru. A union-based
   reference set would have forced `two`/`few`/`many` into English, where `Intl.PluralRules('en')` can never
   select them — permanently dead keys, shipped in all 15 files. **A naive union was the wrong instrument for
   this namespace.**

Also corrected: a missing plural category does **not** break `$tn()` at runtime. `resolvePluralForm` falls back
category → `other` → `one`, so it renders the **wrong grammatical form silently**. That is worse than a crash —
it is why the defects below survived since MIG-087.

### Three genuine runtime defects the brief did not mention

Found by checking each locale against `Intl.PluralRules` instead of against the union:

| locale | defect | user-visible effect |
|---|---|---|
| `ru` | **no `other` category at all** (`[one,few,many]`) | fractional counts (1.5) fell back to `one` — "1.5 заметка" |
| `es` `fr` `pt` | no `many` | exact millions took the wrong branch |
| `ar` | no `zero` | n=0 fell through to `other` — "0 ملاحظة" instead of "لا ملاحظات" |

All repaired and verified by simulating the runtime resolver end-to-end (ar 0/1/2/3/11/100 · ru 1/2/5/**1.5** ·
es/fr/pt 1/2/1 000 000 · he 1/2/5 · CJK).

### Dead vs. live — the ~196 was two different piles

Investigation (not inference) split the gap:

- **62 LIVE keys** — `sources.label.*` / `sources.description.*` / `sources.review.*` (37 refs, plus the dynamic
  `$t('sources.description.' + s.source)` at `SourceReviewPanel.svelte:1535`), `classifierScan.*` (12 refs),
  `taxonomyTreePicker.*` (6 refs), `searchBadges.concept` (dynamic, `SearchHub.svelte:93`),
  `styleSetter.labels.*` (dynamic via `L()`/`ssSlug`). **Translated into all 13 missing locales.**
- **~40 DEAD keys** — `sight.v5.*`. `SightV5.svelte` **does not exist on disk**; `SIGHT_V5_ENABLED` was retired
  by MIG-028. `src/lib/sight/engine.ts` states the standing policy that retired-engine key paths are "RETAINED
  as architectural-history record". **Boss-ruled: exempt, don't translate** — translating a deleted engine's UI
  into 13 languages would enshrine ~520 dead strings.
- **3 orphans deleted** — `focusPane.promote` (zero refs, in 13 locales, never in en/ar),
  `actions.newLibrary` + `universe.setup.newLibrary` (ar-only strays; the live keys the code reads are
  `libraries.newLibrary` / `sidebar.newLibrary` / `commands.newLibrary`).
- **Editorial metadata excluded** — any key whose last segment starts with `_` (`_comment`,
  `_translation_note`) is documentation for translators, never rendered; excluded from the contract so a note
  added to one locale doesn't demand 14 fake translations of it.

### What shipped

- **`scripts/i18n-parity.mjs`** (new) — the authoritative diff. Reference set = **union across all 15 minus
  documented exemptions** (union, because the drift ran in *both* directions — neither `en` nor `ar` alone is
  the reference). `plurals.*` checked against `Intl.PluralRules` itself, i.e. the exact engine the runtime uses,
  so the tool cannot disagree with production. `--keys`, `--json`; exit 1 on drift. Wired as
  **`npm run i18n:parity`**.
- **806 translated strings** across 13 locales + **11 keys added to `en.json`** + the plural repairs.
  Native equivalents throughout per the standing order — the epistemology vocabulary uses each tradition's own
  terms (`अर्थापत्ति`, `अनुपलब्धि`, `तवातुर`→`बहुल परंपरा`, `تواتر`, `요청 추론`, `多数伝承`), not transliterated English.
  RTL (ar/he/fa/ur) authored natively.
- **`tests/i18n/locale-parity.test.ts`** (new, 55 tests) — imports the script rather than reimplementing it, so
  `npm test` and the CLI can never disagree. Covers: per-locale missing/extra · CLDR category exactness in
  **both** directions (missing *and* unreachable-dead) · non-empty values · `{count}` discipline ·
  **placeholder preservation vs. the English source** (`{N}`/`{M}` dropped or renamed is otherwise silent) ·
  a **self-test** that injects synthetic drift and asserts the analyser sees it, so green is evidence rather
  than absence · an **exemption-expiry test** that fails if `sight.v5.*` ever leaves disk, so the waiver can't rot.

### Reproduce-First applied to the guard itself

Before trusting it green, the guard was proven RED against real on-disk drift: deleted
`sources.review.title` from `de.json` and added `plurals.notes.few` (a category German cannot select) — both
caught with actionable messages; restoring returned green.

### The guard immediately caught a pre-existing bug — NOT fixed, and why

`styleSetter.labels.an` is `""` in **he/ja/ko**. That is deliberate: those languages have no indefinite article,
and the string is the article in the Style Setter's bold-text sample (`StyleSetter.svelte:1475` renders
`{L('An')} {L('apple')}`). But `L()` treats `''` as a miss (`!v || v === key ? en : v`) and falls back to the
English, so Japanese renders **"An りんご"**.

**No locale-data value can fix this** — any non-empty value renders something, and empty renders English. The fix
is one line in `L()` distinguishing "absent" from "intentionally empty", which the task explicitly scoped out
("do not change any component code"). Preserved the linguistic intent behind a narrow, per-entry allowlist with a
companion test that fails if a waiver goes stale. **Filed as PJ-194** — this is a surfaced-not-buried item per
WA#6, awaiting the one-line ruling.

### Verification (honest)

- **`svelte-check --threshold error` → 0 errors.** (16 errors appeared first, all in the two new files —
  `checkJs: true` type-checks `.mjs`; fixed with JSDoc typedefs, not by exempting the files.)
- **`vitest run` → 854/854 pass** on one run. A second run showed **2 failures in Sight v6 *perf* timing
  assertions** (`perf.test.ts`, `tradition-perf.test.ts`). Proven **pre-existing and flaky**: the full suite on a
  **stashed clean tree** failed **3** assertions in the same family, and two consecutive runs of identical code
  gave 854/854 then 852/854. Unrelated to locale data. (Same family as PJ-172's serial-lane issue.)
- `node scripts/i18n-parity.mjs` → **All 15 locales in parity.**

### One incident worth recording

Mid-session the i18n test suddenly failed to load with a bare `SyntaxError: Invalid or unexpected token` — no
stack. The temptation was to blame the JSDoc edits made just before. Investigation instead of theory
(`No Guessing` law): Node imported the module fine, esbuild transformed both files fine, no BOM, no lone CR.
The actual cause was **CRLF line endings introduced by the `git stash` round-trip** used to test the perf
flakiness — `core.autocrlf` rewrote the working copy, and vite's `.mjs` pipeline choked where Node and esbuild
did not. Normalising the 15 JSON files + 2 new files back to LF fixed it and left the diff byte-identical
(1,139 insertions / 97 deletions). **Lesson: `git stash` on this repo mutates working-tree line endings; verify
line endings after any stash round-trip.**

### SO#2 — help files / User Manual: checked, no change required

The User Manual already asserts *"full multilingual support (15 languages, RTL-native)"* (line 5) and
*"All operators work in 15 languages"* (line 175). This work **makes the existing claim true** rather than
changing behaviour the manual describes; there is no per-panel translation-status section to update. No help
topic documents locale coverage. **Recorded so the check is not silently skipped.**

### SO#9 — PJ ledger reconciled

`docs/Constellation Pending Jobs v1.63.md`. Closed: nothing (this drift was never filed — it is exactly the
completeness gap SO#9 exists to catch). **Filed: PJ-194** (the `L()` empty-string fallback) and **PJ-195** (the
orientation doc is 7,715 lines against SO#6's ~1,500-line split threshold — long-standing, now recorded).

### Files

| file | change |
|---|---|
| `scripts/i18n-parity.mjs` | **new** — parity tool, CLDR-aware, exit-1 on drift |
| `tests/i18n/locale-parity.test.ts` | **new** — 55 tests incl. self-test + exemption expiry |
| `src/lib/i18n/*.json` (×15) | 806 translations + 11 en keys + plural repairs + 3 orphans removed |
| `package.json` | `i18n:parity` script |
| `docs/Constellation Orientation & Onboarding v3.80.md` | **new** — SO#6 |
| `docs/Constellation Pending Jobs v1.63.md` | **new** — SO#9 |
| `docs/MoCh/MoCh-2026-08-01-0920.md` | **new** — SO#7 |

**Gates at close:** vitest **854/854** (73 files; 2 pre-existing perf flakes on repeat runs) ·
svelte-check **0** · i18n parity **15/15 ✓** · Rust untouched.
